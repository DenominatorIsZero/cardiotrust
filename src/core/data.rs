use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    simulation: Option<Simulation>,
    measurement: Option<Measurement>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Simulation;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Measurement;
