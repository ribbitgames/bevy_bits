use bevy::color::palettes::css::{BLUE, GREEN, RED, WHITE, YELLOW};
use bevy::math::primitives;
use bevy::prelude::*;
use bits_helpers::input::{just_pressed_world_position, pressed_world_position};
use bits_helpers::welcome_screen::{despawn_welcome_screen, spawn_welcome_screen_shape};
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use ribbit::ShapeGrabber;

const PADDLE_SIZE: Vec2 = Vec2::new(80.0, 20.0);
const SHAPE_SIZE: f32 = 30.0;
const SHAPE_SPAWN_RATE: f32 = 0.5;
const GAME_DURATION: f32 = 30.0;

mod ribbit;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

pub fn run() {
    bits_helpers::get_default_app::<ShapeGrabber>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .init_state::<GameState>()
    .init_resource::<SpawnTimer>()
    .init_resource::<Score>()
    .init_resource::<GameTimer>()
    .init_resource::<TargetShape>()
    .add_systems(Startup, setup)
    .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
    .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
    .add_systems(OnEnter(GameState::Playing), spawn_paddle)
    .add_systems(
        Update,
        handle_welcome_input.run_if(in_state(GameState::Welcome)),
    )
    .add_systems(
        Update,
        (
            paddle_movement,
            spawn_shapes,
            shape_movement,
            catch_shapes,
            update_score_text,
            update_timer,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
    .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Shape;

#[derive(Component)]
struct CorrectShape;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimerText;

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource)]
struct GameTimer(Timer);

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Resource)]
struct TargetShape {
    shape_type: ShapeType,
    color: Color,
}

#[derive(Component)]
struct DragState {
    is_dragging: bool,
    drag_start: Vec2,
    initial_paddle_pos: Vec2,
}

#[derive(Clone, Copy, PartialEq)]
enum ShapeType {
    Circle,
    Square,
    Triangle,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SHAPE_SPAWN_RATE, TimerMode::Repeating))
    }
}

impl Default for GameTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(GAME_DURATION, TimerMode::Once))
    }
}

impl Default for TargetShape {
    fn default() -> Self {
        Self {
            shape_type: ShapeType::Circle,
            color: Color::Srgba(RED),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

const COLORS: [Color; 4] = [
    Color::Srgba(RED),
    Color::Srgba(BLUE),
    Color::Srgba(GREEN),
    Color::Srgba(YELLOW),
];

const SHAPES: [ShapeType; 3] = [ShapeType::Circle, ShapeType::Square, ShapeType::Triangle];

fn spawn_welcome_screen(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut target_shape: ResMut<TargetShape>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    target_shape.shape_type = *SHAPES
        .choose(&mut rng)
        .expect("shapes array should not be empty");

    target_shape.color = *COLORS
        .choose(&mut rng)
        .expect("colors array should not be empty");

    let target_mesh = match target_shape.shape_type {
        ShapeType::Circle => Mesh::from(primitives::Circle::new(SHAPE_SIZE / 2.0)),
        ShapeType::Square => Mesh::from(primitives::Rectangle::new(SHAPE_SIZE, SHAPE_SIZE)),
        ShapeType::Triangle => Mesh::from(primitives::RegularPolygon::new(SHAPE_SIZE / 2.0, 3)),
    };

    spawn_welcome_screen_shape(
        commands,
        asset_server,
        meshes,
        materials,
        "Grab ONLY this shape",
        target_mesh,
        target_shape.color,
    );
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

fn spawn_paddle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn the paddle
    commands.spawn((
        Mesh2d::from(meshes.add(Mesh::from(primitives::Rectangle::new(
            PADDLE_SIZE.x,
            PADDLE_SIZE.y,
        )))),
        MeshMaterial2d::from(materials.add(ColorMaterial::from(Color::Srgba(WHITE)))),
        Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 2.0 + 50.0, 0.0)),
        Paddle,
        DragState {
            is_dragging: false,
            drag_start: Vec2::ZERO,
            initial_paddle_pos: Vec2::new(0.0, -WINDOW_HEIGHT / 2.0 + 50.0),
        },
    ));

    // Spawn the score text
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::Srgba(WHITE)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));

    // Spawn the timer text
    commands.spawn((
        Text::new(format!("Time: {GAME_DURATION:.1}")),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::Srgba(WHITE)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        TimerText,
    ));
}

fn paddle_movement(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    windows: Query<&Window>,
    mut paddle_query: Query<(&mut Transform, &mut DragState), With<Paddle>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok((mut paddle_transform, mut drag_state)) = paddle_query.get_single_mut() else {
        return;
    };

    if let Some(world_position) =
        just_pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
    {
        // Start dragging
        drag_state.is_dragging = true;
        drag_state.drag_start = world_position;
        drag_state.initial_paddle_pos = paddle_transform.translation.truncate();
    } else if mouse_input.just_released(MouseButton::Left) || touch_input.any_just_released() {
        // Stop dragging
        drag_state.is_dragging = false;
    }

    if drag_state.is_dragging {
        let Some(pressed_world_position) =
            pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
        else {
            return;
        };
        // Calculate the drag delta
        let drag_delta = pressed_world_position - drag_state.drag_start;

        // Update paddle position based on drag
        let new_x = drag_state.initial_paddle_pos.x + drag_delta.x;

        // Clamp the paddle position to stay within the screen bounds
        let half_paddle_width = PADDLE_SIZE.x / 2.0;
        paddle_transform.translation.x = new_x.clamp(
            -WINDOW_WIDTH / 2.0 + half_paddle_width,
            WINDOW_WIDTH / 2.0 - half_paddle_width,
        );
    }
}

fn spawn_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawn_timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    target_shape: Res<TargetShape>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.just_finished() {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(
            -WINDOW_WIDTH / 2.0 + SHAPE_SIZE / 2.0..WINDOW_WIDTH / 2.0 - SHAPE_SIZE / 2.0,
        );
        let is_correct_shape = rng.gen_bool(0.25); // 25% chance of spawning the correct shape

        let (shape_type, color, is_correct) = if is_correct_shape {
            (target_shape.shape_type, target_shape.color, true)
        } else {
            let random_shape = *SHAPES
                .choose(&mut rng)
                .expect("shape array should not be empty");

            let random_color = *COLORS
                .choose(&mut rng)
                .expect("color array should not be empty");

            if random_shape == target_shape.shape_type && random_color == target_shape.color {
                // If we accidentally generated the correct shape, change the color
                let different_color = COLORS
                    .iter()
                    .filter(|&c| *c != target_shape.color)
                    .choose(&mut rng)
                    .copied()
                    .expect("there should be at least one different color");
                (random_shape, different_color, false)
            } else {
                (random_shape, random_color, false)
            }
        };

        let mesh = match shape_type {
            ShapeType::Circle => Mesh::from(primitives::Circle::new(SHAPE_SIZE / 2.0)),
            ShapeType::Square => Mesh::from(primitives::Rectangle::new(SHAPE_SIZE, SHAPE_SIZE)),
            ShapeType::Triangle => Mesh::from(primitives::RegularPolygon::new(SHAPE_SIZE / 2.0, 3)),
        };

        let mut entity_commands = commands.spawn((
            Mesh2d::from(meshes.add(mesh)),
            MeshMaterial2d::from(materials.add(ColorMaterial::from(color))),
            Transform::from_translation(Vec3::new(x, WINDOW_HEIGHT / 2.0 + SHAPE_SIZE, 0.0)),
            Shape,
        ));

        if is_correct {
            entity_commands.insert(CorrectShape);
        }
    }
}

fn shape_movement(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation.y -= 200.0 * time.delta_secs();
    }
}

fn catch_shapes(
    mut commands: Commands,
    paddle_query: Query<&Transform, With<Paddle>>,
    shape_query: Query<(Entity, &Transform, Option<&CorrectShape>), With<Shape>>,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let paddle_transform = paddle_query.single();

    for (shape_entity, shape_transform, correct_shape) in shape_query.iter() {
        if shape_transform.translation.y
            < paddle_transform.translation.y + PADDLE_SIZE.y / 2.0 + SHAPE_SIZE / 2.0
            && shape_transform.translation.y
                > paddle_transform.translation.y - PADDLE_SIZE.y / 2.0 - SHAPE_SIZE / 2.0
            && (shape_transform.translation.x - paddle_transform.translation.x).abs()
                < PADDLE_SIZE.x / 2.0 + SHAPE_SIZE / 2.0
        {
            commands.entity(shape_entity).despawn();

            if correct_shape.is_some() {
                score.0 += 1;
                if score.0 >= 10 {
                    next_state.set(GameState::GameOver);
                }
            } else {
                next_state.set(GameState::GameOver);
            }
        } else if shape_transform.translation.y < -WINDOW_HEIGHT / 2.0 - SHAPE_SIZE {
            commands.entity(shape_entity).despawn();
        }
    }
}
fn update_score_text(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.0 = format!("Score: {}", score.0);
    }
}

fn update_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut timer_text: Query<&mut Text, With<TimerText>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    game_timer.0.tick(time.delta());
    let remaining_time = GAME_DURATION - game_timer.0.elapsed_secs();

    if let Ok(mut text) = timer_text.get_single_mut() {
        text.0 = format!("Time: {:.1}", remaining_time.max(0.0));
    }

    if game_timer.0.just_finished() {
        next_state.set(GameState::GameOver);
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
                ..default()
            },
            BackgroundColor::from(Srgba::new(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Game Over!\nFinal Score: {}", score.0)),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::Srgba(WHITE)),
            ));
        });
}
