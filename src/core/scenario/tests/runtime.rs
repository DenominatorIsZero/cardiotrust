use std::{path::Path, sync::mpsc::channel, thread};

use super::RUN_IN_TESTS;
use crate::{
    core::{
        algorithm::refinement::Optimizer,
        config::algorithm::AlgorithmType,
        model::spatial::voxels::VoxelType,
        scenario::{run, Scenario},
    },
    tests::setup_folder,
};

const COMMON_PATH: &str = "tests/core/scenario/runtime/";
const NUMBER_OF_EPOCHS: usize = 1000;
const LEARNING_RATE: f32 = 1e4;
const VOXELS_IN_CUBE: [i32; 9] = [2, 3, 4, 5, 6, 7, 8, 9, 10];
const VOXELS_IN_LINE: [i32; 9] = [8, 27, 64, 125, 216, 343, 512, 729, 1000];
const VOXELS_IN_SHEET: [i32; 9] = [3, 5, 8, 11, 15, 19, 23, 27, 32];

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum ScenarioType {
    Line,
    Sheet,
    Cube,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive runtime test"]
fn heavy_runtime_line() {
    let base_id = "Runtime Line";
    let path = Path::new(COMMON_PATH);

    let scenario_type = ScenarioType::Line;
    let voxel_counts = VOXELS_IN_LINE.to_vec();

    create_and_run(scenario_type, &voxel_counts, base_id, path);
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive runtime test"]
fn heavy_runtime_sheet() {
    let base_id = "Runtime Sheet";
    let path = Path::new(COMMON_PATH);

    let scenario_type = ScenarioType::Sheet;
    let voxel_counts = VOXELS_IN_SHEET.to_vec();

    create_and_run(scenario_type, &voxel_counts, base_id, path);
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive runtime test"]
fn heavy_runtime_cube() {
    let base_id = "Runtime Cube";
    let path = Path::new(COMMON_PATH);

    let scenario_type = ScenarioType::Cube;
    let voxel_counts = VOXELS_IN_CUBE.to_vec();

    create_and_run(scenario_type, &voxel_counts, base_id, path);
}

#[tracing::instrument(level = "trace")]
fn create_and_run(
    scenario_type: ScenarioType,
    voxel_counts: &Vec<i32>,
    base_id: &str,
    path: &Path,
) {
    let mut join_handles = Vec::new();
    let mut scenarios = Vec::new();

    let lower_delay_samples = 4.1;
    let upper_delay_samples = 5.2;

    for voxel_count in voxel_counts {
        for algorithm_type in [AlgorithmType::ModelBased, AlgorithmType::ModelBasedGPU] {
            let id = format!("{base_id} - {algorithm_type:?} - {voxel_count}");
            let path = Path::new("results").join(&id);
            println!("Looking for scenario {path:?}");
            let scenario = if path.is_dir() {
                println!("Found scenario. Loading it!");
                let scenario = Scenario::load(path.as_path());
                scenario
            } else {
                println!("Didn't find scenario. Building it!");
                let scenario = build_scenario(
                    upper_delay_samples,
                    lower_delay_samples,
                    *voxel_count,
                    algorithm_type,
                    scenario_type,
                    &id,
                );
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
            handle.join().unwrap();
        }
        for scenario in &mut scenarios {
            let path = Path::new("results").join(scenario.id.clone());
            *scenario = Scenario::load(path.as_path());
        }
    }
    plot_results(path, base_id, &scenarios, voxel_counts, scenario_type);
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
    voxel_count: i32,
    algorithm_type: AlgorithmType,
    scenario_type: ScenarioType,
    id: &str,
) -> Scenario {
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
    // Adjust heart size
    match scenario_type {
        ScenarioType::Line => {
            scenario
                .config
                .simulation
                .model
                .handcrafted
                .as_mut()
                .unwrap()
                .heart_size_mm = [
                voxel_size_mm,
                voxel_size_mm * (voxel_count) as f32,
                voxel_size_mm,
            ];
            scenario.config.simulation.model.common.heart_offset_mm = [
                25.0,
                -250.0 - (voxel_size_mm * (voxel_count) as f32) / 2.0,
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
                .unwrap()
                .heart_size_mm = [
                voxel_size_mm * (voxel_count) as f32,
                voxel_size_mm * (voxel_count) as f32,
                voxel_size_mm,
            ];
            scenario.config.simulation.model.common.heart_offset_mm = [
                25.0 - (voxel_size_mm * (voxel_count) as f32) / 2.0,
                -250.0 - (voxel_size_mm * (voxel_count) as f32) / 2.0,
                180.0,
            ];
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
        }
        ScenarioType::Cube => {
            scenario
                .config
                .simulation
                .model
                .handcrafted
                .as_mut()
                .unwrap()
                .heart_size_mm = [
                voxel_size_mm * (voxel_count) as f32,
                voxel_size_mm * (voxel_count) as f32,
                voxel_size_mm * (voxel_count) as f32,
            ];
            scenario.config.simulation.model.common.heart_offset_mm = [
                25.0 - (voxel_size_mm * (voxel_count) as f32) / 2.0,
                -250.0 - (voxel_size_mm * (voxel_count) as f32) / 2.0,
                180.0 - (voxel_size_mm * (voxel_count) as f32) / 2.0,
            ];
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
        }
    }
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
    scenario.config.algorithm.epochs = NUMBER_OF_EPOCHS;
    scenario.config.algorithm.learning_rate = LEARNING_RATE;
    scenario.config.algorithm.optimizer = Optimizer::Sgd;
    scenario.config.algorithm.algorithm_type = algorithm_type;
    scenario.config.algorithm.freeze_delays = false;
    scenario.config.algorithm.freeze_gains = true;
    scenario.config.algorithm.difference_regularization_strength = 0.0;
    scenario.config.algorithm.slow_down_stregth = 0.0;

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
fn plot_results(
    path: &Path,
    base_title: &str,
    scenarios: &Vec<Scenario>,
    voxel_counts: &Vec<i32>,
    scenario_type: ScenarioType,
) {
    setup_folder(path);
}
