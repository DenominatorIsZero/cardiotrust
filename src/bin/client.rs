use bevy::{prelude::*, window::PresentMode};
use rusty_cde::{
    ui::ClientUiPlugin, vis::VisPlugin, websocket::WebsocketPlugin, ScenarioList, SelectedSenario,
};

fn main() {
    App::new()
        .add_plugins(
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
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_resource::<SelectedSenario>()
        .insert_resource(ScenarioList::empty())
        .add_plugins(WebsocketPlugin)
        .add_plugins(ClientUiPlugin)
        .add_plugins(VisPlugin)
        .run();
}
