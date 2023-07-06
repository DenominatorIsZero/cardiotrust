use serde::{Deserialize, Serialize};

use super::model::Model;
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Algorithm {
    pub epochs: usize,
    pub snapshots_interval: usize,
    pub learning_rate: f32,
    pub model: Model,
}
impl Algorithm {
    pub fn default() -> Algorithm {
        Algorithm {
            epochs: 1,
            snapshots_interval: 0,
            learning_rate: 1e-3,
            model: Model::default(),
        }
    }
}
