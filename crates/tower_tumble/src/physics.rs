use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::{GameProgress, GameState, LevelSettings};

pub struct PhysicsPlugin;

// Constants for physics settings
pub const BLOCK_DENSITY: f32 = 1.0;
pub const BLOCK_FRICTION: f32 = 0.9; // Increased from 0.7
pub const BLOCK_RESTITUTION: f32 = 0.05; // Reduced from 0.1 for less bounce
pub const STABILITY_CHECK_INTERVAL: f32 = 1.0; // Increased from 0.5
pub const INITIAL_SETTLING_TIME: f32 = 5.0; // New constant for settling time

// Threshold for movement to detect instability
pub const INSTABILITY_THRESHOLD: f32 = 3.5; // Increased from 2.0 to be more forgiving

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            // Comment out debug rendering for release builds
            // .add_plugins(RapierDebugRenderPlugin::default())
            .insert_resource(StabilityCheckTimer(Timer::from_seconds(
                STABILITY_CHECK_INTERVAL,
                TimerMode::Repeating,
            )))
            .insert_resource(GravitySettings(5.0)) // Lower default gravity
            .add_systems(Update, apply_block_physics)
            .add_systems(Update, update_block_settling)
            .add_systems(Update, sync_physics_entities)
            .add_systems(
                Update,
                check_tower_stability.run_if(in_state(GameState::Playing)),
            )
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

/// Component to track the state of each block
#[derive(Component)]
pub struct BlockState {
    /// Tracks if the block is still settling
    pub settling: bool,
    /// Timer for settling phase
    pub settling_timer: Timer,
}

/// System to update physics settings based on level
fn update_physics_settings(
    level_settings: Res<LevelSettings>,
    mut gravity: ResMut<GravitySettings>,
    mut query: Query<(&mut GravityScale, Option<&BlockState>)>,
    in_game: Res<State<GameState>>,
) {
    // Only update in the Playing state
    if *in_game.get() != GameState::Playing {
        return;
    }

    // Update gravity setting
    gravity.0 = level_settings.gravity;

    // Apply gravity to all rigid bodies
    for (mut gravity_scale, block_state) in &mut query {
        // Skip blocks that are still in settling phase
        if let Some(state) = block_state {
            if state.settling {
                continue;
            }
        }

        gravity_scale.0 = gravity.0; // GravityScale is a simple f32
    }
}

/// System to gradually introduce gravity after blocks have settled
fn update_block_settling(
    time: Res<Time>,
    mut blocks: Query<(&mut BlockState, &mut GravityScale, &mut LockedAxes)>,
    gravity: Res<GravitySettings>,
    game_progress: Res<GameProgress>,
) {
    // Don't adjust gravity if the game is still in initial wait phase
    if game_progress.initial_wait_timer.is_some() {
        return;
    }

    for (mut state, mut gravity_scale, mut locked_axes) in &mut blocks {
        if state.settling {
            if state.settling_timer.tick(time.delta()).just_finished() {
                // Settling complete, apply normal gravity gradually
                state.settling = false;

                // Gradually increase gravity instead of immediately setting to full
                gravity_scale.0 = gravity.0 * 0.3; // Start with 30% of full gravity

                // Unlock all axes once settled
                *locked_axes = LockedAxes::empty();
            }
        } else if gravity_scale.0 < gravity.0 {
            // Continue gradually increasing gravity until reaching full gravity
            gravity_scale.0 = (gravity_scale.0 + 0.01).min(gravity.0);
        }
    }
}

/// System to check if the tower has become unstable
fn check_tower_stability(
    time: Res<Time>,
    mut stability_timer: ResMut<StabilityCheckTimer>,
    blocks: Query<(&Transform, &TowerBlock, Option<&BlockState>, &Velocity)>,
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

    // Count how many blocks are still settling
    let settling_blocks = blocks
        .iter()
        .filter(|(_, _, state, _)| state.map_or(false, |s| s.settling))
        .count();

    // Don't check stability if any blocks are still settling
    if settling_blocks > 0 {
        return;
    }

    // Check if any blocks have both moved significantly AND have high velocity
    let mut unstable_count = 0;
    let mut blocks_with_high_velocity = 0;

    for (transform, block, _, velocity) in &blocks {
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
    // Make the floor wider and thicker for better stability
    commands.spawn((
        Collider::cuboid(800.0, 20.0), // Wider and thicker
        Transform::from_xyz(0.0, -300.0, 0.0),
        RigidBody::Fixed,
        Friction::coefficient(1.0), // Higher friction to prevent sliding
        Restitution::coefficient(0.0), // No bounce from floor
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
    ));

    // Right wall
    commands.spawn((
        Collider::cuboid(10.0, 600.0),
        Transform::from_xyz(270.0, 0.0, 0.0),
        RigidBody::Fixed,
        Friction::coefficient(0.7),
        Restitution::coefficient(0.1),
    ));
}

/// System to apply physics properties to newly created blocks
fn apply_block_physics(
    mut commands: Commands,
    new_blocks: Query<
        (Entity, &crate::blocks::BlockSprite),
        (Added<crate::blocks::BlockSprite>, Without<RigidBody>),
    >,
    _gravity: Res<GravitySettings>,
) {
    for (entity, _) in &new_blocks {
        // Add physics components to blocks with the initial stable configuration
        commands.entity(entity).insert((
            RigidBody::Dynamic,
            Friction::coefficient(BLOCK_FRICTION),
            Restitution::coefficient(BLOCK_RESTITUTION),
            ColliderMassProperties::Density(BLOCK_DENSITY),
            Sleeping::disabled(),
            Velocity::zero(),
            ExternalForce::default(),
            GravityScale(0.001), // Almost no gravity during initial settling
            ActiveEvents::COLLISION_EVENTS,
            BlockState {
                settling: true,
                settling_timer: Timer::from_seconds(INITIAL_SETTLING_TIME, TimerMode::Once),
            },
            // Add damping to reduce jitter and vibration
            Damping {
                linear_damping: 0.95,  // Increased from 0.8
                angular_damping: 0.95, // Increased from 0.9
            },
            // Allow limited rotation for visual interest while maintaining stability
            LockedAxes::TRANSLATION_LOCKED_X | LockedAxes::TRANSLATION_LOCKED_Y,
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
