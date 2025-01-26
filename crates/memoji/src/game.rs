use bevy::prelude::*;

use crate::cards::Card;

pub struct GamePlugin;

/// Fixed delay before showing cards at start of each stage (seconds)
const INITIAL_WAIT_TIME: f32 = 2.0;
/// Time per card to reveal during initial showing (seconds)
pub const REVEAL_TIME_PER_CARD: f32 = 0.5;
/// Maximum mistakes allowed before game over
const MAX_MISTAKES: u32 = 3;
/// Time to show all cards after game over (seconds)
const GAME_OVER_REVEAL_TIME: f32 = 3.0;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameDifficulty>()
            .init_resource::<StageState>()
            .init_resource::<FlipState>()
            .init_resource::<GameProgress>()
            .add_systems(Update, handle_stage_transition)
            .add_systems(
                Update,
                (handle_reveal_sequence, handle_game_over_sequence)
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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Resource)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    StageComplete,
    GameOver,
}

#[derive(Resource)]
pub struct GameProgress {
    /// Timer before showing cards at stage start
    pub initial_wait_timer: Option<Timer>,

    /// Timer controlling how long cards stay revealed
    pub reveal_timer: Option<Timer>,

    /// Whether cards are currently being shown to player
    pub cards_revealed: bool,

    /// Number of incorrect matches made
    pub mistakes: u32,

    /// Maximum mistakes allowed before game over
    pub max_mistakes: u32,

    /// Whether player has lost by exceeding max mistakes
    pub game_over: bool,

    /// Timer for showing all cards after game over
    pub game_over_reveal_timer: Option<Timer>,
}

impl Default for GameProgress {
    fn default() -> Self {
        Self {
            initial_wait_timer: Some(Timer::from_seconds(INITIAL_WAIT_TIME, TimerMode::Once)),
            reveal_timer: None, // Will be set based on card count when stage starts
            cards_revealed: false,
            mistakes: 0,
            max_mistakes: MAX_MISTAKES,
            game_over: false,
            game_over_reveal_timer: None,
        }
    }
}

impl GameProgress {
    /// Records a mistake and returns whether game is over
    /// If mistakes exceed maximum, triggers game over sequence
    pub fn record_mistake(&mut self) -> bool {
        self.mistakes += 1;
        if self.mistakes >= self.max_mistakes {
            self.game_over = true;
            self.game_over_reveal_timer =
                Some(Timer::from_seconds(GAME_OVER_REVEAL_TIME, TimerMode::Once));
        }
        self.game_over
    }

    /// Returns true if card interaction should be blocked
    /// Blocks interaction during: reveals, wait times, and game over
    pub const fn is_interaction_blocked(&self) -> bool {
        self.cards_revealed
            || self.reveal_timer.is_some()
            || self.initial_wait_timer.is_some()
            || self.game_over
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
}

impl Default for GameDifficulty {
    fn default() -> Self {
        Self {
            stage: 1,
            grid_cols: 4,
            grid_rows: 2,
            grid_spacing: 70.0,
            num_pairs: 4,
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
        // Start with 6 cards (3 pairs), add 2 cards (1 pair) every 2 stages
        let total_cards = 6 + (self.stage / 2) * 2;
        let total_cards = total_cards.min(24); // Cap at 24 cards

        // Adjust grid dimensions based on total cards
        self.grid_cols = (total_cards as f32).sqrt().ceil() as u32;
        self.grid_rows = total_cards.div_ceil(self.grid_cols);
        self.num_pairs = (total_cards / 2) as usize;
    }
}

#[derive(Resource, Default)]
pub struct FlipState {
    pub face_up_cards: Vec<Entity>,
    pub unmatch_timer: Option<Timer>,
}

fn handle_stage_transition(
    mut commands: Commands,
    time: Res<Time>,
    mut stage_state: ResMut<StageState>,
    mut game_difficulty: ResMut<GameDifficulty>,
    mut game_progress: ResMut<GameProgress>,
    mut next_state: ResMut<NextState<GameState>>,
    cards: Query<Entity, With<Card>>,
) {
    if let Some(timer) = &mut stage_state.transition_timer {
        if timer.tick(time.delta()).just_finished() {
            // After cleanup, transition to stage complete
            next_state.set(GameState::StageComplete);

            // Clear cards
            for card_entity in cards.iter() {
                commands.entity(card_entity).despawn_recursive();
            }

            // Prep next stage
            game_difficulty.advance_stage();
            *game_progress = GameProgress {
                initial_wait_timer: Some(Timer::from_seconds(INITIAL_WAIT_TIME, TimerMode::Once)),
                reveal_timer: Some(Timer::from_seconds(
                    (game_difficulty.grid_rows * game_difficulty.grid_cols) as f32
                        * REVEAL_TIME_PER_CARD,
                    TimerMode::Once,
                )),
                cards_revealed: false,
                mistakes: 0,
                max_mistakes: MAX_MISTAKES,
                game_over: false,
                game_over_reveal_timer: None,
            };

            stage_state.stage_complete = false;
            stage_state.transition_timer = None;
        }
    }
}

fn handle_reveal_sequence(
    time: Res<Time>,
    mut game_progress: ResMut<GameProgress>,
    mut cards: Query<&mut Card>,
) {
    if let Some(timer) = &mut game_progress.initial_wait_timer {
        if timer.tick(time.delta()).just_finished() {
            let total_cards = cards.iter().count();

            for mut card in &mut cards {
                card.face_up = true;
            }
            game_progress.cards_revealed = true;
            game_progress.initial_wait_timer = None;
            game_progress.reveal_timer = Some(Timer::from_seconds(
                total_cards as f32 * REVEAL_TIME_PER_CARD,
                TimerMode::Once,
            ));
        }
        return;
    }

    if game_progress.cards_revealed {
        if let Some(timer) = &mut game_progress.reveal_timer {
            if timer.tick(time.delta()).just_finished() {
                for mut card in &mut cards {
                    card.face_up = false;
                }
                game_progress.cards_revealed = false;
                game_progress.reveal_timer = None;
            }
        }
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
        // First reveal all cards
        for (_, mut card) in &mut cards {
            card.face_up = true;
        }

        if let Some(timer) = &mut game_progress.game_over_reveal_timer {
            if timer.tick(time.delta()).just_finished() {
                // Cleanup all cards
                for (entity, _) in cards.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                next_state.set(GameState::GameOver);
            }
        }
    }
}
