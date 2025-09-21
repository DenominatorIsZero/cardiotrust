use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
    sync::mpsc::channel,
    thread,
};

use anyhow::{Context, Result};

use nalgebra::ComplexField;
use ndarray::{s, Array1};
use ndarray_npy::WriteNpyExt;

use super::{RUN_IN_TESTS, SAVE_NPY};
use crate::{
    core::{
        algorithm::{metrics::BatchWiseMetric, refinement::Optimizer},
        config::model::ControlFunction,
        model::{functional::allpass::from_coef_to_samples, spatial::voxels::VoxelType},
        scenario::{run, Scenario},
    },
    tests::{clean_files, setup_folder},
    vis::plotting::png::line::{line_plot, log_y_plot},
};

const COMMON_PATH: &str = "tests/core/scenario/single_ap/";

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive integration test"]
fn heavy_no_roll_up() -> Result<()> {
    let base_id = "Single AP - No Roll - up - ".to_string();
    let path = Path::new(COMMON_PATH).join("no_roll_up");

    let integer_part = 5.0;
    let fractional_step = 0.1;
    let steps = (1.0 / fractional_step).ceil() as usize;

    let initial_fractional = 0.01;
    let initial_delay = integer_part + initial_fractional;

    let mut target_delays = Vec::new();

    for target_fractional_part in 1..=steps {
        let target_fractional = (target_fractional_part as f32 * fractional_step).clamp(0.01, 0.99);
        let target_delay = integer_part + target_fractional;
        target_delays.push(target_delay);
    }

    create_and_run(target_delays, initial_delay, &base_id, &path)?;
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
fn heavy_no_roll_down() -> Result<()> {
    let base_id = "Single AP - No Roll - down - ".to_string();
    let path = Path::new(COMMON_PATH).join("no_roll_down");

    let integer_part = 5.0;
    let fractional_step = 0.1;
    let steps = (1.0 / fractional_step).ceil() as usize;

    let initial_fractional = 0.99;
    let initial_delay = integer_part + initial_fractional;

    let mut target_delays = Vec::new();
    for target_fractional_part in 0..steps {
        let target_fractional = (target_fractional_part as f32 * fractional_step).clamp(0.01, 0.99);
        let target_delay = integer_part + target_fractional;
        target_delays.push(target_delay);
    }

    create_and_run(target_delays, initial_delay, &base_id, &path)?;
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
fn heavy_yes_roll_up() -> Result<()> {
    let base_id = "Single AP - Yes Roll - Up - ".to_string();
    let path = Path::new(COMMON_PATH).join("yes_roll_up");

    let integer_part = 10.0;
    let fractional_part = 0.9;
    let steps = 10;

    let initial_delay = integer_part + fractional_part;

    let mut target_delays = Vec::new();

    for target_interger_part in 1..=steps {
        let target_delay = integer_part + target_interger_part as f32 + fractional_part;
        target_delays.push(target_delay);
    }
    create_and_run(target_delays, initial_delay, &base_id, &path)?;
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
fn heavy_yes_roll_down() -> Result<()> {
    let base_id = "Single AP - Yes Roll - Down - ".to_string();
    let path = Path::new(COMMON_PATH).join("yes_roll_down");

    let integer_part = 11.0;
    let fractional_part = 0.5;
    let steps = 10;

    let initial_delay = integer_part + fractional_part;

    let mut target_delays = Vec::new();

    for target_interger_part in 1..=steps {
        let target_delay = integer_part - target_interger_part as f32 + fractional_part;
        target_delays.push(target_delay);
    }

    create_and_run(target_delays, initial_delay, &base_id, &path)?;
    Ok(())
}

#[tracing::instrument(level = "trace")]
fn build_scenario(target_velocity: f32, initial_velocity: f32, id: &str) -> Result<Scenario> {
    let mut scenario = Scenario::build(Some(id.to_string()));
    // configure control function
    scenario.config.simulation.model.common.control_function = ControlFunction::Ohara;
    // configure sensors
    scenario
        .config
        .simulation
        .model
        .common
        .sensor_array_origin_mm = [
        scenario.config.simulation.model.common.heart_offset_mm[0]
            + scenario
                .config
                .simulation
                .model
                .handcrafted
                .as_ref()
                .context("Handcrafted model configuration should be available for sensor positioning")?
                .heart_size_mm[0]
                / 2.0,
        scenario.config.simulation.model.common.heart_offset_mm[1]
            + scenario
                .config
                .simulation
                .model
                .handcrafted
                .as_ref()
                .context("Handcrafted model configuration should be available for sensor positioning")?
                .heart_size_mm[1]
                / 2.0,
        scenario
            .config
            .simulation
            .model
            .common
            .sensor_array_origin_mm[2],
    ];

    // Set pathological true
    scenario.config.simulation.model.common.pathological = true;
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
        .context("Handcrafted model configuration should be available for heart size adjustment")?
        .heart_size_mm = [2.5, 5.0, 2.5];
    // Adjust pathology
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model configuration should be available for pathology x start adjustment")?
        .pathology_x_start_percentage = 0.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model configuration should be available for pathology x stop adjustment")?
        .pathology_x_stop_percentage = 1.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model configuration should be available for pathology y start adjustment")?
        .pathology_y_start_percentage = 0.0;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .context("Handcrafted model configuration should be available for pathology y stop adjustment")?
        .pathology_y_stop_percentage = 0.4;
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
        .context("Sinoatrial voxel type should exist in simulation propagation velocities")? = target_velocity;
    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .context("Pathological voxel type should exist in simulation propagation velocities")? = target_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Sinoatrial)
        .context("Sinoatrial voxel type should exist in algorithm propagation velocities")? = initial_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .context("Pathological voxel type should exist in algorithm propagation velocities")? = initial_velocity;
    // set optimization parameters
    scenario.config.algorithm.epochs = 10_000;
    scenario.config.algorithm.learning_rate = 1e5;
    scenario.config.algorithm.optimizer = Optimizer::Sgd;
    scenario.config.algorithm.freeze_delays = false;
    scenario.config.algorithm.freeze_gains = true;
    let number_of_snapshots = 1000;
    scenario.config.algorithm.snapshots_interval =
        scenario.config.algorithm.epochs / number_of_snapshots;

    scenario.schedule().context("Failed to schedule scenario for single AP test")?;
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
fn plot_results(path: &Path, base_title: &str, scenarios: Vec<Scenario>) -> Result<()> {
    setup_folder(path);
    let files = vec![
        path.join("loss.png"),
        path.join("loss_close_up.png"),
        path.join("params.png"),
        path.join("params_error.png"),
        path.join("delays.png"),
        path.join("delays_close_up.png"),
        path.join("delays_error.png"),
    ];
    clean_files(&files);

    let first_scenario = scenarios.first().context("At least one scenario should be provided for plotting")?;
    println!(
        "{:?}",
        first_scenario
            .results
            .as_ref()
            .context("First scenario should have results available for plotting")?
            .model
            .as_ref()
            .context("First scenario should have model results available for plotting")?
            .functional_description
            .ap_params
            .coefs
    );

    let x_epochs = Array1::range(0.0, first_scenario.config.algorithm.epochs as f32, 1.0);
    let num_snapshots = first_scenario
        .results
        .as_ref()
        .context("First scenario should have results available for snapshot count")?
        .snapshots
        .as_ref()
        .context("First scenario should have snapshots available for plotting")?
        .number_of_snapshots;
    let x_snapshots = Array1::range(0.0, num_snapshots as f32, 1.0);
    let mut losses_owned: Vec<BatchWiseMetric> = Vec::new();
    let mut params_owned: Vec<Array1<f32>> = Vec::new();
    let mut params_error_owned: Vec<Array1<f32>> = Vec::new();
    let mut delays_owned: Vec<Array1<f32>> = Vec::new();
    let mut delays_error_owned: Vec<Array1<f32>> = Vec::new();
    let mut labels_owned: Vec<String> = Vec::new();
    let mut gt_states = Vec::new();
    let mut gt_measurements = Vec::new();

    let initial_states = scenarios
        .first()
        .context("At least one scenario should be provided for initial states")?
        .results
        .as_ref()
        .context("First scenario should have results available for initial states")?
        .estimations
        .system_states
        .clone();
    let initial_measurements = scenarios
        .first()
        .context("At least one scenario should be provided for initial measurements")?
        .results
        .as_ref()
        .context("First scenario should have results available for initial measurements")?
        .estimations
        .measurements
        .clone();

    println!("{}", scenarios.len());
    for scenario in scenarios {
        losses_owned.push(
            scenario
                .results
                .as_ref()
                .context("Scenario should have results available for loss plotting")?
                .metrics
                .loss_mse_batch
                .clone(),
        );
        let mut ap_param = Array1::<f32>::zeros(num_snapshots);
        let mut delays = Array1::<f32>::zeros(num_snapshots);
        let mut delays_error = Array1::<f32>::zeros(num_snapshots);
        let mut ap_param_error = Array1::<f32>::zeros(num_snapshots);

        gt_states.push(
            scenario
                .data
                .as_ref()
                .context("Scenario should have data available for ground truth states")?
                .simulation
                .system_states
                .clone(),
        );
        gt_measurements.push(
            scenario
                .data
                .as_ref()
                .context("Scenario should have data available for ground truth measurements")?
                .simulation
                .measurements
                .clone(),
        );

        let target_param = scenario
            .data
            .as_ref()
            .context("Scenario should have data available for target parameter calculation")?
            .simulation
            .model
            .functional_description
            .ap_params
            .coefs[(0, 15)];
        let target_delay = from_coef_to_samples(
            scenario
                .data
                .as_ref()
                .context("Scenario should have data available for target delay calculation")?
                .simulation
                .model
                .functional_description
                .ap_params
                .coefs[(0, 15)],
        ) + scenario
            .data
            .as_ref()
            .context("Scenario should have data available for target delay offset calculation")?
            .simulation
            .model
            .functional_description
            .ap_params
            .delays[(0, 15)] as f32;

        let snapshots = scenario
            .results
            .as_ref()
            .context("Scenario should have results available for snapshots processing")?
            .snapshots
            .as_ref()
            .context("Scenario should have snapshots available for parameter analysis")?;
        for i in 0..num_snapshots {
            delays[i] = from_coef_to_samples(snapshots.ap_coefs[(i, 0, 15)])
                + snapshots.ap_delays[(i, 0, 15)] as f32;
            delays_error[i] = target_delay - delays[i];
            ap_param[i] = snapshots.ap_coefs[(i, 0, 15)];
            ap_param_error[i] = target_param - ap_param[i];
        }
        params_owned.push(ap_param);
        params_error_owned.push(ap_param_error);
        delays_owned.push(delays);
        delays_error_owned.push(delays_error);
        labels_owned.push(format!(
            "{:.2}",
            scenario
                .data
                .as_ref()
                .context("Scenario should have data available for label generation")?
                .simulation
                .model
                .functional_description
                .ap_params
                .initial_delays[(0, 15)]
        ));
    }

    let losses = losses_owned
        .iter()
        .map(std::ops::Deref::deref)
        .collect::<Vec<&Array1<f32>>>();
    let params = params_owned.iter().collect::<Vec<&Array1<f32>>>();
    let params_error = params_error_owned.iter().collect::<Vec<&Array1<f32>>>();
    let delays = delays_owned.iter().collect::<Vec<&Array1<f32>>>();
    let delays_error = delays_error_owned.iter().collect::<Vec<&Array1<f32>>>();
    let labels: Vec<&str> = labels_owned
        .iter()
        .map(std::string::String::as_str)
        .collect();

    if SAVE_NPY {
        let npy_path = path.join("npy");
        fs::create_dir_all(&npy_path).context("Failed to create npy directory")?;
        let writer = BufWriter::new(File::create(npy_path.join("x_epochs.npy")).context("Failed to create x_epochs.npy file")?);
        x_epochs.write_npy(writer).context("Failed to write x_epochs.npy file")?;
        let writer = BufWriter::new(File::create(npy_path.join("x_snapshots.npy")).context("Failed to create x_snapshots.npy file")?);
        x_snapshots.write_npy(writer).context("Failed to write x_snapshots.npy file")?;
        let writer = BufWriter::new(File::create(npy_path.join("initial_states.npy")).context("Failed to create initial_states.npy file")?);
        initial_states.write_npy(writer).context("Failed to write initial_states.npy file")?;
        let writer = BufWriter::new(File::create(npy_path.join("initial_measurements.npy")).context("Failed to create initial_measurements.npy file")?);
        initial_measurements.write_npy(writer).context("Failed to write initial_measurements.npy file")?;
        for (n, label) in labels.iter().enumerate() {
            let writer =
                BufWriter::new(File::create(npy_path.join(format!("loss_{label}.npy"))).context("Failed to create loss npy file")?);
            losses[n].write_npy(writer).context("Failed to write loss npy file")?;
            let writer =
                BufWriter::new(File::create(npy_path.join(format!("param_{label}.npy"))).context("Failed to create param npy file")?);
            params[n].write_npy(writer).context("Failed to write param npy file")?;
            let writer = BufWriter::new(
                File::create(npy_path.join(format!("param_error_{label}.npy"))).context("Failed to create param_error npy file")?,
            );
            params_error[n].write_npy(writer).context("Failed to write param_error npy file")?;
            let writer =
                BufWriter::new(File::create(npy_path.join(format!("delay_{label}.npy"))).context("Failed to create delay npy file")?);
            delays[n].write_npy(writer).context("Failed to write delay npy file")?;
            let writer = BufWriter::new(
                File::create(npy_path.join(format!("delay_error_{label}.npy"))).context("Failed to create delay_error npy file")?,
            );
            delays_error[n].write_npy(writer).context("Failed to write delay_error npy file")?;
            let writer =
                BufWriter::new(File::create(npy_path.join(format!("gt_states_{label}.npy"))).context("Failed to create gt_states npy file")?);
            gt_states[n].write_npy(writer).context("Failed to write gt_states npy file")?;
            let writer = BufWriter::new(
                File::create(npy_path.join(format!("gt_measurements_{label}.npy"))).context("Failed to create gt_measurements npy file")?,
            );
            gt_measurements[n].write_npy(writer).context("Failed to write gt_measurements npy file")?;
        }
    }

    println!("ys length: {}", losses.len());

    log_y_plot(
        Some(&x_epochs),
        losses,
        Some(files[0].as_path()),
        Some(format!("{base_title} - Loss").as_str()),
        Some("Loss MSE"),
        Some("Epoch"),
        Some(&labels),
        None,
    )
    .context("Failed to create loss plot")?;

    let x_epochs_close_up = x_epochs.slice(s![..10]).to_owned();
    let losses_owned_close_up = losses_owned
        .iter()
        .map(|v| v.slice(s![..10]).to_owned())
        .collect::<Vec<Array1<f32>>>();
    let losses_close_up = losses_owned_close_up.iter().collect::<Vec<&Array1<f32>>>();

    log_y_plot(
        Some(&x_epochs_close_up),
        losses_close_up,
        Some(files[1].as_path()),
        Some(format!("{base_title} - Loss").as_str()),
        Some("Loss MSE"),
        Some("Epoch"),
        Some(&labels),
        None,
    )
    .context("Failed to create loss close-up plot")?;

    line_plot(
        Some(&x_snapshots),
        params,
        Some(files[2].as_path()),
        Some(format!("{base_title} - AP Coef").as_str()),
        Some("AP Coef (Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .context("Failed to create AP coefficient plot")?;

    line_plot(
        Some(&x_snapshots),
        params_error,
        Some(files[3].as_path()),
        Some(format!("{base_title} - AP Coef Error").as_str()),
        Some("AP Coef (Target - Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .context("Failed to create AP coefficient error plot")?;

    line_plot(
        Some(&x_snapshots),
        delays,
        Some(files[4].as_path()),
        Some(format!("{base_title} - AP Delay").as_str()),
        Some("AP Delay (Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .context("Failed to create AP delay plot")?;

    let x_snapshots_close_up = x_snapshots.slice(s![..10]).to_owned();
    let delays_owned_close_up = delays_owned
        .iter()
        .map(|v| v.slice(s![..10]).to_owned())
        .collect::<Vec<Array1<f32>>>();
    let delays_close_up = delays_owned_close_up.iter().collect::<Vec<&Array1<f32>>>();

    line_plot(
        Some(&x_snapshots_close_up),
        delays_close_up,
        Some(files[5].as_path()),
        Some(format!("{base_title} - AP Delay").as_str()),
        Some("AP Delay (Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .context("Failed to create AP delay close-up plot")?;

    line_plot(
        Some(&x_snapshots),
        delays_error,
        Some(files[6].as_path()),
        Some(format!("{base_title} - AP Delay Error").as_str()),
        Some("AP Delay (Target - Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .context("Failed to create AP delay error plot")?;

    Ok(())
}

#[tracing::instrument(level = "trace")]
fn create_and_run(target_delays: Vec<f32>, initial_delay: f32, base_id: &str, path: &Path) -> Result<()> {
    let mut scenarios = Vec::new();
    let mut join_handles = Vec::new();

    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    for target_delay in target_delays {
        let target_delay_seconds = target_delay / sample_rate_hz;
        let target_velocity = voxel_size_mm / 1000.0 / target_delay_seconds;
        let id = format!("{base_id} {initial_delay:.2} [samples] to {target_delay:.2} [samples]");
        let path = Path::new("results").join(&id);
        if path.is_dir() {
            println!("Found scenario. Loading it!");
            let mut scenario = Scenario::load(path.as_path())?;
            scenario.load_data()?;
            scenario.load_results()?;
            scenarios.push(scenario);
        } else {
            println!("Didn't find scenario. Building it!");
            let scenario = build_scenario(target_velocity, initial_velocity, &id)?;
            if RUN_IN_TESTS {
                let send_scenario = scenario.clone();
                let (epoch_tx, _) = channel();
                let (summary_tx, _) = channel();
                let handle = thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
                println!("handle {handle:?}");
                join_handles.push(handle);
            }
            scenarios.push(scenario);
        }
    }

    if RUN_IN_TESTS {
        for handle in join_handles {
            handle.join().map_err(|_| anyhow::anyhow!("Failed to join thread for scenario execution"))??;
        }
        for scenario in &mut scenarios {
            let path = Path::new("results").join(scenario.id.clone());
            *scenario = Scenario::load(path.as_path())?;
            scenario.load_data()?;
            scenario.load_results()?;
        }
    }

    plot_results(path, base_id, scenarios)?;
    Ok(())
}
