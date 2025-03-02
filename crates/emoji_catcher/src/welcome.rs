use bevy::color::palettes::css::GREEN;
use bevy::prelude::*;
use bits_helpers::{FONT, WINDOW_HEIGHT, emoji};

use crate::core::{GameState, TargetEmojiIndex};

/// Component marker for welcome screen entities.
#[derive(Component)]
pub struct WelcomeScreen;

/// Component that marks a welcome screen waiting for emoji system
#[derive(Component)]
pub struct WelcomeWaitingForEmoji;

/// Spawns the welcome screen base structure without emojis.
pub fn spawn_welcome_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the welcome screen parent entity.
    let welcome_entity = commands
        .spawn((
            WelcomeScreen,
            WelcomeWaitingForEmoji,
            Transform::default(),
            Visibility::Visible,
        ))
        .id();

    // Spawn child text elements attached to the welcome screen.
    commands.entity(welcome_entity).with_children(|parent| {
        // Spawn title.
        parent.spawn((
            Text2d::new("Emoji Catcher"),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 40.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(0.0, WINDOW_HEIGHT / 4.0, 0.0)),
        ));

        // Spawn "Catch these:" instruction.
        parent.spawn((
            Text2d::new("Catch these:"),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 24.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::Srgba(GREEN)),
            Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
        ));

        // Spawn "Tap to Start" instruction.
        parent.spawn((
            Text2d::new("Tap to Start"),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 32.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 4.0, 0.0)),
        ));
    });
}

/// Attempts to add the emoji to welcome screens that are waiting for it.
pub fn add_emoji_to_welcome_screen(
    mut commands: Commands,
    atlas: Res<emoji::EmojiAtlas>,
    validation: Res<emoji::AtlasValidation>,
    query: Query<Entity, With<WelcomeWaitingForEmoji>>,
    mut target_emoji: ResMut<TargetEmojiIndex>,
) {
    // Check if emoji system is ready
    if !emoji::is_emoji_system_ready(&validation) {
        return;
    }

    // Don't proceed if we've already set a target emoji
    if target_emoji.0.is_some() {
        // Just remove the waiting component from any entities
        for welcome_entity in &query {
            commands
                .entity(welcome_entity)
                .remove::<WelcomeWaitingForEmoji>();
        }
        return;
    }

    // Get a random emoji index for demonstration
    let emoji_indices = emoji::get_random_emojis(&atlas, &validation, 1);
    if emoji_indices.is_empty() {
        return;
    }

    // Store the target emoji index for gameplay
    if let Some(&index) = emoji_indices.first() {
        target_emoji.0 = Some(index);
    }

    // Process each waiting welcome screen
    for welcome_entity in &query {
        if let Some(index) = emoji_indices.first() {
            if let Some(emoji_entity) = emoji::spawn_emoji(
                &mut commands,
                &atlas,
                &validation,
                *index,
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.0)),
            ) {
                // Attach the emoji as a child of the welcome screen entity
                commands
                    .entity(welcome_entity)
                    .add_children(&[emoji_entity]);
            }
        }

        // Remove the waiting component since we've processed this entity
        commands
            .entity(welcome_entity)
            .remove::<WelcomeWaitingForEmoji>();
    }
}

/// Handles input on the welcome screen.
pub fn handle_welcome_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    waiting_query: Query<(), With<WelcomeWaitingForEmoji>>,
    target_emoji: Res<TargetEmojiIndex>,
) {
    // Don't allow input if welcome screen is still waiting for emoji
    if !waiting_query.is_empty() {
        return;
    }

    // Don't allow input if target emoji hasn't been set
    if target_emoji.0.is_none() {
        return;
    }

    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

/// Cleans up the welcome screen by despawning its entities.
pub fn despawn_welcome_screen(mut commands: Commands, query: Query<Entity, With<WelcomeScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
