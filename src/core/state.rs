//! State persistence for Themis.
//!
//! Tracks the currently loaded profile and saves it to disk following
//! the XDG Base Directory specification.
//!
//! # Example
//!
//! ```
//! use themis::core::state::State;
//! use tempfile::TempDir;
//!
//! // Create a new state after loading a profile
//! let state = State::new("nord-dark".to_string());
//! assert_eq!(state.current.as_ref().unwrap().profile, "nord-dark");
//!
//! // Save to a custom path (for testing)
//! let temp = TempDir::new().unwrap();
//! let path = temp.path().join("state.json");
//! state.save_to(&path).unwrap();
//!
//! // Load it back
//! let loaded = State::load_from(&path).unwrap().unwrap();
//! assert_eq!(loaded.current.unwrap().profile, "nord-dark");
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

const STATE_FILE: &str = "state.json";

/// Persistent state for Themis, saved between invocations.
///
/// State is stored at `$XDG_STATE_HOME/themis/state.json` (defaults to
/// `~/.local/state/themis/state.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Timestamp of last successful load (ISO 8601 format)
    pub last_run: String,

    /// Whether the last operation succeeded
    pub success: bool,

    /// Current profile information
    pub current: Option<CurrentState>,
}

/// Information about the currently loaded profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentState {
    /// Name of the currently loaded profile
    pub profile: String,
}

impl State {
    /// Create a new state for a successfully loaded profile.
    ///
    /// # Example
    ///
    /// ```
    /// use themis::core::state::State;
    ///
    /// let state = State::new("gruvbox".to_string());
    /// assert!(state.success);
    /// assert_eq!(state.current.unwrap().profile, "gruvbox");
    /// ```
    #[must_use]
    pub fn new(profile: String) -> Self {
        Self {
            last_run: chrono_now(),
            success: true,
            current: Some(CurrentState { profile }),
        }
    }

    /// Get the state file path following XDG Base Directory spec.
    ///
    /// Returns `$XDG_STATE_HOME/themis/state.json` if `XDG_STATE_HOME` is set,
    /// otherwise `~/.local/state/themis/state.json`.
    pub fn state_path() -> Result<PathBuf> {
        let state_home = if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
            PathBuf::from(xdg)
        } else {
            let home = std::env::var("HOME").context("HOME environment variable not set")?;
            PathBuf::from(home).join(".local/state")
        };
        Ok(state_home.join("themis").join(STATE_FILE))
    }

    /// Load state from the default XDG location.
    ///
    /// Returns `Ok(None)` if no state file exists yet.
    pub fn load() -> Result<Option<Self>> {
        let path = Self::state_path()?;

        if !path.exists() {
            debug!("No state file found at {:?}", path);
            return Ok(None);
        }

        let content = fs::read_to_string(&path).context("Failed to read state file")?;

        let state: Self = serde_json::from_str(&content).context("Failed to parse state file")?;

        Ok(Some(state))
    }

    /// Load state from a specific path.
    ///
    /// Returns `Ok(None)` if the file doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// use themis::core::state::State;
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let path = temp.path().join("state.json");
    ///
    /// // No file yet
    /// assert!(State::load_from(&path).unwrap().is_none());
    ///
    /// // Save and reload
    /// State::new("dracula".to_string()).save_to(&path).unwrap();
    /// let state = State::load_from(&path).unwrap().unwrap();
    /// assert_eq!(state.current.unwrap().profile, "dracula");
    /// ```
    pub fn load_from(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path).context("Failed to read state file")?;
        let state: Self = serde_json::from_str(&content).context("Failed to parse state file")?;

        Ok(Some(state))
    }

    /// Save state to the default XDG location.
    ///
    /// Creates parent directories if they don't exist.
    pub fn save(&self) -> Result<()> {
        let path = Self::state_path()?;
        self.save_to(&path)
    }

    /// Save state to a specific path.
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Example
    ///
    /// ```
    /// use themis::core::state::State;
    /// use tempfile::TempDir;
    /// use std::fs;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let path = temp.path().join("nested/dir/state.json");
    ///
    /// // Parent directories are created automatically
    /// State::new("catppuccin".to_string()).save_to(&path).unwrap();
    /// assert!(path.exists());
    ///
    /// // State is saved as JSON
    /// let content = fs::read_to_string(&path).unwrap();
    /// assert!(content.contains("catppuccin"));
    /// ```
    pub fn save_to(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create state directory")?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize state")?;

        fs::write(path, content).context("Failed to write state file")?;

        debug!("Saved state to {:?}", path);
        Ok(())
    }
}

/// Get current timestamp in ISO 8601 format
fn chrono_now() -> String {
    // Simple timestamp without external chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = duration.as_secs();

    // Convert to rough ISO format (good enough for display)
    // This is a simplified version - for production, use chrono crate
    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;
    let seconds = remaining_secs % 60;

    // Calculate approximate date (rough, doesn't account for leap years precisely)
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };

        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [u64; 12] = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months {
        if remaining_days < days {
            break;
        }
        remaining_days -= days;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests assert via unwrap/expect"
    )]

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_state_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.json");

        let state = State::new("nord-dark".to_string());
        state.save_to(&state_path).unwrap();

        let loaded = State::load_from(&state_path).unwrap().unwrap();
        assert_eq!(loaded.current.as_ref().unwrap().profile, "nord-dark");
        assert!(loaded.success);
    }

    #[test]
    fn test_state_load_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("nonexistent.json");

        let loaded = State::load_from(&state_path).unwrap();
        assert!(loaded.is_none());
    }
}
