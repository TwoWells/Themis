use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A Profile defines a complete theme configuration.
/// It can include a palette and add additional variables.
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub metadata: Option<ProfileMetadata>,

    /// Include a palette or parent profile by name (e.g., "nord")
    pub include: Option<String>,

    /// The variable definitions
    #[serde(default)]
    pub vars: HashMap<String, Value>,
}

/// A Palette defines color variables that can be included by profiles.
/// Palettes can also include other palettes for inheritance.
pub type Palette = Profile;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub description: Option<String>,
}
