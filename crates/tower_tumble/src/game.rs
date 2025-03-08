use bevy::prelude::*;

pub struct GamePlugin;

/// Time to wait before allowing block interaction (seconds)
const INITIAL_WAIT_TIME: f32 = 1.0;
/// Maximum number of blocks that can be removed before tower collapse
const MAX_BLOCKS_REMOVED: u32 = 15;
/// Time limit for each level in seconds
const LEVEL_TIME_LIMIT: f32 = 90.0;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameProgress>()
            .init_resource::<LevelSettings>()
            .add_systems(Update, handle_level_transition)
            .add_systems(
                Update,
                (update_game_timer, check_level_complete)
                    .run_if(in_state(GameState::Playing))
                    .chain(),
            );
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    LevelComplete,
    GameOver,
}

#[derive(Resource)]
pub struct GameProgress {
    /// Current score
    pub score: u32,

    /// Timer for current level
    pub level_timer: Timer,

    /// Number of blocks safely removed
    pub blocks_removed: u32,

    /// Whether tower has collapsed
    pub tower_collapsed: bool,

    /// Whether level is completed
    pub level_complete: bool,

    /// Timer before allowing block interactions
    pub initial_wait_timer: Option<Timer>,
}

impl Default for GameProgress {
    fn default() -> Self {
        Self {
            score: 0,
            level_timer: Timer::from_seconds(LEVEL_TIME_LIMIT, TimerMode::Once),
            blocks_removed: 0,
            tower_collapsed: false,
            level_complete: false,
            initial_wait_timer: Some(Timer::from_seconds(INITIAL_WAIT_TIME, TimerMode::Once)),
        }
    }
}

impl GameProgress {
    /// Records a successful block removal and returns whether level is complete
    pub fn record_block_removal(&mut self) -> bool {
        self.blocks_removed += 1;
        self.score += 10; // Basic score per block

        // Level is complete when certain number of blocks are removed
        if self.blocks_removed >= MAX_BLOCKS_REMOVED {
            self.level_complete = true;
        }

        self.level_complete
    }

    /// Records tower collapse and game over
    pub fn record_tower_collapse(&mut self) {
        self.tower_collapsed = true;
    }

    /// Returns true if block interaction should be blocked
    /// Blocks interaction during initial wait and when level is complete/over
    pub fn is_interaction_blocked(&self) -> bool {
        self.initial_wait_timer.is_some() || self.level_complete || self.tower_collapsed
    }

    /// Adds time bonus to score based on remaining time
    pub fn add_time_bonus(&mut self) {
        let remaining_time = self.level_timer.remaining_secs();
        let bonus = (remaining_time as u32) * 5; // 5 points per second remaining
        self.score += bonus;
    }
}

#[derive(Resource, Debug)]
pub struct LevelSettings {
    /// Current level number (starts at 1)
    pub level: u32,

    /// Number of blocks in tower
    pub num_blocks: u32,

    /// Tower height (number of rows)
    pub tower_height: u32,

    /// Tower width (blocks per row)
    pub tower_width: u32,

    /// Block size in pixels
    pub block_size: f32,

    /// Gravity strength
    pub gravity: f32,
}

impl Default for LevelSettings {
    fn default() -> Self {
        Self {
            level: 1,
            num_blocks: 30,
            tower_height: 10,
            tower_width: 3,
            block_size: 50.0,
            gravity: 9.8,
        }
    }
}

impl LevelSettings {
    /// Progress to next level and recalculate settings
    pub fn advance_level(&mut self) {
        self.level += 1;
        self.recalculate_settings();
    }

    /// Calculate level settings based on current level
    fn recalculate_settings(&mut self) {
        // Increment tower height by 1 every 2 levels
        self.tower_height = 10 + (self.level / 2);

        // Gradually increase width for higher levels
        if self.level > 3 {
            self.tower_width = 3 + (self.level - 3) / 2;
        }

        // Adjust block count based on dimensions
        self.num_blocks = self.tower_height * self.tower_width;

        // Adjust gravity for higher levels (increases difficulty)
        self.gravity = 9.8 + (self.level as f32 * 0.5);
    }
}

/// System to update the level timer
fn update_game_timer(time: Res<Time>, mut game_progress: ResMut<GameProgress>) {
    // Update initial wait timer
    if let Some(timer) = &mut game_progress.initial_wait_timer {
        if timer.tick(time.delta()).just_finished() {
            game_progress.initial_wait_timer = None;
        }
        return;
    }

    // Update level timer if game is active
    if !game_progress.level_complete && !game_progress.tower_collapsed {
        if game_progress.level_timer.tick(time.delta()).just_finished() {
            // Time's up - set game over
            game_progress.tower_collapsed = true;
        }
    }
}

/// System to check if the level is complete or game is over
fn check_level_complete(
    game_progress: Res<GameProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if game_progress.level_complete {
        next_state.set(GameState::LevelComplete);
    } else if game_progress.tower_collapsed {
        next_state.set(GameState::GameOver);
    }
}

/// System to handle transition between levels
fn handle_level_transition(
    mut commands: Commands,
    mut level_settings: ResMut<LevelSettings>,
    mut game_progress: ResMut<GameProgress>,
    game_state: Res<State<GameState>>,
) {
    if *game_state.get() == GameState::LevelComplete {
        // Advance to next level
        level_settings.advance_level();

        // Reset game progress for next level
        *game_progress = GameProgress {
            score: game_progress.score, // Keep the score
            level_timer: Timer::from_seconds(LEVEL_TIME_LIMIT, TimerMode::Once),
            blocks_removed: 0,
            tower_collapsed: false,
            level_complete: false,
            initial_wait_timer: Some(Timer::from_seconds(INITIAL_WAIT_TIME, TimerMode::Once)),
        };
    }
}
