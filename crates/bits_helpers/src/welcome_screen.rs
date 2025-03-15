use bevy::prelude::*;

use crate::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Component)]
pub struct WelcomeScreenElement;

pub fn spawn_welcome_screen(mut commands: Commands, asset_server: Res<AssetServer>, action: &str) {
    // Background
    commands.spawn((
        Sprite::from_color(Color::BLACK, Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
        WelcomeScreenElement,
    ));

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
