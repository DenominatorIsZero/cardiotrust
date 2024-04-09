use bevy::math::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct CuttingPlane {
    pub position: Vec3,
    pub normal: Vec3,
    pub visible: bool,
    pub opacity: f32,
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn spawn_cutting_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let position = Vec3::new(0.0, 0.0, 0.0);
    let normal = Vec3::new(0.0, 0.0, 1.0);

    let rotation = Quat::from_rotation_arc(Vec3::Y, normal);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Plane3d {
                normal: Direction3d::Y,
            })),
            material: materials.add(StandardMaterial::from(Color::rgba(1.0, 1.0, 1.0, 0.5))),
            transform: Transform {
                translation: position,
                rotation,
                scale: Vec3::new(1000.0, 1000.0, 1000.0),
            },
            ..default()
        },
        CuttingPlane {
            position,
            normal,
            visible: true,
            opacity: 0.5,
        },
    ));
}
