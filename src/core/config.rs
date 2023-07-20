pub mod algorithm;
pub mod model;
pub mod simulation;

use serde::{Deserialize, Serialize};
use std::path::Path;

use self::{algorithm::Algorithm, simulation::Simulation};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub measurement: Option<Box<Path>>,
    pub simulation: Option<Simulation>,
    pub algorithm: Algorithm,
}
impl Default for Config {
    #[must_use]
    fn default() -> Self {
        Self {
            measurement: None,
            simulation: Some(Simulation::default()),
            algorithm: Algorithm::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelPreset {
    Healthy,
    Pathological,
}
