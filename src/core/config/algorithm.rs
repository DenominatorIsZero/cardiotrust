use serde::{Deserialize, Serialize};

use super::model::Model;
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Algorithm {
    pub epochs: usize,
    pub snapshots_interval: usize,
    pub learning_rate: f32,
    #[serde(default)]
    pub regularization_strength: f32,
    pub model: Model,
    #[serde(default)]
    pub constrain_system_states: bool,
}
impl Default for Algorithm {
    #[must_use]
    fn default() -> Self {
        Self {
            epochs: 1,
            snapshots_interval: 0,
            learning_rate: 1e-5,
            regularization_strength: 0.1,
            model: Model::default(),
            constrain_system_states: true,
        }
    }
}
