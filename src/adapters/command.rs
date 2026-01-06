use crate::core::traits::CommandExecutor;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub struct RealCommandExecutor;

impl CommandExecutor for RealCommandExecutor {
    fn run_command(&self, command: &str) -> Result<()> {
        // Run via sh -c to allow piping and shell features
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .with_context(|| format!("Failed to spawn shell command: {}", command))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}\nStderr: {}", command, stderr);
        }

        Ok(())
    }

    fn run_script(
        &self,
        path: &Path,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<()> {
        let mut cmd = Command::new(path);
        
        cmd.args(args);
        cmd.envs(env);

        let output = cmd.output()
            .with_context(|| format!("Failed to spawn script: {:?}", path))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Script failed: {:?}\nStderr: {}", path, stderr);
        }

        Ok(())
    }
}
