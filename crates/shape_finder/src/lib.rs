use core::time::Duration;

use bevy::color::palettes::css::{BLUE, GREEN, RED, WHITE, YELLOW};
use bevy::prelude::*;
use bits_helpers::floating_score::{animate_floating_scores, spawn_floating_score};
use bits_helpers::welcome_screen::{despawn_welcome_screen, spawn_welcome_screen_shape};
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};
use rand::prelude::IteratorRandom;
use rand::seq::SliceRandom;
use rand::Rng;
use ribbit::ShapeFinder;

mod ribbit;

const SHAPE_COUNT: usize = 50;
const GAME_DURATION: f32 = 20.0;
const SHAPE_SPEED: f32 = 100.0;
const CORRECT_SHAPE_COUNT: usize = 5;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum ShapeType {
    #[default]
    Circle,
    Square,
    Triangle,
    Hexagon,
}

const SHAPES: [ShapeType; 4] = [
    ShapeType::Circle,
    ShapeType::Square,
    ShapeType::Triangle,
    ShapeType::Hexagon,
];

const COLORS: [Color; 4] = [
    Color::Srgba(RED),
    Color::Srgba(BLUE),
    Color::Srgba(GREEN),
    Color::Srgba(YELLOW),
];

#[derive(Component)]
struct Shape {
    type_: ShapeType,
    color: Color,
    size: f32,
}

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Resource)]
struct GameTimer(Timer);

#[derive(Resource, Default)]
struct Score(i32);

#[derive(Resource)]
struct TargetShapeInfo {
    type_: ShapeType,
    color: Color,
}

#[derive(Event)]
struct ShapeClickedEvent {
    entity: Entity,
    position: Vec2,
    is_correct: bool,
}

pub fn run() {
    let mut app = bits_helpers::get_default_app::<ShapeFinder>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    app.init_state::<GameState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        .init_resource::<TargetShapeInfo>()
        .add_event::<ShapeClickedEvent>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
        .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
        .add_systems(OnEnter(GameState::Playing), (spawn_shapes, spawn_timer))
        .add_systems(Update, handle_shape_clicked.after(handle_playing_input))
        .add_systems(
            Update,
            (
                handle_welcome_input.run_if(in_state(GameState::Welcome)),
                (
                    handle_playing_input,
                    move_shapes,
                    update_timer,
                    animate_floating_scores,
                )
                    .run_if(in_state(GameState::Playing)),
            ),
        )
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_welcome_screen(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut target_info: ResMut<TargetShapeInfo>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();

    target_info.type_ = *SHAPES.choose(&mut rng).unwrap_or(&ShapeType::Circle);
    target_info.color = *COLORS.choose(&mut rng).unwrap_or(&Color::Srgba(RED));

    let target_mesh = match target_info.type_ {
        ShapeType::Square => Mesh::from(bevy::math::primitives::Rectangle::new(40.0, 40.0)),
        ShapeType::Triangle => Mesh::from(bevy::math::primitives::RegularPolygon::new(20.0, 3)),
        ShapeType::Hexagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(20.0, 6)),
        ShapeType::Circle => Mesh::from(bevy::math::primitives::Circle::new(20.0)),
    };

    spawn_welcome_screen_shape(
        commands,
        asset_server,
        meshes,
        materials,
        "Click ONLY on this shape!",
        target_mesh,
        target_info.color,
    );
}

fn handle_welcome_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_timer: ResMut<GameTimer>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        game_timer.0.reset();
        next_state.set(GameState::Playing);
    }
}

fn spawn_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    target_info: Res<TargetShapeInfo>,
) {
    let mut rng = rand::thread_rng();
    let mut shapes = Vec::new();

    // Spawn correct shapes
    for _ in 0..CORRECT_SHAPE_COUNT {
        shapes.push((target_info.type_, target_info.color));
    }

    // Spawn other shapes
    for _ in CORRECT_SHAPE_COUNT..SHAPE_COUNT {
        let shape_type = *SHAPES.choose(&mut rng).unwrap_or(&ShapeType::Circle);
        let color = *COLORS.choose(&mut rng).unwrap_or(&Color::Srgba(RED));
        if shape_type != target_info.type_ || color != target_info.color {
            shapes.push((shape_type, color));
        } else {
            // If we accidentally generated a correct shape, change the color
            let different_color = COLORS
                .iter()
                .filter(|&&c| c != target_info.color)
                .choose(&mut rng)
                .copied()
                .unwrap_or(Color::Srgba(RED));
            shapes.push((shape_type, different_color));
        }
    }

    shapes.shuffle(&mut rng);

    for (shape_type, color) in shapes {
        let size = rng.gen_range(20.0..40.0);
        let x = rng.gen_range(-WINDOW_WIDTH / 2.0 + size..WINDOW_WIDTH / 2.0 - size);
        let y = rng.gen_range(-WINDOW_HEIGHT / 2.0 + size..WINDOW_HEIGHT / 2.0 - size);

        let mesh = match shape_type {
            ShapeType::Circle => Mesh::from(bevy::math::primitives::Circle::new(size / 2.0)),
            ShapeType::Square => Mesh::from(bevy::math::primitives::Rectangle::new(size, size)),
            ShapeType::Triangle => {
                Mesh::from(bevy::math::primitives::RegularPolygon::new(size / 2.0, 3))
            }
            ShapeType::Hexagon => {
                Mesh::from(bevy::math::primitives::RegularPolygon::new(size / 2.0, 6))
            }
        };

        let velocity =
            Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize() * SHAPE_SPEED;

        commands.spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Shape {
                type_: shape_type,
                color,
                size,
            },
            Velocity(velocity),
        ));
    }
}

fn spawn_timer(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new("Time: {GAME_DURATION:.1}"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(2.0),
            ..default()
        },
    ));
}

fn update_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut timer_text: Query<&mut Text>,
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

fn handle_playing_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    shapes: Query<(Entity, &Transform, &Shape)>,
    target_info: Res<TargetShapeInfo>,
    mut shape_clicked_events: EventWriter<ShapeClickedEvent>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera_q.single();
        let window = windows.single();

        let Some(cursor_position) = window.cursor_position() else {
            return;
        };

        if let Some(world_position) = camera
            .viewport_to_world(camera_transform, cursor_position)
            .ok()
            .map(|ray| ray.origin.truncate())
        {
            for (entity, transform, shape) in shapes.iter() {
                if transform.translation.truncate().distance(world_position) < shape.size / 2.0 {
                    let is_correct =
                        shape.type_ == target_info.type_ && shape.color == target_info.color;
                    shape_clicked_events.send(ShapeClickedEvent {
                        entity,
                        position: cursor_position,
                        is_correct,
                    });
                    break;
                }
            }
        }
    }
}

fn handle_shape_clicked(
    mut commands: Commands,
    mut shape_clicked_events: EventReader<ShapeClickedEvent>,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
) {
    for event in shape_clicked_events.read() {
        if event.is_correct {
            score.0 += 1;
            spawn_floating_score(&mut commands, event.position, "+1", GREEN, &asset_server);
        } else {
            score.0 -= 1;
            spawn_floating_score(&mut commands, event.position, "-1", RED, &asset_server);
        }
        commands.entity(event.entity).despawn();
    }
}

fn move_shapes(mut query: Query<(&mut Transform, &mut Velocity, &Shape)>, time: Res<Time>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some(
        [(mut transform1, mut velocity1, shape1), (mut transform2, mut velocity2, shape2)],
    ) = combinations.fetch_next()
    {
        let pos1 = transform1.translation.truncate();
        let pos2 = transform2.translation.truncate();
        let distance = pos1.distance(pos2);
        let min_distance = (shape1.size + shape2.size) / 2.0;

        if distance < min_distance {
            // Calculate collision response
            let normal = (pos2 - pos1).normalize();
            let relative_velocity = velocity2.0 - velocity1.0;
            let impulse = 2.0 * relative_velocity.dot(normal) / 2.0; // Assuming equal mass

            velocity1.0 += normal * impulse;
            velocity2.0 -= normal * impulse;

            // Separate the shapes
            let separation = (min_distance - distance) / 2.0;
            transform1.translation -= (normal * separation).extend(0.0);
            transform2.translation += (normal * separation).extend(0.0);
        }
    }

    // Move shapes and handle wall collisions
    for (mut transform, mut velocity, shape) in &mut query {
        let mut new_pos = transform.translation + velocity.0.extend(0.0) * time.delta_secs();

        if new_pos.x - shape.size / 2.0 < -WINDOW_WIDTH / 2.0
            || new_pos.x + shape.size / 2.0 > WINDOW_WIDTH / 2.0
        {
            new_pos.x = new_pos.x.clamp(
                -WINDOW_WIDTH / 2.0 + shape.size / 2.0,
                WINDOW_WIDTH / 2.0 - shape.size / 2.0,
            );
            velocity.0.x *= -1.0;
        }
        if new_pos.y - shape.size / 2.0 < -WINDOW_HEIGHT / 2.0
            || new_pos.y + shape.size / 2.0 > WINDOW_HEIGHT / 2.0
        {
            new_pos.y = new_pos.y.clamp(
                -WINDOW_HEIGHT / 2.0 + shape.size / 2.0,
                WINDOW_HEIGHT / 2.0 - shape.size / 2.0,
            );
            velocity.0.y *= -1.0;
        }

        transform.translation = new_pos;
    }
}

fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new(format!("Game Over!\nYour score: {}", score.0)),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

impl Default for GameTimer {
    fn default() -> Self {
        Self(Timer::new(
            Duration::from_secs_f32(GAME_DURATION),
            TimerMode::Once,
        ))
    }
}

impl Default for TargetShapeInfo {
    fn default() -> Self {
        Self {
            type_: ShapeType::Circle,
            color: Color::Srgba(WHITE),
        }
    }
}
