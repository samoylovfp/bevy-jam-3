use bevy::{
    prelude::{
        default, shape, Assets, Camera, Camera2dBundle, Commands, Component, Handle, Image, Mesh,
        Query, ResMut, Transform, Vec2, Vec3,
    },
    reflect::TypeUuid,
    render::{
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::{Material2d, MaterialMesh2dBundle},
    window::Window,
};

use crate::{game::PlayerEffects, RenderTargetImage};

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "bc2f08eb-a0fb-43f1-a908-54871ea597d5"]
pub(crate) struct BVJPostProcessing {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,
    #[uniform(2)]
    blur_strength: f32,
}

impl Material2d for BVJPostProcessing {
    fn fragment_shader() -> ShaderRef {
        "shaders/post.wgsl".into()
    }
}

#[derive(Component)]
pub(crate) struct GameCamera;

pub(crate) fn setup_postpro(
    mut cmd: Commands,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut post_processing_materials: ResMut<Assets<BVJPostProcessing>>,
) -> RenderTargetImage {
    let window = windows.single();

    let size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    // This specifies the layer used for the post processing camera, which will be attached to the post processing camera and 2d quad.
    let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        size.width as f32,
        size.height as f32,
    ))));

    // This material has the texture that has been rendered.
    let material_handle = post_processing_materials.add(BVJPostProcessing {
        source_image: image_handle.clone(),
        blur_strength: 0.002,
    });

    // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
    cmd.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        post_processing_pass_layer,
    ));

    // The post-processing pass camera.
    cmd.spawn((
        Camera2dBundle {
            camera: Camera {
                // renders after the first main camera which has default value: 0.
                order: 1,
                is_active: false,
                ..default()
            },
            ..Camera2dBundle::default()
        },
        post_processing_pass_layer,
        GameCamera,
    ));

    RenderTargetImage(image_handle)
}

pub(crate) fn change_blur(
    mut post_processing_materials: ResMut<Assets<BVJPostProcessing>>,
    effects: Query<&PlayerEffects>,
) {
    let eff = effects.single();
    let blur_strength = (eff.height + eff.width - 1.0) / 200.0;
    post_processing_materials
        .iter_mut()
        .next()
        .unwrap()
        .1
        .blur_strength = blur_strength;
}
