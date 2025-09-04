use std::{
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};

use approx::relative_eq;
use nalgebra::DMatrix;
use ndarray::{s, Array2, ArrayBase, Dim, ViewRepr};
use ndarray_npy::WriteNpyExt;
use ocl::Buffer;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use super::measurement::MeasurementMatrix;
use crate::core::config::model::Model;

#[allow(clippy::unsafe_derive_deserialize, clippy::module_name_repetitions)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct KalmanGain(Array2<f32>);

impl KalmanGain {
    /// Creates a new Gain with the given number of states and sensors,
    /// initializing the values to a matrix of zeros.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_states: usize, number_of_sensors: usize) -> Self {
        debug!("Creating empty gain matrix");
        Self(Array2::zeros((number_of_states, number_of_sensors)))
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
        let mut process_covariance =
            Array2::<f32>::zeros((measurement_matrix.shape()[2], measurement_matrix.shape()[2]));
        let mut measurement_covariance =
            Array2::<f32>::zeros((measurement_matrix.shape()[1], measurement_matrix.shape()[1]));

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
                *v = normal.sample(&mut rand::rng());
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
                *v = normal.sample(&mut rand::rng());
            });
        }

        let h: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 2]>> =
            &measurement_matrix.slice(s![0, .., ..]);

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

        Self(k)
    }

    /// Saves the Kalman gain matrix to a .npy file at the given path.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving Kalman gain matrix to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("kalman_gain.npy")).unwrap());
        self.write_npy(writer).unwrap();
    }

    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> ocl::Buffer<f32> {
        Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(self.as_slice().unwrap())
            .build()
            .unwrap()
    }

    pub(crate) fn update_from_gpu(&mut self, kalman_gain: &Buffer<f32>) {
        kalman_gain
            .read(self.as_slice_mut().unwrap())
            .enq()
            .unwrap();
    }
}

impl Deref for KalmanGain {
    type Target = Array2<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KalmanGain {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Array2<f32> {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::spatial::SpatialDescription;

    #[test]
    fn from_model_config_no_crash() {
        let config = Model::default();
        let spatial_description = SpatialDescription::from_model_config(&config);
        let measurement_matrix =
            MeasurementMatrix::from_model_spatial_description(&spatial_description);

        let _kalman_gain = KalmanGain::from_model_config(&config, &measurement_matrix);
    }
}
