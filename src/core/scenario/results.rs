use serde::{Deserialize, Serialize};

use crate::core::{
    algorithm::{estimation::EstimationsFlat, refinement::derivation::DerivativesFlat},
    model::{functional::FunctionalDescription, Model},
};

use super::algorithm::metrics::Metrics;
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Results {
    pub metrics: Metrics,
    pub estimations_flat: EstimationsFlat,
    pub derivatives_flat: DerivativesFlat,
    pub snapshots_flat: Vec<SnapshotFlat>,
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
        let estimations_flat =
            EstimationsFlat::empty(number_of_states, number_of_sensors, number_of_steps);
        let derivatives_flat = DerivativesFlat::new(number_of_states);
        let snapshots_flat = Vec::new();

        Self {
            metrics: Metrics::new(number_of_epochs, number_of_steps),
            estimations_flat,
            derivatives_flat,
            model: None,
            snapshots_flat,
        }
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        self.metrics.save_npy(&path.join("metrics"));
        self.estimations_flat
            .save_npy(&path.join("estimations_flat"));
        self.model.as_ref().unwrap().save_npy(&path.join("model"));
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SnapshotFlat {
    pub estimations: EstimationsFlat,
    pub functional_description: FunctionalDescription,
}

impl SnapshotFlat {
    #[must_use]
    pub fn new(
        estimations: &EstimationsFlat,
        functional_description: &FunctionalDescription,
    ) -> Self {
        Self {
            estimations: estimations.clone(),
            functional_description: functional_description.clone(),
        }
    }
}
