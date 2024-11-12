use std::{path::Path, sync::mpsc::channel, thread};

use ndarray::Array1;

use super::RUN_IN_TESTS;
use crate::{
    core::{
        algorithm::{metrics::BatchWiseMetric, refinement::Optimizer},
        model::spatial::voxels::VoxelType,
        scenario::{run, Scenario},
    },
    tests::{clean_files, setup_folder},
    vis::plotting::png::line::log_y_plot,
};

const COMMON_PATH: &str = "tests/core/scenario/sheet_ap/";

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore]
fn no_roll_down() {
    let base_id = "Sheet AP - No Roll - Down - ".to_string();
    let base_title = "Sheet AP - No Roll - Down";
    let path = Path::new(COMMON_PATH).join("no_roll_down");

    let integer_part = 5.0;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_fractional = 0.5;
    let initial_delay = integer_part + initial_fractional;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let target_fractional = 0.55;
    let target_delay = integer_part + target_fractional;
    let target_delay_s = target_delay / sample_rate_hz;
    let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;

    create_and_run(
        target_velocity,
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
#[ignore]
fn no_roll_up() {
    let base_id = "Sheet AP - No Roll - Up - ".to_string();
    let base_title = "Sheet AP - No Roll - Up";
    let path = Path::new(COMMON_PATH).join("no_roll_up");

    let integer_part = 5.0;
    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let initial_fractional = 0.55;
    let initial_delay = integer_part + initial_fractional;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let target_fractional = 0.5;
    let target_delay = integer_part + target_fractional;
    let target_delay_s = target_delay / sample_rate_hz;
    let target_velocity = voxel_size_mm / 1000.0 / target_delay_s;

    create_and_run(
        target_velocity,
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
    scenario.config.algorithm.optimizer = Optimizer::Sgd;
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
fn plot_results(path: &Path, base_title: &str, scenario: &Scenario) {
    setup_folder(path);
    let files = vec![path.join("loss.png")];
    clean_files(&files);

    println!(
        "{:?}",
        scenario
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

    let x_epochs = Array1::range(0.0, scenario.config.algorithm.epochs as f32, 1.0);
    let losses_owned: Vec<BatchWiseMetric> = vec![scenario
        .results
        .as_ref()
        .unwrap()
        .metrics
        .loss_mse_batch
        .clone()];

    let losses = losses_owned
        .iter()
        .map(std::ops::Deref::deref)
        .collect::<Vec<&Array1<f32>>>();

    println!("ys length: {}", losses.len());

    log_y_plot(
        Some(&x_epochs),
        losses,
        Some(files[0].as_path()),
        Some(format!("{base_title} - Loss").as_str()),
        Some("Loss MSE"),
        Some("Epoch"),
        None,
        None,
    )
    .unwrap();
}

fn create_and_run(
    target_velocity: f32,
    initial_velocity: f32,
    base_id: &str,
    img_path: &Path,
    base_title: &str,
) {
    let mut join_handles = Vec::new();

    let id = format!("{base_id} {initial_velocity:.2} [m per s] to {target_velocity:.2} [m per s]");
    let path = Path::new("results").join(id);
    let mut scenario = if path.is_dir() {
        println!("Found scenario. Loading it!");
        let mut scenario = Scenario::load(path.as_path());
        scenario.load_data();
        scenario.load_results();
        scenario
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
        scenario
    };

    if RUN_IN_TESTS {
        for handle in join_handles {
            handle.join().unwrap();
        }
        let path = Path::new("results").join(scenario.id.clone());
        scenario = Scenario::load(path.as_path());
        scenario.load_data();
        scenario.load_results();
    }

    plot_results(img_path, base_title, &scenario);
}
