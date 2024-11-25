use std::{path::Path, sync::mpsc::channel, thread};

use egui::epaint::tessellator::PathType;
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

const COMMON_PATH: &str = "tests/core/scenario/smoothness_regularization/";

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore]
fn heavy_center_patch() {
    let base_id = "Smoothness Regularization".to_string();
    let base_title = "Smoothness Regularization";
    let path = Path::new(COMMON_PATH).join("center_patch");

    let voxel_size_mm = 2.5;
    let sample_rate_hz = 2000.0;

    let bulk_delay = 4.25;
    let bulk_delay_s = bulk_delay / sample_rate_hz;
    let bulk_velocity = voxel_size_mm / 1000.0 / bulk_delay_s;

    let patch_delay = 6.75;
    let patch_delay_s = patch_delay / sample_rate_hz;
    let patch_velocity = voxel_size_mm / 1000.0 / patch_delay_s;

    let smoothness_regularization_stengths = vec![1e-1, 1e-0, 1e+1, 1e+2];

    create_and_run(
        patch_velocity,
        bulk_velocity,
        smoothness_regularization_stengths,
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
#[tracing::instrument(level = "trace")]
fn build_scenario(
    patch_velocity: f32,
    bulk_velocity: f32,
    smoothness_regularization_stength: f32,
    base_id: &str,
) -> Scenario {
    let mut scenario = Scenario::build(Some(format!(
        "{base_id} bulk: {bulk_velocity:.2} [m per s], patch {patch_velocity:.2} [m per s], srs: {smoothness_regularization_stength:.2e}"
    )));

    // Set tissue types
    scenario.config.simulation.model.common.pathological = true;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .include_atrium = false;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .include_av = false;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .include_hps = false;
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
        .pathology_y_start_percentage = 0.4;
    scenario
        .config
        .simulation
        .model
        .handcrafted
        .as_mut()
        .unwrap()
        .pathology_y_stop_percentage = 0.6;
    // Adjust propagation velocities
    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Sinoatrial)
        .unwrap() = bulk_velocity;
    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Ventricle)
        .unwrap() = bulk_velocity;
    *scenario
        .config
        .simulation
        .model
        .common
        .propagation_velocities_m_per_s
        .get_mut(&VoxelType::Pathological)
        .unwrap() = patch_velocity;
    // Copy settings to algorithm model
    scenario.config.algorithm.model = scenario.config.simulation.model.clone();
    // set optimization parameters
    scenario.config.algorithm.epochs = 250;
    scenario.config.algorithm.learning_rate = 1e4;
    scenario.config.algorithm.optimizer = Optimizer::Sgd;
    scenario.config.algorithm.freeze_delays = false;
    scenario.config.algorithm.freeze_gains = true;
    scenario.config.algorithm.mse_strength = 0.0;
    scenario.config.algorithm.smoothness_regularization_strength =
        smoothness_regularization_stength;
    let number_of_snapshots = 50;
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
#[tracing::instrument(level = "trace")]
fn plot_results(path: &Path, base_title: &str, scenarios: Vec<Scenario>) {
    setup_folder(path);
    let files = vec![path.join("loss.png")];
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

    let mut labels_owned: Vec<String> = Vec::new();
    let x_epochs = Array1::range(0.0, first_scenario.config.algorithm.epochs as f32, 1.0);
    let num_snapshots = first_scenario.results.as_ref().unwrap().snapshots.len() as f32;
    let x_snapshots = Array1::range(0.0, num_snapshots, 1.0);
    let mut losses_owned: Vec<BatchWiseMetric> = Vec::new();

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
        labels_owned.push(format!(
            "{:.2e}",
            scenario.config.algorithm.smoothness_regularization_strength
        ));
    }

    let losses = losses_owned
        .iter()
        .map(std::ops::Deref::deref)
        .collect::<Vec<&Array1<f32>>>();
    let labels: Vec<&str> = labels_owned
        .iter()
        .map(std::string::String::as_str)
        .collect();

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
}

#[tracing::instrument(level = "trace")]
fn create_and_run(
    patch_velocity: f32,
    bulk_velocity: f32,
    smoothness_regularization_strengths: Vec<f32>,
    base_id: &str,
    img_path: &Path,
    base_title: &str,
) {
    let mut scenarios = Vec::new();
    let mut join_handles = Vec::new();

    for smoothness_regularization_strength in smoothness_regularization_strengths {
        let id = format!(
        "{base_id} bulk: {bulk_velocity:.2} [m per s], patch {patch_velocity:.2} [m per s], srs: {smoothness_regularization_strength:.2e}"
    );
        let path = Path::new("results").join(id);
        if path.is_dir() {
            println!("Found scenario. Loading it!");
            let mut scenario = Scenario::load(path.as_path());
            scenario.load_data();
            scenario.load_results();
            scenarios.push(scenario);
        } else {
            println!("Didn't find scenario. Building it!");
            let scenario = build_scenario(
                patch_velocity,
                bulk_velocity,
                smoothness_regularization_strength,
                base_id,
            );
            if RUN_IN_TESTS {
                let send_scenario = scenario.clone();
                let (epoch_tx, _) = channel();
                let (summary_tx, _) = channel();
                let handle = thread::spawn(move || run(send_scenario, &epoch_tx, &summary_tx));
                println!("handle {handle:?}");
                join_handles.push(handle);
            }
            scenarios.push(scenario);
        };
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

    plot_results(img_path, base_title, scenarios);
}
