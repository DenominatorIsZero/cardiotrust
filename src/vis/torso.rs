use std::f32::consts::PI;

use bevy::prelude::*;

use super::options::VisibilityOptions;

#[derive(Component)]
pub struct Torso;

/// Spawns a 3D torso mesh into the scene using the given `AssetServer` and
/// materials. The mesh is loaded from the "torso.glb" file, rotated, scaled,
/// and translated. A PBR material with a transparent color is created and
/// applied to the mesh.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(skip(commands, materials, ass), level = "debug")]
pub(crate) fn spawn_torso(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    debug!("Running system to spawn torso.");
    let my_mesh: Handle<Mesh> = ass.load("torso.glb#Mesh0/Primitive0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundlex
    commands.spawn((
        Mesh3d(my_mesh),
        // Notice how there is no need to set the `alpha_mode` explicitly here.
        // When converting a color to a material using `into()`, the alpha mode is
        // automatically set to `Blend` if the alpha channel is anything lower than 1.0.
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgba(
            85.0 / 255.0,
            79.0 / 255.0,
            72.0 / 255.0,
            0.5,
        )))),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::ONE * 1000.0)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, PI)),
        Torso,
    ));
}

#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn update_torso_visibility(
    mut room: Query<&mut Visibility, With<Torso>>,
    options: Res<VisibilityOptions>,
) {
    if options.is_changed() {
        for mut visibility in &mut room {
            if options.torso {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
