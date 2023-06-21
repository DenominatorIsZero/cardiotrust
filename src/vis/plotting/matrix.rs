use ndarray::{s, Array2, Array3};
use plotly::{
    common::ColorScale,
    layout::{Axis, GridPattern, LayoutGrid},
    HeatMap, Layout, Plot,
};

use crate::core::{
    data::shapes::ArraySystemStates,
    model::{
        functional::allpass::shapes::ArrayActivationTime,
        spatial::voxels::{VoxelType, Voxels},
    },
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

/// Plots current densities at given time for x-y plane at z=0
/// Creates four subplots:
///     - in x direction
///     - in y direction
///     - in z direction
///     - absolute value
pub fn plot_states_at_time(
    system_states: &ArraySystemStates,
    voxels: &Voxels,
    time_index: usize,
    file_name: &str,
    title: &str,
) {
    let system_states = &system_states.values;
    let z_index = 0;

    let mut in_x: Vec<Vec<f32>> = Vec::new();
    let mut in_y: Vec<Vec<f32>> = Vec::new();
    let mut in_z: Vec<Vec<f32>> = Vec::new();
    let mut abs: Vec<Vec<f32>> = Vec::new();

    for y_index in 0..voxels.count_xyz()[1] {
        let mut row_x: Vec<f32> = Vec::new();
        let mut row_y: Vec<f32> = Vec::new();
        let mut row_z: Vec<f32> = Vec::new();
        let mut row_abs: Vec<f32> = Vec::new();
        for x_index in 0..voxels.count_xyz()[0] {
            let voxel_index = [x_index, y_index, z_index];
            let state_index = voxels.numbers.values[voxel_index];
            match state_index {
                None => {
                    row_x.push(0.0);
                    row_y.push(0.0);
                    row_z.push(0.0);
                    row_abs.push(0.0);
                }
                Some(state_index) => {
                    row_x.push(system_states[(time_index, state_index)]);
                    row_y.push(system_states[(time_index, state_index + 1)]);
                    row_z.push(system_states[(time_index, state_index + 2)]);
                    row_abs.push(
                        system_states
                            .slice(s![time_index, state_index..state_index + 3])
                            .mapv(|v| v.powi(2))
                            .sum()
                            .sqrt(),
                    );
                }
            }
        }
        in_x.push(row_x);
        in_y.push(row_y);
        in_z.push(row_z);
        abs.push(row_abs);
    }

    let trace_x = HeatMap::new_z(in_x)
        .name("x")
        .x_axis("x1")
        .y_axis("y1")
        .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    let trace_y = HeatMap::new_z(in_y)
        .name("y")
        .x_axis("x2")
        .y_axis("y2")
        .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    let trace_z = HeatMap::new_z(in_z)
        .name("z")
        .x_axis("x3")
        .y_axis("y3")
        .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    let trace_abs = HeatMap::new_z(abs)
        .name("abs")
        .x_axis("x4")
        .y_axis("y4")
        .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));

    let mut plot = Plot::new();
    plot.add_trace(trace_x);
    plot.add_trace(trace_y);
    plot.add_trace(trace_z);
    plot.add_trace(trace_abs);

    let width =
        (1000.0 * voxels.count_xyz()[0] as f32 / voxels.count_xyz()[1] as f32) as usize + 175;
    let height = (1000.0 * voxels.count_xyz()[1] as f32 / voxels.count_xyz()[0] as f32) as usize;

    let layout = Layout::new()
        .grid(
            LayoutGrid::new()
                .rows(2)
                .columns(2)
                .pattern(GridPattern::Independent),
        )
        .title(title.into())
        .x_axis(
            Axis::new()
                .title("x".into())
                .range(vec![0, voxels.count_xyz()[0] - 1]),
        )
        .y_axis(
            Axis::new()
                .title("y".into())
                .range(vec![voxels.count_xyz()[1] - 1, 0])
                .anchor("x"),
        )
        .x_axis2(
            Axis::new()
                .title("x".into())
                .range(vec![0, voxels.count_xyz()[0] - 1]),
        )
        .y_axis2(
            Axis::new()
                .title("y".into())
                .range(vec![voxels.count_xyz()[1] - 1, 0])
                .anchor("x"),
        )
        .x_axis3(
            Axis::new()
                .title("x".into())
                .range(vec![0, voxels.count_xyz()[0] - 1]),
        )
        .y_axis3(
            Axis::new()
                .title("y".into())
                .range(vec![voxels.count_xyz()[1] - 1, 0])
                .anchor("x"),
        )
        .x_axis4(
            Axis::new()
                .title("x".into())
                .range(vec![0, voxels.count_xyz()[0] - 1]),
        )
        .y_axis4(
            Axis::new()
                .title("y".into())
                .range(vec![voxels.count_xyz()[1] - 1, 0])
                .anchor("x"),
        )
        .height(height)
        .width(width);

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
