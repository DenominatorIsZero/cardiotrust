use bevy::math::prelude::*;
use bevy::prelude::*;

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
pub struct CuttingPlaneComponent;

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

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Plane3d {
                normal: Dir3::Y,
                half_size: Vec2 { x: 1.0, y: 1.0 },
            })),
            material: materials.add(StandardMaterial::from(Color::srgba(1.0, 1.0, 1.0, 0.5))),
            transform: Transform {
                translation: position,
                rotation,
                scale: Vec3::new(1000.0, 1000.0, 1000.0),
            },
            ..default()
        },
        CuttingPlaneComponent,
    ));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_cutting_plane(
    mut cutting_planes: Query<
        (&mut Transform, &Handle<StandardMaterial>),
        With<CuttingPlaneComponent>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<CuttingPlaneSettings>,
) {
    if settings.is_changed() {
        let (mut transform, material) = cutting_planes.single_mut();
        let opacity = if settings.visible {
            settings.opacity
        } else {
            0.0
        };
        materials.get_mut(material).unwrap().base_color = Color::srgba(1.0, 1.0, 1.0, opacity);

        transform.translation = settings.position;
        let rotation = Quat::from_rotation_arc(Vec3::Y, settings.normal);
        transform.rotation = rotation;
    }
}
