use std::sync::mpsc::Sender;

use anyhow::{Context, Result};
use ndarray_stats::QuantileExt;
use tracing::{debug, info};

use super::{
    results::Results, summary::Summary, Scenario, ScenarioPayload, ScenarioStorage, Status,
};
use crate::core::{
    algorithm::{
        self, calculate_pseudo_inverse,
        gpu::{epoch::EpochKernel, GPU},
        metrics,
        refinement::derivation::calculate_average_delays,
    },
    config::algorithm::AlgorithmType,
    data::Data,
    model::Model,
};

/// Runs the simulation for the given scenario, model, and data.
///
/// Updates the results and summary structs with the output. Sends the final epoch
/// count and summary via the provided channels. Saves the results to the scenario.
///
/// # Errors
///
/// Returns an error if the model parameters are invalid, an unimplemented algorithm
/// is selected, or any other simulation failure occurs.
#[tracing::instrument(level = "info", skip_all, fields(id = %scenario.id))]
pub fn run(
    mut scenario: Scenario,
    storage: ScenarioStorage,
    epoch_tx: &Sender<usize>,
    summary_tx: &Sender<Summary>,
) -> Result<()> {
    debug!("Running scenario with id {}", scenario.id);

    let simulation = &scenario.config.simulation;

    let data = Data::from_simulation_config(simulation)
        .context("Failed to create simulation data from config - invalid model parameters")?;
    let mut model = Model::from_model_config(
        &scenario.config.algorithm.model,
        simulation.sample_rate_hz,
        simulation.duration_s,
    )
    .context("Failed to create model from config - invalid model parameters")?;

    // synchronice model and simulation sensor parameters
    model.synchronize_parameters(&data);

    let _ = epoch_tx.send(0);

    let number_of_snapshots = if scenario.config.algorithm.snapshots_interval == 0 {
        0
    } else {
        scenario.config.algorithm.epochs / scenario.config.algorithm.snapshots_interval + 1
    };

    let mut results = Results::new(
        scenario.config.algorithm.epochs,
        model.functional_description.control_function_values.shape()[0],
        model.spatial_description.sensors.count(),
        model.spatial_description.voxels.count_states(),
        model.spatial_description.sensors.count_beats(),
        number_of_snapshots,
        scenario.config.algorithm.batch_size,
        scenario.config.algorithm.optimizer,
    );

    let mut summary = Summary::default();

    match scenario.config.algorithm.algorithm_type {
        AlgorithmType::ModelBased => {
            results.model = Some(model);
            run_model_based(
                &mut scenario,
                &mut results,
                &data,
                &mut summary,
                epoch_tx,
                summary_tx,
            )
            .context("Failed to execute model-based algorithm")?;
        }
        AlgorithmType::ModelBasedGPU => {
            results.model = Some(model);
            run_model_based_gpu(
                &mut scenario,
                &mut results,
                &data,
                &mut summary,
                epoch_tx,
                summary_tx,
            )
            .context("Failed to execute model-based GPU algorithm")?;
        }
        AlgorithmType::PseudoInverse => {
            run_pseudo_inverse(&scenario, &model, &mut results, &data, &mut summary)
                .context("Failed to execute pseudo inverse algorithm")?;
            results.model = Some(model);
        }
    }

    super::calculate_plotting_arrays(&mut results, &data)?;

    metrics::calculate_final(
        &mut results.metrics,
        &results.estimations,
        &data.simulation.model.spatial_description.voxels.types,
        &results
            .model
            .as_ref()
            .context("Model should be set after algorithm execution")?
            .spatial_description
            .voxels
            .numbers,
    );

    let optimal_threshold = results
        .metrics
        .dice_score_over_threshold
        .argmax_skipnan()
        .unwrap_or_default();

    #[allow(clippy::cast_precision_loss)]
    {
        summary.threshold = optimal_threshold as f32 / 100.0;
    }
    summary.dice = results.metrics.dice_score_over_threshold[optimal_threshold];
    summary.iou = results.metrics.iou_over_threshold[optimal_threshold];
    summary.recall = results.metrics.recall_over_threshold[optimal_threshold];
    summary.precision = results.metrics.precision_over_threshold[optimal_threshold];

    let payload = ScenarioPayload { data, results };
    scenario.summary = Some(summary.clone());
    scenario.status = Status::Done;
    storage
        .save_metadata(&scenario)
        .context("Failed to save completed scenario metadata")?;
    storage
        .save_payload(scenario.get_id(), &payload)
        .context("Failed to save completed scenario payload")?;
    let _ = epoch_tx.send(scenario.config.algorithm.epochs - 1);
    let _ = summary_tx.send(summary);
    Ok(())
}

/// Runs the pseudo inverse algorithm on the given scenario, model, and data.
/// Calculates the pseudo inverse, runs estimations, and calculates summary metrics.
///
/// # Errors
///
/// Returns an error if the pseudo inverse algorithm fails due to SVD computation issues.
#[tracing::instrument(level = "info", skip_all)]
fn run_pseudo_inverse(
    scenario: &Scenario,
    model: &Model,
    results: &mut Results,
    data: &Data,
    summary: &mut Summary,
) -> Result<()> {
    info!("Running pseudo inverse algorithm");
    calculate_pseudo_inverse(
        &model.functional_description,
        results,
        data,
        &scenario.config.algorithm,
    )?;
    summary.loss = results.metrics.loss_batch[0];
    summary.loss_mse = results.metrics.loss_mse_batch[0];
    summary.loss_maximum_regularization = results.metrics.loss_maximum_regularization_batch[0];
    Ok(())
}

/// Runs the model-based algorithm on the given scenario, model, and data.
/// Calculates model parameters over epochs and calculates summary metrics.
/// Reduces learning rate at intervals. Saves snapshots at intervals.
/// Sends epoch and summary updates over channels.
/// Exits early if loss becomes non-finite.
#[tracing::instrument(level = "info", skip_all)]
fn run_model_based(
    scenario: &mut Scenario,
    results: &mut Results,
    data: &Data,
    summary: &mut Summary,
    epoch_tx: &Sender<usize>,
    summary_tx: &Sender<Summary>,
) -> Result<()> {
    info!("Running model-based algorithm");
    let original_learning_rate = scenario.config.algorithm.learning_rate;
    let mut batch_index = 0;
    for epoch_index in 0..scenario.config.algorithm.epochs {
        if epoch_index == 0 {
            scenario.config.algorithm.learning_rate = 0.0;
        } else if epoch_index == 1 {
            scenario.config.algorithm.learning_rate = original_learning_rate;
        }
        if scenario.config.algorithm.learning_rate_reduction_interval != 0
            && (epoch_index % scenario.config.algorithm.learning_rate_reduction_interval == 0)
        {
            scenario.config.algorithm.learning_rate *=
                scenario.config.algorithm.learning_rate_reduction_factor;
        }
        algorithm::run_epoch(results, &mut batch_index, data, &scenario.config.algorithm)
            .with_context(|| format!("Failed to run algorithm epoch {epoch_index}"))?;
        scenario.status = Status::Running(epoch_index);

        summary.loss = results.metrics.loss_batch[batch_index - 1];
        summary.loss_mse = results.metrics.loss_mse_batch[batch_index - 1];
        summary.loss_maximum_regularization =
            results.metrics.loss_maximum_regularization_batch[batch_index - 1];

        if scenario.config.algorithm.snapshots_interval != 0
            && epoch_index % scenario.config.algorithm.snapshots_interval == 0
        {
            results
                .snapshots
                .as_mut()
                .context("Snapshots should be initialized for GPU algorithm")?
                .push(
                    &results.estimations,
                    &results
                        .model
                        .as_ref()
                        .context("Model should be set during GPU algorithm execution")?
                        .functional_description
                        .ap_params,
                );
        }

        let _ = epoch_tx.send(epoch_index);
        let _ = summary_tx.send(summary.clone());
        // Check if algorithm diverged. If so return early
        if !summary.loss.is_normal() {
            break;
        }
    }
    calculate_average_delays(
        &mut results.estimations.average_delays,
        &results
            .model
            .as_ref()
            .context("Model should be set during algorithm execution")?
            .functional_description
            .ap_params,
    )?;
    scenario.config.algorithm.learning_rate = original_learning_rate;
    Ok(())
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
#[tracing::instrument(level = "info", skip_all)]
fn run_model_based_gpu(
    scenario: &mut Scenario,
    results: &mut Results,
    data: &Data,
    summary: &mut Summary,
    epoch_tx: &Sender<usize>,
    summary_tx: &Sender<Summary>,
) -> Result<()> {
    info!("Running model-based algorithm on gpu");
    // move data to gpu
    let gpu = GPU::new()?;
    let results_gpu = results.to_gpu(&gpu.queue)?;
    let actual_measurements = data.simulation.measurements.to_gpu(&gpu.queue)?;
    let number_of_states = results
        .model
        .as_ref()
        .context("Model should be set during GPU algorithm execution")?
        .spatial_description
        .voxels
        .count_states();
    let number_of_sensors = results
        .model
        .as_ref()
        .context("Model should be set during GPU algorithm execution")?
        .spatial_description
        .sensors
        .count();
    let number_of_steps = results.estimations.measurements.num_steps();
    let mut epoch_kernel = EpochKernel::new(
        &gpu,
        &results_gpu,
        &actual_measurements,
        &scenario.config.algorithm,
        number_of_states as i32,
        number_of_sensors as i32,
        number_of_steps as i32,
    )?;

    for epoch_index in 0..scenario.config.algorithm.epochs {
        if epoch_index == 0 {
            epoch_kernel.set_freeze_delays(true);
            epoch_kernel.set_freeze_gains(true);
        } else if epoch_index == 1 {
            epoch_kernel.set_freeze_delays(scenario.config.algorithm.freeze_delays);
            epoch_kernel.set_freeze_gains(scenario.config.algorithm.freeze_gains);
        }
        epoch_kernel.execute()?;
        results.metrics.update_from_gpu(&results_gpu.metrics)?;

        summary.loss = results.metrics.loss_batch[epoch_index];
        summary.loss_mse = results.metrics.loss_mse_batch[epoch_index];
        summary.loss_maximum_regularization =
            results.metrics.loss_maximum_regularization_batch[epoch_index];

        if scenario.config.algorithm.snapshots_interval != 0
            && epoch_index % scenario.config.algorithm.snapshots_interval == 0
        {
            results
                .estimations
                .update_from_gpu(&results_gpu.estimations)?;
            results
                .model
                .as_mut()
                .context("Model should be set during GPU algorithm execution")?
                .functional_description
                .ap_params
                .update_from_gpu(&results_gpu.model.functional_description.ap_params)?;
            results
                .snapshots
                .as_mut()
                .context("Snapshots should be initialized for GPU algorithm")?
                .push(
                    &results.estimations,
                    &results
                        .model
                        .as_ref()
                        .context("Model should be set during GPU algorithm execution")?
                        .functional_description
                        .ap_params,
                );
        }

        let _ = epoch_tx.send(epoch_index);
        let _ = summary_tx.send(summary.clone());
        // Check if algorithm diverged. If so return early
        if !summary.loss.is_normal() {
            break;
        }
    }
    results.update_from_gpu(&results_gpu)?;
    calculate_average_delays(
        &mut results.estimations.average_delays,
        &results
            .model
            .as_ref()
            .context("Model should be set during GPU algorithm execution")?
            .functional_description
            .ap_params,
    )?;
    Ok(())
}
