use std::path::Path;

use anyhow::Result;
use ndarray::{Array2, Axis};
use tracing::trace;

use super::PngBundle;
use crate::{
    core::{
        data::shapes::{SystemStates, SystemStatesSpherical, SystemStatesSphericalMax},
        model::spatial::voxels::{VoxelNumbers, VoxelPositions},
    },
    vis::plotting::{
        png::matrix::{matrix_angle_plot, matrix_plot},
        PlotSlice, StatePlotMode, StateSphericalPlotMode,
    },
};

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
pub(crate) fn states_plot(
    states: &SystemStates,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    voxel_numbers: &VoxelNumbers,
    path: &Path,
    slice: Option<PlotSlice>,
    mode: Option<StatePlotMode>,
    time_step: usize,
) -> Result<PngBundle> {
    trace!("Generating activation time plot");
    let slice = slice.unwrap_or(PlotSlice::Z(0));
    let mode = mode.unwrap_or(StatePlotMode::X);
    let step = Some((voxel_size_mm, voxel_size_mm));

    let (numbers, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let numbers = voxel_numbers.index_axis(Axis(0), index);
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 1)],
                voxel_positions_mm[(0, 0, 0, 2)],
            ));
            let title =
                format!("System States in {mode:?} (x-index = {index}, time-index = {time_step})");
            let x_label = Some("y [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((true, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Y(index) => {
            let numbers = voxel_numbers.index_axis(Axis(1), index);
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 0)],
                voxel_positions_mm[(0, 0, 0, 2)],
            ));
            let title =
                format!("System States in {mode:?} (y-index = {index}, time-index = {time_step})");
            let x_label = Some("x [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Z(index) => {
            let numbers = voxel_numbers.index_axis(Axis(2), index);
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 0)],
                voxel_positions_mm[(0, 0, 0, 1)],
            ));
            let title =
                format!("System States in {mode:?} (z-index = {index}, time-index = {time_step})");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
    };

    let mut data = Array2::zeros(numbers.raw_dim());

    let state_offset = match mode {
        StatePlotMode::X => 0,
        StatePlotMode::Y => 1,
        StatePlotMode::Z => 2,
    };
    for ((x, y), number) in numbers.indexed_iter() {
        data[(x, y)] = number
            .as_ref()
            .map_or(0.0, |number| states[(time_step, *number + state_offset)]);
    }

    matrix_plot(
        &data,
        None,
        step,
        offset,
        Some(path),
        Some(title.as_str()),
        y_label,
        x_label,
        Some("[A/mm^2]"),
        None,
        flip_axis,
    )
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
pub(crate) fn states_spherical_plot(
    states: &SystemStatesSpherical,
    states_max: &SystemStatesSphericalMax,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    voxel_numbers: &VoxelNumbers,
    path: Option<&Path>,
    slice: Option<PlotSlice>,
    mode: Option<StateSphericalPlotMode>,
    time_step: Option<usize>,
    range: Option<(f32, f32)>,
) -> Result<PngBundle> {
    trace!("Generating activation time plot");
    let slice = slice.unwrap_or(PlotSlice::Z(0));
    let mode = mode.unwrap_or(StateSphericalPlotMode::ABS);
    if voxel_size_mm <= 0.0 {
        return Err(anyhow::anyhow!("Voxel size must be a positive number"));
    }
    let step = Some((voxel_size_mm, voxel_size_mm));

    let title_time = time_step.map_or_else(
        || "max".to_string(),
        |time_step| format!("time-index {time_step}"),
    );

    let (numbers, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let numbers = voxel_numbers.index_axis(Axis(0), index);
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 1)],
                voxel_positions_mm[(0, 0, 0, 2)],
            ));
            let title = format!("System States {mode:?} (x-index = {index}, {title_time})");
            let x_label = Some("y [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((true, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Y(index) => {
            let numbers = voxel_numbers.index_axis(Axis(1), index);
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 0)],
                voxel_positions_mm[(0, 0, 0, 2)],
            ));
            let title = format!("System States {mode:?} (y-index = {index}, {title_time})");
            let x_label = Some("x [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Z(index) => {
            let numbers = voxel_numbers.index_axis(Axis(2), index);
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 0)],
                voxel_positions_mm[(0, 0, 0, 1)],
            ));
            let title = format!("System States {mode:?} (z-index = {index}, {title_time})");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
    };

    match mode {
        StateSphericalPlotMode::ABS => {
            let mut data = Array2::zeros(numbers.raw_dim());
            for ((x, y), number) in numbers.indexed_iter() {
                data[(x, y)] = time_step.map_or_else(
                    || {
                        number
                            .as_ref()
                            .map_or(0.0, |number| states_max.magnitude[*number / 3])
                    },
                    |time_step| {
                        number
                            .as_ref()
                            .map_or(0.0, |number| states.magnitude[(time_step, *number / 3)])
                    },
                );
            }
            matrix_plot(
                &data,
                range,
                step,
                offset,
                path,
                Some(title.as_str()),
                y_label,
                x_label,
                Some("[A/mm^2]"),
                None,
                flip_axis,
            )
        }
        StateSphericalPlotMode::ANGLE => {
            let mut theta = Array2::zeros(numbers.raw_dim());
            let mut phi = Array2::zeros(numbers.raw_dim());
            for ((x, y), number) in numbers.indexed_iter() {
                theta[(x, y)] = time_step.map_or_else(
                    || {
                        number
                            .as_ref()
                            .map_or(0.0, |number| states_max.theta[*number / 3])
                    },
                    |time_step| {
                        number
                            .as_ref()
                            .map_or(0.0, |number| states.theta[(time_step, *number / 3)])
                    },
                );
                phi[(x, y)] = time_step.map_or_else(
                    || {
                        number
                            .as_ref()
                            .map_or(0.0, |number| states_max.phi[*number / 3])
                    },
                    |time_step| {
                        number
                            .as_ref()
                            .map_or(0.0, |number| states.phi[(time_step, *number / 3)])
                    },
                );
            }
            matrix_angle_plot(
                &theta,
                &phi,
                step,
                offset,
                path,
                Some(title.as_str()),
                y_label,
                x_label,
                None,
                flip_axis,
            )
        }
    }
}

#[cfg(test)]
mod tests;
