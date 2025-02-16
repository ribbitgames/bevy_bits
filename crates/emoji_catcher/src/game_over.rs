use bevy::prelude::*;
use bits_helpers::{FONT, WINDOW_HEIGHT};

use crate::core::{GameState, Score};

/// Component marker for game over screen entities
#[derive(Component)]
pub struct GameOverScreen;

/// Spawns the game over screen with final score
pub fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
) {
    commands
        .spawn((
            GameOverScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            // Game Over text
            parent.spawn((
                Text2d::new("Game Over!"),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 48.0,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, WINDOW_HEIGHT / 4.0, 0.0)),
            ));

            // Final score
            parent.spawn((
                Text2d::new(format!("Final Score: {}", score.0)),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 32.0,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ));

            // Restart instruction
            parent.spawn((
                Text2d::new("Tap to Play Again"),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 24.0,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 4.0, 0.0)),
            ));
        });
}

/// Handles input on the game over screen
pub fn handle_game_over_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
) {
    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        // Reset score and return to welcome screen
        score.0 = 0;
        next_state.set(GameState::Welcome);
    }
}

/// Cleans up the game over screen
pub fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
