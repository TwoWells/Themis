use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub metadata: ProfileMetadata,

    /// The name of the parent profile to inherit from (e.g., "dark")
    pub extends: Option<String>,

    /// The variable definitions
    #[serde(default)]
    pub vars: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub description: Option<String>,
}
