// Unfortunately component queries are pretty involved
// make clippy not worry about them
#![allow(clippy::type_complexity)]

//! A simple 3D scene with light shining over a cube sitting on a plane.

mod audio;
mod finish;
mod game;
mod hud;
mod menu;
mod post_processing;

use audio::AudioPlugin;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::sprite::Material2dPlugin;
use bevy::window::CursorGrabMode;
use bevy::window::Window;

use bevy_rapier3d::prelude::ActiveEvents;
use bevy_rapier3d::prelude::Sensor;
use bevy_rapier3d::prelude::{
    Collider, ExternalImpulse, LockedAxes, NoUserData, RapierPhysicsPlugin, RigidBody, Velocity,
};
use game::LaserTrigger;
use game::check_triggers;
use game::GameState;
use game::GrowthState;
use game::PlayerEffects;
use menu::GameTrigger;
use post_processing::setup_postpro;
use post_processing::BVJPostProcessing;
use post_processing::GameCamera;
use serde::Deserialize;

/// Move and yaw
#[derive(Component)]
pub struct PlayerBody;

/// Pitch
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

/// Check if touching the floor
#[derive(Component)]
struct PlayerLegs {
    touching_objects: usize,
}

#[derive(Component)]
pub struct CameraMenu;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Material2dPlugin::<BVJPostProcessing>::default())
        .add_startup_system(setup_postpro.pipe(setup_player))
        .add_state::<AppState>()
        .add_system(grab_mouse)
        .add_system(check_triggers)
        .add_event::<GameTrigger>()
		.add_event::<LaserTrigger>()
        .add_event::<hud::SubtitleTrigger>()
        .add_plugin(AudioPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(GameState::JustSpawned)
        .insert_resource(CollidersLoaded(false))
        .add_startup_system(spawn_gltf)
        .add_startup_system(spawn_menu_camera)
        .add_system(menu::apply_gltf_extras.in_base_set(CoreSet::PreUpdate))
        .add_system(menu::activate_menu_camera.in_schedule(OnEnter(AppState::Menu)))
        .add_system(menu::spawn_menu_screen.in_schedule(OnEnter(AppState::Menu)))
        .add_systems((menu::create_colliders, menu::start_game).in_set(OnUpdate(AppState::Menu)))
        .add_system(game::activate_game_camera.in_schedule(OnEnter(AppState::InGame)))
        .add_system(game::spawn_player.in_schedule(OnEnter(AppState::InGame)))
        .add_system(hud::spawn_hud.in_schedule(OnEnter(AppState::InGame)))
        .add_systems(
            (
                game::touch_ground,
                game::movement,
                game::debug_pos,
                game::change_size,
                game::back_to_menu,
                game::process_triggers,
                post_processing::change_blur,
                hud::update_body_icon,
                hud::update_subtitle,
            )
                .in_set(OnUpdate(AppState::InGame)),
        )
        .add_system(hud::despawn_hud.in_schedule(OnExit(AppState::InGame)))
        .add_system(menu::activate_menu_camera.in_schedule(OnEnter(AppState::Finish)))
        .add_system(finish::spawn_finish_screen.in_schedule(OnEnter(AppState::Finish)))
        .add_system(finish::restart.in_set(OnUpdate(AppState::Finish)))
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn spawn_menu_camera(mut cmd: Commands) {
    cmd.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        CameraMenu,
    ));
}

#[derive(Resource)]
struct RenderTargetImage(Handle<Image>);

fn setup_player(In(RenderTargetImage(render_target)): In<RenderTargetImage>, mut cmd: Commands) {
    // For some stupid reason KinematicCharacterControl does not work without camera
    // so we add a disabled one

    // fatness
    let capsule_diameter = 0.3;
    // capsule total height
    let capsule_total_height = 1.4;

    let capsule_total_half_height = capsule_total_height / 2.0;
    let capsule_segment_half_height = capsule_total_half_height - (capsule_diameter / 2.0);
    let eyes_height = 0.93;
    // so the player cannot climb walls
    let leg_with_ratio = 0.9;
    // so legs keep in contact with surface while skipping
    let leg_down_margin = 0.1;

	let mut size_timer = Timer::from_seconds(2.0, TimerMode::Once);
	size_timer.pause();

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
			height_timer: size_timer.clone(),
			width_timer: size_timer,
            height_state: GrowthState::Big,
            width_state: GrowthState::Big,
        },
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
            GameCamera,
            Camera3dBundle {
                camera: Camera {
                    is_active: false,
                    target: RenderTarget::Image(render_target),
                    ..default()
                },

                transform: Transform::from_xyz(0.0, capsule_total_half_height * eyes_height, 0.0),
                ..default()
            },
            UiCameraConfig { show_ui: false },
        ));
        // legs
        parent.spawn((
            PlayerLegs {
                touching_objects: 0,
            },
            Collider::ball(capsule_diameter / 2.0 * leg_with_ratio),
            Transform::from_xyz(0.0, -capsule_total_half_height - leg_down_margin, 0.0),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
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
