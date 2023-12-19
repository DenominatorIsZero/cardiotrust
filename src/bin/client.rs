use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            // Uncomment this to override the default log settings:
            // level: bevy::log::Level::TRACE,
            // filter: "wgpu=warn,bevy_ecs=info".to_string(),
            ..default()
        }))
        .add_systems(Update, log_system)
        .run();
}

fn init_websocket() {}

fn log_system() {
    //    trace!("very noisy");
    //    debug!("helpful for debugging");
    info!("Hello world!");
    //    warn!("something bad happened that isn't a failure, but thats worth calling out");
    //    error!("something failed");
}
