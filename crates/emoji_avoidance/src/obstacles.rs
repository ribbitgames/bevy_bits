use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use rand::prelude::*;

use crate::game::{GameState, GameTimer, WINDOW_HEIGHT, WINDOW_WIDTH};

/// Minimum obstacle size (in pixels)
const OBSTACLE_MIN_SIZE: f32 = 40.0;
/// Maximum obstacle size (in pixels)
const OBSTACLE_MAX_SIZE: f32 = 80.0;
/// Initial time interval (in seconds) between obstacle spawns
const INITIAL_OBSTACLE_SPAWN_RATE: f32 = 1.0;
/// Base obstacle speed (in pixels per second)
const INITIAL_OBSTACLE_SPEED: f32 = 150.0;
/// Rate at which obstacle speed increases over time
const DIFFICULTY_INCREASE_RATE: f32 = 0.1;
/// Minimum allowed spawn interval (in seconds)
const MIN_SPAWN_INTERVAL: f32 = 0.3;
/// Minimum rotation speed in radians per second (approx -2π)
const MIN_ROTATION_SPEED: f32 = -1.5 * std::f32::consts::PI;
/// Maximum rotation speed in radians per second (approx 2π)
const MAX_ROTATION_SPEED: f32 = 1.5 * std::f32::consts::PI;

/// Component representing an obstacle with its physics state.
#[derive(Component)]
pub struct Obstacle {
    /// The speed of the obstacle.
    pub speed: f32,
    /// The index of the emoji used for the obstacle.
    pub _emoji_index: usize,
    /// The collision radius of the obstacle.
    pub radius: f32,
    /// Rotation speed in radians per second
    pub rotation_speed: f32,
}

/// Resource that handles the timer used for obstacle spawning.
#[derive(Resource)]
pub struct SpawnTimer(Timer);

impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            INITIAL_OBSTACLE_SPAWN_RATE,
            TimerMode::Repeating,
        ))
    }
}

/// Plugin that registers all obstacle-related systems.
pub struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnTimer>()
            .add_systems(Update, spawn_obstacles.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                update_obstacles.run_if(in_state(GameState::Playing)),
            );
    }
}

/// Spawns new obstacles at the top of the screen with a random horizontal offset.
fn spawn_obstacles(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    mut spawn_timer: ResMut<SpawnTimer>,
    game_timer: Res<GameTimer>,
    time: Res<Time>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        let mut rng = rand::thread_rng();
        let size = rng.gen_range(OBSTACLE_MIN_SIZE..OBSTACLE_MAX_SIZE);
        let x = rng.gen_range(-WINDOW_WIDTH / 2.0 + size / 2.0..WINDOW_WIDTH / 2.0 - size / 2.0);
        let start_pos = Vec2::new(x, WINDOW_HEIGHT / 2.0 + size / 2.0);

        let available_emojis = emoji::get_random_emojis(&atlas, &validation, 1);
        if let Some(&emoji_index) = available_emojis.first() {
            let speed = game_timer
                .0
                .mul_add(DIFFICULTY_INCREASE_RATE, INITIAL_OBSTACLE_SPEED);

            // Random rotation speed between -2π and 2π radians per second
            // Some emojis won't rotate (25% chance)
            let rotation_speed = if rng.gen_bool(0.75) {
                rng.gen_range(MIN_ROTATION_SPEED..MAX_ROTATION_SPEED)
            } else {
                0.0
            };

            if let Some(obstacle_entity) = emoji::spawn_emoji(
                &mut commands,
                &atlas,
                &validation,
                emoji_index,
                start_pos,
                size / OBSTACLE_MAX_SIZE,
            ) {
                commands.entity(obstacle_entity).insert(Obstacle {
                    speed,
                    _emoji_index: emoji_index,
                    radius: size / 2.0,
                    rotation_speed,
                });
            }
        }

        let new_spawn_rate = game_timer
            .0
            .mul_add(-0.02, INITIAL_OBSTACLE_SPAWN_RATE)
            .max(MIN_SPAWN_INTERVAL);
        spawn_timer.0 = Timer::from_seconds(new_spawn_rate, TimerMode::Repeating);
    }
}

/// Updates obstacle positions and rotations using direct delta time.
fn update_obstacles(time: Res<Time>, mut query: Query<(&Obstacle, &mut Transform)>) {
    let dt = time.delta_secs();

    for (obstacle, mut transform) in &mut query {
        // Update position - moving straight down
        transform.translation.y -= obstacle.speed * dt;

        // Update rotation
        if obstacle.rotation_speed != 0.0 {
            transform.rotate(Quat::from_rotation_z(obstacle.rotation_speed * dt));
        }
    }
}

/// Checks collision between two circular objects.
pub fn check_collision(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> bool {
    pos1.distance_squared(pos2) < (radius1 + radius2).powi(2)
}
