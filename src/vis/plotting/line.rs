use ndarray::Array1;
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
use std::{error::Error, io, path::Path};
use tracing::trace;

use crate::vis::plotting::COLORS;

use super::{AXIS_STYLE, CAPTION_STYLE, STANDARD_RESOLUTION, X_MARGIN, Y_MARGIN};

/// Generates an XY plot from the provided x and y data.
///
/// Saves the plot to the optionally provided path as a PNG,
/// returns the raw pixel buffer.
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn xy_plot(
    x: Option<&Array1<f32>>,
    ys: Vec<&Array1<f32>>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    resolution: Option<(u32, u32)>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    trace!("Generating xy plot.");

    let (width, height) = resolution.unwrap_or(STANDARD_RESOLUTION);

    let mut buffer = allocate_buffer(width, height);

    let y_len = ys[0].len();

    for y in &ys {
        if y.len() != y_len {
            return Err(Box::new(std::io::Error::new(
                io::ErrorKind::InvalidInput,
                "y data must have same length",
            )));
        }
    }

    let default_x = x
        .is_none()
        .then(|| Array1::linspace(0.0, y_len as f32, y_len));
    let x = x.unwrap_or_else(|| default_x.as_ref().unwrap());

    if x.len() != y_len {
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "x and y must have same length",
        )));
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
            .margin(5)
            .x_label_area_size(50)
            .y_label_area_size(75)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc(x_label)
            .x_label_style(AXIS_STYLE.into_font())
            .y_desc(y_label)
            .y_label_style(AXIS_STYLE.into_font())
            .draw()?;

        for (i, y) in ys.iter().enumerate() {
            chart.draw_series(LineSeries::new(
                x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                &COLORS[i % COLORS.len()],
            ))?;
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

    Ok(buffer)
}

/// Allocates a buffer for storing pixel data for an image of the given width and height.
///
/// The buffer is allocated as a `Vec<u8>` with 3 bytes per pixel (for RGB color). The size of the
/// buffer is calculated from the width and height.
///
/// This function is used to allocate image buffers before rendering to them for plotting.
#[tracing::instrument(level = "trace")]
fn allocate_buffer(width: u32, height: u32) -> Vec<u8> {
    trace!("Allocating buffer.");
    let buffer: Vec<u8> = vec![0; width as usize * height as usize * 3];
    buffer
}

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
        vec![y],
        Some(path),
        Some(title),
        Some(y_label),
        Some(x_label),
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
pub fn standard_time_plot(
    y: &Array1<f32>,
    sample_rate_hz: f32,
    path: &Path,
    title: &str,
    y_label: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    trace!("Generating time plot.");
    if sample_rate_hz <= 0.0 {
        return Err(Box::new(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "sample_rate_hz must be greater than zero",
        )));
    }
    let x = Array1::linspace(0.0, y.len() as f32 / sample_rate_hz, y.len());
    xy_plot(
        Some(&x),
        vec![y],
        Some(path),
        Some(title),
        Some(y_label),
        Some("t [s]"),
        None,
    )
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use super::*;
    const COMMON_PATH: &str = "tests/vis/plotting/line";

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
    fn test_xy_plot() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_xy_plot.png")];
        clean(&files);

        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);
        xy_plot(
            Some(&x),
            vec![&y],
            Some(files[0].as_path()),
            Some("y=x^2"),
            Some("x [a.u.]"),
            Some("y [a.u.]"),
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    fn test_xy_plot_defaults() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_xy_plot_default.png")];
        clean(&files);

        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);
        xy_plot(
            None,
            vec![&y],
            Some(files[0].as_path()),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    fn test_xy_plot_no_path() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_xy_plot_no_path.png")];
        clean(&files);

        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);
        xy_plot(None, vec![&y], None, None, None, None, None).unwrap();

        assert!(!files[0].is_file());
    }

    #[test]
    fn test_xy_plot_default_resolution() {
        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);

        let buffer = xy_plot(None, vec![&y], None, None, None, None, None).unwrap();

        assert_eq!(
            buffer.len(),
            STANDARD_RESOLUTION.0 as usize * STANDARD_RESOLUTION.1 as usize * 3
        );
    }

    #[test]
    fn test_xy_plot_custom_resolution() {
        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);

        let resolution = (400, 300);

        let buffer = xy_plot(None, vec![&y], None, None, None, None, Some(resolution)).unwrap();

        assert_eq!(
            buffer.len(),
            resolution.0 as usize * resolution.1 as usize * 3
        );
    }

    #[test]
    fn test_xy_plot_incompatible_x_y() {
        let x = Array1::linspace(0.0, 10.0, 100);
        let y = Array1::zeros(90);

        assert!(xy_plot(Some(&x), vec![&y], None, None, None, None, None).is_err());
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

    #[test]
    fn test_standard_time_plot_normal() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_time_plot_normal.png")];
        clean(&files);

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let sample_rate_hz = 1.0;

        let title = "Test Plot";
        let y_label = "Y Label";

        standard_time_plot(&y, sample_rate_hz, files[0].as_path(), title, y_label).unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    fn test_standard_time_plot_zero_sample_rate() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_time_plot_zero_sample_rate.png")];
        clean(&files);

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let sample_rate_hz = 0.0;

        let title = "Test Plot";
        let y_label = "Y Label";

        let result = standard_time_plot(&y, sample_rate_hz, files[0].as_path(), title, y_label);

        assert!(result.is_err());
        assert!(!files[0].is_file());
    }

    #[test]
    fn test_standard_time_plot_negative_sample_rate() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_time_plot_negative_sample_rate.png")];
        clean(&files);

        let y = Array1::from_vec(vec![1.0, 2.0, 3.0]);

        let sample_rate_hz = -1.0;

        let title = "Test Plot";
        let y_label = "Y Label";

        let result = standard_time_plot(&y, sample_rate_hz, files[0].as_path(), title, y_label);

        assert!(result.is_err());
        assert!(!files[0].is_file());
    }
}
