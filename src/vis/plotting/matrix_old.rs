use itertools::Itertools;
use ndarray::{s, Array1, Array2, Array3};
use ndarray_stats::QuantileExt;
use tracing::trace;

use crate::core::{
    data::shapes::ArraySystemStates,
    model::{
        functional::allpass::shapes::ArrayActivationTime,
        spatial::voxels::{VoxelType, Voxels},
    },
};

#[tracing::instrument(level = "trace")]
pub fn plot_voxel_types(types: &Array3<VoxelType>, file_name: &str, title: &str) {
    trace!("Plotting sytem states.");
    // let mut z: Vec<Vec<i32>> = Vec::new();
    // for y in 0..types.shape()[1] {
    //     let mut row: Vec<i32> = Vec::new();
    //     for x in 0..types.shape()[0] {
    //         row.push(types[(x, y, 0)] as i32);
    //     }
    //     z.push(row);
    // }
    // let mut row: Vec<i32> = Vec::new();
    // for x in 0..types.shape()[0] {
    //     if x < 7 {
    //         row.push(i32::try_from(x).unwrap_or_default());
    //     } else {
    //         row.push(0);
    //     }
    // }
    // z.push(row);

    todo!()
    // let trace = HeatMap::new_z(z).color_scale(ColorScale::Palette(
    //     plotly::common::ColorScalePalette::Earth,
    // ));
    // let mut plot = Plot::new();

    // #[allow(
    //     clippy::cast_possible_truncation,
    //     clippy::cast_sign_loss,
    //     clippy::cast_precision_loss
    // )]
    // let width = (500.0 * types.shape()[0] as f32 / types.shape()[1] as f32) as usize + 175;
    // #[allow(
    //     clippy::cast_possible_truncation,
    //     clippy::cast_sign_loss,
    //     clippy::cast_precision_loss
    // )]
    // let height = (500.0 * types.shape()[1] as f32 / types.shape()[0] as f32) as usize;

    // let layout = Layout::new()
    //     .title(title.into())
    //     .x_axis(
    //         Axis::new()
    //             .title("x".into())
    //             .range(vec![0, types.shape()[0] - 1]),
    //     )
    //     .y_axis(
    //         Axis::new()
    //             .title("y".into())
    //             .range(vec![types.shape()[1] - 1, 0])
    //             .anchor("x"),
    //     )
    //     .height(height)
    //     .width(width);

    // plot.add_trace(trace);
    // plot.set_layout(layout);

    // save_plot(file_name, &plot, width, height, 1.0);
}

/// Plots current densities at given time for x-y plane at z=0
/// Creates four subplots:
///     - in x direction
///     - in y direction
///     - in z direction
///     - absolute value
#[allow(clippy::too_many_lines)]
#[tracing::instrument(level = "trace")]
pub fn plot_states_at_time(
    system_states: &ArraySystemStates,
    voxels: &Voxels,
    min_j_init: f32,
    max_j_init: f32,
    time_index: usize,
    file_name: &str,
    title: &str,
) {
    trace!("Plotting system states at time.");
    // let system_states = &system_states.values;
    // let z_index = 0;

    // let mut in_x: Vec<Vec<f32>> = Vec::new();
    // let mut in_y: Vec<Vec<f32>> = Vec::new();
    // let mut in_z: Vec<Vec<f32>> = Vec::new();
    // let mut abs: Vec<Vec<f32>> = Vec::new();

    // let mut min_j = min_j_init;
    // let mut max_j = max_j_init;

    // for y_index in 0..voxels.count_xyz()[1] {
    //     let mut row_x: Vec<f32> = Vec::new();
    //     let mut row_y: Vec<f32> = Vec::new();
    //     let mut row_z: Vec<f32> = Vec::new();
    //     let mut row_abs: Vec<f32> = Vec::new();
    //     for x_index in 0..voxels.count_xyz()[0] {
    //         let voxel_index = [x_index, y_index, z_index];
    //         let state_index = voxels.numbers.values[voxel_index];
    //         match state_index {
    //             None => {
    //                 row_x.push(0.0);
    //                 row_y.push(0.0);
    //                 row_z.push(0.0);
    //                 row_abs.push(0.0);
    //             }
    //             Some(state_index) => {
    //                 row_x.push(system_states[(time_index, state_index)]);
    //                 row_y.push(system_states[(time_index, state_index + 1)]);
    //                 row_z.push(system_states[(time_index, state_index + 2)]);
    //                 row_abs.push(
    //                     system_states
    //                         .slice(s![time_index, state_index..state_index + 3])
    //                         .mapv(f32::abs)
    //                         .sum(),
    //                 );
    //             }
    //         }
    //     }
    //     in_x.push(row_x.clone());
    //     min_j = f32::min(
    //         min_j,
    //         row_x.into_iter().reduce(f32::min).unwrap_or_default(),
    //     );
    //     in_y.push(row_y.clone());
    //     min_j = f32::min(
    //         min_j,
    //         row_y.into_iter().reduce(f32::min).unwrap_or_default(),
    //     );
    //     in_z.push(row_z.clone());
    //     min_j = f32::min(
    //         min_j,
    //         row_z.into_iter().reduce(f32::min).unwrap_or_default(),
    //     );
    //     abs.push(row_abs.clone());
    //     min_j = f32::min(
    //         min_j,
    //         row_abs
    //             .clone()
    //             .into_iter()
    //             .reduce(f32::min)
    //             .unwrap_or_default(),
    //     );
    //     max_j = f32::max(
    //         max_j,
    //         row_abs.into_iter().reduce(f32::max).unwrap_or_default(),
    //     );
    // }

    // // add invisible row to make all subplots have the same scale
    // let mut row_x: Vec<f32> = Vec::new();
    // let mut row_y: Vec<f32> = Vec::new();
    // let mut row_z: Vec<f32> = Vec::new();
    // let mut row_abs: Vec<f32> = Vec::new();
    // row_x.push(min_j);
    // row_y.push(min_j);
    // row_z.push(min_j);
    // row_abs.push(min_j);
    // for _ in 1..voxels.count_xyz()[0] {
    //     row_x.push(max_j);
    //     row_y.push(max_j);
    //     row_z.push(max_j);
    //     row_abs.push(max_j);
    // }
    // in_x.push(row_x);
    // in_y.push(row_y);
    // in_z.push(row_z);
    // abs.push(row_abs);

    todo!()

    // let trace_x = HeatMap::new_z(in_x)
    //     .name("x")
    //     .x_axis("x1")
    //     .y_axis("y1")
    //     .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    // let trace_y = HeatMap::new_z(in_y)
    //     .name("y")
    //     .x_axis("x2")
    //     .y_axis("y2")
    //     .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    // let trace_z = HeatMap::new_z(in_z)
    //     .name("z")
    //     .x_axis("x3")
    //     .y_axis("y3")
    //     .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    // let trace_abs = HeatMap::new_z(abs)
    //     .name("abs")
    //     .x_axis("x4")
    //     .y_axis("y4")
    //     .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));

    // let mut plot = Plot::new();
    // plot.add_trace(trace_x);
    // plot.add_trace(trace_y);
    // plot.add_trace(trace_z);
    // plot.add_trace(trace_abs);

    // #[allow(
    //     clippy::cast_possible_truncation,
    //     clippy::cast_sign_loss,
    //     clippy::cast_precision_loss
    // )]
    // let width =
    //     (1000.0 * voxels.count_xyz()[0] as f32 / voxels.count_xyz()[1] as f32) as usize + 175;
    // #[allow(
    //     clippy::cast_possible_truncation,
    //     clippy::cast_sign_loss,
    //     clippy::cast_precision_loss
    // )]
    // let height = (1000.0 * voxels.count_xyz()[1] as f32 / voxels.count_xyz()[0] as f32) as usize;

    // let layout = Layout::new()
    //     .grid(
    //         LayoutGrid::new()
    //             .rows(2)
    //             .columns(2)
    //             .pattern(GridPattern::Independent),
    //     )
    //     .title(title.into())
    //     .x_axis(
    //         Axis::new()
    //             .title("x".into())
    //             .range(vec![0, voxels.count_xyz()[0] - 1]),
    //     )
    //     .y_axis(
    //         Axis::new()
    //             .title("y".into())
    //             .range(vec![voxels.count_xyz()[1] - 1, 0])
    //             .anchor("x"),
    //     )
    //     .x_axis2(
    //         Axis::new()
    //             .title("x".into())
    //             .range(vec![0, voxels.count_xyz()[0] - 1]),
    //     )
    //     .y_axis2(
    //         Axis::new()
    //             .title("y".into())
    //             .range(vec![voxels.count_xyz()[1] - 1, 0])
    //             .anchor("x"),
    //     )
    //     .x_axis3(
    //         Axis::new()
    //             .title("x".into())
    //             .range(vec![0, voxels.count_xyz()[0] - 1]),
    //     )
    //     .y_axis3(
    //         Axis::new()
    //             .title("y".into())
    //             .range(vec![voxels.count_xyz()[1] - 1, 0])
    //             .anchor("x"),
    //     )
    //     .x_axis4(
    //         Axis::new()
    //             .title("x".into())
    //             .range(vec![0, voxels.count_xyz()[0] - 1]),
    //     )
    //     .y_axis4(
    //         Axis::new()
    //             .title("y".into())
    //             .range(vec![voxels.count_xyz()[1] - 1, 0])
    //             .anchor("x"),
    //     )
    //     .height(height)
    //     .width(width);

    // plot.set_layout(layout);

    // save_plot(file_name, &plot, width, height, 1.0);
}

/// Plots maximum current densities for x-y plane at z=0
/// Creates four subplots:
///     - in x direction
///     - in y direction
///     - in z direction
///     - absolute value
#[tracing::instrument(level = "trace")]
pub fn plot_states_max(
    system_states: &ArraySystemStates,
    voxels: &Voxels,
    file_name: &str,
    title: &str,
) {
    trace!("Plotting states max");
    let mut in_x: Vec<Vec<f32>> = Vec::new();
    let mut in_y: Vec<Vec<f32>> = Vec::new();
    let mut in_z: Vec<Vec<f32>> = Vec::new();
    let mut abs: Vec<Vec<f32>> = Vec::new();

    // calculate_states_max(
    //     system_states,
    //     voxels,
    //     &mut in_x,
    //     &mut in_y,
    //     &mut in_z,
    //     &mut abs,
    // );

    todo!()

    // let trace_x = HeatMap::new_z(in_x)
    //     .name("x")
    //     .x_axis("x1")
    //     .y_axis("y1")
    //     .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    // let trace_y = HeatMap::new_z(in_y)
    //     .name("y")
    //     .x_axis("x2")
    //     .y_axis("y2")
    //     .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    // let trace_z = HeatMap::new_z(in_z)
    //     .name("z")
    //     .x_axis("x3")
    //     .y_axis("y3")
    //     .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));
    // let trace_abs = HeatMap::new_z(abs)
    //     .name("abs")
    //     .x_axis("x4")
    //     .y_axis("y4")
    //     .color_scale(ColorScale::Palette(plotly::common::ColorScalePalette::Jet));

    // let mut plot = Plot::new();
    // plot.add_trace(trace_x);
    // plot.add_trace(trace_y);
    // plot.add_trace(trace_z);
    // plot.add_trace(trace_abs);

    // #[allow(
    //     clippy::cast_precision_loss,
    //     clippy::cast_sign_loss,
    //     clippy::cast_possible_truncation
    // )]
    // let width =
    //     (1000.0 * voxels.count_xyz()[0] as f32 / voxels.count_xyz()[1] as f32) as usize + 175;
    // #[allow(
    //     clippy::cast_precision_loss,
    //     clippy::cast_sign_loss,
    //     clippy::cast_possible_truncation
    // )]
    // let height = (1000.0 * voxels.count_xyz()[1] as f32 / voxels.count_xyz()[0] as f32) as usize;

    // let layout = build_plot_states_max_layout(voxels, title, width, height);

    // plot.set_layout(layout);

    // save_plot(file_name, &plot, width, height, 1.0);
}

#[tracing::instrument(level = "trace")]
pub fn plot_states_max_delta(
    estimated_system_states: &ArraySystemStates,
    actual_system_states: &ArraySystemStates,
    voxels: &Voxels,
    file_name: &str,
    title: &str,
) {
    trace!("Plotting states max delta");
    // TODO: reconsider this. Might be not what I want to show..

    let mut delta_system_states = estimated_system_states.clone();
    delta_system_states.values -= &actual_system_states.values;

    plot_states_max(&delta_system_states, voxels, file_name, title);
}

/// .
///
/// # Panics
///
/// Panics if something fishy happens with io rights.
#[tracing::instrument(level = "trace")]
pub fn plot_states_over_time(
    system_states: &ArraySystemStates,
    voxels: &Voxels,
    fps: u32,
    playback_speed: f32,
    file_name: &str,
    title: &str,
) {
    trace!("Plotting states over time");
    // let directory = format!("./tmp/{file_name}/");
    // let dir_path = Path::new(&directory);
    // if dir_path.is_dir() {
    //     fs::remove_dir_all(dir_path).expect("Could not delete temporary directory");
    // }
    // fs::create_dir_all(dir_path).expect("Could not create temporary directory.");

    // let sample_number = system_states.values.shape()[0];
    // #[allow(
    //     clippy::cast_precision_loss,
    //     clippy::cast_sign_loss,
    //     clippy::cast_possible_truncation
    // )]
    // let image_number = (fps as f32 / playback_speed) as usize;
    // let time_step = sample_number / image_number;

    // let min_j_init = *system_states.values.min_skipnan();
    // let max_j_init = *system_states.values.max_skipnan(); // TODO: This should really be over the absolute values...

    // let time_indices: Vec<usize> = (0..sample_number).step_by(time_step).collect();
    // let mut image_names = Vec::new();

    // for (image_index, time_index) in time_indices.into_iter().enumerate() {
    //     let image_name = format!("./tmp/{file_name}/{image_index}");
    //     plot_states_at_time(
    //         system_states,
    //         voxels,
    //         min_j_init,
    //         max_j_init,
    //         time_index,
    //         &image_name,
    //         title,
    //     );
    //     image_names.push(format!("{image_name}.png"));
    // }

    todo!()
    // let images = engiffen::load_images(image_names.as_slice());
    // let gif = engiffen::engiffen(&images, fps as usize, engiffen::Quantizer::Naive)
    //     .expect("Could not create gif from images.");
    // let mut output_file =
    //     File::create(format!("{file_name}.gif")).expect("Could not create gif file.");
    // gif.write(&mut output_file)
    //     .expect("Could not write gif file.");
    // fs::remove_dir_all(dir_path).expect("Could not remove temporary folders.");
}
