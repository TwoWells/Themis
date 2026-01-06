use crate::core::traits::{CommandExecutor, FileSystem};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::info;

/// A FileSystem adapter that reads real files but only logs writes.
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
