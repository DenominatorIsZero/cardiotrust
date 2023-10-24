use serde::{Deserialize, Serialize};

use crate::core::model::{functional::FunctionalDescription, Model};

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
            estimations: Estimations::empty(number_of_states, number_of_sensors, number_of_steps),
            derivatives: Derivatives::new(number_of_states),
            model: None,
            snapshots: Vec::new(),
        }
    }

    pub(crate) fn save_npy(&self, path: std::path::PathBuf) {
        self.metrics.save_npy(path.join("metrics"));
        self.estimations.save_npy(path.join("estimations"));
        self.model.as_ref().unwrap().save_npy(path.join("model"));
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub estimations: Estimations,
    pub functional_description: FunctionalDescription,
}

impl Snapshot {
    #[must_use]
    pub fn new(estimations: &Estimations, functional_description: &FunctionalDescription) -> Self {
        Self {
            estimations: estimations.clone(),
            functional_description: functional_description.clone(),
        }
    }
}
