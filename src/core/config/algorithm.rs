use serde::{Deserialize, Serialize};
use tracing::debug;

use super::model::Model;
use crate::core::algorithm::refinement::Optimizer;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[allow(clippy::module_name_repetitions)]
pub enum AlgorithmType {
    #[default]
    ModelBased,
    PseudoInverse,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub enum APDerivative {
    Simple,
    #[default]
    Textbook,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Algorithm {
    pub model: Model,
    #[serde(default)]
    pub algorithm_type: AlgorithmType,
    #[serde(default)]
    pub optimizer: Optimizer,
    pub epochs: usize,
    #[serde(default)]
    pub batch_size: usize,
    pub snapshots_interval: usize,
    pub learning_rate: f32,
    #[serde(default)]
    pub learning_rate_reduction_factor: f32,
    #[serde(default)]
    pub learning_rate_reduction_interval: usize,
    #[serde(default)]
    pub mse_strength: f32,
    #[serde(default)]
    // used for SGD optimization of ap coefficients to ensure convergence.
    pub slow_down_stregth: f32,
    #[serde(default)]
    pub maximum_regularization_strength: f32,
    #[serde(default)]
    pub maximum_regularization_threshold: f32,
    #[serde(default)]
    pub difference_regularization_strength: f32,
    #[serde(default)]
    pub smoothness_regularization_strength: f32,
    #[serde(default)]
    pub freeze_gains: bool,
    pub freeze_delays: bool,
    pub update_kalman_gain: bool,
    #[serde(default)]
    pub ap_derivative: APDerivative,
}
impl Default for Algorithm {
    /// Returns a default `Algorithm` configuration with reasonable defaults for most use cases.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Creating default algorithm");
        Self {
            algorithm_type: AlgorithmType::default(),
            optimizer: Optimizer::default(),
            epochs: 10,
            batch_size: 0,
            snapshots_interval: 0,
            learning_rate: 200.0,
            learning_rate_reduction_factor: 0.0,
            learning_rate_reduction_interval: 0,
            mse_strength: 1.0,
            slow_down_stregth: 0.,
            maximum_regularization_strength: 1.0,
            maximum_regularization_threshold: 1.01,
            difference_regularization_strength: 0.0,
            smoothness_regularization_strength: 0.0,
            model: Model::default(),
            freeze_gains: false,
            freeze_delays: true,
            update_kalman_gain: false,
            ap_derivative: APDerivative::default(),
        }
    }
}
