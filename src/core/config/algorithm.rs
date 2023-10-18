use serde::{Deserialize, Serialize};

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
    pub learning_rate_reduction_factor: f32,
    #[serde(default)]
    pub learning_rate_reduction_interval: usize,
    #[serde(default)]
    pub regularization_strength: f32,
    pub model: Model,
    #[serde(default)]
    pub constrain_system_states: bool,
    pub freeze_gains: bool,
    pub freeze_delays: bool,
    pub calculate_kalman_gain: bool,
}
impl Default for Algorithm {
    #[must_use]
    fn default() -> Self {
        Self {
            algorithm_type: AlgorithmType::ModelBased,
            epochs: 1,
            batch_size: 0,
            snapshots_interval: 0,
            learning_rate: 1e-3,
            learning_rate_reduction_factor: 0.0,
            learning_rate_reduction_interval: 0,
            regularization_strength: 0.1,
            model: Model::default(),
            constrain_system_states: true,
            freeze_gains: false,
            freeze_delays: false,
            calculate_kalman_gain: false,
        }
    }
}
