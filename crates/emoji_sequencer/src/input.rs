use bevy::prelude::*;
use bits_helpers::input::just_pressed_world_position;

use crate::cards::Card;
use crate::game::{
    FeedbackState, GameDifficulty, GameProgress, GameState, ScoreState, SequenceState, StageState,
};
use crate::variables::GameVariables;

#[derive(Resource, Default)]
/// Resource that tracks the current input state of the game.
pub struct InputState {
    pub _enabled: bool,
}

/// Plugin responsible for handling player input and card interactions.
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

/// Processes player clicks on grid cards and updates game state accordingly.
fn handle_card_clicks(
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
    vars: Res<GameVariables>,
) {
    if game_progress.is_interaction_blocked() || feedback_state.unmatch_timer.is_some() {
        return;
    }

    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    let Some((clicked_entity, emoji_index, children)) = cards
        .iter()
        .find(|(_, _transform, global_transform, card, _)| {
            if card.sequence_position.is_some() || card.locked {
                return false;
            }
            let distance = world_position.distance(global_transform.translation().truncate());
            distance < vars.card_click_radius
        })
        .map(|(entity, _transform, _global_transform, card, children)| {
            (entity, card.emoji_index, children)
        })
    else {
        return;
    };

    let next_index = sequence_state.player_sequence.len();
    let is_correct = sequence_state.target_sequence.get(next_index) == Some(&emoji_index);

    if !is_correct {
        if !game_progress.record_mistake(&vars) {
            feedback_state.mismatch_entity = Some(clicked_entity);
            feedback_state.unmatch_timer =
                Some(Timer::from_seconds(vars.mismatch_delay, TimerMode::Once));

            for &child in children {
                if let Ok(mut sprite) = sprite_query.get_mut(child) {
                    sprite.color = vars.wrong_color;
                }
            }
        } else {
            for (_, _, _, mut card, _) in &mut cards {
                card.face_up = true;
            }
            game_progress.game_over_reveal_timer = Some(Timer::from_seconds(
                vars.game_over_reveal_duration,
                TimerMode::Once,
            ));
        }
        return;
    }

    for &child in children {
        if let Ok(mut sprite) = sprite_query.get_mut(child) {
            sprite.color = vars.correct_color;
        }
    }

    if let Ok((_, _, _, mut card, _)) = cards.get_mut(clicked_entity) {
        card.locked = true;
    }

    sequence_state.player_sequence.push(emoji_index);

    if sequence_state.player_sequence.len() == 1 {
        game_progress.attempt_timer = Timer::from_seconds(0.0, TimerMode::Once);
    }

    for (_, _, _, mut seq_card, _) in &mut cards {
        if seq_card.sequence_position == Some(next_index) {
            seq_card.face_up = true;
            seq_card.locked = true;
        }
    }

    if sequence_state.player_sequence.len() == difficulty.sequence_length as usize {
        let attempt_time = game_progress.attempt_timer.elapsed_secs();
        let speed_bonus = if attempt_time <= vars.speed_bonus_threshold {
            vars.max_speed_bonus
        } else {
            let time_factor = 1.0
                - ((attempt_time - vars.speed_bonus_threshold) / vars.speed_bonus_threshold)
                    .min(1.0);
            (vars.max_speed_bonus as f32 * time_factor) as u32
        };

        score_state.stage_score = vars.stage_completion_score + speed_bonus;
        score_state.total_score += score_state.stage_score;
        stage_state.stage_complete = true;
        stage_state.transition_timer = Some(Timer::from_seconds(
            vars.stage_transition_duration,
            TimerMode::Once,
        ));
    }
}

/// Resets the color of cards that were incorrectly selected after a delay.
fn reset_mismatch_color(
    time: Res<Time>,
    mut feedback_state: ResMut<FeedbackState>,
    mut sprite_query: Query<&mut Sprite>,
    mut cards: Query<(Entity, &Children)>,
    vars: Res<GameVariables>,
) {
    let Some(timer) = feedback_state.unmatch_timer.as_mut() else {
        return;
    };
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    if let Some(entity) = feedback_state.mismatch_entity {
        if let Ok((_, children)) = cards.get_mut(entity) {
            for &child in children {
                if let Ok(mut sprite) = sprite_query.get_mut(child) {
                    sprite.color = vars.default_color;
                }
            }
        }
    }
    feedback_state.unmatch_timer = None;
    feedback_state.mismatch_entity = None;
}
