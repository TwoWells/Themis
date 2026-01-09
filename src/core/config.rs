//! Configuration schema for TheMan.
//!
//! The main configuration file (`theman.yaml`) defines which applications
//! are enrolled and how they should be themed.
//!
//! # Example
//!
//! ```
//! use theman::core::config::Config;
//!
//! let yaml = r#"
//! enroll:
//!   kitty:
//!     type: template
//!     input: "~/.config/theman/templates/kitty.j2"
//!     output: "~/.config/kitty/.theman.conf"
//!   gtk:
//!     type: command
//!     commands:
//!       - "gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'"
//!
//! overrides:
//!   global:
//!     font_size: 12
//!   kitty:
//!     font_size: 14
//! "#;
//!
//! let config: Config = serde_yaml::from_str(yaml).unwrap();
//! assert_eq!(config.enroll.len(), 2);
//!
//! // App-specific overrides take precedence over global
//! let kitty_overrides = config.get_overrides_for("kitty");
//! assert_eq!(kitty_overrides.get("font_size").unwrap(), 14);
//! ```

use super::integration::Integration;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Main configuration for TheMan, loaded from `theman.yaml`.
///
/// # Fields
///
/// - `enroll`: Applications to theme, processed in YAML definition order
/// - `overrides`: Variable overrides (global or per-app)
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The currently active profile name (e.g., "nord")
    pub current_profile: Option<String>,

    /// Map of enrolled applications, processed in YAML definition order.
    /// Key: App Name (e.g., "kitty")
    /// Value: Integration Definition
    #[serde(default)]
    pub enroll: IndexMap<String, Integration>,

    /// Global or App-specific variable overrides.
    /// Key: "global" or App Name
    /// Value: Map of overrides
    #[serde(default)]
    pub overrides: HashMap<String, HashMap<String, Value>>,
}

impl Config {
    /// Returns global overrides merged with app-specific overrides.
    ///
    /// App-specific overrides take precedence over global overrides.
    ///
    /// # Example
    ///
    /// ```
    /// use theman::core::config::Config;
    ///
    /// let yaml = r#"
    /// enroll: {}
    /// overrides:
    ///   global:
    ///     font: "Sans"
    ///     size: 12
    ///   waybar:
    ///     size: 10
    /// "#;
    ///
    /// let config: Config = serde_yaml::from_str(yaml).unwrap();
    ///
    /// // waybar gets global font but its own size
    /// let waybar = config.get_overrides_for("waybar");
    /// assert_eq!(waybar.get("font").unwrap(), "Sans");
    /// assert_eq!(waybar.get("size").unwrap(), 10);
    ///
    /// // kitty gets all global values (no app-specific overrides)
    /// let kitty = config.get_overrides_for("kitty");
    /// assert_eq!(kitty.get("size").unwrap(), 12);
    /// ```
    pub fn get_overrides_for(&self, app_name: &str) -> HashMap<String, Value> {
        let mut resolved = HashMap::new();

        // 1. Merge Globals
        if let Some(globals) = self.overrides.get("global") {
            resolved.extend(globals.clone());
        }

        // 2. Merge App Specifics
        if let Some(app_vars) = self.overrides.get(app_name) {
            resolved.extend(app_vars.clone());
        }

        resolved
    }
}
