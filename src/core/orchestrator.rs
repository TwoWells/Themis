//! Core orchestration logic for Themis.
//!
//! The [`Orchestrator`] is the main entry point for loading profiles and
//! applying themes to enrolled applications. It coordinates:
//!
//! - Loading and parsing configuration (`themis.yaml`)
//! - Resolving profile variables with palette inheritance
//! - Applying integrations (templates, symlinks, commands, scripts)
//!
//! # Architecture
//!
//! The orchestrator uses dependency injection for all I/O operations,
//! making it fully testable:
//!
//! ```text
//! Orchestrator<FS, TR, CE>
//!   ├── FS: FileSystem     - Read/write files, create symlinks
//!   ├── TR: TemplateRenderer - Render Jinja2 templates
//!   └── CE: CommandExecutor  - Run shell commands and scripts
//! ```
//!
//! # Example
//!
//! ```no_run
//! use themis::adapters::filesystem::RealFileSystem;
//! use themis::adapters::template::TeraAdapter;
//! use themis::adapters::command::RealCommandExecutor;
//! use themis::core::orchestrator::Orchestrator;
//! use std::path::PathBuf;
//!
//! let orchestrator = Orchestrator::new(
//!     RealFileSystem,
//!     TeraAdapter::new(),
//!     RealCommandExecutor,
//!     PathBuf::from("/home/user/.config/themis"),
//! );
//!
//! // Load a profile and apply to all enrolled apps
//! let result = orchestrator.load_profile("nord").unwrap();
//! if result.is_ok() {
//!     println!("Loaded {} apps", result.success_count());
//! }
//! ```

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

use crate::core::config::Config;
use crate::core::integration::Integration;
use crate::core::profile::Profile;
use crate::core::traits::{CommandExecutor, FileSystem, TemplateRenderer};

/// System-wide data directory (palettes, templates)
pub const SYSTEM_DATA_DIR: &str = "/usr/share/themis";

/// Result of loading a profile, including any failures.
///
/// Even when some apps fail, the orchestrator continues processing
/// the remaining apps. This struct captures both successes and failures
/// so the caller can decide how to handle partial failures.
///
/// # Example
///
/// ```
/// use themis::core::orchestrator::LoadResult;
///
/// // Simulate a result with one failure
/// let result = LoadResult {
///     succeeded: vec!["kitty".to_string(), "waybar".to_string()],
///     failures: vec![],
/// };
///
/// assert!(result.is_ok());
/// assert_eq!(result.success_count(), 2);
/// ```
#[derive(Debug)]
pub struct LoadResult {
    /// Apps that were successfully configured
    pub succeeded: Vec<String>,
    /// Apps that failed with their error messages
    pub failures: Vec<AppFailure>,
}

/// A failure that occurred while applying an integration.
#[derive(Debug)]
pub struct AppFailure {
    /// Name of the app that failed
    pub app_name: String,
    /// Error message describing the failure
    pub error: String,
}

impl LoadResult {
    /// Returns true if all apps were configured successfully.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.failures.is_empty()
    }

    /// Returns the number of apps that failed.
    #[must_use]
    pub const fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Returns the number of apps that succeeded.
    #[must_use]
    pub const fn success_count(&self) -> usize {
        self.succeeded.len()
    }
}

/// The main orchestrator that loads profiles and applies themes.
///
/// Generic over filesystem, template renderer, and command executor
/// to support both real I/O and testing with mocks.
pub struct Orchestrator<FS, TR, CE> {
    fs: FS,
    template_renderer: TR,
    command_executor: CE,
    /// User config directory (~/.config/themis)
    config_dir: PathBuf,
    /// System data directory (/usr/share/themis)
    system_dir: PathBuf,
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
            system_dir: PathBuf::from(SYSTEM_DATA_DIR),
        }
    }

    /// Create orchestrator with custom system dir (useful for testing)
    pub const fn with_system_dir(
        fs: FS,
        template_renderer: TR,
        command_executor: CE,
        config_dir: PathBuf,
        system_dir: PathBuf,
    ) -> Self {
        Self {
            fs,
            template_renderer,
            command_executor,
            config_dir,
            system_dir,
        }
    }

    /// Primary entry point: Load a profile and apply it to enrolled apps.
    ///
    /// Returns a `LoadResult` containing which apps succeeded and which failed.
    /// Early errors (config parsing, profile resolution) still return `Err`.
    pub fn load_profile(&self, profile_name: &str) -> Result<LoadResult> {
        info!("Loading profile: {}", profile_name);

        // 1. Load Main Config (themis.yaml)
        let config = self.load_config()?;

        // 2. Load and Resolve Profile (handling inheritance)
        let resolved_vars = self.resolve_profile_vars(profile_name)?;

        // Collect successes and failures
        let mut succeeded = Vec::new();
        let mut failures = Vec::new();

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
            context.insert("app_name".to_string(), Value::String(app_name.clone()));

            // 5. Execute Integration
            match self.apply_integration(integration, &context) {
                Ok(()) => {
                    succeeded.push(app_name.clone());
                }
                Err(e) => {
                    error!("Failed to apply integration for {}: {:?}", app_name, e);
                    failures.push(AppFailure {
                        app_name: app_name.clone(),
                        error: format!("{e:?}"),
                    });
                }
            }
        }

        // 6. Print Summary
        let result = LoadResult {
            succeeded,
            failures,
        };

        if result.is_ok() {
            info!(
                "Profile '{}' loaded successfully ({} apps)",
                profile_name,
                result.success_count()
            );
        } else {
            error!(
                "Profile '{}' loaded with errors: {}/{} apps failed",
                profile_name,
                result.failure_count(),
                result.failure_count() + result.success_count()
            );
            for failure in &result.failures {
                error!("  - {}: {}", failure.app_name, failure.error);
            }
        }

        Ok(result)
    }

    fn load_config(&self) -> Result<Config> {
        let config_path = self.config_dir.join("themis.yaml");
        let content = self
            .fs
            .read_to_string(&config_path)
            .context("Failed to read themis.yaml")?;

        serde_yaml::from_str(&content).context("Failed to parse themis.yaml")
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
                "Circular include detected: '{profile_name}' appears twice in the inheritance chain"
            );
        }

        let profile = self.load_profile_file(profile_name)?;
        let mut vars = HashMap::new();

        // 1. If include is set, load included palette/profile first (recursive)
        if let Some(included_name) = &profile.include {
            debug!("Including palette/profile: {}", included_name);
            let included_vars = self.resolve_palette_vars_inner(included_name, visited)?;
            vars.extend(included_vars);
        }

        // 2. Apply current profile vars (override included values)
        vars.extend(profile.vars);

        Ok(vars)
    }

    /// Resolve palette variables, searching user palettes then system palettes.
    fn resolve_palette_vars_inner(
        &self,
        palette_name: &str,
        visited: &mut HashSet<String>,
    ) -> Result<HashMap<String, Value>> {
        // Cycle detection
        if !visited.insert(palette_name.to_string()) {
            bail!(
                "Circular include detected: '{palette_name}' appears twice in the inheritance chain"
            );
        }

        let palette = self.load_palette_file(palette_name)?;
        let mut vars = HashMap::new();

        // 1. If palette includes another palette, load it first
        if let Some(included_name) = &palette.include {
            debug!("Palette '{}' includes: {}", palette_name, included_name);
            let included_vars = self.resolve_palette_vars_inner(included_name, visited)?;
            vars.extend(included_vars);
        }

        // 2. Apply palette vars
        vars.extend(palette.vars);

        Ok(vars)
    }

    /// Load a profile from user profiles directory.
    fn load_profile_file(&self, name: &str) -> Result<Profile> {
        let path = self
            .config_dir
            .join("profiles")
            .join(format!("{name}.yaml"));
        let content = self
            .fs
            .read_to_string(&path)
            .with_context(|| format!("Profile not found: {name}"))?;

        serde_yaml::from_str(&content).with_context(|| format!("Failed to parse profile: {name}"))
    }

    /// Load a palette, searching user palettes first, then system palettes.
    fn load_palette_file(&self, name: &str) -> Result<Profile> {
        let user_path = self
            .config_dir
            .join("palettes")
            .join(format!("{name}.yaml"));
        let system_path = self
            .system_dir
            .join("palettes")
            .join(format!("{name}.yaml"));

        // Try user palette first
        if self.fs.exists(&user_path) {
            debug!("Loading user palette: {:?}", user_path);
            let content = self.fs.read_to_string(&user_path)?;
            return serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse user palette: {name}"));
        }

        // Fall back to system palette
        if self.fs.exists(&system_path) {
            debug!("Loading system palette: {:?}", system_path);
            let content = self.fs.read_to_string(&system_path)?;
            return serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse system palette: {name}"));
        }

        bail!("Palette not found: {name} (searched user and system directories)")
    }

    #[allow(
        clippy::similar_names,
        reason = "`content` and `context` are distinct, well-established domain terms"
    )]
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
                    bail!("Template file not found: {input_path}");
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

                // Prepare Env: Prefix everything with THEMIS_
                let mut env_vars = HashMap::new();
                for (k, v) in context {
                    let env_key = format!("THEMIS_{}", k.to_uppercase());
                    match v {
                        Value::String(s) => {
                            env_vars.insert(env_key, s.clone());
                        }
                        Value::Number(n) => {
                            env_vars.insert(env_key, n.to_string());
                        }
                        Value::Bool(b) => {
                            env_vars.insert(env_key, b.to_string());
                        }
                        Value::Null => {
                            debug!("Skipping null value for env var: {}", k);
                        }
                        Value::Array(arr) => {
                            // Join array elements with colon (Unix convention)
                            let joined: Vec<String> = arr
                                .iter()
                                .filter_map(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    Value::Number(n) => Some(n.to_string()),
                                    Value::Bool(b) => Some(b.to_string()),
                                    _ => None, // Skip nested arrays/objects/nulls
                                })
                                .collect();
                            env_vars.insert(env_key, joined.join(":"));
                        }
                        Value::Object(_) => {
                            warn!(
                                "Skipping object value for env var '{}': objects cannot be passed as environment variables",
                                k
                            );
                        }
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
        let cmd = format!("pkill -{signal_name} {app_name}");
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
