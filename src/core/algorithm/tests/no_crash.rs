use std::path::Path;

use ndarray::Dim;

use crate::core::config::algorithm::Algorithm as AlgorithmConfig;
use crate::core::config::model::{SensorArrayGeometry, SensorArrayMotion};
use crate::core::config::simulation::Simulation as SimulationConfig;
use crate::core::model::Model;

use super::super::*;
use super::run;

const COMMON_PATH: &str = "tests/core/algorithm/no_crash";

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
fn run_epoch_no_crash() {
    let number_of_states = 3000;
    let number_of_sensors = 300;
    let number_of_steps = 3;
    let number_of_epochs = 10;
    let config = AlgorithmConfig::default();
    let voxels_in_dims = Dim([1000, 1, 1]);
    let number_of_beats = 10;

    let mut functional_description = FunctionalDescription::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        number_of_beats,
        voxels_in_dims,
    );
    let mut results = Results::new(
        number_of_epochs,
        number_of_steps,
        number_of_sensors,
        number_of_states,
        number_of_beats,
        config.batch_size,
        config.optimizer,
    );
    let data = Data::empty(
        number_of_sensors,
        number_of_states,
        number_of_steps,
        voxels_in_dims,
        number_of_beats,
    );

    let mut batch_index = 0;
    run_epoch(
        &mut functional_description,
        &mut results,
        &mut batch_index,
        &data,
        &config,
    );
}

#[test]
fn run_no_crash() {
    let number_of_states = 3000;
    let number_of_sensors = 300;
    let number_of_steps = 3;
    let number_of_beats = 7;
    let voxels_in_dims = Dim([1000, 1, 1]);

    let algorithm_config = AlgorithmConfig {
        epochs: 3,
        ..Default::default()
    };
    let mut functional_description = FunctionalDescription::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        number_of_beats,
        voxels_in_dims,
    );
    let mut results = Results::new(
        algorithm_config.epochs,
        number_of_steps,
        number_of_sensors,
        number_of_states,
        number_of_beats,
        algorithm_config.batch_size,
        algorithm_config.optimizer,
    );
    let data = Data::empty(
        number_of_sensors,
        number_of_states,
        number_of_steps,
        voxels_in_dims,
        number_of_beats,
    );

    run(
        &mut functional_description,
        &mut results,
        &data,
        &algorithm_config,
    );
}

#[test]
fn pseudo_inverse_success() {
    let mut simulation_config = SimulationConfig::default();
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
        algorithm_config.batch_size,
        algorithm_config.optimizer,
    );

    calculate_pseudo_inverse(
        &model.functional_description,
        &mut results,
        &data,
        &algorithm_config,
    );
}
