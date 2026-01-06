use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An Integration defines how to manage a specific application.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Integration {
    /// Render a template file to a target path
    Template {
        input: String,
        output: String,
        /// Optional command to reload the application
        #[serde(default)]
        reload_cmd: Option<String>,
        #[serde(default)]
        reload_signal: Option<String>,
    },

    /// Symlink a source file/dir to a target
    Symlink {
        source: String,
        target: String,
        #[serde(default)]
        reload_cmd: Option<String>,
    },

    /// Execute an external script
    Script {
        path: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },

    /// Run a list of shell commands
    Command { commands: Vec<String> },
}
