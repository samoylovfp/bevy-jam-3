use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_kira_audio::{AudioChannel, AudioControl};
use bevy_rapier3d::prelude::{CollisionEvent, ExternalImpulse, Velocity};

use crate::{
    audio::{SpawnRoomSpeaker, AUDIO_FILES},
    menu::{GameTrigger, ShowOn},
    post_processing::GameCamera,
    AppState, CameraMenu, PlayerBody, PlayerHead, PlayerLegs, PlayerSpawn,
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

#[derive(Resource, PartialEq, Eq)]
pub enum GameState {
    JustSpawned,
    InTestingRoom,
    TurnOnLaser1,
    Laser1EffectDiscussion,
    TurnOnLaser2
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
    pub height_timer: Timer,
    pub width_timer: Timer,
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

#[derive(Component, PartialEq)]
pub enum LaserTrigger {
    Height,
    Width,
}

pub fn change_size(
    keyboard: Res<Input<KeyCode>>,
    mut body: Query<(&mut Transform, &mut PlayerEffects), With<PlayerBody>>,
    time: Res<Time>,
    mut events: EventReader<LaserTrigger>,
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

    effects.height_timer.tick(time.delta());
    effects.width_timer.tick(time.delta());

    if effects.height_timer.just_finished() {
        effects.height_state = GrowthState::Increasing;
    }
    if effects.width_timer.just_finished() {
        effects.width_state = GrowthState::Increasing;
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
    for event in events.iter() {
        if *event == LaserTrigger::Height {
            effects.height_state = GrowthState::Decreasing;
            effects.height_timer.reset();
            effects.height_timer.unpause();
        } else if *event == LaserTrigger::Width {
            effects.width_state = GrowthState::Decreasing;
            effects.width_timer.reset();
            effects.width_timer.unpause();
        }
    }
    if keyboard.pressed(KeyCode::B) && effects.height_state == GrowthState::Big {
        effects.height_state = GrowthState::Decreasing;
        effects.height_timer.reset();
        effects.height_timer.unpause();
    }
    if keyboard.pressed(KeyCode::N) && effects.width_state == GrowthState::Big {
        effects.width_state = GrowthState::Decreasing;
        effects.width_timer.reset();
        effects.width_timer.unpause();
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

pub(crate) fn check_triggers(
    mut collision_events: EventReader<CollisionEvent>,
    player: Query<Entity, With<PlayerBody>>,
    triggers: Query<&GameTrigger>,
    mut trigger_events: EventWriter<GameTrigger>,
) {
    let player = player.single();
    for event in collision_events.iter() {
        match event {
            CollisionEvent::Started(e1, e2, _) if (e1 == &player || e2 == &player) => {
                let trigger = triggers.get(*e1).or(triggers.get(*e2));
                if let Ok(t) = trigger {
                    trigger_events.send(t.clone())
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn process_triggers(
    mut cmd: Commands,
    mut events: EventReader<GameTrigger>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    mut laser_event: EventWriter<LaserTrigger>,
    audio_channel: Res<AudioChannel<SpawnRoomSpeaker>>,
    asset_server: Res<AssetServer>,
    triggers: Query<(Entity, &GameTrigger)>,
) {
    for event in events.iter() {
        match event {
            GameTrigger::ExitLevel => next_state.set(AppState::Finish),
            GameTrigger::Sensor_04 => {
                if matches!(*game_state, GameState::JustSpawned) {
                    *game_state = GameState::InTestingRoom;
                }
            }
            GameTrigger::LaserWidth | GameTrigger::LaserWidth_04 => {
                if matches!(event, GameTrigger::LaserWidth_04)
                    && matches!(*game_state, GameState::TurnOnLaser1)
                {
                    *game_state = GameState::Laser1EffectDiscussion
                }
                match *game_state {
                    GameState::TurnOnLaser1
                    | GameState::Laser1EffectDiscussion
                    | GameState::TurnOnLaser2 => {
                        laser_event.send(LaserTrigger::Width);
                    }
                    _ => {}
                }
            }
            GameTrigger::LaserHeight | GameTrigger::LaserHeight_11 => match *game_state {
                GameState::TurnOnLaser2 => {
                    laser_event.send(LaserTrigger::Height);
                }
                _ => {}
            },
            GameTrigger::Sensor_17 => {
                audio_channel
                    .play(asset_server.load(String::from("sounds/dialogues/") + AUDIO_FILES[16]));
                for (ent, trigger) in triggers.iter() {
                    if trigger == event {
                        cmd.entity(ent).despawn_recursive()
                    }
                }
            }
            GameTrigger::Sensor_18 => {
                audio_channel
                    .play(asset_server.load(String::from("sounds/dialogues/") + AUDIO_FILES[17]));
                for (ent, trigger) in triggers.iter() {
                    if trigger == event {
                        cmd.entity(ent).despawn_recursive()
                    }
                }
            }
            GameTrigger::Sensor_19 => {
                audio_channel
                    .play(asset_server.load(String::from("sounds/dialogues/") + AUDIO_FILES[18]));
                for (ent, trigger) in triggers.iter() {
                    if trigger == event {
                        cmd.entity(ent).despawn_recursive()
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn show_lasers(mut lasers: Query<(&mut Visibility, &ShowOn)>, game_state: Res<GameState>) {
    for (mut vis, show_on) in lasers.iter_mut() {
        if *game_state == show_on.0 {
            *vis = Visibility::Visible;
        }
    }
}
