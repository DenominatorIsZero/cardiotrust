use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
    sync::mpsc::channel,
    thread,
};

use anyhow::Result;

use ndarray::Array1;
use ndarray_npy::WriteNpyExt;

use super::{RUN_IN_TESTS, SAVE_NPY};
use crate::{
    core::{
        algorithm::refinement::Optimizer,
        config::{algorithm::APDerivative, model::ControlFunction},
        model::spatial::voxels::VoxelType,
        scenario::{run, Scenario},
    },
    tests::{clean_files, setup_folder},
    vis::plotting::png::line::{line_plot, log_y_plot},
};

const COMMON_PATH: &str = "tests/core/scenario/losslandscape/";

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]
#[test]
#[ignore = "expensive integration test"]
fn heavy_loss_landscape() -> Result<()> {
    let base_id = "Single AP - Loss Landscape".to_string();
    let base_title = "Single AP - Loss Landscape";
    let path = Path::new(COMMON_PATH).join("loss_landscape");

    let initial_delay = 11.5;
    let single_sensor = false;
    let control_function = ControlFunction::Ohara;

    let support_points = 2001;
    let min_delay = 1.5;
    let max_delay = 21.5;

    create_and_run(
        initial_delay,
        min_delay,
        max_delay,
        single_sensor,
        control_function,
        support_points,
        &base_id,
        &path,
        base_title,
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
fn heavy_loss_landscape_single_sensor() -> Result<()> {
    let base_id = "Single AP - Loss Landscape - Single Sensor".to_string();
    let base_title = "Single AP - Loss Landscape - Single Sensor";
    let path = Path::new(COMMON_PATH).join("single_sensor");

    let single_sensor = true;
    let control_function = ControlFunction::Ohara;
    let initial_delay = 11.5;

    let support_points = 2001;
    let min_delay = 1.5;
    let max_delay = 21.5;

    create_and_run(
        initial_delay,
        min_delay,
        max_delay,
        single_sensor,
        control_function,
        support_points,
        &base_id,
        &path,
        base_title,
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
fn heavy_loss_landscape_triangle() -> Result<()> {
    let base_id = "Single AP - Loss Landscape - Triangle".to_string();
    let base_title = "Single AP - Loss Landscape - Triangle";
    let path = Path::new(COMMON_PATH).join("triangle");

    let single_sensor = true;
    let control_function = ControlFunction::Triangle;
    let initial_delay = 11.01;

    let support_points = 2001;
    let min_delay = 1.5;
    let max_delay = 21.5;

    create_and_run(
        initial_delay,
        min_delay,
        max_delay,
        single_sensor,
        control_function,
        support_points,
        &base_id,
        &path,
        base_title,
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
fn heavy_loss_landscape_ramp() -> Result<()> {
    let base_id = "Single AP - Loss Landscape - Ramp".to_string();
    let base_title = "Single AP - Loss Landscape - Ramp";
    let path = Path::new(COMMON_PATH).join("ramp");

    let single_sensor = true;
    let control_function = ControlFunction::Ramp;
    let initial_delay = 11.01;

    let support_points = 2001;
    let min_delay = 1.5;
    let max_delay = 21.5;

    create_and_run(
        initial_delay,
        min_delay,
        max_delay,
        single_sensor,
        control_function,
        support_points,
        &base_id,
        &path,
        base_title,
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
fn create_and_run(
    initial_delay: f32,
    min_delay: f32,
    max_delay: f32,
    single_sensor: bool,
    control_function: ControlFunction,
    support_points: usize,
    base_id: &str,
    path: &Path,
    base_title: &str,
) -> Result<()> {
    let sample_rate_hz = 2000.0;
    let voxel_size_mm = 2.5;
    let step = (max_delay - min_delay) / support_points as f32;
    let initial_delay_s = initial_delay / sample_rate_hz;
    let initial_velocity = voxel_size_mm / 1000.0 / initial_delay_s;

    let mut scenarios = Vec::new();
    let mut join_handles = Vec::new();

    for i in 0..support_points {
        let gt_delay = (i as f32).mul_add(step, min_delay);
        let gt_delay_s = gt_delay / sample_rate_hz;
        let gt_velocity = voxel_size_mm / 1000.0 / gt_delay_s;

        let id = format!("{base_id} {initial_delay:.2} [samples] to {gt_delay:.2} [samples]");
        let path = Path::new("results").join(&id);
        if path.is_dir() {
            println!("Found scenario. Loading it!");
            let mut scenario = Scenario::load(path.as_path())?;
            scenario.load_data()?;
            scenario.load_results()?;
            scenarios.push(scenario);
        } else {
            println!("Didn't find scenario. Building it!");
            let scenario = build_scenario(
                gt_velocity,
                initial_velocity,
                single_sensor,
                control_function,
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
            scenarios.push(scenario);
        }
    }

    if RUN_IN_TESTS {
        for handle in join_handles {
            handle.join().unwrap();
        }
        for scenario in &mut scenarios {
            let path = Path::new("results").join(scenario.id.clone());
            *scenario = Scenario::load(path.as_path())?;
            scenario.load_data()?;
            scenario.load_results()?;
        }
    }

    plot_results(path, base_title, scenarios);
    Ok(())
}

#[tracing::instrument(level = "trace")]
fn build_scenario(
    target_velocity: f32,
    initial_velocity: f32,
    single_sensor: bool,
    control_function: ControlFunction,
    id: &str,
) -> Scenario {
    let mut scenario = Scenario::build(Some(id.to_string()));

    // Configure Sensor
    if single_sensor {
        scenario.config.simulation.model.common.sensors_per_axis = [1, 1, 1];
        scenario.config.simulation.model.common.three_d_sensors = false;
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
                    .unwrap()
                    .heart_size_mm[0]
                    / 2.0,
            scenario.config.simulation.model.common.heart_offset_mm[1]
                + scenario
                    .config
                    .simulation
                    .model
                    .handcrafted
                    .as_ref()
                    .unwrap()
                    .heart_size_mm[1]
                    / 2.0,
            scenario
                .config
                .simulation
                .model
                .common
                .sensor_array_origin_mm[2],
        ];
    }
    // Configure ControlFunction:
    scenario.config.simulation.model.common.control_function = control_function;
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
    scenario.config.algorithm.epochs = 1;
    scenario.config.algorithm.learning_rate = 0.0;
    scenario.config.algorithm.optimizer = Optimizer::Sgd;
    scenario.config.algorithm.ap_derivative = APDerivative::Textbook;
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
#[tracing::instrument(level = "trace")]
fn plot_results(path: &Path, base_title: &str, scenarios: Vec<Scenario>) {
    setup_folder(path);
    let files = vec![path.join("loss.png"), path.join("gradients.png")];
    clean_files(&files);

    let mut delays = Array1::<f32>::zeros(scenarios.len());
    let mut losses = Array1::<f32>::zeros(scenarios.len());
    let mut gradients = Array1::<f32>::zeros(scenarios.len());

    println!("{}", scenarios.len());
    for (i, scenario) in scenarios.iter().enumerate() {
        losses[i] = scenario.results.as_ref().unwrap().metrics.loss_mse_batch[0];
        delays[i] = scenario
            .data
            .as_ref()
            .unwrap()
            .simulation
            .model
            .functional_description
            .ap_params
            .initial_delays[(0, 15)];
        gradients[i] = scenario.results.as_ref().unwrap().derivatives.coefs[(0, 15)];
    }

    if SAVE_NPY {
        let path = path.join("npy");
        fs::create_dir_all(&path).unwrap();
        let writer = BufWriter::new(File::create(path.join("delays.npy")).unwrap());
        delays.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("losses.npy")).unwrap());
        losses.write_npy(writer).unwrap();
        let writer = BufWriter::new(File::create(path.join("gradients.npy")).unwrap());
        gradients.write_npy(writer).unwrap();
    }

    println!("ys length: {}", losses.len());

    log_y_plot(
        Some(&delays),
        vec![&losses],
        Some(files[0].as_path()),
        Some(format!("{base_title} - MSE").as_str()),
        Some("Loss MSE"),
        Some("GT Delay"),
        None,
        None,
    )
    .unwrap();
    line_plot(
        Some(&delays),
        vec![&gradients],
        Some(files[1].as_path()),
        Some(format!("{base_title} - Gradients").as_str()),
        Some("Gradient"),
        Some("GT Delay"),
        None,
        None,
    )
    .unwrap();
}
