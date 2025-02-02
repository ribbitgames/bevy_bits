use bevy::prelude::*;

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

#[derive(Event)]
pub struct ChainEvent(pub u32);

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
    pub grid_spacing: f32,     // Distance between emoji centers
    pub emoji_scale: f32,      // Scale factor for emojis
    pub num_emoji_types: usize,
}

impl Default for LevelConfig {
    fn default() -> Self {
        // Screen dimensions and margins
        let window_width = bits_helpers::WINDOW_WIDTH;
        let window_height = bits_helpers::WINDOW_HEIGHT;
        let ui_margin = 50.0_f32;

        // Grid dimensions
        let cols = 6;
        let rows = 8;

        // Emoji sizing
        let emoji_base_size = 128.0_f32; // Base size of emoji sprites
        let emoji_scale = 0.4_f32; // Reduce emoji size to fit better
        let emoji_rendered_size = emoji_base_size * emoji_scale;
        let minimum_padding = 15.0_f32; // Minimum space between emojis

        // Calculate minimum spacing needed between emoji centers
        let min_spacing = emoji_rendered_size + minimum_padding;

        // Calculate available space
        let available_width = window_width - (ui_margin * 2.0);
        let available_height = window_height - (ui_margin * 2.0);

        // Calculate spacing that will fit within screen bounds
        let spacing_by_width = available_width / (cols as f32);
        let spacing_by_height = available_height / (rows as f32);

        // Use the larger of minimum spacing and smaller of width/height calculations
        let grid_spacing = f32::max(min_spacing, f32::min(spacing_by_width, spacing_by_height));

        Self {
            grid_size: (rows, cols),
            grid_spacing,
            emoji_scale,
            num_emoji_types: 6,
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameProgress>()
            .init_resource::<LevelConfig>()
            .add_event::<ChainEvent>()
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

fn handle_scoring(mut progress: ResMut<GameProgress>, mut chain_events: EventReader<ChainEvent>) {
    for ChainEvent(chain_count) in chain_events.read() {
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
