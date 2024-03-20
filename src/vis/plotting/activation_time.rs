use ndarray::{Array2, ArrayBase, Axis, Ix2};
use ndarray_stats::QuantileExt;
use plotters::prelude::*;
use scarlet::colormap::{ColorMap, ListedColorMap};
use std::{error::Error, io, path::Path};
use tracing::trace;

use crate::{
    core::{
        data::shapes::ArraySystemStates,
        model::{
            functional::allpass::shapes::ArrayActivationTime,
            spatial::voxels::{VoxelNumbers, VoxelPositions},
        },
    },
    vis::plotting::{
        allocate_buffer, matrix::matrix_plot, PlotSlice, AXIS_LABEL_AREA, AXIS_LABEL_NUM_MAX,
        CHART_MARGIN, COLORBAR_BOTTOM_MARGIN, COLORBAR_COLOR_NUMBERS, COLORBAR_TOP_MARGIN,
        COLORBAR_WIDTH, LABEL_AREA_RIGHT_MARGIN, LABEL_AREA_WIDTH, UNIT_AREA_TOP_MARGIN,
    },
};

use super::{AXIS_STYLE, CAPTION_STYLE, STANDARD_RESOLUTION};

/// Plots the activation time for a given slice (x, y or z) of the
/// activation time matrix.
#[tracing::instrument(level = "trace")]
pub(crate) fn activation_time_plot(
    activation_time_ms: &ArrayActivationTime,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    path: &Path,
    slice: Option<PlotSlice>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    trace!("Generating activation time plot");
    let slice = slice.unwrap_or(PlotSlice::Z(0));
    let step = Some((voxel_size_mm, voxel_size_mm));

    let (data, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let data = activation_time_ms
                .values
                .index_axis(Axis(0), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 1)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            ));
            let x = voxel_positions_mm.values[(index, 0, 0, 0)];
            let title = format!("Activation time x-index = {index}, x = {x} mm");
            let x_label = Some("y [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (data, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Y(index) => {
            let data = activation_time_ms
                .values
                .index_axis(Axis(1), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            ));
            let y = voxel_positions_mm.values[(0, index, 0, 1)];
            let title = format!("Activation time y-index = {index}, y = {y} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (data, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Z(index) => {
            let data = activation_time_ms
                .values
                .index_axis(Axis(2), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 1)],
            ));
            let z = voxel_positions_mm.values[(0, 0, index, 2)];
            let title = format!("Activation time z-index = {index}, z = {z} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = Some((false, true));

            (data, offset, title, x_label, y_label, flip_axis)
        }
    };

    matrix_plot(
        &data,
        None,
        step,
        offset,
        Some(path),
        Some(title.as_str()),
        y_label,
        x_label,
        Some("[ms]"),
        None,
        flip_axis,
    )
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use ndarray::Array2;

    use crate::core::{config::simulation::Simulation as SimulationConfig, data::Data};

    use super::*;
    const COMMON_PATH: &str = "tests/vis/plotting/actovation_time";

    #[tracing::instrument(level = "trace")]
    fn setup() {
        if !Path::new(COMMON_PATH).exists() {
            std::fs::create_dir_all(COMMON_PATH).unwrap();
        }
    }

    #[tracing::instrument(level = "trace")]
    fn clean(files: &Vec<PathBuf>) {
        for file in files {
            if file.is_file() {
                std::fs::remove_file(file).unwrap();
            }
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_default() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_activation_time_plot_default.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        activation_time_plot(
            data.get_activation_time_ms(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::Z(0)),
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_x_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_activation_time_plot_x_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        activation_time_plot(
            data.get_activation_time_ms(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::X(10)),
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_y_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_activation_time_plot_y_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        activation_time_plot(
            data.get_activation_time_ms(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::Y(5)),
        )
        .unwrap();

        assert!(files[0].is_file());
    }
}
