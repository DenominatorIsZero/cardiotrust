use std::path::PathBuf;

#[tracing::instrument(level = "trace")]
pub fn setup_folder(path: PathBuf) {
    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}

#[tracing::instrument(level = "trace")]
pub fn clean_files(files: &Vec<PathBuf>) {
    for file in files {
        if file.is_file() {
            std::fs::remove_file(file).unwrap();
        }
    }
}
