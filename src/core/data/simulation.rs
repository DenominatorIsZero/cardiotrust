use ndarray::Dim;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::error::Error;

use super::shapes::ArraySystemStates;
use crate::core::{
    algorithm::estimation::prediction::calculate_system_prediction,
    config::simulation::Simulation as SimulationConfig,
    data::ArrayMeasurements,
    model::{functional::allpass::shapes::ArrayGains, Model},
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub measurements: ArrayMeasurements,
    pub system_states: ArraySystemStates,
    pub model: Model,
}
impl Simulation {
    /// Creates an empty Simulation with the given dimensions and number of
    /// sensors, states, and steps.
    #[must_use]
    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> Self {
        Self {
            measurements: ArrayMeasurements::empty(number_of_steps, number_of_sensors),
            system_states: ArraySystemStates::empty(number_of_steps, number_of_states),
            model: Model::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
                voxels_in_dims,
            ),
        }
    }

    /// Creates a new Simulation instance from the provided `SimulationConfig`.
    ///
    /// Initializes an empty Simulation with the model, number of sensors, states,
    /// and time steps specified in the config. The model is validated before
    /// creating the Simulation.
    ///
    /// # Errors
    ///
    /// Returns an error if the model fails to initialize from the config.
    #[tracing::instrument]
    pub fn from_config(config: &SimulationConfig) -> Result<Self, Box<dyn Error>> {
        let model =
            Model::from_model_config(&config.model, config.sample_rate_hz, config.duration_s)?;
        let number_of_sensors = model.spatial_description.sensors.count();
        let number_of_states = model.spatial_description.voxels.count_states();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let number_of_steps = (config.sample_rate_hz * config.duration_s) as usize;

        let measurements = ArrayMeasurements::empty(number_of_steps, number_of_sensors);
        let system_states = ArraySystemStates::empty(number_of_steps, number_of_states);

        Ok(Self {
            measurements,
            system_states,
            model,
        })
    }

    /// Runs a simulation by calculating system predictions, adding measurement
    /// noise, and storing results in the measurements and `system_states` fields.
    ///
    /// # Panics
    ///
    /// if there are negative values in the measurement covariance matrix.
    #[tracing::instrument]
    pub fn run(&mut self) {
        let measurements = &mut self.measurements;
        let system_states = &mut self.system_states;
        let model = &self.model;

        let mut ap_outputs: ArrayGains<f32> = ArrayGains::empty(system_states.values.shape()[1]);
        for time_index in 0..system_states.values.shape()[0] {
            calculate_system_prediction(
                &mut ap_outputs,
                system_states,
                measurements,
                &model.functional_description,
                time_index,
            );
        }
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for sensor_index in 0..measurements.values.shape()[1] {
            let dist = Normal::new(
                0.0,
                model.functional_description.measurement_covariance.values
                    [[sensor_index, sensor_index]],
            )
            .unwrap();
            for time_index in 0..measurements.values.shape()[0] {
                measurements.values[[time_index, sensor_index]] += dist.sample(&mut rng);
            }
        }
    }

    /// Saves the simulation data (measurements, system states, model) to `NumPy` files at the given path.
    /// The measurements, system states, and model are saved to separate .npy files.
    #[tracing::instrument]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        self.measurements.save_npy(path);
        self.system_states.save_npy(path);
        self.model.save_npy(path);
    }
}

#[cfg(test)]
mod test {
    use approx::{assert_relative_eq, RelativeEq};

    use ndarray::s;
    use ndarray_stats::QuantileExt;

    use crate::{
        core::model::spatial::voxels::VoxelType,
        vis::plotting::{
            matrix::{plot_states_at_time, plot_states_max, plot_states_over_time},
            time::{plot_state_xyz, standard_time_plot},
        },
    };

    use super::*;

    #[test]
    fn create_simulation_no_crash() {
        let config = &SimulationConfig::default();
        let simulation = Simulation::from_config(config);
        assert!(simulation.is_ok());
        let simulation = simulation.unwrap();
        let max = *simulation.system_states.values.max_skipnan();
        assert_relative_eq!(max, 0.0);
        let max = *simulation.measurements.values.max_skipnan();
        assert_relative_eq!(max, 0.0);
    }

    #[test]
    fn run_simulation_default() {
        let config = &SimulationConfig::default();
        let mut simulation = Simulation::from_config(config).unwrap();
        simulation.run();
        let max = *simulation.system_states.values.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.001, 0.001));
        let max = *simulation.measurements.values.max_skipnan();
        assert!(max > 0.0);
    }

    #[test]
    #[ignore]
    fn run_simulation_default_and_plot() {
        let config = &SimulationConfig::default();
        let mut simulation = Simulation::from_config(config).unwrap();
        simulation.run();
        let max = *simulation.system_states.values.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.001, 0.001));
        let max = *simulation.measurements.values.max_skipnan();
        assert!(max > 0.0);

        let sa_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Sinoatrial);

        plot_state_xyz(
            &simulation.system_states,
            sa_index,
            config.sample_rate_hz,
            "tests/simulation_sa",
            "Simulated Current Density Sinoatrial Node",
        );

        let av_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Atrioventricular);
        plot_state_xyz(
            &simulation.system_states,
            av_index,
            config.sample_rate_hz,
            "tests/simulation_av",
            "Simulated Current Density Atrioventricular Node",
        );

        standard_time_plot(
            &simulation.measurements.values.slice(s![.., 0]).to_owned(),
            config.sample_rate_hz,
            "tests/simulation_sensor_0_x",
            "Simulated Measurement Sensor 0 - x",
            "H [pT]",
        );
        standard_time_plot(
            &simulation.measurements.values.slice(s![.., 1]).to_owned(),
            config.sample_rate_hz,
            "tests/simulation_sensor_0_y",
            "Simulated Measurement Sensor 0 - y",
            "H [pT]",
        );
        standard_time_plot(
            &simulation.measurements.values.slice(s![.., 2]).to_owned(),
            config.sample_rate_hz,
            "tests/simulation_sensor_0_z",
            "Simulated Measurement Sensor 0 - z",
            "H [pT]",
        );

        let time_index = simulation.system_states.values.shape()[0] / 3;

        plot_states_at_time(
            &simulation.system_states,
            &simulation.model.spatial_description.voxels,
            f32::MAX,
            f32::MIN,
            time_index,
            &format!("tests/simulation_states_{time_index}"),
            &format!("Simulated Current Densities at Time Index {time_index}"),
        );

        plot_states_max(
            &simulation.system_states,
            &simulation.model.spatial_description.voxels,
            "tests/simulation_states_max",
            "Maximum Simulated Current Densities",
        );

        let fps = 20;
        let playback_speed = 0.1;
        plot_states_over_time(
            &simulation.system_states,
            &simulation.model.spatial_description.voxels,
            fps,
            playback_speed,
            "tests/simulation_states",
            "Simulated Current Densities",
        );
    }

    #[test]
    fn run_simulation_pathological() {
        let mut config = SimulationConfig::default();
        config.model.pathological = true;
        let mut simulation = Simulation::from_config(&config).unwrap();
        simulation.run();
        let max = *simulation.system_states.values.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.001, 0.001));
        let max = *simulation.measurements.values.max_skipnan();
        assert!(max > 0.0);
    }

    #[test]
    #[ignore]
    fn run_simulation_pathological_and_plot() {
        let mut config = SimulationConfig::default();
        config.model.pathological = true;
        let mut simulation = Simulation::from_config(&config).unwrap();
        simulation.run();
        let max = *simulation.system_states.values.max_skipnan();
        assert!(max.relative_eq(&1.0, 0.001, 0.001));
        let max = *simulation.measurements.values.max_skipnan();
        assert!(max > 0.0);

        let sa_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Sinoatrial);

        plot_state_xyz(
            &simulation.system_states,
            sa_index,
            config.sample_rate_hz,
            "tests/simulation_sa_pathological",
            "Simulated Current Density Sinoatrial Node",
        );

        let av_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Atrioventricular);
        plot_state_xyz(
            &simulation.system_states,
            av_index,
            config.sample_rate_hz,
            "tests/simulation_av_pathological",
            "Simulated Current Density Atrioventricular Node",
        );

        let pathology_index = simulation
            .model
            .spatial_description
            .voxels
            .get_first_state_of_type(VoxelType::Pathological);
        plot_state_xyz(
            &simulation.system_states,
            pathology_index,
            config.sample_rate_hz,
            "tests/simulation_pathological",
            "Simulated Current Density Pathological Voxel",
        );

        standard_time_plot(
            &simulation.measurements.values.slice(s![.., 0]).to_owned(),
            config.sample_rate_hz,
            "tests/simulation_sensor_0_x_pathological",
            "Simulated Measurement Sensor 0 - x",
            "H [pT]",
        );
        standard_time_plot(
            &simulation.measurements.values.slice(s![.., 1]).to_owned(),
            config.sample_rate_hz,
            "tests/simulation_sensor_0_y_pathological",
            "Simulated Measurement Sensor 0 - y",
            "H [pT]",
        );
        standard_time_plot(
            &simulation.measurements.values.slice(s![.., 2]).to_owned(),
            config.sample_rate_hz,
            "tests/simulation_sensor_0_z_pathological",
            "Simulated Measurement Sensor 0 - z",
            "H [pT]",
        );

        let time_index = simulation.system_states.values.shape()[0] / 3;

        plot_states_at_time(
            &simulation.system_states,
            &simulation.model.spatial_description.voxels,
            f32::MAX,
            f32::MIN,
            time_index,
            &format!("tests/simulation_states_{time_index}_pathological"),
            &format!("Simulated Current Densities at Time Index {time_index}"),
        );

        plot_states_max(
            &simulation.system_states,
            &simulation.model.spatial_description.voxels,
            "tests/simulation_states_max_pathological",
            "Maximum Simulated Current Densities",
        );

        let fps = 20;
        let playback_speed = 0.1;
        plot_states_over_time(
            &simulation.system_states,
            &simulation.model.spatial_description.voxels,
            fps,
            playback_speed,
            "tests/simulation_states_pathological",
            "Simulated Current Densities",
        );
    }
}
