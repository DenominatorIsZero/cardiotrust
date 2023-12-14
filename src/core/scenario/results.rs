use serde::{Deserialize, Serialize};

use crate::core::{
    algorithm::{estimation::EstimationsFlat, refinement::derivation::DerivativesFlat},
    model::{functional::FunctionalDescription, Model},
};

use super::algorithm::{
    estimation::EstimationsNormal, metrics::Metrics, refinement::derivation::DerivativesNormal,
};
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Results {
    pub metrics: Metrics,
    pub estimations_normal: Option<EstimationsNormal>,
    pub derivatives_normal: Option<DerivativesNormal>,
    pub snapshots_normal: Option<Vec<SnapshotNormal>>,
    pub estimations_flat: Option<EstimationsFlat>,
    pub derivatives_flat: Option<DerivativesFlat>,
    pub snapshots_flat: Option<Vec<SnapshotFlat>>,
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
        use_flat_arrays: bool,
    ) -> Self {
        let mut estimations_normal = None;
        let mut derivatives_normal = None;
        let mut snapshots_normal = None;
        let mut estimations_flat = None;
        let mut derivatives_flat = None;
        let mut snapshots_flat = None;

        if use_flat_arrays {
            estimations_flat = Some(EstimationsFlat::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
            ));
            derivatives_flat = Some(DerivativesFlat::new(number_of_states));
            snapshots_flat = Some(Vec::new());
        } else {
            estimations_normal = Some(EstimationsNormal::empty(
                number_of_states,
                number_of_sensors,
                number_of_steps,
            ));
            derivatives_normal = Some(DerivativesNormal::new(number_of_states));
            snapshots_normal = Some(Vec::new());
        }

        Self {
            metrics: Metrics::new(number_of_epochs, number_of_steps),
            estimations_normal,
            derivatives_normal,
            estimations_flat,
            derivatives_flat,
            model: None,
            snapshots_normal,
            snapshots_flat,
        }
    }

    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        self.metrics.save_npy(&path.join("metrics"));
        if self.estimations_normal.is_some() {
            self.estimations_normal
                .as_ref()
                .expect("Estimations normal to be some.")
                .save_npy(&path.join("estimations_normal"));
        } else {
            self.estimations_flat
                .as_ref()
                .expect("Estimations flat to be some.")
                .save_npy(&path.join("estimations_flat"));
        }
        self.model.as_ref().unwrap().save_npy(&path.join("model"));
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SnapshotNormal {
    pub estimations: EstimationsNormal,
    pub functional_description: FunctionalDescription,
}

impl SnapshotNormal {
    #[must_use]
    pub fn new(
        estimations: &EstimationsNormal,
        functional_description: &FunctionalDescription,
    ) -> Self {
        Self {
            estimations: estimations.clone(),
            functional_description: functional_description.clone(),
        }
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
