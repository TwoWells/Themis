// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
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
    const fn new() -> Self {
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

    #[must_use]
    pub const fn is_ok(&self) -> bool {
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
            "Config file not found: {}\nRun 'themis init' to create it.",
            config_file.display()
        ));
        return Ok(result);
    }

    let config: Config = match std::fs::read_to_string(&config_file)
        .context("Failed to read themis.yaml")
        .and_then(|content| serde_yaml::from_str(&content).context("Failed to parse themis.yaml"))
    {
        Ok(c) => c,
        Err(e) => {
            result.error(format!("Invalid config file: {e}"));
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
            if Path::new(input_path.as_ref()).exists() {
                info!("[{app_name}] Template: OK");
            } else {
                result.error(format!("[{app_name}] Template not found: {input}"));
            }

            // Check output parent directory exists
            let output_path = shellexpand::tilde(output);
            if let Some(parent) = Path::new(output_path.as_ref()).parent()
                && !parent.exists()
            {
                result.warn(format!(
                    "[{app_name}] Output directory doesn't exist: {}",
                    parent.display()
                ));
            }
        }
        Integration::Symlink { target, .. } => {
            // Source is a template, we can't fully validate without context
            // But we can check if target parent exists
            let target_path = shellexpand::tilde(target);
            if let Some(parent) = Path::new(target_path.as_ref()).parent()
                && !parent.exists()
            {
                result.warn(format!(
                    "[{app_name}] Symlink target directory doesn't exist: {}",
                    parent.display()
                ));
            }
            info!("[{app_name}] Symlink config: OK");
        }
        Integration::Command { commands } => {
            if commands.is_empty() {
                result.warn(format!("[{app_name}] No commands defined"));
            } else {
                info!("[{app_name}] Commands: {} defined", commands.len());
            }
        }
        Integration::Script { path, .. } => {
            let script_path = shellexpand::tilde(path);
            if Path::new(script_path.as_ref()).exists() {
                info!("[{app_name}] Script: OK");
            } else {
                result.error(format!("[{app_name}] Script not found: {path}"));
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
    let Ok(entries) = std::fs::read_dir(profiles_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            let Some(stem) = path.file_stem() else {
                continue;
            };
            let name = stem.to_string_lossy();

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    result.error(format!("Profile '{name}' couldn't be read: {e}"));
                    continue;
                }
            };

            let profile: Profile = match serde_yaml::from_str(&content) {
                Ok(p) => p,
                Err(e) => {
                    result.error(format!("Profile '{name}' is invalid YAML: {e}"));
                    continue;
                }
            };

            // Check if included palette exists
            if let Some(ref palette_name) = profile.include
                && !palette_exists(palette_name, config_dir, system_dir)
            {
                result.error(format!(
                    "Profile '{name}' includes palette '{palette_name}' which doesn't exist"
                ));
            }
            info!("Profile '{name}': OK");
        }
    }
}

fn verify_palettes(
    palettes_dir: &Path,
    config_dir: &Path,
    system_dir: &Path,
    result: &mut VerifyResult,
) {
    let Ok(entries) = std::fs::read_dir(palettes_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            let Some(stem) = path.file_stem() else {
                continue;
            };
            let name = stem.to_string_lossy();

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    result.error(format!("Palette '{name}' couldn't be read: {e}"));
                    continue;
                }
            };

            let palette: Profile = match serde_yaml::from_str(&content) {
                Ok(p) => p,
                Err(e) => {
                    result.error(format!("Palette '{name}' is invalid YAML: {e}"));
                    continue;
                }
            };

            // Check if included palette exists
            if let Some(ref parent_name) = palette.include
                && !palette_exists(parent_name, config_dir, system_dir)
            {
                result.error(format!(
                    "Palette '{name}' includes '{parent_name}' which doesn't exist"
                ));
            }
            info!("Palette '{name}': OK");
        }
    }
}

fn palette_exists(name: &str, config_dir: &Path, system_dir: &Path) -> bool {
    let user_path = config_dir.join("palettes").join(format!("{name}.yaml"));
    let system_path = system_dir.join("palettes").join(format!("{name}.yaml"));
    user_path.exists() || system_path.exists()
}
