use bevy::{prelude::*, window::WindowMode};

use cardiotrust::{
    scheduler::SchedulerPlugin, ui::UiPlugin, vis::VisPlugin, ScenarioList, SelectedSenario,
};

fn main() {
    App::new()
        .init_resource::<ScenarioList>()
        .init_resource::<SelectedSenario>()
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty CDE".into(),
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiPlugin)
        .add_plugins(SchedulerPlugin)
        .add_plugins(VisPlugin)
        .run();
}
