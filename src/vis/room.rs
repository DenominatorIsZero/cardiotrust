use bevy::prelude::*;

use super::options::VisibilityOptions;

#[derive(Component)]
pub struct Room;

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(skip(commands, ass), level = "debug")]
pub(crate) fn spawn_room(mut commands: Commands, ass: Res<AssetServer>) {
    let glb_handle = ass.load("bed.glb#Scene0");

    commands.spawn((
        SceneBundle {
            scene: glb_handle,
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::ONE * 1000.0),
            ..Default::default()
        },
        Room,
    ));

    let glb_handle = ass.load("room.glb#Scene0");

    commands.spawn((
        SceneBundle {
            scene: glb_handle,
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::ONE * 1000.0),
            ..Default::default()
        },
        Room,
    ));
}

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn update_room_visibility(
    mut room: Query<&mut Visibility, With<Room>>,
    options: Res<VisibilityOptions>,
) {
    if options.is_changed() {
        for mut visibility in &mut room {
            if options.room {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
