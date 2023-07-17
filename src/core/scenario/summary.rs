use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub loss: f32,
    pub delta_states_mean: f32,
    pub delta_states_max: f32,
    pub delta_measurements_mean: f32,
    pub delta_measurements_max: f32,
    pub delta_gains_mean: f32,
    pub delta_gains_max: f32,
    pub delta_delays_mean: f32,
    pub delta_delays_max: f32,
}

impl Summary {
    pub fn new() -> Summary {
        Summary {
            loss: 0.0,
            delta_states_mean: 0.0,
            delta_states_max: 0.0,
            delta_measurements_mean: 0.0,
            delta_measurements_max: 0.0,
            delta_gains_mean: 0.0,
            delta_gains_max: 0.0,
            delta_delays_mean: 0.0,
            delta_delays_max: 0.0,
        }
    }
}
