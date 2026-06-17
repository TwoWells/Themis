// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! Real filesystem I/O: reads, writes, directory creation, and symlinks.
use crate::core::traits::FileSystem;
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

/// Performs real filesystem operations, including forced symlink creation.
pub struct RealFileSystem;

// Mutation-testing note: cargo-mutants leaves the std-delegating leaf methods
// here as documented survivors — `exists`/`is_file` (one-line wrappers over
// `Path::exists`/`Path::is_file`) and `create_dir_all` (a trait method nothing
// in the orchestrator path calls; `write_all`/`create_symlink` create parent
// dirs via `std::fs` directly). The branching logic that consumes these — the
// orchestrator's user-then-system palette fallback and template-presence check —
// is pinned through MockFileSystem unit tests (no orchestrator mutants survive),
// so mutating a leaf delegation only flips a value no assertion can observe
// without re-testing `std::fs` itself.
impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
    }

    fn write_all(&self, path: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directory for {}", path.display())
            })?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))
    }

    fn create_symlink(&self, source: &Path, target: &Path) -> Result<()> {
        // Remove existing target if it exists (force link)
        if target.exists() || target.is_symlink() {
            fs::remove_file(target).ok(); // Ignore error if it doesn't exist
        }

        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        symlink(source, target).with_context(|| {
            format!(
                "Failed to symlink {} -> {}",
                source.display(),
                target.display()
            )
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }
}
