use bevy::prelude::*;
use bits_helpers::input::just_pressed_world_position;

use crate::cards::*;
use crate::game::*;

#[derive(Resource, Default)]
pub struct InputState {
    pub _enabled: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>().add_systems(
            Update,
            handle_card_clicks.run_if(in_state(GameState::Playing)),
        );
    }
}

/// Handles player clicks on grid cards and updates sequence cards
fn handle_card_clicks(
    mut commands: Commands,
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut cards: Query<(Entity, &Transform, &GlobalTransform, &mut Card)>,
    mut sequence_state: ResMut<SequenceState>,
    mut game_progress: ResMut<GameProgress>,
    mut stage_state: ResMut<StageState>,
    mut score_state: ResMut<ScoreState>,
    mut feedback_state: ResMut<FeedbackState>,
    mut sprite_query: Query<&mut Sprite>,
    difficulty: Res<GameDifficulty>,
) {
    // Block input unless we're in the Ready state
    if game_progress.is_interaction_blocked() || feedback_state.unmatch_timer.is_some() {
        return;
    }

    // Get click/touch position
    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    // First find the clicked card and check if it's valid
    let clicked_info = cards
        .iter()
        .find(|(_, transform, global_transform, card)| {
            // Skip sequence cards and locked cards
            if card.sequence_position.is_some() || card.locked {
                return false;
            }

            let distance = world_position.distance(global_transform.translation().truncate());
            distance < 35.0
        })
        .map(|(entity, _, _, card)| (entity, card.emoji_index));

    if let Some((clicked_entity, emoji_index)) = clicked_info {
        // Check if this is the correct next emoji
        let next_index = sequence_state.player_sequence.len();
        let is_correct = sequence_state.target_sequence.get(next_index) == Some(&emoji_index);

        if is_correct {
            // Lock and color the correct card
            if let Ok(mut sprite) = sprite_query.get_mut(clicked_entity) {
                sprite.color = CORRECT_COLOR;
            }
            if let Ok((_, _, _, mut card)) = cards.get_mut(clicked_entity) {
                card.locked = true;
            }

            // Record selection
            sequence_state.player_sequence.push(emoji_index);

            // Start timing on first selection
            if sequence_state.player_sequence.len() == 1 {
                game_progress.attempt_timer = Timer::from_seconds(0.0, TimerMode::Once);
            }

            // Update sequence card visibility
            for (_, _, _, mut seq_card) in cards.iter_mut() {
                if seq_card.sequence_position == Some(next_index) {
                    seq_card.face_up = true;
                }
            }

            // Check if sequence is complete
            if sequence_state.player_sequence.len() == difficulty.sequence_length as usize {
                // Calculate score based on completion time
                let attempt_time = game_progress.attempt_timer.elapsed_secs();
                let speed_bonus = if attempt_time <= SPEED_BONUS_THRESHOLD {
                    MAX_SPEED_BONUS
                } else {
                    let time_factor = 1.0
                        - ((attempt_time - SPEED_BONUS_THRESHOLD) / SPEED_BONUS_THRESHOLD).min(1.0);
                    (MAX_SPEED_BONUS as f32 * time_factor) as u32
                };

                score_state.stage_score = STAGE_COMPLETION_SCORE + speed_bonus;
                score_state.total_score += score_state.stage_score;
                stage_state.stage_complete = true;
                stage_state.transition_timer = Some(Timer::from_seconds(2.0, TimerMode::Once));
            }
        } else {
            // Handle incorrect selection
            if !game_progress.record_mistake() {
                // Apply mismatch color to clicked card
                feedback_state.mismatch_entity = Some(clicked_entity);
                feedback_state.unmatch_timer =
                    Some(Timer::from_seconds(MISMATCH_DELAY, TimerMode::Once));

                // Apply red color to clicked card
                if let Ok(mut sprite) = sprite_query.get_mut(clicked_entity) {
                    sprite.color = MISMATCH_COLOR;
                }

                // Clear partial sequence
                sequence_state.player_sequence.clear();

                // Hide all sequence cards
                for (_, _, _, mut card) in cards.iter_mut() {
                    if card.sequence_position.is_some() {
                        card.face_up = false;
                    }
                }
            } else {
                // Game over - show all cards briefly before transition
                for (_, _, _, mut card) in cards.iter_mut() {
                    card.face_up = true;
                }
                game_progress.game_over_reveal_timer =
                    Some(Timer::from_seconds(3.0, TimerMode::Once));
            }
        }
    }
}
