use core::panic;

use ndarray::{arr1, s, Array2};

use crate::core::config::model::Model;

#[derive(Debug, PartialEq, Clone)]
pub struct Sensors {
    pub positions_mm: Array2<f32>,
    pub orientations_xyz: Array2<f32>,
}

impl Sensors {
    pub fn empty(number_of_sensors: usize) -> Sensors {
        Sensors {
            positions_mm: Array2::zeros((number_of_sensors, 3)),
            orientations_xyz: Array2::zeros((number_of_sensors, 3)),
        }
    }

    pub fn from_model_config(config: &Model) -> Sensors {
        let distance = [
            config.sensor_array_size_mm[0] / config.sensors_per_axis[0] as f32,
            config.sensor_array_size_mm[1] / config.sensors_per_axis[1] as f32,
            config.sensor_array_size_mm[2] / config.sensors_per_axis[2] as f32,
        ];
        let mut sensors = Sensors::empty(config.sensors_per_axis.iter().product());
        let mut i: usize = 0;
        for x in 0..config.sensors_per_axis[0] {
            for y in 0..config.sensors_per_axis[1] {
                for z in 0..config.sensors_per_axis[2] {
                    sensors.positions_mm.slice_mut(s![i, ..]).assign(&arr1(&[
                        x as f32 * distance[0] + config.sensor_array_origin_mm[0],
                        y as f32 * distance[1] + config.sensor_array_origin_mm[1],
                        z as f32 * distance[2] + config.sensor_array_origin_mm[2],
                    ]));
                    let orientation = match i % 3 {
                        0 => arr1(&[1.0, 0.0, 0.0]),
                        1 => arr1(&[0.0, 1.0, 0.0]),
                        2 => arr1(&[0.0, 0.0, 1.0]),
                        _ => panic!(),
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

    pub fn count(&self) -> usize {
        self.positions_mm.shape()[0]
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
        let mut config = Model::default();
        config.sensors_per_axis = [10, 20, 30];
        let sensors = Sensors::from_model_config(&config);

        assert_eq!(6000, sensors.count());
    }
}
