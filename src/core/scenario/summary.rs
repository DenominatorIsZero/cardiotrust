use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub loss: f32,
    pub loss_mse: f32,
    pub loss_maximum_regularization: f32,
    pub delta_states_mean: f32,
    pub delta_states_max: f32,
    pub delta_measurements_mean: f32,
    pub delta_measurements_max: f32,
    pub delta_gains_mean: f32,
    pub delta_gains_max: f32,
    pub delta_delays_mean: f32,
    pub delta_delays_max: f32,
    #[serde(default)]
    pub dice: f32,
    #[serde(default)]
    pub iou: f32,
    #[serde(default)]
    pub precision: f32,
    #[serde(default)]
    pub recall: f32,
    #[serde(default)]
    pub threshold: f32,
}

impl Default for Summary {
    #[must_use]
    fn default() -> Self {
        Self {
            loss: 0.0,
            loss_mse: 0.0,
            loss_maximum_regularization: 0.0,
            delta_states_mean: 0.0,
            delta_states_max: 0.0,
            delta_measurements_mean: 0.0,
            delta_measurements_max: 0.0,
            delta_gains_mean: 0.0,
            delta_gains_max: 0.0,
            delta_delays_mean: 0.0,
            delta_delays_max: 0.0,
            dice: 0.0,
            iou: 0.0,
            precision: 0.0,
            recall: 0.0,
            threshold: 0.0,
        }
    }
}
