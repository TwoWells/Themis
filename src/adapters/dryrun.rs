//! Dry-run adapters for previewing changes without side effects.
//!
//! These adapters implement the same traits as the real adapters but
//! only log what would happen instead of actually performing I/O.
//!
//! # Usage
//!
//! Use these adapters with the `--dry-run` flag to preview theme changes:
//!
//! ```text
//! $ themis load nord --dry-run
//! INFO [dry-run] Would write to "/home/user/.config/kitty/.themis.conf":
//! INFO [dry-run]   foreground #eceff4
//! INFO [dry-run]   background #2e3440
//! INFO [dry-run] Would run: pkill -USR1 kitty
//! ```
//!
//! # Behavior
//!
//! - `DryRunFileSystem`: Reads real files (needed for templates) but only logs writes
//! - `DryRunCommandExecutor`: Logs commands and scripts instead of executing them

use crate::core::traits::{CommandExecutor, FileSystem};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::info;

/// A FileSystem adapter that reads real files but only logs writes.
///
/// This allows the orchestrator to read config files and templates
/// while previewing what would be written without actually modifying
/// the filesystem.
pub struct DryRunFileSystem;

impl FileSystem for DryRunFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        // Actually read - we need real config/template content
        Ok(fs::read_to_string(path)?)
    }

    fn write_all(&self, path: &Path, content: &str) -> Result<()> {
        info!("[dry-run] Would write to {:?}:", path);
        // Show a preview of the content (first few lines)
        for line in content.lines().take(10) {
            info!("[dry-run]   {}", line);
        }
        if content.lines().count() > 10 {
            info!(
                "[dry-run]   ... ({} more lines)",
                content.lines().count() - 10
            );
        }
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        info!("[dry-run] Would create directory {:?}", path);
        Ok(())
    }

    fn create_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        info!("[dry-run] Would symlink {:?} -> {:?}", target, source);
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }
}

/// A CommandExecutor that logs commands instead of running them.
///
/// Shell commands and scripts are logged with their arguments and
/// environment variables, allowing users to preview the exact
/// operations that would be performed.
pub struct DryRunCommandExecutor;

impl CommandExecutor for DryRunCommandExecutor {
    fn run_command(&self, command: &str) -> Result<()> {
        info!("[dry-run] Would run: {}", command);
        Ok(())
    }

    fn run_script(
        &self,
        path: &Path,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<()> {
        info!("[dry-run] Would run script: {:?}", path);
        if !args.is_empty() {
            info!("[dry-run]   args: {:?}", args);
        }
        if !env.is_empty() {
            info!("[dry-run]   env: {:?}", env);
        }
        Ok(())
    }
}
