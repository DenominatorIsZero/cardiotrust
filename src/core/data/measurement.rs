use serde::{Deserialize, Serialize};

use crate::core::data::Measurements;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub measurements: Measurements,
}
