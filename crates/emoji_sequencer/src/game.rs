use bevy::prelude::*;

use crate::cards::Card;
use crate::effects::CelebrationState;
use crate::variables::GameVariables;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameDifficulty>()
            .init_resource::<StageState>()
            .init_resource::<SequenceState>()
            .init_resource::<GameProgress>()
            .init_resource::<ScoreState>()
            .init_resource::<FeedbackState>()
            .add_systems(Update, handle_stage_transition)
            .add_systems(
                Update,
                (handle_game_over_sequence, handle_feedback_reset)
                    .run_if(in_state(GameState::Playing))
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub struct StageState {
    pub stage_complete: bool,
    pub transition_timer: Option<Timer>,
}

#[derive(Resource, Default)]
pub struct FeedbackState {
    /// Timer for resetting mismatch color
    pub unmatch_timer: Option<Timer>,
    /// Entity that showed mismatch
    pub mismatch_entity: Option<Entity>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Resource)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    StageComplete,
    GameOver,
}

/// Represents the sequence of steps during card reveal and gameplay
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum SequenceStep {
    #[default]
    SpawningSequence,
    RevealingSequence,
    HidingSequence,
    SpawningGrid,
    Ready,
}

#[derive(Resource)]
pub struct GameProgress {
    /// Current step in the sequence
    pub sequence_step: SequenceStep,
    /// Timer for current reveal or transition phase
    pub step_timer: Option<Timer>,
    /// Current position in sequence reveal
    pub current_reveal_index: usize,
    /// Number of incorrect sequences
    pub mistakes: u32,
    /// Maximum mistakes allowed before game over
    pub max_mistakes: u32,
    /// Whether player has lost
    pub game_over: bool,
    /// Timer for showing all cards after game over
    pub game_over_reveal_timer: Option<Timer>,
    /// Timer tracking time taken for current sequence attempt
    pub attempt_timer: Timer,
}

impl Default for GameProgress {
    fn default() -> Self {
        let vars = GameVariables::default();
        Self {
            sequence_step: SequenceStep::SpawningSequence,
            step_timer: None,
            current_reveal_index: 0,
            mistakes: 0,
            max_mistakes: vars.max_mistakes,
            game_over: false,
            game_over_reveal_timer: None,
            attempt_timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

impl GameProgress {
    /// Records a mistake and returns whether game is over
    pub fn record_mistake(&mut self, vars: &GameVariables) -> bool {
        self.mistakes += 1;
        if self.mistakes >= self.max_mistakes {
            self.game_over = true;
            self.game_over_reveal_timer = Some(Timer::from_seconds(
                vars.game_over_reveal_duration,
                TimerMode::Once,
            ));
        }
        self.game_over
    }

    /// Returns true if input should be blocked
    pub fn is_interaction_blocked(&self) -> bool {
        self.sequence_step != SequenceStep::Ready || self.game_over || self.step_timer.is_some()
    }
}

#[derive(Resource)]
pub struct GameDifficulty {
    /// Current stage number (starts at 1)
    pub stage: u32,
    /// number of cards per sequence (starts at 3)
    pub sequence_length: u32,
    /// Number of columns in the emoji grid
    pub grid_cols: u32,
    /// Number of rows in the emoji grid
    pub grid_rows: u32,
    /// Spacing between emojis in the grid
    pub grid_spacing: f32,
    /// Total number of emojis to show (including sequence)
    pub total_emojis: usize,
}

impl Default for GameDifficulty {
    fn default() -> Self {
        let vars = GameVariables::default();
        Self {
            stage: 1,
            sequence_length: vars.initial_sequence_length,
            grid_cols: vars.initial_grid_cols,
            grid_rows: vars.initial_grid_rows,
            grid_spacing: vars.initial_grid_spacing,
            total_emojis: vars.initial_total_emojis,
        }
    }
}

impl GameDifficulty {
    /// Progress to next stage and recalculate difficulty parameters
    pub fn advance_stage(&mut self) {
        self.stage += 1;

        // Increase sequence length every 2 stages
        if self.stage % 2 == 0 {
            self.sequence_length += 1;
        }

        // Always recalculate difficulty when advancing
        self.recalculate_difficulty();
    }

    /// Calculate difficulty parameters based on current stage
    fn recalculate_difficulty(&mut self) {
        // Increase total emojis every 2 stages
        self.total_emojis = 6 + (self.stage / 2) as usize;

        // Adjust grid dimensions based on total emojis
        self.grid_cols = (self.total_emojis as f32).sqrt().ceil() as u32;
        self.grid_rows = (self.total_emojis as u32).div_ceil(self.grid_cols);
    }
}

#[derive(Resource, Default)]
pub struct SequenceState {
    /// The sequence of emoji indices to memorize
    pub target_sequence: Vec<usize>,
    /// The sequence of emoji indices clicked by player
    pub player_sequence: Vec<usize>,
    /// Timer for showing feedback on sequence attempt
    pub feedback_timer: Option<Timer>,
    /// Timer for showing full sequence before hiding
    pub completion_timer: Option<Timer>,
}

#[derive(Resource, Default)]
pub struct ScoreState {
    /// Total accumulated score
    pub total_score: u32,
    /// Score for current stage
    pub stage_score: u32,
}

/// Handles transition between game stages, including cleanup and state reset
fn handle_stage_transition(
    mut commands: Commands,
    celebration_state: Res<CelebrationState>,
    mut stage_state: ResMut<StageState>,
    mut game_difficulty: ResMut<GameDifficulty>,
    mut game_progress: ResMut<GameProgress>,
    mut sequence_state: ResMut<SequenceState>,
    mut next_state: ResMut<NextState<GameState>>,
    cards: Query<Entity, With<Card>>,
) {
    if stage_state.stage_complete {
        if celebration_state.is_celebrating {
            return;
        }

        // Clear all existing cards
        for card_entity in cards.iter() {
            commands.entity(card_entity).despawn_recursive();
        }

        // Update difficulty
        game_difficulty.advance_stage();
        game_difficulty.recalculate_difficulty();

        // Reset game progress
        *game_progress = GameProgress::default();

        // Reset sequence state
        sequence_state.target_sequence.clear();
        sequence_state.player_sequence.clear();
        sequence_state.feedback_timer = None;
        sequence_state.completion_timer = None;

        // Reset stage state
        stage_state.stage_complete = false;
        stage_state.transition_timer = None;

        // Finally transition to StageComplete
        next_state.set(GameState::StageComplete);
    }
}

fn handle_game_over_sequence(
    mut commands: Commands,
    time: Res<Time>,
    mut game_progress: ResMut<GameProgress>,
    mut cards: Query<(Entity, &mut Card)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if game_progress.game_over && game_progress.game_over_reveal_timer.is_some() {
        // Show all cards
        for (_, mut card) in &mut cards {
            card.face_up = true;
        }

        if let Some(timer) = &mut game_progress.game_over_reveal_timer {
            if timer.tick(time.delta()).just_finished() {
                // Cleanup
                for (entity, _) in cards.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                next_state.set(GameState::GameOver);
            }
        }
    }
}

fn handle_feedback_reset(
    time: Res<Time>,
    vars: Res<GameVariables>,
    mut feedback_state: ResMut<FeedbackState>,
    mut sprite_query: Query<&mut Sprite>,
) {
    if let Some(timer) = &mut feedback_state.unmatch_timer {
        if timer.tick(time.delta()).just_finished() {
            // Reset color of mismatched card
            if let Some(entity) = feedback_state.mismatch_entity {
                if let Ok(mut sprite) = sprite_query.get_mut(entity) {
                    sprite.color = vars.default_color;
                }
            }
            feedback_state.unmatch_timer = None;
            feedback_state.mismatch_entity = None;
        }
    }
}
