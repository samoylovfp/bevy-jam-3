//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::utils::HashSet;

use bevy::{
    gltf::{GltfExtras, GltfMesh, GltfNode},
    prelude::*,
};
use bevy_rapier3d::prelude::{
    Collider, ComputedColliderShape, NoUserData, RapierPhysicsPlugin, RigidBody,
};
use bevy_rapier3d::render::RapierDebugRenderPlugin;
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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(CollidersLoaded(false))
        .add_startup_system(setup_cam)
        .add_startup_system(spawn_gltf)
        .add_system(apply_gltf_extras.in_base_set(CoreSet::PreUpdate))
        .add_system(create_colliders.in_base_set(CoreSet::Update))
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

fn spawn_gltf(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // note that we have to include the `Scene0` label
    let my_gltf = ass.load("bvj-3-lib.gltf#Scene0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    commands.spawn(SceneBundle {
        scene: my_gltf,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // Physics test cube
    let cube_mesh = Mesh::from(shape::Cube { size: 0.2 });
    let collider = Collider::from_bevy_mesh(&cube_mesh, &default()).unwrap();
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(cube_mesh),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(1.0, 1.0, -1.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(collider);
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

#[derive(Resource)]
struct CollidersLoaded(bool);

fn create_colliders(
    mut cmd: Commands,
    mut loaded: ResMut<CollidersLoaded>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    bevy_meshes: Res<Assets<Mesh>>,
    entities_with_meshes: Query<(Entity, &Handle<Mesh>), Without<Collider>>,
) {
    // FIXME: change into a system set run condition
    if loaded.0 {
        return;
    }

    if gltf_meshes.iter().next().is_none() {
        // so we do not mark as loaded before we have any meshes
        return;
    }

    let mut meshes_came_from_gltf = HashSet::new();
    for (_, gltf_mesh) in gltf_meshes.iter() {
        for prim in &gltf_mesh.primitives {
            meshes_came_from_gltf.insert(prim.mesh.clone());
        }
    }
    let mut colliders = 0;

    for (ent, mesh_id) in entities_with_meshes.iter() {
        if meshes_came_from_gltf.contains(mesh_id) {
            let mesh = bevy_meshes.get(mesh_id).unwrap();
            let collider =
                Collider::from_bevy_mesh(mesh, &ComputedColliderShape::default()).unwrap();
            cmd.entity(ent).insert(RigidBody::Fixed).insert(collider);
            colliders += 1;
        }
    }
    println!(
        "Created {colliders} colliders, {} meshes from gltf",
        meshes_came_from_gltf.len()
    );
    if colliders > 0 {
        loaded.0 = true;
    }
}
