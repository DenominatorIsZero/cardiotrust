use ndarray::{s, Array1};
use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};

use crate::core::model::spatial::voxels::{VoxelNumbers, VoxelType, VoxelTypes};

use super::{
    estimation::{Estimations},
    refinement::derivation::Derivatives,
};

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub loss: ArrayMetricsSample,
    pub loss_epoch: ArrayMetricsEpoch,

    pub loss_mse: ArrayMetricsSample,
    pub loss_mse_epoch: ArrayMetricsEpoch,
    pub loss_maximum_regularization: ArrayMetricsSample,
    pub loss_maximum_regularization_epoch: ArrayMetricsEpoch,

    pub delta_states_mean: ArrayMetricsSample,
    pub delta_states_mean_epoch: ArrayMetricsEpoch,
    pub delta_states_max: ArrayMetricsSample,
    pub delta_states_max_epoch: ArrayMetricsEpoch,

    pub delta_measurements_mean: ArrayMetricsSample,
    pub delta_measurements_mean_epoch: ArrayMetricsEpoch,
    pub delta_measurements_max: ArrayMetricsSample,
    pub delta_measurements_max_epoch: ArrayMetricsEpoch,

    pub delta_gains_mean: ArrayMetricsSample,
    pub delta_gains_mean_epoch: ArrayMetricsEpoch,
    pub delta_gains_max: ArrayMetricsSample,
    pub delta_gains_max_epoch: ArrayMetricsEpoch,

    pub delta_delays_mean: ArrayMetricsSample,
    pub delta_delays_mean_epoch: ArrayMetricsEpoch,
    pub delta_delays_max: ArrayMetricsSample,
    pub delta_delays_max_epoch: ArrayMetricsEpoch,

    #[serde(default)]
    pub dice_score_over_threshold: Array1<f32>,
    #[serde(default)]
    pub iou_over_threshold: Array1<f32>,
    #[serde(default)]
    pub precision_over_threshold: Array1<f32>,
    #[serde(default)]
    pub recall_over_threshold: Array1<f32>,
}

impl Metrics {
    #[must_use]
    pub fn new(number_of_epochs: usize, number_of_steps: usize) -> Self {
        Self {
            loss: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            loss_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            loss_mse: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            loss_mse_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            loss_maximum_regularization: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            loss_maximum_regularization_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_states_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_states_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_states_max: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_states_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_measurements_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_measurements_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_measurements_max: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_measurements_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_gains_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_gains_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_gains_max: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_gains_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_delays_mean: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_delays_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_delays_max: ArrayMetricsSample::new(number_of_epochs, number_of_steps),
            delta_delays_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            dice_score_over_threshold: Array1::zeros(101),
            iou_over_threshold: Array1::zeros(101),
            precision_over_threshold: Array1::zeros(101),
            recall_over_threshold: Array1::zeros(101),
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if any array is None.
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_step(
        &mut self,
        estimations: &Estimations,
        derivatives: &Derivatives,
        regularization_strength: f32,
        time_index: usize,
        epoch_index: usize,
    ) {
        let index = time_index
            + epoch_index * (self.loss.values.shape()[0] / self.loss_epoch.values.shape()[0]);

        self.loss_mse.values[index] = estimations.residuals.values.mapv(|v| v.powi(2)).sum()
            / estimations.residuals.values.raw_dim()[0] as f32;
        self.loss_maximum_regularization.values[index] = derivatives
            .maximum_regularization
            .values
            .mapv(f32::abs)
            .sum();
        self.loss.values[index] = (1.0 - regularization_strength).mul_add(
            self.loss_mse.values[index],
            regularization_strength * self.loss_maximum_regularization.values[index],
        );

        let states_delta_abs = estimations.system_states_delta.values.mapv(f32::abs);
        self.delta_states_mean.values[index] = states_delta_abs.mean().unwrap();
        self.delta_states_max.values[index] = *states_delta_abs.max_skipnan();

        let measurements_delta_abs = estimations.post_update_residuals.values.mapv(f32::abs);
        self.delta_measurements_mean.values[index] = measurements_delta_abs.mean().unwrap();
        self.delta_measurements_max.values[index] = *measurements_delta_abs.max_skipnan();

        let gains_delta_abs = estimations.gains_delta.values.mapv(f32::abs);
        self.delta_gains_mean.values[index] = gains_delta_abs.mean().unwrap();
        self.delta_gains_max.values[index] = *gains_delta_abs.max_skipnan();

        let delays_delta_abs = estimations.delays_delta.values.mapv(f32::abs);
        self.delta_delays_mean.values[index] = delays_delta_abs.mean().unwrap();
        self.delta_delays_max.values[index] = *delays_delta_abs.max_skipnan();
    }

    /// .
    ///
    /// # Panics
    ///
    /// Panics if any loss array is None.
    pub fn calculate_epoch(&mut self, epoch_index: usize) {
        let number_of_steps = self.loss.values.shape()[0] / self.loss_epoch.values.shape()[0];
        let start_index = epoch_index * number_of_steps;
        let stop_index = (epoch_index + 1) * number_of_steps;
        let slice = s![start_index..stop_index];

        self.loss_mse_epoch.values[epoch_index] = self.loss_mse.values.slice(slice).mean().unwrap();
        self.loss_maximum_regularization_epoch.values[epoch_index] = self
            .loss_maximum_regularization
            .values
            .slice(slice)
            .mean()
            .unwrap();
        self.loss_epoch.values[epoch_index] = self.loss.values.slice(slice).mean().unwrap();

        self.delta_states_mean_epoch.values[epoch_index] =
            self.delta_states_mean.values.slice(slice).mean().unwrap();
        self.delta_states_max_epoch.values[epoch_index] =
            *self.delta_states_max.values.slice(slice).max_skipnan();

        self.delta_measurements_mean_epoch.values[epoch_index] = self
            .delta_measurements_mean
            .values
            .slice(slice)
            .mean()
            .unwrap();
        self.delta_measurements_max_epoch.values[epoch_index] = *self
            .delta_measurements_max
            .values
            .slice(slice)
            .max_skipnan();

        self.delta_gains_mean_epoch.values[epoch_index] =
            self.delta_gains_mean.values[stop_index - 1];
        self.delta_gains_max_epoch.values[epoch_index] =
            self.delta_gains_max.values[stop_index - 1];

        self.delta_delays_mean_epoch.values[epoch_index] =
            self.delta_delays_mean.values[stop_index - 1];
        self.delta_delays_max_epoch.values[epoch_index] =
            self.delta_delays_max.values[stop_index - 1];
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_final(
        &mut self,
        estimations: &Estimations,
        ground_truth: &VoxelTypes,
        voxel_numbers: &VoxelNumbers,
    ) {
        for i in 0..=100 {
            let threshold = i as f32 / 100.0;
            let (dice, iou, precision, recall) =
                calculate_for_threshold(estimations, ground_truth, voxel_numbers, threshold);
            self.dice_score_over_threshold[i] = dice;
            self.iou_over_threshold[i] = iou;
            self.precision_over_threshold[i] = precision;
            self.recall_over_threshold[i] = recall;
        }
    }
}

fn calculate_for_threshold(
    estimations: &Estimations,
    ground_truth: &VoxelTypes,
    voxel_numbers: &VoxelNumbers,
    threshold: f32,
) -> (f32, f32, f32, f32) {
    let predictions = predict_voxeltype(estimations, ground_truth, voxel_numbers, threshold);

    let dice = calculate_dice(&predictions, &ground_truth);
    let iou = calcultae_iou(&predictions, &ground_truth);
    let precision = calculate_precision(&predictions, &ground_truth);
    let recall = calculate_recall(&predictions, &ground_truth);

    (dice, iou, precision, recall)
}

#[allow(clippy::cast_precision_loss)]
fn calculate_recall(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
    let gt_positives = ground_truth
        .values
        .iter()
        .filter(|voxel_type| **voxel_type == VoxelType::Pathological)
        .count();

    let true_positives = predictions
        .values
        .iter()
        .zip(ground_truth.values.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth == VoxelType::Pathological && **prediction == VoxelType::Pathological
        })
        .count();

    if gt_positives == 0 {
        1.0
    } else {
        true_positives as f32 / gt_positives as f32
    }
}

#[allow(clippy::cast_precision_loss)]
fn calculate_precision(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
    let predicted_positves = predictions
        .values
        .iter()
        .filter(|voxel_type| **voxel_type == VoxelType::Pathological)
        .count();

    let true_positives = predictions
        .values
        .iter()
        .zip(ground_truth.values.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth == VoxelType::Pathological && **prediction == VoxelType::Pathological
        })
        .count();

    if predicted_positves == 0 {
        0.0
    } else {
        true_positives as f32 / predicted_positves as f32
    }
}

#[allow(clippy::cast_precision_loss)]
fn calcultae_iou(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
    let intersection = predictions
        .values
        .iter()
        .zip(ground_truth.values.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth == VoxelType::Pathological && **prediction == VoxelType::Pathological
        })
        .count();

    let union = predictions
        .values
        .iter()
        .zip(ground_truth.values.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth == VoxelType::Pathological || **prediction == VoxelType::Pathological
        })
        .count();

    if union == 0 {
        1.0
    } else {
        intersection as f32 / union as f32
    }
}

#[allow(clippy::cast_precision_loss)]
fn calculate_dice(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
    let true_positives = predictions
        .values
        .iter()
        .zip(ground_truth.values.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth == VoxelType::Pathological && **prediction == VoxelType::Pathological
        })
        .count();
    let false_positives = predictions
        .values
        .iter()
        .zip(ground_truth.values.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth != VoxelType::Pathological && **prediction == VoxelType::Pathological
        })
        .count();
    let false_negatives = predictions
        .values
        .iter()
        .zip(ground_truth.values.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth == VoxelType::Pathological && **prediction != VoxelType::Pathological
        })
        .count();

    let denominator = 2 * true_positives + false_positives + false_negatives;

    if denominator == 0 {
        1.0
    } else {
        (2 * true_positives) as f32 / denominator as f32
    }
}

pub fn predict_voxeltype(
    estimations: &Estimations,
    ground_truth: &VoxelTypes,
    voxel_numbers: &VoxelNumbers,
    threshold: f32,
) -> VoxelTypes {
    let mut predictions = VoxelTypes::empty([
        ground_truth.values.shape()[0],
        ground_truth.values.shape()[1],
        ground_truth.values.shape()[2],
    ]);

    let mut abs = Array1::zeros(estimations.system_states.values.shape()[0]);
    let system_states = &estimations.system_states.values;

    predictions
        .values
        .iter_mut()
        .zip(voxel_numbers.values.iter())
        .filter(|(_, number)| number.is_some())
        .for_each(|(prediction, number)| {
            let voxel_index = number.unwrap();
            abs.indexed_iter_mut().for_each(|(time_index, entry)| {
                *entry = system_states[[time_index, voxel_index]].abs()
                    + system_states[[time_index, voxel_index + 1]].abs()
                    + system_states[[time_index, voxel_index + 2]].abs();
            });
            if *abs.max_skipnan() <= threshold {
                *prediction = VoxelType::Pathological;
            } else {
                // just using ventricle here to differentiate the prediction
                // from pathological and none.
                // Might make more sense to introduce a 'healthy' type...
                *prediction = VoxelType::Ventricle;
            }
        });

    predictions
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayMetricsSample {
    pub values: Array1<f32>,
}

impl ArrayMetricsSample {
    #[must_use]
    pub fn new(number_of_epochs: usize, number_of_steps: usize) -> Self {
        Self {
            values: Array1::zeros(number_of_epochs * number_of_steps),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayMetricsEpoch {
    pub values: Array1<f32>,
}

impl ArrayMetricsEpoch {
    #[must_use]
    pub fn new(number_of_epochs: usize) -> Self {
        Self {
            values: Array1::zeros(number_of_epochs),
        }
    }
}
