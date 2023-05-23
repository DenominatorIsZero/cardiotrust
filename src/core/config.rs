mod algorithm;
pub mod simulation;

use serde::{Deserialize, Serialize};
use std::path::Path;

use self::{algorithm::Algorithm, simulation::Simulation};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub measurement: Option<Box<Path>>,
    pub simulation: Option<Simulation>,
    pub algorithm: Algorithm,
}
impl Config {
    pub fn default() -> Config {
        Config {
            measurement: None,
            simulation: Some(Simulation::default()),
            algorithm: Algorithm::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ModelPreset {
    Healthy,
    Scarred,
}
