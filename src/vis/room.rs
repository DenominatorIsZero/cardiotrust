use bevy::prelude::*;

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(skip(commands, ass), level = "debug")]
pub(crate) fn spawn_room(mut commands: Commands, ass: Res<AssetServer>) {
    let glb_handle = ass.load("bed.glb#Scene0");

    commands.spawn(SceneBundle {
        scene: glb_handle,
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::ONE * 1000.0),
        ..Default::default()
    });

    let glb_handle = ass.load("room.glb#Scene0");

    commands.spawn(SceneBundle {
        scene: glb_handle,
        transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::ONE * 1000.0),
        ..Default::default()
    });
}
