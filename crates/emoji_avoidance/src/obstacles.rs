use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use rand::prelude::*;

use crate::game::{GameState, GameTimer, WINDOW_HEIGHT, WINDOW_WIDTH};

const OBSTACLE_MIN_SIZE: f32 = 40.0;
const OBSTACLE_MAX_SIZE: f32 = 80.0;
const INITIAL_OBSTACLE_SPAWN_RATE: f32 = 1.0;
const INITIAL_OBSTACLE_SPEED: f32 = 150.0;
const DIFFICULTY_INCREASE_RATE: f32 = 0.1;
const MIN_SPAWN_INTERVAL: f32 = 0.3;
const FIXED_TIMESTEP: f32 = 1.0 / 60.0; // 60 FPS fixed timestep

#[derive(Component)]
pub struct Obstacle {
    pub speed: f32,
    pub emoji_index: usize,
    pub radius: f32,
    pub velocity: Vec2,
    pub previous_pos: Vec2,
    pub target_pos: Vec2,
    pub remainder: f32,
}

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

pub struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnTimer>().add_systems(
            Update,
            (spawn_obstacles.before(obstacle_movement), obstacle_movement)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

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
                    emoji_index,
                    radius: size / 2.0,
                    velocity: Vec2::new(0.0, -speed),
                    previous_pos: start_pos,
                    target_pos: start_pos,
                    remainder: 0.0,
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

/// Smoothstep interpolation between two points
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn obstacle_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Obstacle)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (entity, mut transform, mut obstacle) in query.iter_mut() {
        // Store all values we need before any mutations
        let mut step = delta_time + obstacle.remainder;
        let old_target = obstacle.target_pos;
        let old_velocity = obstacle.velocity;
        let current_speed = obstacle.speed;
        let current_y = obstacle.target_pos.y;
        let radius = obstacle.radius;

        // Pre-calculate the wobble and target velocity
        let wobble = (current_y * 0.1).sin() * 5.0;
        let target_velocity = Vec2::new(wobble, -current_speed);

        // Calculate new velocity and position
        let new_velocity = old_velocity.lerp(target_velocity, 0.1);
        let new_pos = old_target + new_velocity * FIXED_TIMESTEP;

        // Now perform all mutations
        obstacle.previous_pos = old_target;
        obstacle.velocity = new_velocity;
        obstacle.target_pos = new_pos;
        obstacle.remainder = step - FIXED_TIMESTEP;

        // Calculate interpolation
        let alpha = obstacle.remainder / FIXED_TIMESTEP;
        let smooth_alpha = smoothstep(0.0, 1.0, alpha);
        let interpolated_pos = old_target.lerp(new_pos, smooth_alpha);

        // Update transform
        transform.translation.x = interpolated_pos.x;
        transform.translation.y = interpolated_pos.y;

        // Add slight rotation based on horizontal velocity
        let rotation_speed = new_velocity.x * 0.01;
        transform.rotate_z(rotation_speed * FIXED_TIMESTEP);

        // Despawn obstacles that have moved off screen
        if transform.translation.y < -WINDOW_HEIGHT / 2.0 - radius {
            commands.entity(entity).despawn();
        }
    }
}

pub fn check_collision(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> bool {
    pos1.distance_squared(pos2) < (radius1 + radius2).powi(2)
}
