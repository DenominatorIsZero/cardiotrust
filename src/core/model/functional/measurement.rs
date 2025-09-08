use std::{
    f32::consts::PI,
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};

use approx::relative_eq;
use ndarray::{s, Array2, Array3, ArrayView2};
use ndarray_npy::WriteNpyExt;
use ocl::{Buffer, Queue};
use physical_constants::VACUUM_MAG_PERMEABILITY;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use crate::core::{config::model::Model, model::spatial::SpatialDescription};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions, clippy::unsafe_derive_deserialize)]
pub struct MeasurementMatrix(Array3<f32>);

impl MeasurementMatrix {
    /// Creates a new `MeasurementMatrix` with the given number of sensor states
    /// and number of sensors, initializing all values to 0.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(
        number_of_beats: usize,
        number_of_states: usize,
        number_of_sensors: usize,
    ) -> Self {
        debug!("Creating empty measurement matrix");
        Self(Array3::zeros((
            number_of_beats,
            number_of_sensors,
            number_of_states,
        )))
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn to_gpu(&self, queue: &Queue) -> Buffer<f32> {
        Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(self.as_slice().unwrap())
            .build()
            .unwrap()
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
    pub fn from_model_spatial_description(spatial_description: &SpatialDescription) -> Self {
        debug!("Creating measurement matrix from model config");
        let mut measurement_matrix = Self::empty(
            spatial_description.sensors.count_beats(),
            spatial_description.voxels.count_states(),
            spatial_description.sensors.count(),
        );

        let m = &mut measurement_matrix;

        let types = &spatial_description.voxels.types;
        let voxel_numbers = &spatial_description.voxels.numbers;
        let voxel_positions_mm = &spatial_description.voxels.positions_mm;
        let sensor_positions = &spatial_description.sensors.positions_mm;
        let sensor_offsets = &spatial_description.sensors.array_offsets_mm;
        let sensor_orientations = &spatial_description.sensors.orientations_xyz;

        let voxel_volume_m3 = (spatial_description.voxels.size_mm / 1000.0).powi(3);

        #[allow(clippy::cast_possible_truncation)]
        let common_factor = (VACUUM_MAG_PERMEABILITY as f32 * voxel_volume_m3) / (4.0 * PI) * 1e12;

        for beat in 0..spatial_description.sensors.count_beats() {
            for (index, v_type) in types.indexed_iter() {
                if !v_type.is_connectable() {
                    continue;
                }

                let v_num = voxel_numbers[index].unwrap();
                let v_pos_mm = voxel_positions_mm.slice(s![index.0, index.1, index.2, ..]);

                for s_num in 0..spatial_description.sensors.count() {
                    let s_pos_mm = &sensor_positions.slice(s![s_num, ..])
                        + &sensor_offsets.slice(s![beat, ..]);
                    let s_ori = sensor_orientations.slice(s![s_num, ..]);

                    let distace_m = (&s_pos_mm - &v_pos_mm) / 1000.0;
                    let distance_cubed_m3 = distace_m.mapv(|v| v.powi(2)).sum().sqrt().powi(3);

                    m[(beat, s_num, v_num)] = common_factor
                        * s_ori[2].mul_add(distace_m[1], -s_ori[1] * distace_m[2])
                        / distance_cubed_m3;
                    m[(beat, s_num, v_num + 1)] = common_factor
                        * s_ori[0].mul_add(distace_m[2], -s_ori[2] * distace_m[0])
                        / distance_cubed_m3;
                    m[(beat, s_num, v_num + 2)] = common_factor
                        * s_ori[1].mul_add(distace_m[0], -s_ori[0] * distace_m[1])
                        / distance_cubed_m3;
                }
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
        self.write_npy(writer).unwrap();
    }

    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn at_beat(&self, beat: usize) -> MeasurementMatrixAtBeat {
        MeasurementMatrixAtBeat(self.slice(s![beat, .., ..]))
    }

    pub(crate) fn update_from_gpu(&mut self, measurement_matrix: &Buffer<f32>) {
        measurement_matrix
            .read(self.as_slice_mut().unwrap())
            .enq()
            .unwrap();
    }
}

impl Deref for MeasurementMatrix {
    type Target = Array3<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MeasurementMatrix {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy)]
pub struct MeasurementMatrixAtBeat<'a>(ArrayView2<'a, f32>);

impl<'a> Deref for MeasurementMatrixAtBeat<'a> {
    type Target = ArrayView2<'a, f32>;
    #[tracing::instrument(level = "trace", skip_all)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions, clippy::unsafe_derive_deserialize)]
pub struct MeasurementCovariance(Array2<f32>);

impl MeasurementCovariance {
    /// Creates a new `MeasurementCovariance` with the given number of sensors,
    /// initialized to all zeros.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_sensors: usize) -> Self {
        debug!("Creating empty measurement covariance");
        Self(Array2::zeros((number_of_sensors, number_of_sensors)))
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

        measurement_covariance
    }

    /// Saves the measurement covariance matrix to a .npy file at the given path.
    /// Creates the directory if it does not exist.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving measurement covariance matrix to npy file");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("measurement_covariance.npy")).unwrap());
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

    pub(crate) fn update_from_gpu(&mut self, measurement_covariance: &Buffer<f32>) {
        measurement_covariance
            .read(self.as_slice_mut().unwrap())
            .enq()
            .unwrap();
    }
}

impl Deref for MeasurementCovariance {
    type Target = Array2<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MeasurementCovariance {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::{
        core::config::model::{Common, SensorArrayGeometry},
        vis::plotting::png::matrix::matrix_plot,
    };

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
            MeasurementMatrix::from_model_spatial_description(&spatial_description);

        assert!(!measurement_matrix.is_empty());
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
            MeasurementMatrix::from_model_spatial_description(&spatial_description);

        assert!(!measurement_matrix.is_empty());

        let path = Path::new(COMMON_PATH).join("measurement_matrix_default.png");
        matrix_plot(
            &measurement_matrix.slice(s![0, .., ..]),
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

    #[test]
    fn from_model_config_no_crash_and_plot_spase() {
        setup(None);
        let config = Model {
            common: Common {
                sensors_per_axis: [3, 3, 3],
                number_of_sensors: 10,
                sensor_array_geometry: SensorArrayGeometry::SparseCube,
                voxel_size_mm: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let spatial_description = SpatialDescription::from_model_config(&config);

        let measurement_matrix =
            MeasurementMatrix::from_model_spatial_description(&spatial_description);

        assert!(!measurement_matrix.is_empty());

        let path = Path::new(COMMON_PATH).join("measurement_matrix_default_sparse.png");
        matrix_plot(
            &measurement_matrix.slice(s![0, .., ..]),
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

    #[test]
    fn equality_sparse_full() {
        let config_full = Model {
            common: Common {
                sensors_per_axis: [10, 10, 10],
                sensor_array_geometry: SensorArrayGeometry::Cube,
                three_d_sensors: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let config_sparse = Model {
            common: Common {
                sensors_per_axis: [10, 10, 10],
                sensor_array_geometry: SensorArrayGeometry::SparseCube,
                three_d_sensors: true,
                number_of_sensors: 1000,
                ..Default::default()
            },
            ..Default::default()
        };

        let spatial_description_full = SpatialDescription::from_model_config(&config_full);
        let measurement_matrix_full =
            MeasurementMatrix::from_model_spatial_description(&spatial_description_full);

        let spatial_description_sparse = SpatialDescription::from_model_config(&config_sparse);
        let measurement_matrix_sparse =
            MeasurementMatrix::from_model_spatial_description(&spatial_description_sparse);

        assert_eq!(measurement_matrix_full, measurement_matrix_sparse);
    }
}
