// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::info;

const SAMPLE_CONFIG: &str = r#"# Themis Configuration
# See: https://github.com/TwoWells/Themis

# Enroll applications to be themed.
# Each app needs an integration type: template, symlink, command, or script.
enroll:
  # Example: Kitty terminal (uses template + live reload)
  # kitty:
  #   type: template
  #   input: "~/.config/themis/templates/kitty.j2"
  #   output: "~/.config/kitty/.themis.conf"
  #
  # Example: Waybar (uses template + signal reload)
  # waybar:
  #   type: template
  #   input: "~/.config/themis/templates/waybar.css.j2"
  #   output: "~/.config/waybar/colors.css"
  #   reload_signal: SIGUSR2
  #
  # Example: GTK (uses gsettings commands)
  # gtk:
  #   type: command
  #   commands:
  #     - "gsettings set org.gnome.desktop.interface gtk-theme '{{ gtk_theme }}'"
  #     - "gsettings set org.gnome.desktop.interface color-scheme 'prefer-{{ mode }}'"

# Override profile variables globally or per-app
overrides:
  # Global overrides (apply to all apps)
  # global:
  #   font_family: "JetBrains Mono"
  #
  # App-specific overrides
  # kitty:
  #   opacity: 0.95
"#;

const SAMPLE_PROFILE: &str = r##"# Example Profile
# Include a palette and add your customizations

# Include a system or user palette (e.g., nord, dracula, tokyo-night)
# include: nord

# Add or override variables
vars:
  # Colors (if not using a palette)
  bg: "#1a1b26"
  fg: "#c0caf5"

  # Terminal colors
  color0: "#15161e"
  color1: "#f7768e"
  color2: "#9ece6a"
  color3: "#e0af68"
  color4: "#7aa2f7"
  color5: "#bb9af7"
  color6: "#7dcfff"
  color7: "#a9b1d6"

  # Brights
  color8: "#414868"
  color9: "#f7768e"
  color10: "#9ece6a"
  color11: "#e0af68"
  color12: "#7aa2f7"
  color13: "#bb9af7"
  color14: "#7dcfff"
  color15: "#c0caf5"

  # Additional settings
  mode: "dark"
  font_family: "monospace"
  font_size: 12
"##;

pub fn run(config_dir: &Path) -> Result<()> {
    // Check if already initialized
    let config_file = config_dir.join("themis.yaml");
    if config_file.exists() {
        info!("Configuration already exists at {:?}", config_dir);
        info!("To reinitialize, remove the directory first.");
        return Ok(());
    }

    info!("Initializing Themis configuration...");

    // Create directories
    let dirs = ["profiles", "palettes", "templates"];
    for dir in &dirs {
        let path = config_dir.join(dir);
        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        info!("Created {:?}", path);
    }

    // Write sample config
    fs::write(&config_file, SAMPLE_CONFIG).context("Failed to write themis.yaml")?;
    info!("Created {:?}", config_file);

    // Write sample profile
    let profile_path = config_dir.join("profiles/example.yaml");
    fs::write(&profile_path, SAMPLE_PROFILE).context("Failed to write example profile")?;
    info!("Created {:?}", profile_path);

    info!("");
    info!("Themis initialized successfully!");
    info!("");
    info!("Next steps:");
    info!("  1. Edit {:?} to enroll your apps", config_file);
    info!("  2. Edit {:?} or create a new profile", profile_path);
    info!("  3. Run: themis load example");

    Ok(())
}
