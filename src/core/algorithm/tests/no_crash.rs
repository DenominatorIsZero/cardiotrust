use ndarray::Dim;

use super::{super::*, run};
use crate::core::{
    config::{
        algorithm::Algorithm as AlgorithmConfig,
        model::{SensorArrayGeometry, SensorArrayMotion},
        simulation::Simulation as SimulationConfig,
    },
    model::Model,
};

#[test]
fn run_epoch_no_crash() -> anyhow::Result<()> {
    let number_of_states = 3000;
    let number_of_sensors = 300;
    let number_of_steps = 3;
    let number_of_epochs = 10;
    let number_of_snapshots = 0;
    let config = AlgorithmConfig::default();
    let voxels_in_dims = Dim([1000, 1, 1]);
    let number_of_beats = 10;

    let model = Model::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        voxels_in_dims,
        number_of_beats,
    );

    let mut results = Results::new(
        number_of_epochs,
        number_of_steps,
        number_of_sensors,
        number_of_states,
        number_of_beats,
        number_of_snapshots,
        config.batch_size,
        config.optimizer,
    );
    results.model = Some(model);
    let data = Data::empty(
        number_of_sensors,
        number_of_states,
        number_of_steps,
        voxels_in_dims,
        number_of_beats,
    );

    let mut batch_index = 0;
    run_epoch(&mut results, &mut batch_index, &data, &config)?;
    Ok(())
}

#[test]
fn run_no_crash() -> anyhow::Result<()> {
    let number_of_states = 3000;
    let number_of_sensors = 300;
    let number_of_steps = 3;
    let number_of_beats = 7;
    let number_of_snapshots = 0;
    let voxels_in_dims = Dim([1000, 1, 1]);

    let algorithm_config = AlgorithmConfig {
        epochs: 3,
        ..Default::default()
    };
    let model = Model::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        voxels_in_dims,
        number_of_beats,
    );
    let mut results = Results::new(
        algorithm_config.epochs,
        number_of_steps,
        number_of_sensors,
        number_of_states,
        number_of_beats,
        number_of_snapshots,
        algorithm_config.batch_size,
        algorithm_config.optimizer,
    );
    results.model = Some(model);
    let data = Data::empty(
        number_of_sensors,
        number_of_states,
        number_of_steps,
        voxels_in_dims,
        number_of_beats,
    );

    run(&mut results, &data, &algorithm_config)?;
    Ok(())
}

#[test]
fn pseudo_inverse_success() -> anyhow::Result<()> {
    let mut simulation_config = SimulationConfig::default();
    simulation_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
    simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
    let data = Data::from_simulation_config(&simulation_config)?;

    let mut algorithm_config = Algorithm::default();
    algorithm_config.model.common.sensor_array_geometry = SensorArrayGeometry::Cube;
    algorithm_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

    let model = Model::from_model_config(
        &algorithm_config.model,
        simulation_config.sample_rate_hz,
        simulation_config.duration_s,
    )?;

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

    calculate_pseudo_inverse(
        &model.functional_description,
        &mut results,
        &data,
        &algorithm_config,
    )?;
    Ok(())
}
