use bevy::prelude::*;

use crate::core::scenario::Scenario;

/// Spawns sensor visualizations in the 3D scene.
///
/// Loads the 3D mesh assets for the sensor arrows and points.
/// Then iterates through the sensor positions and orientations defined in the scenario,
/// and spawns an entity for each with the appropriate transform.
/// The color and scale are also set based on the sensor orientation.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn spawn_sensors(
    commands: &mut Commands,
    ass: Res<AssetServer>,
    materials: &mut Assets<StandardMaterial>,
    scenario: &Scenario,
) {
    debug!("Running system to spawn sensors.");
    let data = scenario.data.as_ref().expect("Data to be some");
    let model = data.get_model();
    let sensors = &model.spatial_description.sensors;

    // note that we have to include the `Scene0` label
    let shaft_mesh: Handle<Mesh> = ass.load("RoundArrow.glb#Mesh0/Primitive0");
    let point_mesh: Handle<Mesh> = ass.load("RoundArrow.glb#Mesh1/Primitive0");

    for index_sensor in 0..sensors.positions_mm.shape()[0] {
        let x_pos_mm = sensors.positions_mm[(index_sensor, 0)];
        let y_pos_mm = sensors.positions_mm[(index_sensor, 1)];
        let z_pos_mm = sensors.positions_mm[(index_sensor, 2)];
        let x_ori = sensors.orientations_xyz[(index_sensor, 0)];
        let y_ori = sensors.orientations_xyz[(index_sensor, 1)];
        let z_ori = sensors.orientations_xyz[(index_sensor, 2)];

        let rot = Vec3::new(x_ori, y_ori, z_ori);

        commands.spawn(PbrBundle {
            mesh: shaft_mesh.clone(),
            material: materials.add(StandardMaterial::from(Color::rgba(
                x_ori, y_ori, z_ori, 1.0,
            ))),
            transform: Transform::from_xyz(x_pos_mm, y_pos_mm, z_pos_mm)
                .with_scale(Vec3::ONE * 10.0)
                .with_rotation(Quat::from_rotation_arc(-Vec3::Z, rot)),
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: point_mesh.clone(),
            material: materials.add(StandardMaterial::from(Color::rgba(
                x_ori, y_ori, z_ori, 1.0,
            ))),
            transform: Transform::from_xyz(x_pos_mm, y_pos_mm, z_pos_mm)
                .with_scale(Vec3::ONE * 10.0)
                .with_rotation(Quat::from_rotation_arc(-Vec3::Z, rot)),
            ..default()
        });
    }
}

#[derive(Component)]
pub(crate) struct SensorBracket {
    pub radius_mm: f32,
    pub position_mm: Vec3,
}

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn spawn_sensor_bracket(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    scenario: &Scenario,
) {
    let sensors = &scenario
        .data
        .as_ref()
        .unwrap()
        .get_model()
        .spatial_description
        .sensors;
    let radius = sensors.array_radius_mm;
    let position = &sensors.array_center_mm;
    let mesh = meshes.add(Mesh::from(Cylinder {
        radius: 1.0,
        half_height: 30.0,
    }));
    commands.spawn((
        PbrBundle {
            mesh,
            material: materials.add(StandardMaterial::from(Color::rgba(1.0, 1.0, 1.0, 0.2))),
            transform: Transform::from_xyz(position[0], position[1], position[2]),
            ..default()
        },
        SensorBracket {
            radius_mm: radius,
            position_mm: Vec3::new(position[0], position[1], position[2]),
        },
    ));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sensor_bracket(mut sensor_brackets: Query<(&mut Transform, &SensorBracket)>) {
    let sensor_bracket = sensor_brackets.get_single_mut();

    if let Ok((mut transform, sensor_bracket)) = sensor_bracket {
        transform.translation = sensor_bracket.position_mm;
        transform.scale.x = sensor_bracket.radius_mm;
        transform.scale.z = sensor_bracket.radius_mm;
    }
}
