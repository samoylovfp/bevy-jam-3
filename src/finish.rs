use bevy::{
    prelude::{
        default, AssetServer, Commands, Entity, Input, KeyCode, NextState, Query, Res, ResMut, With,
    },
    sprite::SpriteBundle,
};

use crate::{menu::MenuScreen, AppState};

pub fn spawn_finish_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu_screen: Query<Entity, With<MenuScreen>>,
) {
    commands.entity(menu_screen.single()).despawn();
    commands.spawn(SpriteBundle {
        texture: asset_server.load("screens/win_screen.png"),
        ..default()
    });
}

pub fn restart(keyboard: Res<Input<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keyboard.pressed(KeyCode::Space) {
        next_state.set(AppState::InGame);
    }
}
