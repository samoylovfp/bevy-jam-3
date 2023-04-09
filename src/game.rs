use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::{CollisionEvent, ExternalImpulse, Velocity};

use crate::{
    menu::ExitLevel, post_processing::GameCamera, AppState, CameraMenu, PlayerBody, PlayerHead,
    PlayerLegs, PlayerSpawn,
};

pub(crate) fn activate_game_camera(
    mut camera_menu: Query<&mut Camera, (With<CameraMenu>, Without<GameCamera>)>,
    mut camera_player: Query<&mut Camera, (With<GameCamera>, Without<CameraMenu>)>,
) {
    camera_menu.single_mut().is_active = false;
    camera_player
        .iter_mut()
        .for_each(|mut c| c.is_active = true);
}

pub fn spawn_player(
    mut player: Query<(&mut Transform, &PlayerSpawn, &mut PlayerEffects), With<PlayerBody>>,
) {
    let (mut player, spawn, mut effects) = player.single_mut();

    player.translation = spawn.0 .0;
    let mut target = spawn.0 .1;
    // So we dont tilt the body
    target.y = player.translation.y;
    player.look_at(target, Vec3::Y);

    effects.height = 1.0;
    effects.width = 1.0;
    effects.height_state = GrowthState::Big;
    effects.width_state = GrowthState::Big;
    player.scale.x = 1.0;
    player.scale.y = 1.0;
}

pub(crate) fn movement(
    keyboard: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut body: Query<
        (&Velocity, &mut Transform, &mut ExternalImpulse),
        (With<PlayerBody>, Without<PlayerHead>),
    >,
    mut head: Query<&mut Transform, (With<PlayerHead>, Without<PlayerBody>)>,
    legs: Query<&PlayerLegs>,
) {
    let mouse_sensitivity = 0.001;
    let mut head_transform = head.single_mut();
    let (body_vel, mut body_transform, mut body_forces) = body.single_mut();
    let legs = legs.single();

    let input_delta: Vec2 = mouse_motion_events.into_iter().map(|e| e.delta).sum();

    body_transform.rotate_y(-input_delta.x * mouse_sensitivity);
    head_transform.rotate_local_x(-input_delta.y * mouse_sensitivity);

    let mut desired_velocity = Vec3::ZERO;
    for (key, move_direction) in [
        (KeyCode::W, body_transform.forward()),
        (KeyCode::A, body_transform.left()),
        (KeyCode::S, body_transform.back()),
        (KeyCode::D, body_transform.right()),
        (KeyCode::Space, Vec3::Y),
    ] {
        if keyboard.pressed(key) {
            desired_velocity += move_direction
        }
    }

    let max_speed = 5.0;
    let accel = 0.03;

    body_forces.impulse =
        (desired_velocity.normalize_or_zero() * max_speed - body_vel.linvel) * accel;
    if legs.touching_objects == 0 {
        body_forces.impulse.y = 0.0;
    }
}

pub fn debug_pos(
    pos: Query<(&Transform, &GlobalTransform), Or<(With<PlayerHead>, With<PlayerBody>)>>,
) {
    for (i, (t, g)) in pos.iter().enumerate() {
        debug!("{i} {:?} {:?}", t.translation, g.translation());
    }
}

#[derive(Component)]
pub struct PlayerEffects {
    pub height: f32,
    pub width: f32,
    pub height_state: GrowthState,
    pub width_state: GrowthState,
}

#[derive(PartialEq)]
pub enum GrowthState {
    Small,
    Big,
    Increasing,
    Decreasing,
}

pub fn change_size(
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

pub fn back_to_menu(keyboard: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keyboard.pressed(KeyCode::M) {
        next_state.set(AppState::Menu);
    }
}

pub(crate) fn touch_ground(
    mut collision_events: EventReader<CollisionEvent>,
    mut legs: Query<(Entity, &mut PlayerLegs)>,
) {
    let (legs_ent, mut legs_comp) = legs.single_mut();
    for event in collision_events.iter() {
        match event {
            CollisionEvent::Started(e1, e2, _) if e1 == &legs_ent || e2 == &legs_ent => {
                legs_comp.touching_objects += 1;
                if legs_comp.touching_objects == 1 {
                    info!("On ground!");
                }
            }
            CollisionEvent::Stopped(e1, e2, _) if e1 == &legs_ent || e2 == &legs_ent => {
                legs_comp.touching_objects -= 1;
                if legs_comp.touching_objects == 0 {
                    info!("Off ground!");
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn exit_level(
    mut collision_events: EventReader<CollisionEvent>,
    player: Query<Entity, With<PlayerBody>>,
    exit: Query<Entity, With<ExitLevel>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let player = player.single();
    let exit = exit.single();
    for event in collision_events.iter() {
        match event {
            CollisionEvent::Started(e1, e2, _)
                if (e1 == &exit || e2 == &exit) && (e1 == &player || e2 == &player) =>
            {
                next_state.set(AppState::Finish);
            }
            _ => {}
        }
    }
}
