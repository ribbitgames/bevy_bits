use std::time::Duration;

use bevy::prelude::*;
use bits_helpers::input::{just_pressed_world_position, just_released_world_position};
use bits_helpers::FONT;
use puzzle15::{Panel, PuzzlePanels};
use ribbit::Puzzle15;

mod puzzle15;
mod ribbit;

const PANEL_OFFSET: f32 = 80.;
const PANEL_SIZE: f32 = 78.;
const CENTER_OFFSET: f32 = -PANEL_OFFSET * 1.5;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Init,
    Game,
    Result,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum PanelState {
    #[default]
    StandBy,
    Slide,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component, Clone, Copy, Default)]
pub struct GridPos {
    pub x: usize,
    pub y: usize,
}

impl GridPos {
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self {
            //x: ((x - CENTER_OFFSET) / PANEL_OFFSET) as usize,
            //y: ((-y - CENTER_OFFSET) / PANEL_OFFSET) as usize,
            x: ((x + 160.) / PANEL_OFFSET) as usize,
            y: ((-y + 160.) / PANEL_OFFSET) as usize,
        }
    }
}

impl From<GridPos> for Transform {
    fn from(pos: GridPos) -> Self {
        Self::from_xyz(
            (pos.x as f32).mul_add(PANEL_OFFSET, CENTER_OFFSET),
            -(pos.y as f32).mul_add(PANEL_OFFSET, CENTER_OFFSET),
            0.,
        )
    }
}

impl From<GridPos> for Vec3 {
    fn from(pos: GridPos) -> Self {
        Self::new(
            (pos.x as f32).mul_add(PANEL_OFFSET, CENTER_OFFSET),
            -(pos.y as f32).mul_add(PANEL_OFFSET, CENTER_OFFSET),
            0.,
        )
    }
}

#[derive(Component, Default)]
struct PuzzlePlayer {
    pos: Vec2,
    index: Option<usize>,
    alpha: f32,
}

#[derive(Component)]
pub struct PanelVisual {
    index: usize,
}

#[derive(Component)]
struct ResultTimer {
    timer: Timer,
}

pub fn run() {
    bits_helpers::get_default_app::<Puzzle15>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .init_state::<GameState>()
        .init_state::<PanelState>()
        .add_systems(OnEnter(GameState::Game), reset_puzzle)
        .add_systems(OnEnter(GameState::Result), result_init)
        .add_systems(
            Update,
            (
                init_puzzle.run_if(in_state(GameState::Init)),
                mouse_events_puzzle
                    .run_if(in_state(GameState::Game))
                    .run_if(in_state(PanelState::StandBy)),
                panel_slide_system
                    .run_if(in_state(GameState::Game))
                    .run_if(in_state(PanelState::Slide)),
                result_puzzle.run_if(in_state(GameState::Result)),
            ),
        )
        .run();
}

fn init_puzzle(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    // Camera
    commands.spawn(Camera2d).insert(MainCamera);
    // Player
    commands.spawn(PuzzlePlayer {
        pos: Vec2::new(0., 0.),
        ..Default::default()
    });
    // Frame
    commands
        .spawn((
            Sprite::from_color(Color::WHITE, Vec2::new(336., 336.)),
            Transform::from_xyz(0., 0., -10.),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite::from_color(Color::BLACK, Vec2::new(320., 320.)),
                Transform::from_xyz(0., 0., 5.),
            ));
        });
    // Puzzle Panels
    commands.spawn(PuzzlePanels::new(4, 4));
    next_state.set(GameState::Game);
}

fn reset_puzzle(
    mut commands: Commands,
    mut panels_query: Query<&mut PuzzlePanels>,
    asset_server: Res<AssetServer>,
) {
    let mut puzzle_panels = panels_query.single_mut();
    puzzle_panels.reset();
    println!("{}", *puzzle_panels);
    puzzle_panels.slide_random(4);
    println!("{}", *puzzle_panels);
    spawn_panels(&mut commands, &puzzle_panels, &asset_server);
    println!("reset puzzle");
}

fn spawn_panels(
    commands: &mut Commands,
    puzzle_panels: &PuzzlePanels,
    asset_server: &Res<AssetServer>,
) {
    let panel_size = Vec2::new(PANEL_SIZE, PANEL_SIZE);
    for y in 0..puzzle_panels.height() {
        for x in 0..puzzle_panels.width() {
            let panel_number = match puzzle_panels.get_panel(x, y) {
                Some(Panel::PanelNumber(number)) => (*number) as usize,
                _ => {
                    continue;
                }
            };
            commands
                .spawn((
                    Sprite::from_color(Color::WHITE, panel_size),
                    PanelVisual {
                        index: panel_number,
                    },
                    GridPos { x, y },
                    Transform::from(GridPos { x, y }),
                ))
                .with_child((
                    Text2d::new(panel_number.to_string()),
                    TextFont {
                        font: asset_server.load(FONT),
                        font_size: 60.,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                    Transform::from_xyz(0., 0., 10.),
                ));
        }
    }
}

fn panel_slide_system(
    panels_query: Query<&PuzzlePanels>,
    mut player_query: Query<&mut PuzzlePlayer>,
    mut panel_query: Query<(&PanelVisual, &mut GridPos, &mut Transform)>,
    time: ResMut<Time<Fixed>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_panel_state: ResMut<NextState<PanelState>>,
) {
    let puzzle_panels = panels_query.single();
    let mut player = player_query.single_mut();
    player.alpha += time.delta_secs() * 8.;
    if player.alpha > 1. {
        player.alpha = 1.;
    }
    for (panel, mut pos, mut transform) in &mut panel_query {
        let index = puzzle_panels.get_index(panel.index as i32);
        //pos.x = index.x as usize;
        //pos.y = index.y as usize;
        //*transform = (*pos).into();

        let src: Vec3 = (*pos).into();
        let dst_grid_pos = GridPos {
            x: index.x as usize,
            y: index.y as usize,
        };
        let dst = dst_grid_pos.into();
        transform.translation = src.lerp(dst, player.alpha);
        if player.alpha >= 1. {
            pos.x = index.x as usize;
            pos.y = index.y as usize;
        }
    }
    if player.alpha >= 1. {
        player.alpha = 0.;
        next_panel_state.set(PanelState::StandBy);
        if puzzle_panels.is_solved() {
            next_game_state.set(GameState::Result);
        }
    }
}

fn result_init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Text2d::new("Good job!"),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 48.,
                ..default()
            },
            TextColor(Color::BLACK),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_xyz(0., 240., 10.),
        ))
        .insert(ResultTimer {
            timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
        });
    println!("result init");
}

fn result_puzzle(
    mut commands: Commands,
    mut timer_query: Query<(Entity, &mut ResultTimer)>,
    time: Res<Time>,
    mut p_q: Query<(Entity, &mut PanelVisual)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    //println!("result main");
    let (timer_entity, mut timer) = timer_query.single_mut();
    timer.timer.tick(time.delta());
    if timer.timer.finished() {
        commands.entity(timer_entity).despawn();
        for (entity, _panel) in &mut p_q {
            commands.entity(entity).despawn_recursive();
        }
        next_state.set(GameState::Game);
    }
}

fn mouse_events_puzzle(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut player_query: Query<&mut PuzzlePlayer>,
    mut panels_query: Query<&mut PuzzlePanels>,
    p_query: Query<(&PanelVisual, &Sprite, &Transform)>,
    mut next_state: ResMut<NextState<PanelState>>,
) {
    if let Some(world_position) =
        just_pressed_world_position(&mouse_button_input, &touch_input, &window, &camera)
    {
        let mut player = player_query.single_mut();
        player.pos = world_position;
        player.index = get_sprite_at_pos(world_position, &p_query);
    };

    if let Some(world_position) =
        just_released_world_position(&mouse_button_input, &touch_input, &window, &camera)
    {
        let player = player_query.single();
        let Some(index) = player.index else {
            return;
        };

        let mut puzzle_panels = panels_query.single_mut();
        let diff = world_position - player.pos;
        if get_sprite_at_pos(world_position, &p_query) == player.index {
            return;
        }
        let pos = puzzle_panels.get_index(index as i32);
        if diff.x.abs() > diff.y.abs() {
            let dir = IVec2::new(diff.x.signum() as i32, 0);
            puzzle_panels.slide(IVec2::new(pos.x, pos.y), dir);
            println!("{}", *puzzle_panels);
            println!("released dir = {dir}");
        } else {
            let dir = IVec2::new(0, -diff.y.signum() as i32);
            puzzle_panels.slide(IVec2::new(pos.x, pos.y), dir);
            println!("{}", *puzzle_panels);
            println!("released dir = {dir}");
        }
        //if puzzle_panels.is_solved() {
        //    next_state.set(Puzzle15GameState::Result);
        //}
        next_state.set(PanelState::Slide);
    };
}

fn get_sprite_at_pos(
    pos: Vec2,
    p_query: &Query<(&PanelVisual, &Sprite, &Transform)>,
) -> Option<usize> {
    for (panel, sprite, transform) in p_query {
        let size = sprite.custom_size.unwrap_or(Vec2::new(1.0, 1.0));
        let rect = Rect::from_center_size(transform.translation.truncate(), size);
        if rect.contains(pos) {
            println!("{} is in the panel {}", pos, panel.index);
            return Some(panel.index);
        }
    }
    None
}
