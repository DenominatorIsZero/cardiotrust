use std::path::Path;

use approx::{assert_relative_eq, RelativeEq};

use ndarray::s;
use ndarray_stats::QuantileExt;

use crate::{
    core::{config::model::Mri, model::spatial::voxels::VoxelType},
    tests::setup_folder,
    vis::plotting::{
        gif::states::states_spherical_plot_over_time,
        png::{
            line::{plot_state_xyz, standard_time_plot},
            states::states_spherical_plot,
        },
        PlotSlice, StateSphericalPlotMode,
    },
};

use super::*;

const COMMON_PATH: &str = "tests/core/data/simulation";

#[test]
fn create_simulation_no_crash() {
    let config = &SimulationConfig::default();
    let simulation = Simulation::from_config(config);
    assert!(simulation.is_ok());
    let simulation = simulation.unwrap();
    let max = *simulation.system_states.max_skipnan();
    assert_relative_eq!(max, 0.0);
    let max = *simulation.measurements.max_skipnan();
    assert_relative_eq!(max, 0.0);
}

#[test]
fn run_simulation_default() {
    let config = &SimulationConfig::default();
    let mut simulation = Simulation::from_config(config).unwrap();
    simulation.run();
    let max = *simulation.system_states.max_skipnan();
    assert!(max.relative_eq(&1.0, 0.001, 0.001));
    let max = *simulation.measurements.max_skipnan();
    assert!(max > 0.0);
}

#[test]
#[ignore]
#[allow(clippy::too_many_lines)]
fn run_simulation_default_and_plot() {
    let folder = Path::new(COMMON_PATH).join("healthy");
    setup_folder(&folder);
    let config = &SimulationConfig::default();
    let mut simulation = Simulation::from_config(config).unwrap();
    simulation.run();
    let max = *simulation.system_states.max_skipnan();
    assert!(max.relative_eq(&1.0, 0.001, 0.001));
    let max = *simulation.measurements.max_skipnan();
    assert!(max > 0.0);

    let sa_index = simulation
        .model
        .spatial_description
        .voxels
        .get_first_state_of_type(VoxelType::Sinoatrial);

    let path = folder.join("sa.png");
    plot_state_xyz(
        &simulation.system_states,
        sa_index,
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Current Density Sinoatrial Node",
    )
    .unwrap();

    let av_index = simulation
        .model
        .spatial_description
        .voxels
        .get_first_state_of_type(VoxelType::Atrioventricular);

    let path = folder.join("av.png");
    plot_state_xyz(
        &simulation.system_states,
        av_index,
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Current Density Atrioventricular Node",
    )
    .unwrap();

    let path = folder.join("sensor_0_x.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 0]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - x",
        "H [pT]",
    )
    .unwrap();

    let path = folder.join("sensor_0_y.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 1]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - y",
        "H [pT]",
    )
    .unwrap();

    let path = folder.join("sensor_0_z.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 2]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - z",
        "H [pT]",
    )
    .unwrap();

    let time_index = simulation.system_states.shape()[0] / 3;

    let path = folder.join(format!("states{time_index}.png"));
    states_spherical_plot(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        Some(time_index),
        Some((0.0, 1.0)),
    )
    .unwrap();

    let path = folder.join("states_max.png");
    states_spherical_plot(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        None,
        None,
    )
    .unwrap();

    let fps = 20;
    let playback_speed = 0.1;

    let path = folder.join("states.gif");
    states_spherical_plot_over_time(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        config.sample_rate_hz,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        Some(playback_speed),
        Some(fps),
    )
    .unwrap();
}

#[test]
fn run_simulation_pathological() {
    let mut config = SimulationConfig::default();
    config.model.common.pathological = true;
    let mut simulation = Simulation::from_config(&config).unwrap();
    simulation.run();
    let max = *simulation.system_states.max_skipnan();
    assert!(max.relative_eq(&1.0, 0.001, 0.001));
    let max = *simulation.measurements.max_skipnan();
    assert!(max > 0.0);
}

#[test]
#[ignore]
#[allow(clippy::too_many_lines)]
fn run_simulation_pathological_and_plot() {
    let folder = Path::new(COMMON_PATH).join("pathological");
    setup_folder(&folder);
    let mut config = SimulationConfig::default();
    config.model.common.pathological = true;
    let mut simulation = Simulation::from_config(&config).unwrap();
    simulation.run();
    let max = *simulation.system_states.max_skipnan();
    assert!(max.relative_eq(&1.0, 0.001, 0.001));
    let max = *simulation.measurements.max_skipnan();
    assert!(max > 0.0);

    let sa_index = simulation
        .model
        .spatial_description
        .voxels
        .get_first_state_of_type(VoxelType::Sinoatrial);

    let path = folder.join("sa.png");
    plot_state_xyz(
        &simulation.system_states,
        sa_index,
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Current Density Sinoatrial Node",
    )
    .unwrap();

    let av_index = simulation
        .model
        .spatial_description
        .voxels
        .get_first_state_of_type(VoxelType::Atrioventricular);

    let path = folder.join("av.png");
    plot_state_xyz(
        &simulation.system_states,
        av_index,
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Current Density Atrioventricular Node",
    )
    .unwrap();

    let path = folder.join("sensor_0_x.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 0]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - x",
        "H [pT]",
    )
    .unwrap();

    let path = folder.join("sensor_0_y.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 1]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - y",
        "H [pT]",
    )
    .unwrap();

    let path = folder.join("sensor_0_z.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 2]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - z",
        "H [pT]",
    )
    .unwrap();

    let time_index = simulation.system_states.shape()[0] / 3;

    let path = folder.join(format!("states{time_index}.png"));
    states_spherical_plot(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        Some(time_index),
        None,
    )
    .unwrap();

    let path = folder.join("states_max.png");
    states_spherical_plot(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        None,
        None,
    )
    .unwrap();

    let fps = 20;
    let playback_speed = 0.1;
    let path = folder.join("states.gif");
    states_spherical_plot_over_time(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        config.sample_rate_hz,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        Some(playback_speed),
        Some(fps),
    )
    .unwrap();
}

#[test]
fn run_simulation_mri() {
    let mut config = SimulationConfig::default();
    config.model.handcrafted = None;
    config.model.mri = Some(Mri::default());
    let mut simulation = Simulation::from_config(&config).unwrap();
    simulation.run();
    let max = *simulation.system_states.max_skipnan();
    assert!(max.relative_eq(&1.0, 0.002, 0.002), "max: {max}");
    let max = *simulation.measurements.max_skipnan();
    assert!(max > 0.0);
}

#[test]
#[ignore]
#[allow(clippy::too_many_lines)]
fn run_simulation_mri_and_plot() {
    let folder = Path::new(COMMON_PATH).join("mri");
    setup_folder(&folder);
    let mut config = SimulationConfig::default();
    config.model.handcrafted = None;
    config.model.mri = Some(Mri::default());
    let mut simulation = Simulation::from_config(&config).unwrap();
    simulation.run();
    let max = *simulation.system_states.max_skipnan();
    assert!(max.relative_eq(&1.0, 0.002, 0.002));
    let max = *simulation.measurements.max_skipnan();
    assert!(max > 0.0);

    let sa_index = simulation
        .model
        .spatial_description
        .voxels
        .get_first_state_of_type(VoxelType::Sinoatrial);

    let path = folder.join("sa.png");
    plot_state_xyz(
        &simulation.system_states,
        sa_index,
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Current Density Sinoatrial Node",
    )
    .unwrap();

    let path = folder.join("sensor_0_x.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 0]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - x",
        "H [pT]",
    )
    .unwrap();

    let path = folder.join("sensor_0_y.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 1]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - y",
        "H [pT]",
    )
    .unwrap();

    let path = folder.join("sensor_0_z.png");
    standard_time_plot(
        &simulation.measurements.slice(s![0, .., 2]).to_owned(),
        config.sample_rate_hz,
        path.as_path(),
        "Simulated Measurement Sensor 0 - z",
        "H [pT]",
    )
    .unwrap();

    let time_index = simulation.system_states.shape()[0] / 3;

    let path = folder.join(format!("states{time_index}.png"));
    states_spherical_plot(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(25)),
        Some(StateSphericalPlotMode::ABS),
        Some(time_index),
        None,
    )
    .unwrap();

    let path = folder.join("states_max.png");
    states_spherical_plot(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(25)),
        Some(StateSphericalPlotMode::ABS),
        None,
        None,
    )
    .unwrap();

    let fps = 20;
    let playback_speed = 0.1;
    let path = folder.join("states.gif");
    states_spherical_plot_over_time(
        &simulation.system_states_spherical,
        &simulation.system_states_spherical_max,
        &simulation.model.spatial_description.voxels.positions_mm,
        simulation.model.spatial_description.voxels.size_mm,
        config.sample_rate_hz,
        &simulation.model.spatial_description.voxels.numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(25)),
        Some(StateSphericalPlotMode::ABS),
        Some(playback_speed),
        Some(fps),
    )
    .unwrap();
}
