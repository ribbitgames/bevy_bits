use bevy::prelude::*;

// Game states
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

// Components
#[derive(Component)]
pub struct Marble {
    pub size: f32,
    pub is_target: bool,
}

#[derive(Component)]
pub struct Platform {
    pub width: f32,
}

#[derive(Component)]
pub struct Bucket {
    pub color: Color,
    pub width: f32,
}

// Resources
#[derive(Resource, Default)]
pub struct Score(pub i32);

#[derive(Resource)]
pub struct GameTimer {
    pub timer: Timer,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(30.0, TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct SpawnTimer {
    pub timer: Timer,
    pub spawn_rate: f32,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            spawn_rate: 2.0,
        }
    }
}

// Configuration
pub mod config {
    use bevy::prelude::*;

    pub const MARBLE_SIZE: f32 = 20.0;
    pub const MARBLE_RESTITUTION: f32 = 0.5;
    pub const PLATFORM_SIZE: Vec2 = Vec2::new(100.0, 10.0);
    pub const BUCKET_SIZE: Vec2 = Vec2::new(60.0, 20.0);
    pub const GRAVITY: f32 = 980.0; // Pixels/second^2
}
