use serde::{Deserialize, Serialize};

use crate::core::model::Model;

use super::algorithm::{
    estimation::Estimations, metrics::Metrics, refinement::derivation::Derivatives,
};
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Results {
    pub metrics: Metrics,
    pub estimations: Estimations,
    pub derivatives: Derivatives,
    pub model: Option<Model>,
    pub snapshots: Vec<Snapshot>,
}

impl Results {
    #[must_use]
    pub fn new(
        number_of_epochs: usize,
        number_of_steps: usize,
        number_of_sensors: usize,
        number_of_states: usize,
    ) -> Self {
        Self {
            metrics: Metrics::new(number_of_epochs, number_of_steps),
            estimations: Estimations::new(number_of_states, number_of_sensors, number_of_steps),
            derivatives: Derivatives::new(number_of_states),
            model: None,
            snapshots: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Snapshot {}
