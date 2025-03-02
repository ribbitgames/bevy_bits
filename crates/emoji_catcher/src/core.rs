use bevy::prelude::*;

/// Game states that control the flow of the application
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

/// Component for falling emoji entities
#[derive(Component)]
pub struct FallingEmoji {
    /// Speed of the falling emoji
    pub speed: f32,
    /// Size of the emoji for collision detection
    pub size: f32,
    /// Whether this is the target emoji to catch
    pub is_target: bool,
}

/// Resource that stores the index of the target emoji to catch
#[derive(Resource, Default)]
pub struct TargetEmojiIndex(pub Option<usize>);

/// Component for the player's catcher
#[derive(Component)]
pub struct Catcher {
    /// Width of the catcher for collision detection
    pub width: f32,
}

/// Controls emoji spawning timing and difficulty
#[derive(Resource)]
pub struct SpawnTimer {
    /// Timer for spawning new emojis
    pub timer: Timer,
    /// Current base speed for new emojis
    pub current_speed: f32,
    /// Current spawn rate
    pub spawn_rate: f32,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            current_speed: 100.0,
            spawn_rate: 1.0,
        }
    }
}

/// Global game timer
#[expect(dead_code)]
#[derive(Resource, Default)]
pub struct GameTimer(pub Timer);

/// Tracks player's score
#[derive(Resource, Default)]
pub struct Score(pub i32);

/// Game configuration constants
pub mod config {
    use bevy::prelude::Vec2;

    // Catcher configuration
    pub const CATCHER_SIZE: Vec2 = Vec2::new(80.0, 40.0);

    // Emoji configuration
    pub const MIN_EMOJI_SIZE: f32 = 30.0;
    pub const MAX_EMOJI_SIZE: f32 = 60.0;
    pub const MAX_FALL_SPEED: f32 = 400.0;

    // Difficulty scaling
    pub const SPEED_INCREASE_RATE: f32 = 10.0; // Speed increase per second
    pub const MIN_SPAWN_INTERVAL: f32 = 0.5; // Minimum time between spawns
    pub const SPAWN_RATE_DECREASE: f32 = 0.05; // How much spawn interval decreases per second
}
