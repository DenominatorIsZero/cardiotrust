use serde::{Deserialize, Serialize};

use super::algorithm::metrics::Metrics;
use crate::core::{
    algorithm::{estimation::Estimations, refinement::derivation::Derivatives},
    model::{functional::FunctionalDescription, Model},
};

/// Results contains the outputs from running a scenario.
///
/// This includes metrics, estimations, derivatives, snapshots,
/// the model, etc. It is returned after running the scenario.
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
    /// Creates a new Results instance with empty estimations, derivatives,
    /// snapshots, and model. The metrics are initialized based on the provided
    /// number of epochs, steps, sensors, and states.
    #[must_use]
    #[tracing::instrument]
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

    /// Saves the metrics, estimations, and model as .npy files to the given path.
    #[tracing::instrument]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        self.metrics.save_npy(&path.join("metrics"));
        self.estimations.save_npy(&path.join("estimations"));
        self.model.as_ref().unwrap().save_npy(&path.join("model"));
    }
}

/// Snapshot contains estimations and functional description at a point in time.
/// Used to capture model state during scenario execution.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub estimations: Estimations,
    pub functional_description: FunctionalDescription,
}

impl Snapshot {
    #[must_use]
    /// Creates a new Snapshot instance with the provided estimations and
    /// functional description.
    #[tracing::instrument]
    pub fn new(estimations: &Estimations, functional_description: &FunctionalDescription) -> Self {
        Self {
            estimations: estimations.clone(),
            functional_description: functional_description.clone(),
        }
    }
}
