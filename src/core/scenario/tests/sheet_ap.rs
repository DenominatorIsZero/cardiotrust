use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
    sync::mpsc::channel,
    thread,
};

use anyhow::Result;
use approx::RelativeEq;
use ndarray::Array1;
use ndarray_npy::WriteNpyExt;

use super::RUN_IN_TESTS;
use crate::{
    core::{
        algorithm::{metrics::BatchWiseMetric, refinement::Optimizer},
        model::{functional::allpass::from_coef_to_samples, spatial::voxels::VoxelType},
        scenario::{run, tests::SAVE_NPY, Scenario},
    },
    tests::{clean_files, setup_folder},
    vis::plotting::png::line::{line_plot, log_y_plot},
};

const COMMON_PATH: &str = "tests/core/scenario/sheet_ap/";

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive scenario test"]
fn heavy_homogeneous_down() -> Result<()> {
    let base_id = "Sheet AP - Homogenous - Down - ";
    let path = Path::new(COMMON_PATH).join("homogeneous_down");

    let voxels_per_axis = vec![7];
    let learning_rates = Array1::<f32>::logspace(10.0, 4.0, 5.0, 5).to_vec();

    let initial_delay = 5.2;
    let target_delay = 4.1;

    create_and_run(
        initial_delay,
        target_delay,
        voxels_per_axis,
        learning_rates,
        base_id,
        &path,
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
    initial_delay: f32,
    target_delay: f32,
    voxels_per_axis: i32,
    learning_rate: f32,
    id: String,
) -> Result<Scenario> {
    let mut scenario = Scenario::build(Some(id));

    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

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
        .measurement_covariance_mean = 1e-12;
    // Adjust heart size
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .heart_size_mm = [
        voxel_size_mm * (voxels_per_axis) as f32,
        voxel_size_mm * (voxels_per_axis) as f32,
        voxel_size_mm,
    ];
    scenario.config.simulation.model.common.heart_offset_mm = [
        25.0 - (voxel_size_mm * (voxels_per_axis) as f32) / 2.0,
        -250.0 - (voxel_size_mm * (voxels_per_axis) as f32) / 2.0,
        180.0,
    ];
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
        .measurement_covariance_mean = 1e-12;
    // Adjust pathology
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_x_start_percentage = 0.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_x_stop_percentage = 1.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_y_start_percentage = 0.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_y_stop_percentage = 1.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .sa_x_center_percentage = 0.5;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .sa_y_center_percentage = 1.0;
    // Copy settings to algorithm model
    scenario.config.algorithm.model = scenario.config.simulation.model.clone();
    // Adjust propagation velocities

    let targe_delay_s = target_delay / sample_rate_hz;
    let target_velocity = voxel_size_mm / 1000.0 / targe_delay_s;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Sinoatrial)
        .ok_or_else(|| anyhow::anyhow!("Failed to get velocity for voxel type"))? = target_velocity;
    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .ok_or_else(|| anyhow::anyhow!("Failed to get velocity for voxel type"))? = target_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Sinoatrial)
        .ok_or_else(|| anyhow::anyhow!("Failed to get velocity for voxel type"))? =
        initial_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .ok_or_else(|| anyhow::anyhow!("Failed to get velocity for voxel type"))? =
        initial_velocity;
    // set optimization parameters
    scenario.config.algorithm.epochs = 500_000;
    scenario.config.algorithm.learning_rate = learning_rate;
    scenario.config.algorithm.optimizer = Optimizer::Sgd;
    scenario.config.algorithm.freeze_delays = false;
    scenario.config.algorithm.freeze_gains = true;
    let number_of_snapshots = 1000;
    scenario.config.algorithm.snapshots_interval =
        scenario.config.algorithm.epochs / number_of_snapshots;

    scenario.schedule()?;
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
    voxels_per_axis: Vec<i32>,
    learning_rates: Vec<f32>,
) -> Result<()> {
    setup_folder(path)?;
    for voxels_per_axis in &voxels_per_axis {
        let files = vec![
            path.join(format!("down_{voxels_per_axis:03}_loss.png")),
            path.join(format!("down_{voxels_per_axis:03}_delays.png")),
            path.join(format!("down_{voxels_per_axis:03}_delays_error.png")),
            path.join(format!("up{voxels_per_axis:03}_loss.png")),
            path.join(format!("up{voxels_per_axis:03}_delays.png")),
            path.join(format!("up{voxels_per_axis:03}_delays_error.png")),
        ];
        clean_files(&files)?;
    }

    let mut first_scenario = scenarios
        .first()
        .ok_or_else(|| anyhow::anyhow!("Expected at least one scenario"))?
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
            .unwrap()
            .model
            .as_ref()
            .unwrap()
            .functional_description
            .ap_params
            .coefs
    );
    let x_epochs = Array1::range(0.0, first_scenario.config.algorithm.epochs as f32, 1.0);
    let num_snapshots = first_scenario
        .results
        .as_ref()
        .unwrap()
        .snapshots
        .as_ref()
        .unwrap()
        .number_of_snapshots
        - 1;
    let x_snapshots = Array1::range(0.0, num_snapshots as f32, 1.0);

    for voxels_per_axis in voxels_per_axis {
        let mut min_loss_n = 0;
        let mut min_loss = 1e9;
        let mut losses_owned: Vec<BatchWiseMetric> = Vec::new();
        let mut labels_owned: Vec<String> = Vec::new();
        let mut delays_owned: Vec<Array1<f32>> = Vec::new();
        let mut delays_error_owned: Vec<Array1<f32>> = Vec::new();

        for (n, scenario) in scenarios.iter().enumerate() {
            if !scenario.id.contains(&format!("Num {voxels_per_axis},"))
                || !scenario
                    .summary
                    .as_ref()
                    .is_some_and(|s| s.loss.is_finite())
            {
                continue;
            }
            let mut scenario = (*scenario).clone();
            if let Some(summary) = scenario.summary.as_ref() {
                if summary.loss_mse < min_loss {
                    min_loss = summary.loss_mse;
                    min_loss_n = n;
                }
            }
            scenario.load_results()?;
            losses_owned.push(
                scenario
                    .results
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                    .metrics
                    .loss_mse_batch
                    .clone(),
            );
            labels_owned.push(format!(
                "l_r {:.2e}",
                scenario.config.algorithm.learning_rate,
            ));
            drop(scenario);
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
            fs::create_dir_all(&path)?;
            let writer = BufWriter::new(File::create(path.join("x_epochs.npy"))?);
            x_epochs.write_npy(writer)?;
            let writer = BufWriter::new(File::create(path.join("x_snapshots.npy"))?);
            x_snapshots.write_npy(writer)?;
            for (label, loss) in labels.iter().zip(losses.iter()) {
                let writer = BufWriter::new(File::create(
                    path.join(format!("loss - v_per_a {voxels_per_axis} {label}.npy")),
                )?);
                loss.write_npy(writer)?;
            }
        }
        println!("len of x_epochs: {}", x_epochs.len());
        println!("len of losses: {}", losses[0].len());
        log_y_plot(
            Some(&x_epochs),
            losses,
            Some(&path.join(format!("{voxels_per_axis:03}_loss.png"))),
            Some(format!("{base_title}Loss").as_str()),
            Some("Loss MSE"),
            Some("Epoch"),
            Some(&labels),
            None,
        )?;

        let mut scenario = scenarios[min_loss_n].clone();
        scenario.load_data()?;
        scenario.load_results()?;

        for index_x in 0..voxels_per_axis as usize {
            for index_y in 0..voxels_per_axis as usize {
                let voxel_index = scenario
                    .data
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                    .simulation
                    .model
                    .spatial_description
                    .voxels
                    .numbers[(index_x, index_y, 0)]
                    .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                    / 3;
                // iterate over all allpasses per voxel
                for offset_index in 0..scenario
                    .data
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                    .simulation
                    .model
                    .functional_description
                    .ap_params
                    .coefs
                    .shape()[1]
                {
                    // check if ap gain is not 0
                    let mut non_zero_found = false;
                    for input_dimension in 0..3 {
                        for output_dimension in 0..3 {
                            let gain = scenario
                                .data
                                .as_ref()
                                .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                                .simulation
                                .model
                                .functional_description
                                .ap_params
                                .gains[(
                                voxel_index * 3 + input_dimension,
                                offset_index * 3 + output_dimension,
                            )];
                            if gain.relative_ne(&0.0, 0.001, 0.001) {
                                non_zero_found = true;
                                break;
                            }
                        }
                    }
                    if !non_zero_found {
                        continue;
                    }
                    // add ap to plotting data
                    let mut delays = Array1::<f32>::zeros(num_snapshots);
                    let mut delays_error = Array1::<f32>::zeros(num_snapshots);
                    let target_delay = from_coef_to_samples(
                        scenario
                            .data
                            .as_ref()
                            .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                            .simulation
                            .model
                            .functional_description
                            .ap_params
                            .coefs[(voxel_index, offset_index)],
                    ) + scenario
                        .data
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                        .simulation
                        .model
                        .functional_description
                        .ap_params
                        .delays[(voxel_index, offset_index)]
                        as f32;

                    let snapshots = scenario
                        .results
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                        .snapshots
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Expected snapshots to be present"))?;
                    for i in 0..num_snapshots {
                        delays[i] = from_coef_to_samples(
                            snapshots.ap_coefs[(i, voxel_index, offset_index)],
                        ) + snapshots.ap_delays[(i, voxel_index, offset_index)] as f32;
                        delays_error[i] = target_delay - delays[i];
                    }
                    delays_owned.push(delays);
                    delays_error_owned.push(delays_error);
                }
            }
        }

        let delays = delays_owned.iter().collect::<Vec<&Array1<f32>>>();
        let delays_error = delays_error_owned.iter().collect::<Vec<&Array1<f32>>>();

        if SAVE_NPY {
            let path = path.join("npy");
            for (i, delay) in delays.iter().enumerate() {
                let writer = BufWriter::new(File::create(
                    path.join(format!("v_per_a {voxels_per_axis}, delay {i}.npy",)),
                )?);
                delay.write_npy(writer)?;
            }
        }

        if SAVE_NPY {
            let mut delay_error_mean = Array1::<f32>::zeros(delays[0].dim());
            for (i, delay_error_mean) in delay_error_mean.iter_mut().enumerate() {
                for error in &delays_error {
                    *delay_error_mean += error[i];
                }
                *delay_error_mean /= delays_error.len() as f32;
            }
            let path = path.join("npy");
            let writer = BufWriter::new(File::create(
                path.join(format!("v_per_a {voxels_per_axis}, delay_error_mean.npy",)),
            )?);
            delay_error_mean.write_npy(writer)?;

            let mut delay_error_mae = Array1::<f32>::zeros(delays[0].dim());
            for (i, delay_error_mae) in delay_error_mae.iter_mut().enumerate() {
                for error in &delays_error {
                    *delay_error_mae += error[i].abs();
                }
                *delay_error_mae /= delays_error.len() as f32;
            }

            let writer = BufWriter::new(File::create(
                path.join(format!("v_per_a {voxels_per_axis}, delay_error_mae.npy",)),
            )?);
            delay_error_mae.write_npy(writer)?;

            let mut delay_error_std = Array1::<f32>::zeros(delays[0].dim());
            for (i, delay_error_std) in delay_error_std.iter_mut().enumerate() {
                for error in &delays_error {
                    *delay_error_std += (error[i] - delay_error_mean[i]).powi(2);
                }
                *delay_error_std = (*delay_error_std / delays_error.len() as f32).sqrt();
            }

            let writer = BufWriter::new(File::create(
                path.join(format!("v_per_a {voxels_per_axis}, delay_error_std.npy",)),
            )?);
            delay_error_std.write_npy(writer)?;
        }

        line_plot(
            Some(&x_snapshots),
            delays,
            Some(&path.join(format!("{voxels_per_axis:03}_delays.png"))),
            Some(format!("{base_title}AP Delay").as_str()),
            Some("AP Delay (Estimated)"),
            Some("Snapshot"),
            None,
            None,
        )?;

        line_plot(
            Some(&x_snapshots),
            delays_error,
            Some(&path.join(format!("{voxels_per_axis:03}_delays_error.png"))),
            Some(format!("{base_title}AP Delay Error").as_str()),
            Some("AP Delay (Target - Estimated)"),
            Some("Snapshot"),
            None,
            None,
        )?;
    }
    Ok(())
}

#[tracing::instrument(level = "trace")]
fn create_and_run(
    initial_delay: f32,
    target_delay: f32,
    voxels_per_axis: Vec<i32>,
    learning_rates: Vec<f32>,
    base_id: &str,
    img_path: &Path,
) -> Result<()> {
    let mut join_handles = Vec::new();
    let mut scenarios = Vec::new();

    for voxels_per_axis in &voxels_per_axis {
        for learning_rate in &learning_rates {
            let id = format!("{base_id}Num {voxels_per_axis}, l_r {learning_rate:.2e}",);
            let path = Path::new("results").join(&id);
            println!("Looking for scenario {path:?}");
            let scenario = if path.is_dir() {
                println!("Found scenario. Loading it!");
                let scenario = Scenario::load(path.as_path())?;
                scenario
            } else {
                println!("Didn't find scenario. Building it!");
                let scenario = build_scenario(
                    initial_delay,
                    target_delay,
                    *voxels_per_axis,
                    *learning_rate,
                    id,
                )?;
                if RUN_IN_TESTS {
                    let send_scenario = scenario.clone();
                    let (epoch_tx, _) = channel();
                    let (summary_tx, _) = channel();
                    let handle = thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
                    println!("handle {handle:?}");
                    join_handles.push(handle);
                }
                scenario
            };
            scenarios.push(scenario);
        }
    }

    if RUN_IN_TESTS {
        for handle in join_handles {
            handle
                .join()
                .map_err(|e| anyhow::anyhow!("Thread panicked: {e:?}"))??;
        }
        for scenario in &mut scenarios {
            let path = Path::new("results").join(scenario.id.clone());
            *scenario = Scenario::load(path.as_path())?;
        }
    }

    plot_results(
        img_path,
        base_id,
        &scenarios,
        voxels_per_axis,
        learning_rates,
    )?;
    Ok(())
}
