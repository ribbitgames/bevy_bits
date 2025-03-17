use bevy::prelude::*;
use bevy::utils::Duration;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas, EmojiPlugin};
use bits_helpers::input::just_pressed_world_position;
use bits_helpers::{WINDOW_HEIGHT, WINDOW_WIDTH, send_bit_message};
use ribbit::WhackAMole;
use ribbit_bits::{BitMessage, BitResult};
use ui::{BottomTextUI, ScoreUI, TimeUI};

mod ribbit;
mod ui;

const COLUMN: usize = 3;
const ROW: usize = 4;

const TIME_LIMIT: u64 = 20;
const MOLE_VARIATIONS: usize = 3;

const HOLE_SCALE: f32 = 0.9;
const HOLE_THICKNESS: f32 = 0.1;
const HOLE_COLOR: Color = Color::srgba(1., 1., 1., 1.);
const HOLE_OFFSET: f32 = WINDOW_WIDTH / (COLUMN as f32);
const HOLE_RADIUS: f32 = HOLE_OFFSET * HOLE_SCALE * 0.5;

const POINTS: [i32; MOLE_VARIATIONS] = [1, -1, 5];

#[derive(Component, Default)]
struct LegendBase {
    index: usize,
}

#[derive(Component, Default)]
struct Legend;

#[derive(Component, Default)]
struct LifespanGame;

#[derive(Component, Default)]
struct Mole {
    timer: Timer,
    point: i32,
}

#[derive(Component, Default)]
struct Wave {
    count: usize,
    timer: Timer,
}

#[derive(Component, Default)]
struct Feedback {
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
    Reset,
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
        .add_systems(OnExit(GameState::Game), game_exit)
        .add_systems(
            Update,
            (
                init.run_if(in_state(GameState::Init)),
                update,
                update_mole,
                update_feedback,
                update_game_timer.run_if(in_state(GameState::Game)),
                update_wave.run_if(in_state(GameState::Game)),
                reset.run_if(in_state(GameState::Reset)),
            ),
        )
        .run();
}

fn reset(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    commands.insert_resource(GameProgress::default());
    next_state.set(GameState::Game);
}

fn init_enter(
    mut commands: Commands,
    grid: Res<Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut score_text: ResMut<ScoreUI>,
    mut time_text: ResMut<TimeUI>,
    mut bottom_text: ResMut<BottomTextUI>,
) {
    commands.spawn(Camera2d);
    spawn_background(&mut commands, &grid, &mut meshes, &mut materials);
    spawn_legend_base(&mut commands);

    // Initialize UI plugin
    score_text.set_digit(2);
    score_text.update(0);
    score_text.set_visiblity(Visibility::Inherited);
    time_text.update(Duration::new(TIME_LIMIT, 0));
    time_text.set_visiblity(Visibility::Inherited);
    bottom_text.update("Whack a mole!".to_string());
    bottom_text.set_visiblity(Visibility::Hidden);
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

fn spawn_legend_base(commands: &mut Commands) {
    let node_id = commands
        .spawn({
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::fr(MOLE_VARIATIONS as u16, 1.),
                ..default()
            }
        })
        .id();
    for (index, point) in POINTS.iter().enumerate() {
        commands.entity(node_id).with_children(|parent| {
            parent.spawn((
                Node {
                    align_self: AlignSelf::End,
                    justify_self: JustifySelf::Center,
                    ..default()
                },
                Text::new(format!("{point} pt")),
                LegendBase { index },
            ));
        });
    }
}

fn game_enter(
    mut commands: Commands,
    mut game: ResMut<GameProgress>,
    mut score_text: ResMut<ScoreUI>,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    query: Query<(&LegendBase, &GlobalTransform)>,
) {
    let indices = emoji::get_random_emojis(&atlas, &validation, MOLE_VARIATIONS);

    commands
        .spawn(GameManager {
            game_timer: GameTimer {
                timer: Timer::new(Duration::from_secs(TIME_LIMIT), TimerMode::Once),
            },
            wave: Wave {
                timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
                ..default()
            },
            emojis: EmojiIndices {
                indices: indices.clone(),
            },
        })
        .insert(LifespanGame);
    spawn_legend(&mut commands, &indices, &atlas, &validation, &query);
    game.score = 0;
    score_text.update(game.score);
}

fn spawn_legend(
    commands: &mut Commands,
    indices: &[usize],
    atlas: &Res<EmojiAtlas>,
    validation: &Res<AtlasValidation>,
    query: &Query<(&LegendBase, &GlobalTransform)>,
) {
    // It needs to be tied to the UI position (legend base) eventually...
    for (base, _transform) in query {
        if let Some(id) = emoji::spawn_emoji(
            commands,
            atlas,
            validation,
            *indices.get(base.index).expect(""),
            Transform {
                translation: Vec3::new(
                    (WINDOW_WIDTH / MOLE_VARIATIONS as f32).mul_add(
                        0.5,
                        WINDOW_WIDTH.mul_add(
                            -0.5,
                            base.index as f32 * WINDOW_WIDTH / MOLE_VARIATIONS as f32,
                        ),
                    ),
                    -WINDOW_HEIGHT * 0.425,
                    0.,
                ),
                scale: Vec3::new(0.5, 0.5, 1.),
                ..default()
            },
        ) {
            commands.entity(id).insert((Legend, LifespanGame));
        }
    }
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

        for (entity, mole, transform) in &mut query {
            let diff = Vec2::new(
                transform.translation.x - world_position.x,
                transform.translation.y - world_position.y,
            );
            if diff.length_squared() < HOLE_RADIUS * HOLE_RADIUS {
                match mole.point {
                    x if x > 0 => {
                        game.score = game.score.saturating_add(mole.point as u32);
                        spawn_feedback(&mut commands, transform.translation, mole.point);
                    }
                    x if x < 0 => {
                        game.score = game.score.saturating_sub(mole.point.unsigned_abs());
                        spawn_feedback(&mut commands, transform.translation, mole.point);
                    }
                    _ => (),
                }
                score_text.update(game.score);
                commands.entity(entity).despawn();
            }
        }
    }
}

fn update_game_timer(
    //mut commands: Commands,
    time: Res<Time>,
    mut time_text: ResMut<TimeUI>,
    mut query: Query<(Entity, &mut GameTimer, &Wave)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (_entity, mut game_timer, _wave) = query.single_mut();
    game_timer.timer.tick(time.delta());
    time_text.update(game_timer.timer.remaining());
    if game_timer.timer.finished() {
        //commands.entity(entity).despawn();
        send_bit_message(BitMessage::End(BitResult::Success));
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
        let mut positions = grid.grid.clone();
        fastrand::shuffle(&mut positions);
        let mut variations: Vec<usize> = (1..MOLE_VARIATIONS).collect();
        fastrand::shuffle(&mut variations);
        variations.insert(0, 0);
        let num = fastrand::usize(1..std::cmp::max(2, wave.count >> 1));
        for i in 0..num {
            let pos = positions.get(i).expect("");
            spawn_mole(
                &mut commands,
                *pos,
                &atlas,
                &validation,
                *variations.get(i).expect(""),
                &emojis.indices,
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
    variation: usize,
    emojis: &[usize],
) {
    let index = *emojis.get(variation).expect("");
    if let Some(id) = emoji::spawn_emoji(
        commands,
        atlas,
        validation,
        index,
        Transform::from_xyz(pos.x, pos.y, 0.),
    ) {
        let point = POINTS.get(variation).map_or(1, |p| *p);
        commands.entity(id).insert(Mole {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
            point,
        });
    }
}

fn spawn_feedback(commands: &mut Commands, pos: Vec3, point: i32) {
    let color = if point >= 0 {
        Color::srgb(0., 1., 0.)
    } else {
        Color::srgb(1., 0., 0.)
    };
    commands.spawn((
        Feedback {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
        },
        Text2d::new(format!("{point:+}")),
        Transform::from_xyz(pos.x, HOLE_OFFSET.mul_add(0.5, pos.y), 1.),
        TextColor(color),
    ));
}

fn update_feedback(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Feedback)>,
) {
    for (entity, mut feedback) in &mut query {
        feedback.timer.tick(time.delta());
        if feedback.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn game_exit(mut commands: Commands, query: Query<(Entity, &LifespanGame)>) {
    for (entity, _) in &query {
        commands.entity(entity).despawn_recursive();
    }
}
