use bevy::prelude::*;
use bevy::utils::Duration;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas, EmojiPlugin};
use bits_helpers::input::just_pressed_world_position;
use bits_helpers::{WINDOW_HEIGHT, WINDOW_WIDTH};
use ribbit::WheresWaldo;

mod ribbit;

const BACKGROUND_SIZE_X: f32 = WINDOW_WIDTH;
const BACKGROUND_SIZE_Y: f32 = BACKGROUND_SIZE_X;
const BACKGROUND_COLOR: Color = Color::Srgba(Srgba {
    red: 0.,
    green: 0.5,
    blue: 0.,
    alpha: 1.,
});

const SPRITE_SIZE_X: f32 = 32.;
const SPRITE_SIZE_Y: f32 = 32.;

const SPRITE_SCALE: f32 = SPRITE_SIZE_X / (emoji::EMOJI_SIZE.x as f32);

const UI_Y: f32 = -BACKGROUND_SIZE_Y * 0.5 - 32.;
const UI_RESULT_Y: f32 = UI_Y - 32.;

const COLLISION_RADIUS: f32 = SPRITE_SIZE_X * 0.75;

const NUMBER_OF_CANDIDATES: u32 = 40;

const MAX_MISTAKES: u32 = 3;
const MAX_DURATION: u64 = 15;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Character;

#[derive(Component)]
struct Waldo;

#[derive(Component)]
struct FeedbackUI {
    timer: Timer,
}

#[derive(Component)]
struct GameTimer {
    timer: Timer,
}

#[derive(Component)]
struct ProgressUI;

#[derive(Event)]
struct InquireEvent {
    pos: Vec2,
}

#[derive(Resource)]
struct Grid {
    grid: Vec<Vec2>,
}

#[derive(Resource)]
struct GameProgress {
    mistakes: i32,
    result: bool,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Init,
    Game,
    Result,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameSystemSet {
    Input,
    Action,
}

pub fn run() {
    bits_helpers::get_default_app::<WheresWaldo>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .add_plugins(EmojiPlugin)
        .init_state::<GameState>()
        .add_event::<InquireEvent>()
        .configure_sets(
            Update,
            (
                GameSystemSet::Input,
                GameSystemSet::Action.before(GameSystemSet::Input),
            ),
        )
        .add_systems(OnEnter(GameState::Init), init_enter)
        .add_systems(OnEnter(GameState::Game), game_enter)
        .add_systems(OnEnter(GameState::Result), result_enter)
        .add_systems(OnExit(GameState::Result), result_exit)
        .add_systems(
            Update,
            (
                mouse_events.in_set(GameSystemSet::Input),
                init.run_if(in_state(GameState::Init)),
                inquire_position
                    .in_set(GameSystemSet::Action)
                    .run_if(in_state(GameState::Game)),
                update_feedback_ui_timer.run_if(in_state(GameState::Game)),
                update_gametimer_ui_timer.run_if(in_state(GameState::Game)),
                update_progress_ui,
                result
                    .in_set(GameSystemSet::Action)
                    .run_if(in_state(GameState::Result)),
            ),
        )
        .run();
}

// For the initalizatino state
fn init_enter(mut commands: Commands) {
    setup_static_entities(&mut commands);
}

fn setup_static_entities(commands: &mut Commands) {
    // Camera
    commands.spawn(Camera2d).insert(MainCamera);
    // Background
    commands.spawn((
        Sprite::from_color(
            BACKGROUND_COLOR,
            Vec2::new(BACKGROUND_SIZE_X, BACKGROUND_SIZE_Y),
        ),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        Transform::from_xyz(0., 0., -10.),
    ));
    // UI
    commands.spawn((
        Text2d::new("Where's   ?"),
        TextFont {
            font_size: 24.,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0., UI_Y, 0.)),
    ));
    spawn_progress_ui(commands);

    // Grid
    let p: Vec<Vec2> = (0..121)
        .map(|x| {
            Vec2::new(
                (((x % 11) - 5) as f32) * SPRITE_SIZE_X,
                (((x / 11) - 5) as f32) * SPRITE_SIZE_Y,
            )
        })
        .collect();
    commands.insert_resource(Grid { grid: p });

    // Game Progress
    commands.insert_resource(GameProgress {
        mistakes: 0,
        result: false,
    });
}

fn init(validation: Res<AtlasValidation>, mut next_state: ResMut<NextState<GameState>>) {
    if emoji::is_emoji_system_ready(&validation) {
        next_state.set(GameState::Game);
    }
}

// For the game state
fn game_enter(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    mut progress: ResMut<GameProgress>,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
) {
    create_puzzle(&mut commands, &mut grid, &mut progress, &atlas, &validation);
}

// For the result state
fn result_enter(mut commands: Commands, progress: Res<GameProgress>) {
    if progress.result {
        spawn_feedback_ui(&mut commands, "Good Job!", 0);
    } else {
        spawn_feedback_ui(&mut commands, "Game Over!", 0);
    }
}

fn result(
    mut commands: Commands,
    query: Query<(Entity, &FeedbackUI)>,
    mut inquire_events: EventReader<InquireEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut ev_pos = Vec2::ZERO;
    let mut is_valid = false;

    for ev in inquire_events.read() {
        if !is_valid {
            is_valid = true;
            ev_pos = ev.pos;
        }
    }
    if is_valid {
        if ev_pos.x.abs() > WINDOW_WIDTH * 0.5 || ev_pos.y.abs() > WINDOW_HEIGHT * 0.5 {
            return;
        }
        for (entity, _feedback_ui) in &query {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Game);
    }
}

fn result_exit(
    mut commands: Commands,
    query: Query<(Entity, &Character)>,
    q: Query<(Entity, &GameTimer)>,
) {
    clear_puzzle(&mut commands, &query, &q);
}

// Game related
fn mouse_events(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut inquire_event: EventWriter<InquireEvent>,
) {
    if let Some(world_position) = just_pressed_world_position(
        &mouse_button_input,
        &touch_input,
        &window_query,
        &camera_query,
    ) {
        inquire_event.send(InquireEvent {
            pos: world_position,
        });
    }
}

fn get_random_transform(grid_position: Vec2) -> Transform {
    let mut rng = fastrand::Rng::new();
    let position_noize: f32 = 0.125;
    let rotation_noize: f32 = 0.25;
    Transform::from_translation(Vec3::new(
        (rng.f32() * position_noize).mul_add(SPRITE_SIZE_X, grid_position.x),
        (rng.f32() * position_noize).mul_add(SPRITE_SIZE_Y, grid_position.y),
        rng.f32(),
    ))
    .with_rotation(Quat::from_rotation_z(rng.f32() * rotation_noize))
}

fn create_puzzle(
    commands: &mut Commands,
    grid: &mut ResMut<Grid>,
    progress: &mut ResMut<GameProgress>,
    atlas: &Res<EmojiAtlas>,
    validation: &Res<AtlasValidation>,
) {
    // Get random emojis for the puzzle
    let selected_indices =
        emoji::get_random_emojis(atlas, validation, NUMBER_OF_CANDIDATES as usize);
    if selected_indices.is_empty() {
        return;
    }

    let mut grid_positions = Vec::new();
    for x in 0..8 {
        for y in 0..8 {
            grid_positions.push(Vec2::new(
                SPRITE_SIZE_X * (x as f32 - 3.5),
                SPRITE_SIZE_Y * (y as f32 - 3.5),
            ));
        }
    }
    fastrand::shuffle(&mut grid_positions);
    grid.grid.clone_from(&grid_positions);

    let waldo_index = *selected_indices
        .first()
        .expect("selected_indices has been checked earlier in the function");

    let waldo_position = grid_positions
        .pop()
        .expect("grid_positions should not be empty");
    if let Some(waldo_entity) = emoji::spawn_emoji(
        commands,
        atlas,
        validation,
        waldo_index,
        Transform {
            translation: get_random_transform(waldo_position).translation,
            scale: Vec3::splat(SPRITE_SCALE),
            ..default()
        },
    ) {
        commands.entity(waldo_entity).insert((Character, Waldo));
    }

    if let Some(waldo_entity) = emoji::spawn_emoji(
        commands,
        atlas,
        validation,
        waldo_index,
        Transform {
            translation: Vec3::new(50., UI_Y + 5.0, 0.),
            scale: Vec3::splat(SPRITE_SCALE),
            ..default()
        },
    ) {
        commands.entity(waldo_entity).insert(Character);
    }

    for (position, &index) in grid_positions
        .iter()
        .zip(selected_indices.iter().skip(1))
        .take(NUMBER_OF_CANDIDATES as usize - 1)
    {
        if let Some(entity) = emoji::spawn_emoji(
            commands,
            atlas,
            validation,
            index,
            Transform {
                translation: get_random_transform(*position).translation,
                scale: Vec3::splat(SPRITE_SCALE),
                ..default()
            },
        ) {
            commands.entity(entity).insert(Character);
        }
    }

    spawn_gametimer_ui(commands, MAX_DURATION);

    progress.mistakes = 0;
    progress.result = false;
}

fn clear_puzzle(
    commands: &mut Commands,
    query: &Query<(Entity, &Character)>,
    q: &Query<(Entity, &GameTimer)>,
) {
    for (entity, _character) in query {
        commands.entity(entity).despawn();
    }
    for (entity, _character) in q {
        commands.entity(entity).despawn();
    }
}

fn inquire_position(
    mut commands: Commands,
    mut inquire_event: EventReader<InquireEvent>,
    character_query: Query<(&Transform, &Character), Without<Waldo>>,
    waldo_query: Query<(&Transform, &Character), With<Waldo>>,
    mut progress: ResMut<GameProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut ev_pos = Vec2::ZERO;
    let mut is_valid = false;
    for ev in inquire_event.read() {
        //println!("{}", ev.pos);
        if !is_valid {
            is_valid = true;
            ev_pos = ev.pos;
        }
    }

    if is_valid {
        if ev_pos.x.abs() > BACKGROUND_SIZE_X * 0.5 || ev_pos.y.abs() > BACKGROUND_SIZE_Y * 0.5 {
            // pos is outside the background area
            return;
        }

        let squared_radius = COLLISION_RADIUS * COLLISION_RADIUS;
        for (transform, _character) in &waldo_query {
            let dist = ev_pos.distance_squared(transform.translation.truncate());
            if dist < squared_radius {
                progress.result = true;
                next_state.set(GameState::Result);
                return;
            }
        }

        for (transform, _character) in &character_query {
            let dist = ev_pos.distance_squared(transform.translation.truncate());
            if dist < squared_radius {
                progress.mistakes += 1;
                if progress.mistakes < 3 {
                    spawn_feedback_ui(&mut commands, "It's not me!", 1);
                } else {
                    next_state.set(GameState::Result);
                }
                return;
            }
        }
    }
}

// For UI
fn spawn_feedback_ui(commands: &mut Commands, text: &str, secs: u64) {
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size: 24.,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0., UI_RESULT_Y, 0.)),
        FeedbackUI {
            timer: Timer::new(Duration::from_secs(secs), TimerMode::Once),
        },
    ));
}

fn update_feedback_ui_timer(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FeedbackUI)>,
    time: Res<Time>,
) {
    for (entity, mut feedback) in &mut query {
        feedback.timer.tick(time.delta());
        if feedback.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_gametimer_ui(commands: &mut Commands, duration: u64) {
    let id = commands
        .spawn((
            Text2d::new(duration.to_string() + "s remains"),
            TextFont {
                font_size: 24.,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(0., -UI_RESULT_Y, 0.),
        ))
        .id();
    if duration > 0 {
        commands.entity(id).insert(GameTimer {
            timer: Timer::new(Duration::from_secs(duration), TimerMode::Once),
        });
    }
}

fn update_gametimer_ui_timer(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GameTimer, &mut Text2d)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut gametimer, mut text) in &mut query {
        gametimer.timer.tick(time.delta());
        let remain = gametimer.timer.duration() - gametimer.timer.elapsed();
        *text = Text2d::new(format!("{}s remains", remain.as_secs() + 1));
        if gametimer.timer.finished() {
            next_state.set(GameState::Result);
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_progress_ui(commands: &mut Commands) {
    commands.spawn((
        Text2d::new(format!("0/{MAX_MISTAKES} mistakes")),
        TextFont {
            font_size: 24.,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0., -UI_RESULT_Y - 32., 0.),
        ProgressUI,
    ));
}

fn update_progress_ui(mut query: Query<(&ProgressUI, &mut Text2d)>, progress: Res<GameProgress>) {
    for (_progress_ui, mut text) in &mut query {
        *text = Text2d::new(format!("{}/{} mistakes", progress.mistakes, MAX_MISTAKES,));
    }
}
