use ndarray::{arr1, s, Array2};

use crate::core::config::simulation::Simulation;

#[derive(Debug, PartialEq)]
pub struct Sensors {
    positions: Array2<f32>,
    orientations: Array2<f32>,
}

impl Sensors {
    pub fn empty(number_of_sensors: usize) -> Sensors {
        Sensors {
            positions: Array2::zeros((number_of_sensors, 3)),
            orientations: Array2::zeros((number_of_sensors, 3)),
        }
    }

    pub fn from_simulation_config(config: &Simulation) -> Sensors {
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
                    sensors.positions.slice_mut(s![i, ..]).assign(&arr1(&[
                        x as f32 * distance[0],
                        y as f32 * distance[1],
                        z as f32 * distance[2],
                    ]));
                    let orientation = match i % 3 {
                        0 => arr1(&[1.0, 0.0, 0.0]),
                        1 => arr1(&[0.0, 1.0, 0.0]),
                        2 => arr1(&[0.0, 0.0, 1.0]),
                    };
                    sensors
                        .orientations
                        .slice_mut(s![i, ..])
                        .assign(&orientation);
                    i += 1;
                }
            }
        }
        sensors
    }
}
