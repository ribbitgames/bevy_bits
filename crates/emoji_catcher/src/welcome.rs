use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bits_helpers::{emoji, FONT, WINDOW_HEIGHT};

use crate::core::GameState;

/// Component marker for welcome screen entities
#[derive(Component)]
pub struct WelcomeScreen;

/// Spawns the welcome screen
pub fn spawn_welcome_screen(
    mut commands: Commands,
    atlas: Res<emoji::EmojiAtlas>,
    validation: Res<emoji::AtlasValidation>,
    asset_server: Res<AssetServer>,
) {
    if !emoji::is_emoji_system_ready(&validation) {
        return;
    }

    // Get example emojis for demonstration
    let emoji_indices = emoji::get_random_emojis(&atlas, &validation, 2);
    if emoji_indices.len() < 2 {
        return;
    }

    // Create welcome screen container
    let welcome_screen_entity = commands
        .spawn((WelcomeScreen, Transform::default(), Visibility::default()))
        .id();

    // Spawn title and instructions
    commands.spawn((
        Text2d::new("Emoji Catcher"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, WINDOW_HEIGHT / 4.0, 0.0)),
        WelcomeScreen,
    ));

    // Spawn instruction text
    commands.spawn((
        Text2d::new("Catch these:"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::Srgba(GREEN)),
        Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
        WelcomeScreen,
    ));

    // Spawn example good emoji
    if let Some(emoji_entity) = emoji::spawn_emoji(
        &mut commands,
        &atlas,
        &validation,
        emoji_indices[0],
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.0)),
    ) {
        commands.entity(emoji_entity).insert(WelcomeScreen);
    }

    // Spawn avoid text
    commands.spawn((
        Text2d::new("Avoid these:"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::Srgba(RED)),
        Transform::from_translation(Vec3::new(0.0, -50.0, 0.0)),
        WelcomeScreen,
    ));

    // Spawn example bad emoji
    if let Some(emoji_entity) = emoji::spawn_emoji(
        &mut commands,
        &atlas,
        &validation,
        emoji_indices[1],
        Transform::from_xyz(0.0, -100.0, 0.0).with_scale(Vec3::splat(1.0)),
    ) {
        commands.entity(emoji_entity).insert(WelcomeScreen);
    }

    // Spawn start instruction
    commands.spawn((
        Text2d::new("Tap to Start"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 4.0, 0.0)),
        WelcomeScreen,
    ));
}

/// Handles input on the welcome screen
pub fn handle_welcome_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

/// Cleans up the welcome screen
pub fn despawn_welcome_screen(mut commands: Commands, query: Query<Entity, With<WelcomeScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
