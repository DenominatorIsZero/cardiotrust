use std::path::Path;

use approx::{assert_relative_eq, RelativeEq};
use ndarray::s;
use ndarray_stats::QuantileExt;

use super::*;
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
    let max = *simulation.measurements.max_skipnan();
    assert!(max > 0.0);
    // make sure the max in each voxel is one
    for index_voxel in 0..simulation.model.spatial_description.voxels.count_states() / 3 {
        for index_time in 0..simulation.system_states.shape()[0] {
            let value = simulation.system_states_spherical.magnitude[(index_time, index_voxel)];
            assert!(
                value < 1.003,
                "voxel: {index_voxel}, time: {index_time}, value: {value}"
            );
        }
    }

    let x_y_z = simulation.model.spatial_description.voxels.count_xyz();
    for x in 0..x_y_z[0] {
        for y in 0..x_y_z[1] {
            for z in 0..x_y_z[2] {
                if !simulation.model.spatial_description.voxels.types[(x, y, z)].is_connectable() {
                    continue;
                }
                let state = simulation.model.spatial_description.voxels.numbers[(x, y, z)].unwrap();
                crawl_through_states(&simulation, state);
            }
        }
    }
}

#[tracing::instrument(level = "trace", skip_all)]
fn crawl_through_states(simulation: &Simulation, state: usize) {
    let voxel = state / 3;

    let value = simulation.system_states_spherical_max.magnitude[voxel];
    let gains = simulation
        .model
        .functional_description
        .ap_params
        .gains
        .slice(s![state, ..]);
    if value < 0.99 {
        println!("voxel: {voxel}, value: {value}, gain: {gains}");
        let output_offset = gains.mapv(f32::abs).argmax_skipnan().unwrap();
        let output_state = simulation
            .model
            .functional_description
            .ap_params
            .output_state_indices[(state, output_offset)]
            .unwrap();
        let output_value = simulation.system_states_spherical_max.magnitude[output_state / 3];
        println!("output_offset: {output_offset}, output_state: {output_state}, output_value: {output_value}");
        crawl_through_states(simulation, output_state);
    }
    assert!(
        value > 0.99,
        "voxel: {voxel}, value: {value}, gain: {gains}"
    );
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
