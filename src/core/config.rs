use super::integration::Integration;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    /// Returns the global overrides merged with app-specific overrides for a given app.
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
