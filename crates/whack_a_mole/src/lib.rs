use bevy::prelude::*;
use bevy::utils::Duration;
use bits_helpers::WINDOW_WIDTH;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas, EmojiPlugin};
use bits_helpers::input::just_pressed_world_position;
use ribbit::WhackAMole;
use ui::{BottomTextUI, CenterTextUI, ScoreUI, TimeUI};

mod ribbit;
mod ui;

const COLUMN: usize = 3;
const ROW: usize = 4;

const HOLE_SCALE: f32 = 0.9;
const HOLE_THICKNESS: f32 = 0.1;
const HOLE_COLOR: Color = Color::srgba(1., 1., 1., 1.);
const HOLE_OFFSET: f32 = WINDOW_WIDTH / (COLUMN as f32);
const HOLE_RADIUS: f32 = HOLE_OFFSET * HOLE_SCALE * 0.5;

#[derive(Component, Default)]
struct Mole {
    timer: Timer,
}

#[derive(Component, Default)]
struct Wave {
    count: usize,
    timer: Timer,
}

#[derive(Component, Default)]
struct GameTimer {
    timer: Timer,
}

#[derive(Component, Default)]
struct EmojiIndices {
    indices: Vec<usize>,
}

#[derive(Bundle, Default)]
struct GameManager {
    game_timer: GameTimer,
    wave: Wave,
    emojis: EmojiIndices,
}

#[derive(Resource)]
struct Grid {
    grid: Vec<Vec2>,
}

#[derive(Resource, Default)]
struct GameProgress {
    score: u32,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Init,
    Game,
    Result,
}

pub fn run() {
    bits_helpers::get_default_app::<WhackAMole>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .add_plugins(EmojiPlugin)
        .add_plugins(ui::UIPlugin)
        .init_state::<GameState>()
        .insert_resource(setup_grid())
        .insert_resource(GameProgress::default())
        .add_systems(OnEnter(GameState::Init), init_enter)
        .add_systems(OnEnter(GameState::Game), game_enter)
        .add_systems(OnEnter(GameState::Result), result_enter)
        .add_systems(
            Update,
            (
                init.run_if(in_state(GameState::Init)),
                update,
                update_mole,
                update_game_timer.run_if(in_state(GameState::Game)),
                update_wave.run_if(in_state(GameState::Game)),
                result.run_if(in_state(GameState::Result)),
            ),
        )
        .run();
}

fn init_enter(
    mut commands: Commands,
    grid: Res<Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut score_text: ResMut<ScoreUI>,
    mut time_text: ResMut<TimeUI>,
    mut center_text: ResMut<CenterTextUI>,
    mut bottom_text: ResMut<BottomTextUI>,
) {
    commands.spawn(Camera2d);
    spawn_background(&mut commands, &grid, &mut meshes, &mut materials);

    // Initialize UI plugin
    score_text.set_digit(2);
    score_text.update(0);
    score_text.set_visiblity(Visibility::Inherited);
    time_text.update(Duration::new(15, 0));
    time_text.set_visiblity(Visibility::Inherited);
    center_text.update("Game Over!".to_string());
    center_text.set_visiblity(Visibility::Hidden);
    bottom_text.update("Whack a mole!".to_string());
    bottom_text.set_visiblity(Visibility::Inherited);
}

fn init(validation: Res<AtlasValidation>, mut next_state: ResMut<NextState<GameState>>) {
    if emoji::is_emoji_system_ready(&validation) {
        next_state.set(GameState::Game);
    }
}

fn setup_grid() -> Grid {
    // Grid
    let positions: Vec<Vec2> = (0..(COLUMN * ROW))
        .map(|x| {
            Vec2::new(
                ((x % COLUMN) as f32).mul_add(
                    HOLE_OFFSET,
                    HOLE_OFFSET * 0.5f32.mul_add(((COLUMN - 1) % 2) as f32, -((COLUMN / 2) as f32)),
                ),
                ((x / COLUMN) as f32).mul_add(
                    HOLE_OFFSET,
                    HOLE_OFFSET * 0.5f32.mul_add(((ROW - 1) % 2) as f32, -((ROW / 2) as f32)),
                ),
            )
        })
        .collect();
    Grid { grid: positions }
}

fn spawn_background(
    commands: &mut Commands,
    grid: &Res<Grid>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let shape = meshes.add(Annulus::new(
        HOLE_RADIUS * (1.0 - HOLE_THICKNESS),
        HOLE_RADIUS,
    ));

    for pos in &grid.grid {
        commands.spawn((
            Mesh2d(shape.clone()),
            MeshMaterial2d(materials.add(HOLE_COLOR)),
            Transform::from_xyz(pos.x, pos.y, -10.),
        ));
    }
}

fn game_enter(
    mut commands: Commands,
    mut game: ResMut<GameProgress>,
    mut score_text: ResMut<ScoreUI>,
    mut center_text: ResMut<CenterTextUI>,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
) {
    commands.spawn(GameManager {
        game_timer: GameTimer {
            timer: Timer::new(Duration::from_secs(15), TimerMode::Once),
        },
        wave: Wave {
            timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
            ..default()
        },
        emojis: EmojiIndices {
            indices: emoji::get_random_emojis(&atlas, &validation, 1),
        },
    });
    game.score = 0;
    score_text.update(game.score);
    center_text.set_visiblity(Visibility::Hidden);
}

fn update(
    mut commands: Commands,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    state: Res<State<GameState>>,
    mut game: ResMut<GameProgress>,
    mut score_text: ResMut<ScoreUI>,
    mut next_state: ResMut<NextState<GameState>>,
    mut query: Query<(Entity, &Mole, &Transform)>,
) {
    if let Some(world_position) = just_pressed_world_position(
        &mouse_button_input,
        &touch_input,
        &window_query,
        &camera_query,
    ) {
        if *state.get() == GameState::Result {
            next_state.set(GameState::Game);
            return;
        }
        println!("{world_position}");

        for (entity, _, transform) in &mut query {
            let diff = Vec2::new(
                transform.translation.x - world_position.x,
                transform.translation.y - world_position.y,
            );
            if diff.length_squared() < HOLE_RADIUS * HOLE_RADIUS {
                game.score += 1;
                score_text.update(game.score);
                commands.entity(entity).despawn();
            }
        }
    }
}

fn update_game_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut time_text: ResMut<TimeUI>,
    mut query: Query<(Entity, &mut GameTimer, &Wave)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (entity, mut game_timer, _wave) = query.single_mut();
    game_timer.timer.tick(time.delta());
    time_text.update(game_timer.timer.remaining());
    if game_timer.timer.finished() {
        commands.entity(entity).despawn();
        next_state.set(GameState::Result);
    }
}

fn update_wave(
    mut commands: Commands,
    grid: Res<Grid>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Wave, &EmojiIndices)>,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
) {
    let (_entity, mut wave, emojis) = query.single_mut();
    wave.timer.tick(time.delta());
    if wave.timer.finished() {
        let num = fastrand::usize(1..std::cmp::max(2, wave.count >> 1));
        for _i in 0..num {
            let idx = fastrand::usize(0..grid.grid.len());
            let pos = grid.grid.get(idx).expect("");
            spawn_mole(
                &mut commands,
                *pos,
                &atlas,
                &validation,
                *emojis.indices.first().expect(""),
            );
        }
        wave.count += 1;
    }
}

fn update_mole(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Mole)>) {
    for (entity, mut mole) in &mut query {
        mole.timer.tick(time.delta());
        if mole.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_mole(
    commands: &mut Commands,
    pos: Vec2,
    atlas: &Res<EmojiAtlas>,
    validation: &Res<AtlasValidation>,
    index: usize,
) {
    if let Some(id) = emoji::spawn_emoji(
        commands,
        atlas,
        validation,
        index,
        Transform::from_xyz(pos.x, pos.y, 0.),
    ) {
        commands.entity(id).insert(Mole {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
        });
    }
}

fn result_enter(mut center_text: ResMut<CenterTextUI>) {
    center_text.set_visiblity(Visibility::Inherited);
}

const fn result() {}
