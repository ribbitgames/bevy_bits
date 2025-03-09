use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::{GameProgress, GameState, LevelSettings};

pub struct PhysicsPlugin;

// Constants for physics settings
pub const BLOCK_DENSITY: f32 = 1.0;
pub const BLOCK_FRICTION: f32 = 0.9;
pub const BLOCK_RESTITUTION: f32 = 0.05;
pub const STABILITY_CHECK_INTERVAL: f32 = 1.0;

// Threshold for movement to detect instability
pub const INSTABILITY_THRESHOLD: f32 = 3.5;

/// Component to mark entities that should be cleaned up when exiting the game state
#[derive(Component)]
pub struct GameEntity;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            // Enable debug visualization
            .add_plugins(RapierDebugRenderPlugin::default())
            // Set up basic resources
            .insert_resource(StabilityCheckTimer(Timer::from_seconds(
                STABILITY_CHECK_INTERVAL,
                TimerMode::Repeating,
            )))
            .insert_resource(GravitySettings(5.0))
            // Only keep essential systems
            .add_systems(
                Update,
                check_tower_stability.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, update_physics_settings)
            // Add despawn system on exit
            .add_systems(OnExit(GameState::Playing), despawn_game_entities);
    }
}

/// System to despawn all game entities when exiting the Playing state
pub fn despawn_game_entities(
    mut commands: Commands,
    query: Query<Entity, With<GameEntity>>,
    interaction_text_query: Query<Entity, With<crate::game::InteractionStateText>>,
) {
    // Clean up all game entities
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }

    // Clean up interaction text UI
    for entity in &interaction_text_query {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Resource)]
struct StabilityCheckTimer(Timer);

/// Resource to store gravity settings
#[derive(Resource)]
struct GravitySettings(f32);

/// Component to mark blocks in the tower
#[derive(Component)]
pub struct TowerBlock {
    pub removable: bool,
    pub being_grabbed: bool,
    pub initial_position: Vec2,
}

/// System to update physics settings based on level
fn update_physics_settings(
    level_settings: Res<LevelSettings>,
    mut gravity: ResMut<GravitySettings>,
    mut gravity_scales: Query<&mut GravityScale>,
    in_game: Res<State<GameState>>,
) {
    // Only update in the Playing state
    if *in_game.get() != GameState::Playing {
        return;
    }

    // Update gravity setting
    gravity.0 = level_settings.gravity;

    // Apply gravity to all rigid bodies
    for mut gravity_scale in &mut gravity_scales {
        gravity_scale.0 = gravity.0;
    }
}

/// System to check if the tower has become unstable
fn check_tower_stability(
    time: Res<Time>,
    mut stability_timer: ResMut<StabilityCheckTimer>,
    blocks: Query<(&Transform, &TowerBlock, &Velocity)>,
    mut game_progress: ResMut<GameProgress>,
) {
    // Skip if blocks are still in their initial wait period
    if game_progress.initial_wait_timer.is_some() {
        return;
    }

    // Only check stability periodically
    if !stability_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    // Skip stability check if the game is already over or if blocks are still being grabbed
    if game_progress.tower_collapsed || game_progress.level_complete {
        return;
    }

    // Check if any blocks have both moved significantly AND have high velocity
    let mut unstable_count = 0;
    let mut blocks_with_high_velocity = 0;

    for (transform, block, velocity) in &blocks {
        // Skip blocks being actively grabbed
        if block.being_grabbed {
            continue;
        }

        let current_pos = transform.translation.truncate();
        let displacement = (current_pos - block.initial_position).length();
        let velocity_magnitude = velocity.linvel.length();

        // Count blocks that moved significantly
        if displacement > INSTABILITY_THRESHOLD {
            unstable_count += 1;

            // Also track which of these have high velocity (still moving)
            if velocity_magnitude > 2.0 {
                blocks_with_high_velocity += 1;
            }
        }
    }

    // Tower is only collapsed if multiple blocks are unstable AND some are still moving
    if unstable_count > 7 && blocks_with_high_velocity > 2 {
        game_progress.record_tower_collapse();
    }
}

/// Creates a floor collider to prevent blocks from falling off-screen
pub fn spawn_floor(mut commands: Commands) {
    commands.spawn((
        Collider::cuboid(800.0, 20.0),
        Transform::from_xyz(0.0, -300.0, 0.0),
        RigidBody::Fixed,
        Friction::coefficient(1.0),
        Restitution::coefficient(0.0),
        GameEntity,
    ));
}

/// Creates walls on sides to keep blocks from falling off sides
pub fn spawn_walls(mut commands: Commands) {
    // Left wall
    commands.spawn((
        Collider::cuboid(10.0, 600.0),
        Transform::from_xyz(-270.0, 0.0, 0.0),
        RigidBody::Fixed,
        Friction::coefficient(0.7),
        Restitution::coefficient(0.1),
        GameEntity,
    ));

    // Right wall
    commands.spawn((
        Collider::cuboid(10.0, 600.0),
        Transform::from_xyz(270.0, 0.0, 0.0),
        RigidBody::Fixed,
        Friction::coefficient(0.7),
        Restitution::coefficient(0.1),
        GameEntity,
    ));
}
