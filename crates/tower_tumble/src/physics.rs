use avian2d::PhysicsPlugins;
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::game::{GameProgress, GameState, LevelSettings};

pub struct PhysicsPlugin;

pub const BLOCK_DENSITY: f32 = 1.0;
pub const BLOCK_FRICTION: f32 = 0.9;
pub const BLOCK_RESTITUTION: f32 = 0.05;
pub const STABILITY_CHECK_INTERVAL: f32 = 1.0;
pub const INSTABILITY_THRESHOLD: f32 = 3.5;

#[derive(Component)]
pub struct GameEntity;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(StabilityCheckTimer(Timer::from_seconds(
                STABILITY_CHECK_INTERVAL,
                TimerMode::Repeating,
            )))
            .insert_resource(Gravity(Vec2::new(0.0, -9.81))) // Increased to real-world gravity
            .add_systems(
                Update,
                check_tower_stability.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, update_physics_settings)
            .add_systems(OnExit(GameState::Playing), despawn_game_entities);
    }
}

pub fn despawn_game_entities(
    mut commands: Commands,
    query: Query<Entity, With<GameEntity>>,
    interaction_text_query: Query<Entity, With<crate::game::InteractionStateText>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &interaction_text_query {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Resource)]
struct StabilityCheckTimer(Timer);

#[derive(Component)]
pub struct TowerBlock {
    pub removable: bool,
    pub being_grabbed: bool,
    pub initial_position: Vec2,
}

#[derive(Bundle)]
struct PhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    friction: Friction,
    restitution: Restitution,
    game_entity: GameEntity,
    transform: Transform,
    global_transform: GlobalTransform,
}

fn update_physics_settings(
    level_settings: Res<LevelSettings>,
    mut gravity: ResMut<Gravity>,
    in_game: Res<State<GameState>>,
) {
    if *in_game.get() != GameState::Playing {
        return;
    }
    gravity.0 = Vec2::new(0.0, -level_settings.gravity); // Note the negative sign
}

fn check_tower_stability(
    time: Res<Time>,
    mut stability_timer: ResMut<StabilityCheckTimer>,
    blocks: Query<(&Transform, &TowerBlock)>,
    mut game_progress: ResMut<GameProgress>,
) {
    if game_progress.initial_wait_timer.is_some() {
        return;
    }

    if !stability_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    if game_progress.tower_collapsed || game_progress.level_complete {
        return;
    }

    let mut unstable_count = 0;

    for (transform, block) in &blocks {
        if block.being_grabbed {
            continue;
        }

        let current_pos = transform.translation.truncate();
        let displacement = (current_pos - block.initial_position).length();

        if displacement > INSTABILITY_THRESHOLD {
            unstable_count += 1;
        }
    }

    if unstable_count > 7 {
        game_progress.record_tower_collapse();
    }
}

pub fn spawn_floor(mut commands: Commands) {
    commands.spawn(PhysicsBundle {
        collider: Collider::rectangle(800.0, 20.0),
        rigid_body: RigidBody::Static,
        friction: Friction::new(BLOCK_FRICTION),
        restitution: Restitution::new(0.0),
        game_entity: GameEntity,
        transform: Transform::from_xyz(0.0, -300.0, 0.0),
        global_transform: GlobalTransform::default(),
    });
}

pub fn spawn_walls(mut commands: Commands) {
    commands.spawn(PhysicsBundle {
        collider: Collider::rectangle(10.0, 600.0),
        rigid_body: RigidBody::Static,
        friction: Friction::new(0.7),
        restitution: Restitution::new(0.1),
        game_entity: GameEntity,
        transform: Transform::from_xyz(-270.0, 0.0, 0.0),
        global_transform: GlobalTransform::default(),
    });

    commands.spawn(PhysicsBundle {
        collider: Collider::rectangle(10.0, 600.0),
        rigid_body: RigidBody::Static,
        friction: Friction::new(0.7),
        restitution: Restitution::new(0.1),
        game_entity: GameEntity,
        transform: Transform::from_xyz(270.0, 0.0, 0.0),
        global_transform: GlobalTransform::default(),
    });
}
