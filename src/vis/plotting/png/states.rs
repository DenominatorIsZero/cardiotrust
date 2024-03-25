use ndarray::{Array2, Axis};

use std::{error::Error, path::Path};
use tracing::trace;

use crate::vis::plotting::StateSphericalPlotMode;
use crate::{
    core::{
        data::shapes::{
            ArraySystemStates, ArraySystemStatesSpherical, ArraySystemStatesSphericalMax,
        },
        model::spatial::voxels::{VoxelNumbers, VoxelPositions},
    },
    vis::plotting::{
        png::matrix::{matrix_angle_plot, matrix_plot},
        PlotSlice, StatePlotMode,
    },
};

use super::PngBundle;

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
pub(crate) fn states_plot(
    states: &ArraySystemStates,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    voxel_numbers: &VoxelNumbers,
    path: &Path,
    slice: Option<PlotSlice>,
    mode: Option<StatePlotMode>,
    time_step: usize,
) -> Result<PngBundle, Box<dyn Error>> {
    trace!("Generating activation time plot");
    let slice = slice.unwrap_or(PlotSlice::Z(0));
    let mode = mode.unwrap_or(StatePlotMode::X);
    let step = Some((voxel_size_mm, voxel_size_mm));

    let (numbers, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let numbers = voxel_numbers.values.index_axis(Axis(0), index);
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 1)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            ));
            let title =
                format!("System States in {mode:?} (x-index = {index}, time-index = {time_step})");
            let x_label = Some("y [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Y(index) => {
            let numbers = voxel_numbers.values.index_axis(Axis(1), index);
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            ));
            let title =
                format!("System States in {mode:?} (y-index = {index}, time-index = {time_step})");
            let x_label = Some("x [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Z(index) => {
            let numbers = voxel_numbers.values.index_axis(Axis(2), index);
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 1)],
            ));
            let title =
                format!("System States in {mode:?} (z-index = {index}, time-index = {time_step})");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = Some((false, true));

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
        data[(x, y)] = number.as_ref().map_or(0.0, |number| {
            states.values[(time_step, *number + state_offset)]
        });
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
    states: &ArraySystemStatesSpherical,
    states_max: &ArraySystemStatesSphericalMax,
    voxel_positions_mm: &VoxelPositions,
    voxel_size_mm: f32,
    voxel_numbers: &VoxelNumbers,
    path: Option<&Path>,
    slice: Option<PlotSlice>,
    mode: Option<StateSphericalPlotMode>,
    time_step: Option<usize>,
    range: Option<(f32, f32)>,
) -> Result<PngBundle, Box<dyn Error>> {
    trace!("Generating activation time plot");
    let slice = slice.unwrap_or(PlotSlice::Z(0));
    let mode = mode.unwrap_or(StateSphericalPlotMode::ABS);
    if voxel_size_mm <= 0.0 {
        return Err("Voxel size must be a positive number".into());
    }
    let step = Some((voxel_size_mm, voxel_size_mm));

    let title_time = time_step.map_or_else(
        || "max".to_string(),
        |time_step| format!("time-index {time_step}"),
    );

    let (numbers, offset, title, x_label, y_label, flip_axis) = match slice {
        PlotSlice::X(index) => {
            let numbers = voxel_numbers.values.index_axis(Axis(0), index);
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 1)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            ));
            let title = format!("System States {mode:?} (x-index = {index}, {title_time})");
            let x_label = Some("y [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Y(index) => {
            let numbers = voxel_numbers.values.index_axis(Axis(1), index);
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 2)],
            ));
            let title = format!("System States {mode:?} (y-index = {index}, {title_time})");
            let x_label = Some("x [mm]");
            let y_label = Some("z [mm]");
            let flip_axis = Some((false, false));

            (numbers, offset, title, x_label, y_label, flip_axis)
        }
        PlotSlice::Z(index) => {
            let numbers = voxel_numbers.values.index_axis(Axis(2), index);
            let offset = Some((
                voxel_positions_mm.values[(0, 0, 0, 0)],
                voxel_positions_mm.values[(0, 0, 0, 1)],
            ));
            let title = format!("System States {mode:?} (z-index = {index}, {title_time})");
            let x_label = Some("x [mm]");
            let y_label = Some("y [mm]");
            let flip_axis = Some((false, true));

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
mod test {

    use std::path::PathBuf;

    use crate::core::{config::simulation::Simulation as SimulationConfig, data::Data};

    use super::*;
    const COMMON_PATH: &str = "tests/vis/plotting/png/states";

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
    fn test_states_plot_default() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_plot_default.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_plot(
            data.get_system_states(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            files[0].as_path(),
            Some(PlotSlice::Z(0)),
            Some(StatePlotMode::X),
            350,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_plot_x_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_plot_x_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_plot(
            data.get_system_states(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            files[0].as_path(),
            Some(PlotSlice::X(10)),
            Some(StatePlotMode::X),
            350,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_plot_y_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_plot_y_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");
        states_plot(
            data.get_system_states(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            files[0].as_path(),
            Some(PlotSlice::Y(5)),
            Some(StatePlotMode::X),
            350,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_plot_in_y() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_plot_in_y.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_plot(
            data.get_system_states(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            files[0].as_path(),
            Some(PlotSlice::Z(0)),
            Some(StatePlotMode::Y),
            350,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_plot_in_z() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_plot_in_z.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_plot(
            data.get_system_states(),
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            files[0].as_path(),
            Some(PlotSlice::Z(0)),
            Some(StatePlotMode::Z),
            350,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_spherical_plot_abs_z_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_spherical_plot_abs_z_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            Some(350),
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_spherical_plot_abs_y_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_spherical_plot_abs_y_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Y(5)),
            Some(StateSphericalPlotMode::ABS),
            Some(350),
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_spherical_plot_abs_x_slice() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_spherical_plot_abs_x_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::X(10)),
            Some(StateSphericalPlotMode::ABS),
            Some(350),
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_spherical_plot_angle_z_slice() {
        setup();
        let files =
            vec![Path::new(COMMON_PATH).join("test_states_spherical_plot_angle_z_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ANGLE),
            Some(350),
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_spherical_plot_angle_y_slice() {
        setup();
        let files =
            vec![Path::new(COMMON_PATH).join("test_states_spherical_plot_angle_y_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Y(5)),
            Some(StateSphericalPlotMode::ANGLE),
            Some(350),
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_spherical_plot_angle_x_slice() {
        setup();
        let files =
            vec![Path::new(COMMON_PATH).join("test_states_spherical_plot_angle_x_slice.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::X(10)),
            Some(StateSphericalPlotMode::ANGLE),
            Some(350),
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_spherical_plot_abs_max() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_spherical_plot_abs_max.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            Some(PlotSlice::Z(0)),
            Some(StateSphericalPlotMode::ABS),
            None,
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn test_states_spherical_plot_angle_max() {
        setup();
        let files = vec![Path::new(COMMON_PATH).join("test_states_spherical_plot_angle_max.png")];
        clean(&files);

        let mut simulation_config = SimulationConfig::default();
        simulation_config.model.pathological = true;
        let data = Data::from_simulation_config(&simulation_config)
            .expect("Model parameters to be valid.");

        states_spherical_plot(
            &data.simulation.as_ref().unwrap().system_states_spherical,
            &data
                .simulation
                .as_ref()
                .unwrap()
                .system_states_spherical_max,
            &data.get_model().spatial_description.voxels.positions_mm,
            data.get_model().spatial_description.voxels.size_mm,
            &data.get_model().spatial_description.voxels.numbers,
            Some(files[0].as_path()),
            None,
            Some(StateSphericalPlotMode::ANGLE),
            None,
            None,
        )
        .unwrap();

        assert!(files[0].is_file());
    }
}
