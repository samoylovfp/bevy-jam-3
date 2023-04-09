// Unfortunately component queries are pretty involved
// make clippy not worry about them
#![allow(clippy::type_complexity)]

//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::input::mouse::MouseMotion;
use bevy::utils::HashSet;

use bevy::window::CursorGrabMode;
use bevy::window::Window;
use bevy::{
    gltf::{GltfExtras, GltfMesh},
    prelude::*,
};

use bevy_rapier3d::prelude::{
    Collider, ComputedColliderShape, ExternalImpulse, LockedAxes, NoUserData, RapierPhysicsPlugin,
    RigidBody, Velocity,
};
use serde::Deserialize;

#[derive(Component)]
struct PlayerBody;

#[derive(Component)]
struct PlayerHead;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_system(grab_mouse)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(CollidersLoaded(false))
        .add_startup_system(setup_player)
        .add_startup_system(spawn_gltf)
        .add_system(apply_gltf_extras.in_base_set(CoreSet::PreUpdate))
        .add_system(create_colliders.in_base_set(CoreSet::Update))
        .add_system(movement)
        .add_system(debug_pos)
        .add_system(change_size)
        .run();
}

fn setup_player(mut cmd: Commands) {
    // For some stupid reason KinematicCharacterControl does not work without camera
    // so we add a disabled one

    // fatness
    let capsule_diameter = 0.3;
    // capsule total height
    let capsule_total_height = 1.4;

    let capsule_total_half_height = capsule_total_height / 2.0;
    let capsule_segment_half_height = capsule_total_half_height - (capsule_diameter / 2.0);
    let eyes_height = 0.93;

    cmd.spawn((
        PlayerBody,
        PlayerEffects {
            height: 1.0,
            width: 1.0,
            height_state: GrowthState::Big,
            width_state: GrowthState::Big,
        },
        Camera3dBundle {
            camera: Camera {
                is_active: false,
                ..default()
            },
            ..default()
        },
        RigidBody::Dynamic,
        ExternalImpulse::default(),
        Velocity::default(),
        Collider::capsule_y(capsule_segment_half_height, capsule_diameter / 2.0),
        LockedAxes::ROTATION_LOCKED,
    ))
    .with_children(|parent| {
        // head
        parent.spawn((
            PlayerHead,
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, capsule_total_half_height * eyes_height, 0.0),
                ..default()
            },
        ));
    });
}

fn spawn_gltf(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // note that we have to include the `Scene0` label
    let my_gltf = ass.load("bvj-3-level-7.glb#Scene0");

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
    gltf_extras: Query<(Entity, &GltfExtras, &Transform, &Children), Without<PlayerBody>>,
    mut body: Query<&mut Transform, With<PlayerBody>>,
    bevy_meshes: Res<Assets<Mesh>>,
    bevy_mesh_components: Query<&Handle<Mesh>>,
) {
    for (ent, gltf_extras, transform, ent_children) in gltf_extras.iter() {
        let meta: NodeMeta = serde_json::from_str(&gltf_extras.value).unwrap();
        info!("Found role {:?}", meta.role);

        match meta.role.as_str() {
            "PlayerSpawn" => {
                body.single_mut().translation = transform.translation;
                cmd.entity(ent).despawn_recursive()
            }
            // TODO: broken, need to change ViewDirection instead, and include body orientation
            "PlayerSpawnLookAt" => {
                let mut body_transform = body.single_mut();
                let mut target = transform.translation;
                // So we dont tilt the body
                target.y = body_transform.translation.y;
                body_transform.look_at(target, Vec3::Y);
                cmd.entity(ent).despawn_recursive()
            }
            "Collider" => {
                let mut coll_created = false;
                for child in ent_children {
                    let Ok(mesh_handle) = bevy_mesh_components.get(*child) else {continue};
                    let mesh = bevy_meshes.get(mesh_handle).unwrap();
                    dbg!(mesh.attribute(Mesh::ATTRIBUTE_POSITION));
                    let collider = Collider::from_bevy_mesh(
                        mesh,
                        &default(),
                    )
                    .unwrap();
                    cmd.spawn(dbg!((RigidBody::Fixed, collider, *transform, GlobalTransform::default())));
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

fn movement(
    keyboard: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut body: Query<
        (&Velocity, &mut Transform, &mut ExternalImpulse),
        (With<PlayerBody>, Without<PlayerHead>),
    >,
    mut head: Query<&mut Transform, (With<PlayerHead>, Without<PlayerBody>)>,
) {
    let mouse_sensitivity = 0.001;
    let mut head_transform = head.single_mut();
    let (body_vel, mut body_transform, mut body_forces) = body.single_mut();

    let input_delta: Vec2 = mouse_motion_events.into_iter().map(|e| e.delta).sum();

    body_transform.rotate_y(-input_delta.x * mouse_sensitivity);
    head_transform.rotate_local_x(-input_delta.y * mouse_sensitivity);

    let mut desired_velocity = Vec3::ZERO;
    for (key, move_direction) in [
        (KeyCode::W, body_transform.forward()),
        (KeyCode::A, body_transform.left()),
        (KeyCode::S, body_transform.back()),
        (KeyCode::D, body_transform.right()),
    ] {
        if keyboard.pressed(key) {
            desired_velocity += Vec3 {
                y: 0.0,
                ..move_direction
            };
        }
    }

    let max_speed = 5.0;
    let accel = 0.03;

    body_forces.impulse =
        (desired_velocity.normalize_or_zero() * max_speed - body_vel.linvel) * accel;
    // Forbid flying
    body_forces.impulse.y = 0.0;
}

fn debug_pos(pos: Query<(&Transform, &GlobalTransform), Or<(With<PlayerHead>, With<PlayerBody>)>>) {
    for (i, (t, g)) in pos.iter().enumerate() {
        debug!("{i} {:?} {:?}", t.translation, g.translation());
    }
}

#[derive(Component)]
struct PlayerEffects {
    height: f32,
    width: f32,
    height_state: GrowthState,
    width_state: GrowthState,
}

#[derive(PartialEq)]
enum GrowthState {
    Small,
    Big,
    Increasing,
    Decreasing,
}

fn change_size(
    keyboard: Res<Input<KeyCode>>,
    mut body: Query<(&mut Transform, &mut PlayerEffects), With<PlayerBody>>,
) {
    let (mut body, mut effects) = body.single_mut();

    match effects.height_state {
        GrowthState::Increasing => {
            body.scale.y += 0.02;
            effects.height += 0.02;
            if effects.height > 1.0 {
                body.scale.y = 1.0;
                effects.height = 1.0;
                effects.height_state = GrowthState::Big;
            }
        }
        GrowthState::Decreasing => {
            body.scale.y -= 0.02;
            effects.height -= 0.02;
            if effects.height < 0.5 {
                body.scale.y = 0.5;
                effects.height = 0.5;
                effects.height_state = GrowthState::Small;
            }
        }
        _ => (),
    }

    match effects.width_state {
        GrowthState::Increasing => {
            body.scale.x += 0.02;
            effects.width += 0.02;
            if effects.width > 1.0 {
                body.scale.x = 1.0;
                effects.width = 1.0;
                effects.width_state = GrowthState::Big;
            }
        }
        GrowthState::Decreasing => {
            body.scale.x -= 0.02;
            effects.width -= 0.02;
            if effects.width < 0.5 {
                body.scale.x = 0.5;
                effects.width = 0.5;
                effects.width_state = GrowthState::Small;
            }
        }
        _ => (),
    }

    if keyboard.pressed(KeyCode::B) {
        match effects.height_state {
            GrowthState::Small => effects.height_state = GrowthState::Increasing,
            GrowthState::Big => effects.height_state = GrowthState::Decreasing,
            _ => (),
        };
    }
    if keyboard.pressed(KeyCode::N) {
        match effects.width_state {
            GrowthState::Small => effects.width_state = GrowthState::Increasing,
            GrowthState::Big => effects.width_state = GrowthState::Decreasing,
            _ => (),
        };
    }
}

// This system grabs the mouse when the left mouse button is pressed
// and releases it when the escape key is pressed
fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}
