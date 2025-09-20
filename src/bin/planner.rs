use std::process::Command;

use anyhow::{Context, Result};
use bevy::prelude::*;
use cardiotrust::core::{
    algorithm::refinement::Optimizer,
    config::{algorithm::Algorithm, model::SensorArrayMotion, simulation::Simulation},
    scenario::Scenario,
};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt};

#[tracing::instrument(level = "info")]
fn main() {
    if let Err(e) = run_planner() {
        eprintln!("Scenario planning failed: {:#}", e);
        std::process::exit(1);
    }
}

#[tracing::instrument(level = "info")]
fn run_planner() -> Result<()> {
    // Set up logging with graceful fallback
    setup_logging().context("Failed to set up logging for planner")?;

    // Get git hash with fallback to "unknown"
    let git_hash = get_git_hash();

    info!("Starting CardioTRust planner. Git hash: {}", git_hash);

    plan_scenarios().context("Failed to plan scenarios")?;

    Ok(())
}

#[tracing::instrument(level = "info")]
fn plan_scenarios() -> Result<()> {
    let learning_rate = 100.0;
    let steps = 30_000;
    let batch_size = 1;
    let experiment_name = "Moving_Sensors_2023_05_28";
    let maximum_regularization_strength = 1.0;
    let maximum_regularization_threshold = 1.001;
    let optimizer = Optimizer::Sgd;
    let freeze_gains = false;
    let freeze_delays = true;

    let mut algorithm_config = Algorithm {
        optimizer,
        epochs: steps,
        batch_size,
        learning_rate,
        maximum_regularization_strength,
        maximum_regularization_threshold,
        freeze_gains,
        freeze_delays,
        ..Default::default()
    };

    let mut simulation_config = Simulation::default();
    simulation_config.model.common.pathological = true;
    simulation_config.model.common.sensor_array_origin_mm = [0.0, -225.0, 25.0];
    simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;
    simulation_config.model.common.measurement_covariance_mean = 1e-20;

    let mut scenario = Scenario::build(Some(format!("{experiment_name} - (I) - Static Array")));
    scenario.config.algorithm = algorithm_config.clone();
    scenario.config.simulation = simulation_config.clone();
    scenario.schedule()
        .with_context(|| format!("Failed to schedule static array scenario for experiment '{}'", experiment_name))?;
    scenario.save()
        .with_context(|| format!("Failed to save static array scenario for experiment '{}'", experiment_name))?;

    if false {
        for y_step_exp in 1..=10 {
            let y_step = 2_usize.pow(y_step_exp) + 1;
            simulation_config.model.common.sensor_array_motion_steps = [1, y_step, 1];
            simulation_config.model.common.sensor_array_origin_mm = [0.0, -525.0, 25.0];
            simulation_config.model.common.sensor_array_motion_range_mm = [0.0, 600.0, 0.0];
            algorithm_config.epochs = (steps as f32 / y_step as f32).ceil() as usize;
            simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Grid;
            let mut scenario = Scenario::build(Some(format!(
                "{experiment_name} - (II) - Move Along Y - {y_step:0>4} Steps"
            )));
            scenario.config.algorithm = algorithm_config.clone();
            scenario.config.simulation = simulation_config.clone();
            scenario.schedule()
                .with_context(|| format!("Failed to schedule Y-motion scenario for experiment '{}', {} steps", experiment_name, y_step))?;
            scenario.save()
                .with_context(|| format!("Failed to save Y-motion scenario for experiment '{}', {} steps", experiment_name, y_step))?;
        }
    }

    for step in 2..=10_usize {
        let total_steps = step.pow(3);
        simulation_config.model.common.sensor_array_motion_steps = [step, step, step];
        simulation_config.model.common.sensor_array_origin_mm = [-75.0, -525.0, -25.0];
        simulation_config.model.common.sensor_array_motion_range_mm = [150.0, 600.0, 150.0];
        algorithm_config.epochs = (steps as f32 / total_steps as f32).round() as usize;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Grid;
        let mut scenario = Scenario::build(Some(format!(
            "{experiment_name} - (II) - Move Along XYZ - {total_steps:0>4} Steps"
        )));
        scenario.config.algorithm = algorithm_config.clone();
        scenario.config.simulation = simulation_config.clone();
        scenario.schedule()
            .with_context(|| format!("Failed to schedule XYZ-motion scenario for experiment '{}', {} total steps", experiment_name, total_steps))?;
        scenario.save()
            .with_context(|| format!("Failed to save XYZ-motion scenario for experiment '{}', {} total steps", experiment_name, total_steps))?;
    }

    for lr_exp in -3..=4 {
        let step: usize = 10;
        let total_steps = step.pow(3);
        simulation_config.model.common.sensor_array_motion_steps = [step, step, step];
        simulation_config.model.common.sensor_array_origin_mm = [-75.0, -525.0, -25.0];
        simulation_config.model.common.sensor_array_motion_range_mm = [150.0, 600.0, 150.0];
        algorithm_config.epochs = (steps as f32 / total_steps as f32).round() as usize;
        let lr = 10.0_f32.powf(lr_exp as f32);
        algorithm_config.learning_rate = 1.0;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Grid;
        let mut scenario = Scenario::build(Some(format!(
            "{experiment_name} - (III) - Move Along XYZ (LR Sweep)- {lr} LR"
        )));
        scenario.config.algorithm = algorithm_config.clone();
        scenario.config.simulation = simulation_config.clone();
        scenario.schedule()
            .with_context(|| format!("Failed to schedule LR sweep scenario for experiment '{}', learning rate {}", experiment_name, lr))?;
        scenario.save()
            .with_context(|| format!("Failed to save LR sweep scenario for experiment '{}', learning rate {}", experiment_name, lr))?;
    }

    Ok(())
}

#[tracing::instrument(level = "debug")]
fn setup_logging() -> Result<()> {
    // Try to set up file logging, fall back to stdout-only if it fails
    if let Err(e) = try_setup_file_logging() {
        eprintln!("Warning: Could not set up file logging ({}), using stdout only", e);
        setup_stdout_logging()?;
    }

    Ok(())
}

#[tracing::instrument(level = "debug")]
fn setup_stdout_logging() -> Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_thread_names(true)
                .with_ansi(true),
        );

    tracing::subscriber::set_global_default(subscriber)
        .with_context(|| "Failed to set up stdout logging")?;

    Ok(())
}

#[tracing::instrument(level = "debug")]
fn try_setup_file_logging() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("./logs", "CardioPlanner.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Store the guard to prevent it from being dropped
    std::mem::forget(_guard);

    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_thread_names(true)
                .with_ansi(true),
        )
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking)
                .with_thread_names(true)
                .with_line_number(true)
                .fmt_fields(fmt::format::PrettyFields::new())
                .with_ansi(false),
        );

    tracing::subscriber::set_global_default(subscriber)
        .with_context(|| "Failed to set up file logging")?;

    Ok(())
}

#[tracing::instrument(level = "debug")]
fn get_git_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|hash| hash.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
