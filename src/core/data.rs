pub mod shapes;
pub mod simulation;

use std::error::Error;

use ndarray::Dim;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::simulation::Simulation;
use crate::core::{
    config::{simulation::Simulation as SimulationConfig, Config},
    data::shapes::Measurements,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Data {
    pub simulation: Simulation,
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
        number_of_beats: usize,
    ) -> Self {
        debug!("Creating empty data");
        Self {
            simulation: Simulation::empty(
                number_of_sensors,
                number_of_states,
                number_of_steps,
                voxels_in_dims,
                number_of_beats,
            ),
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
        Ok(Self { simulation })
    }

    /// # Panics
    ///
    /// Panics if simulation is none.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving data to npy");
        self.simulation.save_npy(&path.join("simulation"));
    }

    pub(crate) fn get_default() -> Self {
        let mut sim_config = SimulationConfig::default();
        sim_config.model.common.pathological = true;

        Self::from_simulation_config(&sim_config).unwrap()
    }
}
