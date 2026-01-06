use crate::core::traits::FileSystem;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::os::unix::fs::symlink;

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))
    }

    fn write_all(&self, path: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory for {:?}", path))?;
        }
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write file: {:?}", path))
    }
    
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {:?}", path))
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

        symlink(source, target)
            .with_context(|| format!("Failed to symlink {:?} -> {:?}", source, target))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }
}
