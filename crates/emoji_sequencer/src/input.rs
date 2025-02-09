use bevy::prelude::*;
use bits_helpers::input::just_pressed_world_position;

use crate::cards::{Card, CORRECT_COLOR, DEFAULT_COLOR, WRONG_COLOR};
use crate::game::{
    FeedbackState, GameDifficulty, GameProgress, GameState, ScoreState, SequenceState, StageState,
    MAX_SPEED_BONUS, MISMATCH_DELAY, SPEED_BONUS_THRESHOLD, STAGE_COMPLETION_SCORE,
};

#[derive(Resource, Default)]
/// Resource that tracks the current input state of the game.
///
/// Currently, it holds a flag indicating whether input is enabled. This may be expanded
/// in the future to include additional input-related state.
pub struct InputState {
    pub _enabled: bool,
}

/// Plugin responsible for handling player input and card interactions.
///
/// This plugin registers two systems:
/// - `handle_card_clicks`: Processes clicks on grid cards when the game is in the Playing state.
/// - `reset_mismatch_color`: Resets card colors after an incorrect selection.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    /// Configures the `InputPlugin` by initializing resources and adding the input-related systems.
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
///
/// This system performs the following steps:
/// 1. Aborts if input is currently blocked (e.g. during feedback delays).
/// 2. Converts mouse/touch input into a world position.
/// 3. Locates a clicked card that is not yet locked or already in the sequence.
/// 4. Checks if the clicked card matches the next expected emoji in the target sequence:
///    - On a correct click, updates card visuals, locks the card, reveals the corresponding sequence card,
///      and, if the sequence is complete, calculates the speed bonus and marks the stage as complete.
///    - On an incorrect click, calls `record_mistake()` and either sets visual mismatch feedback (if allowed)
///      or reveals all cards to signal game over.
///
/// # Parameters
/// - `windows`: Provides window information for coordinate transformations.
/// - `buttons`: Contains the current mouse button input state.
/// - `touch_input`: Provides the current state of touch inputs.
/// - `camera_q`: Query to obtain the camera and its transform for converting input to world coordinates.
/// - `cards`: Query for card entities along with their transforms and related data.
/// - `sequence_state`: Tracks both the target sequence and the player's current sequence.
/// - `game_progress`: Manages overall game state and timers for player attempts.
/// - `stage_state`: Holds state related to the current stage, including transition timers.
/// - `score_state`: Accumulates scoring information for the stage and total score.
/// - `feedback_state`: Stores feedback data such as mismatch timers and entities.
/// - `sprite_query`: Query for accessing and modifying sprite components.
/// - `difficulty`: Represents the current game difficulty settings.
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
) {
    // Exit early if interactions are blocked or if a mismatch is still being displayed.
    if game_progress.is_interaction_blocked() || feedback_state.unmatch_timer.is_some() {
        return;
    }

    // Obtain the world position from the most recent mouse or touch input.
    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    // Find a clicked card that is not part of the sequence and is not locked.
    let Some((clicked_entity, emoji_index, children)) = cards
        .iter()
        .find(|(_, _transform, global_transform, card, _)| {
            if card.sequence_position.is_some() || card.locked {
                return false;
            }
            let distance = world_position.distance(global_transform.translation().truncate());
            distance < 35.0
        })
        .map(|(entity, _transform, _global_transform, card, children)| {
            (entity, card.emoji_index, children)
        })
    else {
        return;
    };

    let next_index = sequence_state.player_sequence.len();
    // Check if the clicked card matches the expected emoji in the target sequence.
    let is_correct = sequence_state.target_sequence.get(next_index) == Some(&emoji_index);

    if !is_correct {
        // For an incorrect selection, record the mistake and update feedback.
        if !game_progress.record_mistake() {
            feedback_state.mismatch_entity = Some(clicked_entity);
            feedback_state.unmatch_timer =
                Some(Timer::from_seconds(MISMATCH_DELAY, TimerMode::Once));

            // Change the color of the card's child sprites to indicate an error.
            for &child in children {
                if let Ok(mut sprite) = sprite_query.get_mut(child) {
                    sprite.color = WRONG_COLOR;
                }
            }
        } else {
            // If the mistake limit has been reached, reveal all cards to signal game over.
            for (_, _, _, mut card, _) in &mut cards {
                card.face_up = true;
            }
            game_progress.game_over_reveal_timer = Some(Timer::from_seconds(3.0, TimerMode::Once));
        }
        return;
    }

    // Process a correct card selection:
    // Update the color of the card's child sprites to indicate a correct pick.
    for &child in children {
        if let Ok(mut sprite) = sprite_query.get_mut(child) {
            sprite.color = CORRECT_COLOR;
        }
    }
    // Lock the clicked card to prevent further interaction.
    if let Ok((_, _, _, mut card, _)) = cards.get_mut(clicked_entity) {
        card.locked = true;
    }
    // Append the correctly selected emoji to the player's sequence.
    sequence_state.player_sequence.push(emoji_index);

    // Start the attempt timer when the first card is selected.
    if sequence_state.player_sequence.len() == 1 {
        game_progress.attempt_timer = Timer::from_seconds(0.0, TimerMode::Once);
    }

    // Reveal the corresponding card in the sequence by flipping it face up and locking it.
    for (_, _, _, mut seq_card, _) in &mut cards {
        if seq_card.sequence_position == Some(next_index) {
            seq_card.face_up = true;
            seq_card.locked = true;
        }
    }

    // When the sequence is fully matched, calculate any speed bonus and complete the stage.
    if sequence_state.player_sequence.len() == difficulty.sequence_length as usize {
        let attempt_time = game_progress.attempt_timer.elapsed_secs();
        let speed_bonus = if attempt_time <= SPEED_BONUS_THRESHOLD {
            MAX_SPEED_BONUS
        } else {
            let time_factor =
                1.0 - ((attempt_time - SPEED_BONUS_THRESHOLD) / SPEED_BONUS_THRESHOLD).min(1.0);
            (MAX_SPEED_BONUS as f32 * time_factor) as u32
        };

        score_state.stage_score = STAGE_COMPLETION_SCORE + speed_bonus;
        score_state.total_score += score_state.stage_score;
        stage_state.stage_complete = true;
        stage_state.transition_timer = Some(Timer::from_seconds(2.0, TimerMode::Once));
    }
}

/// Resets the color of cards that were incorrectly selected after a delay.
///
/// This system checks whether the mismatch timer has finished, and if so,
/// resets the color of the affected card back to the default color.
///
/// # Parameters
/// - `time`: Provides the time delta for this frame.
/// - `feedback_state`: Contains state regarding the feedback (e.g., mismatch timing).
/// - `sprite_query`: Query for accessing and modifying sprite components.
/// - `cards`: Query for retrieving card entities along with their child components.
fn reset_mismatch_color(
    time: Res<Time>,
    mut feedback_state: ResMut<FeedbackState>,
    mut sprite_query: Query<&mut Sprite>,
    mut cards: Query<(Entity, &Children)>,
) {
    // Use `let ... else` syntax to immediately return if there's no mismatch timer.
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
                    sprite.color = DEFAULT_COLOR;
                }
            }
        }
    }
    feedback_state.unmatch_timer = None;
    feedback_state.mismatch_entity = None;
}
