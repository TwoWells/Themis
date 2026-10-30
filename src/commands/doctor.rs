// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! The `doctor` command: checks that enrolled apps reference the partials
//! Themis generates (e.g. an `include .themis.conf` line in `kitty.conf`).
use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

use crate::core::config::Config;
use crate::core::integration::Integration;

/// What Themis knows about one app's own config file: where it lives, and how
/// that app's syntax references an included file.
///
/// Deliberately *not* recorded here: which file to look for. That comes from
/// the app's enrollment, so an app whose template writes `current-theme.conf`
/// is checked for `current-theme.conf` rather than for whatever name this
/// table would otherwise have guessed.
struct AppCheck {
    /// App name (matches enrollment key)
    name: &'static str,
    /// Path to config file to check (with ~ expanded)
    config_path: &'static str,
    /// How this app spells an include, with `{}` standing in for the generated
    /// file. Used to word the suggested fix.
    include_fmt: &'static str,
    /// Set when the app names an include without its extension, the way rofi
    /// writes `@theme "current"` for `current.rasi`.
    drops_extension: bool,
}

const APP_CHECKS: &[AppCheck] = &[
    AppCheck {
        name: "kitty",
        config_path: "~/.config/kitty/kitty.conf",
        include_fmt: "include {}",
        drops_extension: false,
    },
    AppCheck {
        name: "waybar",
        config_path: "~/.config/waybar/style.css",
        include_fmt: "@import \"{}\";",
        drops_extension: false,
    },
    AppCheck {
        name: "polybar",
        config_path: "~/.config/polybar/config.ini",
        include_fmt: "include-file = ~/.config/polybar/{}",
        drops_extension: false,
    },
    AppCheck {
        name: "rofi",
        config_path: "~/.config/rofi/config.rasi",
        include_fmt: "@theme \"{}\"",
        drops_extension: true,
    },
    AppCheck {
        name: "foot",
        config_path: "~/.config/foot/foot.ini",
        include_fmt: "include=~/.config/foot/{}",
        drops_extension: false,
    },
    AppCheck {
        name: "alacritty",
        config_path: "~/.config/alacritty/alacritty.toml",
        include_fmt: "import = [\"~/.config/alacritty/{}\"]",
        drops_extension: false,
    },
];

/// The file Themis generates for an app, as its enrollment declares it.
///
/// Commands and scripts produce nothing for the app to include.
const fn generated_path(integration: &Integration) -> Option<&String> {
    match integration {
        Integration::Template { output, .. } => Some(output),
        Integration::Symlink { target, .. } => Some(target),
        Integration::Command { .. } | Integration::Script { .. } => None,
    }
}

/// Outcome of a `doctor` run: which app configs are missing their include.
pub struct DoctorResult {
    /// Human-readable descriptions of problems found, with suggested fixes.
    pub issues: Vec<String>,
    /// Number of apps whose config correctly includes the Themis partial.
    pub ok_count: usize,
    /// Number of enrolled apps with no applicable config check (skipped).
    pub skipped_count: usize,
}

impl DoctorResult {
    const fn new() -> Self {
        Self {
            issues: Vec::new(),
            ok_count: 0,
            skipped_count: 0,
        }
    }

    /// Returns `true` if no issues were found.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Runs the doctor checks for every enrolled app in `config_dir`.
///
/// # Errors
///
/// Returns an error if the config file exists but cannot be read or parsed.
pub fn run(config_dir: &Path) -> Result<DoctorResult> {
    let mut result = DoctorResult::new();

    info!("Running Themis doctor...");
    info!("");

    // Load config to see enrolled apps
    let config_file = config_dir.join("themis.yaml");
    if !config_file.exists() {
        result
            .issues
            .push("Config file not found. Run 'themis init' first.".to_string());
        return Ok(result);
    }

    let content = fs::read_to_string(&config_file)?;
    let config: Config = serde_saphyr::from_str(&content)?;

    // Check each enrolled app
    for (app_name, integration) in &config.enroll {
        check_app(app_name, integration, &mut result);
    }

    // Print summary
    info!("");
    if result.is_healthy() {
        info!(
            "All checks passed! ({} OK, {} skipped)",
            result.ok_count, result.skipped_count
        );
    } else {
        warn!(
            "Found {} issue(s) ({} OK, {} skipped)",
            result.issues.len(),
            result.ok_count,
            result.skipped_count
        );
        info!("");
        info!("Issues to fix:");
        for issue in &result.issues {
            error!("{}", issue);
            info!("");
        }
    }

    Ok(result)
}

fn check_app(app_name: &str, integration: &Integration, result: &mut DoctorResult) {
    // Find the check for this app
    let check = APP_CHECKS.iter().find(|c| c.name == app_name);

    let Some(check) = check else {
        // No known check for this app (e.g., gtk uses gsettings, no config file)
        info!("[{}] No config check needed", app_name);
        result.skipped_count += 1;
        return;
    };

    // Nothing is generated for this app, so there is nothing to include.
    let Some(generated) = generated_path(integration) else {
        info!("[{}] No config check needed", app_name);
        result.skipped_count += 1;
        return;
    };

    let generated = shellexpand::tilde(generated.as_str());
    let generated = Path::new(generated.as_ref());

    let config_path = shellexpand::tilde(check.config_path);
    let path = Path::new(config_path.as_ref());

    // Themis writes this app's config outright rather than a partial beside
    // it -- waybar's style.css, for one -- so asking it to include itself
    // would be nonsense.
    if generated == path {
        info!(
            "[{}] OK - Themis generates {} itself",
            app_name, check.config_path
        );
        result.ok_count += 1;
        return;
    }

    if !path.exists() {
        warn!(
            "[{}] Config file not found: {}",
            app_name, check.config_path
        );
        result.issues.push(format!(
            "[{}] Config file not found: {}\n  Create the file or check if {} is installed.",
            app_name, check.config_path, app_name
        ));
        return;
    }

    let Some(file_name) = generated.file_name().and_then(|n| n.to_str()) else {
        result.issues.push(format!(
            "[{}] Enrolled output has no file name: {}",
            app_name,
            generated.display()
        ));
        return;
    };

    // The reference the app is expected to carry. Where an app drops the
    // extension, accept either spelling: configs often give a full path.
    let stem = generated.file_stem().and_then(|s| s.to_str());
    let reference = if check.drops_extension {
        stem.unwrap_or(file_name)
    } else {
        file_name
    };

    // Read the config file and check that it points at what Themis writes
    match fs::read_to_string(path) {
        Ok(content) => {
            if content.contains(reference) || content.contains(file_name) {
                info!("[{}] OK - found '{}' in config", app_name, reference);
                result.ok_count += 1;
            } else {
                warn!("[{}] Missing include pattern", app_name);
                result.issues.push(format!(
                    "[{}] Add this line to {}:\n  {}",
                    app_name,
                    check.config_path,
                    check.include_fmt.replace("{}", reference)
                ));
            }
        }
        Err(e) => {
            result.issues.push(format!(
                "[{}] Could not read {}: {}",
                app_name, check.config_path, e
            ));
        }
    }
}
