pub mod algorithm;
pub mod model;
pub mod simulation;

use serde::{Deserialize, Serialize};
use tracing::info;

use self::{algorithm::Algorithm, simulation::Simulation};

/// Struct to hold the configuration for a simulation run.
///
/// Contains fields for:
///
/// - `measurement`: Path to the measurement data file.
/// - `simulation`: Simulation parameters.
/// - `algorithm`: Algorithm parameters.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub simulation: Simulation,
    pub algorithm: Algorithm,
}

impl Default for Config {
    /// Returns a default `Config` struct with `measurement` set to `None`.
    #[must_use]
    #[tracing::instrument(level = "info")]
    fn default() -> Self {
        info!("Creating default config");
        Self {
            simulation: Simulation::default(),
            algorithm: Algorithm::default(),
        }
    }
}

/// Enumeration of model presets.
///
/// `Healthy` refers to parameters for a normal, healthy heart model.
/// `Pathological` refers to parameters for a diseased, pathological heart model.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelPreset {
    Healthy,
    Pathological,
}
