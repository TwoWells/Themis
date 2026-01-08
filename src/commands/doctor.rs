use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

use crate::core::config::Config;

/// Known app configurations and what to check for
struct AppCheck {
    /// App name (matches enrollment key)
    name: &'static str,
    /// Path to config file to check (with ~ expanded)
    config_path: &'static str,
    /// Pattern to look for in the config file
    pattern: &'static str,
    /// What to add if pattern is missing
    fix: &'static str,
}

const APP_CHECKS: &[AppCheck] = &[
    AppCheck {
        name: "kitty",
        config_path: "~/.config/kitty/kitty.conf",
        pattern: "include .theman.conf",
        fix: "Add this line to ~/.config/kitty/kitty.conf:\n  include .theman.conf",
    },
    AppCheck {
        name: "waybar",
        config_path: "~/.config/waybar/style.css",
        pattern: "@import \"colors.css\"",
        fix: "Add this line to ~/.config/waybar/style.css:\n  @import \"colors.css\";",
    },
    AppCheck {
        name: "polybar",
        config_path: "~/.config/polybar/config.ini",
        pattern: "include-file",
        fix: "Add this line to ~/.config/polybar/config.ini:\n  include-file = ~/.config/polybar/colors.ini",
    },
    AppCheck {
        name: "rofi",
        config_path: "~/.config/rofi/config.rasi",
        pattern: "@theme \"theman\"",
        fix: "Add this line to ~/.config/rofi/config.rasi:\n  @theme \"theman\";",
    },
    AppCheck {
        name: "foot",
        config_path: "~/.config/foot/foot.ini",
        pattern: "include=",
        fix: "Add this line to ~/.config/foot/foot.ini:\n  include=~/.config/foot/theman.ini",
    },
    AppCheck {
        name: "alacritty",
        config_path: "~/.config/alacritty/alacritty.toml",
        pattern: "import",
        fix: "Add this to ~/.config/alacritty/alacritty.toml:\n  import = [\"~/.config/alacritty/theman.toml\"]",
    },
];

pub struct DoctorResult {
    pub issues: Vec<String>,
    pub ok_count: usize,
    pub skipped_count: usize,
}

impl DoctorResult {
    fn new() -> Self {
        Self {
            issues: Vec::new(),
            ok_count: 0,
            skipped_count: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.issues.is_empty()
    }
}

pub fn run(config_dir: &Path) -> Result<DoctorResult> {
    let mut result = DoctorResult::new();

    info!("Running TheMan doctor...");
    info!("");

    // Load config to see enrolled apps
    let config_file = config_dir.join("theman.yaml");
    if !config_file.exists() {
        result
            .issues
            .push("Config file not found. Run 'theman init' first.".to_string());
        return Ok(result);
    }

    let content = fs::read_to_string(&config_file)?;
    let config: Config = serde_yaml::from_str(&content)?;

    // Check each enrolled app
    for (app_name, _integration) in &config.enroll {
        check_app(app_name, &mut result);
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

fn check_app(app_name: &str, result: &mut DoctorResult) {
    // Find the check for this app
    let check = APP_CHECKS.iter().find(|c| c.name == app_name);

    let Some(check) = check else {
        // No known check for this app (e.g., gtk uses gsettings, no config file)
        info!("[{}] No config check needed", app_name);
        result.skipped_count += 1;
        return;
    };

    let config_path = shellexpand::tilde(check.config_path);
    let path = Path::new(config_path.as_ref());

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

    // Read the config file and check for pattern
    match fs::read_to_string(path) {
        Ok(content) => {
            if content.contains(check.pattern) {
                info!("[{}] OK - found '{}' in config", app_name, check.pattern);
                result.ok_count += 1;
            } else {
                warn!("[{}] Missing include pattern", app_name);
                result.issues.push(format!("[{}] {}", app_name, check.fix));
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
