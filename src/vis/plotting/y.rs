use ndarray::Array1;
use std::{error::Error, path::Path};
use tracing::trace;

use crate::vis::plotting::xy::xy_plot;

/// Generates a standard y plot from the provided y values.
///
/// Plots the y values against their index. Saves the plot to the provided path
/// as a PNG image. Applies the provided title, axis labels, etc.
///
/// Returns the plot data as a `Vec<u8>`, or an error if the plot could not be
/// generated.
#[tracing::instrument(level = "trace")]
pub fn standard_y_plot(
    y: &Array1<f32>,
    path: &Path,
    title: &str,
    y_label: &str,
    x_label: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    trace!("Generating y plot.");
    xy_plot(
        None,
        y,
        Some(path),
        Some(title),
        Some(y_label),
        Some(x_label),
        None,
    )
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use super::*;

    const COMMON_PATH: &str = "tests/vis/plotting/y";

    fn setup() {
        if !Path::new(COMMON_PATH).exists() {
            std::fs::create_dir_all(COMMON_PATH).unwrap();
        }
    }

    fn clean(files: &Vec<PathBuf>) {
        for file in files {
            if file.is_file() {
                std::fs::remove_file(file).unwrap();
            }
        }
    }

    #[test]
    fn test_standard_y_plot_basic() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_y_plot_basic.png")];
        clean(&files);

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        standard_y_plot(&y, files[0].as_path(), "Test Plot", "Y", "X").unwrap();

        assert!(files[0].is_file());
    }
    #[test]
    fn test_standard_y_plot_empty() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_y_plot_empty.png")];
        clean(&files);

        let y = Array1::from_vec(vec![]);

        let result = standard_y_plot(&y, files[0].as_path(), "Test Plot", "Y", "X");

        assert!(result.is_err());
        assert!(!files[0].is_file());
    }

    #[test]
    fn test_standard_y_plot_invalid_path() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("invalid/test_y_plot_invalid.png")];
        clean(&files);

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let result = standard_y_plot(&y, files[0].as_path(), "Test Plot", "Y", "X");

        assert!(result.is_err());
        assert!(!files[0].exists());
    }
}
