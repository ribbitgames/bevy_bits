use bevy::prelude::*;
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};

use crate::core::{GameState, Score, TargetEmojiIndex};

/// Component marker for game over screen entities
#[derive(Component)]
pub struct GameOverScreen;

/// Spawns the game over screen with final score
pub fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
) {
    // Create a semi-transparent overlay
    commands.spawn((
        GameOverScreen,
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.8),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Visible,
    ));

    // Game Over text
    commands.spawn((
        GameOverScreen,
        Text2d::new("Game Over!"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 48.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, WINDOW_HEIGHT / 4.0, 1.0),
    ));

    // Final score
    commands.spawn((
        GameOverScreen,
        Text2d::new(format!("Final Score: {}", score.0)),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    // Restart instruction
    commands.spawn((
        GameOverScreen,
        Text2d::new("Tap to Play Again"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -WINDOW_HEIGHT / 4.0, 1.0),
    ));
}

/// Handles input on the game over screen
pub fn handle_game_over_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut target_emoji: ResMut<TargetEmojiIndex>,
) {
    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        // Reset score and target emoji
        score.0 = 0;
        target_emoji.0 = None; // Reset the target emoji so a new one will be selected
        next_state.set(GameState::Welcome);
    }
}

/// Cleans up the game over screen
pub fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
