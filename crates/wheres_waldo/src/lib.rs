use bevy::prelude::*;
use bevy::utils::Duration;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas, EmojiPlugin};
use bits_helpers::input::just_pressed_world_position;
use rand::prelude::SliceRandom;
use rand::Rng;
use ribbit::WheresWaldo;

mod ribbit;

const BACKGROUND_SIZE_X: f32 = bits_helpers::WINDOW_WIDTH;
const BACKGROUND_SIZE_Y: f32 = BACKGROUND_SIZE_X;
const BACKGROUND_COLOR: Color = Color::Srgba(Srgba {
    red: 0.,
    green: 0.5,
    blue: 0.,
    alpha: 1.,
});

const SPRITE_SIZE_X: f32 = 32.;
const SPRITE_SIZE_Y: f32 = 32.;

// This should get from emoji::EMOJI_SIZE,
//or we should be able to actual sprite size
//for emoji plugin instead of specifying the scale
const SPRITE_SCALE: f32 = SPRITE_SIZE_X / 128.0;

const UI_Y: f32 = -BACKGROUND_SIZE_Y * 0.5 - 32.;
const UI_RESULT_Y: f32 = UI_Y - 32.;

const COLLISION_RADIUS: f32 = SPRITE_SIZE_X * 0.75;

const NUMBER_OF_CANDIDATES: u32 = 40;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Character;

#[derive(Component)]
struct Waldo;

#[derive(Component)]
struct FeedbackUI {
    timer: Timer,
    should_start_new_game: bool,
}

#[derive(Event)]
struct InquireEvent {
    pos: Vec2,
}

#[derive(Resource)]
struct Grid {
    grid: Vec<Vec2>,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Init,
    Game,
    Result,
}

pub fn run() {
    bits_helpers::get_default_app::<WheresWaldo>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .add_plugins(EmojiPlugin)
        .init_state::<GameState>()
        .add_event::<InquireEvent>()
        .add_systems(OnEnter(GameState::Init), setup_static_entities)
        .add_systems(OnEnter(GameState::Game), create_puzzle)
        .add_systems(OnExit(GameState::Result), clear_puzzle)
        .add_systems(
            Update,
            (
                mouse_events,
                wait_for_initialization.run_if(in_state(GameState::Init)),
                inquire_position.run_if(in_state(GameState::Game)),
                update_feedback_ui_timer.run_if(in_state(GameState::Game)),
                result.run_if(in_state(GameState::Result)),
            ),
        )
        .run();
}

fn result(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Game);
}

fn setup_static_entities(mut commands: Commands) {
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

    let p: Vec<Vec2> = (0..121)
        .map(|x| {
            Vec2::new(
                (((x % 11) - 5) as f32) * SPRITE_SIZE_X,
                (((x / 11) - 5) as f32) * SPRITE_SIZE_Y,
            )
        })
        .collect();
    commands.insert_resource(Grid { grid: p });
}

fn wait_for_initialization(
    validation: Res<AtlasValidation>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if emoji::is_emoji_system_ready(&validation) {
        next_state.set(GameState::Game);
    }
}

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
    let mut rng = rand::thread_rng();
    let position_noize: f32 = 0.125;
    let rotation_noize: f32 = 0.25;
    Transform::from_translation(Vec3::new(
        rng.gen_range(-position_noize..position_noize)
            .mul_add(SPRITE_SIZE_X, grid_position.x),
        rng.gen_range(-position_noize..position_noize)
            .mul_add(SPRITE_SIZE_Y, grid_position.y),
        rng.gen_range(0. ..1.),
    ))
    .with_rotation(Quat::from_rotation_z(
        rng.gen_range(-rotation_noize..rotation_noize),
    ))
}

fn create_puzzle(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
) {
    let mut rng = rand::thread_rng();

    // Trying to use similar emojis instead of complelte random ones
    let selected_index = *emoji::get_random_emojis(&atlas, &validation, 1)
        .first()
        .expect("");
    let mut selected_indices: Vec<usize> = (0..100).map(|x| selected_index + x - 50).collect();
    selected_indices.shuffle(&mut rng);
    selected_indices.retain(|index| emoji::is_valid_emoji_index(&atlas, *index));
    selected_indices.truncate(NUMBER_OF_CANDIDATES as usize);

    grid.grid.shuffle(&mut rng);

    for (count, pos) in grid.grid.clone().into_iter().enumerate() {
        if count >= (NUMBER_OF_CANDIDATES as usize) {
            break;
        }
        //println!("pos[{count}] = {pos}");

        let t = get_random_transform(pos);
        if let Some(entity) = emoji::spawn_emoji(
            &mut commands,
            &atlas,
            &validation,
            *selected_indices
                .get(count)
                .expect("The index is out of the range!"),
            Vec2::new(t.translation.x, t.translation.y),
            SPRITE_SCALE,
        ) {
            commands.entity(entity).insert(Character).insert(Transform {
                translation: t.translation,
                rotation: t.rotation,
                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
            });
            if count == 0 {
                commands.entity(entity).insert(Waldo);
            }
        }
    }

    // The one to look for (for UI)
    if let Some(entity) = emoji::spawn_emoji(
        &mut commands,
        &atlas,
        &validation,
        *selected_indices
            .first()
            .expect("The index is out of the range!"),
        Vec2::new(40., UI_Y),
        //SPRITE_SCALE,
        0.2,
    ) {
        commands.entity(entity).insert(Character).insert(Waldo);
    }
}

fn clear_puzzle(mut commands: Commands, query: Query<(Entity, &Character)>) {
    for (entity, _character) in &query {
        commands.entity(entity).despawn();
    }
}

fn inquire_position(
    mut commands: Commands,
    mut inquire_event: EventReader<InquireEvent>,
    character_query: Query<(&Transform, &Character), Without<Waldo>>,
    waldo_query: Query<(&Transform, &Character), With<Waldo>>,
) {
    for ev in inquire_event.read() {
        //println!("{}", ev.pos);
        if ev.pos.x.abs() > BACKGROUND_SIZE_X * 0.5 || ev.pos.y.abs() > BACKGROUND_SIZE_Y * 0.5 {
            // pos is outside the background area
            return;
        }

        let squared_radius = COLLISION_RADIUS * COLLISION_RADIUS;
        for (transform, _character) in &waldo_query {
            let dist = ev.pos.distance_squared(transform.translation.truncate());
            if dist < squared_radius {
                spawn_feedback_ui(&mut commands, true);
                return;
            }
        }

        for (transform, _character) in &character_query {
            let dist = ev.pos.distance_squared(transform.translation.truncate());
            if dist < squared_radius {
                spawn_feedback_ui(&mut commands, false);
                return;
            }
        }
    }
}

fn spawn_feedback_ui(commands: &mut Commands, result: bool) {
    commands.spawn((
        Text2d::new(if result { "Good job!" } else { "It's not me!" }),
        TextFont {
            font_size: 24.,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0., UI_RESULT_Y, 0.)),
        FeedbackUI {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
            should_start_new_game: result,
        },
    ));
}

fn update_feedback_ui_timer(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FeedbackUI)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut feedback) in &mut query {
        feedback.timer.tick(time.delta());
        if feedback.timer.finished() {
            if feedback.should_start_new_game {
                next_state.set(GameState::Result);
            }
            commands.entity(entity).despawn();
        }
    }
}
