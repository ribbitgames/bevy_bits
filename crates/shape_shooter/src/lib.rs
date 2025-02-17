use bevy::math::primitives;
use bevy::prelude::*;
use bits_helpers::input::{just_pressed_world_position, pressed_world_position};
use bits_helpers::welcome_screen::{despawn_welcome_screen, WelcomeScreenElement};
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};
use ribbit::ShapeShooter;

mod ribbit;

const PLAYER_SIZE: f32 = 40.0;
const BULLET_SIZE: f32 = 5.0;
const BULLET_SPEED: f32 = 300.0;
const ENEMY_MIN_SIZE: f32 = 20.0;
const ENEMY_MAX_SIZE: f32 = 60.0;
const ENEMY_SPEED: f32 = 100.0;
const FIRE_RATE: f32 = 0.1; // Doubled fire rate (was 0.2)
const STAR_SPEED: f32 = 50.0;
const DIFFICULTY_INCREASE_RATE: f32 = 0.1;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

#[derive(Component)]
struct Player {
    fire_timer: Timer,
}

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Enemy {
    speed: f32,
    health: i32,
    size: Vec2,
    opacity: f32,
}

#[derive(Component)]
struct Star;

#[derive(Component)]
struct ScoreText;

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Resource, Default)]
struct DragState {
    is_dragging: bool,
}

#[derive(Resource, Default)]
struct GameDuration(f32);

pub fn run() {
    bits_helpers::get_default_app::<ShapeShooter>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .init_state::<GameState>()
    .init_resource::<Score>()
    .init_resource::<DragState>()
    .init_resource::<GameDuration>()
    .add_systems(Startup, setup)
    .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
    .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
    .add_systems(OnEnter(GameState::Playing), (spawn_player, spawn_stars))
    .add_systems(
        Update,
        handle_welcome_input.run_if(in_state(GameState::Welcome)),
    )
    .add_systems(
        Update,
        (
            handle_drag_input,
            player_movement,
            spawn_bullets,
            move_bullets,
            spawn_enemies,
            move_enemies,
            check_collisions,
            update_score,
            move_stars,
            increase_difficulty,
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
                Text::new("Space Shooter"),
                TextFont {
                    font: font.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Destroy enemy shapes!"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Tap and drag to move and shoot"),
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
    commands.spawn((
        Mesh2d::from(meshes.add(Mesh::from(primitives::RegularPolygon::new(
            PLAYER_SIZE / 2.0,
            3,
        )))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
        Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 4.0, 0.0)),
        Player {
            fire_timer: Timer::from_seconds(FIRE_RATE, TimerMode::Repeating),
        },
    ));

    commands.spawn((
        Text::new("Score: 0"),
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
        ScoreText,
    ));
}

fn spawn_stars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in 0..100 {
        let x = fastrand::f32().mul_add(WINDOW_WIDTH, -(WINDOW_WIDTH / 2.0));
        let y = fastrand::f32().mul_add(WINDOW_HEIGHT, -(WINDOW_HEIGHT / 2.0));
        let size = fastrand::f32().mul_add(2.0, 1.0); // Range 1.0..3.0

        commands.spawn((
            Mesh2d::from(meshes.add(Mesh::from(primitives::Circle::new(size / 2.0)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgba(1.0, 1.0, 1.0, 0.5)))),
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Star,
        ));
    }
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
    let Ok(player_transform) = player_query.get_single() else {
        return; // Exit the function if there's no player
    };

    if let Some(world_position) =
        just_pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
    {
        if is_point_in_triangle(
            world_position,
            player_transform.translation.truncate(),
            PLAYER_SIZE,
        ) {
            drag_state.is_dragging = true;
        }
    } else if mouse_input.just_released(MouseButton::Left)
        || touch_input.any_just_released()
        || touch_input.any_just_canceled()
    {
        drag_state.is_dragging = false;
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
    }

    let Ok(mut player_transform) = player_query.get_single_mut() else {
        return;
    };

    let Some(world_position) =
        pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
    else {
        return;
    };

    player_transform.translation.x = world_position.x;
    player_transform.translation.y = world_position.y;
}

fn spawn_bullets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_query: Query<(&Transform, &mut Player)>,
    time: Res<Time>,
    drag_state: Res<DragState>,
) {
    if !drag_state.is_dragging {
        return;
    }

    let (player_transform, mut player) = player_query.single_mut();
    player.fire_timer.tick(time.delta());

    if player.fire_timer.just_finished() {
        commands.spawn((
            Mesh2d::from(meshes.add(Mesh::from(primitives::Circle::new(BULLET_SIZE / 2.0)))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 0.0)))),
            Transform::from_translation(player_transform.translation),
            Bullet,
        ));
    }
}

fn move_bullets(mut bullet_query: Query<&mut Transform, With<Bullet>>, time: Res<Time>) {
    for mut transform in &mut bullet_query {
        transform.translation.y += BULLET_SPEED * time.delta_secs();
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if fastrand::f32() < 0.02 {
        let size = fastrand::f32().mul_add(ENEMY_MAX_SIZE - ENEMY_MIN_SIZE, ENEMY_MIN_SIZE);
        let x = fastrand::f32().mul_add(WINDOW_WIDTH - size, -((WINDOW_WIDTH - size) / 2.0));
        let shape_type = fastrand::u8(0..3);
        let color = Color::srgb(fastrand::f32(), fastrand::f32(), fastrand::f32());

        let (mesh, enemy_size) = match shape_type {
            0 => (
                Mesh::from(primitives::Rectangle::new(size, size)),
                Vec2::new(size, size),
            ),
            1 => (
                Mesh::from(primitives::Circle::new(size / 2.0)),
                Vec2::new(size, size),
            ),
            _ => (
                Mesh::from(primitives::RegularPolygon::new(size / 2.0, 3)),
                Vec2::new(size, size * 0.866),
            ),
        };

        let health =
            ((size - ENEMY_MIN_SIZE) / (ENEMY_MAX_SIZE - ENEMY_MIN_SIZE) * 3.0).round() as i32 + 1;

        commands.spawn((
            Mesh2d::from(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from(color.with_alpha(1.0)))),
            Transform::from_translation(Vec3::new(x, WINDOW_HEIGHT / 2.0 + size / 2.0, 0.0)),
            Enemy {
                speed: ENEMY_SPEED,
                health,
                size: enemy_size,
                opacity: 1.0,
            },
        ));
    }
}

fn move_enemies(mut enemy_query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    for (mut transform, enemy) in &mut enemy_query {
        transform.translation.y -= enemy.speed * time.delta_secs();
    }
}

fn check_aabb_collision(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    pos1.x - size1.x / 2.0 < pos2.x + size2.x / 2.0
        && pos1.x + size1.x / 2.0 > pos2.x - size2.x / 2.0
        && pos1.y - size1.y / 2.0 < pos2.y + size2.y / 2.0
        && pos1.y + size1.y / 2.0 > pos2.y - size2.y / 2.0
}

fn check_collisions(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    mut enemy_query: Query<(
        Entity,
        &Transform,
        &mut Enemy,
        &mut MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player_transform = player_query.single();
    let player_pos = player_transform.translation.truncate();

    for (enemy_entity, enemy_transform, mut enemy, material_handle) in &mut enemy_query {
        let enemy_pos = enemy_transform.translation.truncate();
        if check_aabb_collision(player_pos, Vec2::splat(PLAYER_SIZE), enemy_pos, enemy.size) {
            next_state.set(GameState::GameOver);
            return;
        }

        for (bullet_entity, bullet_transform) in &bullet_query {
            let bullet_pos = bullet_transform.translation.truncate();
            if check_aabb_collision(bullet_pos, Vec2::splat(BULLET_SIZE), enemy_pos, enemy.size) {
                commands.entity(bullet_entity).despawn();
                enemy.health -= 1;
                enemy.opacity = enemy.health as f32 / 4.0; // Assuming max health is 4

                // Update enemy opacity
                if let Some(material) = materials.get_mut(&*material_handle) {
                    material.color = material.color.with_alpha(enemy.opacity);
                }

                if enemy.health <= 0 {
                    commands.entity(enemy_entity).despawn();
                    score.0 += 1;
                }
                break;
            }
        }

        if enemy_transform.translation.y < -WINDOW_HEIGHT / 2.0 - enemy.size.y / 2.0 {
            commands.entity(enemy_entity).despawn();
        }
    }

    for (bullet_entity, bullet_transform) in &bullet_query {
        if bullet_transform.translation.y > WINDOW_HEIGHT / 2.0 + BULLET_SIZE / 2.0 {
            commands.entity(bullet_entity).despawn();
        }
    }
}

fn update_score(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.0 = format!("Score: {}", score.0);
    }
}

fn move_stars(mut star_query: Query<&mut Transform, With<Star>>, time: Res<Time>) {
    for mut transform in &mut star_query {
        transform.translation.y -= STAR_SPEED * time.delta_secs();
        if transform.translation.y < -WINDOW_HEIGHT / 2.0 {
            transform.translation.y = WINDOW_HEIGHT / 2.0;
        }
    }
}

fn increase_difficulty(
    mut enemy_query: Query<&mut Enemy>,
    time: Res<Time>,
    mut game_duration: ResMut<GameDuration>,
) {
    game_duration.0 += time.delta_secs();
    let difficulty_multiplier = game_duration.0.mul_add(DIFFICULTY_INCREASE_RATE, 1.0);

    for mut enemy in &mut enemy_query {
        enemy.speed = ENEMY_SPEED * difficulty_multiplier;
    }
}

fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
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
            BackgroundColor(Color::BLACK),
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
                Text::new(format!("Final Score: {}", score.0)),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn is_point_in_triangle(point: Vec2, triangle_center: Vec2, triangle_size: f32) -> bool {
    let half_size = triangle_size / 2.0;
    point.x >= triangle_center.x - half_size
        && point.x <= triangle_center.x + half_size
        && point.y >= triangle_center.y - half_size
        && point.y <= triangle_center.y + half_size
}
