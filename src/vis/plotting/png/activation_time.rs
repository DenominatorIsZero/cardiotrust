use std::path::Path;

use anyhow::Result;
use ndarray::Axis;
use tracing::trace;

use super::PngBundle;
use crate::{
    core::model::{functional::allpass::shapes::ActivationTimeMs, spatial::voxels::VoxelPositions},
    vis::plotting::{png::matrix::matrix_plot, PlotSlice},
};

/// Plots the activation time for a given slice (x, y or z) of the
/// activation time matrix.
#[tracing::instrument(level = "trace")]
pub(crate) fn activation_time_plot(
    activation_time_ms: &ActivationTimeMs,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    path: &Path,
    slice: Option<PlotSlice>,
) -> Result<PngBundle> {
    trace!("Generating activation time plot");
    let slice = slice.unwrap_or(PlotSlice::Z(0));
    let step = Some((voxel_size_mm, voxel_size_mm));

    let (data, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let data = activation_time_ms
                .index_axis(Axis(0), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 1)],
                voxel_positions_mm[(0, 0, 0, 2)],
            ));
            let x = voxel_positions_mm[(index, 0, 0, 0)];
            let title = format!("Activation time x-index = {index}, x = {x} mm");
            let x_label = Some("y [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((true, false));

            (data, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Y(index) => {
            let data = activation_time_ms
                .index_axis(Axis(1), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 0)],
                voxel_positions_mm[(0, 0, 0, 2)],
            ));
            let y = voxel_positions_mm[(0, index, 0, 1)];
            let title = format!("Activation time y-index = {index}, y = {y} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (data, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Z(index) => {
            let data = activation_time_ms
                .index_axis(Axis(2), index)
                .map(|value| value.unwrap_or(0.0));
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 0)],
                voxel_positions_mm[(0, 0, 0, 1)],
            ));
            let z = voxel_positions_mm[(0, 0, index, 2)];
            let title = format!("Activation time z-index = {index}, z = {z} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = Some((false, false));

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

    use super::*;
    use crate::{
        core::{config::simulation::Simulation as SimulationConfig, data::Data},
        tests::{clean_files, setup_folder},
    };
    const COMMON_PATH: &str = "tests/vis/plotting/png/actovation_time";

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_default() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("test_activation_time_plot_default.png")];
        clean_files(&files)?;

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)?;

        activation_time_plot(
            &data
                .simulation
                .model
                .functional_description
                .ap_params
                .activation_time_ms,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::Z(0)),
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_x_slice() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("test_activation_time_plot_x_slice.png")];
        clean_files(&files)?;

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)?;

        activation_time_plot(
            &data
                .simulation
                .model
                .functional_description
                .ap_params
                .activation_time_ms,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::X(10)),
        )?;

        assert!(files[0].is_file());
        Ok(())
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_activation_time_plot_y_slice() -> Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf())?;
        let files = vec![path.join("test_activation_time_plot_y_slice.png")];
        clean_files(&files)?;

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)?;

        activation_time_plot(
            &data
                .simulation
                .model
                .functional_description
                .ap_params
                .activation_time_ms,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(PlotSlice::Y(5)),
        )?;

        assert!(files[0].is_file());
        Ok(())
    }
}
