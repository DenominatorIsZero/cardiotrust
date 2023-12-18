use serde::{Deserialize, Serialize};

use crate::core::{
    algorithm::{estimation::Estimations, refinement::derivation::Derivatives},
    model::{functional::FunctionalDescription, Model},
};

use super::algorithm::metrics::Metrics;
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Results {
    pub metrics: Metrics,
    pub estimations: Estimations,
    pub derivatives: Derivatives,
    pub snapshots: Vec<Snapshot>,
    pub model: Option<Model>,
}

#[allow(clippy::useless_let_if_seq)]
impl Results {
    #[must_use]
    pub fn new(
        number_of_epochs: usize,
        number_of_steps: usize,
        number_of_sensors: usize,
        number_of_states: usize,
    ) -> Self {
        let estimations = Estimations::empty(number_of_states, number_of_sensors, number_of_steps);
        let derivatives = Derivatives::new(number_of_states);
        let snapshots = Vec::new();

        Self {
            metrics: Metrics::new(number_of_epochs, number_of_steps),
            estimations,
            derivatives,
            model: None,
            snapshots,
        }
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        self.metrics.save_npy(&path.join("metrics"));
        self.estimations.save_npy(&path.join("estimations"));
        self.model.as_ref().unwrap().save_npy(&path.join("model"));
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
