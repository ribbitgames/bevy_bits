use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};

pub struct GamePlugin;

const INITIAL_MOVES: u32 = 15;
const BASE_MATCH_SCORE: u32 = 100;
const CHAIN_MULTIPLIER: u32 = 50;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Resource)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

#[derive(Resource)]
pub struct GameProgress {
    pub score: u32,
    pub moves_remaining: u32,
    pub level: u32,
}

impl Default for GameProgress {
    fn default() -> Self {
        Self {
            score: 0,
            moves_remaining: INITIAL_MOVES,
            level: 1,
        }
    }
}

#[derive(Resource)]
pub struct LevelConfig {
    pub grid_size: (u32, u32), // (rows, columns)
    pub grid_spacing: f32,
    pub num_emoji_types: usize,
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            grid_size: (8, 6),  // Standard match-3 size that works well on mobile
            grid_spacing: 70.0, // Maintaining your existing spacing
            num_emoji_types: 6, // Standard number of different emoji types
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameProgress>()
            .init_resource::<LevelConfig>()
            .add_systems(
                Update,
                (handle_game_over, handle_scoring, update_difficulty)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn handle_game_over(progress: Res<GameProgress>, mut next_state: ResMut<NextState<GameState>>) {
    if progress.moves_remaining == 0 {
        next_state.set(GameState::GameOver);
    }
}

fn handle_scoring(
    mut progress: ResMut<GameProgress>,
    chain_count: u32, // This will be set by the grid system when matches occur
) {
    if chain_count > 0 {
        let base_score = BASE_MATCH_SCORE * chain_count;
        let chain_bonus = CHAIN_MULTIPLIER * (chain_count - 1);
        progress.score += base_score + chain_bonus;
        progress.moves_remaining -= 1;
    }
}

fn update_difficulty(mut config: ResMut<LevelConfig>, progress: Res<GameProgress>) {
    // Adjust difficulty based on level
    config.num_emoji_types = (6 + (progress.level / 3)).min(10) as usize;
}
