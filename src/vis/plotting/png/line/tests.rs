use std::path::Path;

use anyhow::Context;
use ndarray::Array1;

use super::*;
use crate::{
    tests::{clean_files, setup_folder},
    vis::plotting::STANDARD_RESOLUTION,
};

const COMMON_PATH: &str = "tests/vis/plotting/png/line";

#[test]
fn test_line_plot() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf()).context("Failed to setup test folder for line plot test")?;
    let files = vec![path.join("line_plot.png")];
    clean_files(&files).context("Failed to clean test files for line plot test")?;

    let x = Array1::linspace(0.0, 10.0, 100);
    let y = x.map(|x| x * x);
    line_plot(
        Some(&x),
        vec![&y],
        Some(files[0].as_path()),
        Some("y=x^2"),
        Some("x [a.u.]"),
        Some("y [a.u.]"),
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
fn test_log_y_plot() -> anyhow::Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf()).context("Failed to setup test folder for log y plot test")?;
    let files = vec![path.join("log_y_plot.png")];
    clean_files(&files).context("Failed to clean test files for log y plot test")?;

    let x = Array1::linspace(1.0, 10.0, 100);
    let y = x.map(|x| x * x);
    log_y_plot(
        Some(&x),
        vec![&y],
        Some(files[0].as_path()),
        Some("y=x^2"),
        Some("x [a.u.]"),
        Some("y [a.u.]"),
        None,
        None,
    )?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
fn test_line_plot_defaults() -> anyhow::Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for line plot defaults test")?;
    let files = vec![path.join("line_plot_default.png")];
    clean_files(&files).context("Failed to clean test files for line plot defaults test")?;

    let x = Array1::linspace(0.0, 10.0, 100);
    let y = x.map(|x| x * x);
    line_plot(
        None,
        vec![&y],
        Some(files[0].as_path()),
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
fn test_line_plot_no_path() -> anyhow::Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for line plot no path test")?;
    let files = vec![path.join("line_plot_no_path.png")];
    clean_files(&files).context("Failed to clean test files for line plot no path test")?;

    let x = Array1::linspace(0.0, 10.0, 100);
    let y = x.map(|x| x * x);
    line_plot(None, vec![&y], None, None, None, None, None, None)?;

    assert!(!files[0].is_file());
    Ok(())
}

#[test]
fn test_line_plot_default_resolution() -> anyhow::Result<()> {
    let x = Array1::linspace(0.0, 10.0, 100);
    let y = x.map(|x| x * x);

    let bundle = line_plot(None, vec![&y], None, None, None, None, None, None)?;

    assert_eq!(
        bundle.data.len(),
        STANDARD_RESOLUTION.0 as usize * STANDARD_RESOLUTION.1 as usize * 3
    );
    Ok(())
}

#[test]
fn test_line_plot_custom_resolution() -> Result<()> {
    let x = Array1::linspace(0.0, 10.0, 100);
    let y = x.map(|x| x * x);

    let resolution = (400, 300);

    let bundle = line_plot(
        None,
        vec![&y],
        None,
        None,
        None,
        None,
        None,
        Some(resolution),
    )
    .context("Failed to generate line plot with custom resolution")?;

    assert_eq!(
        bundle.data.len(),
        resolution.0 as usize * resolution.1 as usize * 3
    );
    Ok(())
}

#[test]
fn test_line_plot_incompatible_x_y() {
    let x = Array1::linspace(0.0, 10.0, 100);
    let y = Array1::zeros(90);

    assert!(line_plot(Some(&x), vec![&y], None, None, None, None, None, None).is_err());
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_line_plot_multiple_y() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for multiple y series test")?;
    let files = vec![path.join("line_plot_multiple_y.png")];
    clean_files(&files).context("Failed to clean test files for multiple y series test")?;

    let x = Array1::linspace(0.0, 10.0, 100);
    let ys_owned: Vec<Array1<f32>> = (0..10).map(|i| x.map(|x| x * x * i as f32)).collect();
    let ys: Vec<&Array1<f32>> = ys_owned.iter().collect();
    line_plot(
        Some(&x),
        ys,
        Some(files[0].as_path()),
        Some("y=x^2"),
        Some("x [a.u.]"),
        Some("y [a.u.]"),
        None,
        None,
    )
    .context("Failed to generate line plot with multiple y series")?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_line_plot_with_labels() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for line plot with labels test")?;
    let files = vec![path.join("line_plot_with_lables.png")];
    clean_files(&files).context("Failed to clean test files for line plot with labels test")?;

    let x = Array1::linspace(0.0, 10.0, 100);
    let ys_owned: Vec<Array1<f32>> = (0..10).map(|i| x.map(|x| x * x * i as f32)).collect();
    let ys: Vec<&Array1<f32>> = ys_owned.iter().collect();
    let labels_owned: Vec<String> = (0..10).map(|i| format!("y_{i}")).collect();
    let labels: Vec<&str> = labels_owned
        .iter()
        .map(std::string::String::as_str)
        .collect();

    line_plot(
        Some(&x),
        ys,
        Some(files[0].as_path()),
        Some("y=x^2"),
        Some("x [a.u.]"),
        Some("y [a.u.]"),
        Some(&labels),
        None,
    )
    .context("Failed to generate line plot with series labels")?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_line_plot_with_invalid_labels() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for invalid labels test")?;
    let files = vec![path.join("line_plot_with_invalid_lables.png")];
    clean_files(&files).context("Failed to clean test files for invalid labels test")?;

    let x = Array1::linspace(0.0, 10.0, 100);
    let ys_owned: Vec<Array1<f32>> = (0..10).map(|i| x.map(|x| x * x * i as f32)).collect();
    let ys: Vec<&Array1<f32>> = ys_owned.iter().collect();
    let labels_owned: Vec<String> = (0..9).map(|i| format!("y_{i}")).collect();
    let labels: Vec<&str> = labels_owned
        .iter()
        .map(std::string::String::as_str)
        .collect();

    let result = line_plot(
        Some(&x),
        ys,
        Some(files[0].as_path()),
        Some("y=x^2"),
        Some("x [a.u.]"),
        Some("y [a.u.]"),
        Some(&labels),
        None,
    );

    assert!(result.is_err());
    assert!(!files[0].is_file());
    Ok(())
}

#[test]
fn test_standard_y_plot_basic() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for standard y plot test")?;
    let files = vec![path.join("y_plot_basic.png")];
    clean_files(&files).context("Failed to clean test files for standard y plot test")?;

    let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

    standard_y_plot(&y, Some(files[0].as_path()), "Test Plot", "Y", "X")
        .context("Failed to generate standard y plot")?;

    assert!(files[0].is_file());
    Ok(())
}
#[test]
fn test_standard_y_plot_empty() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for empty y plot test")?;
    let files = vec![path.join("y_plot_empty.png")];
    clean_files(&files).context("Failed to clean test files for empty y plot test")?;

    let y = Array1::from_vec(vec![]);

    let result = standard_y_plot(&y, Some(files[0].as_path()), "Test Plot", "Y", "X");

    assert!(result.is_err());
    assert!(!files[0].is_file());
    Ok(())
}

#[test]
fn test_standard_y_plot_invalid_path() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for invalid path test")?;
    let files = vec![path.join("invalid/y_plot_invalid.png")];
    clean_files(&files).context("Failed to clean test files for invalid path test")?;

    let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

    let result = standard_y_plot(&y, Some(files[0].as_path()), "Test Plot", "Y", "X");

    assert!(result.is_err());
    assert!(!files[0].exists());
    Ok(())
}

#[test]
fn test_standard_time_plot_normal() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for standard time plot test")?;
    let files = vec![path.join("time_plot_normal.png")];
    clean_files(&files).context("Failed to clean test files for standard time plot test")?;

    let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

    let sample_rate_hz = 1.0;

    let title = "Test Plot";
    let y_label = "Y Label";

    standard_time_plot(&y, sample_rate_hz, Some(files[0].as_path()), title, y_label)
        .context("Failed to generate standard time plot")?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
fn test_standard_time_plot_zero_sample_rate() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())
        .context("Failed to setup test folder for zero sample rate test")?;
    let files = vec![path.join("time_plot_zero_sample_rate.png")];
    clean_files(&files).context("Failed to clean test files for zero sample rate test")?;

    let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

    let sample_rate_hz = 0.0;

    let title = "Test Plot";
    let y_label = "Y Label";

    let result = standard_time_plot(&y, sample_rate_hz, Some(files[0].as_path()), title, y_label);

    assert!(result.is_err());
    assert!(!files[0].is_file());
    Ok(())
}

#[test]
fn test_standard_time_plot_negative_sample_rate() -> Result<()> {
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("time_plot_negative_sample_rate.png")];
    clean_files(&files)?;

    let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

    let sample_rate_hz = -1.0;

    let title = "Test Plot";
    let y_label = "Y Label";

    let result = standard_time_plot(&y, sample_rate_hz, Some(files[0].as_path()), title, y_label);

    assert!(result.is_err());
    assert!(!files[0].is_file());
    Ok(())
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_xyz_state_plot_basic() -> Result<()> {
    use crate::core::data::shapes::SystemStates;
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("xyz_plot_basic.png")];
    clean_files(&files)?;

    let mut system_states = SystemStates::empty(100, 6);

    for i in 0..100 {
        for j in 0..6 {
            system_states[(i, j)] = i as f32 * j as f32;
        }
    }

    let title = "Test Plot";
    let sample_rate_hz = 10.0;

    plot_state_xyz(&system_states, 1, sample_rate_hz, Some(files[0].as_path()), title)
        .context("Failed to create XYZ state plot")?;

    assert!(files[0].is_file());
    Ok(())
}

#[test]
fn test_xyz_state_plot_invalid_index() -> Result<()> {
    use crate::core::data::shapes::SystemStates;
    let path = Path::new(COMMON_PATH);
    setup_folder(path.to_path_buf())?;
    let files = vec![path.join("xyz_plot_invalid.png")];
    clean_files(&files)?;

    let system_states = SystemStates::empty(100, 6);
    let title = "Test Plot";
    let sample_rate_hz = 10.0;

    let results = plot_state_xyz(&system_states, 5, sample_rate_hz, Some(files[0].as_path()), title);

    assert!(results.is_err());
    assert!(!files[0].is_file());
    Ok(())
}
