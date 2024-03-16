use ndarray::Array1;
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
use std::{error::Error, io, path::Path};
use tracing::trace;

use super::{AXIS_STYLE, CAPTION_STYLE, STANDARD_RESOLUTION, X_MARGIN, Y_MARGIN};

/// Generates an XY plot from the provided x and y data.
///
/// Saves the plot to the optionally provided path as a PNG,
/// returns the raw pixel buffer.
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "trace")]
pub fn xy_plot(
    x: Option<&Array1<f32>>,
    y: &Array1<f32>,
    path: Option<&Path>,
    title: Option<&str>,
    y_label: Option<&str>,
    x_label: Option<&str>,
    resolution: Option<(u32, u32)>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    trace!("Generating xy plot.");

    let (width, height) = resolution.unwrap_or(STANDARD_RESOLUTION);

    let mut buffer = allocate_buffer(width, height);

    let default_x = x
        .is_none()
        .then(|| Array1::linspace(0.0, y.len() as f32, y.len()));
    let x = x.unwrap_or_else(|| default_x.as_ref().unwrap());

    if x.len() != y.len() {
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
    let y_min = y.min()?;
    let y_max = y.max()?;

    let x_range = x_max - x_min;
    let y_range = y_max - y_min;

    let x_min = x_min - x_range * X_MARGIN;
    let x_max = x_max + x_range * X_MARGIN;
    let y_min = y_min - y_range * Y_MARGIN;
    let y_max = y_max + y_range * Y_MARGIN;

    let data: Vec<(f32, f32)> = x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)).collect();

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

        chart.draw_series(LineSeries::new(data, &RED))?;

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

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use super::*;
    const COMMON_PATH: &str = "tests/vis/plotting/xy";

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
            &y,
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
        xy_plot(None, &y, Some(files[0].as_path()), None, None, None, None).unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    fn test_xy_plot_no_path() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_xy_plot_no_path.png")];
        clean(&files);

        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);
        xy_plot(None, &y, None, None, None, None, None).unwrap();

        assert!(!files[0].is_file());
    }

    #[test]
    fn test_xy_plot_default_resolution() {
        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);

        let buffer = xy_plot(None, &y, None, None, None, None, None).unwrap();

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

        let buffer = xy_plot(None, &y, None, None, None, None, Some(resolution)).unwrap();

        assert_eq!(
            buffer.len(),
            resolution.0 as usize * resolution.1 as usize * 3
        );
    }

    #[test]
    fn test_xy_plot_incompatible_x_y() {
        let x = Array1::linspace(0.0, 10.0, 100);
        let y = Array1::zeros(90);

        assert!(xy_plot(Some(&x), &y, None, None, None, None, None).is_err());
    }
}
