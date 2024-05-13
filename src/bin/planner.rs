use std::process::Command;

use bevy::{log::LogPlugin, prelude::*, utils::petgraph::algo};
use js_sys::Math::exp;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use cardiotrust::{
    core::{
        algorithm::refinement::Optimizer,
        config::{algorithm::Algorithm, model::SensorArrayMotion, simulation::Simulation},
        scenario::{self, Scenario},
    },
    scheduler::SchedulerPlugin,
    ui::UiPlugin,
    vis::VisPlugin,
    ScenarioList, SelectedSenario,
};

fn main() {
    let file_appender = tracing_appender::rolling::daily("./logs", "CardioPlanner.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Combine multiple layers together
    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout) // Logs to stdout
                .with_thread_names(true)
                .with_ansi(true),
        ) // For colored logs in the console
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking) // Logs to file
                .with_thread_names(true)
                .with_line_number(true)
                .fmt_fields(fmt::format::PrettyFields::new())
                .with_ansi(false),
        ); // Typically, file logs don't need ANSI colors

    // Apply the combined subscriber to the current context
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();

    info!("Starting CardioTRust application. Git hash: {}", git_hash);

    plan_scenarios()
}

#[tracing::instrument(level = "info")]
fn plan_scenarios() {
    let learning_rate = 200.0;
    let steps = 100;
    let batch_size = 1;
    let experiment_name = "Moving_Sensors_2023_05_13";
    let regularization_strength = 1.0;
    let regularization_threshold = 1.001;
    let optimizer = Optimizer::Sgd;
    let freeze_gains = false;
    let freeze_delays = true;
    let update_kalman_gain = false;

    let mut algorithm_config = Algorithm {
        optimizer,
        epochs: steps,
        batch_size,
        learning_rate,
        regularization_strength,
        regularization_threshold,
        freeze_gains,
        freeze_delays,
        update_kalman_gain,
        ..Default::default()
    };

    let mut simulation_config = Simulation::default();
    simulation_config.model.common.pathological = true;
    simulation_config.model.common.sensor_array_origin_mm = [100.0, -100.0, 325.0];
    simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Static;

    let mut scenario = Scenario::build(Some(format!("{experiment_name} - (I) - Static Array")));
    scenario.config.algorithm = algorithm_config.clone();
    scenario.config.simulation = Some(simulation_config.clone());
    scenario.schedule().expect("Scheduling to succeed");
    scenario.save().expect("Scenario to save");

    for y_step_exp in 1..=10 {
        let y_step = 2_usize.pow(y_step_exp) + 1;
        simulation_config.model.common.sensor_array_motion_steps = [1, y_step, 1];
        simulation_config.model.common.sensor_array_origin_mm = [100.0, -400.0, 325.0];
        simulation_config.model.common.sensor_array_motion_range_mm = [0.0, 600.0, 0.0];
        algorithm_config.epochs = (steps as f32 / y_step as f32).round() as usize;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Grid;
        let mut scenario = Scenario::build(Some(format!(
            "{experiment_name} - (II) - Move Along Y - {y_step:0>4} Steps"
        )));
        scenario.config.algorithm = algorithm_config.clone();
        scenario.config.simulation = Some(simulation_config.clone());
        scenario.schedule().expect("Scheduling to succeed");
        scenario.save().expect("Scenario to save");
    }

    for step in 2..=10_usize {
        let total_steps = step.pow(3);
        simulation_config.model.common.sensor_array_motion_steps = [step, step, step];
        simulation_config.model.common.sensor_array_origin_mm = [-100.0, -400.0, -25.0];
        simulation_config.model.common.sensor_array_motion_range_mm = [300.0, 600.0, 600.0];
        algorithm_config.epochs = (steps as f32 / total_steps as f32).round() as usize;
        simulation_config.model.common.sensor_array_motion = SensorArrayMotion::Grid;
        let mut scenario = Scenario::build(Some(format!(
            "{experiment_name} - (III) - Move Along XYZ - {total_steps:0>4} Steps"
        )));
        scenario.config.algorithm = algorithm_config.clone();
        scenario.config.simulation = Some(simulation_config.clone());
        scenario.schedule().expect("Scheduling to succeed");
        scenario.save().expect("Scenario to save");
    }
}
