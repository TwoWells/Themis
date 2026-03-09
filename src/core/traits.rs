use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Abstract interface for rendering text templates.
/// This isolates the core from specific engines like Tera or Handlebars.
pub trait TemplateRenderer {
    /// Render a string template with the given context.
    fn render(&self, template_content: &str, context: &HashMap<String, Value>) -> Result<String>;
}

/// Abstract interface for executing system commands.
/// This allows for mocking command execution during tests or dry-runs.
pub trait CommandExecutor {
    /// Run a simple shell command (e.g., "pkill -USR1 waybar")
    fn run_command(&self, command: &str) -> Result<()>;

    /// Run an external script with arguments and environment variables
    fn run_script(&self, path: &Path, args: &[String], env: &HashMap<String, String>)
    -> Result<()>;
}

/// Abstract interface for File I/O.
/// Essential for implementing "--dry-run" safely.
pub trait FileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String>;
    fn write_all(&self, path: &Path, content: &str) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn create_symlink(&self, source: &Path, target: &Path) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
}
