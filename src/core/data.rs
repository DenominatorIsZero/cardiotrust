pub mod shapes;
pub mod simulation;

use anyhow::{Context, Result};
use ndarray::Dim;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use self::simulation::Simulation;
use crate::core::{config::simulation::Simulation as SimulationConfig, data::shapes::Measurements};

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
    pub fn from_simulation_config(config: &SimulationConfig) -> Result<Self> {
        debug!("Creating data from simulation config");
        let mut simulation = Simulation::from_config(config)?;
        simulation.run()?;
        simulation.update_activation_time();
        Ok(Self { simulation })
    }

    /// # Panics
    ///
    /// Saves the data to NumPy files at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O operation fails.
    #[tracing::instrument(level = "trace")]
    pub fn save_npy(&self, path: &std::path::Path) -> anyhow::Result<()> {
        trace!("Saving data to npy");
        self.simulation.save_npy(&path.join("simulation"))?;
        Ok(())
    }

    #[allow(dead_code)]
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn get_default() -> Result<Self> {
        let mut sim_config = SimulationConfig::default();
        sim_config.model.common.pathological = true;

        Self::from_simulation_config(&sim_config)
            .context("Failed to create default simulation data")
    }
}
