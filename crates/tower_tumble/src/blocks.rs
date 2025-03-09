use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::{GameState, LevelSettings};
use crate::physics::{GameEntity, TowerBlock};

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_tower)
            .add_systems(OnEnter(GameState::Playing), crate::physics::spawn_floor)
            .add_systems(OnEnter(GameState::Playing), crate::physics::spawn_walls);
    }
}

/// Component for block sprites
#[derive(Component)]
pub struct BlockSprite {
    pub color: Color,
}

// Constants for tower generation
const BASE_BLOCK_COLOR: Color = Color::rgb(0.7, 0.3, 0.2); // Reddish brown
const TOWER_BLOCK_COLOR: Color = Color::rgb(0.3, 0.5, 0.7); // Bluish
const TOWER_BASE_OFFSET: f32 = -270.0; // Lower the tower base to be closer to the floor

/// System to spawn a tower of blocks based on level settings
pub fn spawn_tower(mut commands: Commands, level_settings: Res<LevelSettings>) {
    let block_size = level_settings.block_size;
    let half_size = block_size / 2.0;

    // Calculate tower width in pixels
    let tower_width_pixels = level_settings.tower_width as f32 * block_size;

    // Start position (center of the bottom-left block)
    let start_x = -tower_width_pixels / 2.0 + half_size;
    let start_y = TOWER_BASE_OFFSET + 10.0; // Add small offset to ensure immediate contact with floor

    // Make base blocks have different properties
    for col in 0..level_settings.tower_width {
        // Calculate position for base row
        let x = start_x + col as f32 * block_size;
        let y = start_y;

        // Spawn base blocks with special properties
        spawn_base_block(
            &mut commands,
            Vec2::new(x, y),
            Vec2::new(block_size, block_size),
        );
    }

    // Now spawn rest of tower
    for row in 1..level_settings.tower_height {
        for col in 0..level_settings.tower_width {
            // Calculate position
            let x = start_x + col as f32 * block_size;
            let y = start_y + row as f32 * block_size;

            // Create normal block
            spawn_block(
                &mut commands,
                Vec2::new(x, y),
                Vec2::new(block_size, block_size),
                false, // Not base row
            );
        }
    }
}

/// Spawns a base block with special properties
fn spawn_base_block(commands: &mut Commands, position: Vec2, size: Vec2) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: BASE_BLOCK_COLOR,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(position.extend(0.0)),
            ..default()
        },
        BlockSprite {
            color: BASE_BLOCK_COLOR,
        },
        TowerBlock {
            removable: false, // Cannot be removed
            being_grabbed: false,
            initial_position: position,
        },
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        RigidBody::Fixed, // Base blocks are FIXED, not dynamic!
        GameEntity,       // Mark for cleanup when game ends
    ));
}

/// Spawns a single block at the given position
fn spawn_block(commands: &mut Commands, position: Vec2, size: Vec2, is_base: bool) {
    // Choose color based on whether it's a base block
    let color = if is_base {
        BASE_BLOCK_COLOR
    } else {
        TOWER_BLOCK_COLOR
    };

    // Use a much simpler physics setup
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(position.extend(0.0)),
            ..default()
        },
        BlockSprite { color },
        TowerBlock {
            removable: !is_base,
            being_grabbed: false,
            initial_position: position,
        },
        RigidBody::Dynamic, // Most important - this makes the block movable
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        Velocity::zero(),
        ExternalForce::default(),
        GravityScale(1.0),             // Normal gravity from the start
        Friction::coefficient(0.5),    // Lower friction to move more easily
        Restitution::coefficient(0.1), // Small bounce
        Damping {
            linear_damping: 0.1,  // Very low damping to allow easy movement
            angular_damping: 0.1, // Very low angular damping
        },
        GameEntity, // Mark for cleanup when game ends
    ));
}
