use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
    sync::mpsc::channel,
    thread,
};

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

const COMMON_PATH: &str = "tests/core/scenario/line_ap/";

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive integration test"]
fn heavy_sgd() -> anyhow::Result<()> {
    let base_id = "Line AP";
    let path = Path::new(COMMON_PATH).join("sgd");

    let lower_delay = 4.1;
    let upper_delay = 5.2;

    let optimizer = Optimizer::Sgd;

    let number_of_aps = vec![3, 5, 10, 50];
    let learning_rates = Array1::<f32>::logspace(10.0, 3.0, 6.0, 4).to_vec();

    create_and_run(
        lower_delay,
        upper_delay,
        optimizer,
        number_of_aps,
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
#[test]
#[ignore = "expensive integration test"]
fn heavy_adam() -> anyhow::Result<()> {
    let base_id = "Line AP";
    let path = Path::new(COMMON_PATH).join("adam");

    let lower_delay = 4.1;
    let upper_delay = 5.2;

    let optimizer = Optimizer::Adam;

    let number_of_aps = vec![3, 5, 10, 50];

    let learning_rates = Array1::<f32>::logspace(10.0, 0.0, 2.0, 3).to_vec();

    create_and_run(
        lower_delay,
        upper_delay,
        optimizer,
        number_of_aps,
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
    target_delay_samples: f32,
    initial_delay_samples: f32,
    optimizer: Optimizer,
    learning_rate: f32,
    number_of_aps: i32,
    id: &str,
) -> anyhow::Result<Scenario> {
    let mut scenario = Scenario::build(Some(id.to_string()));

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
        voxel_size_mm,
        voxel_size_mm * (number_of_aps + 1) as f32,
        voxel_size_mm,
    ];
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
        .sa_y_center_percentage = 1.0;
    scenario.config.simulation.model.common.heart_offset_mm = [
        25.0,
        -250.0 - (voxel_size_mm * (number_of_aps + 1) as f32) / 2.0,
        180.0,
    ];
    // Copy settings to algorithm model
    scenario.config.algorithm.model = scenario.config.simulation.model.clone();
    // Adjust propagation velocities
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
        .ok_or_else(|| anyhow::anyhow!("Failed to get velocity for voxel type"))? = initial_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .ok_or_else(|| anyhow::anyhow!("Failed to get velocity for voxel type"))? = initial_velocity;
    // set optimization parameters
    scenario.config.algorithm.epochs = 20_000;
    scenario.config.algorithm.learning_rate = learning_rate;
    scenario.config.algorithm.optimizer = optimizer;
    scenario.config.algorithm.freeze_delays = false;
    scenario.config.algorithm.freeze_gains = true;
    scenario.config.algorithm.difference_regularization_strength = 0.0;
    scenario.config.algorithm.slow_down_stregth = 0.0;
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
    number_of_aps: Vec<i32>,
    learning_rates: Vec<f32>,
) -> anyhow::Result<()> {
    setup_folder(path);
    for number_of_ap in &number_of_aps {
        let files = vec![
            path.join(format!("down_{number_of_ap:03}_loss.png")),
            path.join(format!("down_{number_of_ap:03}_delays.png")),
            path.join(format!("down_{number_of_ap:03}_delays_error.png")),
            path.join(format!("up{number_of_ap:03}_loss.png")),
            path.join(format!("up{number_of_ap:03}_delays.png")),
            path.join(format!("up{number_of_ap:03}_delays_error.png")),
        ];
        clean_files(&files);
    }

    let mut first_scenario = scenarios.first()
        .ok_or_else(|| anyhow::anyhow!("Expected at least one scenario"))?.clone();
    println!("Loading data for first scenario");
    first_scenario.load_data();
    println!("Loading results for first scenario {:?}", first_scenario.id);
    first_scenario.load_results();

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
        .number_of_snapshots;
    let x_snapshots = Array1::range(0.0, num_snapshots as f32, 1.0);

    for number_of_ap in number_of_aps {
        for key in ["Down", "Up"] {
            let mut losses_owned: Vec<BatchWiseMetric> = Vec::new();
            let mut labels_owned: Vec<String> = Vec::new();

            for scenario in scenarios {
                if !scenario.id.contains(&format!("Num: {number_of_ap},"))
                    || !scenario.id.contains(key)
                    || !scenario.summary.as_ref().unwrap().loss.is_finite()
                {
                    continue;
                }
                let mut delays_owned: Vec<Array1<f32>> = Vec::new();
                let mut delays_error_owned: Vec<Array1<f32>> = Vec::new();
                let mut scenario = (*scenario).clone();
                scenario.load_results();
                scenario.load_data();
                losses_owned.push(
                    scenario
                        .results
                        .as_ref()
                        .unwrap()
                        .metrics
                        .loss_mse_batch
                        .clone(),
                );
                labels_owned.push(format!(
                    "l_r {:.2e}",
                    scenario.config.algorithm.learning_rate,
                ));
                let lr = format!("{:.2e}", scenario.config.algorithm.learning_rate,);

                for ap in 0..number_of_ap as usize {
                    let mut delays = Array1::<f32>::zeros(num_snapshots);
                    let mut delays_error = Array1::<f32>::zeros(num_snapshots);
                    let target_delay = from_coef_to_samples(
                        scenario
                            .data
                            .as_ref()
                            .unwrap()
                            .simulation
                            .model
                            .functional_description
                            .ap_params
                            .coefs[(ap, 15)],
                    ) + scenario
                        .data
                        .as_ref()
                        .unwrap()
                        .simulation
                        .model
                        .functional_description
                        .ap_params
                        .delays[(ap, 15)] as f32;

                    let snapshots = scenario
                        .results
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Expected test data to be present"))?
                        .snapshots
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Expected snapshots to be present"))?;
                    for i in 0..num_snapshots {
                        delays[i] = from_coef_to_samples(snapshots.ap_coefs[(i, ap, 15)])
                            + snapshots.ap_delays[(i, ap, 15)] as f32;
                        delays_error[i] = target_delay - delays[i];
                    }
                    delays_owned.push(delays);
                    delays_error_owned.push(delays_error);
                }

                let delays = delays_owned.iter().collect::<Vec<&Array1<f32>>>();
                let delays_error = delays_error_owned.iter().collect::<Vec<&Array1<f32>>>();

                if SAVE_NPY {
                    let path = path.join("npy");
                    fs::create_dir_all(&path).unwrap();
                    for (i, delay) in delays.iter().enumerate() {
                        let writer = BufWriter::new(
                            File::create(path.join(format!(
                                "num_ap: {number_of_ap}, delay: {i}, lr: {lr}.npy",
                            )))
                            .unwrap(),
                        );
                        delay.write_npy(writer).unwrap();
                    }
                }

                line_plot(
                    Some(&x_snapshots),
                    delays,
                    Some(&path.join(format!("{key}_{number_of_ap:03}_lr_{lr}_delays.png"))),
                    Some(format!("{base_title} - AP Delay - lr: {lr}").as_str()),
                    Some("AP Delay (Estimated)"),
                    Some("Snapshot"),
                    None,
                    None,
                )
                ?;

                line_plot(
                    Some(&x_snapshots),
                    delays_error,
                    Some(&path.join(format!("{key}_{number_of_ap:03}_lr_{lr}_delays_error.png"))),
                    Some(format!("{base_title} - AP Delay Error - lr: {lr}").as_str()),
                    Some("AP Delay (Target - Estimated)"),
                    Some("Snapshot"),
                    None,
                    None,
                )
                ?;
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
                fs::create_dir_all(&path).unwrap();
                let writer = BufWriter::new(File::create(path.join("x_epochs.npy")).unwrap());
                x_epochs.write_npy(writer).unwrap();
                let writer = BufWriter::new(File::create(path.join("x_snapshots.npy")).unwrap());
                x_snapshots.write_npy(writer).unwrap();
                for (label, loss) in labels.iter().zip(losses.iter()) {
                    let writer = BufWriter::new(
                        File::create(
                            path.join(format!("loss - num_ap: {number_of_ap} {label}.npy")),
                        )
                        .unwrap(),
                    );
                    loss.write_npy(writer).unwrap();
                }
            }

            log_y_plot(
                Some(&x_epochs),
                losses,
                Some(&path.join(format!("{key}_{number_of_ap:03}_loss.png"))),
                Some(format!("{base_title} - Loss").as_str()),
                Some("Loss MSE"),
                Some("Epoch"),
                Some(&labels),
                None,
            )?;
        }
    }
    Ok(())
}

#[tracing::instrument(level = "trace")]
fn create_and_run(
    lower_delay_samples: f32,
    upper_delay_samples: f32,
    optimizer: Optimizer,
    number_of_aps: Vec<i32>,
    learning_rates: Vec<f32>,
    base_id: &str,
    path: &Path,
) -> anyhow::Result<()> {
    let mut join_handles = Vec::new();
    let mut scenarios = Vec::new();

    // Up
    for number_of_ap in &number_of_aps {
        for learning_rate in &learning_rates {
            let id = format!(
                "{base_id} - {optimizer} - Up - Num: {number_of_ap}, l_r: {learning_rate:.2e}",
            );
            let path = Path::new("results").join(&id);
            println!("Looking for scenario {path:?}");
            let scenario = if path.is_dir() {
                println!("Found scenario. Loading it!");
                Scenario::load(path.as_path())
            } else {
                println!("Didn't find scenario. Building it!");
                let scenario = build_scenario(
                    upper_delay_samples,
                    lower_delay_samples,
                    optimizer,
                    *learning_rate,
                    *number_of_ap,
                    &id,
                )?;
                if RUN_IN_TESTS {
                    let send_scenario = scenario.clone();
                    let (epoch_tx, _) = channel();
                    let (summary_tx, _) = channel();
                    let handle = thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
                    println!("handle {handle:?}");
                    join_handles.push(handle);
                }
                Ok(scenario)
            };
            scenarios.push(scenario?);
        }
    }

    // Down
    for number_of_ap in &number_of_aps {
        for learning_rate in &learning_rates {
            let id = format!(
                "{base_id} - {optimizer} - Down - Num: {number_of_ap}, l_r: {learning_rate:.2e}",
            );
            let path = Path::new("results").join(&id);
            println!("Looking for scenario {path:?}");
            let scenario = if path.is_dir() {
                println!("Found scenario. Loading it!");
                Scenario::load(path.as_path())
            } else {
                println!("Didn't find scenario. Building it!");
                let scenario = build_scenario(
                    lower_delay_samples,
                    upper_delay_samples,
                    optimizer,
                    *learning_rate,
                    *number_of_ap,
                    &id,
                )?;
                if RUN_IN_TESTS {
                    let send_scenario = scenario.clone();
                    let (epoch_tx, _) = channel();
                    let (summary_tx, _) = channel();
                    let handle = thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
                    println!("handle {handle:?}");
                    join_handles.push(handle);
                }
                Ok(scenario)
            };
            scenarios.push(scenario?);
        }
    }

    if RUN_IN_TESTS {
        for handle in join_handles {
            handle.join().unwrap();
        }
        for scenario in &mut scenarios {
            let path = Path::new("results").join(scenario.id.clone());
            *scenario = Scenario::load(path.as_path())?;
        }
    }
    plot_results(path, base_id, &scenarios, number_of_aps, learning_rates)?;
    Ok(())
}
