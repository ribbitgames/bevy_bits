use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::cards::{Card, FlipState, GameState};

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
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
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

    // Check for left mouse button press
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Get window and cursor position
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Get camera
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    // Convert cursor position to world coordinates
    let world_position = match camera.viewport_to_world_2d(camera_transform, cursor_position) {
        Ok(pos) => pos,
        Err(_) => return,
    };

    // Detailed debugging of card information
    for (entity, local_transform, global_transform, mut card) in cards.iter_mut() {
        // Print detailed transform information
        println!("Card Entity: {:?}", entity);
        println!("Local Transform:");
        println!("  Translation: {:?}", local_transform.translation);
        println!("  Rotation: {:?}", local_transform.rotation);
        println!("  Scale: {:?}", local_transform.scale);

        println!("Global Transform:");
        println!("  Translation: {:?}", global_transform.translation());
        println!(
            "  Rotation: {:?}",
            global_transform.to_scale_rotation_translation().1
        );
        println!(
            "  Scale: {:?}",
            global_transform.to_scale_rotation_translation().0
        );

        // Use global transform for position comparison
        let card_position = global_transform.translation().truncate();
        let distance = world_position.distance(card_position);

        println!("World Position: {:?}", world_position);
        println!("Card Position: {:?}", card_position);
        println!("Distance: {}", distance);

        // Adjust click detection as needed
        if distance < 35.0 {
            // Existing card flip logic
            if !card.face_up && !card.locked && flip_state.face_up_cards.len() < 2 {
                card.face_up = true;
                flip_state.face_up_cards.push(entity);
                println!("Card flipped successfully!");
                break;
            }
        }
    }
}
