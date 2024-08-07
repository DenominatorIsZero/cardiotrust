use bevy::prelude::*;
use ndarray::Array2;

use crate::core::scenario::Scenario;

use super::sample_tracker::SampleTracker;

#[derive(Component)]
pub(crate) struct Sensor {
    pub positions_mm: Array2<f32>,
    pub _orientation: Vec3,
}

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
    ass: &Res<AssetServer>,
    materials: &mut Assets<StandardMaterial>,
    scenario: &Scenario,
) {
    debug!("Running system to spawn sensors.");
    let data = scenario.data.as_ref().expect("Data to be some");
    let model = &data.simulation.model;
    let sensors = &model.spatial_description.sensors;

    // note that we have to include the `Scene0` label
    let mesh: Handle<Mesh> = ass.load("RoundArrow.obj");

    let motion_steps = sensors.array_offsets_mm.shape()[0];

    let material_red = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        metallic: 0.0,
        ..Default::default()
    });

    let materials_green = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        metallic: 0.0,
        ..Default::default()
    });

    let material_blue = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        metallic: 0.0,
        ..Default::default()
    });

    for index_sensor in 0..sensors.positions_mm.shape()[0] {
        let material = match index_sensor % 3 {
            0 => material_red.clone(),
            1 => materials_green.clone(),
            _ => material_blue.clone(),
        };
        let mut positions_mm = Array2::zeros((motion_steps, 3));
        for i in 0..motion_steps {
            positions_mm[(i, 0)] =
                sensors.positions_mm[(index_sensor, 0)] + sensors.array_offsets_mm[(i, 0)];
            positions_mm[(i, 1)] =
                sensors.positions_mm[(index_sensor, 1)] + sensors.array_offsets_mm[(i, 1)];
            positions_mm[(i, 2)] =
                sensors.positions_mm[(index_sensor, 2)] + sensors.array_offsets_mm[(i, 2)];
        }
        let x_pos_mm = sensors.positions_mm[(index_sensor, 0)];
        let y_pos_mm = sensors.positions_mm[(index_sensor, 1)];
        let z_pos_mm = sensors.positions_mm[(index_sensor, 2)];
        let x_ori = sensors.orientations_xyz[(index_sensor, 0)];
        let y_ori = sensors.orientations_xyz[(index_sensor, 1)];
        let z_ori = sensors.orientations_xyz[(index_sensor, 2)];

        let rot = Vec3::new(x_ori, y_ori, z_ori);

        commands.spawn((
            PbrBundle {
                mesh: mesh.clone(),
                material,
                transform: Transform::from_xyz(x_pos_mm, y_pos_mm, z_pos_mm)
                    .with_scale(Vec3::ONE * 30.0)
                    .with_rotation(Quat::from_rotation_arc(-Vec3::Z, rot)),
                ..default()
            },
            Sensor {
                positions_mm,
                _orientation: rot,
            },
        ));
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sensors(
    mut sensors: Query<(&mut Transform, &Sensor)>,
    sample_tracker: Res<SampleTracker>,
) {
    if sample_tracker.is_changed() {
        let beat_index = sample_tracker.selected_beat;
        sensors.par_iter_mut().for_each(|(mut transform, sensor)| {
            let position = Vec3 {
                x: sensor.positions_mm[(beat_index, 0)],
                y: sensor.positions_mm[(beat_index, 1)],
                z: sensor.positions_mm[(beat_index, 2)],
            };
            transform.translation = position;
        });
    }
}

#[derive(Component)]
/// A struct representing a sensor bracket in the visualization.
///
/// The sensor bracket has a radius, an array of positions, and an offset all given in mm.
pub(crate) struct SensorBracket {
    pub radius: f32,
    pub positions: Array2<f32>,
    pub offset: Vec3,
}

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn spawn_sensor_bracket(
    _ass: &Res<AssetServer>,
    _commands: &mut Commands,
    scenario: &Scenario,
) {
    let sensors = &scenario
        .data
        .as_ref()
        .unwrap()
        .simulation
        .model
        .spatial_description
        .sensors;
    #[allow(clippy::no_effect_underscore_binding)]
    let _radius = sensors.array_radius_mm;
    let motion_steps = sensors.array_offsets_mm.shape()[0];

    let mut positions = Array2::zeros((motion_steps, 3));
    for i in 0..motion_steps {
        positions[(i, 0)] = sensors.array_center_mm[0] + sensors.array_offsets_mm[(i, 0)];
        positions[(i, 1)] = sensors.array_center_mm[1] + sensors.array_offsets_mm[(i, 1)];
        positions[(i, 2)] = sensors.array_center_mm[2] + sensors.array_offsets_mm[(i, 2)];
    }

    // TODO: readd and add checkbox to enable/disable sensor bracket
    // let glb_handle = ass.load("sensor_array.glb#Scene0");

    // commands.spawn((
    //     SceneBundle {
    //         scene: glb_handle,
    //         transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::ONE * 1000.0),
    //         ..Default::default()
    //     },
    //     SensorBracket {
    //         radius,
    //         positions,
    //         offset: Vec3::ZERO,
    //     },
    // ));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sensor_bracket(
    mut sensor_brackets: Query<(&mut Transform, &SensorBracket)>,
    sample_tracker: Res<SampleTracker>,
) {
    let beat_index = sample_tracker.selected_beat;
    let sensor_bracket = sensor_brackets.get_single_mut();

    if let Ok((mut transform, sensor_bracket)) = sensor_bracket {
        let position = Vec3 {
            x: sensor_bracket.positions[(beat_index, 0)] + sensor_bracket.offset[0],
            y: sensor_bracket.positions[(beat_index, 1)] + sensor_bracket.offset[1],
            z: sensor_bracket.positions[(beat_index, 2)] + sensor_bracket.offset[2],
        };
        transform.translation = position;
        // has to be multiplied by 2.5 to get the correct scale (due to blender model size)
        transform.scale.x = sensor_bracket.radius * 2.5;
        transform.scale.z = sensor_bracket.radius * 2.5;
    }
}
