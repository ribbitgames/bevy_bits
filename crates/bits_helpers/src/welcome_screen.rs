use bevy::prelude::*;

use crate::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Component)]
pub struct WelcomeScreenElement;

pub fn spawn_welcome_screen_shape(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    action: &str,
    shape: Mesh,
    shape_color: Color,
) {
    // Background
    commands.spawn((
        Sprite::from_color(Color::BLACK, Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
        WelcomeScreenElement,
    ));

    // "Mash this shape" text
    commands.spawn((
        Text::new(action),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(25.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            ..default()
        },
        WelcomeScreenElement,
    ));

    // The shape
    commands.spawn((
        Mesh2d(meshes.add(shape)),
        MeshMaterial2d(materials.add(ColorMaterial::from(shape_color))),
        Transform::from_xyz(0.0, 0.0, 1.0),
        WelcomeScreenElement,
    ));

    // "Tap to start" text
    commands.spawn((
        Text::new("Tap to start"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(25.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            ..default()
        },
        WelcomeScreenElement,
    ));
}

pub fn despawn_welcome_screen(
    mut commands: Commands,
    welcome_elements: Query<Entity, With<WelcomeScreenElement>>,
) {
    for entity in welcome_elements.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
