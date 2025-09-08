use std::path::Path;

use super::{super::*, run};
use crate::{
    core::{
        config::{
            model::{SensorArrayGeometry, SensorArrayMotion},
            simulation::Simulation as SimulationConfig,
        },
        model::Model,
    },
    vis::plotting::{
        gif::states::states_spherical_plot_over_time,
        png::{line::standard_y_plot, states::states_spherical_plot},
        PlotSlice, StateSphericalPlotMode,
    },
};

const COMMON_PATH: &str = "tests/core/algorithm/loss_decreases";

#[tracing::instrument(level = "trace")]
fn setup(folder: Option<&str>) {
    let path = folder.map_or_else(
        || Path::new(COMMON_PATH).to_path_buf(),
        |folder| Path::new(COMMON_PATH).join(folder),
    );

    if !path.exists() {
        std::fs::create_dir_all(path).unwrap();
    }
}
#[test]
fn loss_decreases() {
    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
    simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
    let data =
        Data::from_simulation_config(&simulation_config).expect("Model parameters to be valid.");

    let mut algorithm_config = Algorithm {
        learning_rate: 1.0,
        epochs: 3,
        ..Default::default()
    };
    algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
    algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

    let model = Model::from_model_config(
        &algorithm_config.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )
    .expect("Model parameters to be valid.");

    let mut results = Results::new(
        algorithm_config.epochs,
        model.functional_description.control_function_values.shape()[0],
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        simulation_config
            .model
            .common
            .sensor_array_motion_steps
            .iter()
            .product(),
        0,
        algorithm_config.batch_size,
        algorithm_config.optimizer,
    );
    results.model = Some(model);

    run(&mut results, &data, &algorithm_config);

    (0..algorithm_config.epochs - 1).for_each(|i| {
        println!(
            "i: {}, i+1: {}",
            results.metrics.loss_batch[i],
            results.metrics.loss_batch[i + 1]
        );
        assert!(results.metrics.loss_batch[i] > results.metrics.loss_batch[i + 1]);
    });
}

#[test]
#[ignore = "expensive integration test"]
#[allow(clippy::too_many_lines)]
fn loss_decreases_and_plot() {
    setup(Some("default"));
    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.pathological = true;
    simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
    simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
    let data =
        Data::from_simulation_config(&simulation_config).expect("Model parameters to be valid.");

    let mut algorithm_config = Algorithm::default();
    algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
    algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

    let model = Model::from_model_config(
        &algorithm_config.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )
    .expect("Model paramters to be valid");
    algorithm_config.epochs = 10;

    let mut results = Results::new(
        algorithm_config.epochs,
        model.functional_description.control_function_values.shape()[0],
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        simulation_config
            .model
            .common
            .sensor_array_motion_steps
            .iter()
            .product(),
        0,
        algorithm_config.batch_size,
        algorithm_config.optimizer,
    );
    results.model = Some(model);

    run(&mut results, &data, &algorithm_config);

    let path = Path::new(COMMON_PATH).join("default").join("loss.png");
    standard_y_plot(
        &results.metrics.loss,
        Path::new(path.as_path()),
        "Loss",
        "Loss",
        "Step",
    )
    .unwrap();

    let path = Path::new(COMMON_PATH)
        .join("default")
        .join("loss_epoch.png");
    standard_y_plot(
        &results.metrics.loss_batch,
        Path::new(path.as_path()),
        "Sum Loss Per Epoch",
        "Loss",
        "Epoch",
    )
    .unwrap();

    let path = Path::new(COMMON_PATH)
        .join("default")
        .join("states_max.png");
    states_spherical_plot(
        &results.estimations.system_states_spherical,
        &results.estimations.system_states_spherical_max,
        &results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .positions_mm,
        results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .size_mm,
        &results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .numbers,
        Some(path.as_path()),
        None,
        Some(StateSphericalPlotMode::ABS),
        None,
        None,
    )
    .unwrap();

    let fps = 20;
    let playback_speed = 0.1;

    let path = Path::new(COMMON_PATH).join("default").join("states.gif");
    states_spherical_plot_over_time(
        &results.estimations.system_states_spherical,
        &results.estimations.system_states_spherical_max,
        &results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .positions_mm,
        results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .size_mm,
        simulation_config.sample_rate_hz,
        &results
            .model
            .as_ref()
            .unwrap()
            .spatial_description
            .voxels
            .numbers,
        Some(path.as_path()),
        Some(PlotSlice::Z(0)),
        Some(StateSphericalPlotMode::ABS),
        Some(playback_speed),
        Some(fps),
    )
    .unwrap();
}
