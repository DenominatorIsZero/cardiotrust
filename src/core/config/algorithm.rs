use serde::{Deserialize, Serialize};
use tracing::debug;

use super::model::Model;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[allow(clippy::module_name_repetitions)]
pub enum AlgorithmType {
    #[default]
    ModelBased,
    PseudoInverse,
    Loreta,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Algorithm {
    #[serde(default)]
    pub algorithm_type: AlgorithmType,
    pub epochs: usize,
    #[serde(default)]
    pub batch_size: usize,
    pub snapshots_interval: usize,
    pub learning_rate: f32,
    #[serde(default)]
    pub gradient_clamping_threshold: f32,
    #[serde(default)]
    pub learning_rate_reduction_factor: f32,
    #[serde(default)]
    pub learning_rate_reduction_interval: usize,
    #[serde(default)]
    pub regularization_strength: f32,
    #[serde(default)]
    pub regularization_threshold: f32,
    pub model: Model,
    #[serde(default)]
    pub constrain_system_states: bool,
    #[serde(default)]
    pub state_clamping_threshold: f32,
    pub freeze_gains: bool,
    pub freeze_delays: bool,
    pub calculate_kalman_gain: bool,
}
impl Default for Algorithm {
    /// Returns a default `Algorithm` configuration with reasonable defaults for most use cases.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Creating default algorithm");
        Self {
            algorithm_type: AlgorithmType::ModelBased,
            epochs: 1,
            batch_size: 0,
            snapshots_interval: 0,
            learning_rate: 200.0,
            gradient_clamping_threshold: 1.0,
            learning_rate_reduction_factor: 0.0,
            learning_rate_reduction_interval: 0,
            regularization_strength: 1.0,
            regularization_threshold: 1.01,
            model: Model::default(),
            constrain_system_states: false,
            state_clamping_threshold: 1.5,
            freeze_gains: false,
            freeze_delays: true,
            calculate_kalman_gain: false,
        }
    }
}
