use crate::core::data::ArrayMeasurements;

#[derive(Debug, PartialEq)]
pub struct Simulation {
    pub measurements: ArrayMeasurements,
}
impl Simulation {
    pub fn new(number_of_sensors: usize, number_of_steps: usize) -> Simulation {
        Simulation {
            measurements: ArrayMeasurements::new(number_of_steps, number_of_sensors),
        }
    }
}
