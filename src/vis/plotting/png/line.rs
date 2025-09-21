use std::{io, path::Path};

use anyhow::{Context, Result};

use ndarray::{s, Array1, ArrayBase, Data, Ix1};
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
use tracing::trace;

use super::PngBundle;
use crate::{
    core::data::shapes::SystemStates,
    vis::plotting::{
        allocate_buffer, AXIS_LABEL_AREA, AXIS_STYLE, CAPTION_STYLE, CHART_MARGIN, COLORS,
        LEGEND_OPACITY, LEGEND_PATH_LENGTH, STANDARD_RESOLUTION, X_MARGIN, Y_MARGIN,
    },
};

/// Generates an XY plot from the provided x and y data.
///
/// Saves the plot to the optionally provided path as a PNG,
/// returns the raw pixel buffer.
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
pub fn line_plot<A>(
    x: Option<&Array1<f32>>,
    ys: Vec<&ArrayBase<A, Ix1>>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    item_labels: Option<&Vec<&str>>,
    resolution: Option<(u32, u32)>,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating xy plot.");

    let (width, height) = resolution.unwrap_or(STANDARD_RESOLUTION);

    let mut buffer = allocate_buffer(width, height);

    let y_len = ys[0].len();

    for y in &ys {
        if y.len() != y_len {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "y data must have same length",
            ).into());
        }
    }

    if let Some(item_labels) = item_labels {
        if item_labels.len() != ys.len() {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "if not None, item_labels must be same length as ys",
            ).into());
        }
    }

    let default_x;
    let x = match x {
        Some(x) => x,
        None => {
            default_x = Array1::linspace(0.0, y_len as f32, y_len);
            &default_x
        }
    };

    if x.len() != y_len {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "x and y must have same length",
        ).into());
    }

    let title = title.unwrap_or("Plot");
    let y_label = y_label.unwrap_or("y");
    let x_label = x_label.unwrap_or("x");

    let x_min = x.min()?;
    let x_max = x.max()?;
    let mut y_min = f32::INFINITY;
    let mut y_max = -f32::INFINITY;

    for y in &ys {
        let min = y.min()?;
        let max = y.max()?;
        y_min = y_min.min(*min);
        y_max = y_max.max(*max);
    }

    let x_range = x_max - x_min;
    let y_range = y_max - y_min;

    let x_min = x_min - x_range * X_MARGIN;
    let x_max = x_max + x_range * X_MARGIN;
    let y_min = y_range.mul_add(-Y_MARGIN, y_min);
    let y_max = y_range.mul_add(Y_MARGIN, y_max);

    {
        let root = BitMapBackend::with_buffer(&mut buffer[..], (width, height)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, CAPTION_STYLE.into_font())
            .margin(CHART_MARGIN)
            .x_label_area_size(AXIS_LABEL_AREA)
            .y_label_area_size(AXIS_LABEL_AREA)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc(x_label)
            .x_label_style(AXIS_STYLE.into_font())
            .y_desc(y_label)
            .y_label_style(AXIS_STYLE.into_font())
            .draw()?;

        for (i, y) in ys.iter().enumerate() {
            let color = &COLORS[i % COLORS.len()];
            if let Some(item_labels) = item_labels {
                chart
                    .draw_series(LineSeries::new(
                        x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                        color,
                    ))?
                    .label(item_labels[i])
                    .legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + LEGEND_PATH_LENGTH, y)], color)
                    });
            } else {
                chart.draw_series(LineSeries::new(
                    x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                    color,
                ))?;
            }
        }

        if item_labels.is_some() {
            chart
                .configure_series_labels()
                .background_style(WHITE.mix(LEGEND_OPACITY))
                .border_style(BLACK)
                .label_font(AXIS_STYLE.into_font())
                .draw()?;
        }

        root.present()?;
    } // dropping bitmap backend

    if let Some(path) = path {
        image::save_buffer_with_format(
            path,
            &buffer,
            width,
            height,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )?;
    }

    Ok(PngBundle {
        data: buffer,
        width,
        height,
    })
}

#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
pub fn log_y_plot<A>(
    x: Option<&Array1<f32>>,
    ys: Vec<&ArrayBase<A, Ix1>>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    item_labels: Option<&Vec<&str>>,
    resolution: Option<(u32, u32)>,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating xy plot.");

    let (width, height) = resolution.unwrap_or(STANDARD_RESOLUTION);

    let mut buffer = allocate_buffer(width, height);

    let y_len = ys[0].len();

    for y in &ys {
        if y.len() != y_len {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "y data must have same length",
            ).into());
        }
    }

    if let Some(item_labels) = item_labels {
        if item_labels.len() != ys.len() {
            return Err(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "if not None, item_labels must be same length as ys",
            ).into());
        }
    }

    let default_x;
    let x = match x {
        Some(x) => x,
        None => {
            default_x = Array1::linspace(0.0, y_len as f32, y_len);
            &default_x
        }
    };

    if x.len() != y_len {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "x and y must have same length",
        ).into());
    }

    let title = title.unwrap_or("Plot");
    let y_label = y_label.unwrap_or("y");
    let x_label = x_label.unwrap_or("x");

    let x_min = x.min()?;
    let x_max = x.max()?;
    let mut y_min = f32::INFINITY;
    let mut y_max = -f32::INFINITY;

    for y in &ys {
        let min = y.min()?;
        let max = y.max()?;
        y_min = y_min.min(*min);
        y_max = y_max.max(*max);
    }

    let x_range = x_max - x_min;

    let x_min = x_min - x_range * X_MARGIN;
    let x_max = x_max + x_range * X_MARGIN;
    let y_min = (y_min * 0.1).max(1e-20);
    let y_max = y_max * 10.0;

    {
        let root = BitMapBackend::with_buffer(&mut buffer[..], (width, height)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(title, CAPTION_STYLE.into_font())
            .margin(CHART_MARGIN)
            .x_label_area_size(AXIS_LABEL_AREA)
            .y_label_area_size(AXIS_LABEL_AREA)
            .build_cartesian_2d(x_min..x_max, (y_min..y_max).log_scale())?;

        chart
            .configure_mesh()
            .x_desc(x_label)
            .x_label_style(AXIS_STYLE.into_font())
            .y_desc(y_label)
            .y_label_style(AXIS_STYLE.into_font())
            .y_label_formatter(&|y| format!("{y:e}"))
            .draw()?;

        for (i, y) in ys.iter().enumerate() {
            let color = &COLORS[i % COLORS.len()];
            if let Some(item_labels) = item_labels {
                chart
                    .draw_series(LineSeries::new(
                        x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                        color,
                    ))?
                    .label(item_labels[i])
                    .legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + LEGEND_PATH_LENGTH, y)], color)
                    });
            } else {
                chart.draw_series(LineSeries::new(
                    x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                    color,
                ))?;
            }
        }

        if item_labels.is_some() {
            chart
                .configure_series_labels()
                .background_style(WHITE.mix(LEGEND_OPACITY))
                .border_style(BLACK)
                .label_font(AXIS_STYLE.into_font())
                .draw()?;
        }

        root.present()?;
    } // dropping bitmap backend

    if let Some(path) = path {
        image::save_buffer_with_format(
            path,
            &buffer,
            width,
            height,
            image::ColorType::Rgb8,
            image::ImageFormat::Png,
        )?;
    }

    Ok(PngBundle {
        data: buffer,
        width,
        height,
    })
}

/// Generates a standard y plot from the provided y values.
///
/// Plots the y values against their index. Saves the plot to the provided path
/// as a PNG image. Applies the provided title, axis labels, etc.
///
/// Returns the plot data as a `Vec<u8>`, or an error if the plot could not be
/// generated.
#[tracing::instrument(level = "trace")]
pub fn standard_y_plot<A>(
    y: &ArrayBase<A, Ix1>,
    path: &Path,
    title: &str,
    y_label: &str,
    x_label: &str,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating y plot.");
    line_plot(
        None,
        vec![y],
        Some(path),
        Some(title),
        Some(y_label),
        Some(x_label),
        None,
        None,
    )
}

#[tracing::instrument(level = "trace")]
pub fn standard_log_y_plot<A>(
    y: &ArrayBase<A, Ix1>,
    path: &Path,
    title: &str,
    y_label: &str,
    x_label: &str,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating y plot.");
    log_y_plot(
        None,
        vec![y],
        Some(path),
        Some(title),
        Some(y_label),
        Some(x_label),
        None,
        None,
    )
}

/// Generates a standard time plot from the provided y values and sample rate.
///
/// Plots the y values against time in seconds based on the provided sample rate.
/// Saves the plot to the provided path as a PNG image. Applies the provided
/// title and axis labels.
///
/// Returns the plot data as a `Vec<u8>`, or an error if the plot could not be
/// generated.
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn standard_time_plot<A>(
    y: &ArrayBase<A, Ix1>,
    sample_rate_hz: f32,
    path: &Path,
    title: &str,
    y_label: &str,
) -> Result<PngBundle>
where
    A: Data<Elem = f32>,
{
    trace!("Generating time plot.");
    if sample_rate_hz <= 0.0 {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "sample_rate_hz must be greater than zero",
        ).into());
    }
    let x = Array1::linspace(0.0, y.len() as f32 / sample_rate_hz, y.len());
    line_plot(
        Some(&x),
        vec![y],
        Some(path),
        Some(title),
        Some(y_label),
        Some("t [s]"),
        None,
        None,
    )
}

/// Generates a plot of the x, y, and z values for a specific state index from
/// the provided system state data.
///
/// Plots the x, y, and z values for the state at the given index against time
/// in seconds based on the provided sample rate. Saves the plot to the provided
/// path as a PNG image. Applies the provided title and axis labels.
///
/// `system_states` - The system state data to extract values from.
/// `state_index` - The index of the state to plot.
/// `sample_rate_hz` - The sample rate of the data in Hz.  
/// `path` - The path to save the generated plot to.
/// `title` - The title for the plot.
///
/// Returns the plot data as a `Vec<u8>`, or an error if the plot could not be
/// generated.
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn plot_state_xyz(
    system_states: &SystemStates,
    state_index: usize,
    sample_rate_hz: f32,
    path: &Path,
    title: &str,
) -> Result<PngBundle> {
    trace!("Generating state xyz plot.");

    if state_index >= (system_states.num_states() - 2) {
        return Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "state_index out of bounds",
        ).into());
    }

    let state_x = system_states.slice(s![.., state_index]);
    let state_y = system_states.slice(s![.., state_index + 1]);
    let state_z = system_states.slice(s![.., state_index + 2]);
    let x = Array1::linspace(0.0, state_x.len() as f32 / sample_rate_hz, state_x.len());
    let y = vec![&state_x, &state_y, &state_z];
    let labels: Vec<&str> = vec!["x", "y", "z"];
    let title = format!("{title} - State Index: {state_index}");
    line_plot(
        Some(&x),
        y,
        Some(path),
        Some(title.as_str()),
        Some("j [A/mm^2]"),
        Some("t [s]"),
        Some(&labels),
        None,
    )
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::tests::{clean_files, setup_folder};
    const COMMON_PATH: &str = "tests/vis/plotting/png/line";

    #[test]
    fn test_line_plot() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())
            .context("Failed to setup test folder for line plot test")?;
        let files = vec![path.join("line_plot.png")];
        clean_files(&files)
            .context("Failed to clean test files for line plot test")?;

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
        setup_folder(path.to_path_buf())
            .context("Failed to setup test folder for log y plot test")?;
        let files = vec![path.join("log_y_plot.png")];
        clean_files(&files)
            .context("Failed to clean test files for log y plot test")?;

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
        clean_files(&files)
            .context("Failed to clean test files for line plot defaults test")?;

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
        clean_files(&files)
            .context("Failed to clean test files for line plot no path test")?;

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
        clean_files(&files)
            .context("Failed to clean test files for multiple y series test")?;

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
        clean_files(&files)
            .context("Failed to clean test files for line plot with labels test")?;

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
        clean_files(&files)
            .context("Failed to clean test files for invalid labels test")?;

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
        clean_files(&files)
            .context("Failed to clean test files for standard y plot test")?;

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        standard_y_plot(&y, files[0].as_path(), "Test Plot", "Y", "X")
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
        clean_files(&files)
            .context("Failed to clean test files for empty y plot test")?;

        let y = Array1::from_vec(vec![]);

        let result = standard_y_plot(&y, files[0].as_path(), "Test Plot", "Y", "X");

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
        clean_files(&files)
            .context("Failed to clean test files for invalid path test")?;

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let result = standard_y_plot(&y, files[0].as_path(), "Test Plot", "Y", "X");

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
        clean_files(&files)
            .context("Failed to clean test files for standard time plot test")?;

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let sample_rate_hz = 1.0;

        let title = "Test Plot";
        let y_label = "Y Label";

        standard_time_plot(&y, sample_rate_hz, files[0].as_path(), title, y_label)
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
        clean_files(&files)
            .context("Failed to clean test files for zero sample rate test")?;

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let sample_rate_hz = 0.0;

        let title = "Test Plot";
        let y_label = "Y Label";

        let result = standard_time_plot(&y, sample_rate_hz, files[0].as_path(), title, y_label);

        assert!(result.is_err());
        assert!(!files[0].is_file());
        Ok(())
    }

    #[test]
    fn test_standard_time_plot_negative_sample_rate() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("time_plot_negative_sample_rate.png")];
        clean_files(&files);

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let sample_rate_hz = -1.0;

        let title = "Test Plot";
        let y_label = "Y Label";

        let result = standard_time_plot(&y, sample_rate_hz, files[0].as_path(), title, y_label);

        assert!(result.is_err());
        assert!(!files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_xyz_state_plot_basic() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("xyz_plot_basic.png")];
        clean_files(&files);

        let mut system_states = SystemStates::empty(100, 6);

        for i in 0..100 {
            for j in 0..6 {
                system_states[(i, j)] = i as f32 * j as f32;
            }
        }

        let title = "Test Plot";
        let sample_rate_hz = 10.0;

        plot_state_xyz(&system_states, 1, sample_rate_hz, files[0].as_path(), title)
            .context("Failed to create XYZ state plot")?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    fn test_xyz_state_plot_invalid_index() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("xyz_plot_invalid.png")];
        clean_files(&files);

        let system_states = SystemStates::empty(100, 6);
        let title = "Test Plot";
        let sample_rate_hz = 10.0;

        let results = plot_state_xyz(&system_states, 5, sample_rate_hz, files[0].as_path(), title);

        assert!(results.is_err());
        assert!(!files[0].is_file());
        Ok(())
    }
}
