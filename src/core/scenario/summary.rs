use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub loss: f32,
    pub delta_states_mean: f32,
}
