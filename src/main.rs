// Unfortunately component queries are pretty involved
// make clippy not worry about them
#![allow(clippy::type_complexity)]

//! A simple 3D scene with light shining over a cube sitting on a plane.

mod game;
mod menu;

use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy::window::Window;

use bevy_rapier3d::prelude::{
    Collider, ExternalImpulse, LockedAxes, NoUserData, RapierPhysicsPlugin, RigidBody, Velocity,
};
use game::GrowthState;
use game::PlayerEffects;
use serde::Deserialize;

#[derive(Component)]
pub struct PlayerBody;

#[derive(Component)]
pub struct PlayerHead;

#[derive(Component)]
pub struct PlayerSpawn((Vec3, Vec3));

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    Finish,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state::<AppState>()
        .add_system(grab_mouse)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(CollidersLoaded(false))
        .add_startup_system(spawn_gltf)
        .add_startup_system(setup_player)
		.add_system(menu::apply_gltf_extras.in_base_set(CoreSet::PreUpdate))
        .add_systems(
            (
                menu::create_colliders,
                menu::start_game,
            )
                .in_set(OnUpdate(AppState::Menu)),
        )
        .add_system(game::spawn_player.in_schedule(OnEnter(AppState::InGame)))
        .add_systems(
            (
                game::movement,
                game::debug_pos,
                game::change_size,
                game::back_to_menu,
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
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
        PlayerSpawn((
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        )),
        PlayerEffects {
            height: 1.0,
            width: 1.0,
            height_state: GrowthState::Big,
            width_state: GrowthState::Big,
        },
        // Camera3dBundle {
        //     camera: Camera {
        //         is_active: false,
        //         ..default()
        //     },
        //     ..default()
        // },
		TransformBundle::default(),
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

#[derive(Resource)]
pub struct CollidersLoaded(bool);

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
