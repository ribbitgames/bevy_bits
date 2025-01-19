use bevy::prelude::*;

use crate::cards::{Card, GameState};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameDifficulty>()
            .init_resource::<StageState>()
            .add_systems(
                Update,
                (check_stage_completion, handle_stage_transition).chain(),
            );
    }
}

/// Resource to track stage completion state
#[derive(Resource, Default)]
pub struct StageState {
    /// Indicates stage is complete and transition should begin
    pub stage_complete: bool,
    /// Optional timer for stage transition animation/delay
    pub transition_timer: Option<Timer>,
}

impl StageState {
    pub const fn new() -> Self {
        Self {
            stage_complete: false,
            transition_timer: None,
        }
    }
}

#[derive(Resource, Debug)]
pub struct GameDifficulty {
    /// Current stage/level number (starts at 1)
    pub stage: u32,
    /// Number of columns in the grid for current stage
    pub grid_cols: u32,
    /// Number of rows in the grid for current stage
    pub grid_rows: u32,
    /// Spacing between cards in the grid
    pub grid_spacing: f32,
    /// Number of pairs to match
    pub num_pairs: usize,
    /// Time to show all cards at start (seconds)
    pub initial_reveal_time: f32,
    /// Time to show mismatched cards (seconds)
    pub mismatch_delay: f32,
}

impl Default for GameDifficulty {
    fn default() -> Self {
        Self {
            stage: 1,
            grid_cols: 4,
            grid_rows: 2,
            grid_spacing: 70.0,
            num_pairs: 4,
            initial_reveal_time: 3.0,
            mismatch_delay: 1.5,
        }
    }
}

impl GameDifficulty {
    /// Progress to next stage and recalculate difficulty parameters
    pub fn advance_stage(&mut self) {
        self.stage += 1;
        self.recalculate_difficulty();
    }

    /// Calculate difficulty parameters based on current stage
    fn recalculate_difficulty(&mut self) {
        // Helper function for hockey stick curve
        // starts steep, then levels off
        fn hockey_stick_curve(stage: u32, min: f32, max: f32, steepness: f32) -> f32 {
            let x = stage as f32;
            (max - min).mul_add(1.0 - (-x * steepness).exp(), min)
        }

        // Example calculations:

        // Grid size grows quickly at first, then slowly
        let total_cards = hockey_stick_curve(self.stage, 8.0, 24.0, 0.3) as u32;
        // Adjust grid dimensions based on total cards
        self.grid_cols = (total_cards as f32).sqrt().ceil() as u32;
        self.grid_rows = total_cards.div_ceil(self.grid_cols);
        self.num_pairs = (total_cards / 2) as usize;

        // Times decrease quickly at first, then stabilize
        self.initial_reveal_time = hockey_stick_curve(self.stage, 3.0, 1.0, 0.4);
        self.mismatch_delay = hockey_stick_curve(self.stage, 1.5, 0.5, 0.3);
    }
}

/// System to check if all cards are matched and stage is complete
fn check_stage_completion(cards: Query<&Card>, mut stage_state: ResMut<StageState>) {
    // Only check if we haven't already marked the stage as complete
    if stage_state.stage_complete {
        return;
    }

    // Get total number of cards
    let total_cards = cards.iter().count();
    if total_cards == 0 {
        return;
    }

    // Check if all cards are matched (face up and locked)
    let matched_cards = cards
        .iter()
        .filter(|card| card.face_up && card.locked)
        .count();

    // If all cards are matched, mark stage as complete
    if matched_cards == total_cards {
        stage_state.stage_complete = true;
        stage_state.transition_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
    }
}

/// System to handle stage transitions
fn handle_stage_transition(
    mut commands: Commands,
    time: Res<Time>,
    mut stage_state: ResMut<StageState>,
    mut game_difficulty: ResMut<GameDifficulty>,
    mut game_state: ResMut<GameState>,
    cards: Query<Entity, With<Card>>,
) {
    // Check if we're in transition and the timer is active
    if let Some(timer) = &mut stage_state.transition_timer {
        if timer.tick(time.delta()).just_finished() {
            // Despawn all existing cards
            for card_entity in cards.iter() {
                commands.entity(card_entity).despawn_recursive();
            }

            // Advance to next stage
            game_difficulty.advance_stage();

            // Reset game state for new stage
            *game_state = GameState::new();
            game_state.initial_wait_timer = Some(Timer::from_seconds(
                game_difficulty.initial_reveal_time,
                TimerMode::Once,
            ));

            // Reset stage state
            stage_state.stage_complete = false;
            stage_state.transition_timer = None;
        }
    }
}
