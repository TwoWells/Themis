use anyhow::Result;
use tracing::info;

use crate::core::state::State;

pub struct StatusResult {
    pub profile: Option<String>,
    pub last_run: Option<String>,
}

pub fn run() -> Result<StatusResult> {
    let state = State::load()?;

    match state {
        Some(state) => {
            let profile = state.current.as_ref().map(|c| c.profile.clone());
            let last_run = Some(state.last_run.clone());

            if let Some(ref p) = profile {
                info!("Current profile: {}", p);
            } else {
                info!("No profile currently loaded");
            }

            info!("Last run: {}", state.last_run);

            Ok(StatusResult { profile, last_run })
        }
        None => {
            info!("No state found. Run 'theman load <PROFILE>' to load a profile.");
            Ok(StatusResult {
                profile: None,
                last_run: None,
            })
        }
    }
}
