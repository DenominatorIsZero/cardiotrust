#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
use bevy::{prelude::*, window::WindowMode};
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids, VertexPullingRenderPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use rusty_cde::{
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
