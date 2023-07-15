use super::algorithm::{
    estimation::Estimations, metrics::Metrics, refinement::derivation::Derivatives,
};
#[derive(Debug, PartialEq, Clone)]
pub struct Results {
    pub metrics: Metrics,
    pub estimations: Estimations,
    pub derivatives: Derivatives,
    pub snapshots: Vec<Snapshot>,
}

impl Results {
    pub fn new(
        number_of_epochs: usize,
        number_of_steps: usize,
        number_of_sensors: usize,
        number_of_states: usize,
    ) -> Results {
        Results {
            metrics: Metrics::new(number_of_epochs, number_of_steps),
            estimations: Estimations::new(number_of_states, number_of_sensors, number_of_steps),
            derivatives: Derivatives::new(number_of_states),
            snapshots: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Snapshot {}
