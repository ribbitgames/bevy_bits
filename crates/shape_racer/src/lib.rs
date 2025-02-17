use std::time::Duration;

use bevy::math::primitives;
use bevy::prelude::*;
use bits_helpers::input::{just_pressed_world_position, pressed_world_position};
use bits_helpers::welcome_screen::{despawn_welcome_screen, WelcomeScreenElement};
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};
use ribbit::ShapeRacer;

mod ribbit;

const PLAYER_WIDTH: f32 = 40.0;
const PLAYER_HEIGHT: f32 = 60.0;
const OBSTACLE_MIN_SIZE: f32 = 20.0;
const OBSTACLE_MAX_SIZE: f32 = 60.0;
const INITIAL_OBSTACLE_SPAWN_RATE: f32 = 1.0;
const INITIAL_OBSTACLE_SPEED: f32 = 150.0;
const DIFFICULTY_INCREASE_RATE: f32 = 0.1;
const MIN_SPAWN_INTERVAL: f32 = 0.3;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

#[derive(Component)]
struct Player {
    radius: f32,
}

#[derive(Component)]
struct Obstacle {
    speed: f32,
    radius: f32,
}

#[derive(Component)]
struct TimerText;

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource, Default)]
struct GameTimer(f32);

impl From<&GameTimer> for Duration {
    fn from(value: &GameTimer) -> Self {
        let secs = value.0.trunc() as u64;
        let nanos = (value.0.fract() * 1e9) as u32;
        Self::new(secs, nanos)
    }
}

#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
    drag_start: Option<Vec2>,
    initial_player_pos: Option<Vec2>,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            INITIAL_OBSTACLE_SPAWN_RATE,
            TimerMode::Repeating,
        ))
    }
}

pub fn run() {
    bits_helpers::get_default_app::<ShapeRacer>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .init_state::<GameState>()
        .init_resource::<SpawnTimer>()
        .init_resource::<GameTimer>()
        .init_resource::<DragState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
        .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
        .add_systems(OnEnter(GameState::Playing), spawn_player)
        .add_systems(
            Update,
            (handle_welcome_input.run_if(in_state(GameState::Welcome)),),
        )
        .add_systems(
            Update,
            (
                handle_drag_input,
                player_movement,
                spawn_obstacles,
                obstacle_movement,
                check_collisions,
                update_timer,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_welcome_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load(FONT);
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            WelcomeScreenElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Shape Racer"),
                TextFont {
                    font: font.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Avoid obstacles!"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Tap to start"),
                TextFont {
                    font,
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let player_radius = PLAYER_WIDTH.min(PLAYER_HEIGHT) / 2.0;
    commands.spawn((
        Mesh2d(meshes.add(Mesh::from(primitives::Rectangle::new(
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
        )))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Transform::from_translation(Vec3::new(
            0.0,
            -WINDOW_HEIGHT / 2.0 + PLAYER_HEIGHT + 10.0,
            0.0,
        ))
        .with_rotation(Quat::from_rotation_z(core::f32::consts::PI / 2.0)),
        Player {
            radius: player_radius,
        },
    ));

    commands.spawn((
        Text::new("Time: 0.0"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TimerText,
    ));
}

fn handle_welcome_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

fn handle_drag_input(
    mut drag_state: ResMut<DragState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    windows: Query<&Window>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let player_transform = player_query.single();

    if let Some(world_position) =
        just_pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
    {
        if is_point_in_rect(
            world_position,
            player_transform.translation.truncate(),
            Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
        ) {
            drag_state.is_dragging = true;
            drag_state.drag_start = Some(world_position);
            drag_state.initial_player_pos = Some(player_transform.translation.truncate());
        }
    } else if mouse_input.just_released(MouseButton::Left)
        || touch_input.any_just_released()
        || touch_input.any_just_canceled()
    {
        drag_state.is_dragging = false;
        drag_state.drag_start = None;
        drag_state.initial_player_pos = None;
    }
}

fn player_movement(
    drag_state: ResMut<DragState>,
    mut player_query: Query<&mut Transform, With<Player>>,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if !drag_state.is_dragging {
        return;
    };

    let Some(world_position) =
        pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
    else {
        return;
    };

    let mut player_transform = player_query.single_mut();

    player_transform.translation.x = world_position.x;
}

fn spawn_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawn_timer: ResMut<SpawnTimer>,
    game_timer: Res<GameTimer>,
    time: Res<Time>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        let size =
            fastrand::f32().mul_add(OBSTACLE_MAX_SIZE - OBSTACLE_MIN_SIZE, OBSTACLE_MIN_SIZE);
        let x = fastrand::f32().mul_add(WINDOW_WIDTH - size, -((WINDOW_WIDTH - size) / 2.0));
        let shape_type = fastrand::u8(0..3);
        let color = Color::srgb(fastrand::f32(), fastrand::f32(), fastrand::f32());

        let (mesh, radius) = match shape_type {
            0 => (Mesh2d(meshes.add(Rectangle::new(size, size))), size / 2.0),
            1 => (Mesh2d(meshes.add(Circle::new(size / 2.0))), size / 2.0),
            _ => (
                Mesh2d(meshes.add(RegularPolygon::new(size / 2.0, 3))),
                size / 2.0 * 0.866,
            ), // Approximation for triangle
        };

        let speed = game_timer
            .0
            .mul_add(DIFFICULTY_INCREASE_RATE, INITIAL_OBSTACLE_SPEED);

        commands.spawn((
            mesh,
            MeshMaterial2d::from(materials.add(ColorMaterial::from(color))),
            Transform::from_translation(Vec3::new(x, WINDOW_HEIGHT / 2.0 + size / 2.0, 0.0)),
            Obstacle { speed, radius },
        ));

        let new_spawn_rate = game_timer
            .0
            .mul_add(-0.02, INITIAL_OBSTACLE_SPAWN_RATE)
            .max(MIN_SPAWN_INTERVAL);
        spawn_timer.0 = Timer::from_seconds(new_spawn_rate, TimerMode::Repeating);
    }
}

fn obstacle_movement(mut query: Query<(&mut Transform, &Obstacle)>, time: Res<Time>) {
    for (mut transform, obstacle) in &mut query {
        transform.translation.y -= obstacle.speed * time.delta_secs();
    }
}

fn check_collisions(
    mut commands: Commands,
    player_query: Query<(&Transform, &Player)>,
    obstacle_query: Query<(Entity, &Transform, &Obstacle)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (player_transform, player) = player_query.single();
    let player_pos = player_transform.translation.truncate();

    for (obstacle_entity, obstacle_transform, obstacle) in obstacle_query.iter() {
        let obstacle_pos = obstacle_transform.translation.truncate();

        if check_circle_collision(player_pos, player.radius, obstacle_pos, obstacle.radius) {
            next_state.set(GameState::GameOver);
            return;
        }

        if obstacle_transform.translation.y < -WINDOW_HEIGHT / 2.0 - obstacle.radius {
            commands.entity(obstacle_entity).despawn();
        }
    }
}

fn check_circle_collision(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> bool {
    pos1.distance_squared(pos2) < (radius1 + radius2).powi(2)
}

fn update_timer(
    mut game_timer: ResMut<GameTimer>,
    mut query: Query<&mut Text, With<TimerText>>,
    time: Res<Time>,
) {
    game_timer.0 += time.delta_secs();

    if let Ok(mut text) = query.get_single_mut() {
        text.0 = format!("Time: {:.1}", game_timer.0);
    }
}

fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_timer: Res<GameTimer>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Over!"),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new(format!("Time: {:.1}s", game_timer.0)),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn is_point_in_rect(point: Vec2, rect_center: Vec2, rect_size: Vec2) -> bool {
    let half_size = rect_size / 2.0;
    point.x >= rect_center.x - half_size.x
        && point.x <= rect_center.x + half_size.x
        && point.y >= rect_center.y - half_size.y
        && point.y <= rect_center.y + half_size.y
}
