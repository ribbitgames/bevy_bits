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
const PULL_STRENGTH: f32 = 15.0; // How strongly to pull when extracting
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
) {
    // Don't allow grabbing if interaction is blocked
    if game_progress.is_interaction_blocked() {
        return;
    }

    // Return early if already grabbing a block
    if interaction_state.grabbed_entity.is_some() {
        return;
    }

    // Check for a new touch/click
    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

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
            closest_entity = Some(entity);
        }
    }

    // If found a block to grab, update its state
    if let Some(entity) = closest_entity {
        interaction_state.grabbed_entity = Some(entity);
        interaction_state.grab_position = Some(world_position);

        // Mark the block as being grabbed
        if let Ok((_, _, mut block)) = blocks.get_mut(entity) {
            block.being_grabbed = true;
        }
    }
}

/// System to handle moving a grabbed block
fn handle_block_movement(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut blocks: Query<(&Transform, &mut ExternalForce, &mut Velocity)>,
    interaction_state: Res<InteractionState>,
    game_progress: Res<GameProgress>,
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

    // Calculate pull direction and strength
    let pull_dir = current_position - grab_position;

    // Apply force to the block
    if let Ok((transform, mut ext_force, mut velocity)) = blocks.get_mut(entity) {
        // Scale force based on distance
        let force_strength = pull_dir.length().min(PULL_STRENGTH);
        let force = pull_dir.normalize_or_zero() * force_strength * 50.0;

        // Apply the force
        ext_force.force = force;

        // Dampen velocity to prevent too much momentum
        velocity.linvel *= 0.9;
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
