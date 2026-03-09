use anyhow::{Context, Result};
use std::path::Path;
use tracing::{error, info, warn};

use crate::core::config::Config;
use crate::core::integration::Integration;
use crate::core::profile::Profile;

pub struct VerifyResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl VerifyResult {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn run(config_dir: &Path, system_dir: &Path) -> Result<VerifyResult> {
    let mut result = VerifyResult::new();

    info!("Verifying Themis configuration...");

    // 1. Check config file exists and is valid YAML
    let config_file = config_dir.join("themis.yaml");
    if !config_file.exists() {
        result.error(format!(
            "Config file not found: {:?}\nRun 'themis init' to create it.",
            config_file
        ));
        return Ok(result);
    }

    let config: Config = match std::fs::read_to_string(&config_file)
        .context("Failed to read themis.yaml")
        .and_then(|content| serde_yaml::from_str(&content).context("Failed to parse themis.yaml"))
    {
        Ok(c) => c,
        Err(e) => {
            result.error(format!("Invalid config file: {}", e));
            return Ok(result);
        }
    };

    info!("Config file: OK");

    // 2. Check enrolled integrations
    for (app_name, integration) in &config.enroll {
        verify_integration(app_name, integration, config_dir, &mut result);
    }

    // 3. Check profiles directory
    let profiles_dir = config_dir.join("profiles");
    if profiles_dir.is_dir() {
        verify_profiles(&profiles_dir, config_dir, system_dir, &mut result);
    } else {
        result.warn("No profiles directory found");
    }

    // 4. Check palettes directory (user)
    let palettes_dir = config_dir.join("palettes");
    if palettes_dir.is_dir() {
        verify_palettes(&palettes_dir, config_dir, system_dir, &mut result);
    }

    // Print summary
    info!("");
    if result.is_ok() {
        if result.warnings.is_empty() {
            info!("All checks passed!");
        } else {
            info!(
                "Verification passed with {} warning(s)",
                result.warnings.len()
            );
            for w in &result.warnings {
                warn!("{}", w);
            }
        }
    } else {
        error!("Verification failed with {} error(s)", result.errors.len());
        for e in &result.errors {
            error!("{}", e);
        }
    }

    Ok(result)
}

fn verify_integration(
    app_name: &str,
    integration: &Integration,
    _config_dir: &Path,
    result: &mut VerifyResult,
) {
    match integration {
        Integration::Template { input, output, .. } => {
            let input_path = shellexpand::tilde(input);
            if !Path::new(input_path.as_ref()).exists() {
                result.error(format!("[{}] Template not found: {}", app_name, input));
            } else {
                info!("[{}] Template: OK", app_name);
            }

            // Check output parent directory exists
            let output_path = shellexpand::tilde(output);
            if let Some(parent) = Path::new(output_path.as_ref()).parent() {
                if !parent.exists() {
                    result.warn(format!(
                        "[{}] Output directory doesn't exist: {:?}",
                        app_name, parent
                    ));
                }
            }
        }
        Integration::Symlink { target, .. } => {
            // Source is a template, we can't fully validate without context
            // But we can check if target parent exists
            let target_path = shellexpand::tilde(target);
            if let Some(parent) = Path::new(target_path.as_ref()).parent() {
                if !parent.exists() {
                    result.warn(format!(
                        "[{}] Symlink target directory doesn't exist: {:?}",
                        app_name, parent
                    ));
                }
            }
            info!("[{}] Symlink config: OK", app_name);
        }
        Integration::Command { commands } => {
            if commands.is_empty() {
                result.warn(format!("[{}] No commands defined", app_name));
            } else {
                info!("[{}] Commands: {} defined", app_name, commands.len());
            }
        }
        Integration::Script { path, .. } => {
            let script_path = shellexpand::tilde(path);
            if !Path::new(script_path.as_ref()).exists() {
                result.error(format!("[{}] Script not found: {}", app_name, path));
            } else {
                info!("[{}] Script: OK", app_name);
            }
        }
    }
}

fn verify_profiles(
    profiles_dir: &Path,
    config_dir: &Path,
    system_dir: &Path,
    result: &mut VerifyResult,
) {
    let entries = match std::fs::read_dir(profiles_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            let name = path.file_stem().unwrap().to_string_lossy();

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    result.error(format!("Profile '{}' couldn't be read: {}", name, e));
                    continue;
                }
            };

            let profile: Profile = match serde_yaml::from_str(&content) {
                Ok(p) => p,
                Err(e) => {
                    result.error(format!("Profile '{}' is invalid YAML: {}", name, e));
                    continue;
                }
            };

            // Check if included palette exists
            if let Some(ref palette_name) = profile.include {
                if !palette_exists(palette_name, config_dir, system_dir) {
                    result.error(format!(
                        "Profile '{}' includes palette '{}' which doesn't exist",
                        name, palette_name
                    ));
                }
            }
            info!("Profile '{}': OK", name);
        }
    }
}

fn verify_palettes(
    palettes_dir: &Path,
    config_dir: &Path,
    system_dir: &Path,
    result: &mut VerifyResult,
) {
    let entries = match std::fs::read_dir(palettes_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            let name = path.file_stem().unwrap().to_string_lossy();

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    result.error(format!("Palette '{}' couldn't be read: {}", name, e));
                    continue;
                }
            };

            let palette: Profile = match serde_yaml::from_str(&content) {
                Ok(p) => p,
                Err(e) => {
                    result.error(format!("Palette '{}' is invalid YAML: {}", name, e));
                    continue;
                }
            };

            // Check if included palette exists
            if let Some(ref parent_name) = palette.include {
                if !palette_exists(parent_name, config_dir, system_dir) {
                    result.error(format!(
                        "Palette '{}' includes '{}' which doesn't exist",
                        name, parent_name
                    ));
                }
            }
            info!("Palette '{}': OK", name);
        }
    }
}

fn palette_exists(name: &str, config_dir: &Path, system_dir: &Path) -> bool {
    let user_path = config_dir.join("palettes").join(format!("{}.yaml", name));
    let system_path = system_dir.join("palettes").join(format!("{}.yaml", name));
    user_path.exists() || system_path.exists()
}
