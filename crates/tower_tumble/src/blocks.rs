use avian2d::prelude::*;
use bevy::prelude::*;

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

#[derive(Component)]
pub struct BlockSprite {
    pub color: Color,
}

#[derive(Bundle)]
struct BlockBundle {
    sprite: Sprite,
    transform: Transform,
    global_transform: GlobalTransform,
    block_sprite: BlockSprite,
    tower_block: TowerBlock,
    collider: Collider,
    rigid_body: RigidBody,
    friction: Friction,
    restitution: Restitution,
    game_entity: GameEntity,
}

const BASE_BLOCK_COLOR: Color = Color::srgb(0.7, 0.3, 0.2);
const TOWER_BLOCK_COLOR: Color = Color::srgb(0.3, 0.5, 0.7);
const TOWER_BASE_OFFSET: f32 = -270.0;

pub fn spawn_tower(mut commands: Commands, level_settings: Res<LevelSettings>) {
    let block_size = level_settings.block_size;
    let half_size = block_size / 2.0;
    let tower_width_pixels = level_settings.tower_width as f32 * block_size;
    let start_x = -tower_width_pixels / 2.0 + half_size;
    let start_y = TOWER_BASE_OFFSET + 10.0;

    for col in 0..level_settings.tower_width {
        let x = start_x + col as f32 * block_size;
        let y = start_y;
        spawn_base_block(
            &mut commands,
            Vec2::new(x, y),
            Vec2::new(block_size, block_size),
        );
    }

    for row in 1..level_settings.tower_height {
        for col in 0..level_settings.tower_width {
            let x = start_x + col as f32 * block_size;
            let y = start_y + row as f32 * block_size;
            spawn_block(
                &mut commands,
                Vec2::new(x, y),
                Vec2::new(block_size, block_size),
                false,
            );
        }
    }
}

fn spawn_base_block(commands: &mut Commands, position: Vec2, size: Vec2) {
    commands.spawn(BlockBundle {
        sprite: Sprite {
            color: BASE_BLOCK_COLOR,
            custom_size: Some(size),
            ..default()
        },
        transform: Transform::from_translation(position.extend(0.0)),
        global_transform: GlobalTransform::default(),
        block_sprite: BlockSprite {
            color: BASE_BLOCK_COLOR,
        },
        tower_block: TowerBlock {
            removable: false,
            being_grabbed: false,
            initial_position: position,
        },
        collider: Collider::rectangle(size.x, size.y),
        rigid_body: RigidBody::Static,
        friction: Friction::new(0.5),
        restitution: Restitution::new(0.1),
        game_entity: GameEntity,
    });
}

fn spawn_block(commands: &mut Commands, position: Vec2, size: Vec2, is_base: bool) {
    let color = if is_base {
        BASE_BLOCK_COLOR
    } else {
        TOWER_BLOCK_COLOR
    };

    commands.spawn(BlockBundle {
        sprite: Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        transform: Transform::from_translation(position.extend(0.0)),
        global_transform: GlobalTransform::default(),
        block_sprite: BlockSprite { color },
        tower_block: TowerBlock {
            removable: !is_base,
            being_grabbed: false,
            initial_position: position,
        },
        rigid_body: RigidBody::Dynamic,
        collider: Collider::rectangle(size.x, size.y),
        friction: Friction::new(0.5),
        restitution: Restitution::new(0.1),
        game_entity: GameEntity,
    });
}