use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EMOJI_SIZE, EmojiAtlas};

use crate::game::{GameState, GameTimer, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::player::PLAYER_WIDTH;

/// Minimum and maximum scale factors for emoji obstacles
const BASE_EMOJI_SIZE: f32 = EMOJI_SIZE.x as f32;
const OBSTACLE_MIN_SCALE: f32 = 0.8;
const OBSTACLE_MAX_SCALE: f32 = 2.0;

/// Initial time interval (in seconds) between obstacle spawns
const INITIAL_OBSTACLE_SPAWN_RATE: f32 = 0.7;
/// Base obstacle speed (in pixels per second)
const INITIAL_OBSTACLE_SPEED: f32 = 150.0;
/// Maximum speed cap (in pixels per second) to keep game playable
const MAX_OBSTACLE_SPEED: f32 = 500.0;
/// Rate at which obstacle speed increases over time
const DIFFICULTY_INCREASE_RATE: f32 = 0.3;
/// Minimum allowed spawn interval (in seconds)
const MIN_SPAWN_INTERVAL: f32 = 0.3;
/// Minimum rotation speed in radians per second (approx -2π)
const MIN_ROTATION_SPEED: f32 = -1.5 * std::f32::consts::PI;
/// Maximum rotation speed in radians per second (approx 2π)
const MAX_ROTATION_SPEED: f32 = 1.5 * std::f32::consts::PI;
/// Minimum required gap width for player passage (with some buffer)
const MINIMUM_PATH_WIDTH: f32 = PLAYER_WIDTH + 20.0;

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
            )
            .add_systems(OnExit(GameState::Playing), despawn_obstacles);
    }
}

fn despawn_obstacles(mut commands: Commands, query: Query<Entity, With<Obstacle>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Checks if a potential obstacle position would block all paths
fn would_block_all_paths(
    potential_pos: Vec2,
    potential_size: f32,
    query: &Query<(&Transform, &Obstacle)>,
    window_width: f32,
) -> bool {
    // Get all obstacles in the same horizontal band
    let relevant_obstacles: Vec<(Vec2, f32)> = query
        .iter()
        .filter(|(transform, _)| {
            (transform.translation.y - potential_pos.y).abs() < MINIMUM_PATH_WIDTH
        })
        .map(|(transform, obstacle)| (transform.translation.truncate(), obstacle.radius * 2.0))
        .collect();

    // If there are no nearby obstacles, position is safe
    if relevant_obstacles.is_empty() {
        return false;
    }

    // Add the potential new obstacle to the list
    let mut all_positions = relevant_obstacles;
    all_positions.push((potential_pos, potential_size));

    // Sort by x position for gap checking
    all_positions.sort_by(|a, b| {
        a.0.x
            .partial_cmp(&b.0.x)
            .expect("Failed to compare positions")
    });

    // Check for gaps between obstacles
    let mut prev_right = -window_width / 2.0;
    for (pos, size) in all_positions {
        let left_edge = pos.x - size / 2.0;
        let gap_width = left_edge - prev_right;

        if gap_width >= MINIMUM_PATH_WIDTH {
            return false; // Found a valid gap
        }

        prev_right = pos.x + size / 2.0;
    }

    // Check final gap on right side
    let final_gap = window_width / 2.0 - prev_right;
    final_gap < MINIMUM_PATH_WIDTH
}

fn spawn_obstacles(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    mut spawn_timer: ResMut<SpawnTimer>,
    game_timer: Res<GameTimer>,
    time: Res<Time>,
    obstacle_query: Query<(&Transform, &Obstacle)>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        let mut attempts = 0;
        const MAX_ATTEMPTS: i32 = 10;

        while attempts < MAX_ATTEMPTS {
            let scale = fastrand::f32()
                .mul_add(OBSTACLE_MAX_SCALE - OBSTACLE_MIN_SCALE, OBSTACLE_MIN_SCALE);
            let size = BASE_EMOJI_SIZE * scale;

            let x_range = WINDOW_WIDTH - size;
            let x = fastrand::f32().mul_add(x_range, -(x_range / 2.0));
            let start_pos = Vec2::new(x, WINDOW_HEIGHT / 2.0 + size / 2.0);

            if !would_block_all_paths(start_pos, size, &obstacle_query, WINDOW_WIDTH) {
                let available_emojis = emoji::get_random_emojis(&atlas, &validation, 1);
                if let Some(&emoji_index) = available_emojis.first() {
                    let speed = calculate_speed(game_timer.0);

                    let rotation_speed = if fastrand::f32() < 0.75 {
                        fastrand::f32()
                            .mul_add(MAX_ROTATION_SPEED - MIN_ROTATION_SPEED, MIN_ROTATION_SPEED)
                    } else {
                        0.0
                    };

                    // Create transform for obstacle
                    let obstacle_transform = Transform::from_xyz(start_pos.x, start_pos.y, 0.0)
                        .with_scale(Vec3::splat(scale));

                    if let Some(obstacle_entity) = emoji::spawn_emoji(
                        &mut commands,
                        &atlas,
                        &validation,
                        emoji_index,
                        obstacle_transform,
                    ) {
                        commands.entity(obstacle_entity).insert(Obstacle {
                            speed,
                            _emoji_index: emoji_index,
                            radius: size / 2.0,
                            rotation_speed,
                        });
                    }
                }
                break;
            }
            attempts += 1;
        }

        let new_spawn_rate = game_timer
            .0
            .mul_add(-0.02, INITIAL_OBSTACLE_SPAWN_RATE)
            .max(MIN_SPAWN_INTERVAL);
        spawn_timer.0 = Timer::from_seconds(new_spawn_rate, TimerMode::Repeating);
    }
}

/// Helper function to calculate current speed based on game time
fn calculate_speed(game_time: f32) -> f32 {
    (game_time.mul_add(DIFFICULTY_INCREASE_RATE, INITIAL_OBSTACLE_SPEED)).min(MAX_OBSTACLE_SPEED)
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
