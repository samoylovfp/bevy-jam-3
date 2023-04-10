use bevy::{
    prelude::{
        default, AssetServer, BuildChildren, Color, Commands, Component, DespawnRecursiveExt,
        Entity, ImageBundle, NodeBundle, Query, Res, TextBundle, Transform, With, Without,
    },
    text::{TextAlignment, TextStyle},
    ui::{AlignItems, JustifyContent, PositionType, Size, Style, UiImage, UiRect, Val},
};

use crate::PlayerBody;

#[derive(Component)]
pub struct Hud;

#[derive(Component)]
pub struct BodyIcon;

#[derive(Component)]
pub struct Subtitle;

pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(100.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            Hud,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "subtitle",
                    TextStyle {
                        font: asset_server.load("PublicPixel-z84yD.ttf"),
                        font_size: 15.0,
                        color: Color::WHITE,
                    },
                )
                .with_text_alignment(TextAlignment::Center)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(5.0),
                        ..default()
                    },
                    max_size: Size {
                        width: Val::Px(1000.),
                        height: Val::Undefined,
                    },
                    ..default()
                }),
                Subtitle,
            ));
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(200.0),
                            width: Val::Px(100.0),
                        },
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        ImageBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(100.0),
                                    width: Val::Px(50.0),
                                },
                                ..default()
                            },
                            image: UiImage {
                                texture: asset_server.load("body_icon.png"),
                                ..default()
                            },
                            ..default()
                        },
                        BodyIcon,
                    ));
                });
        });
}

pub fn despawn_hud(mut commands: Commands, hud: Query<Entity, With<Hud>>) {
    commands.entity(hud.single()).despawn_recursive();
}

pub fn update_body_icon(
    mut body_icon: Query<&mut Transform, (With<BodyIcon>, Without<PlayerBody>)>,
    player: Query<&Transform, (With<PlayerBody>, Without<BodyIcon>)>,
) {
    body_icon.single_mut().scale = player.single().scale;
}

pub fn update_subtitle() {}
