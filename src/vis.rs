pub mod cutting_plane;
pub mod heart;
pub mod options;
pub mod plotting;
pub mod room;
pub mod sample_tracker;
pub mod sensors;
pub mod torso;

use bevy::{
    color::palettes::css::{BLUE, GREEN, RED},
    prelude::*,
};
use bevy_egui::EguiStartupSet;
use bevy_editor_cam::controller::component::{EditorCam, OrbitConstraint};
use bevy_obj::ObjPlugin;
use cutting_plane::update_cutting_plane_visibility;
use heart::VoxelData;
use options::VisibilityOptions;
use room::{spawn_room, update_room_visibility};
use sensors::{
    update_sensor_bracket_visibility, update_sensor_visibility, BacketSettings, SensorBracket,
    SensorData,
};
use torso::update_torso_visibility;

use self::{
    heart::{
        init_voxels, on_color_mode_changed, update_heart_voxel_colors, MaterialAtlas, MeshAtlas,
    },
    options::ColorOptions,
    sample_tracker::{init_sample_tracker, update_sample_index, SampleTracker},
    sensors::spawn_sensors,
    torso::spawn_torso,
};
use crate::{
    core::scenario::Scenario,
    ui::UiState,
    vis::{
        cutting_plane::{spawn_cutting_plane, update_cutting_plane_position},
        heart::{setup_material_atlas, setup_mesh_atlas, update_heart_voxel_visibility},
        sensors::{spawn_sensor_bracket, update_sensor_bracket_position, update_sensor_positions},
    },
};

#[derive(Event)]
pub struct SetupHeartAndSensors(pub Scenario);

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct VisPlugin;

impl Plugin for VisPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        info!("Initializing visualization plugin.");
        app // DefaultPickingPlugins are already included in DefaultPlugins in Bevy 0.16
            .add_plugins(bevy_editor_cam::DefaultEditorCamPlugins)
            .add_plugins(ObjPlugin)
            .init_resource::<SampleTracker>()
            .init_resource::<ColorOptions>()
            .init_resource::<VisibilityOptions>()
            .init_resource::<BacketSettings>()
            .add_event::<SetupHeartAndSensors>()
            .add_systems(
                PreStartup,
                setup_light_and_camera.before(EguiStartupSet::InitContexts),
            )
            .add_systems(
                Startup,
                (
                    setup_material_atlas,
                    setup_coordinate_system,
                    setup_mesh_atlas,
                    spawn_torso,
                    spawn_room,
                    spawn_cutting_plane,
                ),
            )
            .add_systems(
                Update,
                (
                    update_cutting_plane_position,
                    update_sensor_positions,
                    update_sensor_visibility,
                    update_sensor_bracket_position,
                    update_sensor_bracket_visibility,
                    update_cutting_plane_visibility,
                    update_torso_visibility,
                    update_room_visibility,
                    update_sample_index,
                    on_color_mode_changed,
                    handle_setup_heart_and_sensors,
                )
                    .run_if(in_state(UiState::Volumetric)),
            )
            .add_systems(
                Update,
                (update_heart_voxel_colors, update_heart_voxel_visibility)
                    .run_if(in_state(UiState::Volumetric))
                    .after(update_sample_index),
            );
    }
}

/// Creates an ambient light to illuminate the full scene.
/// Spawns a camera entity with default pan/orbit controls.
#[tracing::instrument(level = "info", skip_all)]
pub fn setup_light_and_camera(mut commands: Commands) {
    info!("Setting up light and camera.");
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: false,
    });

    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(-100.0, 200.0, 50.0).looking_at(Vec3::ZERO, Vec3::Z),
        ))
        .insert(EditorCam {
            orbit_constraint: OrbitConstraint::Free,
            last_anchor_depth: 2.0,
            ..default()
        });
}

#[tracing::instrument(level = "info", skip_all)]
pub fn setup_coordinate_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    info!("Setting up coordinate system");
    if false {
        spawn_axis(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::X,
            RED.into(),
        );
        spawn_axis(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::Y,
            GREEN.into(),
        );
        spawn_axis(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::Z,
            BLUE.into(),
        );
    }
}

#[tracing::instrument(level = "info", skip_all)]
fn spawn_axis(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    direction: Vec3,
    color: Color,
) {
    let axis_length = 400.0;
    let thickness = 10.0;

    // Shaft
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cylinder {
            radius: thickness,
            half_height: axis_length / 2.0,
        }))),
        MeshMaterial3d(materials.add(StandardMaterial::from(color))),
        Transform {
            translation: direction * (axis_length / 2.0),
            rotation: Quat::from_rotation_arc(Vec3::Y, direction),
            ..default()
        },
    ));
}

/// Sets up the heart mesh, voxel grid, and sensor transforms according
/// to the provided scenario. Initializes the sample tracker based on the
/// scenario as well.
#[allow(clippy::cast_precision_loss, clippy::too_many_arguments)]
#[tracing::instrument(level = "info", skip_all)]
pub fn handle_setup_heart_and_sensors(
    mut ev_setup: EventReader<SetupHeartAndSensors>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sample_tracker: ResMut<SampleTracker>,
    mut sensor_bracket_settings: ResMut<BacketSettings>,
    mut mesh_atlas: ResMut<MeshAtlas>,
    material_atlas: Res<MaterialAtlas>,
    ass: Res<AssetServer>,
    sensors: Query<(Entity, &SensorData)>,
    voxels: Query<(Entity, &VoxelData)>,
    brackets: Query<(Entity, &SensorBracket)>,
) {
    for SetupHeartAndSensors(scenario) in ev_setup.read() {
        info!("Setting up heart and sensors.");
        init_sample_tracker(&mut sample_tracker, scenario);
        spawn_sensors(&mut commands, &ass, &mut materials, scenario, &sensors);
        spawn_sensor_bracket(
            &ass,
            &mut sensor_bracket_settings,
            &mut commands,
            scenario,
            &brackets,
        );
        init_voxels(
            &mut commands,
            &mut meshes,
            &material_atlas,
            &mut mesh_atlas,
            scenario,
            &sample_tracker,
            &voxels,
        );
    }
}
