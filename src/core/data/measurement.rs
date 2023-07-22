use serde::{Deserialize, Serialize};

use crate::core::data::ArrayMeasurements;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub measurements: ArrayMeasurements,
}
