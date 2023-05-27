use serde::{Deserialize, Serialize};

use super::ModelPreset;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Simulation {
    pub sample_rate_hz: f32,
    pub duration_s: f32,
    pub control_function: ControlFunction,
    pub model: ModelPreset,
    pub sensors_per_axis: [u32; 3],
    pub measurement_covariance_mean: f32,
    // the measurement noise covariance matrix will be a diagonal matrix
    // if std is set to zero, every value will be set to mean
    // otherwise the elements along the main diagonal will be drawn from a
    // normal distribution, clipped at np.finfo('float32').eps
    pub measurement_covariance_std: f32,
}
impl Simulation {
    pub fn default() -> Simulation {
        Simulation {
            sample_rate_hz: 2000.0,
            duration_s: 1.0,
            control_function: ControlFunction::Ohara,
            model: ModelPreset::Healthy,
            sensors_per_axis: [4, 4, 2],
            measurement_covariance_mean: 1e-30,
            measurement_covariance_std: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ControlFunction {
    Sinosodal,
    Ohara,
}
