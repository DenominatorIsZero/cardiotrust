use ndarray::Array1;
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
use std::{error::Error, path::Path};
use tracing::trace;

use crate::core::data::shapes::ArraySystemStates;

const STANDARD_RESOLUTION: (u32, u32) = (800, 600);
const X_MARGIN: f32 = 0.0;
const Y_MARGIN: f32 = 0.1;
const CAPTION_STYLE: (&str, i32) = ("Arial", 30);
const AXIS_STYLE: (&str, i32) = ("Arial", 20);

#[tracing::instrument(level = "trace")]
pub fn standard_time_plot(
    y: &Array1<f32>,
    sample_rate_hz: f32,
    file_name: &str,
    title: &str,
    y_label: &str,
) {
    trace!("Generating time plot.");
    // #[allow(clippy::cast_precision_loss)]
    // let t = Array1::from_vec(
    //     (0..y.shape()[0])
    //         .map(|i| i as f32 / sample_rate_hz)
    //         .collect(),
    // );
    // let _t_min = *t.min_skipnan();
    // let _t_max = *t.max_skipnan();
    // let mut y_min = *y.min_skipnan();
    // let mut y_max = *y.max_skipnan();
    // let y_range = y_max - y_min;
    // let y_margin = 0.1_f32;
    // y_min = y_margin.mul_add(-y_range, y_min);
    // y_max = y_margin.mul_add(y_range, y_max);

    todo!()
    // let mut plot = Plot::new();
    // let trace = Scatter::from_array(t, y.clone()).mode(Mode::Lines);
    // plot.add_trace(trace);
    // let layout = Layout::new()
    //     .title(title.into())
    //     .x_axis(
    //         Axis::new()
    //             .title("t [s]".into())
    //             .range(vec![t_min, t_max])
    //             .show_spikes(true),
    //     )
    //     .y_axis(Axis::new().title(y_label.into()).range(vec![y_min, y_max]));
    // plot.set_layout(layout);

    // let width = 800;
    // let height = 600;
    // let scale = 1.0;
    // save_plot(file_name, &plot, width, height, scale);
}

/// .
///
/// # Panics
///
/// Panics if min or max of array couldn't be computed.
#[tracing::instrument(level = "trace")]
pub fn standard_y_plot(
    y: &Array1<f32>,
    file_name: &str,
    title: &str,
    y_label: &str,
    x_label: &str,
) {
    trace!("Generating y plot.");
    // let x = Array1::from_vec((0..y.shape()[0]).collect());
    // let _x_min = *x.min().expect("Could not calculate min of X-array");
    // let _x_max = *x.max().expect("Could not calculate max of X-array");
    // let mut y_min = *y.min_skipnan();
    // let mut y_max = *y.max_skipnan();
    // let y_range = y_max - y_min;
    // let y_margin = 0.1_f32;
    // y_min = y_margin.mul_add(-y_range, y_min);
    // y_max = y_margin.mul_add(y_range, y_max);

    todo!()

    // let mut plot = Plot::new();
    // let trace = Scatter::from_array(x, y.clone()).mode(Mode::Lines);
    // plot.add_trace(trace);
    // let layout = Layout::new()
    //     .title(title.into())
    //     .x_axis(
    //         Axis::new()
    //             .title(x_label.into())
    //             .range(vec![x_min, x_max])
    //             .show_spikes(true),
    //     )
    //     .y_axis(Axis::new().title(y_label.into()).range(vec![y_min, y_max]));
    // plot.set_layout(layout);

    // let width = 800;
    // let height = 600;
    // let scale = 1.0;
    // save_plot(file_name, &plot, width, height, scale);
}

#[tracing::instrument(level = "trace")]
pub fn plot_state_xyz(
    system_states: &ArraySystemStates,
    state_index: usize,
    sample_rate_hz: f32,
    file_name: &str,
    title: &str,
) {
    trace!("Generating state xyz plot.");
    // let x = system_states.values.slice(s![.., state_index]).to_owned();
    // let y = system_states
    //     .values
    //     .slice(s![.., state_index + 1])
    //     .to_owned();
    // let z = system_states
    //     .values
    //     .slice(s![.., state_index + 2])
    //     .to_owned();
    // #[allow(clippy::cast_precision_loss)]
    // let t = Array1::from_vec(
    //     (0..y.shape()[0])
    //         .map(|i| i as f32 / sample_rate_hz)
    //         .collect(),
    // );

    // let mut xyz_min = *arr1(&[*x.min_skipnan(), *y.min_skipnan(), *z.min_skipnan()]).min_skipnan();
    // let mut xyz_max = *arr1(&[*x.max_skipnan(), *y.max_skipnan(), *z.max_skipnan()]).max_skipnan();
    // let xyz_range = xyz_max - xyz_min;
    // let xyz_margin = 0.1_f32;
    // xyz_min = xyz_margin.mul_add(-xyz_range, xyz_min);
    // xyz_max = xyz_margin.mul_add(xyz_range, xyz_max);

    // let _t_min = *t.min_skipnan();
    // let _t_max = *t.max_skipnan();

    todo!()

    // let mut plot = Plot::new();
    // let trace_x = Scatter::from_array(t.clone(), x)
    //     .mode(Mode::Lines)
    //     .line(
    //         plotly::common::Line::new()
    //             .color(NamedColor::SkyBlue)
    //             .width(2.0)
    //             .dash(plotly::common::DashType::Solid),
    //     )
    //     .name("x");
    // let trace_y = Scatter::from_array(t.clone(), y)
    //     .mode(Mode::Lines)
    //     .line(
    //         plotly::common::Line::new()
    //             .color(NamedColor::Orange)
    //             .width(2.0)
    //             .dash(plotly::common::DashType::Dot),
    //     )
    //     .name("y");
    // let trace_z = Scatter::from_array(t, z)
    //     .mode(Mode::Lines)
    //     .line(
    //         plotly::common::Line::new()
    //             .color(NamedColor::SeaGreen)
    //             .width(2.0)
    //             .dash(plotly::common::DashType::Dash),
    //     )
    //     .name("z");
    // plot.add_trace(trace_x);
    // plot.add_trace(trace_y);
    // plot.add_trace(trace_z);
    // let layout = Layout::new()
    //     .title(title.into())
    //     .x_axis(Axis::new().title("t [s]".into()).range(vec![t_min, t_max]))
    //     .y_axis(
    //         Axis::new()
    //             .title("j [A/mm^2]".into())
    //             .range(vec![xyz_min, xyz_max]),
    //     );
    // plot.set_layout(layout);

    // let width = 800;
    // let height = 600;
    // let scale = 1.0;
    // save_plot(file_name, &plot, width, height, scale);
}

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

    let title = title.unwrap_or("Plot");
    let y_label = y_label.unwrap_or("y");
    let x_label = x_label.unwrap_or("x");

    let x_min = x.min().unwrap();
    let x_max = x.max().unwrap();
    let y_min = y.min().unwrap();
    let y_max = y.max().unwrap();

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

    use super::*;

    fn setup() {
        if !std::path::Path::new("tests/vis/plotting/time").exists() {
            std::fs::create_dir_all("tests/vis/plotting/time").unwrap();
        }
    }

    fn clean(files: &Vec<&Path>) {
        for file in files {
            if file.is_file() {
                std::fs::remove_file(file).unwrap();
            }
        }
    }

    #[test]
    fn test_xy_plot() {
        setup();
        let files = vec![Path::new("tests/vis/plotting/time/test_xy_plot.png")];
        clean(&files);

        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);
        xy_plot(
            Some(&x),
            &y,
            Some(files[0]),
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
        let files = vec![Path::new(
            "tests/vis/plotting/time/test_xy_plot_default.png",
        )];
        clean(&files);

        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);
        xy_plot(None, &y, Some(files[0]), None, None, None, None).unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    fn test_xy_plot_no_path() {
        setup();
        let files = vec![Path::new(
            "tests/vis/plotting/time/test_xy_plot_no_path.png",
        )];
        clean(&files);

        let x = Array1::linspace(0.0, 10.0, 100);
        let y = x.map(|x| x * x);
        xy_plot(None, &y, None, None, None, None, None).unwrap();

        assert!(!files[0].is_file());
    }
}
