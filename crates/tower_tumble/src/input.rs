use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bits_helpers::input::{
    just_pressed_world_position, just_released_world_position, pressed_world_position,
};

use crate::game::{GameProgress, GameState};
use crate::physics::TowerBlock;

pub struct InputPlugin;

// Constants for interaction
const GRAB_DISTANCE: f32 = 35.0; // Maximum distance to grab a block
const PULL_STRENGTH: f32 = 200.0; // Massively increased from 50.0
const EXTRACTION_THRESHOLD: f32 = 60.0; // Distance needed to extract a block

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionState>().add_systems(
            Update,
            (
                handle_block_grab,
                handle_block_movement,
                handle_block_release,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource, Default)]
pub struct InteractionState {
    pub grabbed_entity: Option<Entity>,
    pub grab_position: Option<Vec2>,
}

/// System to handle initial block grab
fn handle_block_grab(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut blocks: Query<(Entity, &GlobalTransform, &mut TowerBlock)>,
    mut interaction_state: ResMut<InteractionState>,
    game_progress: Res<GameProgress>,
    mut commands: Commands,
    mut debug_text_query: Query<&mut Text, With<crate::game::InteractionStateText>>,
) {
    // Don't allow grabbing if interaction is blocked
    if game_progress.is_interaction_blocked() {
        if let Ok(mut text) = debug_text_query.get_single_mut() {
            text.0 = format!("GRAB: Interaction blocked");
        }
        return;
    }

    // Return early if already grabbing a block
    if interaction_state.grabbed_entity.is_some() {
        if let Ok(mut text) = debug_text_query.get_single_mut() {
            text.0 = format!(
                "GRAB: Already grabbing {}",
                interaction_state.grabbed_entity.unwrap().index()
            );
        }
        return;
    }

    // Check for a new touch/click
    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        // No click detected
        return;
    };

    if let Ok(mut text) = debug_text_query.get_single_mut() {
        text.0 = format!(
            "CLICK at ({:.1}, {:.1})",
            world_position.x, world_position.y
        );
    }

    // Find the closest block within grab distance
    let mut closest_entity = None;
    let mut closest_distance = GRAB_DISTANCE;

    for (entity, transform, block) in &blocks {
        if !block.removable {
            continue; // Skip blocks that can't be removed
        }

        let block_pos = transform.translation().truncate();
        let distance = world_position.distance(block_pos);

        if distance < closest_distance {
            closest_distance = distance;
            closest_entity = Some((entity, block_pos));
        }
    }

    // If found a block to grab, update its state
    if let Some((entity, position)) = closest_entity {
        interaction_state.grabbed_entity = Some(entity);
        interaction_state.grab_position = Some(world_position);

        // Mark the block as being grabbed
        if let Ok((_, _, mut block)) = blocks.get_mut(entity) {
            block.being_grabbed = true;
        }

        // Update debug text
        if let Ok(mut text) = debug_text_query.get_single_mut() {
            text.0 = format!(
                "GRABBED #{} at ({:.1}, {:.1}), dist: {:.1}",
                entity.index(),
                position.x,
                position.y,
                closest_distance
            );
        }
    } else {
        // No block found to grab
        if let Ok(mut text) = debug_text_query.get_single_mut() {
            text.0 = format!(
                "NO BLOCK FOUND at ({:.1}, {:.1})",
                world_position.x, world_position.y
            );
        }
    }
}

/// System to handle moving a grabbed block
fn handle_block_movement(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut blocks: Query<(
        &mut Transform,
        &mut ExternalForce,
        &mut Velocity,
        &TowerBlock,
    )>,
    interaction_state: Res<InteractionState>,
    game_progress: Res<GameProgress>,
    mut debug_text_query: Query<&mut Text, With<crate::game::InteractionStateText>>,
) {
    // Don't move blocks if interaction is blocked
    if game_progress.is_interaction_blocked() {
        return;
    }

    // Check if we have a grabbed block
    let Some(entity) = interaction_state.grabbed_entity else {
        return;
    };

    // Check for current touch/click position
    let Some(current_position) =
        pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    let Some(grab_position) = interaction_state.grab_position else {
        return;
    };

    // Apply direct movement to the block instead of force
    if let Ok((mut transform, mut ext_force, mut velocity, tower_block)) = blocks.get_mut(entity) {
        if !tower_block.being_grabbed {
            return;
        }

        // Calculate movement vector
        let movement = current_position - grab_position;

        // Instead of applying force, directly move the block by a fraction of the movement
        // This provides more direct control
        let new_position = transform.translation.truncate() + (movement * 0.05);
        transform.translation = new_position.extend(transform.translation.z);

        // Set velocity based on movement to maintain momentum
        velocity.linvel = movement * 2.0;

        // Clear any external forces
        ext_force.force = Vec2::ZERO;

        // Update debug text
        if let Ok(mut text) = debug_text_query.get_single_mut() {
            text.0 = format!(
                "MOVING: #{} to ({:.1}, {:.1})",
                entity.index(),
                new_position.x,
                new_position.y
            );
        }
    }
}

/// System to handle releasing a block
fn handle_block_release(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut blocks: Query<(&Transform, &mut TowerBlock, &mut ExternalForce)>,
    mut interaction_state: ResMut<InteractionState>,
    mut game_progress: ResMut<GameProgress>,
) {
    // Check if we're currently grabbing a block
    let Some(entity) = interaction_state.grabbed_entity else {
        return;
    };

    // Check for release event
    let released =
        just_released_world_position(&buttons, &touch_input, &windows, &camera_q).is_some();

    if released {
        // Get the block's position and update its state
        if let Ok((transform, mut block, mut ext_force)) = blocks.get_mut(entity) {
            let block_pos = transform.translation.truncate();

            // Clear the forces
            ext_force.force = Vec2::ZERO;

            // Check if block was extracted far enough
            let initial_pos = block.initial_position;
            let displacement = (block_pos - initial_pos).length();

            if displacement > EXTRACTION_THRESHOLD {
                // Successfully extracted
                game_progress.record_block_removal();
            }

            // No longer being grabbed
            block.being_grabbed = false;
        }

        // Clear the interaction state
        interaction_state.grabbed_entity = None;
        interaction_state.grab_position = None;
    }
}
