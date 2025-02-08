use std::{
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};

use ndarray::Array1;
use ndarray_npy::WriteNpyExt;
use ndarray_stats::QuantileExt;
use ocl::Buffer;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use super::estimation::Estimations;
use crate::core::model::spatial::voxels::{VoxelNumbers, VoxelType, VoxelTypes};

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub loss: SampleWiseMetric,
    pub loss_batch: BatchWiseMetric,

    pub loss_mse: SampleWiseMetric,
    pub loss_mse_batch: BatchWiseMetric,
    pub loss_maximum_regularization: SampleWiseMetric,
    pub loss_maximum_regularization_batch: BatchWiseMetric,

    #[serde(default)]
    pub dice_score_over_threshold: Array1<f32>,
    #[serde(default)]
    pub iou_over_threshold: Array1<f32>,
    #[serde(default)]
    pub precision_over_threshold: Array1<f32>,
    #[serde(default)]
    pub recall_over_threshold: Array1<f32>,
}

pub struct MetricsGPU {
    pub loss: Buffer<f32>,
    pub loss_batch: Buffer<f32>,
    pub loss_mse: Buffer<f32>,
    pub loss_mse_batch: Buffer<f32>,
    pub loss_maximum_regularization: Buffer<f32>,
    pub loss_maximum_regularization_batch: Buffer<f32>,
}

impl Metrics {
    /// Creates a new `Metrics` struct initialized with zeroed arrays for tracking metrics
    /// over epochs and steps.
    ///
    /// The length of the per-step arrays is set to `number_of_steps`, and the length of the
    /// per-epoch arrays is set to `number_of_epochs`.
    #[must_use]
    #[tracing::instrument(level = "debug")]
    pub fn new(number_of_epochs: usize, number_of_steps: usize, number_of_batches: usize) -> Self {
        debug!("Creating new Metrics struct");
        Self {
            loss: SampleWiseMetric::new(number_of_steps),
            loss_batch: BatchWiseMetric::new(number_of_epochs, number_of_batches),

            loss_mse: SampleWiseMetric::new(number_of_steps),
            loss_mse_batch: BatchWiseMetric::new(number_of_epochs, number_of_batches),
            loss_maximum_regularization: SampleWiseMetric::new(number_of_steps),
            loss_maximum_regularization_batch: BatchWiseMetric::new(
                number_of_epochs,
                number_of_batches,
            ),

            dice_score_over_threshold: Array1::zeros(101),
            iou_over_threshold: Array1::zeros(101),
            precision_over_threshold: Array1::zeros(101),
            recall_over_threshold: Array1::zeros(101),
        }
    }

    /// Saves all metric arrays to .npy files in the given path.
    /// Creates the directory if it does not exist.
    #[tracing::instrument(level = "trace")]
    pub(crate) fn save_npy(&self, path: &std::path::Path) {
        trace!("Saving metrics to npy");
        fs::create_dir_all(path).unwrap();

        self.loss.save_npy(path, "loss.npy");
        self.loss_batch.save_npy(path, "loss_epoch.npy");

        self.loss_mse.save_npy(path, "loss_mse.npy");
        self.loss_mse_batch.save_npy(path, "loss_mse_epoch.npy");
        self.loss_maximum_regularization
            .save_npy(path, "loss_maximum_regularization.npy");
        self.loss_maximum_regularization_batch
            .save_npy(path, "loss_maximum_regularization_epoch.npy");

        let writer = BufWriter::new(File::create(path.join("dice.npy")).unwrap());
        self.dice_score_over_threshold.write_npy(writer).unwrap();

        let writer = BufWriter::new(File::create(path.join("iou.npy")).unwrap());
        self.iou_over_threshold.write_npy(writer).unwrap();

        let writer = BufWriter::new(File::create(path.join("precision.npy")).unwrap());
        self.precision_over_threshold.write_npy(writer).unwrap();

        let writer = BufWriter::new(File::create(path.join("recall.npy")).unwrap());
        self.recall_over_threshold.write_npy(writer).unwrap();
    }

    pub(crate) fn to_gpu(&self, queue: &ocl::Queue) -> MetricsGPU {
        MetricsGPU {
            loss: self.loss.to_gpu(queue),
            loss_batch: self.loss_batch.to_gpu(queue),
            loss_mse: self.loss_mse.to_gpu(queue),
            loss_mse_batch: self.loss_mse_batch.to_gpu(queue),
            loss_maximum_regularization: self.loss_maximum_regularization.to_gpu(queue),
            loss_maximum_regularization_batch: self.loss_maximum_regularization_batch.to_gpu(queue),
        }
    }

    pub(crate) fn update_from_gpu(&mut self, metrics: &MetricsGPU) {
        self.loss.update_from_gpu(&metrics.loss);
        self.loss_batch.update_from_gpu(&metrics.loss_batch);
        self.loss_mse.update_from_gpu(&metrics.loss_mse);
        self.loss_mse_batch.update_from_gpu(&metrics.loss_mse_batch);
        self.loss_maximum_regularization
            .update_from_gpu(&metrics.loss_maximum_regularization);
        self.loss_maximum_regularization_batch
            .update_from_gpu(&metrics.loss_maximum_regularization_batch);
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
#[tracing::instrument(level = "trace", skip_all)]
pub fn calculate_step(
    metrics: &mut Metrics,
    estimations: &Estimations,
    maximum_regularization_sum: f32,
    regularization_strength: f32,
    step: usize,
) {
    trace!("Calculating metrics for step {}", step);

    metrics.loss_mse[step] = estimations.residuals.mapv(|v| v.powi(2)).sum()
        / estimations.measurements.num_sensors() as f32;
    metrics.loss_maximum_regularization[step] = maximum_regularization_sum;
    metrics.loss[step] = regularization_strength.mul_add(
        metrics.loss_maximum_regularization[step],
        metrics.loss_mse[step],
    );
}

/// Calculates epoch metrics by taking the mean of step metrics.
///
/// # Panics
///
/// Panics if any loss array is None.
#[tracing::instrument(level = "debug")]
pub fn calculate_batch(metrics: &mut Metrics, epoch_index: usize) {
    debug!("Calculating metrics for epoch {}", epoch_index);
    metrics.loss_mse_batch[epoch_index] = metrics.loss_mse.mean().unwrap();
    metrics.loss_maximum_regularization_batch[epoch_index] =
        metrics.loss_maximum_regularization.mean().unwrap();
    metrics.loss_batch[epoch_index] = metrics.loss.mean().unwrap();
}

/// Calculates metrics over the full range of thresholds from 0 to 1 by incrementing
/// in steps of 0.01. Stores the dice score, `IoU`, precision, and recall for each
/// threshold value in the given metric arrays.
#[allow(clippy::cast_precision_loss)]
#[tracing::instrument(level = "debug", skip_all)]
pub fn calculate_final(
    metrics: &mut Metrics,
    estimations: &Estimations,
    ground_truth: &VoxelTypes,
    voxel_numbers: &VoxelNumbers,
) {
    debug!("Calculating final metrics");
    for i in 0..=100 {
        let threshold = i as f32 / 100.0;
        let (dice, iou, precision, recall) =
            calculate_for_threshold(estimations, ground_truth, voxel_numbers, threshold);
        metrics.dice_score_over_threshold[i] = dice;
        metrics.iou_over_threshold[i] = iou;
        metrics.precision_over_threshold[i] = precision;
        metrics.recall_over_threshold[i] = recall;
    }
}
/// Calculates Dice score, `IoU`, precision, and recall for the given estimations, ground truth, and voxel numbers at the specified threshold.
///
/// The estimations, ground truth, and voxel numbers are used to generate voxel type predictions at the given threshold.
/// These predictions are then compared to the ground truth to calculate the metrics.
#[tracing::instrument(level = "trace")]
fn calculate_for_threshold(
    estimations: &Estimations,
    ground_truth: &VoxelTypes,
    voxel_numbers: &VoxelNumbers,
    threshold: f32,
) -> (f32, f32, f32, f32) {
    trace!(
        "Calculating segmentation metrics for threshold {}",
        threshold
    );
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
#[tracing::instrument(level = "trace")]
fn calculate_recall(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
    trace!("Calculating recall");
    let gt_positives = ground_truth
        .iter()
        .filter(|voxel_type| **voxel_type == VoxelType::Pathological)
        .count();

    let true_positives = predictions
        .iter()
        .zip(ground_truth.iter())
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
#[tracing::instrument(level = "trace")]
fn calculate_precision(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
    trace!("Calculating precision");
    let predicted_positves = predictions
        .iter()
        .filter(|voxel_type| **voxel_type == VoxelType::Pathological)
        .count();

    let true_positives = predictions
        .iter()
        .zip(ground_truth.iter())
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
#[tracing::instrument(level = "trace")]
fn calculate_iou(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
    trace!("Calculating IoU");
    let intersection = predictions
        .iter()
        .zip(ground_truth.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth == VoxelType::Pathological && **prediction == VoxelType::Pathological
        })
        .count();

    let union = predictions
        .iter()
        .zip(ground_truth.iter())
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
#[tracing::instrument(level = "trace")]
fn calculate_dice(predictions: &VoxelTypes, ground_truth: &VoxelTypes) -> f32 {
    trace!("Calculating Dice");
    let true_positives = predictions
        .iter()
        .zip(ground_truth.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth == VoxelType::Pathological && **prediction == VoxelType::Pathological
        })
        .count();
    let false_positives = predictions
        .iter()
        .zip(ground_truth.iter())
        .filter(|(prediction, ground_truth)| {
            **ground_truth != VoxelType::Pathological && **prediction == VoxelType::Pathological
        })
        .count();
    let false_negatives = predictions
        .iter()
        .zip(ground_truth.iter())
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
#[tracing::instrument(level = "trace")]
pub fn predict_voxeltype(
    estimations: &Estimations,
    ground_truth: &VoxelTypes,
    voxel_numbers: &VoxelNumbers,
    threshold: f32,
) -> VoxelTypes {
    trace!("Predicting voxel types");
    let mut predictions = VoxelTypes::empty([
        ground_truth.shape()[0],
        ground_truth.shape()[1],
        ground_truth.shape()[2],
    ]);

    let mut abs = Array1::zeros(estimations.system_states.shape()[0]);
    let system_states = &estimations.system_states;

    predictions
        .iter_mut()
        .zip(voxel_numbers.iter())
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
pub struct SampleWiseMetric(Array1<f32>);

impl SampleWiseMetric {
    /// Creates a new `ArrayMetricsSample` with the given number of steps, initializing
    /// the values to all zeros.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn new(number_of_steps: usize) -> Self {
        trace!("Creating ArrayMetricsSample");
        Self(Array1::zeros(number_of_steps))
    }

    /// Saves the array values to a .npy file at the given path with the given name.
    /// Creates any missing directories in the path if needed.
    #[tracing::instrument(level = "trace")]
    fn save_npy(&self, path: &std::path::Path, name: &str) {
        trace!("Saving ArrayMetricsSample");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join(name)).unwrap());
        self.write_npy(writer).unwrap();
    }

    fn to_gpu(&self, queue: &ocl::Queue) -> Buffer<f32> {
        Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(self.as_slice().unwrap())
            .build()
            .unwrap()
    }

    fn update_from_gpu(&mut self, loss: &Buffer<f32>) {
        loss.read(self.as_slice_mut().unwrap()).enq().unwrap();
    }
}

impl Deref for SampleWiseMetric {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SampleWiseMetric {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BatchWiseMetric(Array1<f32>);

impl BatchWiseMetric {
    /// Creates a new `ArrayMetricsEpoch` with the given number of epochs, initializing
    /// the values to all zeros.
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn new(number_of_epochs: usize, number_of_batches: usize) -> Self {
        trace!("Creating ArrayMetricsEpoch");
        Self(Array1::zeros(number_of_epochs * number_of_batches))
    }

    /// Saves the array values to a .npy file at the given path with the given name.  
    /// Creates any missing directories in the path if needed.
    #[tracing::instrument(level = "trace")]
    fn save_npy(&self, path: &std::path::Path, name: &str) {
        trace!("Saving ArrayMetricsEpoch to npy");
        fs::create_dir_all(path).unwrap();
        let writer = BufWriter::new(File::create(path.join(name)).unwrap());
        self.write_npy(writer).unwrap();
    }

    fn to_gpu(&self, queue: &ocl::Queue) -> Buffer<f32> {
        Buffer::builder()
            .queue(queue.clone())
            .len(self.len())
            .copy_host_slice(self.as_slice().unwrap())
            .build()
            .unwrap()
    }

    fn update_from_gpu(&mut self, loss_batch: &Buffer<f32>) {
        loss_batch.read(self.as_slice_mut().unwrap()).enq().unwrap();
    }
}

impl Deref for BatchWiseMetric {
    type Target = Array1<f32>;

    #[tracing::instrument(level = "trace")]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BatchWiseMetric {
    #[tracing::instrument(level = "trace")]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
