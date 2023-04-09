use bevy::{
    gltf::{GltfExtras, GltfMesh},
    prelude::*,
    utils::HashSet,
};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, RigidBody};

use crate::{
    post_processing::GameCamera, AppState, CameraMenu, CollidersLoaded, NodeMeta, PlayerBody,
    PlayerHead, PlayerSpawn,
};

pub(crate) fn activate_menu_camera(
    mut camera_menu: Query<&mut Camera, (With<CameraMenu>, Without<GameCamera>)>,
    mut camera_player: Query<&mut Camera, (With<GameCamera>, Without<CameraMenu>)>,
) {
    camera_menu.single_mut().is_active = true;
    camera_player
        .iter_mut()
        .for_each(|mut c| c.is_active = false);
}

pub fn spawn_menu_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SpriteBundle {
        texture: asset_server.load("screens/start_screen.png"),
        ..default()
    });
}

pub fn apply_gltf_extras(
    mut cmd: Commands,
    gltf_extras: Query<(Entity, &GltfExtras, &Transform, &Children), Without<PlayerBody>>,
    mut player_spawn_info: Query<&mut PlayerSpawn, With<PlayerBody>>,
    bevy_meshes: Res<Assets<Mesh>>,
    bevy_mesh_components: Query<&Handle<Mesh>>,
) {
    for (ent, gltf_extras, transform, ent_children) in gltf_extras.iter() {
        let meta: NodeMeta = serde_json::from_str(&gltf_extras.value).unwrap();
        info!("Found role {:?}", meta.role);

        match meta.role.as_str() {
            "PlayerSpawn" => {
                player_spawn_info.single_mut().0 .0 = transform.translation;
                cmd.entity(ent).despawn_recursive()
            }

            "PlayerSpawnLookAt" => {
                player_spawn_info.single_mut().0 .1 = transform.translation;
                cmd.entity(ent).despawn_recursive()
            }
            "Collider" => {
                let mut coll_created = false;
                for child in ent_children {
                    let Ok(mesh_handle) = bevy_mesh_components.get(*child) else {continue};
                    let mesh = bevy_meshes.get(mesh_handle).unwrap();
                    let collider = Collider::from_bevy_mesh(mesh, &default()).unwrap();
                    cmd.spawn((
                        RigidBody::Fixed,
                        collider,
                        *transform,
                        GlobalTransform::default(),
                    ));
                    coll_created = true;
                }
                if coll_created {
                    cmd.entity(ent).despawn_recursive()
                }
            }
            "ExitLevel" => cmd.entity(ent).despawn_recursive(),
            r => warn!("Unknown role {r}"),
        }
    }
}

pub fn create_colliders(
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

pub fn start_game(keyboard: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keyboard.pressed(KeyCode::G) {
        next_state.set(AppState::InGame);
    }
}
