use ndarray::{arr1, s, Array2};
use ndarray_npy::WriteNpyExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};
use tracing::{debug, trace};

use crate::core::config::model::Model;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Sensors {
    pub positions_mm: Array2<f32>,
    pub orientations_xyz: Array2<f32>,
}

impl Sensors {
    /// Creates a new `Sensors` instance with the given number of sensors, initializing
    /// all position and orientation values to 0.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_sensors: usize) -> Self {
        debug!("Creating empty sensors");
        Self {
            positions_mm: Array2::zeros((number_of_sensors, 3)),
            orientations_xyz: Array2::zeros((number_of_sensors, 3)),
        }
    }

    /// Creates a new `Sensors` instance initialized with sensor positions and
    /// orientations based on the provided `Model` config.
    ///
    /// The sensor positions are spaced evenly throughout the configured sensor
    /// array volume, starting from the configured `sensor_array_origin_mm`.
    ///
    /// The sensor orientations alternate between x, y, and z axes aligned.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn from_model_config(config: &Model) -> Self {
        debug!("Creating sensors from model config");
        #[allow(clippy::cast_precision_loss)]
        let distance = [
            config.sensor_array_size_mm[0] / config.sensors_per_axis[0] as f32,
            config.sensor_array_size_mm[1] / config.sensors_per_axis[1] as f32,
            config.sensor_array_size_mm[2] / config.sensors_per_axis[2] as f32,
        ];
        let mut sensors = Self::empty(config.sensors_per_axis.iter().product());
        let mut i: usize = 0;
        for x in 0..config.sensors_per_axis[0] {
            for y in 0..config.sensors_per_axis[1] {
                for z in 0..config.sensors_per_axis[2] {
                    #[allow(clippy::cast_precision_loss)]
                    sensors.positions_mm.slice_mut(s![i, ..]).assign(&arr1(&[
                        (x as f32).mul_add(distance[0], config.sensor_array_origin_mm[0]),
                        (y as f32).mul_add(distance[1], config.sensor_array_origin_mm[1]),
                        (z as f32).mul_add(distance[2], config.sensor_array_origin_mm[2]),
                    ]));
                    let orientation = match i % 3 {
                        0 => arr1(&[1.0, 0.0, 0.0]),
                        1 => arr1(&[0.0, 1.0, 0.0]),
                        2 => arr1(&[0.0, 0.0, 1.0]),
                        _ => arr1(&[0.0, 0.0, 0.0]),
                    };
                    sensors
                        .orientations_xyz
                        .slice_mut(s![i, ..])
                        .assign(&orientation);
                    i += 1;
                }
            }
        }
        sensors
    }

    /// Returns the number of sensors.
    ///
    /// This is determined by the size of the first dimension of the
    /// `positions_mm` array.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn count(&self) -> usize {
        trace!("Retrieving number of sensors");
        self.positions_mm.shape()[0]
    }

    /// Saves the sensor positions and orientations to .npy files in the given path.
    /// Creates the directory if it does not exist.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving sensors to npy files");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join("sensor_positions_mm.npy")).unwrap());
        self.positions_mm.write_npy(writer).unwrap();
        let writer =
            BufWriter::new(File::create(path.join("sensor_orientations_xyz.npy")).unwrap());
        self.orientations_xyz.write_npy(writer).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_empty() {
        let number_of_sensors = 300;
        let sensors = Sensors::empty(number_of_sensors);

        assert_eq!(number_of_sensors, sensors.count());
    }

    #[test]
    fn count_from_simulation() {
        let config = Model {
            sensors_per_axis: [10, 20, 30],
            ..Default::default()
        };
        let sensors = Sensors::from_model_config(&config);

        assert_eq!(6000, sensors.count());
    }
}
