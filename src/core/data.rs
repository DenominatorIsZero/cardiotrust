pub mod measurement;
pub mod shapes;
pub mod simulation;

use ndarray::Dim;

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

    pub fn empty(
        number_of_sensors: usize,
        number_of_states: usize,
        number_of_steps: usize,
        voxels_in_dims: Dim<[usize; 3]>,
    ) -> Data {
        Data {
            simulation: Some(Simulation::empty(
                number_of_sensors,
                number_of_states,
                number_of_steps,
                voxels_in_dims,
            )),
            measurement: None,
        }
    }
}
