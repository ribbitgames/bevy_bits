use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::cards::{Card, FlipState, GameState};

/// Component to track whether an entity is clickable
#[derive(Component, Default)]
pub struct Clickable;

/// Resource to track input state
#[derive(Resource, Default)]
pub struct InputState {
    /// Whether input processing is currently enabled
    pub _enabled: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(Update, handle_card_clicks);
    }
}

/// Handles mouse clicks on cards, implementing the core card flipping mechanics
///
/// This system will:
/// 1. Check if input is allowed based on game state
/// 2. Convert mouse position to world coordinates
/// 3. Detect clicked card
/// 4. Handle card flipping logic while respecting game rules
pub fn handle_card_clicks(
    _commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut cards: Query<(Entity, &Transform, &mut Card)>,
    mut flip_state: ResMut<FlipState>,
    game_state: Res<GameState>,
) {
    // Only process clicks when game is in playable state
    if game_state.cards_revealed
        || game_state.reveal_timer.is_some()
        || game_state.initial_wait_timer.is_some()
    {
        return;
    }

    // Only process clicks when no unmatch animation is playing
    if flip_state.unmatch_timer.is_some() {
        return;
    }

    // Only handle left mouse button clicks
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Get the window and cursor position
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Get camera transform
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    // Convert cursor position to world coordinates
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // Find clicked card
    for (entity, transform, mut card) in &mut cards {
        let card_pos = transform.translation.truncate();
        let distance = world_position.distance(card_pos);

        // Check if click is within card bounds (using a reasonable hit area)
        if distance < 30.0 {
            // Don't process already flipped or locked cards
            if card.face_up || card.locked {
                return;
            }

            // Don't allow more than 2 cards to be face up
            if flip_state.face_up_cards.len() >= 2 {
                return;
            }

            // Flip the card
            card.face_up = true;
            flip_state.face_up_cards.push(entity);

            // No need to check other cards
            break;
        }
    }
}
