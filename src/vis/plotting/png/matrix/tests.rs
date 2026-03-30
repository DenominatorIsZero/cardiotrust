use std::path::Path;

use ndarray::Array2;

use super::*;
use crate::tests::{clean_files, setup_folder};

const COMMON_PATH: &str = "tests/vis/plotting/png/matrix";

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_high() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_high.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((4, 8));

    for x in 0..4 {
        for y in 0..8 {
            data[(x, y)] = ((x + 1) + (y * 4)) as f32;
        }
    }

    matrix_plot(
        &data,
        None,
        None,
        None,
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_wide() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_wide.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((8, 4));

    for x in 0..8 {
        for y in 0..4 {
            data[(x, y)] = ((x + 1) + (y * 8)) as f32;
        }
    }

    matrix_plot(
        &data,
        None,
        None,
        None,
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_single_row() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_single_row.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((8, 1));

    for x in 0..8 {
        for y in 0..1 {
            data[(x, y)] = ((x + 1) + (y * 8)) as f32;
        }
    }

    matrix_plot(
        &data,
        None,
        None,
        None,
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_single_column() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_single_column.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((1, 8));

    for x in 0..1 {
        for y in 0..8 {
            data[(x, y)] = ((x + 1) + (y * 8)) as f32;
        }
    }

    matrix_plot(
        &data,
        None,
        None,
        None,
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_large() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_large.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((1000, 1000));

    for x in 0..1000 {
        for y in 0..1000 {
            data[(x, y)] = ((x + 1) + (y * 1000)) as f32;
        }
    }

    matrix_plot(
        &data,
        None,
        None,
        None,
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_custom_labels() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_custom_lables.png")];
    clean_files(&files)?;

    let data = Array2::zeros((4, 4));

    matrix_plot(
        &data,
        None,
        None,
        None,
        Some(files[0].as_path()),
        Some("Custom Title"),
        Some("Custom X"),
        Some("Custom Y"),
        Some("Custom Unit"),
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_custom_range() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_custom_range.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((4, 4));
    data[(0, 0)] = 5.0;

    matrix_plot(
        &data,
        Some((0.0, 10.0)),
        None,
        None,
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_custom_step() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_custom_step.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((4, 4));
    data[(0, 0)] = 5.0;

    matrix_plot(
        &data,
        None,
        Some((0.25, 0.25)),
        None,
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_custom_offset() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_custom_offset.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((4, 4));
    data[(0, 0)] = 5.0;

    matrix_plot(
        &data,
        None,
        None,
        Some((10.0, 100.0)),
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_matrix_plot_invalid_step() -> anyhow::Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("matrix_plot_invalid_step.png")];
    clean_files(&files)?;

    let mut data = Array2::zeros((4, 4));
    data[(0, 0)] = 5.0;

    let results = matrix_plot(
        &data,
        None,
        Some((0.0, 1.0)),
        None,
        Some(files[0].as_path()),
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert!(results.is_err());
    assert!(!files[0].is_file());
    Ok(())
}
