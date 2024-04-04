use serde::{Deserialize, Serialize};
use tracing::debug;

use super::model::Model;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Simulation {
    pub model: Model,
    pub sample_rate_hz: f32,
    pub duration_s: f32,
}
impl Default for Simulation {
    /// Returns a default `Simulation` struct with sample rate 2000 Hz,
    /// duration 1 second, and default model.
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Creating default simulation");
        Self {
            model: Model::default(),
            sample_rate_hz: 2000.0,
            duration_s: 1.0,
        }
    }
}
