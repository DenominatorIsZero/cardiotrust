use approx::relative_eq;
use ndarray::{s, Array2};
use ndarray_npy::WriteNpyExt;
use physical_constants::VACUUM_MAG_PERMEABILITY;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::{
    f32::consts::PI,
    fs::{self, File},
    io::BufWriter,
};
use tracing::{debug, trace};

use crate::core::{
    config::model::Model,
    model::spatial::{voxels::VoxelType, SpatialDescription},
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions, clippy::unsafe_derive_deserialize)]
pub struct MeasurementMatrix {
    pub values: Array2<f32>,
}

impl MeasurementMatrix {
    /// Creates a new `MeasurementMatrix` with the given number of sensor states
    /// and number of sensors, initializing all values to 0.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_states: usize, number_of_sensors: usize) -> Self {
        debug!("Creating empty measurement matrix");
        Self {
            values: Array2::zeros((number_of_sensors, number_of_states)),
        }
    }

    /// Creates a new `MeasurementMatrix` from the given `Model` config and
    /// `SpatialDescription`. Initializes the matrix values by calculating the
    /// magnetic flux density at each sensor position for each voxel, based on
    /// voxel type, position, sensor position and orientation.
    /// Uses the Biot-Savart law to calculate the magnetic flux density.
    ///
    /// # Panics
    ///
    /// Panics if voxel numbers are not initialized correctly.
    #[must_use]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_model_config(config: &Model, spatial_description: &SpatialDescription) -> Self {
        debug!("Creating measurement matrix from model config");
        let mut measurement_matrix = Self::empty(
            spatial_description.voxels.count_states(),
            spatial_description.sensors.count(),
        );

        let m = &mut measurement_matrix.values;

        let types = &spatial_description.voxels.types.values;
        let voxel_numbers = &spatial_description.voxels.numbers.values;
        let voxel_positions_mm = &spatial_description.voxels.positions_mm.values;
        let sensor_positions = &spatial_description.sensors.positions_mm;
        let sensor_orientations = &spatial_description.sensors.orientations_xyz;

        let voxel_volume_m3 = (spatial_description.voxels.size_mm / 1000.0).powi(3);

        #[allow(clippy::cast_possible_truncation)]
        let common_factor = (VACUUM_MAG_PERMEABILITY as f32 * voxel_volume_m3) / (4.0 * PI) * 1e12;

        for (index, v_type) in types.indexed_iter() {
            if !v_type.is_connectable() {
                continue;
            }

            let v_num = voxel_numbers[index].unwrap();
            let v_pos_mm = voxel_positions_mm.slice(s![index.0, index.1, index.2, ..]);

            for s_num in 0..spatial_description.sensors.count() {
                let s_pos_mm = sensor_positions.slice(s![s_num, ..]);
                let s_ori = sensor_orientations.slice(s![s_num, ..]);

                let distace_m = (&s_pos_mm - &v_pos_mm) / 1000.0;
                let distance_cubed_m3 = distace_m.mapv(|v| v.powi(2)).sum().sqrt().powi(3);

                m[(s_num, v_num)] = common_factor
                    * s_ori[2].mul_add(distace_m[1], -s_ori[1] * distace_m[2])
                    / distance_cubed_m3;
                m[(s_num, v_num + 1)] = common_factor
                    * s_ori[0].mul_add(distace_m[2], -s_ori[2] * distace_m[0])
                    / distance_cubed_m3;
                m[(s_num, v_num + 2)] = common_factor
                    * s_ori[1].mul_add(distace_m[0], -s_ori[0] * distace_m[1])
                    / distance_cubed_m3;
            }
        }

        measurement_matrix
    }

    /// Saves the measurement matrix to a .npy file at the given path.
    /// Creates the directory if it does not exist.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving measurement matrix to npy file");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("measurement_matrix.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions, clippy::unsafe_derive_deserialize)]
pub struct MeasurementCovariance {
    pub values: Array2<f32>,
}

impl MeasurementCovariance {
    /// Creates a new `MeasurementCovariance` with the given number of sensors,
    /// initialized to all zeros.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_sensors: usize) -> Self {
        debug!("Creating empty measurement covariance");
        Self {
            values: Array2::zeros((number_of_sensors, number_of_sensors)),
        }
    }

    /// Creates a new `MeasurementCovariance` initialized from the model
    /// configuration. The diagonal is filled with random values drawn from
    /// a normal distribution with the configured mean and standard deviation.
    /// If the standard deviation is 0, the diagonal is filled with the mean.
    //
    /// # Panics
    ///
    /// Panics if voxel numbers are not initialized correctly.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(config: &Model, spatial_description: &SpatialDescription) -> Self {
        debug!("Creating measurement covariance from model config");
        let mut measurement_covariance = Self::empty(spatial_description.sensors.count());

        if relative_eq!(config.common.measurement_covariance_std, 0.0) {
            measurement_covariance
                .values
                .diag_mut()
                .fill(config.common.measurement_covariance_mean);
        } else {
            let normal = Normal::<f32>::new(
                config.common.measurement_covariance_mean,
                config.common.measurement_covariance_std,
            )
            .unwrap();
            measurement_covariance
                .values
                .diag_mut()
                .iter_mut()
                .for_each(|v| {
                    *v = normal.sample(&mut rand::thread_rng());
                });
        }

        measurement_covariance
    }

    /// Saves the process covariance matrix to a .npy file at the given path.
    /// Creates the directory if it does not exist.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving process covariance matrix to npy file");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("process_covariance.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{core::config::model::Common, vis::plotting::png::matrix::matrix_plot};

    use super::*;

    const COMMON_PATH: &str = "tests/core/model/functional/measurement/";

    #[tracing::instrument(level = "trace")]
    fn setup(folder: Option<&str>) {
        let path = folder.map_or_else(
            || Path::new(COMMON_PATH).to_path_buf(),
            |folder| Path::new(COMMON_PATH).join(folder),
        );

        if !path.exists() {
            std::fs::create_dir_all(path).unwrap();
        }
    }

    #[test]
    fn from_model_config_no_crash() {
        let config = Model {
            common: Common {
                sensors_per_axis: [3, 3, 3],
                voxel_size_mm: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let spatial_description = SpatialDescription::from_model_config(&config);

        let measurement_matrix =
            MeasurementMatrix::from_model_config(&config, &spatial_description);

        assert!(!measurement_matrix.values.is_empty());
    }

    #[test]
    fn from_model_config_no_crash_and_plot() {
        setup(None);
        let config = Model {
            common: Common {
                sensors_per_axis: [3, 3, 3],
                voxel_size_mm: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let spatial_description = SpatialDescription::from_model_config(&config);

        let measurement_matrix =
            MeasurementMatrix::from_model_config(&config, &spatial_description);

        assert!(!measurement_matrix.values.is_empty());

        let path = Path::new(COMMON_PATH).join("measurement_matrix_default.png");
        matrix_plot(
            &measurement_matrix.values,
            None,
            None,
            None,
            Some(path.as_path()),
            Some("Default Measurement Matrix"),
            Some("State Index"),
            Some("Sensor Index"),
            Some("[pT / A / m^2]"),
            None,
            None,
        )
        .unwrap();
    }
}
