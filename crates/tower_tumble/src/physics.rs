use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::{GameProgress, GameState, LevelSettings};

pub struct PhysicsPlugin;

// Constants for physics settings
pub const BLOCK_DENSITY: f32 = 1.0;
pub const BLOCK_FRICTION: f32 = 0.7;
pub const BLOCK_RESTITUTION: f32 = 0.2; // Slightly bouncy
pub const STABILITY_CHECK_INTERVAL: f32 = 0.5; // How often to check tower stability (seconds)

// Threshold for movement to detect instability
pub const INSTABILITY_THRESHOLD: f32 = 1.0;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugins(RapierDebugRenderPlugin::default()) // Debug rendering for development
            .insert_resource(StabilityCheckTimer(Timer::from_seconds(
                STABILITY_CHECK_INTERVAL,
                TimerMode::Repeating,
            )))
            .insert_resource(GravitySettings(9.8)) // Default gravity
            .add_systems(Update, apply_block_physics)
            .add_systems(Update, sync_physics_entities)
            .add_systems(Update, check_tower_stability)
            .add_systems(Update, update_physics_settings);
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
    mut query: Query<&mut GravityScale>, // Changed from RigidBodyForces to GravityScale
    in_game: Res<State<GameState>>,
) {
    // Only update in the Playing state
    if *in_game.get() != GameState::Playing {
        return;
    }

    // Update gravity setting
    gravity.0 = level_settings.gravity;

    // Apply gravity to all rigid bodies
    for mut gravity_scale in &mut query {
        gravity_scale.0 = gravity.0; // GravityScale is a simple f32
    }
}

/// System to check if the tower has become unstable
fn check_tower_stability(
    time: Res<Time>,
    mut stability_timer: ResMut<StabilityCheckTimer>,
    blocks: Query<(&Transform, &TowerBlock)>,
    mut game_progress: ResMut<GameProgress>,
    in_game: Res<State<GameState>>,
) {
    // Only check stability when in playing state
    if *in_game.get() != GameState::Playing {
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

    // Check if any blocks have moved significantly from their initial positions
    let mut unstable_count = 0;
    for (transform, block) in &blocks {
        let current_pos = transform.translation.truncate();
        let displacement = (current_pos - block.initial_position).length();

        // Count blocks that moved significantly
        if displacement > INSTABILITY_THRESHOLD && !block.being_grabbed {
            unstable_count += 1;
        }
    }

    // If too many blocks are unstable, consider the tower collapsed
    if unstable_count > 3 {
        game_progress.record_tower_collapse();
    }
}

/// Creates a floor collider to prevent blocks from falling off-screen
pub fn spawn_floor(mut commands: Commands) {
    commands.spawn((
        Collider::cuboid(500.0, 10.0),
        Transform::from_xyz(0.0, -300.0, 0.0),
        RigidBody::Fixed,
    ));
}

/// Creates walls on sides to keep blocks from falling off sides
pub fn spawn_walls(mut commands: Commands) {
    // Left wall
    commands.spawn((
        Collider::cuboid(10.0, 600.0),
        Transform::from_xyz(-270.0, 0.0, 0.0),
        RigidBody::Fixed,
    ));

    // Right wall
    commands.spawn((
        Collider::cuboid(10.0, 600.0),
        Transform::from_xyz(270.0, 0.0, 0.0),
        RigidBody::Fixed,
    ));
}

/// System to apply physics properties to newly created blocks
fn apply_block_physics(
    mut commands: Commands,
    new_blocks: Query<(Entity, &crate::blocks::BlockSprite), Added<crate::blocks::BlockSprite>>,
    gravity: Res<GravitySettings>,
) {
    for (entity, _) in &new_blocks {
        // Add physics components to blocks
        commands.entity(entity).insert((
            RigidBody::Dynamic,
            Friction::coefficient(BLOCK_FRICTION),
            Restitution::coefficient(BLOCK_RESTITUTION),
            ColliderMassProperties::Density(BLOCK_DENSITY),
            Sleeping::disabled(),
            Velocity::zero(),
            ExternalForce::default(),
            GravityScale(gravity.0), // Replaced RigidBodyForces with GravityScale
            ActiveEvents::COLLISION_EVENTS,
        ));
    }
}

/// System to synchronize physics entities with visual entities
fn sync_physics_entities(
    blocks: Query<(&Transform, &TowerBlock), With<crate::blocks::BlockSprite>>,
    mut physics_entities: Query<
        &mut Transform,
        (With<RigidBody>, Without<crate::blocks::BlockSprite>),
    >,
    mut external_forces: Query<&mut ExternalForce>,
) {
    // Find all visual blocks and their corresponding physics entities
    for (block_transform, tower_block) in &blocks {
        // Find physics entities near this block's position
        for mut physics_transform in &mut physics_entities {
            let distance = block_transform
                .translation
                .truncate()
                .distance(physics_transform.translation.truncate());
            if distance < 1.0 {
                // Close enough to be the same block
                // Update the physics position if the block is being grabbed
                if tower_block.being_grabbed {
                    physics_transform.translation = block_transform.translation;

                    // Try to find and reset external force to avoid accumulation
                    for mut ext_force in &mut external_forces {
                        ext_force.force = Vec2::ZERO;
                    }
                }
            }
        }
    }
}
