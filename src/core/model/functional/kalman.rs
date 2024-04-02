use approx::relative_eq;
use nalgebra::DMatrix;
use ndarray::Array2;
use ndarray_npy::WriteNpyExt;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};
use tracing::{debug, trace};

use super::measurement::MeasurementMatrix;
use crate::core::config::model::Model;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Gain {
    pub values: Array2<f32>,
}

impl Gain {
    /// Creates a new Gain with the given number of states and sensors,
    /// initializing the values to a matrix of zeros.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_states: usize, number_of_sensors: usize) -> Self {
        debug!("Creating empty gain matrix");
        Self {
            values: Array2::zeros((number_of_states, number_of_sensors)),
        }
    }

    /// Creates a new Gain matrix by calculating it from the provided model config and
    /// measurement matrix. The gain matrix is calculated as:
    /// K = P * H^T * (H * P * H^T + R)^-1
    /// Where P is the process covariance, H is the measurement matrix, and R is the
    /// measurement covariance.
    ///
    /// # Panics
    ///
    /// Panics if covariances are invalid.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(config: &Model, measurement_matrix: &MeasurementMatrix) -> Self {
        debug!("Creating gain matrix from model config");
        let mut process_covariance = Array2::<f32>::zeros((
            measurement_matrix.values.shape()[1],
            measurement_matrix.values.shape()[1],
        ));
        let mut measurement_covariance = Array2::<f32>::zeros((
            measurement_matrix.values.shape()[0],
            measurement_matrix.values.shape()[0],
        ));

        if relative_eq!(config.common.process_covariance_std, 0.0) {
            process_covariance
                .diag_mut()
                .fill(config.common.process_covariance_mean);
        } else {
            let normal = Normal::<f32>::new(
                config.common.process_covariance_mean,
                config.common.process_covariance_std,
            )
            .unwrap();
            process_covariance.diag_mut().iter_mut().for_each(|v| {
                *v = normal.sample(&mut rand::thread_rng());
            });
        }

        if relative_eq!(config.common.measurement_covariance_std, 0.0) {
            measurement_covariance
                .diag_mut()
                .fill(config.common.measurement_covariance_mean);
        } else {
            let normal = Normal::<f32>::new(
                config.common.measurement_covariance_mean,
                config.common.measurement_covariance_std,
            )
            .unwrap();
            measurement_covariance.diag_mut().iter_mut().for_each(|v| {
                *v = normal.sample(&mut rand::thread_rng());
            });
        }

        let h = &measurement_matrix.values;

        let s = h.dot(&process_covariance).dot(&h.t()) + measurement_covariance;
        let mut s = DMatrix::from_row_slice(
            s.shape()[0],
            s.shape()[1],
            s.as_slice().expect("Slice to be some."),
        );
        s.try_inverse_mut();
        let mut s_inv = Array2::<f32>::zeros(s.shape());
        s.iter()
            .zip(s_inv.iter_mut())
            .for_each(|(s, s_inv)| *s_inv = *s);
        let k = process_covariance.dot(&h.t()).dot(&s_inv);

        Self { values: k }
    }

    /// Saves the Kalman gain matrix to a .npy file at the given path.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving Kalman gain matrix to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("kalman_gain.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
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
