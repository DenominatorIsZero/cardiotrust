use approx::relative_eq;
use ndarray::Array2;
use ndarray_linalg::Inverse;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

use crate::core::config::model::Model;

use super::measurement::MeasurementMatrix;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Gain {
    pub values: Array2<f32>,
}

impl Gain {
    #[must_use]
    pub fn empty(number_of_states: usize, number_of_sensors: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_states, number_of_sensors)),
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if covariances are invalid.
    #[must_use]
    pub fn from_model_config(config: &Model, measurement_matrix: &MeasurementMatrix) -> Self {
        let mut process_covariance = Array2::<f32>::zeros((
            measurement_matrix.values.shape()[1],
            measurement_matrix.values.shape()[1],
        ));
        let mut measurement_covariance = Array2::<f32>::zeros((
            measurement_matrix.values.shape()[0],
            measurement_matrix.values.shape()[0],
        ));

        if relative_eq!(config.process_covariance_std, 0.0) {
            process_covariance
                .diag_mut()
                .fill(config.process_covariance_mean);
        } else {
            let normal = Normal::<f32>::new(
                config.process_covariance_mean,
                config.process_covariance_std,
            )
            .unwrap();
            process_covariance.diag_mut().iter_mut().for_each(|v| {
                *v = normal.sample(&mut rand::thread_rng());
            });
        }

        if relative_eq!(config.measurement_covariance_std, 0.0) {
            measurement_covariance
                .diag_mut()
                .fill(config.measurement_covariance_mean);
        } else {
            let normal = Normal::<f32>::new(
                config.measurement_covariance_mean,
                config.measurement_covariance_std,
            )
            .unwrap();
            measurement_covariance.diag_mut().iter_mut().for_each(|v| {
                *v = normal.sample(&mut rand::thread_rng());
            });
        }

        let h = &measurement_matrix.values;

        let s = h.dot(&process_covariance).dot(&h.t()) + measurement_covariance;
        let s_inv = s.inv().unwrap();
        let k = process_covariance.dot(&h.t()).dot(&s_inv);

        Self { values: k }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::model::spatial::SpatialDescription;

    use super::*;

    #[test]
    fn from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config);
        let measurement_matrix =
            MeasurementMatrix::from_model_config(&config, &spatial_description);

        let _kalman_gain = Gain::from_model_config(&config, &measurement_matrix);
    }
}
