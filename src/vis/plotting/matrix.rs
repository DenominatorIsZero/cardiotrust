use ndarray::{Array2, Array3};
use plotly::{common::ColorScale, layout::Axis, HeatMap, Layout, Plot};

use crate::core::model::{
    functional::allpass::shapes::ArrayActivationTime, spatial::voxels::VoxelType,
};

use super::save_plot;

pub fn plot_voxel_types(types: &Array3<VoxelType>, file_name: &str, title: &str) {
    let mut z: Vec<Vec<i32>> = Vec::new();
    for y in 0..types.shape()[1] {
        let mut row: Vec<i32> = Vec::new();
        for x in 0..types.shape()[0] {
            row.push(types[(x, y, 0)] as i32);
        }
        z.push(row);
    }
    let mut row: Vec<i32> = Vec::new();
    for x in 0..types.shape()[0] {
        if x < 7 {
            row.push(x as i32);
        } else {
            row.push(0)
        }
    }
    z.push(row);

    let trace = HeatMap::new_z(z).color_scale(ColorScale::Palette(
        plotly::common::ColorScalePalette::Earth,
    ));
    let mut plot = Plot::new();

    let width = (500.0 * types.shape()[0] as f32 / types.shape()[1] as f32) as usize + 175;
    let height = (500.0 * types.shape()[1] as f32 / types.shape()[0] as f32) as usize;

    let layout = Layout::new()
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("x".into())
                .range(vec![0, types.shape()[0] - 1]),
        )
        .y_axis(
            Axis::new()
                .title("y".into())
                .range(vec![types.shape()[1] - 1, 0])
                .anchor("x"),
        )
        .height(height)
        .width(width);

    plot.add_trace(trace);
    plot.set_layout(layout);

    save_plot(file_name, plot, width, height, 1.0);
}

pub fn plot_activation_time(activation_times: &ArrayActivationTime, file_name: &str, title: &str) {
    let times = &activation_times.values;
    let mut z: Vec<Vec<f32>> = Vec::new();
    for y in 0..times.shape()[1] {
        let mut row: Vec<f32> = Vec::new();
        for x in 0..times.shape()[0] {
            row.push(times[(x, y, 0)].unwrap_or(-1.0));
        }
        z.push(row);
    }

    let trace =
        HeatMap::new_z(z).color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    let mut plot = Plot::new();

    let width = (500.0 * times.shape()[0] as f32 / times.shape()[1] as f32) as usize + 175;
    let height = (500.0 * times.shape()[1] as f32 / times.shape()[0] as f32) as usize;

    let layout = Layout::new()
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("x".into())
                .range(vec![0, times.shape()[0] - 1]),
        )
        .y_axis(
            Axis::new()
                .title("y".into())
                .range(vec![times.shape()[1] - 1, 0])
                .anchor("x"),
        )
        .height(height)
        .width(width);

    plot.add_trace(trace);
    plot.set_layout(layout);

    save_plot(file_name, plot, width, height, 1.0);
}

pub fn plot_matrix(matrix: &Array2<f32>, file_name: &str, title: &str) {
    let mut z: Vec<Vec<f32>> = Vec::new();
    for y in 0..matrix.shape()[1] {
        let mut row: Vec<f32> = Vec::new();
        for x in 0..matrix.shape()[0] {
            row.push(matrix[(x, y)]);
        }
        z.push(row);
    }

    let trace =
        HeatMap::new_z(z).color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::RdBu));
    let mut plot = Plot::new();

    let width = (500.0 * matrix.shape()[0] as f32 / matrix.shape()[1] as f32) as usize + 175;
    let height = (500.0 * matrix.shape()[1] as f32 / matrix.shape()[0] as f32) as usize;

    let layout = Layout::new()
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("Axis 1".into())
                .range(vec![-0.5, matrix.shape()[0] as f32 - 0.5]),
        )
        .y_axis(
            Axis::new()
                .title("Axis 2".into())
                .range(vec![-0.5, matrix.shape()[1] as f32 - 0.5]),
        )
        .height(height)
        .width(width);

    plot.add_trace(trace);
    plot.set_layout(layout);

    save_plot(file_name, plot, width, height, 1.0);
}
