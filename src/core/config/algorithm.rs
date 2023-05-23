use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Algorithm {
    epochs: usize,
    snapshots_every_n: usize,
    learning_rate: f32,
    measurement_covariance_mean: f32,
    // the measurement noise covariance matrix will be a diagonal matrix
    // if std is set to zero, every value will be set to mean
    // otherwise the elements along the main diagonal will be drawn from a
    // normal distribution
    measurement_covariance_std: f32,
    process_covariance_mean: f32,
    // the covariance noise covariance matrix will be a diagonal matrix
    // if std is set to zero, every value will be set to mean
    // otherwise the elements along the main diagonal will be drawn from a
    // normal distribution
    process_covariance_std: f32,
    apply_system_update: bool,
}
impl Algorithm {
    pub fn default() -> Algorithm {
        Algorithm {
            epochs: 1,
            snapshots_every_n: 0,
            learning_rate: 1e8,
            measurement_covariance_mean: 1e-30,
            measurement_covariance_std: 0.0,
            process_covariance_mean: 1e-30,
            process_covariance_std: 0.0,
            apply_system_update: true,
        }
    }
}
