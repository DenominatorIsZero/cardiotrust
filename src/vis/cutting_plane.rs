use bevy::{math::prelude::*, prelude::*};

use super::options::VisibilityOptions;

#[derive(Resource, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct CuttingPlaneSettings {
    pub position: Vec3,
    pub normal: Vec3,
    pub visible: bool,
    pub enabled: bool,
    pub opacity: f32,
}

#[derive(Component)]
#[allow(clippy::module_name_repetitions)]
pub struct CuttingPlane;

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn spawn_cutting_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let position = Vec3::new(0.1, 0.1, 0.1);
    let normal = Vec3::new(0.0, 1.0, 0.0);

    let rotation = Quat::from_rotation_arc(Vec3::Y, normal);

    let cutting_plane = CuttingPlaneSettings {
        position,
        normal,
        visible: false,
        enabled: false,
        opacity: 0.5,
    };

    commands.insert_resource(cutting_plane);

    let mut material = StandardMaterial::from(Color::srgba(1.0, 1.0, 1.0, 0.5));
    material.cull_mode = None;

    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d {
            normal: Dir3::Y,
            half_size: Vec2 { x: 1.0, y: 1.0 },
        }))),
        MeshMaterial3d(materials.add(material)),
        Transform {
            translation: position,
            rotation,
            scale: Vec3::new(1000.0, 1000.0, 1000.0),
        },
        CuttingPlane,
    ));
}

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn update_cutting_plane_position(
    mut cutting_planes: Query<&mut Transform, With<CuttingPlane>>,
    settings: Res<CuttingPlaneSettings>,
) {
    if settings.is_changed() {
        if let Ok(mut transform) = cutting_planes.single_mut() {
            transform.translation = settings.position;
            let rotation = Quat::from_rotation_arc(Vec3::Y, settings.normal);
            transform.rotation = rotation;
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn update_cutting_plane_visibility(
    mut cutting_plane: Query<&mut Visibility, With<CuttingPlane>>,
    options: Res<VisibilityOptions>,
) {
    if options.is_changed() {
        for mut visibility in &mut cutting_plane {
            if options.cutting_plane {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
