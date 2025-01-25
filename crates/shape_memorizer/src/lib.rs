use core::time::Duration;

use bevy::color::palettes::css::{BLUE, GREEN, RED, YELLOW};
use bevy::prelude::*;
use bits_helpers::floating_score::{animate_floating_scores, spawn_floating_score};
use bits_helpers::welcome_screen::{despawn_welcome_screen, spawn_welcome_screen_shape};
use bits_helpers::FONT;
use rand::seq::SliceRandom;
use rand::Rng;
use ribbit::ShapeMemorizer;

mod ribbit;

const MEMORIZATION_TIME: f32 = 3.0;
const MAX_MISTAKES: usize = 1;
const CELL_SIZE: f32 = 60.0;
const GRID_COLS: usize = 4;
const GRID_ROWS: usize = 8;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Memorization,
    Playing,
    GameOver,
}

#[derive(Component, Clone)]
struct GridCell {
    row: usize,
    col: usize,
    shape: Shape,
    color: Color,
    is_target: bool,
    is_revealed: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Circle,
    Square,
    Triangle,
    Hexagon,
}

#[derive(Resource)]
struct GameData {
    target_shape: Shape,
    target_color: Color,
    mistakes: usize,
    correct_guesses: usize,
    start_time: f32,
    grid: Vec<GridCell>,
}

#[derive(Resource)]
struct MemorizationTimer(Timer);

#[derive(Component)]
struct CountdownText;

const SHAPES: [Shape; 4] = [
    Shape::Circle,
    Shape::Square,
    Shape::Triangle,
    Shape::Hexagon,
];

const COLORS: [Color; 4] = [
    Color::Srgba(RED),
    Color::Srgba(BLUE),
    Color::Srgba(GREEN),
    Color::Srgba(YELLOW),
];

pub fn run() {
    bits_helpers::get_default_app::<ShapeMemorizer>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .init_state::<GameState>()
    .init_resource::<GameData>()
    .init_resource::<MemorizationTimer>()
    .add_systems(Startup, setup)
    .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
    .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
    .add_systems(OnEnter(GameState::Memorization), setup_game)
    .add_systems(
        Update,
        (
            handle_welcome_input.run_if(in_state(GameState::Welcome)),
            update_memorization_timer.run_if(in_state(GameState::Memorization)),
            handle_game_input.run_if(in_state(GameState::Playing)),
            animate_floating_scores,
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
    mut game_data: ResMut<GameData>,
) {
    let mut rng = rand::thread_rng();

    game_data.target_shape = *SHAPES
        .choose(&mut rng)
        .expect("There's no shape to choose from");
    game_data.target_color = *COLORS
        .choose(&mut rng)
        .expect("There's no color to choose from");

    let mesh = match game_data.target_shape {
        Shape::Circle => Mesh::from(bevy::math::primitives::Circle::new(30.0)),
        Shape::Square => Mesh::from(bevy::math::primitives::Rectangle::new(60.0, 60.0)),
        Shape::Triangle => Mesh::from(bevy::math::primitives::RegularPolygon::new(30.0, 3)),
        Shape::Hexagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(30.0, 6)),
    };

    spawn_welcome_screen_shape(
        commands,
        asset_server,
        meshes,
        materials,
        "Find this shape",
        mesh,
        game_data.target_color,
    );
}

fn handle_welcome_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Memorization);
    }
}

fn setup_game(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    game_data.mistakes = 0;
    game_data.correct_guesses = 0;
    game_data.start_time = 0.0;

    let mut rng = rand::thread_rng();

    let mut grid = vec![];
    let mut target_positions = vec![];

    // Create grid cells
    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLS {
            let shape = *SHAPES
                .choose(&mut rng)
                .expect("There's no shape to choose from");
            let color = *COLORS
                .choose(&mut rng)
                .expect("There's no color to choose from");
            let is_target = shape == game_data.target_shape && color == game_data.target_color;

            if is_target {
                target_positions.push((row, col));
            }

            grid.push(GridCell {
                row,
                col,
                shape,
                color,
                is_target,
                is_revealed: false,
            });
        }
    }

    // Ensure we have exactly 3 target shapes
    while target_positions.len() < 3 {
        let row = rng.gen_range(0..GRID_ROWS);
        let col = rng.gen_range(0..GRID_COLS);
        if target_positions.contains(&(row, col)) {
            continue;
        }
        let Some(cell) = grid.get_mut(row * GRID_COLS + col) else {
            error!("Invalid grid position");
            continue;
        };
        cell.shape = game_data.target_shape;
        cell.color = game_data.target_color;
        cell.is_target = true;

        target_positions.push((row, col));
    }

    game_data.grid.clone_from(&grid);

    let grid_width = GRID_COLS as f32 * CELL_SIZE;
    let grid_height = GRID_ROWS as f32 * CELL_SIZE;
    let start_x = -grid_width / 2.0;
    let start_y = grid_height / 2.0;

    // Spawn grid cells
    for cell in grid {
        let x = (cell.col as f32).mul_add(CELL_SIZE, start_x) + CELL_SIZE / 2.0;
        let y = (cell.row as f32).mul_add(-CELL_SIZE, start_y) - CELL_SIZE / 2.0;

        let mesh = match cell.shape {
            Shape::Circle => Mesh::from(bevy::math::primitives::Circle::new(CELL_SIZE * 0.4)),
            Shape::Square => Mesh::from(bevy::math::primitives::Rectangle::new(
                CELL_SIZE * 0.8,
                CELL_SIZE * 0.8,
            )),
            Shape::Triangle => Mesh::from(bevy::math::primitives::RegularPolygon::new(
                CELL_SIZE * 0.4,
                3,
            )),
            Shape::Hexagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(
                CELL_SIZE * 0.4,
                6,
            )),
        };

        commands.spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from(cell.color))),
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Visibility::Visible,
            cell,
        ));
    }

    // Spawn countdown text
    commands.spawn((
        Text::new(format!("{MEMORIZATION_TIME:.1}")),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 60.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        CountdownText,
    ));
}

fn update_memorization_timer(
    time: Res<Time>,
    mut timer: ResMut<MemorizationTimer>,
    mut countdown_query: Query<&mut Text, With<CountdownText>>,
    mut commands: Commands,
    cell_query: Query<(Entity, &GridCell)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    asset_server: Res<AssetServer>,
) {
    timer.0.tick(time.delta());

    if let Ok(mut text) = countdown_query.get_single_mut() {
        text.0 = format!("{:.1}", timer.0.remaining_secs().max(0.0));
    }

    if timer.0.just_finished() {
        // Hide all shapes
        for (entity, _) in cell_query.iter() {
            commands.entity(entity).despawn();
        }

        let grid_width = GRID_COLS as f32 * CELL_SIZE;
        let grid_height = GRID_ROWS as f32 * CELL_SIZE;
        let start_x = -grid_width / 2.0;
        let start_y = grid_height / 2.0;

        // Spawn question marks
        for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                let current_pos = row * GRID_COLS + col;
                let Some(cell) = game_data.grid.get(current_pos) else {
                    error!("Could not load Cell {current_pos}");
                    continue;
                };

                let x = (col as f32).mul_add(CELL_SIZE, start_x) + CELL_SIZE / 2.0;
                let y = (row as f32).mul_add(-CELL_SIZE, start_y) - CELL_SIZE / 2.0;

                commands.spawn((
                    Text2d::new("?"),
                    TextFont {
                        font: asset_server.load(FONT),
                        font_size: CELL_SIZE * 0.8,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_translation(Vec3::new(x, y, 0.0)),
                    GridCell {
                        row,
                        col,
                        shape: cell.shape,
                        color: cell.color,
                        is_target: cell.is_target,
                        is_revealed: false,
                    },
                ));
            }
        }

        game_data.start_time = time.elapsed_secs();
        next_state.set(GameState::Playing);
    }
}

fn handle_game_input(
    mut commands: Commands,
    windows: Query<&Window>,
    mut cell_query: Query<(Entity, &mut GridCell)>,
    mut game_data: ResMut<GameData>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        let window = windows.single();
        let Some(position) = window.cursor_position() else {
            return;
        };

        let grid_width = GRID_COLS as f32 * CELL_SIZE;
        let grid_height = GRID_ROWS as f32 * CELL_SIZE;
        let start_x = -grid_width / 2.0;
        let start_y = grid_height / 2.0;

        let x = position.x - window.width() / 2.0;
        let y = window.height() / 2.0 - position.y;

        if x < start_x || x >= start_x + grid_width || y < start_y - grid_height || y >= start_y {
            return;
        }

        let col = ((x - start_x) / CELL_SIZE) as usize;
        let row = ((start_y - y) / CELL_SIZE) as usize;

        for (entity, mut cell) in &mut cell_query {
            if cell.row != row || cell.col != col || cell.is_revealed {
                continue;
            }
            cell.is_revealed = true;

            commands.entity(entity).despawn();

            let x = (col as f32).mul_add(CELL_SIZE, start_x) + CELL_SIZE / 2.0;
            let y = (row as f32).mul_add(-CELL_SIZE, start_y) - CELL_SIZE / 2.0;

            let mesh = match cell.shape {
                Shape::Circle => Mesh::from(bevy::math::primitives::Circle::new(CELL_SIZE * 0.4)),
                Shape::Square => Mesh::from(bevy::math::primitives::Rectangle::new(
                    CELL_SIZE * 0.8,
                    CELL_SIZE * 0.8,
                )),
                Shape::Triangle => Mesh::from(bevy::math::primitives::RegularPolygon::new(
                    CELL_SIZE * 0.4,
                    3,
                )),
                Shape::Hexagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(
                    CELL_SIZE * 0.4,
                    6,
                )),
            };

            commands.spawn((
                Mesh2d(meshes.add(mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from(cell.color))),
                Transform::from_translation(Vec3::new(x, y, 0.0)),
                Visibility::Visible,
            ));

            let world_position = Vec2::new(x, y);

            if cell.is_target {
                game_data.correct_guesses += 1;
                spawn_floating_score(&mut commands, world_position, "+1", GREEN, &asset_server);
                if game_data.correct_guesses >= 3 {
                    next_state.set(GameState::GameOver);
                }
            } else {
                game_data.mistakes += 1;
                spawn_floating_score(&mut commands, world_position, "-1", RED, &asset_server);
                if game_data.mistakes > MAX_MISTAKES {
                    next_state.set(GameState::GameOver);
                }
            }

            break;
        }
    }
}

fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_data: Res<GameData>,
    time: Res<Time>,
) {
    let game_duration = time.elapsed_secs() - game_data.start_time;
    let message = if game_data.correct_guesses >= 3 {
        format!("You won!\nTime: {game_duration:.1} seconds")
    } else {
        "Game Over!\nToo many mistakes".to_string()
    };

    commands.spawn((
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(5.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            ..default()
        },
        Text::new(message),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    ));
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            target_shape: Shape::Circle,
            target_color: Color::WHITE,
            mistakes: 0,
            correct_guesses: 0,
            start_time: 0.0,
            grid: Vec::new(),
        }
    }
}

impl Default for MemorizationTimer {
    fn default() -> Self {
        Self(Timer::new(
            Duration::from_secs_f32(MEMORIZATION_TIME),
            TimerMode::Once,
        ))
    }
}
