// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! Integration definitions: how Themis applies a theme to one application.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An Integration defines how to manage a specific application.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Integration {
    /// Render a template file to a target path
    Template {
        /// Path to the template file (`~` is expanded).
        input: String,
        /// Path the rendered output is written to (`~` is expanded).
        output: String,
        /// Optional command to reload the application
        #[serde(default)]
        reload_cmd: Option<String>,
        /// Optional signal (e.g. `SIGUSR2`) sent to the app via `pkill` to reload.
        #[serde(default)]
        reload_signal: Option<String>,
    },

    /// Symlink a source file/dir to a target
    Symlink {
        /// Source path to link from; rendered as a template, then `~`-expanded.
        source: String,
        /// Target path the symlink is created at (`~` is expanded).
        target: String,
        /// Optional command to reload the application after linking.
        #[serde(default)]
        reload_cmd: Option<String>,
    },

    /// Execute an external script
    Script {
        /// Path to the executable script (`~` is expanded).
        path: String,
        /// Arguments passed to the script; each is rendered as a template.
        #[serde(default)]
        args: Vec<String>,
        /// Extra environment variables, merged over the `THEMIS_*` variable set.
        #[serde(default)]
        env: HashMap<String, String>,
    },

    /// Run a list of shell commands
    Command {
        /// Shell commands to run in order; each is rendered as a template.
        commands: Vec<String>,
    },
}
