use bevy::prelude::*;
use bits_helpers::input::{
    just_pressed_world_position, just_released_world_position, pressed_world_position,
};

use crate::game::{GameProgress, GameState};
use crate::physics::TowerBlock;

pub struct InputPlugin;

const GRAB_DISTANCE: f32 = 35.0;
const EXTRACTION_THRESHOLD: f32 = 60.0;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionState>().add_systems(
            Update,
            (
                handle_block_grab,
                handle_block_movement,
                handle_block_release,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource, Default)]
pub struct InteractionState {
    pub grabbed_entity: Option<Entity>,
    pub grab_position: Option<Vec2>,
}

fn handle_block_grab(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut blocks: Query<(Entity, &GlobalTransform, &mut TowerBlock)>,
    mut interaction_state: ResMut<InteractionState>,
    game_progress: Res<GameProgress>,
    _commands: Commands,
    mut debug_text_query: Query<&mut Text, With<crate::game::InteractionStateText>>,
) {
    if game_progress.is_interaction_blocked() {
        if let Ok(mut text) = debug_text_query.get_single_mut() {
            text.0 = "GRAB: Interaction blocked".to_string();
        }
        return;
    }

    if interaction_state.grabbed_entity.is_some() {
        if let Ok(mut text) = debug_text_query.get_single_mut() {
            text.0 = format!(
                "GRAB: Already grabbing {}",
                interaction_state.grabbed_entity.unwrap().index()
            );
        }
        return;
    }

    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    if let Ok(mut text) = debug_text_query.get_single_mut() {
        text.0 = format!(
            "CLICK at ({:.1}, {:.1})",
            world_position.x, world_position.y
        );
    }

    let mut closest_entity = None;
    let mut closest_distance = GRAB_DISTANCE;

    for (entity, transform, block) in &mut blocks {
        if !block.removable {
            continue;
        }

        let block_pos = transform.translation().truncate();
        let distance = world_position.distance(block_pos);

        if distance < closest_distance {
            closest_distance = distance;
            closest_entity = Some((entity, block_pos));
        }
    }

    if let Some((entity, position)) = closest_entity {
        interaction_state.grabbed_entity = Some(entity);
        interaction_state.grab_position = Some(world_position);

        if let Ok((_, _, mut block)) = blocks.get_mut(entity) {
            block.being_grabbed = true;
        }

        if let Ok(mut text) = debug_text_query.get_single_mut() {
            text.0 = format!(
                "GRABBED #{} at ({:.1}, {:.1}), dist: {:.1}",
                entity.index(),
                position.x,
                position.y,
                closest_distance
            );
        }
    } else if let Ok(mut text) = debug_text_query.get_single_mut() {
        text.0 = format!(
            "NO BLOCK FOUND at ({:.1}, {:.1})",
            world_position.x,
            world_position.y
        );
    }
}

fn handle_block_movement(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut blocks: Query<(&mut Transform, &TowerBlock)>,
    interaction_state: Res<InteractionState>,
    game_progress: Res<GameProgress>,
    mut debug_text_query: Query<&mut Text, With<crate::game::InteractionStateText>>,
) {
    if game_progress.is_interaction_blocked() {
        return;
    }

    let Some(entity) = interaction_state.grabbed_entity else {
        return;
    };

    let Some(current_position) =
        pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    let Some(grab_position) = interaction_state.grab_position else {
        return;
    };

    if let Ok((mut transform, tower_block)) = blocks.get_mut(entity) {
        if !tower_block.being_grabbed {
            return;
        }

        let movement = current_position - grab_position;
        let new_position = transform.translation.truncate() + (movement * 0.05);
        transform.translation = new_position.extend(transform.translation.z);

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

fn handle_block_release(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut blocks: Query<(&Transform, &mut TowerBlock)>,
    mut interaction_state: ResMut<InteractionState>,
    mut game_progress: ResMut<GameProgress>,
) {
    let Some(entity) = interaction_state.grabbed_entity else {
        return;
    };

    let released =
        just_released_world_position(&buttons, &touch_input, &windows, &camera_q).is_some();

    if released {
        if let Ok((transform, mut block)) = blocks.get_mut(entity) {
            let block_pos = transform.translation.truncate();
            let initial_pos = block.initial_position;
            let displacement = (block_pos - initial_pos).length();

            if displacement > EXTRACTION_THRESHOLD {
                game_progress.record_block_removal();
            }

            block.being_grabbed = false;
        }

        interaction_state.grabbed_entity = None;
        interaction_state.grab_position = None;
    }
}