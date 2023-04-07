//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{gltf::GltfExtras, prelude::*};
use serde::Deserialize;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransform, LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_startup_system(setup_cam)
        .add_startup_system(spawn_gltf)
        .add_system(apply_gltf_extras)
        .run();
}

fn setup_cam(mut cmd: Commands) {
    cmd.spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                smoothing_weight: 0.0,
                ..default()
            },
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(-1., 1., 1.),
            Vec3::Y,
        ));
}

fn spawn_gltf(mut commands: Commands, ass: Res<AssetServer>) {
    // note that we have to include the `Scene0` label
    let my_gltf = ass.load("bvj-3-lib.gltf#Scene0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    commands.spawn(SceneBundle {
        scene: my_gltf,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

#[derive(Deserialize)]
struct NodeMeta {
    role: String,
}

fn apply_gltf_extras(
    mut cmd: Commands,
    gltf_extras: Query<(Entity, &GltfExtras, &Transform), Without<Camera>>,
    mut cam: Query<&mut LookTransform, With<Camera>>,
) {
    for (ent, gltf_extras, transform) in gltf_extras.iter() {
        let meta: NodeMeta = serde_json::from_str(&gltf_extras.value).unwrap();
        match meta.role.as_str() {
            "PlayerSpawn" => cam.single_mut().eye = transform.translation,
            "PlayerSpawnLookAt" => cam.single_mut().target = transform.translation,
            r => panic!("Unknown role {r}"),
        }
        cmd.entity(ent).despawn_recursive()
    }
}
