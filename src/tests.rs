use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Creates a directory at the specified path.
///
/// # Errors
///
/// Returns an error if the directory cannot be created.
#[tracing::instrument(level = "trace")]
pub fn setup_folder<P>(path: P) -> Result<()>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    std::fs::create_dir_all(&path).context("Failed to create test directory")?;
    Ok(())
}

/// Removes all files in the provided vector.
///
/// # Errors
///
/// Returns an error if any file cannot be removed.
#[tracing::instrument(level = "trace")]
pub fn clean_files(files: &Vec<PathBuf>) -> Result<()> {
    for file in files {
        if file.is_file() {
            std::fs::remove_file(file).with_context(|| format!("Failed to remove test file: {}", file.display()))?;
        }
    }
    Ok(())
}
