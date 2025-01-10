use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy::window::PrimaryWindow;
use rand::prelude::SliceRandom;
use rand::Rng;
use ribbit::WheresWaldo;

mod ribbit;

const SPRITE_IMAGE: &str = "animals.png";

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

const ATLAS_COLUMNS: u32 = 10;
const ATLAS_ROWS: u32 = 8;

const UI_Y: f32 = -BACKGROUND_SIZE_Y * 0.5 - 32.;
const UI_RESULT_Y: f32 = UI_Y - 32.;

const COLLISION_RADIUS: f32 = 24.0;

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
struct CreatePuzzle;

#[derive(Event)]
struct ClearPuzzle;

#[derive(Event)]
struct ShowFeedback {
    is_correct: bool,
}

#[derive(Event)]
struct InquireEvent {
    pos: Vec2,
}

#[derive(Resource)]
struct Grid {
    grid: Vec<Vec2>,
}

#[derive(Resource)]
struct TextureInfo {
    texture_handle: Handle<Image>,
    atlas_handle: Handle<TextureAtlasLayout>,
}

pub fn run() {
    bits_helpers::get_default_app::<WheresWaldo>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .add_event::<CreatePuzzle>()
        .add_event::<ClearPuzzle>()
        .add_event::<InquireEvent>()
        .add_event::<ShowFeedback>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                mouse_events,
                create_puzzle,
                clear_puzzle,
                inquire_position,
                spawn_feedback_ui,
                update_feedback_ui_timer,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut create_events: EventWriter<CreatePuzzle>,
) {
    commands.spawn(Camera2d).insert(MainCamera);

    let texture: Handle<Image> = asset_server.load(SPRITE_IMAGE);
    let texture_atlas = TextureAtlasLayout::from_grid(
        UVec2 {
            x: SPRITE_SIZE_X as u32,
            y: SPRITE_SIZE_Y as u32,
        },
        ATLAS_COLUMNS,
        ATLAS_ROWS,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(TextureInfo {
        texture_handle: texture,
        atlas_handle: texture_atlas_handle,
    });

    // Background
    commands.spawn((
        Sprite::from_color(
            BACKGROUND_COLOR,
            Vec2::new(BACKGROUND_SIZE_X, BACKGROUND_SIZE_Y),
        ),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
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

    // Start the game by sending this event
    create_events.send(CreatePuzzle);
}

fn mouse_events(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_event: EventReader<MouseButtonInput>,
    mut inquire_event: EventWriter<InquireEvent>,
) {
    for event in mouse_event.read() {
        if let MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            ..
        } = event
        {
            let window = window_query.single();
            let (camera, camera_transform) = camera_query.single();
            if let Some(world_cursor_position) =
                window.cursor_position().and_then(|viewport_position| {
                    camera
                        .viewport_to_world_2d(camera_transform, viewport_position)
                        .ok()
                })
            {
                inquire_event.send(InquireEvent {
                    pos: world_cursor_position,
                });
            }
        }
    }
}

fn get_random_transform(grid_position: Vec2) -> Transform {
    let mut rng = rand::thread_rng();
    let position_noize: f32 = 0.3;
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
fn spawn_sprite(
    commands: &mut Commands,
    texture_handle: Handle<Image>,
    atlas_handle: Handle<TextureAtlasLayout>,
    index: usize,
    transform: Transform,
) -> Entity {
    let entity_id = commands
        .spawn((
            Sprite::from_atlas_image(
                texture_handle,
                TextureAtlas {
                    layout: atlas_handle,
                    index,
                },
            ),
            transform,
            Character,
        ))
        .id();

    entity_id
}

fn create_puzzle(
    mut commands: Commands,
    mut grid: ResMut<Grid>,
    texture_info: Res<TextureInfo>,
    mut create_events: EventReader<CreatePuzzle>,
) {
    for _ev in create_events.read() {
        let mut rng = rand::thread_rng();
        let length = (ATLAS_COLUMNS * ATLAS_ROWS) as usize;

        // The one to look for
        let index_waldo = rng.gen_range(0..length);

        grid.grid.shuffle(&mut rng);

        for (count, pos) in grid.grid.clone().into_iter().enumerate() {
            if count >= (NUMBER_OF_CANDIDATES as usize) {
                break;
            }
            //println!("pos[{count}] = {pos}");

            let index = if count == 0 {
                index_waldo
            } else {
                let temp = rng.gen_range(0..length - 1);
                if temp == index_waldo {
                    length - 1
                } else {
                    temp
                }
            };
            let id = spawn_sprite(
                &mut commands,
                texture_info.texture_handle.clone(),
                texture_info.atlas_handle.clone(),
                index,
                get_random_transform(pos),
            );
            if count == 0 {
                commands.entity(id).insert(Waldo);
            }
        }

        // The one to look for (for UI)
        let id = spawn_sprite(
            &mut commands,
            texture_info.texture_handle.clone(),
            texture_info.atlas_handle.clone(),
            index_waldo,
            Transform::from_translation(Vec3::new(40., UI_Y, 0.)),
        );
        commands.entity(id).insert(Waldo);
    }
}

fn clear_puzzle(
    mut commands: Commands,
    mut clear_events: EventReader<ClearPuzzle>,
    mut create_events: EventWriter<CreatePuzzle>,
    query: Query<(Entity, &Character)>,
) {
    for _ev in clear_events.read() {
        for (entity, _character) in &query {
            commands.entity(entity).despawn();
        }
        // Start a new game
        create_events.send(CreatePuzzle);
    }
}

fn inquire_position(
    mut inquire_event: EventReader<InquireEvent>,
    mut spawn_feedback_event: EventWriter<ShowFeedback>,
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
                spawn_feedback_event.send(ShowFeedback { is_correct: true });
                return;
            }
        }

        for (transform, _character) in &character_query {
            let dist = ev.pos.distance_squared(transform.translation.truncate());
            if dist < squared_radius {
                spawn_feedback_event.send(ShowFeedback { is_correct: false });
                return;
            }
        }
    }
}

fn spawn_feedback_ui(mut commands: Commands, mut spawn_events: EventReader<ShowFeedback>) {
    for ev in spawn_events.read() {
        commands.spawn((
            Text2d::new(if ev.is_correct {
                "Good job!"
            } else {
                "It's not me!"
            }),
            TextFont {
                font_size: 24.,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(0., UI_RESULT_Y, 0.)),
            FeedbackUI {
                timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
                should_start_new_game: ev.is_correct,
            },
        ));
    }
}

fn update_feedback_ui_timer(
    mut commands: Commands,
    mut clear_events: EventWriter<ClearPuzzle>,
    mut query: Query<(Entity, &mut FeedbackUI)>,
    time: Res<Time>,
) {
    for (entity, mut feedback) in &mut query {
        feedback.timer.tick(time.delta());
        if feedback.timer.finished() {
            if feedback.should_start_new_game {
                clear_events.send(ClearPuzzle);
            }
            commands.entity(entity).despawn();
        }
    }
}
