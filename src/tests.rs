use std::path::{Path, PathBuf};

#[tracing::instrument(level = "trace")]
pub fn setup_folder<P>(path: P)
where
    P: AsRef<Path> + std::fmt::Debug,
{
    std::fs::create_dir_all(path).unwrap();
}

#[tracing::instrument(level = "trace")]
pub fn clean_files(files: &Vec<PathBuf>) {
    for file in files {
        if file.is_file() {
            std::fs::remove_file(file).unwrap();
        }
    }
}
