use bevy::prelude::*;
use bevy_compositor::state::{MainCamera, MainTexture};
use bevy_compositor::SmithayPlugin;

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
) {
    let roboto_mono = asset_server.load("fonts/RobotoMono-SemiBold.ttf");
    let text_font = TextFont::from_font(roboto_mono).with_font_size(24.0);

    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                target: main_texture.0.clone(),
                ..default()
            },
            Transform::default().looking_at(Vec3::Z * -180.0, Vec3::Y),
            MainCamera,
        ))
        .id();

    commands
        .spawn((
            BackgroundColor(Color::srgb_u8(0x12, 0x12, 0x12)),
            Node {
                display: Display::Grid,
                position_type: PositionType::Absolute,
                width: Val::Px(2560.0),
                height: Val::Px(32.0),
                margin: UiRect::horizontal(Val::Px(8.0)).with_top(Val::Px(8.0)),
                padding: UiRect::all(Val::Px(2.0)),
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            TargetCamera(camera),
        ))
        .with_children(|builder| {
            for workspace in 1..=4 {
                builder.spawn((
                    Text::new(workspace.to_string()),
                    TextColor(Color::WHITE),
                    TextFont::clone(&text_font),
                    TargetCamera(camera),
                ));
            }

            builder.spawn((
                Node {
                    grid_column: GridPlacement::start(2),
                    ..default()
                },
                Text::new("00:00"),
                TextColor(Color::WHITE),
                TextFont::clone(&text_font),
                TargetCamera(camera),
            ));
        });
}
