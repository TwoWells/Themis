// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! Abstract I/O interfaces used by the orchestrator.
//!
//! These traits decouple the core engine from concrete I/O so it can be driven
//! by real adapters, dry-run loggers, or test mocks.
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Abstract interface for rendering text templates.
/// This isolates the core from specific engines like Tera or Handlebars.
pub trait TemplateRenderer {
    /// Render a string template with the given context.
    ///
    /// # Errors
    ///
    /// Returns an error if the template fails to parse or render.
    fn render(&self, template_content: &str, context: &HashMap<String, Value>) -> Result<String>;
}

/// Abstract interface for executing system commands.
/// This allows for mocking command execution during tests or dry-runs.
pub trait CommandExecutor {
    /// Run a simple shell command (e.g., "pkill -USR1 waybar")
    ///
    /// # Errors
    ///
    /// Returns an error if the command cannot be spawned or exits non-zero.
    fn run_command(&self, command: &str) -> Result<()>;

    /// Run an external script with arguments and environment variables
    ///
    /// # Errors
    ///
    /// Returns an error if the script cannot be spawned or exits non-zero.
    fn run_script(&self, path: &Path, args: &[String], env: &HashMap<String, String>)
    -> Result<()>;
}

/// Abstract interface for File I/O.
/// Essential for implementing "--dry-run" safely.
pub trait FileSystem {
    /// Read the entire contents of a file into a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    fn read_to_string(&self, path: &Path) -> Result<String>;
    /// Write `content` to `path`, creating parent directories as needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the file (or its parents) cannot be written.
    fn write_all(&self, path: &Path, content: &str) -> Result<()>;
    /// Recursively create a directory and all missing parents.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    /// Create a symlink at `target` pointing to `source`, replacing any
    /// existing file at `target`.
    ///
    /// # Errors
    ///
    /// Returns an error if the symlink cannot be created.
    fn create_symlink(&self, source: &Path, target: &Path) -> Result<()>;
    /// Returns `true` if `path` exists.
    fn exists(&self, path: &Path) -> bool;
    /// Returns `true` if `path` exists and is a regular file.
    fn is_file(&self, path: &Path) -> bool;
}
