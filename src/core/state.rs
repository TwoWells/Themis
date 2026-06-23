// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
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
    /// otherwise `~/.local/state/themis/state.json`. Resolution is delegated to
    /// the shared [`crate::core::paths`] module so config, state, and data all
    /// agree on a single source of truth.
    ///
    /// # Errors
    ///
    /// Returns an error if neither `XDG_STATE_HOME` nor `HOME` is set.
    pub fn state_path() -> Result<PathBuf> {
        crate::core::paths::state_file()
    }

    /// Load state from the default XDG location.
    ///
    /// Returns `Ok(None)` if no state file exists yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the state path cannot be determined, or if the file
    /// exists but cannot be read or parsed.
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
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the state path cannot be determined or the file
    /// cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::state_path()?;
        self.save_to(&path)
    }

    /// Save state to a specific path.
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the parent directories or file cannot be written.
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

/// Get current timestamp in ISO 8601 format.
fn chrono_now() -> String {
    format_iso8601(now_unix())
}

// Mutation-testing note: the `SystemTime::now()` read below is the one
// genuinely untestable line — its `-> 0` / `-> 1` mutants survive by design.
// The formatting it feeds is covered by `format_iso8601`'s vector tests.
/// Read the wall clock and return seconds since the Unix epoch.
fn now_unix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Format seconds since the Unix epoch as an ISO 8601 UTC timestamp.
///
/// Pure: no clock read, no I/O. Uses a hand-rolled epoch-to-date conversion
/// to avoid an external chrono dependency.
fn format_iso8601(secs: u64) -> String {
    // Convert to ISO format (good enough for display)
    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;
    let seconds = remaining_secs % 60;

    // Calculate approximate date (rough, doesn't account for leap years precisely)
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    // Mutation-testing note: the century terms of the Gregorian rule used here
    // and below (`year % 100` / `year % 400`) only change behavior for years
    // divisible by 100. `last_run` is always a current-era display timestamp,
    // never a century year, so those operator mutants survive — those branches
    // are untested by choice; the rule itself is correct.
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

    // Expected strings are derived from an independent oracle (`jq`'s
    // libc-backed `gmtime`/`strftime`), not from this implementation:
    //   echo <secs> | jq -r '. | gmtime | strftime("%Y-%m-%dT%H:%M:%SZ")'
    #[test]
    fn test_format_iso8601_epoch() {
        assert_eq!(format_iso8601(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_format_iso8601_known_non_leap_datetime() {
        // 1700000000 -> 2023-11-14T22:13:20Z (2023 is not a leap year)
        assert_eq!(format_iso8601(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn test_format_iso8601_leap_day() {
        // 1582979696 -> 2020-02-29T12:34:56Z (exercises the Feb 29 leap branch)
        assert_eq!(format_iso8601(1_582_979_696), "2020-02-29T12:34:56Z");
    }

    #[test]
    fn test_format_iso8601_year_boundary() {
        // 1609459200 -> 2021-01-01T00:00:00Z (first second of a new year)
        assert_eq!(format_iso8601(1_609_459_200), "2021-01-01T00:00:00Z");
    }

    #[test]
    fn test_format_iso8601_end_of_month() {
        // 1675209599 -> 2023-01-31T23:59:59Z (last second before Feb rollover)
        assert_eq!(format_iso8601(1_675_209_599), "2023-01-31T23:59:59Z");
    }

    #[test]
    fn test_format_iso8601_first_of_month() {
        // 1675209600 -> 2023-02-01T00:00:00Z (one second after the end-of-month
        // case). Pins the month-loop boundary `remaining_days < days`: a `<=`
        // regression would roll the first of the month back into "2023-01-32".
        assert_eq!(format_iso8601(1_675_209_600), "2023-02-01T00:00:00Z");
    }

    #[test]
    fn test_chrono_now_is_iso8601_shaped() {
        // chrono_now reads the wall clock, so its exact value can't be pinned;
        // assert it wires now_unix into format_iso8601 and yields a real
        // timestamp (kills the `-> String::new()` / `-> "xyzzy"` wrapper mutants).
        let ts = chrono_now();
        assert_eq!(ts.len(), 20);
        assert_eq!(ts.as_bytes()[10], b'T');
        assert!(ts.ends_with('Z'));
    }
}
