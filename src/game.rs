use bevy::{prelude::*, input::mouse::MouseMotion};
use bevy_rapier3d::prelude::{Velocity, ExternalImpulse};

use crate::{PlayerSpawn, PlayerBody, PlayerHead, AppState};

pub fn spawn_player(mut player: Query<(&mut Transform, &PlayerSpawn, &mut PlayerEffects), With<PlayerBody>>) {
	let (mut player, spawn, mut effects) = player.single_mut();

	player.translation = spawn.0.0;
	let mut target = spawn.0.1;
	// So we dont tilt the body
	target.y = player.translation.y;
	player.look_at(target, Vec3::Y);

	println!("move player to {}", player.translation);

	effects.height = 1.0;
	effects.width = 1.0;
	effects.height_state = GrowthState::Big;
	effects.width_state = GrowthState::Big;
}

pub fn movement(
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

pub fn debug_pos(pos: Query<(&Transform, &GlobalTransform), Or<(With<PlayerHead>, With<PlayerBody>)>>) {
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

pub fn back_to_menu(
	keyboard: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>
) {
	if keyboard.pressed(KeyCode::Space) {
		next_state.set(AppState::Menu);
	}
}
