use bevy::prelude::*;

/// Game states that control the flow of the application
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    StageComplete,
    GameOver,
}

/// Configuration for a single game stage
#[derive(Clone)]
pub struct Stage {
    /// Total number of emojis to spawn in the stage
    pub total_emojis: usize,
    /// Number of correct (target) emojis to find
    pub correct_emojis: usize,
    /// Base movement speed for emojis
    pub emoji_speed: f32,
    /// Time limit for the stage in seconds
    pub time_limit: f32,
}

/// Global stage configuration and tracking
#[derive(Resource)]
pub struct StageConfig {
    /// Current stage settings
    pub stage: Stage,
    /// Current stage number (1-based)
    pub current_stage_number: usize,
}

impl Default for StageConfig {
    fn default() -> Self {
        Self {
            stage: Stage {
                total_emojis: 30,
                correct_emojis: 5,
                emoji_speed: 100.0,
                time_limit: 20.0,
            },
            current_stage_number: 1,
        }
    }
}

/// Component for moving emoji entities
#[derive(Component)]
pub struct MovingEmoji {
    /// Index in the emoji atlas
    pub index: usize,
    /// Size of the emoji in pixels
    pub size: f32,
}

/// Component for emoji velocity
#[derive(Component)]
pub struct Velocity(pub Vec2);

/// Tracks total score across all stages
#[derive(Resource, Default)]
pub struct Score(pub i32);

/// Tracks currently targeted emoji
#[derive(Resource, Default)]
pub struct TargetEmojiInfo {
    pub index: usize,
}

/// Tracks number of correct emojis found in current stage
#[derive(Resource, Default)]
pub struct CorrectEmojisFound(pub usize);

/// Timer for current stage
#[derive(Resource)]
pub struct GameTimer(pub Timer);

impl Default for GameTimer {
    fn default() -> Self {
        Self(Timer::new(
            std::time::Duration::from_secs_f32(20.0),
            TimerMode::Once,
        ))
    }
}

/// Event fired when an emoji is clicked
#[derive(Event)]
pub struct EmojiClickedEvent {
    pub entity: Entity,
    pub position: Vec2,
    pub is_correct: bool,
}
