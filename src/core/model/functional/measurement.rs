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
    #[must_use]
    pub fn empty(number_of_states: usize, number_of_sensors: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_sensors, number_of_states)),
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if voxel numbers are not initialized correctly.
    #[must_use]
    pub fn from_model_config(_config: &Model, spatial_description: &SpatialDescription) -> Self {
        let mut measurement_matrix = Self::empty(
            spatial_description.voxels.count() * 3,
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
            if *v_type == VoxelType::None {
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

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path.clone()).unwrap();
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
    #[must_use]
    pub fn empty(number_of_sensors: usize) -> Self {
        Self {
            values: Array2::zeros((number_of_sensors, number_of_sensors)),
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if voxel numbers are not initialized correctly.
    #[must_use]
    pub fn from_model_config(config: &Model, spatial_description: &SpatialDescription) -> Self {
        let mut measurement_covariance = Self::empty(spatial_description.sensors.count());

        if relative_eq!(config.measurement_covariance_std, 0.0) {
            measurement_covariance
                .values
                .diag_mut()
                .fill(config.measurement_covariance_mean);
        } else {
            let normal = Normal::<f32>::new(
                config.measurement_covariance_mean,
                config.measurement_covariance_std,
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

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path.clone()).unwrap();
        let writer = BufWriter::new(File::create(path.join("process_covariance.npy")).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::vis::plotting::matrix::plot_matrix_as_heatmap;

    use super::*;

    #[test]
    fn from_model_config_no_crash() {
        let config = Model {
            sensors_per_axis: [3, 3, 3],
            voxel_size_mm: 20.0,
            ..Default::default()
        };
        let spatial_description = SpatialDescription::from_model_config(&config);

        let measurement_matrix =
            MeasurementMatrix::from_model_config(&config, &spatial_description);

        assert!(!measurement_matrix.values.is_empty());
    }

    #[test]
    #[ignore]
    fn from_model_config_no_crash_and_plot() {
        let config = Model {
            sensors_per_axis: [3, 3, 3],
            voxel_size_mm: 20.0,
            ..Default::default()
        };
        let spatial_description = SpatialDescription::from_model_config(&config);

        let measurement_matrix =
            MeasurementMatrix::from_model_config(&config, &spatial_description);

        assert!(!measurement_matrix.values.is_empty());

        plot_matrix_as_heatmap(
            &measurement_matrix.values,
            "tests/measurement_matrix_default",
            "Measurement Matrix",
        );
    }
}
