use bevy::prelude::*;

use crate::cards::Card;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameDifficulty>()
            .init_resource::<StageState>()
            .init_resource::<FlipState>()
            .insert_resource(GameState::new())
            .add_systems(
                Update,
                (
                    check_stage_completion,
                    handle_stage_transition,
                    handle_reveal_sequence,
                )
                    .chain(),
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

#[derive(Resource, Default)]
pub struct FlipState {
    /// Currently face-up cards that aren't locked
    pub face_up_cards: Vec<Entity>,
    /// Timer for automatic flip-down of unmatched pairs
    pub unmatch_timer: Option<Timer>,
}

#[derive(Resource, Default)]
pub struct GameState {
    /// Timer for initial face-down state
    pub initial_wait_timer: Option<Timer>,
    /// Timer for how long cards stay revealed
    pub reveal_timer: Option<Timer>,
    /// Whether we're in the initial reveal phase
    pub cards_revealed: bool,
    /// Number of mistakes made in current stage
    pub mistakes: u32,
    /// Maximum mistakes allowed before game over
    pub max_mistakes: u32,
    /// Whether the game is over due to too many mistakes
    pub game_over: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            initial_wait_timer: Some(Timer::from_seconds(1.0, TimerMode::Once)),
            reveal_timer: Some(Timer::from_seconds(2.0, TimerMode::Once)),
            cards_revealed: false,
            mistakes: 0,
            max_mistakes: 3,
            game_over: false,
        }
    }

    /// Records a mistake and checks if game is over
    pub fn record_mistake(&mut self) -> bool {
        self.mistakes += 1;
        if self.mistakes >= self.max_mistakes {
            self.game_over = true;
        }
        self.game_over
    }

    /// Checks if two cards match and updates game state
    pub fn check_for_match(
        &self,
        cards: &Query<(Entity, &Card)>, // Changed from &Query<&Card>
        card1: Entity,
        card2: Entity,
    ) -> bool {
        let (Ok((_, card1)), Ok((_, card2))) = (cards.get(card1), cards.get(card2)) else {
            return false;
        };

        card1.emoji_index == card2.emoji_index
    }

    /// Handles mismatch state and returns if game is over
    pub fn handle_mismatch(&mut self) -> bool {
        self.record_mistake()
    }

    /// Checks if all cards are matched
    pub fn check_all_matched(&self, cards: &Query<(Entity, &Card)>) -> bool {
        cards.iter().all(|(_, card)| card.face_up && card.locked)
    }

    /// Returns true if game is in a state where card interaction should be blocked
    pub const fn is_interaction_blocked(&self) -> bool {
        self.cards_revealed
            || self.reveal_timer.is_some()
            || self.initial_wait_timer.is_some()
            || self.game_over
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

/// System to handle the reveal sequence at the start of each stage
fn handle_reveal_sequence(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut cards: Query<&mut Card>,
) {
    // Handle initial wait timer
    if let Some(timer) = &mut game_state.initial_wait_timer {
        if timer.tick(time.delta()).just_finished() {
            // Initial wait is over, reveal all cards
            for mut card in &mut cards {
                card.face_up = true;
            }
            game_state.cards_revealed = true;
            game_state.initial_wait_timer = None;
        }
        return;
    }

    // Handle reveal timer
    if game_state.cards_revealed {
        if let Some(timer) = &mut game_state.reveal_timer {
            if timer.tick(time.delta()).just_finished() {
                // Reveal time is over, hide all cards
                for mut card in &mut cards {
                    card.face_up = false;
                }
                game_state.cards_revealed = false;
                game_state.reveal_timer = None;
            }
        }
    }
}
