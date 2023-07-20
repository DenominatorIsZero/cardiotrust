use ndarray::{arr1, Array1};

use crate::core::config::model::Model;

#[derive(Debug, PartialEq, Clone)]
pub struct Heart {
    pub origin_mm: Array1<f32>,
    pub size_mm: Array1<f32>,
}

impl Heart {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            origin_mm: Array1::zeros(3),
            size_mm: Array1::zeros(3),
        }
    }

    #[must_use]
    pub fn from_model_config(config: &Model) -> Self {
        Self {
            origin_mm: arr1(&config.heart_origin_mm),
            size_mm: arr1(&config.heart_size_mm),
        }
    }
}
