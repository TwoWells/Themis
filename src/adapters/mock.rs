#![allow(dead_code, reason = "mocks might have unused methods in some tests")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test-only mocks lock infallible mutexes via unwrap"
)]

use crate::core::traits::{CommandExecutor, FileSystem, TemplateRenderer};
use anyhow::{Result, bail};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// --- Mock File System ---
#[derive(Clone, Default)]
pub struct MockFileSystem {
    pub files: Arc<Mutex<HashMap<PathBuf, String>>>,
    pub symlinks: Arc<Mutex<HashMap<PathBuf, PathBuf>>>,
}

impl MockFileSystem {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&self, path: impl AsRef<Path>, content: &str) {
        self.files
            .lock()
            .unwrap()
            .insert(path.as_ref().to_path_buf(), content.to_string());
    }
}

impl FileSystem for MockFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        let files = self.files.lock().unwrap();
        match files.get(path) {
            Some(content) => Ok(content.clone()),
            None => bail!("MockFS: File not found: {}", path.display()),
        }
    }

    fn write_all(&self, path: &Path, content: &str) -> Result<()> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), content.to_string());
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn create_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        self.symlinks
            .lock()
            .unwrap()
            .insert(target.to_path_buf(), source.to_path_buf());
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.exists(path)
    }
}

// --- Mock Template Renderer ---
#[derive(Clone, Default)]
pub struct MockTemplateRenderer;

impl TemplateRenderer for MockTemplateRenderer {
    fn render(&self, template: &str, context: &HashMap<String, Value>) -> Result<String> {
        // Simple "fake" renderer that just replaces {{ key }} with value string
        // Sufficient for unit testing logic flow, not engine correctness.
        let mut result = template.to_string();
        for (k, v) in context {
            let placeholder = format!("{{{{ {k} }}}}");
            let replacement = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => continue,
            };
            result = result.replace(&placeholder, &replacement);
        }
        Ok(result)
    }
}

// --- Mock Command Executor ---
#[derive(Clone, Default)]
pub struct MockCommandExecutor {
    pub executed: Arc<Mutex<Vec<String>>>,
    pub script_env: Arc<Mutex<HashMap<String, String>>>,
}

impl CommandExecutor for MockCommandExecutor {
    fn run_command(&self, command: &str) -> Result<()> {
        self.executed.lock().unwrap().push(command.to_string());
        Ok(())
    }

    fn run_script(
        &self,
        path: &Path,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<()> {
        let cmd = format!("{} {}", path.display(), args.join(" "));
        self.executed.lock().unwrap().push(cmd);
        // Capture env vars for testing
        self.script_env.lock().unwrap().extend(env.clone());
        Ok(())
    }
}
