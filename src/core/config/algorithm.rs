use serde::{Deserialize, Serialize};

use super::model::Model;
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Algorithm {
    pub epochs: usize,
    pub snapshots_interval: usize,
    pub learning_rate: f32,
    pub model: Model,
    pub constrain_current_density: bool,
}
impl Default for Algorithm {
    #[must_use]
    fn default() -> Self {
        Self {
            epochs: 1,
            snapshots_interval: 0,
            learning_rate: 1e-3,
            model: Model::default(),
            constrain_current_density: true,
        }
    }
}
