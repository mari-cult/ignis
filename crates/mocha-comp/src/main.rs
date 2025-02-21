use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::{
    asset::RenderAssetUsages,
    render::{
        camera::RenderTarget,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    },
};
use bevy_compositor::state::{MainCamera, MainTexture};
use bevy_compositor::SmithayPlugin;
use rand::seq::IndexedRandom;
use std::f32;
use unicode_segmentation::UnicodeSegmentation;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins((DefaultPlugins, SmithayPlugin))
        .add_systems(Update, setup.run_if(resource_added::<MainTexture>))
        .run();
}

fn setup(
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
    main_texture: Res<MainTexture>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = window.single();
    let width = window.resolution.physical_width() as f32;
    let height = window.resolution.physical_height() as f32;

    let orange = Color::srgb_u8(0xF5, 0x54, 0x42);
    let orange = Color::srgb_u8(0xE7, 0xCF, 0xAB);
    let orange = Color::srgb_u8(0xFF, 0x00, 0x00);

    let roboto_medium = asset_server.load("fonts/Roboto-Medium.ttf");
    let roboto_semibold = asset_server.load("fonts/Roboto-SemiBold.ttf");

    let text = include_str!("background.txt").to_lowercase();
    let words: Vec<_> = text.unicode_words().collect();
    let words = words.repeat(6000 / words.len());
    let words: Vec<_> = words
        .choose_multiple(&mut rand::rng(), 6000)
        .copied()
        .collect();

    let text = words.join(" ");

    let width = 2560.0;
    let height = 2560.0;

    let length = (width + height) * (f32::consts::SQRT_2 / 2.0);

    let size = Extent3d {
        width: length as u32,
        height: length as u32,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

    let image = images.add(image);

    let camera = commands
        .spawn((
            Camera {
                order: -1,
                target: RenderTarget::Image(image.clone()),
                ..default()
            },
            Camera3d::default(),
        ))
        .id();

    commands
        .spawn((
            BackgroundColor(orange),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            TargetCamera(camera),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new(text),
                TextColor(Color::BLACK.with_alpha(0.7)),
                TextFont::from_font(roboto_semibold.clone()).with_font_size(24.0),
                TextLayout::new_with_justify(JustifyText::Justified),
                TargetCamera(camera),
            ));
        });

    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                target: main_texture.0.clone(),
                ..default()
            },
            MainCamera,
        ))
        .id();

    commands.spawn((
        Node {
            width: Val::Px(length),
            height: Val::Px(length),
            left: Val::Px(-1440.0 / 2.0),
            top: Val::Px(-1440.0 / 2.0),
            ..default()
        },
        ImageNode::from(image),
        Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            0.0,
            0.0,
            -27.0_f32.to_radians(),
        )),
        TargetCamera(camera),
        ZIndex(-1),
    ));

    commands
        .spawn((
            Node {
                display: Display::Grid,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            TargetCamera(camera),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    BackgroundColor(Color::srgb_u8(0x12, 0x12, 0x12).with_alpha(0.95)),
                    BorderRadius::all(Val::Px(20.0)),
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    TargetCamera(camera),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("title"),
                        TextColor(Color::WHITE),
                        TextFont::from_font(roboto_semibold.clone()).with_font_size(40.0),
                        TextLayout::new_with_justify(JustifyText::Justified),
                    ));

                    builder.spawn((
                        Text::new("body fr fr on god no cap"),
                        TextColor(Color::WHITE),
                        TextFont::from_font(roboto_medium.clone()).with_font_size(22.0),
                        TextLayout::new_with_justify(JustifyText::Justified),
                    ));
                });
        });
}
