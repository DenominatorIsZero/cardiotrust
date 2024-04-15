use bevy::{prelude::*, window::PresentMode};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use cardiotrust::{
    ui::ClientUiPlugin, vis::VisPlugin, websocket::WebsocketPlugin, ScenarioList, SelectedSenario,
};

#[tracing::instrument(level = "info")]
fn main() {
    info!("Starting Websocket Client application.");
    App::new()
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceDefault,
            },
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    // Uncomment this to override the default log settings:
                    // level: bevy::log::Level::TRACE,
                    // filter: "wgpu=warn,bevy_ecs=info".to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoNoVsync, // Reduces input lag.
                        canvas: Some("#bevy".to_string()),
                        ..default()
                    }),
                    ..default()
                }),
        ))
        .init_resource::<SelectedSenario>()
        .insert_resource(ScenarioList::empty())
        .add_plugins(WebsocketPlugin)
        .add_plugins(ClientUiPlugin)
        .add_plugins(VisPlugin)
        .run();
}
