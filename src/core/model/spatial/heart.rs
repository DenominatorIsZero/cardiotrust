use ndarray::{arr1, Array1};

use crate::core::config::simulation::Simulation;

#[derive(Debug, PartialEq)]
pub struct Heart {
    pub origin_mm: Array1<f32>,
    pub size_mm: Array1<f32>,
}

impl Heart {
    pub fn empty() -> Heart {
        Heart {
            origin_mm: Array1::zeros(3),
            size_mm: Array1::zeros(3),
        }
    }

    pub fn from_simulation_config(config: &Simulation) -> Heart {
        Heart {
            origin_mm: arr1(&config.heart_origin_mm),
            size_mm: arr1(&config.heart_size_mm),
        }
    }
}
