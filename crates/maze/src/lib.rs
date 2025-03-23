use std::f32::consts::SQRT_2;

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::utils::default;
use bits_helpers::emoji::{self, AtlasValidation, EMOJI_SIZE, EmojiAtlas, EmojiPlugin};
use bits_helpers::input::{
    just_pressed_world_position, just_released_world_position, pressed_world_position,
};
use bits_helpers::send_bit_message;
use maze::MazeGenerator;
use ribbit::Maze;
use ribbit_bits::{BitMessage, BitResult};

mod maze;
mod ribbit;

const MAZE_SIZE_X: usize = 14;
const MAZE_SIZE_Y: usize = 14;

const TILE_SIZE: f32 = 24.;
const WALL_DEPTH: f32 = 4.;

const NUM_ITEMS: usize = 3;
const ITEM_SIZE: f32 = TILE_SIZE * 0.625;

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

#[derive(Component)]
struct LifespanGame;

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

#[derive(Resource, Default)]
struct GameManager {
    count: usize,
    keyboard: bool,
}

const VIRTUAL_CONTROLLER_FRAME_SIZE: f32 = 64.0;
const VIRTUAL_CONTROLLER_LEVER_SIZE: f32 = 32.0;
const VIRTUAL_CONTROLLER_THRESHOLD: f32 = 0.5;
const VIRTUAL_CONTROLLER_NEUTRAL: f32 = 0.25;
const VIRTUAL_CONTROLLER_COLOR: Color = Color::srgba(0.75, 0.75, 0.75, 0.5);

#[derive(Component, Default)]
struct VirtualControllerFrame;

#[derive(Component, Default)]
struct VirtualControllerLever;

pub fn run() {
    bits_helpers::get_default_app::<Maze>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .add_plugins(EmojiPlugin)
        .init_state::<GameState>()
        .insert_resource(GameManager::default())
        .add_systems(OnEnter(GameState::Init), init_enter)
        .add_systems(OnEnter(GameState::Game), reset_maze)
        .add_systems(
            Update,
            (
                init.run_if(in_state(GameState::Init)),
                item_collect_system.run_if(in_state(GameState::Game)),
                gridbase_player_move_system.run_if(in_state(GameState::Game)),
                game_result.run_if(in_state(GameState::Result)),
                keyboard_events_maze.run_if(in_state(GameState::Game)),
                input_events_maze.run_if(in_state(GameState::Game)),
            ),
        )
        .run();
}

fn init_enter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("init");
    // Camera
    commands.spawn(Camera2d).insert(MainCamera);
    // Maze Generator
    commands.spawn(MazeGenerator::new(MAZE_SIZE_X, MAZE_SIZE_Y));
    // Spawn maze grid (It needs only once)
    //spawn_grid(&mut commands);
    spawn_objective(&mut commands);
    // Virtual controller
    spawn_virtual_controller(&mut commands, &mut meshes, &mut materials);
}

fn init(
    mut commands: Commands,
    validation: Res<AtlasValidation>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if emoji::is_emoji_system_ready(&validation) {
        // Spawn maze grid (It needs only once)
        spawn_grid(&mut commands);
        next_state.set(GameState::Game);
    }
}

fn spawn_grid(commands: &mut Commands) {
    let room_size = Vec2::new(TILE_SIZE - WALL_DEPTH, TILE_SIZE - WALL_DEPTH);
    for y in 0..MAZE_SIZE_Y {
        for x in 0..MAZE_SIZE_X {
            let grid_pos = GridPos {
                x,
                y,
                priority: -10.,
            };
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
    commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            display: Display::Grid,
            //grid_template_rows: RepeatedGridTrack::fr(3, 1.),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    align_self: AlignSelf::Start,
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                Text::new("Collect  !"),
                TextFont {
                    font_size: 42.,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    /*
    commands.spawn((
        Sprite::from_color(Color::srgb(1., 1., 0.), Vec2::new(32., 32.)),
        Transform::from_xyz(72., 296., 1.),
    ));
    */
}

fn spawn_virtual_controller(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let shape_frame = meshes.add(Annulus::new(
        VIRTUAL_CONTROLLER_FRAME_SIZE * 0.875,
        VIRTUAL_CONTROLLER_FRAME_SIZE,
    ));

    let shape_lever = meshes.add(Circle::new(VIRTUAL_CONTROLLER_LEVER_SIZE));

    commands.spawn((
        Mesh2d(shape_frame),
        MeshMaterial2d(materials.add(VIRTUAL_CONTROLLER_COLOR)),
        Transform::from_xyz(0., 0., 10.),
        VirtualControllerFrame,
        Visibility::Hidden,
    ));
    commands.spawn((
        Mesh2d(shape_lever),
        MeshMaterial2d(materials.add(VIRTUAL_CONTROLLER_COLOR)),
        Transform::from_xyz(0., 0., 10.),
        VirtualControllerLever,
        Visibility::Hidden,
    ));
}

fn reset_maze(
    mut commands: Commands,
    mut game_manager: ResMut<GameManager>,
    mut maze_query: Query<&mut MazeGenerator>,
    mut clear_query: Query<Entity, With<LifespanGame>>,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
) {
    println!("reset");
    clear_maze(&mut commands, &mut clear_query);
    let mut maze = maze_query.single_mut();
    // Generate a new maze
    maze.generate();
    println!("{}", *maze);
    spawn_maze(&mut commands, &maze);

    // Get all the deadends of the maze and shuffle them using fastrand
    let mut deadends = maze.get_deadends();
    // Replace rand::rng() with fastrand shuffle
    for i in (1..deadends.len()).rev() {
        let j = fastrand::usize(..=i);
        deadends.swap(i, j);
    }

    let indices = emoji::get_random_emojis(&atlas, &validation, 2);

    // Rest of the function remains the same
    let loop_num = (NUM_ITEMS + 1).min(deadends.len());
    for (i, deadend) in deadends.iter().enumerate() {
        if i >= loop_num {
            break;
        }
        if i == 0 {
            spawn_player(
                &mut commands,
                *deadend,
                &atlas,
                &validation,
                *indices.first().expect(""),
            );
        } else {
            spawn_item(
                &mut commands,
                *deadend,
                &atlas,
                &validation,
                *indices.last().expect(""),
            );
        }
    }

    // for objective
    if let Some(id) = emoji::spawn_emoji(
        &mut commands,
        &atlas,
        &validation,
        *indices.last().expect(""),
        Transform::from_xyz(72., 296., 1.).with_scale(Vec3::new(
            32. / EMOJI_SIZE.x as f32,
            32. / EMOJI_SIZE.y as f32,
            1.,
        )),
    ) {
        commands.entity(id).insert(LifespanGame);
    }

    game_manager.count = 0;
}

fn clear_maze(commands: &mut Commands, query: &mut Query<Entity, With<LifespanGame>>) {
    for entity in query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_maze(commands: &mut Commands, maze: &MazeGenerator) {
    let room_size = Vec2::new(TILE_SIZE - WALL_DEPTH, TILE_SIZE - WALL_DEPTH);

    for y in 0..maze.height() {
        for x in 0..maze.width() {
            let grid_pos = GridPos { x, y, priority: 0. };
            let pos: Vec3 = grid_pos.into();
            if maze.can_go(IVec2::new(x as i32, y as i32), IVec2::new(1, 0)) {
                commands.spawn((
                    Sprite::from_color(Color::BLACK, room_size),
                    Transform::from_translation(pos + Vec3::new(2., 0., 0.)),
                    LifespanGame,
                ));
            }
            if maze.can_go(IVec2::new(x as i32, y as i32), IVec2::new(0, 1)) {
                commands.spawn((
                    Sprite::from_color(Color::BLACK, room_size),
                    Transform::from_translation(pos + Vec3::new(0., -2., 0.)),
                    LifespanGame,
                ));
            }
            if maze.can_go(IVec2::new(x as i32, y as i32), IVec2::new(-1, 0)) {
                commands.spawn((
                    Sprite::from_color(Color::BLACK, room_size),
                    Transform::from_translation(pos + Vec3::new(-2., 0., 0.)),
                    LifespanGame,
                ));
            }
            if maze.can_go(IVec2::new(x as i32, y as i32), IVec2::new(0, -1)) {
                commands.spawn((
                    Sprite::from_color(Color::BLACK, room_size),
                    Transform::from_translation(pos + Vec3::new(0., 2., 0.)),
                    LifespanGame,
                ));
            }
        }
    }
}

fn spawn_item(
    commands: &mut Commands,
    pos: IVec2,
    atlas: &Res<EmojiAtlas>,
    validation: &Res<AtlasValidation>,
    index: usize,
) {
    let grid_pos = GridPos {
        x: pos.x as usize,
        y: pos.y as usize,
        priority: 2.,
    };
    if let Some(id) = emoji::spawn_emoji(
        commands,
        atlas,
        validation,
        index,
        Transform::from(grid_pos).with_scale(Vec3::new(
            ITEM_SIZE / EMOJI_SIZE.x as f32,
            ITEM_SIZE / EMOJI_SIZE.y as f32,
            1.,
        )),
    ) {
        commands
            .entity(id)
            .insert(MazeItem { pos })
            .insert(LifespanGame);
    }
}

fn spawn_player(
    commands: &mut Commands,
    pos: IVec2,
    atlas: &Res<EmojiAtlas>,
    validation: &Res<AtlasValidation>,
    index: usize,
) {
    let grid_pos = GridPos {
        x: pos.x as usize,
        y: pos.y as usize,
        priority: 3.,
    };
    if let Some(id) = emoji::spawn_emoji(
        commands,
        atlas,
        validation,
        index,
        Transform::from(grid_pos).with_scale(Vec3::new(
            PLAYER_SIZE / EMOJI_SIZE.x as f32,
            PLAYER_SIZE / EMOJI_SIZE.y as f32,
            1.,
        )),
    ) {
        commands
            .entity(id)
            .insert(MazePlayer {
                pos,
                next_pos: pos,
                ..Default::default()
            })
            .insert(LifespanGame);
    }
}

fn item_collect_system(
    mut commands: Commands,
    mut game_manager: ResMut<GameManager>,
    player_query: Query<&MazePlayer>,
    mut item_query: Query<(&mut MazeItem, Entity)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player = player_query.single();
    for (item, entity) in &mut item_query {
        if player.pos == item.pos {
            commands.entity(entity).despawn();
            game_manager.count += 1;
            if game_manager.count >= NUM_ITEMS {
                send_bit_message(BitMessage::End(BitResult::Success));
                next_state.set(GameState::Result);
            }
        }
    }
}

const fn game_result() {}

fn keyboard_events_maze(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_manager: ResMut<GameManager>,
    mut player_query: Query<&mut MazePlayer>,
) {
    let mut player = player_query.single_mut();
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        player.input.x = 0;
        player.input.y = -1;
        game_manager.keyboard = true;
    } else if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        player.input.x = 0;
        player.input.y = 1;
        game_manager.keyboard = true;
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        player.input.x = -1;
        player.input.y = 0;
        game_manager.keyboard = true;
    } else if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        player.input.x = 1;
        player.input.y = 0;
        game_manager.keyboard = true;
    } else if game_manager.keyboard {
        player.input.x = 0;
        player.input.y = 0;
    }
}

fn input_events_maze(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut game_manager: ResMut<GameManager>,
    mut player_query: Query<&mut MazePlayer>,
    mut frame_query: Query<
        (&mut Transform, &mut Visibility),
        (
            With<VirtualControllerFrame>,
            Without<VirtualControllerLever>,
        ),
    >,
    mut lever_query: Query<
        (&mut Transform, &mut Visibility),
        (
            With<VirtualControllerLever>,
            Without<VirtualControllerFrame>,
        ),
    >,
) {
    if let Some(world_position) =
        just_pressed_world_position(&mouse_button_input, &touch_input, &window, &camera)
    {
        let (mut frame, mut frame_visibility) = frame_query.single_mut();
        frame.translation.x = world_position.x;
        frame.translation.y = world_position.y;
        *frame_visibility = Visibility::Inherited;
        let (_, mut lever_visiblity) = lever_query.single_mut();
        *lever_visiblity = Visibility::Inherited;
        game_manager.keyboard = false;
    };

    if let Some(world_position) =
        pressed_world_position(&mouse_button_input, &touch_input, &window, &camera)
    {
        let (frame, _) = frame_query.single_mut();
        let (mut lever, _) = lever_query.single_mut();
        let diff = Vec2::new(
            world_position.x - frame.translation.x,
            world_position.y - frame.translation.y,
        );
        let limit = VIRTUAL_CONTROLLER_FRAME_SIZE - VIRTUAL_CONTROLLER_LEVER_SIZE;
        let dist = limit.mul_add(
            -VIRTUAL_CONTROLLER_NEUTRAL,
            diff.length()
                .clamp(limit * VIRTUAL_CONTROLLER_NEUTRAL, limit),
        );
        let input = (diff.normalize() * dist).normalize();
        let offset = input * limit;
        lever.translation.x = frame.translation.x + offset.x;
        lever.translation.y = frame.translation.y + offset.y;
        let mut player = player_query.single_mut();
        if input.length() < VIRTUAL_CONTROLLER_THRESHOLD {
            if !game_manager.keyboard {
                player.input.x = 0;
                player.input.y = 0;
            }
            return;
        }
        //println!("{input}");
        let dot_x = input.dot(Vec2::X);
        let dot_y = input.dot(Vec2::Y);
        let inv_sqrt_2 = 1. / SQRT_2;
        if dot_y > inv_sqrt_2 {
            player.input.x = 0;
            player.input.y = -1;
        } else if dot_y < -inv_sqrt_2 {
            player.input.x = 0;
            player.input.y = 1;
        } else if dot_x < -inv_sqrt_2 {
            player.input.x = -1;
            player.input.y = 0;
        } else if dot_x > inv_sqrt_2 {
            player.input.x = 1;
            player.input.y = 0;
        } else {
            player.input.x = 0;
            player.input.y = 0;
        }
    };

    if let Some(_world_position) =
        just_released_world_position(&mouse_button_input, &touch_input, &window, &camera)
    {
        let (_, mut frame_visibility) = frame_query.single_mut();
        *frame_visibility = Visibility::Hidden;
        let (_, mut lever_visibility) = lever_query.single_mut();
        *lever_visibility = Visibility::Hidden;

        let mut player = player_query.single_mut();
        if !game_manager.keyboard {
            player.input.x = 0;
            player.input.y = 0;
        }
    };
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
