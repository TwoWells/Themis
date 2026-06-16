// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! The `status` command: reports the currently loaded profile from saved state.
use anyhow::Result;
use tracing::info;

use crate::core::state::State;

/// The currently loaded profile and when it was last applied.
pub struct StatusResult {
    /// Name of the active profile, or `None` if nothing is loaded.
    pub profile: Option<String>,
    /// Timestamp of the last successful load, or `None` if never run.
    pub last_run: Option<String>,
}

/// Reads persisted state and reports the current profile.
///
/// # Errors
///
/// Returns an error if the state file exists but cannot be read or parsed.
pub fn run() -> Result<StatusResult> {
    let state = State::load()?;

    if let Some(state) = state {
        let profile = state.current.as_ref().map(|c| c.profile.clone());
        let last_run = Some(state.last_run.clone());

        if let Some(ref p) = profile {
            info!("Current profile: {}", p);
        } else {
            info!("No profile currently loaded");
        }

        info!("Last run: {}", state.last_run);

        Ok(StatusResult { profile, last_run })
    } else {
        info!("No state found. Run 'themis load <PROFILE>' to load a profile.");
        Ok(StatusResult {
            profile: None,
            last_run: None,
        })
    }
}
