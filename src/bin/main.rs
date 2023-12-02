#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
use bevy::{prelude::*, window::WindowMode};
use bevy_aabb_instancing::{Cuboid, CuboidMaterialId, Cuboids, VertexPullingRenderPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use rusty_cde::{scheduler::SchedulerPlugin, ui::UiPlugin, ScenarioList, SelectedSenario};

fn main() {
    App::new()
        .init_resource::<ScenarioList>()
        .init_resource::<SelectedSenario>()
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty CDE".into(),
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiPlugin)
        .add_plugins(SchedulerPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(VertexPullingRenderPlugin { outlines: true })
        .add_systems(Startup, setup)
        .add_systems(Update, update_cuboids_colors)
        .run();
}

#[derive(Component)]
struct Indices {
    color_mult: f32,
}

pub fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
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

    const PATCHES_PER_DIM: usize = 20;
    const PATCH_SIZE: usize = 15;
    const SCENE_RADIUS: f32 = 150.0;

    for x_batch in 0..PATCHES_PER_DIM {
        for z_batch in 0..PATCHES_PER_DIM {
            let mut instances = Vec::with_capacity(PATCH_SIZE * PATCH_SIZE);
            for x in 0..PATCH_SIZE {
                for z in 0..PATCH_SIZE {
                    let x = (x_batch * PATCH_SIZE) as f32 + x as f32 - SCENE_RADIUS;
                    let z = (z_batch * PATCH_SIZE) as f32 + z as f32 - SCENE_RADIUS;
                    let d = (x * x + z * z).sqrt();
                    let amp = 0.2 * d;
                    let y = amp * ((0.05 * x).cos() * (0.05 * z).sin());
                    let c = Vec3::new(x, y, z);
                    let h = 0.01 * d;
                    let min = c - Vec3::new(0.5, h, 0.5);
                    let max = c + Vec3::new(0.5, h, 0.5);
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

#[allow(dead_code)]
fn update_cuboids_colors(time: Res<Time>, mut query: Query<(&mut Cuboids, &Indices)>) {
    query.par_iter_mut().for_each_mut(|(mut cuboids, index)| {
        cuboids.instances.iter_mut().for_each(|instance| {
            // instance.color = Color::as_rgba_u32(Color::Rgba {
            //     red: ((time.elapsed_seconds() * 130.0 + instance.minimum.x) / 7.0).sin()
            //         + index.color_mult,
            //     green: ((time.elapsed_seconds() * 70.0 + instance.minimum.z) / 13.0).sin()
            //         + index.color_mult,
            //     blue: ((time.elapsed_seconds() * 30.0 + instance.minimum.y) / 5.0).sin()
            //         + index.color_mult,
            //     alpha: 1.0,
            // })
            instance.color = Color::as_rgba_u32(Color::Rgba {
                red: index.color_mult,
                blue: index.color_mult,
                green: index.color_mult,
                alpha: 1.0,
            });
        });
    });
}
