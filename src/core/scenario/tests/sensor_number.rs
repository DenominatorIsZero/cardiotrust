use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
    sync::mpsc::channel,
    thread,
};

use anyhow::{Context, Result};
use ndarray::Array1;
use ndarray_npy::WriteNpyExt;

use super::RUN_IN_TESTS;
use crate::{
    core::{
        algorithm::{metrics::BatchWiseMetric, refinement::Optimizer},
        config::model::SensorArrayGeometry,
        model::functional::allpass::from_coef_to_samples,
        scenario::{run, tests::SAVE_NPY, Scenario},
    },
    tests::{clean_files, setup_folder},
    vis::plotting::png::line::{line_plot, log_y_plot},
};

const COMMON_PATH: &str = "tests/core/scenario/sensor_number/";
const NUMBER_OF_AP: i32 = 50;
const VOXELS_PER_AXIS: i32 = 7;
const NUMBER_OF_EPOCHS_LINE: usize = 20_000;
const NUMBER_OF_EPOCHS_GRID: usize = 20_000;
const LEARNING_RATE_LINE: f32 = 1e5;
const LEARNING_RATE_GRID: f32 = 1e4;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum ScenarioType {
    Line,
    Sheet,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive integration test"]
fn heavy_sensor_number_sheet() -> Result<()> {
    let base_id = "Sensor Number Sheet";
    let path = Path::new(COMMON_PATH);

    let number_of_sensors = vec![1, 64];
    let measurement_varinaces = Array1::<f32>::logspace(10.0, -12.0, 0.0, 4);
    let trials = 1;
    let scenario_type = ScenarioType::Sheet;

    create_and_run(
        scenario_type,
        number_of_sensors,
        measurement_varinaces,
        trials,
        base_id,
        path,
    )?;
    Ok(())
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive integration test"]
fn heavy_sensor_number_line() -> Result<()> {
    let base_id = "Sensor Number Line";
    let path = Path::new(COMMON_PATH);

    let number_of_sensors = vec![1, 4, 8, 16, 32, 48, 64];
    let measurement_varinaces = Array1::<f32>::logspace(10.0, -12.0, 0.0, 10);
    let trials = 10;
    let scenario_type = ScenarioType::Line;

    create_and_run(
        scenario_type,
        number_of_sensors,
        measurement_varinaces,
        trials,
        base_id,
        path,
    )?;
    Ok(())
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[tracing::instrument(level = "trace")]
fn build_scenario(
    target_delay_samples: f32,
    initial_delay_samples: f32,
    number_of_sensors: i32,
    measurement_covarince: f32,
    scenario_type: ScenarioType,
    id: &str,
) -> Result<Scenario> {
    let mut scenario = Scenario::build(Some(id.to_string()))?;

    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let target_delay_s = target_delay_samples / sample_rate_hz;
    let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;

    let initial_delay_s = initial_delay_samples / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    // Set pathological true
    scenario.config.simulation.model.common.pathological = true;
    scenario.config.simulation.model.common.voxel_size_mm = voxel_size_mm;
    scenario
        .config
        .simulation
        .model
        .common
        .current_factor_in_pathology = 1.0;
    scenario
        .config
        .simulation
        .model
        .common
        .measurement_covariance_mean = measurement_covarince;
    // Adjust heart size
    match scenario_type {
        ScenarioType::Line => {
            scenario
                .config
                .simulation
                .model
                .handcrafted
                .as_mut()
                .context("Handcrafted model should be available for configuration")?
                .heart_size_mm = [
                voxel_size_mm,
                voxel_size_mm * (NUMBER_OF_AP + 1) as f32,
                voxel_size_mm,
            ];
            scenario.config.simulation.model.common.heart_offset_mm = [
                25.0,
                -250.0 - (voxel_size_mm * (NUMBER_OF_AP + 1) as f32) / 2.0,
                180.0,
            ];
        }
        ScenarioType::Sheet => {
            scenario
                .config
                .simulation
                .model
                .handcrafted
                .as_mut()
                .context("Handcrafted model should be available for configuration")?
                .heart_size_mm = [
                voxel_size_mm * (VOXELS_PER_AXIS) as f32,
                voxel_size_mm * (VOXELS_PER_AXIS) as f32,
                voxel_size_mm,
            ];
            scenario.config.simulation.model.common.heart_offset_mm = [
                25.0 - (voxel_size_mm * (VOXELS_PER_AXIS) as f32) / 2.0,
                -250.0 - (voxel_size_mm * (VOXELS_PER_AXIS) as f32) / 2.0,
                180.0,
            ];
            scenario
                .config
                .simulation
                .model
                .handcrafted
                .as_mut()
                .context("Handcrafted model should be available for configuration")?
                .sa_x_center_percentage = 0.5;
            scenario
                .config
                .simulation
                .model
                .handcrafted
                .as_mut()
                .context("Handcrafted model should be available for configuration")?
                .sa_y_center_percentage = 1.0;
        }
    }
    // Adjust pathology
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model should be available for pathology configuration")?
        .pathology_x_start_percentage = 0.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model should be available for pathology configuration")?
        .pathology_x_stop_percentage = 1.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model should be available for pathology configuration")?
        .pathology_y_start_percentage = 0.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model should be available for pathology configuration")?
        .pathology_y_stop_percentage = 1.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model should be available for sensor configuration")?
        .sa_y_center_percentage = 1.0;
    scenario
        .config
        .simulation
        .model
        .common
        .sensor_array_geometry = SensorArrayGeometry::SparseCube;
    scenario.config.simulation.model.common.sensors_per_axis = [4, 4, 4];
    scenario.config.simulation.model.common.number_of_sensors = number_of_sensors as usize;
    // Copy settings to algorithm model
    scenario.config.algorithm.model = scenario.config.simulation.model.clone();
    // Adjust propagation velocities
    scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities
        .sinoatrial = target_velocity;
    scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities
        .pathological = target_velocity;
    scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities
        .sinoatrial = initial_velocity;
    scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities
        .pathological = initial_velocity;
    // set optimization parameters
    scenario.config.algorithm.epochs = match scenario_type {
        ScenarioType::Line => NUMBER_OF_EPOCHS_LINE,
        ScenarioType::Sheet => NUMBER_OF_EPOCHS_GRID,
    };
    scenario.config.algorithm.learning_rate = match scenario_type {
        ScenarioType::Line => LEARNING_RATE_LINE,
        ScenarioType::Sheet => LEARNING_RATE_GRID,
    };
    scenario.config.algorithm.optimizer = Optimizer::Sgd;
    scenario.config.algorithm.freeze_delays = false;
    scenario.config.algorithm.freeze_gains = true;
    scenario.config.algorithm.difference_regularization_strength = 0.0;
    scenario.config.algorithm.slow_down_stregth = 0.0;
    let number_of_snapshots = 1000;
    scenario.config.algorithm.snapshots_interval =
        scenario.config.algorithm.epochs / number_of_snapshots;

    scenario.schedule().context("Failed to schedule scenario")?;
    let _ = scenario.save();
    Ok(scenario)
}

#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
#[tracing::instrument(level = "trace")]
fn plot_results(
    path: &Path,
    base_title: &str,
    scenarios: &Vec<Scenario>,
    number_of_sensors: Vec<i32>,
    measurement_covariances: Array1<f32>,
    trials: usize,
    scenario_type: ScenarioType,
) -> Result<()> {
    setup_folder(path)?;
    for number_of_sensors in &number_of_sensors {
        for trial in 0..trials {
            let files = vec![
                path.join(format!("num_{number_of_sensors:03}_loss.png")),
                path.join(format!(
                    "num_{number_of_sensors:03}_trial{trial}_delays.png"
                )),
                path.join(format!(
                    "num{number_of_sensors:03}_trial{trial}_delays_error.png"
                )),
            ];
            clean_files(&files)?;
        }
    }

    let mut first_scenario = scenarios
        .first()
        .context("Expected at least one scenario to plot")?
        .clone();
    println!("Loading data for first scenario");
    first_scenario.load_data()?;
    println!("Loading results for first scenario {:?}", first_scenario.id);
    first_scenario.load_results()?;

    println!(
        "{:?}",
        first_scenario
            .results
            .as_ref()
            .context("Results should be available for plotting")?
            .model
            .as_ref()
            .context("Model should be available in results for plotting")?
            .functional_description
            .ap_params
            .coefs
    );
    let x_epochs = Array1::range(0.0, first_scenario.config.algorithm.epochs as f32, 1.0);
    let num_snapshots = first_scenario
        .results
        .as_ref()
        .context("Results should be available for snapshot plotting")?
        .snapshots
        .as_ref()
        .context("Snapshots should be available for plotting")?
        .number_of_snapshots;
    let x_snapshots = Array1::range(0.0, num_snapshots as f32, 1.0);

    for number_of_sensors in number_of_sensors {
        for measurement_covariance in &measurement_covariances {
            let mut losses_owned: Vec<BatchWiseMetric> = Vec::new();
            let mut labels_owned: Vec<String> = Vec::new();

            for trial in 0..trials {
                for scenario in scenarios {
                    if !scenario
                        .id
                        .contains(&format!("Num: {number_of_sensors:03},"))
                        || !scenario.id.contains(&format!("trial: {trial:03}"))
                        || !scenario.id.contains(&format!("{scenario_type:?}"))
                        || !scenario
                            .id
                            .contains(&format!("Noise: {measurement_covariance:.3e}"))
                        || !scenario
                            .summary
                            .as_ref()
                            .context("Scenario summary should be available")?
                            .loss
                            .is_finite()
                    {
                        continue;
                    }
                    let mut delays_owned: Vec<Array1<f32>> = Vec::new();
                    let mut delays_error_owned: Vec<Array1<f32>> = Vec::new();
                    let mut scenario = (*scenario).clone();
                    scenario.load_results()?;
                    scenario.load_data()?;
                    losses_owned.push(
                        scenario
                            .results
                            .as_ref()
                            .context("Results should be available for loss extraction")?
                            .metrics
                            .loss_mse_batch
                            .clone(),
                    );
                    labels_owned.push(format!("trial {trial:03}"));

                    let results_model = scenario
                        .results
                        .as_ref()
                        .context("Results should be available for model validation")?
                        .model
                        .as_ref()
                        .context("Model should be available in results for validation")?;
                    let data_model = scenario
                        .data
                        .as_ref()
                        .context("Data should be available for model validation")?;
                    assert_eq!(
                        results_model.functional_description.measurement_matrix,
                        data_model
                            .simulation
                            .model
                            .functional_description
                            .measurement_matrix
                    );
                    match scenario_type {
                        ScenarioType::Line => {
                            for ap in 0..NUMBER_OF_AP as usize {
                                let mut delays = Array1::<f32>::zeros(num_snapshots);
                                let mut delays_error = Array1::<f32>::zeros(num_snapshots);
                                let scenario_data = scenario.data.as_ref().context(
                                    "Scenario data should be available for delay calculation",
                                )?;
                                let target_delay = from_coef_to_samples(
                                    scenario_data
                                        .simulation
                                        .model
                                        .functional_description
                                        .ap_params
                                        .coefs[(ap, 15)],
                                ) + scenario_data
                                    .simulation
                                    .model
                                    .functional_description
                                    .ap_params
                                    .delays[(ap, 15)]
                                    as f32;
                                let snapshots = scenario
                                    .results
                                    .as_ref()
                                    .context("Results should be available for snapshot access")?
                                    .snapshots
                                    .as_ref()
                                    .context(
                                        "Snapshots should be available for delay extraction",
                                    )?;
                                for i in 0..num_snapshots {
                                    delays[i] =
                                        from_coef_to_samples(snapshots.ap_coefs[(i, ap, 15)])
                                            + snapshots.ap_delays[(i, ap, 15)] as f32;
                                    delays_error[i] = target_delay - delays[i];
                                }
                                delays_owned.push(delays);
                                delays_error_owned.push(delays_error);
                            }
                        }
                        ScenarioType::Sheet => {
                            panic!("Not implemented yet");
                        }
                    }

                    let delays = delays_owned.iter().collect::<Vec<&Array1<f32>>>();
                    let delays_error = delays_error_owned.iter().collect::<Vec<&Array1<f32>>>();

                    if SAVE_NPY {
                        let path = path.join("npy");
                        fs::create_dir_all(&path).context("Failed to create NPY directory")?;
                        for (i, delay) in delays.iter().enumerate() {
                            let writer = BufWriter::new(
                                File::create(path.join(format!(
                                    "{scenario_type:?} num: {number_of_sensors:03}, noise: {measurement_covariance:.3e} trial {trial:03} delay: {i:03}.npy",
                                )))
                                .context("Failed to create NPY file for delay data")?,
                            );
                            delay
                                .write_npy(writer)
                                .context("Failed to write delay data to NPY file")?;
                        }
                    }

                    line_plot(
                        Some(&x_snapshots),
                        delays,
                        Some(&path.join(format!(
                            "{scenario_type:?}_num_{number_of_sensors:03}_noise_{measurement_covariance:.3e}_trial_{trial:03}_delays.png"
                        ))),
                        Some(format!("{base_title} - AP Delay").as_str()),
                        Some("AP Delay (Estimated)"),
                        Some("Snapshot"),
                        None,
                        None,
                    )
                    .context("Failed to create delays plot")?;

                    line_plot(
                        Some(&x_snapshots),
                        delays_error,
                        Some(&path.join(format!(
                            "{scenario_type:?}_num_{number_of_sensors:03}_noise_{measurement_covariance:.3e}_trial_{trial:03}_delays_error.png"
                        ))),
                        Some(format!("{base_title} - AP Delay Error").as_str()),
                        Some("AP Delay (Target - Estimated)"),
                        Some("Snapshot"),
                        None,
                        None,
                    )
                    .context("Failed to create delays error plot")?;
                    drop(scenario);
                }
            }

            let losses = losses_owned
                .iter()
                .map(std::ops::Deref::deref)
                .collect::<Vec<&Array1<f32>>>();
            let labels: Vec<&str> = labels_owned
                .iter()
                .map(std::string::String::as_str)
                .collect();

            if SAVE_NPY {
                let path = path.join("npy");
                fs::create_dir_all(&path).context("Failed to create NPY directory for results")?;
                let writer = BufWriter::new(
                    File::create(path.join(format!("x_epochs_{scenario_type:?}.npy")))
                        .context("Failed to create x_epochs NPY file")?,
                );
                x_epochs
                    .write_npy(writer)
                    .context("Failed to write x_epochs NPY data")?;
                let writer = BufWriter::new(
                    File::create(path.join(format!("x_snapshots_{scenario_type:?}.npy")))
                        .context("Failed to create x_snapshots NPY file")?,
                );
                x_snapshots
                    .write_npy(writer)
                    .context("Failed to write x_snapshots NPY data")?;
                for (label, loss) in labels.iter().zip(losses.iter()) {
                    let writer = BufWriter::new(
                        File::create(path.join(format!(
                            "loss - {scenario_type:?} - num: {number_of_sensors:03} - noise: {measurement_covariance:.3e} {label}.npy"
                        )))
                        .context("Failed to create loss NPY file")?,
                    );
                    loss.write_npy(writer)
                        .context("Failed to write loss NPY data")?;
                }
            }

            log_y_plot(
                Some(&x_epochs),
                losses,
                Some(&path.join(format!(
                    "{scenario_type:?}_num_{number_of_sensors:03}_noise_{measurement_covariance:.3e}_loss.png"
                ))),
                Some(format!("{base_title} - Loss").as_str()),
                Some("Loss MSE"),
                Some("Epoch"),
                Some(&labels),
                None,
            )
            .context("Failed to create loss plot")?;
        }
    }
    Ok(())
}

#[tracing::instrument(level = "trace")]
fn create_and_run(
    scenario_type: ScenarioType,
    number_of_sensors: Vec<i32>,
    measurement_covariances: Array1<f32>,
    trials: usize,
    base_id: &str,
    path: &Path,
) -> Result<()> {
    let mut join_handles = Vec::new();
    let mut scenarios = Vec::new();

    let lower_delay_samples = 4.1;
    let upper_delay_samples = 5.2;

    for measurement_covariance in &measurement_covariances {
        for number_of_sensors in &number_of_sensors {
            for trial in 0..trials {
                let id = format!("{base_id} - Num: {number_of_sensors:03}, Noise: {measurement_covariance:.3e}, trial: {trial:03}");
                let path = Path::new("results").join(&id);
                println!("Looking for scenario {path:?}");
                let scenario = if path.is_dir() {
                    println!("Found scenario. Loading it!");
                    let scenario = Scenario::load(path.as_path())?;
                    scenario
                } else {
                    println!("Didn't find scenario. Building it!");
                    let scenario = build_scenario(
                        upper_delay_samples,
                        lower_delay_samples,
                        *number_of_sensors,
                        *measurement_covariance,
                        scenario_type,
                        &id,
                    )?;
                    if RUN_IN_TESTS {
                        let send_scenario = scenario.clone();
                        let (epoch_tx, _) = channel();
                        let (summary_tx, _) = channel();
                        let handle =
                            thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
                        println!("handle {handle:?}");
                        join_handles.push(handle);
                    }
                    scenario
                };
                scenarios.push(scenario);
            }
        }
    }

    if RUN_IN_TESTS {
        for handle in join_handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("Failed to join worker thread"))??;
        }
        for scenario in &mut scenarios {
            let path = Path::new("results").join(scenario.id.clone());
            *scenario = Scenario::load(path.as_path())?;
        }
    }
    plot_results(
        path,
        base_id,
        &scenarios,
        number_of_sensors,
        measurement_covariances,
        trials,
        scenario_type,
    )?;
    Ok(())
}
