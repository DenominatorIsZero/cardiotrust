use serde::{Deserialize, Serialize};

use super::model::Model;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub enum Type {
    #[default]
    ModelBased,
    PseudoInverse,
    Loreta,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Algorithm {
    pub epochs: usize,
    #[serde(default)]
    pub batch_size: usize,
    pub snapshots_interval: usize,
    pub learning_rate: f32,
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
            epochs: 1,
            batch_size: 0,
            snapshots_interval: 0,
            learning_rate: 1e0,
            regularization_strength: 0.1,
            model: Model::default(),
            constrain_system_states: true,
            freeze_gains: false,
            freeze_delays: false,
            calculate_kalman_gain: false,
        }
    }
}
