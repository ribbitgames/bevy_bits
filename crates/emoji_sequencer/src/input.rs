use bevy::prelude::*;
use bits_helpers::input::just_pressed_world_position;

use crate::cards::{Card, CORRECT_COLOR, DEFAULT_COLOR, WRONG_COLOR};
use crate::game::{
    FeedbackState, GameDifficulty, GameProgress, GameState, ScoreState, SequenceState, StageState,
    MAX_SPEED_BONUS, MISMATCH_DELAY, SPEED_BONUS_THRESHOLD, STAGE_COMPLETION_SCORE,
};

#[derive(Resource, Default)]
pub struct InputState {
    pub _enabled: bool,
}

#[derive(Component)]
pub struct MismatchTimer {
    pub timer: Timer,
    pub entity: Entity,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(
                Update,
                handle_card_clicks.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, reset_mismatch_color);
    }
}

/// Handles player clicks on grid cards and updates sequence cards
fn handle_card_clicks(
    mut commands: Commands,
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut cards: Query<(Entity, &Transform, &GlobalTransform, &mut Card, &Children)>,
    mut sequence_state: ResMut<SequenceState>,
    mut game_progress: ResMut<GameProgress>,
    mut stage_state: ResMut<StageState>,
    mut score_state: ResMut<ScoreState>,
    mut feedback_state: ResMut<FeedbackState>,
    mut sprite_query: Query<&mut Sprite>,
    difficulty: Res<GameDifficulty>,
) {
    if game_progress.is_interaction_blocked() || feedback_state.unmatch_timer.is_some() {
        return;
    }

    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    let clicked_info = cards
        .iter()
        .find(|(_, transform, global_transform, card, _)| {
            if card.sequence_position.is_some() || card.locked {
                return false;
            }
            let distance = world_position.distance(global_transform.translation().truncate());
            distance < 35.0
        })
        .map(|(entity, _, _, card, children)| (entity, card.emoji_index, children));

    if let Some((clicked_entity, emoji_index, children)) = clicked_info {
        let next_index = sequence_state.player_sequence.len();
        let is_correct = sequence_state.target_sequence.get(next_index) == Some(&emoji_index);

        if is_correct {
            for &child in children.iter() {
                if let Ok(mut sprite) = sprite_query.get_mut(child) {
                    sprite.color = CORRECT_COLOR;
                }
            }
            if let Ok((_, _, _, mut card, _)) = cards.get_mut(clicked_entity) {
                card.locked = true;
            }

            sequence_state.player_sequence.push(emoji_index);

            if sequence_state.player_sequence.len() == 1 {
                game_progress.attempt_timer = Timer::from_seconds(0.0, TimerMode::Once);
            }

            for (_, _, _, mut seq_card, _) in cards.iter_mut() {
                if seq_card.sequence_position == Some(next_index) {
                    seq_card.face_up = true;
                    seq_card.locked = true;
                }
            }

            if sequence_state.player_sequence.len() == difficulty.sequence_length as usize {
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
            if !game_progress.record_mistake() {
                feedback_state.mismatch_entity = Some(clicked_entity);
                feedback_state.unmatch_timer =
                    Some(Timer::from_seconds(MISMATCH_DELAY, TimerMode::Once));

                for &child in children.iter() {
                    if let Ok(mut sprite) = sprite_query.get_mut(child) {
                        sprite.color = WRONG_COLOR;
                    }
                }
                commands.spawn(MismatchTimer {
                    timer: Timer::from_seconds(MISMATCH_DELAY, TimerMode::Once),
                    entity: clicked_entity,
                });
            } else {
                for (_, _, _, mut card, _) in cards.iter_mut() {
                    card.face_up = true;
                }
                game_progress.game_over_reveal_timer =
                    Some(Timer::from_seconds(3.0, TimerMode::Once));
            }
        }
    }
}

fn reset_mismatch_color(
    time: Res<Time>,
    mut feedback_state: ResMut<FeedbackState>,
    mut sprite_query: Query<&mut Sprite>,
    mut cards: Query<(Entity, &Children)>,
) {
    if let Some(timer) = &mut feedback_state.unmatch_timer {
        if timer.tick(time.delta()).just_finished() {
            if let Some(entity) = feedback_state.mismatch_entity {
                if let Ok((_, children)) = cards.get_mut(entity) {
                    for &child in children.iter() {
                        if let Ok(mut sprite) = sprite_query.get_mut(child) {
                            sprite.color = DEFAULT_COLOR;
                        }
                    }
                }
            }
            feedback_state.unmatch_timer = None;
            feedback_state.mismatch_entity = None;
        }
    }
}
