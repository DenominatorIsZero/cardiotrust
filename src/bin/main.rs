use bevy::{log::LogPlugin, prelude::*};
use tracing::info;

use cardiotrust::{
    scheduler::SchedulerPlugin, ui::UiPlugin, vis::VisPlugin, ScenarioList, SelectedSenario,
};

#[tracing::instrument]
fn main() {
    let file_appender = tracing_appender::rolling::hourly("./logs/", "cardiotrust.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    let (stdio_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());

    tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_writer(file_writer)
        .with_writer(stdio_writer)
        .init();

    info!("Starting CardioTRust");
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
