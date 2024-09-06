use ndarray_npy::WriteNpyExt;
use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
    ops::{Deref, DerefMut},
};
use std::{path::Path, sync::mpsc::channel, thread};

use nalgebra::ComplexField;
use ndarray::{Array1, Array2};

use super::{RUN_IN_TESTS, SAVE_NPY};

use crate::{
    core::{
        algorithm::metrics::BatchWiseMetric,
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
fn no_roll_down() {
    let base_id = "Single AP - No Roll - Down - ".to_string();
    let base_title = "Single AP - No Roll - Down";
    let path = Path::new(COMMON_PATH).join("no_roll_down");

    let integer_part = 5.0;
    let fractional_step = 0.1;
    let steps = (1.0 / fractional_step).ceil() as usize;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_fractional = 0.0;
    let initial_delay = integer_part + initial_fractional;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let mut target_velocities = Vec::new();

    for target_fractional_part in 1..=steps {
        let target_fractional = (target_fractional_part as f32 * fractional_step).max(0.01);
        let target_delay = integer_part + target_fractional;
        let target_delay_s = target_delay / sample_rate_hz;
        let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;
        target_velocities.push(target_velocity);
    }

    create_and_run(
        target_velocities,
        initial_velocity,
        &base_id,
        &path,
        base_title,
    );
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
fn no_roll_up() {
    let base_id = "Single AP - No Roll - Up - ".to_string();
    let base_title = "Single AP - No Roll - Up";
    let path = Path::new(COMMON_PATH).join("no_roll_up");

    let integer_part = 5.0;
    let fractional_step = 0.1;
    let steps = (1.0 / fractional_step).ceil() as usize;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_fractional = 0.99;
    let initial_delay = integer_part + initial_fractional;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let mut target_velocities = Vec::new();
    for target_fractional_part in 0..steps {
        let target_fractional = (target_fractional_part as f32 * fractional_step).min(0.99);
        let target_delay = integer_part + target_fractional;
        let target_delay_s = target_delay / sample_rate_hz;
        let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;
        target_velocities.push(target_velocity);
    }

    create_and_run(
        target_velocities,
        initial_velocity,
        &base_id,
        &path,
        base_title,
    );
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
fn yes_roll_down() {
    let base_id = "Single AP - Yes Roll - Down - ".to_string();
    let base_title = "Single AP - Yes Roll - Up";
    let path = Path::new(COMMON_PATH).join("yes_roll_down");

    let integer_part = 2.0;
    let fractional_part = 0.2;
    let steps = 10;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_delay = integer_part + fractional_part;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let mut target_velocities = Vec::new();

    for target_interger_part in 1..=steps {
        let target_delay = integer_part + target_interger_part as f32 + fractional_part;
        let target_delay_s = target_delay / sample_rate_hz;
        let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;
        target_velocities.push(target_velocity);
    }

    create_and_run(
        target_velocities,
        initial_velocity,
        &base_id,
        &path,
        base_title,
    );
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
fn yes_roll_up() {
    let base_id = "Single AP - Yes Roll - Up - ".to_string();
    let base_title = "Single AP - Yes Roll - Up";
    let path = Path::new(COMMON_PATH).join("yes_roll_up");

    let integer_part = 11.0;
    let fractional_part = 0.5;
    let steps = 10;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_delay = integer_part + fractional_part;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let mut target_velocities = Vec::new();

    for target_interger_part in 1..=steps {
        let target_delay = integer_part - target_interger_part as f32 + fractional_part;
        let target_delay_s = target_delay / sample_rate_hz;
        let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;
        target_velocities.push(target_velocity);
    }

    create_and_run(
        target_velocities,
        initial_velocity,
        &base_id,
        &path,
        base_title,
    );
}

fn build_scenario(target_velocity: f32, initial_velocity: f32, base_id: &str) -> Scenario {
    let mut scenario = Scenario::build(Some(format!(
        "{base_id} {initial_velocity:.2} [m per s] to {target_velocity:.2} [m per s]"
    )));

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
        .unwrap()
        .heart_size_mm = [2.5, 5.0, 2.5];
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
        .unwrap() = target_velocity;
    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .unwrap() = target_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Sinoatrial)
        .unwrap() = initial_velocity;
    *scenario
        .config
        .algorithm
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .unwrap() = initial_velocity;
    // set optimization parameters
    scenario.config.algorithm.epochs = 5_000;
    scenario.config.algorithm.learning_rate = 1e4;
    scenario.config.algorithm.freeze_delays = false;
    scenario.config.algorithm.freeze_gains = true;
    let number_of_snapshots = 1000;
    scenario.config.algorithm.snapshots_interval =
        scenario.config.algorithm.epochs / number_of_snapshots;

    scenario.schedule().unwrap();
    let _ = scenario.save();
    scenario
}

#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn plot_results(path: &Path, base_title: &str, scenarios: Vec<Scenario>) {
    setup_folder(path);
    let files = vec![
        path.join("loss.png"),
        path.join("params.png"),
        path.join("params_error.png"),
        path.join("delays.png"),
        path.join("delays_error.png"),
    ];
    clean_files(&files);

    let first_scenario = scenarios.first().unwrap();
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
    let num_snapshots = first_scenario.results.as_ref().unwrap().snapshots.len() as f32;
    let x_snapshots = Array1::range(0.0, num_snapshots, 1.0);
    let mut losses_owned: Vec<BatchWiseMetric> = Vec::new();
    let mut params_owned: Vec<Array1<f32>> = Vec::new();
    let mut params_error_owned: Vec<Array1<f32>> = Vec::new();
    let mut delays_owned: Vec<Array1<f32>> = Vec::new();
    let mut delays_error_owned: Vec<Array1<f32>> = Vec::new();
    let mut labels_owned: Vec<String> = Vec::new();

    println!("{}", scenarios.len());
    for scenario in scenarios {
        losses_owned.push(
            scenario
                .results
                .as_ref()
                .unwrap()
                .metrics
                .loss_mse_batch
                .clone(),
        );
        let mut ap_param = Array1::<f32>::zeros(num_snapshots as usize);
        let mut delays = Array1::<f32>::zeros(num_snapshots as usize);
        let mut delays_error = Array1::<f32>::zeros(num_snapshots as usize);
        let mut ap_param_error = Array1::<f32>::zeros(num_snapshots as usize);

        let target_param = scenario
            .data
            .as_ref()
            .unwrap()
            .simulation
            .model
            .functional_description
            .ap_params
            .coefs[(0, 15)];
        let target_delay = from_coef_to_samples(
            scenario
                .data
                .as_ref()
                .unwrap()
                .simulation
                .model
                .functional_description
                .ap_params
                .coefs[(0, 15)],
        ) + scenario
            .data
            .as_ref()
            .unwrap()
            .simulation
            .model
            .functional_description
            .ap_params
            .delays[(0, 15)] as f32;

        for (i, snapshot) in scenario
            .results
            .as_ref()
            .unwrap()
            .snapshots
            .iter()
            .enumerate()
        {
            delays[i] =
                from_coef_to_samples(snapshot.functional_description.ap_params.coefs[(0, 15)])
                    + snapshot.functional_description.ap_params.delays[(0, 15)] as f32;
            delays_error[i] = target_delay - delays[i];
            ap_param[i] = snapshot.functional_description.ap_params.coefs[(0, 15)];
            ap_param_error[i] = target_param - ap_param[i];
        }
        params_owned.push(ap_param);
        params_error_owned.push(ap_param_error);
        delays_owned.push(delays);
        delays_error_owned.push(delays_error);
        labels_owned.push(format!(
            "{:.2}",
            scenario
                .config
                .simulation
                .model
                .common
                .propagation_velocities_m_per_s
                .get(&VoxelType::Sinoatrial)
                .unwrap()
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
        let path = path.join("npy");
        fs::create_dir_all(&path).unwrap();
        let writer = BufWriter::new(File::create(path.join("x_epochs.npy")).unwrap());
        x_epochs.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("x_snapshots.npy")).unwrap());
        x_snapshots.write_npy(writer).unwrap();
        for (n, label) in labels.iter().enumerate() {
            let writer =
                BufWriter::new(File::create(path.join(format!("loss_{label}.npy"))).unwrap());
            losses[n].write_npy(writer).unwrap();
            let writer =
                BufWriter::new(File::create(path.join(format!("param_{label}.npy"))).unwrap());
            params[n].write_npy(writer).unwrap();
            let writer = BufWriter::new(
                File::create(path.join(format!("param_error_{label}.npy"))).unwrap(),
            );
            params_error[n].write_npy(writer).unwrap();
            let writer =
                BufWriter::new(File::create(path.join(format!("delay_{label}.npy"))).unwrap());
            delays[n].write_npy(writer).unwrap();
            let writer = BufWriter::new(
                File::create(path.join(format!("delay_error_{label}.npy"))).unwrap(),
            );
            delays_error[n].write_npy(writer).unwrap();
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
    .unwrap();

    line_plot(
        Some(&x_snapshots),
        params,
        Some(files[1].as_path()),
        Some(format!("{base_title} - AP Coef").as_str()),
        Some("AP Coef (Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .unwrap();

    line_plot(
        Some(&x_snapshots),
        params_error,
        Some(files[2].as_path()),
        Some(format!("{base_title} - AP Coef Error").as_str()),
        Some("AP Coef (Target - Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .unwrap();

    line_plot(
        Some(&x_snapshots),
        delays,
        Some(files[3].as_path()),
        Some(format!("{base_title} - AP Delay").as_str()),
        Some("AP Delay (Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .unwrap();

    line_plot(
        Some(&x_snapshots),
        delays_error,
        Some(files[4].as_path()),
        Some(format!("{base_title} - AP Delay Error").as_str()),
        Some("AP Delay (Target - Estimated)"),
        Some("Snapshot"),
        Some(&labels),
        None,
    )
    .unwrap();
}

fn create_and_run(
    target_velocities: Vec<f32>,
    initial_velocity: f32,
    base_id: &str,
    path: &Path,
    base_title: &str,
) {
    let mut scenarios = Vec::new();
    let mut join_handles = Vec::new();

    for target_velocity in target_velocities {
        let id =
            format!("{base_id} {initial_velocity:.2} [m per s] to {target_velocity:.2} [m per s]");
        let path = Path::new("results").join(&id);
        if path.is_dir() {
            println!("Found scenario. Loading it!");
            let mut scenario = Scenario::load(path.as_path());
            scenario.load_data();
            scenario.load_results();
            scenarios.push(scenario);
        } else {
            println!("Didn't find scenario. Building it!");
            let scenario = build_scenario(target_velocity, initial_velocity, base_id);
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
            handle.join().unwrap();
        }
        for scenario in &mut scenarios {
            let path = Path::new("results").join(scenario.id.clone());
            *scenario = Scenario::load(path.as_path());
            scenario.load_data();
            scenario.load_results();
        }
    }

    plot_results(path, base_title, scenarios);
}
