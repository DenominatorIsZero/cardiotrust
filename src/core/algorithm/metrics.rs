use ndarray::Array1;
use ndarray_npy::WriteNpyExt;
use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};

use super::{estimation::Estimations, refinement::derivation::Derivatives};
use crate::core::model::spatial::voxels::{VoxelNumbers, VoxelType, VoxelTypes};

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
    /// Creates a new `Metrics` struct initialized with zeroed arrays for tracking metrics
    /// over epochs and steps.
    ///
    /// The length of the per-step arrays is set to `number_of_steps`, and the length of the
    /// per-epoch arrays is set to `number_of_epochs`.
    #[must_use]
    pub fn new(number_of_epochs: usize, number_of_steps: usize) -> Self {
        Self {
            loss: ArrayMetricsSample::new(number_of_steps),
            loss_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            loss_mse: ArrayMetricsSample::new(number_of_steps),
            loss_mse_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            loss_maximum_regularization: ArrayMetricsSample::new(number_of_steps),
            loss_maximum_regularization_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_states_mean: ArrayMetricsSample::new(number_of_steps),
            delta_states_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_states_max: ArrayMetricsSample::new(number_of_steps),
            delta_states_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_measurements_mean: ArrayMetricsSample::new(number_of_steps),
            delta_measurements_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_measurements_max: ArrayMetricsSample::new(number_of_steps),
            delta_measurements_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_gains_mean: ArrayMetricsSample::new(number_of_steps),
            delta_gains_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_gains_max: ArrayMetricsSample::new(number_of_steps),
            delta_gains_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            delta_delays_mean: ArrayMetricsSample::new(number_of_steps),
            delta_delays_mean_epoch: ArrayMetricsEpoch::new(number_of_epochs),
            delta_delays_max: ArrayMetricsSample::new(number_of_steps),
            delta_delays_max_epoch: ArrayMetricsEpoch::new(number_of_epochs),

            dice_score_over_threshold: Array1::zeros(101),
            iou_over_threshold: Array1::zeros(101),
            precision_over_threshold: Array1::zeros(101),
            recall_over_threshold: Array1::zeros(101),
        }
    }

    /// Calculates metrics for the current step.
    ///
    /// Updates the metrics fields with calculations for the current step:
    /// - MSE loss
    /// - Maximum regularization loss
    /// - Total loss
    /// - Mean and max of absolute deltas for:
    ///   - System states
    ///   - Measurements (residuals)
    ///   - Gains
    ///   - Delays
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
    ) {
        let index = time_index;

        self.loss_mse.values[index] = estimations.residuals.values.mapv(|v| v.powi(2)).sum()
            / estimations.residuals.values.raw_dim()[0] as f32;
        self.loss_maximum_regularization.values[index] = derivatives.maximum_regularization_sum;
        self.loss.values[index] = regularization_strength.mul_add(
            self.loss_maximum_regularization.values[index],
            self.loss_mse.values[index],
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

    /// Calculates epoch metrics by taking the mean of step metrics.
    ///
    /// # Panics
    ///
    /// Panics if any loss array is None.
    pub fn calculate_epoch(&mut self, epoch_index: usize) {
        self.loss_mse_epoch.values[epoch_index] = self.loss_mse.values.mean().unwrap();
        self.loss_maximum_regularization_epoch.values[epoch_index] =
            self.loss_maximum_regularization.values.mean().unwrap();
        self.loss_epoch.values[epoch_index] = self.loss.values.mean().unwrap();

        self.delta_states_mean_epoch.values[epoch_index] =
            self.delta_states_mean.values.mean().unwrap();
        self.delta_states_max_epoch.values[epoch_index] =
            *self.delta_states_max.values.max_skipnan();

        self.delta_measurements_mean_epoch.values[epoch_index] =
            self.delta_measurements_mean.values.mean().unwrap();
        self.delta_measurements_max_epoch.values[epoch_index] =
            *self.delta_measurements_max.values.max_skipnan();

        self.delta_gains_mean_epoch.values[epoch_index] =
            *self.delta_gains_mean.values.last().unwrap();
        self.delta_gains_max_epoch.values[epoch_index] =
            *self.delta_gains_max.values.last().unwrap();

        self.delta_delays_mean_epoch.values[epoch_index] =
            *self.delta_delays_mean.values.last().unwrap();
        self.delta_delays_max_epoch.values[epoch_index] =
            *self.delta_delays_max.values.last().unwrap();
    }

    /// Calculates metrics over the full range of thresholds from 0 to 1 by incrementing
    /// in steps of 0.01. Stores the dice score, `IoU`, precision, and recall for each
    /// threshold value in the given metric arrays.
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

    /// Saves all metric arrays to .npy files in the given path.
    /// Creates the directory if it does not exist.
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();

        self.loss.save_npy(path, "loss.npy");
        self.loss_epoch.save_npy(path, "loss_epoch.npy");

        self.loss_mse.save_npy(path, "loss_mse.npy");
        self.loss_mse_epoch.save_npy(path, "loss_mse_epoch.npy");
        self.loss_maximum_regularization
            .save_npy(path, "loss_maximum_regularization.npy");
        self.loss_maximum_regularization_epoch
            .save_npy(path, "loss_maximum_regularization_epoch.npy");

        self.delta_delays_mean
            .save_npy(path, "delta_states_mean.npy");
        self.delta_delays_mean_epoch
            .save_npy(path, "delta_states_mean_epoch.npy");
        self.delta_delays_max.save_npy(path, "delta_states_max.npy");
        self.delta_delays_max_epoch
            .save_npy(path, "delta_states_max_epoch.npy");

        self.delta_measurements_mean
            .save_npy(path, "delta_measurements_mean.npy");
        self.delta_measurements_mean_epoch
            .save_npy(path, "delta_measurements_mean_epoch.npy");
        self.delta_measurements_max
            .save_npy(path, "delta_measurements_max.npy");
        self.delta_measurements_max_epoch
            .save_npy(path, "delta_measurements_max_epoch.npy");

        self.delta_gains_mean.save_npy(path, "delta_gains_mean.npy");
        self.delta_gains_mean_epoch
            .save_npy(path, "delta_gains_mean_epoch.npy");
        self.delta_gains_max.save_npy(path, "delta_gains_max.npy");
        self.delta_gains_max_epoch
            .save_npy(path, "delta_gains_max_epoch.npy");

        self.delta_delays_mean
            .save_npy(path, "delta_delays_mean.npy");
        self.delta_delays_mean_epoch
            .save_npy(path, "delta_delays_mean_epoch.npy");
        self.delta_delays_max.save_npy(path, "delta_delays_max.npy");
        self.delta_delays_max_epoch
            .save_npy(path, "delta_delays_max_epoch.npy");

        let writer = BufWriter::new(File::create(path.join("dice.npy")).unwrap());
        self.dice_score_over_threshold.write_npy(writer).unwrap();

        let writer = BufWriter::new(File::create(path.join("iou.npy")).unwrap());
        self.iou_over_threshold.write_npy(writer).unwrap();

        let writer = BufWriter::new(File::create(path.join("precision.npy")).unwrap());
        self.precision_over_threshold.write_npy(writer).unwrap();

        let writer = BufWriter::new(File::create(path.join("recall.npy")).unwrap());
        self.recall_over_threshold.write_npy(writer).unwrap();
    }
}

/// Calculates Dice score, `IoU`, precision, and recall for the given estimations, ground truth, and voxel numbers at the specified threshold.
///
/// The estimations, ground truth, and voxel numbers are used to generate voxel type predictions at the given threshold.
/// These predictions are then compared to the ground truth to calculate the metrics.
fn calculate_for_threshold(
    estimations: &Estimations,
    ground_truth: &VoxelTypes,
    voxel_numbers: &VoxelNumbers,
    threshold: f32,
) -> (f32, f32, f32, f32) {
    let predictions = predict_voxeltype(estimations, ground_truth, voxel_numbers, threshold);

    let dice = calculate_dice(&predictions, ground_truth);
    let iou = calculate_iou(&predictions, ground_truth);
    let precision = calculate_precision(&predictions, ground_truth);
    let recall = calculate_recall(&predictions, ground_truth);

    (dice, iou, precision, recall)
}

/// Calculates the recall for the given predictions and ground truth voxel types.
///
/// Recall is defined as the ratio of true positives to total positives.
/// Returns 1.0 if there are no ground truth positives.
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

/// Calculates the precision for the given predictions and ground truth voxel types.
///
/// Precision is defined as the ratio of true positives to total predicted positives.
/// Returns 0.0 if there are no predicted positives.
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

/// Calculates the Intersection over Union (`IoU`) for the given predictions
/// and ground truth voxel types.
///
/// The `IoU` is defined as the ratio of the intersection (true positives)
/// to the union (true positives + false positives + false negatives).
/// Returns 0.0 if there is no intersection.
#[allow(clippy::cast_precision_loss)]
fn calculate_iou(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
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
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Calculates the Dice coefficient for the given predictions and ground
/// truth voxel types.
///
/// The Dice coefficient is defined as twice the number of true positives
/// divided by the total number of positives in both the predictions and
/// ground truth. It ranges from 0 to 1, with 1 being perfect agreement
/// between predictions and ground truth.
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
        0.0
    } else {
        (2 * true_positives) as f32 / denominator as f32
    }
}

/// Predicts the voxel type (pathological or ventricle) for each voxel, based on the
/// provided estimations and ground truth data. Voxels are predicted as pathological if
/// the maximum absolute value of the system state estimations for that voxel is below
/// the provided threshold. Otherwise they are predicted as ventricle.
///
/// # Panics
///
/// Panics if the provided estimations and ground truth data do not have the same shape.
#[must_use]
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
    /// Creates a new `ArrayMetricsSample` with the given number of steps, initializing
    /// the values to all zeros.
    #[must_use]
    pub fn new(number_of_steps: usize) -> Self {
        Self {
            values: Array1::zeros(number_of_steps),
        }
    }

    /// Saves the array values to a .npy file at the given path with the given name.
    /// Creates any missing directories in the path if needed.
    fn save_npy(&self, path: &std::path::Path, name: &str) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join(name)).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArrayMetricsEpoch {
    pub values: Array1<f32>,
}

impl ArrayMetricsEpoch {
    /// Creates a new `ArrayMetricsEpoch` with the given number of epochs, initializing
    /// the values to all zeros.
    #[must_use]
    pub fn new(number_of_epochs: usize) -> Self {
        Self {
            values: Array1::zeros(number_of_epochs),
        }
    }

    /// Saves the array values to a .npy file at the given path with the given name.  
    /// Creates any missing directories in the path if needed.
    fn save_npy(&self, path: &std::path::Path, name: &str) {
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join(name)).unwrap());
        self.values.write_npy(writer).unwrap();
    }
}
