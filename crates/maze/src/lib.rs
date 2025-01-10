use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::utils::default;
use maze::MazeGenerator;
use rand::seq::SliceRandom;
use ribbit::Maze;

mod maze;
mod ribbit;

const MAZE_SIZE_X: usize = 14;
const MAZE_SIZE_Y: usize = 14;

const TILE_SIZE: f32 = 24.;
const WALL_DEPTH: f32 = 4.;

const NUM_ITEMS: usize = 3;
const ITEM_SIZE: f32 = TILE_SIZE * 0.5;

const PLAYER_SIZE: f32 = TILE_SIZE * 0.75;
const MOVE_SPEED: f32 = 4.0; // tile size / sec

#[derive(Clone, Copy, Default)]
struct GridPos {
    x: usize,
    y: usize,
    priority: f32,
}

impl From<GridPos> for Vec3 {
    fn from(pos: GridPos) -> Self {
        let tile_offset = Vec2::new(TILE_SIZE, TILE_SIZE);
        let center_offset = tile_offset * (14.0f32.mul_add(0.5, -0.5));
        Self::new(
            (pos.x as f32).mul_add(tile_offset.x, -center_offset.x),
            (pos.y as f32).mul_add(-tile_offset.y, center_offset.y),
            pos.priority,
        )
    }
}

impl From<GridPos> for Transform {
    fn from(pos: GridPos) -> Self {
        Self::from_translation(pos.into())
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Init,
    Game,
    Result,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component, Default)]
struct MazePlayer {
    is_moving: bool,
    pos: IVec2,
    next_pos: IVec2,
    alpha: f32,
    input: IVec2,
}

#[derive(Component, Default)]
struct MazeItem {
    pos: IVec2,
}

pub fn run() {
    bits_helpers::get_default_app::<Maze>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Game), reset_maze)
        .add_systems(
            Update,
            (
                init_maze.run_if(in_state(GameState::Init)),
                item_collect_system.run_if(in_state(GameState::Game)),
                gridbase_player_move_system.run_if(in_state(GameState::Game)),
                game_result.run_if(in_state(GameState::Result)),
                keyboard_events_maze.run_if(in_state(GameState::Game)),
            ),
        )
        .run();
}

fn init_maze(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    println!("init");
    // Camera
    commands.spawn(Camera2d).insert(MainCamera);
    // Maze Generator
    commands.spawn(MazeGenerator::new(MAZE_SIZE_X, MAZE_SIZE_Y));
    // Spawn maze grid (It needs only once)
    spawn_grid(&mut commands);
    spawn_objective(&mut commands);
    next_state.set(GameState::Game);
}

fn spawn_grid(commands: &mut Commands) {
    let room_size = Vec2::new(TILE_SIZE - WALL_DEPTH, TILE_SIZE - WALL_DEPTH);
    for y in 0..MAZE_SIZE_Y {
        for x in 0..MAZE_SIZE_X {
            let grid_pos = GridPos { x, y, priority: 0. };
            commands
                .spawn((
                    Sprite::from_color(Color::WHITE, Vec2::new(TILE_SIZE, TILE_SIZE)),
                    Transform::from(grid_pos),
                ))
                .with_children(|parent| {
                    parent.spawn(Sprite::from_color(Color::BLACK, room_size));
                });
        }
    }
}

fn spawn_objective(commands: &mut Commands) {
    commands.spawn((
        Text::new("Collect  !"),
        TextFont {
            font_size: 48.,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(1., 1., 0.), Vec2::new(32., 32.)),
        Transform::from_xyz(72., 240., 10.),
    ));
}

fn reset_maze(mut commands: Commands, mut maze_query: Query<&mut MazeGenerator>) {
    println!("reset");
    let mut maze = maze_query.single_mut();
    // Generate a new maze
    maze.generate();
    println!("{}", *maze);
    spawn_maze(&mut commands, &maze);
    // Get all the deadends of the maze, and shuffle them
    let mut deadends = maze.get_deadends();
    let mut rng = rand::thread_rng();
    deadends.shuffle(&mut rng);
    // Spawn the player at the 1st deadend, and spawn NUM_ITEMS items at the following deadends
    let loop_num = (NUM_ITEMS + 1).min(deadends.len());
    for (i, deadend) in deadends.iter().enumerate() {
        if i >= loop_num {
            break;
        }
        if i == 0 {
            spawn_player(&mut commands, *deadend);
        } else {
            spawn_item(&mut commands, *deadend);
        }
    }
}

fn spawn_maze(commands: &mut Commands, maze: &MazeGenerator) {
    let room_size = Vec2::new(TILE_SIZE - WALL_DEPTH, TILE_SIZE - WALL_DEPTH);

    for y in 0..maze.height() {
        for x in 0..maze.width() {
            let grid_pos = GridPos { x, y, priority: 1. };
            let pos: Vec3 = grid_pos.into();
            if maze.can_go(IVec2::new(x as i32, y as i32), IVec2::new(1, 0)) {
                commands.spawn((
                    Sprite::from_color(Color::BLACK, room_size),
                    Transform::from_translation(pos + Vec3::new(2., 0., 0.)),
                ));
            }
            if maze.can_go(IVec2::new(x as i32, y as i32), IVec2::new(0, 1)) {
                commands.spawn((
                    Sprite::from_color(Color::BLACK, room_size),
                    Transform::from_translation(pos + Vec3::new(0., -2., 0.)),
                ));
            }
            if maze.can_go(IVec2::new(x as i32, y as i32), IVec2::new(-1, 0)) {
                commands.spawn((
                    Sprite::from_color(Color::BLACK, room_size),
                    Transform::from_translation(pos + Vec3::new(-2., 0., 0.)),
                ));
            }
            if maze.can_go(IVec2::new(x as i32, y as i32), IVec2::new(0, -1)) {
                commands.spawn((
                    Sprite::from_color(Color::BLACK, room_size),
                    Transform::from_translation(pos + Vec3::new(0., 2., 0.)),
                ));
            }
        }
    }
}

fn spawn_item(commands: &mut Commands, pos: IVec2) {
    let grid_pos = GridPos {
        x: pos.x as usize,
        y: pos.y as usize,
        priority: 2.,
    };

    commands.spawn((
        Sprite::from_color(Color::srgb(1., 1., 0.), Vec2::new(ITEM_SIZE, ITEM_SIZE)),
        Transform::from(grid_pos),
        MazeItem { pos },
    ));
}

fn spawn_player(commands: &mut Commands, pos: IVec2) {
    let grid_pos = GridPos {
        x: pos.x as usize,
        y: pos.y as usize,
        priority: 3.,
    };

    commands.spawn((
        Sprite::from_color(Color::srgb(0., 1., 0.), Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
        Transform::from(grid_pos),
        MazePlayer {
            pos,
            next_pos: pos,
            ..Default::default()
        },
    ));
}

fn item_collect_system(
    mut commands: Commands,
    player_query: Query<&MazePlayer>,
    mut item_query: Query<(&mut MazeItem, Entity)>,
) {
    let player = player_query.single();
    for (item, entity) in &mut item_query {
        if player.pos == item.pos {
            commands.entity(entity).despawn();
        }
    }
}

const fn game_result() {}

fn keyboard_events_maze(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut MazePlayer>,
) {
    let mut player = player_query.single_mut();
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        player.input.x = 0;
        player.input.y = -1;
    } else if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        player.input.x = 0;
        player.input.y = 1;
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        player.input.x = -1;
        player.input.y = 0;
    } else if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        player.input.x = 1;
        player.input.y = 0;
    } else {
        player.input.x = 0;
        player.input.y = 0;
    }
}

fn gridbase_player_move_system(
    mut player_query: Query<(&mut MazePlayer, &mut Transform)>,
    time: ResMut<Time<Fixed>>,
    maze_query: Query<&MazeGenerator>,
) {
    let maze = maze_query.single();
    if let Ok((mut player, mut transform)) = player_query.get_single_mut() {
        if player.input != IVec2::ZERO || player.is_moving {
            if !player.is_moving && maze.can_go(player.pos, player.input) {
                player.alpha = 0.;
                player.is_moving = true;
                player.next_pos = player.pos + player.input;
            }

            let src_grid_pos = GridPos {
                x: player.pos.x as usize,
                y: player.pos.y as usize,
                priority: 3.,
            };
            let dst_grid_pos = GridPos {
                x: player.next_pos.x as usize,
                y: player.next_pos.y as usize,
                priority: 3.,
            };
            let src: Vec3 = src_grid_pos.into();
            let dst: Vec3 = dst_grid_pos.into();

            player.alpha += time.delta_secs() * MOVE_SPEED;
            if player.alpha >= 1. {
                player.alpha = 1.;
                player.is_moving = false;
                player.pos = player.next_pos;
            }

            transform.translation = src.lerp(dst, player.alpha);
        }
    }
}
