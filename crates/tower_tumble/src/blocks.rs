use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::{GameState, LevelSettings};
use crate::physics::TowerBlock;

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
const TOWER_BASE_OFFSET: f32 = -250.0; // Y offset for the base of the tower

/// System to spawn a tower of blocks based on level settings
pub fn spawn_tower(mut commands: Commands, level_settings: Res<LevelSettings>) {
    let block_size = level_settings.block_size;
    let half_size = block_size / 2.0;

    // Calculate tower width in pixels
    let tower_width_pixels = level_settings.tower_width as f32 * block_size;

    // Start position (center of the bottom-left block)
    let start_x = -tower_width_pixels / 2.0 + half_size;
    let start_y = TOWER_BASE_OFFSET;

    // Spawn blocks row by row
    for row in 0..level_settings.tower_height {
        for col in 0..level_settings.tower_width {
            let is_base_row = row == 0;

            // Calculate position
            let x = start_x + col as f32 * block_size;
            let y = start_y + row as f32 * block_size;

            // Create block
            spawn_block(
                &mut commands,
                Vec2::new(x, y),
                Vec2::new(block_size, block_size),
                is_base_row, // Base row blocks are not removable
            );
        }
    }
}

/// Spawns a single block at the given position
fn spawn_block(commands: &mut Commands, position: Vec2, size: Vec2, is_base: bool) {
    // Choose color based on whether it's a base block
    let color = if is_base {
        BASE_BLOCK_COLOR
    } else {
        TOWER_BLOCK_COLOR
    };

    // Create the visual block entity
    let entity = commands
        .spawn((
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
            Collider::cuboid(size.x / 2.0, size.y / 2.0),
        ))
        .id();
}
