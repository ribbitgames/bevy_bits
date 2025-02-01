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
/// Fixed timestep duration for physics simulation (60 updates per second)
const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Global Resource: PhysicsTime
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Global resource that accumulates delta time for physics updates.
#[derive(Resource)]
pub struct PhysicsTime {
    /// Accumulated time not yet simulated.
    pub accumulator: f32,
}

impl Default for PhysicsTime {
    fn default() -> Self {
        Self { accumulator: 0.0 }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Component: Obstacle
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Component representing an obstacle with its physics state.
///
/// This component stores both the previous state (position and rotation) and the computed target
/// state. The render system will interpolate between these states.
#[derive(Component)]
pub struct Obstacle {
    /// The speed of the obstacle.
    pub speed: f32,
    /// The index of the emoji used for the obstacle.
    /// Renamed to `_emoji_index` to indicate that it is intentionally unused.
    pub _emoji_index: usize,
    /// The collision radius of the obstacle.
    pub radius: f32,
    /// The current velocity vector.
    pub velocity: Vec2,
    /// Position from the last physics update.
    pub previous_pos: Vec2,
    /// Position computed in the current physics update.
    pub target_pos: Vec2,
    /// Rotation from the last physics update.
    pub previous_rotation: Quat,
    /// Rotation computed in the current physics update.
    pub target_rotation: Quat,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Resource: SpawnTimer
////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Plugin Registration
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Plugin that registers all obstacle-related systems.
pub struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsTime>()
            .init_resource::<SpawnTimer>()
            // Spawn obstacles when the game state is Playing.
            .add_systems(Update, spawn_obstacles.run_if(in_state(GameState::Playing)))
            // Run the fixed timestep physics update first.
            .add_systems(
                Update,
                obstacle_physics_update.run_if(in_state(GameState::Playing)),
            )
            // Then run the interpolation system; it must run after the physics update.
            .add_systems(
                Update,
                interpolate_obstacles
                    .run_if(in_state(GameState::Playing))
                    .after(obstacle_physics_update),
            );
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// System: spawn_obstacles
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Spawns new obstacles at the top of the screen with a random horizontal offset.
///
/// This system selects a random emoji from the atlas and creates an obstacle entity with initial
/// physics state where both previous and target positions (and rotations) are identical.
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
                    _emoji_index: emoji_index,
                    radius: size / 2.0,
                    // Start moving downward at the given speed.
                    velocity: Vec2::new(0.0, -speed),
                    // Initialize both positions to the starting point.
                    previous_pos: start_pos,
                    target_pos: start_pos,
                    // Initialize rotations to identity.
                    previous_rotation: Quat::IDENTITY,
                    target_rotation: Quat::IDENTITY,
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// System: obstacle_physics_update
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Fixed timestep physics update system.
///
/// This system accumulates time using the global `PhysicsTime` resource and performs physics updates
/// at a fixed rate. For each fixed update, it:
/// - Copies the current target state into the previous state.
/// - Updates the velocity toward a target (which includes a horizontal wobble).
/// - Computes new target position and rotation.
fn obstacle_physics_update(
    time: Res<Time>,
    mut physics_time: ResMut<PhysicsTime>,
    mut query: Query<(&mut Obstacle, &mut Transform)>,
) {
    let dt = time.delta_secs().min(1.0 / 30.0);
    physics_time.accumulator += dt;

    // Determine the number of fixed steps to perform.
    let steps = (physics_time.accumulator / FIXED_TIMESTEP).floor() as u32;
    for _ in 0..steps {
        // Iterate over mutable references to query items.
        for (mut obstacle, transform) in &mut query {
            // Calculate all new values first.
            let new_previous_pos = obstacle.target_pos;
            let new_previous_rotation = transform.rotation;

            let wobble = (obstacle.target_pos.y * 0.1).sin() * 5.0;
            let target_velocity = Vec2::new(wobble, -obstacle.speed);
            let new_velocity = obstacle.velocity.lerp(target_velocity, 0.15);
            let new_target_pos = obstacle.target_pos + new_velocity * FIXED_TIMESTEP;

            let rotation_speed = new_velocity.x * 0.01;
            let new_target_rotation =
                transform.rotation * Quat::from_rotation_z(rotation_speed * FIXED_TIMESTEP);

            // Apply all updates at once.
            obstacle.previous_pos = new_previous_pos;
            obstacle.previous_rotation = new_previous_rotation;
            obstacle.velocity = new_velocity;
            obstacle.target_pos = new_target_pos;
            obstacle.target_rotation = new_target_rotation;
        }
        physics_time.accumulator -= FIXED_TIMESTEP;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// System: interpolate_obstacles
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Render interpolation system.
///
/// This system runs every frame and interpolates each obstacle's position and rotation between
/// their previous and target states based on the physics accumulator.
fn interpolate_obstacles(
    physics_time: Res<PhysicsTime>,
    mut query: Query<(&Obstacle, &mut Transform)>,
) {
    let alpha = physics_time.accumulator / FIXED_TIMESTEP;
    for (obstacle, mut transform) in &mut query {
        // Copy fields from obstacle into local variables.
        let prev_pos = obstacle.previous_pos;
        let targ_pos = obstacle.target_pos;
        let interpolated_pos = prev_pos.lerp(targ_pos, alpha);
        let current_z = transform.translation.z;
        transform.translation = Vec3::new(interpolated_pos.x, interpolated_pos.y, current_z);

        let prev_rot = obstacle.previous_rotation;
        let targ_rot = obstacle.target_rotation;
        let interpolated_rot = prev_rot.slerp(targ_rot, alpha);
        transform.rotation = interpolated_rot;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Collision Helper Function
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Checks collision between two circular objects by comparing the squared distance
/// to the square of the sum of their radii.
///
/// # Arguments
/// - `pos1`: The position of the first object.
/// - `radius1`: The collision radius of the first object.
/// - `pos2`: The position of the second object.
/// - `radius2`: The collision radius of the second object.
///
/// # Returns
/// - `true` if the objects collide; otherwise, `false`.
pub fn check_collision(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> bool {
    pos1.distance_squared(pos2) < (radius1 + radius2).powi(2)
}
