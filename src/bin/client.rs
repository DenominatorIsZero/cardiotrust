use bevy::prelude::*;
use rusty_cde::{
    ui::ClientUiPlugin,
    vis::{ClientVisPlugin, VisPlugin},
    websocket::WebsocketPlugin,
    ScenarioList, SelectedSenario,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            // Uncomment this to override the default log settings:
            // level: bevy::log::Level::TRACE,
            // filter: "wgpu=warn,bevy_ecs=info".to_string(),
            ..default()
        }))
        .init_resource::<SelectedSenario>()
        .insert_resource(ScenarioList::empty())
        .add_plugins(WebsocketPlugin)
        .add_plugins(ClientUiPlugin)
        .add_plugins(ClientVisPlugin)
        .run();
}
