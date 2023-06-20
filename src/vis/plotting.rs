use ndarray::{arr1, s, Array1, Array2, Array3};
use ndarray_stats::QuantileExt;
use plotly::{
    color::NamedColor,
    common::{ColorScale, Mode},
    layout::Axis,
    HeatMap, Layout, Plot, Scatter,
};

use crate::core::{
    data::shapes::ArraySystemStates,
    model::{functional::allpass::shapes::ArrayActivationTime, spatial::voxels::VoxelType},
};

pub mod matrix;
pub mod time;

fn save_plot(file_name: &str, plot: Plot, width: usize, height: usize, scale: f64) {
    plot.write_html(format!("{file_name}.html"));
    plot.write_image(
        format!("{file_name}.png"),
        plotly::ImageFormat::PNG,
        width,
        height,
        scale,
    );
    plot.write_image(
        format!("{file_name}.svg"),
        plotly::ImageFormat::SVG,
        width,
        height,
        scale,
    );
    plot.write_image(
        format!("{file_name}.pdf"),
        plotly::ImageFormat::PDF,
        width,
        height,
        scale,
    );
}
