use crate::vis::plotting::y::standard_y_plot;
use ndarray::Array1;
use std::{fs, path::PathBuf};
use tempfile::TempDir;


#[test]
fn test_standard_y_plot_invalid_path() {
    let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);
    let path = PathBuf::from("/invalid/path/test.png");

    let result = standard_y_plot(&y, &path, "Test Plot", "Y", "X");

    assert!(result.is_err());
    assert!(!path.exists());
}
