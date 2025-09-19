use std::process::Command;

use anyhow::Result;
use bevy::{log::LogPlugin, prelude::*};
use cardiotrust::{
    scheduler::SchedulerPlugin, ui::UiPlugin, vis::VisPlugin, ScenarioList, SelectedSenario,
};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt};

#[tracing::instrument(level = "info")]
fn main() {
    if let Err(e) = run_app() {
        eprintln!("Application failed to start: {}", e);
        std::process::exit(1);
    }
}

fn run_app() -> Result<()> {
    // Set up logging with graceful fallback
    setup_logging()?;

    // Get git hash with fallback to "unknown"
    let git_hash = get_git_hash();

    info!("Starting CardioTRust application. Git hash: {}", git_hash);

    App::new()
        .init_resource::<ScenarioList>()
        .init_resource::<SelectedSenario>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Cardio TRust".into(),
                        ..default()
                    }),
                    ..default()
                })
                .disable::<LogPlugin>(),
        )
        .add_plugins(UiPlugin)
        .add_plugins(SchedulerPlugin)
        .add_plugins(VisPlugin)
        .run();

    Ok(())
}

fn setup_logging() -> Result<()> {
    // Try to set up file logging, fall back to stdout-only if it fails
    if let Err(e) = try_setup_file_logging() {
        eprintln!("Warning: Could not set up file logging ({}), using stdout only", e);
        setup_stdout_logging()?;
    }

    Ok(())
}

fn setup_stdout_logging() -> Result<()> {
    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_thread_names(true)
                .with_ansi(true),
        );

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("Failed to set up stdout logging: {}", e))?;

    Ok(())
}

fn try_setup_file_logging() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily("./logs", "CardioTRust.log");
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
        .map_err(|e| anyhow::anyhow!("Failed to set up file logging: {}", e))?;

    Ok(())
}

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
