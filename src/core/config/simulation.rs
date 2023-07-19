use serde::{Deserialize, Serialize};

use super::model::Model;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Simulation {
    pub sample_rate_hz: f32,
    pub duration_s: f32,
    pub model: Model,
}
impl Default for Simulation {
    fn default() -> Self {
        Self {
            sample_rate_hz: 2000.0,
            duration_s: 1.0,
            model: Model::default(),
        }
    }
}
