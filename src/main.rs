//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::io::Cursor;

use bevy::{
    prelude::*,
    render::{render_resource::Extent3d, texture::ImageSampler},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    include_flate::flate!(static BURN_TEX: [u8] from "assets/burn.png");
    let img = image::io::Reader::new(Cursor::new(BURN_TEX.as_slice()))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    let mut img = Image::from_dynamic(img, false);
    img.sampler_descriptor = ImageSampler::nearest();
    let material = materials.add(images.add(img).into());

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material,
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
