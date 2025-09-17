use std::{error::Error, path::Path};

use ndarray::{Array2, Axis};
use tracing::trace;

use super::PngBundle;
use crate::{
    core::{
        algorithm::refinement::derivation::AverageDelays,
        model::spatial::voxels::{VoxelNumbers, VoxelPositions},
    },
    vis::plotting::{png::matrix::matrix_plot, PlotSlice},
};

/// Plots the activation time for a given slice (x, y or z) of the
/// activation time matrix.
#[tracing::instrument(level = "trace")]
pub(crate) fn average_delay_plot(
    average_delays: &AverageDelays,
    voxel_numbers: &VoxelNumbers,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    path: &Path,
    max_delay_displayed_samples: Option<f32>,
    slice: Option<PlotSlice>,
) -> Result<PngBundle, Box<dyn Error>> {
    trace!("Generating activation time plot");
    let slice = slice.unwrap_or(PlotSlice::Z(0));
    let step = Some((voxel_size_mm, voxel_size_mm));

    let (numbers, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let numbers = voxel_numbers.index_axis(Axis(0), index);
            let offset = Some((
                voxel_positions_mm[(0, 0, 0, 1)],
                voxel_positions_mm[(0, 0, 0, 2)],
            ));
            let x = voxel_positions_mm[(index, 0, 0, 0)];
            let title = format!("Average Delay x-index = {index}, x = {x} mm");
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
            let y = voxel_positions_mm[(0, index, 0, 1)];
            let title = format!("Average Delay y-index = {index}, y = {y} mm");
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
            let z = voxel_positions_mm[(0, 0, index, 2)];
            let title = format!("Average Delay z-index = {index}, z = {z} mm");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
    };

    let mut data = Array2::zeros(numbers.raw_dim());

    data.iter_mut()
        .zip(numbers.iter())
        .for_each(|(datum, number)| {
            if number.is_some() {
                *datum = average_delays[number.unwrap() / 3]
                    .unwrap_or(0.0)
                    .min(max_delay_displayed_samples.unwrap_or(f32::INFINITY));
            }
        });

    matrix_plot(
        &data,
        None,
        step,
        offset,
        Some(path),
        Some(title.as_str()),
        y_label,
        x_label,
        Some("[samples]"),
        None,
        flip_axis,
    )
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{
        core::{
            algorithm::refinement::derivation::calculate_average_delays,
            config::simulation::Simulation as SimulationConfig, data::Data,
        },
        tests::{clean_files, setup_folder},
    };
    const COMMON_PATH: &str = "tests/vis/plotting/png/delay";

    #[test]
    fn test_average_delay_plot_default() -> anyhow::Result<()> {
        let path = Path::new(COMMON_PATH);
        setup_folder(path.to_path_buf());
        let files = vec![path.join("test_average_delay_plot_default.png")];
        clean_files(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.common.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        let mut average_delays = AverageDelays::empty(data.simulation.system_states.num_states());
        calculate_average_delays(
            &mut average_delays,
            &data.simulation.model.functional_description.ap_params,
        )?;

        average_delay_plot(
            &average_delays,
            &data.simulation.model.spatial_description.voxels.numbers,
            &data
                .simulation
                .model
                .spatial_description
                .voxels
                .positions_mm,
            data.simulation.model.spatial_description.voxels.size_mm,
            files[0].as_path(),
            Some(10.0),
            Some(PlotSlice::Z(0)),
        )
        .unwrap();

        assert!(files[0].is_file());
        Ok(())
    }
}
