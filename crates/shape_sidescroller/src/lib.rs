use bevy::color::palettes::css::{GREEN, RED, WHITE, YELLOW};
use bevy::prelude::*;
use bits_helpers::floating_score::{animate_floating_scores, spawn_floating_score};
use bits_helpers::welcome_screen::{despawn_welcome_screen, spawn_welcome_screen_shape};
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};
use rand::Rng;
use ribbit::ShapeSideScroller;

mod ribbit;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 50.0);
const GROUND_HEIGHT: f32 = 50.0;
const INITIAL_SCROLL_SPEED: f32 = 200.0;
const INITIAL_SCORE: i32 = 10;
const MIN_OBSTACLE_DISTANCE: f32 = 150.0;
const CASTLE_DISTANCE: f32 = 10000.0;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

#[derive(Component)]
struct Player {
    is_jumping: bool,
    jump_velocity: f32,
    is_falling: bool,
    can_double_jump: bool,
    falling_into_hole: bool,
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Hole;

#[derive(Component)]
struct Cloud;

#[derive(Component)]
struct Castle;

#[derive(Component)]
struct CollisionBox {
    size: Vec2,
}

#[derive(Resource)]
struct GameData {
    score: i32,
    scroll_speed: f32,
    distance: f32,
}

pub fn run() {
    bits_helpers::get_default_app::<ShapeSideScroller>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .init_state::<GameState>()
    .init_resource::<GameData>()
    .add_systems(Startup, setup)
    .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
    .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
    .add_systems(OnEnter(GameState::Playing), spawn_game_elements)
    .add_systems(
        Update,
        (
            handle_welcome_input.run_if(in_state(GameState::Welcome)),
            (
                move_player,
                scroll_world,
                spawn_obstacles,
                check_collisions,
                handle_falling_into_hole,
                update_score,
                spawn_and_move_clouds,
                animate_floating_scores,
            )
                .run_if(in_state(GameState::Playing)),
        ),
    )
    .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_welcome_screen(
    commands: Commands,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_welcome_screen_shape(
        commands,
        asset_server,
        meshes,
        materials,
        "Tap to Jump!",
        Mesh::from(bevy::math::primitives::Rectangle::new(
            PLAYER_SIZE.x,
            PLAYER_SIZE.y,
        )),
        Color::Srgba(WHITE),
    );
}

fn handle_welcome_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

fn spawn_game_elements(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut game_data: ResMut<GameData>,
) {
    let ground_y = -WINDOW_HEIGHT / 2.0 + GROUND_HEIGHT / 2.0;

    commands.spawn((
        Mesh2d(
            meshes.add(Mesh::from(bevy::math::primitives::Rectangle::new(
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
            ))),
        ),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::Srgba(WHITE)))),
        Transform::from_xyz(-WINDOW_WIDTH / 4.0, ground_y + GROUND_HEIGHT, 1.0),
        Player {
            is_jumping: false,
            jump_velocity: 0.0,
            is_falling: false,
            can_double_jump: false,
            falling_into_hole: false,
        },
    ));

    commands.spawn((
        Sprite::from_color(Color::Srgba(GREEN), Vec2::new(WINDOW_WIDTH, GROUND_HEIGHT)),
        Transform::from_xyz(0.0, ground_y, -1.0),
        Ground,
    ));

    // Spawn castle
    commands.spawn((
        Sprite::from_color(Color::Srgba(YELLOW), Vec2::new(100.0, 200.0)),
        Transform::from_xyz(CASTLE_DISTANCE, ground_y + GROUND_HEIGHT + 100.0, 1.0),
        Castle,
    ));

    *game_data = GameData {
        score: INITIAL_SCORE,
        scroll_speed: INITIAL_SCROLL_SPEED,
        distance: 0.0,
    };

    commands.spawn((
        Text::new(format!("Score: {}", game_data.score)),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::Srgba(WHITE)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn move_player(
    mut player_set: ParamSet<(
        Query<(&mut Transform, &mut Player)>,
        Query<&Transform, (With<Ground>, Without<Player>)>,
        Query<&Transform, (With<Hole>, Without<Player>)>,
    )>,
    input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    time: Res<Time>,
) {
    let ground_y = player_set.p1().single().translation.y + GROUND_HEIGHT;

    let hole_positions: Vec<Vec2> = player_set
        .p2()
        .iter()
        .map(|transform| transform.translation.truncate())
        .collect();

    for (mut transform, mut player) in &mut player_set.p0() {
        if player.falling_into_hole {
            return; // Do nothing if falling into a hole
        }

        let player_pos = transform.translation.truncate();

        let is_over_hole = hole_positions
            .iter()
            .any(|&hole_pos| (player_pos.x - hole_pos.x).abs() < PLAYER_SIZE.x / 2.0);

        if input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
            if !player.is_jumping && !player.is_falling && !is_over_hole {
                player.is_jumping = true;
                player.jump_velocity = 600.0;
                player.can_double_jump = true;
            } else if player.is_jumping && player.can_double_jump {
                player.jump_velocity = player.jump_velocity.max(450.0);
                player.can_double_jump = false;
            }
        }

        if player.is_jumping || is_over_hole {
            transform.translation.y += player.jump_velocity * time.delta_secs();
            player.jump_velocity -= 1500.0 * time.delta_secs();

            if transform.translation.y <= ground_y && !is_over_hole {
                transform.translation.y = ground_y;
                player.is_jumping = false;
                player.jump_velocity = 0.0;
                player.can_double_jump = false;
            } else if is_over_hole && transform.translation.y <= -WINDOW_HEIGHT / 2.0 {
                player.is_falling = true;
            }
        }
    }
}

fn scroll_world(
    mut obstacle_query: Query<&mut Transform, Or<(With<Obstacle>, With<Hole>, With<Castle>)>>,
    player_query: Query<&Player>,
    mut game_data: ResMut<GameData>,
    time: Res<Time>,
) {
    let player = player_query.single();
    if !player.falling_into_hole && game_data.scroll_speed > 0.0 {
        for mut transform in &mut obstacle_query {
            transform.translation.x -= game_data.scroll_speed * time.delta_secs();
        }
        game_data.distance += game_data.scroll_speed * time.delta_secs();
        game_data.scroll_speed += 5.0 * time.delta_secs(); // Increase speed over time
    }
}

fn spawn_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_data: Res<GameData>,
    obstacle_query: Query<&Transform, Or<(With<Obstacle>, With<Hole>)>>,
    ground_query: Query<&Transform, With<Ground>>,
) {
    let mut rng = rand::rng();
    let ground_y = ground_query.single().translation.y + GROUND_HEIGHT / 2.0;

    if rng.random_bool((0.01 + game_data.distance / 20000.0).min(0.05) as f64) {
        let spawn_position = WINDOW_WIDTH / 2.0;

        let is_clear = obstacle_query.iter().all(|transform| {
            (transform.translation.x - spawn_position).abs() > MIN_OBSTACLE_DISTANCE
        });

        if is_clear {
            if rng.random_bool(0.7) {
                let size = Vec2::new(30.0, 40.0);
                commands.spawn((
                    Mesh2d(
                        meshes.add(Mesh::from(bevy::math::primitives::RegularPolygon::new(
                            size.y / 2.0,
                            3,
                        ))),
                    ),
                    MeshMaterial2d(materials.add(ColorMaterial::from(Color::Srgba(RED)))),
                    Transform::from_xyz(spawn_position, ground_y + size.y / 2.0, 1.0),
                    Obstacle,
                    CollisionBox { size },
                ));
            } else {
                let width = rng.random_range(50.0..100.0);
                commands.spawn((
                    Sprite::from_color(
                        Color::Srgba(Srgba::new(0.1, 0.1, 0.1, 1.0)),
                        Vec2::new(width, GROUND_HEIGHT),
                    ),
                    Transform::from_xyz(spawn_position, ground_y, 1.0),
                    Hole,
                ));
            }
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    castle_query: Query<&Transform, (With<Castle>, Without<Player>)>,
    obstacle_query: Query<(Entity, &Transform, &CollisionBox), (With<Obstacle>, Without<Player>)>,
    hole_query: Query<(Entity, &Transform, &Sprite), (With<Hole>, Without<Player>)>,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
) {
    let (mut player_transform, mut player) = player_query.single_mut();
    let player_pos = player_transform.translation.truncate();
    let player_size = PLAYER_SIZE;

    if player.falling_into_hole {
        return;
    }

    // Check for obstacle collisions
    for (entity, obstacle_transform, collision_box) in obstacle_query.iter() {
        let obstacle_pos = obstacle_transform.translation.truncate();
        let obstacle_size = collision_box.size;

        if check_aabb_collision(player_pos, player_size, obstacle_pos, obstacle_size) {
            game_data.score -= 1;
            commands.entity(entity).despawn();
            spawn_floating_score(&mut commands, player_pos, "-1", RED, &asset_server);

            // Push the player back to their initial x position
            player_transform.translation.x = -WINDOW_WIDTH / 4.0;

            if game_data.score <= 0 {
                next_state.set(GameState::GameOver);
            }
            return;
        }
    }

    // Check for hole collisions
    for (_, hole_transform, hole_sprite) in hole_query.iter() {
        let hole_pos = hole_transform.translation.truncate();
        let hole_size = hole_sprite
            .custom_size
            .unwrap_or(Vec2::new(50.0, GROUND_HEIGHT));

        if check_aabb_collision(player_pos, player_size, hole_pos, hole_size) {
            player.falling_into_hole = true;
            player.is_jumping = false;
            player.jump_velocity = 0.0;
            game_data.scroll_speed = 0.0;
            return;
        }
    }

    // Check for castle collision
    if let Ok(castle_transform) = castle_query.get_single() {
        let castle_pos = castle_transform.translation.truncate();
        let castle_size = Vec2::new(100.0, 200.0); // Use the actual castle size

        if check_aabb_collision(player_pos, player_size, castle_pos, castle_size) {
            next_state.set(GameState::GameOver);
        }
    }

    // Despawn obstacles and holes that are off-screen
    for (entity, obstacle_transform, _) in obstacle_query.iter() {
        if obstacle_transform.translation.x < -WINDOW_WIDTH / 2.0 - 50.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, hole_transform, _) in hole_query.iter() {
        if hole_transform.translation.x < -WINDOW_WIDTH / 2.0 - 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn check_aabb_collision(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    let min1 = pos1 - size1 / 2.0;
    let max1 = pos1 + size1 / 2.0;
    let min2 = pos2 - size2 / 2.0;
    let max2 = pos2 + size2 / 2.0;

    max1.x > min2.x && min1.x < max2.x && max1.y > min2.y && min1.y < max2.y
}

fn handle_falling_into_hole(
    mut player: Query<(&mut Transform, &mut Player)>,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    let (mut transform, player) = player.single_mut();
    if player.falling_into_hole {
        game_data.scroll_speed = 0.0; // Stop world scrolling
        transform.translation.y -= 150.0 * time.delta_secs(); // Slow falling

        if transform.translation.y < -WINDOW_HEIGHT / 2.0 - PLAYER_SIZE.y {
            next_state.set(GameState::GameOver);
        }
    }
}

fn spawn_and_move_clouds(
    mut commands: Commands,
    mut cloud_query: Query<(Entity, &mut Transform), With<Cloud>>,
    time: Res<Time>,
) {
    let mut rng = rand::rng();

    if rng.random_bool(0.02) {
        let cloud_size = Vec2::new(rng.random_range(50.0..100.0), rng.random_range(20.0..40.0));
        commands.spawn((
            Sprite::from_color(Color::Srgba(Srgba::new(0.5, 0.5, 1.0, 0.7)), cloud_size),
            Transform::from_xyz(
                WINDOW_WIDTH / 2.0 + 50.0,
                rng.random_range(0.0..WINDOW_HEIGHT / 2.0),
                -1.0,
            ),
            Cloud,
        ));
    }

    for (entity, mut transform) in &mut cloud_query {
        transform.translation.x -= 50.0 * time.delta_secs();

        if transform.translation.x < -WINDOW_WIDTH / 2.0 - 100.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_score(mut query: Query<&mut Text>, game_data: Res<GameData>) {
    for mut text in &mut query {
        text.0 = format!("Score: {}", game_data.score);
    }
}

fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_data: Res<GameData>,
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
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::Srgba(WHITE)),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", game_data.score)),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::Srgba(WHITE)),
            ));
        });
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: INITIAL_SCORE,
            scroll_speed: INITIAL_SCROLL_SPEED,
            distance: 0.0,
        }
    }
}
