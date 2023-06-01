pub mod measurement;
pub mod shapes;
pub mod simulation;

use self::measurement::Measurement;
use self::simulation::Simulation;

use crate::core::data::shapes::ArrayMeasurements;

#[derive(Debug, PartialEq)]
pub struct Data {
    simulation: Option<Simulation>,
    measurement: Option<Measurement>,
}
impl Data {
    pub fn get_measurements(&self) -> &ArrayMeasurements {
        if let Some(simulation) = self.simulation.as_ref() {
            &(simulation.measurements)
        } else {
            &(self.measurement.as_ref().unwrap().measurements)
        }
    }

    pub fn new(number_of_sensors: usize, number_of_steps: usize) -> Data {
        Data {
            simulation: Some(Simulation::empty(number_of_sensors, number_of_steps)),
            measurement: None,
        }
    }
}
