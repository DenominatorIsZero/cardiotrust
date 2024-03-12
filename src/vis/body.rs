use bevy::prelude::*;
use std::f32::consts::PI;

/// Spawns a 3D torso mesh into the scene using the given `AssetServer` and
/// materials. The mesh is loaded from the "torso.glb" file, rotated, scaled,
/// and translated. A PBR material with a transparent color is created and
/// applied to the mesh.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_torso(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // note that we have to include the `Scene0` label
    let my_mesh: Handle<Mesh> = ass.load("torso.glb#Mesh0/Primitive0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundlex
    commands.spawn(PbrBundle {
        mesh: my_mesh,
        // Notice how there is no need to set the `alpha_mode` explicitly here.
        // When converting a color to a material using `into()`, the alpha mode is
        // automatically set to `Blend` if the alpha channel is anything lower than 1.0.
        material: materials.add(StandardMaterial::from(Color::rgba(
            85.0 / 255.0,
            79.0 / 255.0,
            72.0 / 255.0,
            0.25,
        ))),
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::ONE * 1000.0)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, PI / 2.0, PI, 0.0)),
        ..default()
    });
}
