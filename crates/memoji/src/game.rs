use bevy::prelude::*;

use crate::cards::Card;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameDifficulty>()
            .init_resource::<StageState>()
            .init_resource::<FlipState>()
            .init_resource::<GameProgress>()
            .add_systems(
                Update,
                (
                    check_stage_completion,
                    handle_stage_transition,
                    handle_reveal_sequence.run_if(in_state(GameState::Playing)),
                )
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
    pub initial_wait_timer: Option<Timer>,
    pub reveal_timer: Option<Timer>,
    pub cards_revealed: bool,
    pub mistakes: u32,
    pub max_mistakes: u32,
    pub game_over: bool,
}

impl Default for GameProgress {
    fn default() -> Self {
        Self {
            initial_wait_timer: Some(Timer::from_seconds(1.0, TimerMode::Once)),
            reveal_timer: Some(Timer::from_seconds(2.0, TimerMode::Once)),
            cards_revealed: false,
            mistakes: 0,
            max_mistakes: 3,
            game_over: false,
        }
    }
}

impl GameProgress {
    pub fn record_mistake(&mut self) -> bool {
        self.mistakes += 1;
        if self.mistakes >= self.max_mistakes {
            self.game_over = true;
        }
        self.game_over
    }

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
    pub face_up_cards: Vec<Entity>,
    pub unmatch_timer: Option<Timer>,
}

fn check_stage_completion(
    cards: Query<&Card>,
    mut stage_state: ResMut<StageState>,
    game_progress: ResMut<GameProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if stage_state.stage_complete || game_progress.game_over {
        return;
    }

    let total_cards = cards.iter().count();
    if total_cards == 0 {
        return;
    }

    let matched_cards = cards
        .iter()
        .filter(|card| card.face_up && card.locked)
        .count();

    if matched_cards == total_cards {
        stage_state.stage_complete = true;
        stage_state.transition_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
        next_state.set(GameState::StageComplete);
    }

    if game_progress.game_over {
        next_state.set(GameState::GameOver);
    }
}

fn handle_stage_transition(
    mut commands: Commands,
    time: Res<Time>,
    mut stage_state: ResMut<StageState>,
    mut game_difficulty: ResMut<GameDifficulty>,
    mut game_progress: ResMut<GameProgress>,
    cards: Query<Entity, With<Card>>,
) {
    if let Some(timer) = &mut stage_state.transition_timer {
        if timer.tick(time.delta()).just_finished() {
            for card_entity in cards.iter() {
                commands.entity(card_entity).despawn_recursive();
            }

            game_difficulty.advance_stage();

            *game_progress = GameProgress {
                initial_wait_timer: Some(Timer::from_seconds(
                    game_difficulty.initial_reveal_time,
                    TimerMode::Once,
                )),
                ..default()
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
            for mut card in &mut cards {
                card.face_up = true;
            }
            game_progress.cards_revealed = true;
            game_progress.initial_wait_timer = None;
            return;
        }
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
