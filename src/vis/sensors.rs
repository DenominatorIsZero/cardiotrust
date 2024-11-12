use bevy::{math::vec3, prelude::*};
use ndarray::Array2;

use super::{options::VisibilityOptions, sample_tracker::SampleTracker};
use crate::core::scenario::Scenario;

#[derive(Component)]
pub(crate) struct SensorData {
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
    sensors: &Query<(Entity, &SensorData)>,
) {
    debug!("Running system to spawn sensors.");
    // despawn current sensors
    for (entity, _) in sensors.iter() {
        commands.entity(entity).despawn_recursive();
    }
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
                    .with_scale(Vec3::ONE * 15.0) // this should be a parameter, changeable via the gui...
                    .with_rotation(Quat::from_rotation_arc(-Vec3::Z, rot)),
                ..default()
            },
            SensorData {
                positions_mm,
                _orientation: rot,
            },
        ));
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sensor_positions(
    mut sensors: Query<(&mut Transform, &SensorData)>,
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
pub(crate) struct SensorBracket {}

#[derive(Resource, Debug)]
pub(crate) struct BacketSettings {
    pub radius: f32,
    pub positions: Array2<f32>,
    pub offset: Vec3,
}

impl Default for BacketSettings {
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Initializing default sensor bracket options.");
        Self {
            radius: 1.0,
            positions: Array2::zeros((1, 3)),
            offset: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn spawn_sensor_bracket(
    ass: &Res<AssetServer>,
    sensor_bracket_settings: &mut ResMut<BacketSettings>,
    commands: &mut Commands,
    scenario: &Scenario,
    brackets: &Query<(Entity, &SensorBracket)>,
) {
    for (entity, _) in brackets.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let sensors = &scenario
        .data
        .as_ref()
        .unwrap()
        .simulation
        .model
        .spatial_description
        .sensors;
    #[allow(clippy::no_effect_underscore_binding)]
    let radius = sensors.array_radius_mm;
    let motion_steps = sensors.array_offsets_mm.shape()[0];

    let mut positions = Array2::zeros((motion_steps, 3));
    for i in 0..motion_steps {
        positions[(i, 0)] = sensors.array_center_mm[0] + sensors.array_offsets_mm[(i, 0)];
        positions[(i, 1)] = sensors.array_center_mm[1] + sensors.array_offsets_mm[(i, 1)];
        positions[(i, 2)] = sensors.array_center_mm[2] + sensors.array_offsets_mm[(i, 2)];
    }

    let glb_handle = ass.load("sensor_array.glb#Scene0");

    commands.spawn((
        SceneBundle {
            scene: glb_handle,
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::ONE * 1000.0)
                .with_translation(vec3(
                    positions[(0, 0)],
                    positions[(0, 1)],
                    positions[(0, 2)],
                ))
                .with_scale(vec3(radius * 2.5, 1000.0, radius * 2.5)),
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        SensorBracket {},
    ));

    sensor_bracket_settings.radius = radius;
    sensor_bracket_settings.positions = positions;
    sensor_bracket_settings.offset = Vec3::ZERO;
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sensor_bracket_position(
    mut sensor_brackets: Query<(&mut Transform, &SensorBracket)>,
    sample_tracker: Res<SampleTracker>,
    settings: Res<BacketSettings>,
) {
    if sample_tracker.is_changed() || settings.is_changed() {
        let beat_index = sample_tracker.selected_beat;
        let sensor_bracket = sensor_brackets.get_single_mut();

        if let Ok((mut transform, _)) = sensor_bracket {
            let position = Vec3 {
                x: settings.positions[(beat_index, 0)] + settings.offset[0],
                y: settings.positions[(beat_index, 1)] + settings.offset[1],
                z: settings.positions[(beat_index, 2)] + settings.offset[2],
            };
            transform.translation = position;
            // has to be multiplied by 2.5 to get the correct scale (due to blender model size)
            transform.scale.x = settings.radius * 2.5;
            transform.scale.z = settings.radius * 2.5;
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sensor_bracket_visibility(
    mut sensor_brackets: Query<&mut Visibility, With<SensorBracket>>,
    options: Res<VisibilityOptions>,
) {
    if options.is_changed() {
        for mut visibility in &mut sensor_brackets {
            if options.sensor_bracket {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sensor_visibility(
    mut sensors: Query<&mut Visibility, With<SensorData>>,
    options: Res<VisibilityOptions>,
) {
    if options.is_changed() {
        for mut visibility in &mut sensors {
            if options.sensors {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
