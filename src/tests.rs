use std::path::{Path, PathBuf};

/// Creates a directory at the specified path.
///
/// # Panics
///
/// Panics if the directory cannot be created.
#[tracing::instrument(level = "trace")]
pub fn setup_folder<P>(path: P)
where
    P: AsRef<Path> + std::fmt::Debug,
{
    std::fs::create_dir_all(path).unwrap();
}

/// Removes all files in the provided vector.
///
/// # Panics
///
/// Panics if any file cannot be removed.
#[tracing::instrument(level = "trace")]
pub fn clean_files(files: &Vec<PathBuf>) {
    for file in files {
        if file.is_file() {
            std::fs::remove_file(file).unwrap();
        }
    }
}
