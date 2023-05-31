use super::algorithm::estimation::shapes::ArrayMeasurements;
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
            simulation: Some(Simulation::new(number_of_sensors, number_of_steps)),
            measurement: None,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Simulation {
    pub measurements: ArrayMeasurements,
}
impl Simulation {
    fn new(number_of_sensors: usize, number_of_steps: usize) -> Simulation {
        Simulation {
            measurements: ArrayMeasurements::new(number_of_steps, number_of_sensors),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Measurement {
    pub measurements: ArrayMeasurements,
}
