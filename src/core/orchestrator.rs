use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::core::config::Config;
use crate::core::integration::Integration;
use crate::core::profile::Profile;
use crate::core::traits::{CommandExecutor, FileSystem, TemplateRenderer};

pub struct Orchestrator<FS, TR, CE> {
    fs: FS,
    template_renderer: TR,
    command_executor: CE,
    config_dir: PathBuf,
}

impl<FS, TR, CE> Orchestrator<FS, TR, CE>
where
    FS: FileSystem,
    TR: TemplateRenderer,
    CE: CommandExecutor,
{
    pub fn new(fs: FS, template_renderer: TR, command_executor: CE, config_dir: PathBuf) -> Self {
        Self {
            fs,
            template_renderer,
            command_executor,
            config_dir,
        }
    }

    /// Primary entry point: Load a profile and apply it to enrolled apps.
    pub fn load_profile(&self, profile_name: &str) -> Result<()> {
        info!("Loading profile: {}", profile_name);

        // 1. Load Main Config (theman.yaml)
        let config = self.load_config()?;

        // 2. Load and Resolve Profile (handling inheritance)
        let resolved_vars = self.resolve_profile_vars(profile_name)?;

        // 3. Iterate Enrolled Apps
        for (app_name, integration) in &config.enroll {
            info!("Processing app: {}", app_name);

            // 4. Merge Overrides
            let app_overrides = config.get_overrides_for(app_name);
            let mut context = resolved_vars.clone();
            context.extend(app_overrides);

            // Add metadata
            context.insert(
                "profile_name".to_string(),
                Value::String(profile_name.to_string()),
            );
            context.insert("app_name".to_string(), Value::String(app_name.to_string()));

            // 5. Execute Integration
            if let Err(e) = self.apply_integration(integration, &context) {
                warn!("Failed to apply integration for {}: {:?}", app_name, e);
                // We continue to the next app instead of crashing
            }
        }

        // 6. Save State (TODO)

        info!("Profile '{}' loaded successfully", profile_name);
        Ok(())
    }

    fn load_config(&self) -> Result<Config> {
        let config_path = self.config_dir.join("theman.yaml");
        let content = self
            .fs
            .read_to_string(&config_path)
            .context("Failed to read theman.yaml")?;

        serde_yaml::from_str(&content).context("Failed to parse theman.yaml")
    }

    fn resolve_profile_vars(&self, profile_name: &str) -> Result<HashMap<String, Value>> {
        let mut visited = HashSet::new();
        self.resolve_profile_vars_inner(profile_name, &mut visited)
    }

    fn resolve_profile_vars_inner(
        &self,
        profile_name: &str,
        visited: &mut HashSet<String>,
    ) -> Result<HashMap<String, Value>> {
        // Cycle detection
        if !visited.insert(profile_name.to_string()) {
            bail!(
                "Circular profile inheritance detected: '{}' appears twice in the inheritance chain",
                profile_name
            );
        }

        let profile = self.load_profile_file(profile_name)?;
        let mut vars = HashMap::new();

        // 1. If extends, load parent first (Recursive)
        if let Some(parent_name) = &profile.extends {
            debug!("Inheriting from parent profile: {}", parent_name);
            let parent_vars = self.resolve_profile_vars_inner(parent_name, visited)?;
            vars.extend(parent_vars);
        }

        // 2. Apply current profile vars
        vars.extend(profile.vars);

        Ok(vars)
    }

    fn load_profile_file(&self, name: &str) -> Result<Profile> {
        let path = self
            .config_dir
            .join("profiles")
            .join(format!("{}.yaml", name));
        let content = self
            .fs
            .read_to_string(&path)
            .with_context(|| format!("Profile not found: {}", name))?;

        serde_yaml::from_str(&content).with_context(|| format!("Failed to parse profile: {}", name))
    }

    fn apply_integration(
        &self,
        integration: &Integration,
        context: &HashMap<String, Value>,
    ) -> Result<()> {
        match integration {
            Integration::Template {
                input,
                output,
                reload_cmd,
                reload_signal,
            } => {
                // 1. Resolve paths (expand ~)
                let input_path = shellexpand::tilde(input);
                let output_path = shellexpand::tilde(output);

                // 2. Render content
                // Check if input is a file
                let content = if self.fs.is_file(Path::new(input_path.as_ref())) {
                    // Use the renderer to read and render
                    // Note: Our trait currently only has render(string), we might need render_from_file
                    // For now, read then render
                    let raw = self.fs.read_to_string(Path::new(input_path.as_ref()))?;
                    self.template_renderer.render(&raw, context)?
                } else {
                    // Assume it's an embedded template path?
                    // For now, strict file path.
                    bail!("Template file not found: {}", input_path);
                };

                // 3. Write output
                self.fs
                    .write_all(Path::new(output_path.as_ref()), &content)?;

                // 4. Reload
                if let Some(cmd) = reload_cmd {
                    self.command_executor.run_command(cmd)?;
                }
                if let Some(signal) = reload_signal {
                    self.send_signal(signal, context)?;
                }
            }

            Integration::Symlink {
                source,
                target,
                reload_cmd,
            } => {
                // Render the source path as a template! (To support {{ mode }}.conf)
                let source_rendered = self.template_renderer.render(source, context)?;
                let source_path = shellexpand::tilde(&source_rendered);
                let target_path = shellexpand::tilde(target);

                self.fs.create_symlink(
                    Path::new(source_path.as_ref()),
                    Path::new(target_path.as_ref()),
                )?;

                if let Some(cmd) = reload_cmd {
                    self.command_executor.run_command(cmd)?;
                }
            }

            Integration::Command { commands } => {
                for cmd_tmpl in commands {
                    // Render the command string (inject vars)
                    let cmd = self.template_renderer.render(cmd_tmpl, context)?;
                    self.command_executor.run_command(&cmd)?;
                }
            }

            Integration::Script { path, args, env } => {
                let script_path = shellexpand::tilde(path);

                // Render Args
                let mut rendered_args = Vec::new();
                for arg in args {
                    rendered_args.push(self.template_renderer.render(arg, context)?);
                }

                // Prepare Env: Prefix everything with THEMAN_
                let mut env_vars = HashMap::new();
                for (k, v) in context {
                    // Flatten Value to String for Env
                    if let Value::String(s) = v {
                        env_vars.insert(format!("THEMAN_{}", k.to_uppercase()), s.clone());
                    } else {
                        // Best effort stringify for numbers/bools
                        env_vars.insert(format!("THEMAN_{}", k.to_uppercase()), v.to_string());
                    }
                }
                // Add explicit user env overrides
                for (k, v) in env {
                    env_vars.insert(k.clone(), v.clone());
                }

                self.command_executor.run_script(
                    Path::new(script_path.as_ref()),
                    &rendered_args,
                    &env_vars,
                )?;
            }
        }
        Ok(())
    }

    /// Send a signal to a process by app name using pkill.
    /// Signal should be like "SIGUSR2", "SIGHUP", etc.
    fn send_signal(&self, signal: &str, context: &HashMap<String, Value>) -> Result<()> {
        let app_name = context
            .get("app_name")
            .and_then(|v| v.as_str())
            .context("app_name not found in context")?;

        // Normalize signal: strip "SIG" prefix if present for pkill compatibility
        let signal_name = signal.strip_prefix("SIG").unwrap_or(signal);
        let cmd = format!("pkill -{} {}", signal_name, app_name);
        debug!("Sending signal: {}", cmd);

        // pkill returns non-zero if no process matched, which isn't necessarily an error
        // (the app might not be running). We log but don't fail.
        if let Err(e) = self.command_executor.run_command(&cmd) {
            debug!(
                "Signal command returned error (process may not be running): {}",
                e
            );
        }
        Ok(())
    }
}
