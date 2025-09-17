#[cfg(test)]
mod tests;

use anyhow::Result;
use ndarray::Dim;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, trace};

use super::shapes::{
    ActivationTimePerStateMs, SystemStates, SystemStatesSpherical, SystemStatesSphericalMax,
};
use crate::core::{
    algorithm::{
        estimation::{prediction::calculate_system_prediction, Estimations},
        refinement::derivation::{calculate_average_delays, AverageDelays},
    },
    config::{model::SensorArrayMotion, simulation::Simulation as SimulationConfig},
    data::Measurements,
    model::Model,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub measurements: Measurements,
    pub system_states: SystemStates,
    pub system_states_spherical: SystemStatesSpherical,
    pub system_states_spherical_max: SystemStatesSphericalMax,
    pub activation_times: ActivationTimePerStateMs,
    pub average_delays: AverageDelays,
    pub sample_rate_hz: f32,
    pub model: Model,
}
impl Simulation {
    /// Creates an empty Simulation with the given dimensions and number of
    /// sensors, states, and steps.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
        sensor_motion_steps: usize,
    ) -> Self {
        debug!("Creating empty simulation");
        Self {
            measurements: Measurements::empty(
                sensor_motion_steps,
                number_of_steps,
                number_of_sensors,
            ),
            system_states: SystemStates::empty(number_of_steps, number_of_states),
            system_states_spherical: SystemStatesSpherical::empty(
                number_of_steps,
                number_of_states,
            ),
            system_states_spherical_max: SystemStatesSphericalMax::empty(number_of_states),
            activation_times: ActivationTimePerStateMs::empty(number_of_states),
            average_delays: AverageDelays::empty(number_of_states),
            sample_rate_hz: 1.0,
            model: Model::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
                voxels_in_dims,
                sensor_motion_steps,
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
    #[tracing::instrument(level = "debug")]
    pub fn from_config(config: &SimulationConfig) -> Result<Self> {
        debug!("Creating simulation from config");
        let model =
            Model::from_model_config(&config.model, config.sample_rate_hz, config.duration_s)?;
        let number_of_sensors = model.spatial_description.sensors.count();
        let number_of_states = model.spatial_description.voxels.count_states();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let number_of_steps = (config.sample_rate_hz * config.duration_s) as usize;
        let number_of_beats = match config.model.common.sensor_array_motion {
            SensorArrayMotion::Static => 1,
            SensorArrayMotion::Grid => config
                .model
                .common
                .sensor_array_motion_steps
                .iter()
                .product(),
        };

        let measurements = Measurements::empty(number_of_beats, number_of_steps, number_of_sensors);
        let system_states = SystemStates::empty(number_of_steps, number_of_states);
        let system_states_spherical =
            SystemStatesSpherical::empty(number_of_steps, number_of_states);
        let system_states_spherical_max = SystemStatesSphericalMax::empty(number_of_states);
        let activation_times = ActivationTimePerStateMs::empty(number_of_states);
        let average_delays = AverageDelays::empty(number_of_states);

        Ok(Self {
            measurements,
            system_states,
            system_states_spherical,
            system_states_spherical_max,
            activation_times,
            average_delays,
            sample_rate_hz: config.sample_rate_hz,
            model,
        })
    }

    /// Runs a simulation by calculating system predictions, adding measurement
    /// noise, and storing results in the measurements and `system_states` fields.
    ///
    /// # Errors
    ///
    /// Returns an error if measurement noise configuration fails (negative covariance values).
    #[tracing::instrument(level = "info", skip_all)]
    pub fn run(&mut self) -> Result<()> {
        info!("Running simulation");

        let mut estimations = Estimations::empty(
            self.system_states.num_states(),
            self.measurements.num_sensors(),
            self.measurements.num_steps(),
            self.measurements.num_beats(),
        );

        for beat in 0..self.measurements.num_beats() {
            estimations.reset();
            for step in 0..self.measurements.num_steps() {
                calculate_system_prediction(
                    &mut estimations,
                    &self.model.functional_description,
                    beat,
                    step,
                );
            }
        }

        self.measurements.assign(&*estimations.measurements);
        self.system_states.assign(&*estimations.system_states);

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for sensor_index in 0..self.measurements.num_sensors() {
            let dist = Normal::new(
                0.0,
                self.model.functional_description.measurement_covariance
                    [[sensor_index, sensor_index]],
            )
            .map_err(|e| anyhow::anyhow!("Failed to create measurement noise distribution for sensor {}: {}", sensor_index, e))?;
            for beat_index in 0..self.measurements.num_beats() {
                for time_index in 0..self.measurements.num_steps() {
                    self.measurements[[beat_index, time_index, sensor_index]] +=
                        dist.sample(&mut rng);
                }
            }
        }
        self.calculate_plotting_arrays()?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn calculate_plotting_arrays(&mut self) -> anyhow::Result<()> {
        let system_states = &mut self.system_states;
        self.system_states_spherical.calculate(system_states);
        self.system_states_spherical_max
            .calculate(&self.system_states_spherical);
        self.activation_times
            .calculate(&self.system_states_spherical, self.sample_rate_hz);
        calculate_average_delays(
            &mut self.average_delays,
            &self.model.functional_description.ap_params,
        )?;
        Ok(())
    }

    /// Saves the simulation data (measurements, system states, model) to `NumPy` files at the given path.
    /// The measurements, system states, and model are saved to separate .npy files.
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O operation fails.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) -> anyhow::Result<()> {
        trace!("Saving simulation data to npy");
        self.measurements.save_npy(path)?;
        self.system_states.save_npy(path)?;
        self.model.save_npy(path)?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn update_activation_time(&mut self) {
        self.model.update_activation_time(&self.activation_times);
    }
}
