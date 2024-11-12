use std::process::Command;

use bevy::{log::LogPlugin, prelude::*};
use cardiotrust::{
    scheduler::SchedulerPlugin, ui::UiPlugin, vis::VisPlugin, ScenarioList, SelectedSenario,
};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt};

#[tracing::instrument(level = "info")]
fn main() {
    // Set up the file appender
    let file_appender = tracing_appender::rolling::daily("./logs", "CardioTRust.log");
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

    App::new()
        .init_resource::<ScenarioList>()
        .init_resource::<SelectedSenario>()
        .insert_resource(Msaa::Off)
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
}
