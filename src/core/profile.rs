//! Profile and palette definitions for Themis.
//!
//! Profiles define theme configurations with variable inheritance.
//! A profile can include a palette (or another profile), inheriting
//! its variables and optionally overriding them.
//!
//! # Example
//!
//! ```
//! use themis::core::profile::Profile;
//!
//! // A simple profile with color variables
//! let yaml = "
//! vars:
//!   bg: '#2e3440'
//!   fg: '#eceff4'
//!   accent: '#88c0d0'
//! ";
//!
//! let profile: Profile = serde_yaml::from_str(yaml).unwrap();
//! assert_eq!(profile.vars.get("bg").unwrap(), "#2e3440");
//! ```
//!
//! # Inheritance
//!
//! Profiles can include palettes for inheritance:
//!
//! ```
//! use themis::core::profile::Profile;
//!
//! // Profile that includes a palette and overrides one variable
//! let yaml = "
//! include: nord
//! vars:
//!   accent: '#b48ead'
//! ";
//!
//! let profile: Profile = serde_yaml::from_str(yaml).unwrap();
//! assert_eq!(profile.include, Some("nord".to_string()));
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A Profile defines a complete theme configuration.
///
/// Profiles can include palettes (or other profiles) for inheritance,
/// with their own variables taking precedence over included ones.
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    /// Optional metadata (name, description)
    #[serde(default)]
    pub metadata: Option<ProfileMetadata>,

    /// Include a palette or parent profile by name (e.g., "nord").
    /// The included palette's variables are inherited and can be overridden.
    pub include: Option<String>,

    /// Variable definitions. These override any inherited variables.
    #[serde(default)]
    pub vars: HashMap<String, Value>,
}

/// A Palette is structurally identical to a Profile.
///
/// Palettes define color variables and can include other palettes.
/// They're typically stored in `~/.config/themis/palettes/` or
/// `/usr/share/themis/palettes/`.
pub type Palette = Profile;

/// Optional metadata for a profile.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileMetadata {
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
}
