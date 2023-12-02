use bevy::prelude::*;
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids, VertexPullingRenderPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

mod body;
mod heart;
pub mod plotting;
mod sensors;

#[allow(clippy::module_name_repetitions)]
pub struct VisPlugin;

impl Plugin for VisPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_plugins(VertexPullingRenderPlugin { outlines: true })
            .add_systems(Startup, setup)
            .add_systems(Update, update_cuboids_colors);
    }
}
#[derive(Component)]
struct Indices {
    color_mult: f32,
}

pub fn setup(mut commands: Commands) {
    setup_light_and_camera(&mut commands);
    setup_heart(&mut commands);
}

#[allow(clippy::cast_precision_loss)]
fn setup_heart(commands: &mut Commands) {
    const PATCHES_PER_DIM: usize = 20;
    const PATCH_SIZE: usize = 15;
    const SCENE_RADIUS: f32 = 150.0;

    for x_batch in 0..PATCHES_PER_DIM {
        for z_batch in 0..PATCHES_PER_DIM {
            let mut instances = Vec::with_capacity(PATCH_SIZE * PATCH_SIZE);
            for x_index in 0..PATCH_SIZE {
                for z_index in 0..PATCH_SIZE {
                    let x_position = (x_batch * PATCH_SIZE) as f32 + x_index as f32 - SCENE_RADIUS;
                    let z_position = (z_batch * PATCH_SIZE) as f32 + z_index as f32 - SCENE_RADIUS;
                    let distance_from_origin = x_position.hypot(z_position);
                    let amplitude = 0.2 * distance_from_origin;
                    let y_position =
                        amplitude * ((0.05 * x_position).cos() * (0.05 * z_position).sin());
                    let position = Vec3::new(x_position, y_position, z_position);
                    let height = 0.01 * distance_from_origin;
                    let min = position - Vec3::new(0.5, height, 0.5);
                    let max = position + Vec3::new(0.5, height, 0.5);
                    let scalar_color = Color::as_rgba_u32(Color::Rgba {
                        red: min.x / SCENE_RADIUS,
                        green: min.z / SCENE_RADIUS,
                        blue: 0.0,
                        alpha: 1.0,
                    });
                    let mut cuboid = Cuboid::new(min, max, scalar_color);
                    cuboid.set_depth_bias(0);
                    instances.push(cuboid);
                }
            }
            let cuboids = Cuboids::new(instances);
            let aabb = cuboids.aabb();
            let indices = Indices {
                color_mult: (x_batch * PATCHES_PER_DIM + z_batch) as f32
                    / (PATCHES_PER_DIM * PATCHES_PER_DIM) as f32,
            };
            commands.spawn(SpatialBundle::default()).insert((
                cuboids,
                aabb,
                CuboidMaterialId(0),
                indices,
            ));
        }
    }
}

pub fn setup_light_and_camera(commands: &mut Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

#[allow(clippy::needless_pass_by_value)]
fn update_cuboids_colors(time: Res<Time>, mut query: Query<(&mut Cuboids, &Indices)>) {
    query.par_iter_mut().for_each_mut(|(mut cuboids, index)| {
        cuboids.instances.iter_mut().for_each(|instance| {
            instance.color = Color::as_rgba_u32(Color::Rgba {
                red: index.color_mult,
                blue: index.color_mult,
                green: index.color_mult,
                alpha: 1.0,
            });
            if (time.elapsed_seconds() * 0.1).sin().abs() > index.color_mult {
                instance.make_emissive();
            } else {
                instance.make_non_emissive();
            }
        });
    });
}
