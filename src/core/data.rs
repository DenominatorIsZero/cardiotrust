pub mod measurement;
pub mod shapes;
pub mod simulation;

use ndarray::Dim;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{debug, trace};

use self::{measurement::Measurement, shapes::ArraySystemStates, simulation::Simulation};
use super::model::{
    functional::{
        allpass::shapes::{
            ArrayActivationTime, {ArrayDelays, ArrayGains},
        },
        control::ControlFunction,
    },
    spatial::voxels::VoxelTypes,
    Model,
};
use crate::core::{
    config::simulation::Simulation as SimulationConfig, data::shapes::ArrayMeasurements,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Data {
    pub simulation: Option<Simulation>,
    pub measurement: Option<Measurement>,
}

impl Data {
    /// Creates a new empty `Data` instance with the given dimensions.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
        sensor_motion_steps: usize,
    ) -> Self {
        debug!("Creating empty data");
        Self {
            simulation: Some(Simulation::empty(
                number_of_sensors,
                number_of_states,
                number_of_steps,
                voxels_in_dims,
                sensor_motion_steps,
            )),
            measurement: None,
        }
    }

    /// Creates a new [`Data`] instance from a [`SimulationConfig`].
    ///
    /// Runs the simulation using the provided config, and stores the result in a new `Data` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the `Simulation` from the config fails.
    #[tracing::instrument(level = "debug")]
    pub fn from_simulation_config(config: &SimulationConfig) -> Result<Self, Box<dyn Error>> {
        debug!("Creating data from simulation config");
        let mut simulation = Simulation::from_config(config)?;
        simulation.run();
        simulation.update_activation_time();
        Ok(Self {
            simulation: Some(simulation),
            measurement: None,
        })
    }

    /// # Panics
    ///
    /// Panics if simulation and measurement are both None.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_measurements(&self) -> &ArrayMeasurements {
        trace!("Getting measurements");
        self.simulation.as_ref().map_or_else(
            || {
                &(self
                    .measurement
                    .as_ref()
                    .expect("Measurement to be some")
                    .measurements)
            },
            |simulation| &(simulation.measurements),
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_system_states(&self) -> &ArraySystemStates {
        trace!("Getting system states");
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.system_states,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_control_function_values(&self) -> &ControlFunction {
        trace!("Getting control function values");
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| {
                &simulation
                    .model
                    .functional_description
                    .control_function_values
            },
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_gains(&self) -> &ArrayGains<f32> {
        trace!("Getting gains");
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.model.functional_description.ap_params.gains,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_coefs(&self) -> &ArrayDelays<f32> {
        trace!("Getting coefs");
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.model.functional_description.ap_params.coefs,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_voxel_types(&self) -> &VoxelTypes {
        trace!("Getting voxel types");
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.model.spatial_description.voxels.types,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_model(&self) -> &Model {
        trace!("Getting model");
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.model,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_activation_time_ms(&self) -> &ArrayActivationTime {
        trace!("Getting activation time");
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| {
                &simulation
                    .model
                    .functional_description
                    .ap_params
                    .activation_time_ms
            },
        )
    }

    // # Panics
    ///
    /// Panics if simulation is None
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn get_delays(&self) -> &ArrayDelays<usize> {
        trace!("Getting delays");
        self.simulation.as_ref().map_or_else(
            || todo!("Non simulation case not implemented yet."),
            |simulation| &simulation.model.functional_description.ap_params.delays,
        )
    }

    /// # Panics
    ///
    /// Panics if simulation is none.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving data to npy");
        self.simulation
            .as_ref()
            .expect("Simulation to be some.")
            .save_npy(&path.join("simulation"));
    }
}
