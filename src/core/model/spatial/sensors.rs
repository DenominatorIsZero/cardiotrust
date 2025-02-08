use std::{
    fs::{self, File},
    io::BufWriter,
};

use ndarray::{arr1, s, Array1, Array2};
use ndarray_npy::WriteNpyExt;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use crate::core::config::model::{Common, SensorArrayGeometry, SensorArrayMotion};

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Sensors {
    pub array_center_mm: Array1<f32>,
    pub array_offsets_mm: Array2<f32>,
    pub array_radius_mm: f32,
    pub positions_mm: Array2<f32>,
    pub orientations_xyz: Array2<f32>,
}

impl Sensors {
    /// Creates a new `Sensors` instance with the given number of sensors, initializing
    /// all position and orientation values to 0.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn empty(number_of_sensors: usize, number_of_motion_steps: usize) -> Self {
        debug!("Creating empty sensors");
        Self {
            array_center_mm: Array1::zeros(3),
            array_offsets_mm: Array2::zeros((number_of_motion_steps, 3)),
            array_radius_mm: 100.0,
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
    #[allow(clippy::cast_precision_loss)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_model_config(config: &Common) -> Self {
        debug!("Creating sensors from model config");
        let number_of_motion_steps = match config.sensor_array_motion {
            SensorArrayMotion::Static => 1,
            SensorArrayMotion::Grid => config.sensor_array_motion_steps.iter().product(),
        };
        let mut sensors = match config.sensor_array_geometry {
            SensorArrayGeometry::Cube => {
                #[allow(clippy::cast_precision_loss)]
                let distance = [
                    config.sensor_array_size_mm[0] / config.sensors_per_axis[0] as f32,
                    config.sensor_array_size_mm[1] / config.sensors_per_axis[1] as f32,
                    config.sensor_array_size_mm[2] / config.sensors_per_axis[2] as f32,
                ];
                let dim = if config.three_d_sensors { 3 } else { 1 };
                let num_sensors = config.sensors_per_axis.iter().product::<usize>() * dim;
                let mut sensors = Self::empty(num_sensors, number_of_motion_steps);
                let mut i: usize = 0;
                for x in 0..config.sensors_per_axis[0] {
                    for y in 0..config.sensors_per_axis[1] {
                        for z in 0..config.sensors_per_axis[2] {
                            for _ in 0..dim {
                                #[allow(clippy::cast_precision_loss)]
                                sensors.positions_mm.slice_mut(s![i, ..]).assign(&arr1(&[
                                    (x as f32)
                                        .mul_add(distance[0], config.sensor_array_origin_mm[0]),
                                    (y as f32)
                                        .mul_add(distance[1], config.sensor_array_origin_mm[1]),
                                    (z as f32)
                                        .mul_add(distance[2], config.sensor_array_origin_mm[2]),
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
                }
                sensors
            }
            SensorArrayGeometry::SparseCube => {
                #[allow(clippy::cast_precision_loss)]
                let distance = [
                    config.sensor_array_size_mm[0] / config.sensors_per_axis[0] as f32,
                    config.sensor_array_size_mm[1] / config.sensors_per_axis[1] as f32,
                    config.sensor_array_size_mm[2] / config.sensors_per_axis[2] as f32,
                ];
                let dim = if config.three_d_sensors { 3 } else { 1 };
                let num_sensors = config.number_of_sensors * dim;
                let num_occupied = config.number_of_sensors;
                let num_places = config.sensors_per_axis.iter().product::<usize>();
                assert!(num_occupied <= num_places);
                let mut sensors = Self::empty(num_sensors, number_of_motion_steps);

                // Generate all possible positions
                let mut positions = Vec::with_capacity(num_places);
                for x in 0..config.sensors_per_axis[0] {
                    for y in 0..config.sensors_per_axis[1] {
                        for z in 0..config.sensors_per_axis[2] {
                            positions.push([x, y, z]);
                        }
                    }
                }

                // Randomly select positions
                let mut rng = rand::thread_rng();
                positions.shuffle(&mut rng);
                let mut selected_positions = positions[0..num_occupied].to_vec();

                // Sort positions based on x, then y, then z
                selected_positions
                    .sort_by(|a, b| a[0].cmp(&b[0]).then(a[1].cmp(&b[1])).then(a[2].cmp(&b[2])));

                // Assign positions and orientations to sensors
                let mut i = 0;
                for selected_position in &selected_positions {
                    let [x, y, z] = selected_position;
                    for _ in 0..dim {
                        #[allow(clippy::cast_precision_loss)]
                        sensors.positions_mm.slice_mut(s![i, ..]).assign(&arr1(&[
                            (*x as f32).mul_add(distance[0], config.sensor_array_origin_mm[0]),
                            (*y as f32).mul_add(distance[1], config.sensor_array_origin_mm[1]),
                            (*z as f32).mul_add(distance[2], config.sensor_array_origin_mm[2]),
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
                sensors
            }
            SensorArrayGeometry::Cylinder => {
                let dim = if config.three_d_sensors { 3 } else { 1 };
                let num = config.number_of_sensors * dim;
                let mut sensors = Self::empty(num, number_of_motion_steps);
                let radius = config.sensor_array_radius_mm;
                let origin = &config.sensor_array_origin_mm;
                for i in 0..config.number_of_sensors {
                    let theta =
                        (i as f32 / config.number_of_sensors as f32) * 2.0 * std::f32::consts::PI;
                    let position = arr1(&[
                        theta.sin().mul_add(radius, origin[0]),
                        0.0 + origin[1],
                        theta.cos().mul_add(radius, origin[2]),
                    ]);
                    for d in 0..dim {
                        let orientation = match d {
                            0 => arr1(&[theta.sin(), 0.0, theta.cos()]),
                            1 => arr1(&[theta.cos(), 0.0, -theta.sin()]),
                            2 => arr1(&[0.0, 1.0, 0.0]),
                            _ => arr1(&[0.0, 0.0, 0.0]),
                        };
                        sensors
                            .orientations_xyz
                            .slice_mut(s![dim * i + d, ..])
                            .assign(&orientation);
                        sensors
                            .positions_mm
                            .slice_mut(s![dim * i + d, ..])
                            .assign(&position);
                    }
                }
                sensors.array_center_mm = arr1(&config.sensor_array_origin_mm);
                sensors.array_radius_mm = config.sensor_array_radius_mm;
                sensors
            }
        };
        if config.sensor_array_motion == SensorArrayMotion::Grid {
            let step_size_mm_x = if config.sensor_array_motion_steps[0] > 1 {
                config.sensor_array_motion_range_mm[0]
                    / (config.sensor_array_motion_steps[0] - 1) as f32
            } else {
                0.0
            };
            let step_size_mm_y = if config.sensor_array_motion_steps[1] > 1 {
                config.sensor_array_motion_range_mm[1]
                    / (config.sensor_array_motion_steps[1] - 1) as f32
            } else {
                0.0
            };
            let step_size_mm_z = if config.sensor_array_motion_steps[2] > 1 {
                config.sensor_array_motion_range_mm[2]
                    / (config.sensor_array_motion_steps[2] - 1) as f32
            } else {
                0.0
            };

            let mut motion_index = 0;
            for x in 0..config.sensor_array_motion_steps[0] {
                for y in 0..config.sensor_array_motion_steps[1] {
                    for z in 0..config.sensor_array_motion_steps[2] {
                        sensors.array_offsets_mm[(motion_index, 0)] = step_size_mm_x * x as f32;
                        sensors.array_offsets_mm[(motion_index, 1)] = step_size_mm_y * y as f32;
                        sensors.array_offsets_mm[(motion_index, 2)] = step_size_mm_z * z as f32;
                        motion_index += 1;
                    }
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

    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn count_beats(&self) -> usize {
        trace!("Retrieving number of beats");
        self.array_offsets_mm.shape()[0]
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
        let sensor_motion_steps = 10;
        let sensors = Sensors::empty(number_of_sensors, sensor_motion_steps);

        assert_eq!(number_of_sensors, sensors.count());
    }

    #[test]
    fn count_from_simulation() {
        let config = Common {
            sensors_per_axis: [10, 20, 30],
            sensor_array_geometry: SensorArrayGeometry::Cube,
            three_d_sensors: false,
            ..Default::default()
        };
        let sensors = Sensors::from_model_config(&config);

        assert_eq!(6000, sensors.count());
    }

    #[test]
    fn equality_sparse_full() {
        let config_full = Common {
            sensors_per_axis: [10, 10, 10],
            sensor_array_geometry: SensorArrayGeometry::Cube,
            three_d_sensors: true,
            ..Default::default()
        };
        let config_sparse = Common {
            sensors_per_axis: [10, 10, 10],
            sensor_array_geometry: SensorArrayGeometry::SparseCube,
            three_d_sensors: true,
            number_of_sensors: 1000,
            ..Default::default()
        };
        let sensors = Sensors::from_model_config(&config_full);
        let sensors_2 = Sensors::from_model_config(&config_sparse);

        assert_eq!(sensors, sensors_2);
    }
}
