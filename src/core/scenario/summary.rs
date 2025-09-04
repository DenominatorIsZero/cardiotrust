use serde::{Deserialize, Serialize};
use tracing::trace;

/// Summary contains summary statistics for evaluating a scenario.
///
/// Fields:
///
/// - `loss`: The total loss for the scenario.
/// - `loss_mse`: The MSE loss for the scenario.
/// - `loss_maximum_regularization`: The maximum regularization loss.
/// - `delta_states_mean`: Mean delta across all state dimensions.
/// - `delta_states_max`: Max delta across all state dimensions.
/// - `delta_measurements_mean`: Mean delta across all measurement dimensions.
/// - `delta_measurements_max`: Max delta across all measurement dimensions.
/// - `delta_gains_mean`: Mean delta across all gain dimensions.
/// - `delta_gains_max`: Max delta across all gain dimensions.  
/// - `delta_delays_mean`: Mean delta across all delay dimensions.
/// - `delta_delays_max`: Max delta across all delay dimensions.
/// - `dice`: The DICE score.
/// - `iou`: The `IoU` score.
/// - `precision`: The precision.
/// - `recall`: The recall.
/// - `threshold`: The optimum classification threshold.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Summary {
    #[serde(default)]
    pub loss: f32,
    #[serde(default)]
    pub loss_mse: f32,
    #[serde(default)]
    pub loss_maximum_regularization: f32,
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
    /// Returns a `Summary` struct initialized with default values.
    ///
    /// Default values are 0.0 for all fields.
    #[tracing::instrument(level = "trace")]
    fn default() -> Self {
        trace!("Creating default summary");
        Self {
            loss: 0.0,
            loss_mse: 0.0,
            loss_maximum_regularization: 0.0,
            dice: 0.0,
            iou: 0.0,
            precision: 0.0,
            recall: 0.0,
            threshold: 0.0,
        }
    }
}
