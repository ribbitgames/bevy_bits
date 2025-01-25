use bevy::prelude::*;
use bits_helpers::input::just_pressed_world_position;

use crate::cards::Card;
use crate::game::{FlipState, GameState};

/// Component to track whether an entity is clickable
#[derive(Component, Default)]
pub struct Clickable;

/// Resource to track input state
#[derive(Resource, Default)]
pub struct InputState {
    pub _enabled: bool, // Enable/disable input handling dynamically
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(Update, handle_card_clicks);
    }
}

/// Handles mouse clicks on cards to flip them
pub fn handle_card_clicks(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut cards: Query<(Entity, &Transform, &GlobalTransform, &mut Card)>,
    mut flip_state: ResMut<FlipState>,
    game_state: Res<GameState>,
) {
    // Early return conditions
    if game_state.cards_revealed
        || game_state.reveal_timer.is_some()
        || game_state.initial_wait_timer.is_some()
        || flip_state.unmatch_timer.is_some()
    {
        return;
    }

    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    for (entity, _local_transform, global_transform, mut card) in &mut cards {
        // Use global transform for position comparison
        let card_position = global_transform.translation().truncate();
        let distance = world_position.distance(card_position);

        // Adjust click detection as needed
        if distance < 35.0 {
            // Existing card flip logic
            if !card.face_up && !card.locked && flip_state.face_up_cards.len() < 2 {
                card.face_up = true;
                flip_state.face_up_cards.push(entity);
                break;
            }
        }
    }
}
