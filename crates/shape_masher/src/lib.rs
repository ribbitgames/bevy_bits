use std::time::Duration;

use bevy::color::palettes::css::{
    BLUE, CORAL, GREEN, LIME, MAGENTA, ORANGE, PINK, RED, TEAL, WHITE, YELLOW,
};
use bevy::prelude::*;
use bits_helpers::floating_score::{animate_floating_scores, spawn_floating_score};
use bits_helpers::input::{just_pressed_screen_position, just_pressed_world_position};
use bits_helpers::welcome_screen::{despawn_welcome_screen, spawn_welcome_screen_shape};
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};
use rand::seq::SliceRandom;
use rand::Rng;
use ribbit::ShapeMasher;

mod ribbit;

const SHAPE_SIZE: f32 = 60.0;
const GAME_DURATION: f32 = 20.0;
const NUM_BAD_SHAPES: usize = 10; // Adjust this number as needed
const MIN_DISTANCE_BETWEEN_SHAPES: f32 = SHAPE_SIZE * 1.3;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

#[derive(Component)]
struct ClickableShape;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct ClickCounterText;

#[derive(Resource, Default)]
struct GameTimer(f32);

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Component)]
struct PulseAnimation {
    timer: Timer,
    scale: f32,
    growing: bool,
}

#[derive(Component)]
struct ClickFeedback {
    timer: Timer,
    original_scale: f32,
}

#[derive(Component)]
struct SpawnAnimation {
    timer: Timer,
}

#[derive(Component, Clone, Copy, PartialEq)]
enum Shape {
    Circle,
    Square,
    Triangle,
    Hexagon,
    Pentagon,
    Octagon,
    Quad,
}

const SHAPES: [Shape; 7] = [
    Shape::Circle,
    Shape::Square,
    Shape::Triangle,
    Shape::Hexagon,
    Shape::Pentagon,
    Shape::Octagon,
    Shape::Quad,
];

const COLORS: [Color; 10] = [
    Color::Srgba(RED),
    Color::Srgba(BLUE),
    Color::Srgba(GREEN),
    Color::Srgba(YELLOW),
    Color::Srgba(ORANGE),
    Color::Srgba(LIME),
    Color::Srgba(MAGENTA),
    Color::Srgba(TEAL),
    Color::Srgba(PINK),
    Color::Srgba(CORAL),
];

#[derive(Resource)]
struct TargetShape {
    shape: Shape,
    color: Color,
}

#[derive(Resource)]
struct ShapePool {
    shapes: Vec<(Shape, Color)>,
}

pub fn run() {
    bits_helpers::get_default_app::<ShapeMasher>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .init_state::<GameState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
        .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
        .add_systems(OnEnter(GameState::Playing), spawn_game_elements)
        .add_systems(
            Update,
            (handle_welcome_input.run_if(in_state(GameState::Welcome)),),
        )
        .add_systems(
            Update,
            (
                handle_game_input,
                update_timer,
                animate_shape,
                animate_click_feedback,
                animate_spawn,
                animate_floating_scores,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let mut rng = rand::thread_rng();
    let shapes: Vec<(Shape, Color)> = (0..NUM_BAD_SHAPES)
        .map(|_| {
            let shape = *SHAPES
                .choose(&mut rng)
                .expect("SHAPES array should not be empty");
            let color = *COLORS
                .choose(&mut rng)
                .expect("COLORS array should not be empty");
            (shape, color)
        })
        .collect();

    commands.insert_resource(ShapePool { shapes });
}

fn spawn_welcome_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    let target_shape = *SHAPES
        .choose(&mut rng)
        .expect("SHAPES array should not be empty");
    let target_color = *COLORS
        .choose(&mut rng)
        .expect("COLORS array should not be empty");

    // Store the target shape and color as a resource
    commands.insert_resource(TargetShape {
        shape: target_shape,
        color: target_color,
    });

    let mesh = create_shape_mesh(target_shape, SHAPE_SIZE);

    spawn_welcome_screen_shape(
        commands,
        asset_server,
        meshes,
        materials,
        "Mash this shape",
        mesh,
        target_color,
    );
}

fn spawn_game_elements(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut game_timer: ResMut<GameTimer>,
    mut score: ResMut<Score>,
    target_shape: Res<TargetShape>,
    shape_pool: Res<ShapePool>,
) {
    game_timer.0 = GAME_DURATION;
    score.0 = 0;

    spawn_all_shapes(
        &mut commands,
        &mut meshes,
        &mut materials,
        &target_shape,
        &shape_pool,
    );

    commands.spawn((
        Text::new(format!("Time: {GAME_DURATION:.1}")),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::Srgba(WHITE)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TimerText,
    ));

    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::Srgba(WHITE)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        ClickCounterText,
    ));
}

fn spawn_all_shapes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    target_shape: &Res<TargetShape>,
    shape_pool: &Res<ShapePool>,
) {
    let mut occupied_positions = Vec::new();

    // Spawn target shape
    let target_pos = spawn_shape_at_random_position(
        commands,
        meshes,
        materials,
        target_shape.shape,
        target_shape.color,
        &occupied_positions,
    );
    occupied_positions.push(target_pos);

    // Spawn bad shapes
    for (shape, color) in &shape_pool.shapes {
        let pos = spawn_shape_at_random_position(
            commands,
            meshes,
            materials,
            *shape,
            *color,
            &occupied_positions,
        );
        occupied_positions.push(pos);
    }
}

fn spawn_shape_at_random_position(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    shape: Shape,
    color: Color,
    occupied_positions: &[Vec2],
) -> Vec2 {
    let mut rng = rand::thread_rng();
    let mut position;
    let mut attempts = 0;

    loop {
        position = Vec2::new(
            rng.gen_range(-WINDOW_WIDTH / 2.0 + SHAPE_SIZE..WINDOW_WIDTH / 2.0 - SHAPE_SIZE),
            rng.gen_range(-WINDOW_HEIGHT / 2.0 + SHAPE_SIZE..WINDOW_HEIGHT / 2.0 - SHAPE_SIZE),
        );

        if occupied_positions
            .iter()
            .all(|&pos| pos.distance(position) >= MIN_DISTANCE_BETWEEN_SHAPES)
        {
            break;
        }

        attempts += 1;
        if attempts > 100 {
            // If we can't find a non-overlapping position after 100 attempts, just use the last generated position
            break;
        }
    }

    let mesh = create_shape_mesh(shape, SHAPE_SIZE);

    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(color))),
        Transform::from_translation(position.extend(0.0)).with_scale(Vec3::ZERO),
        ClickableShape,
        shape,
        PulseAnimation {
            timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
            scale: 1.0,
            growing: true,
        },
        SpawnAnimation {
            timer: Timer::new(Duration::from_millis(200), TimerMode::Once),
        },
    ));

    position
}

fn create_shape_mesh(shape_type: Shape, cell_size: f32) -> Mesh {
    let shape_size = cell_size * 0.9; // Adjust this factor as needed

    match shape_type {
        Shape::Circle => Mesh::from(bevy::math::primitives::Circle::new(shape_size / 2.0)),
        Shape::Square => Mesh::from(bevy::math::primitives::Rectangle::new(
            shape_size, shape_size,
        )),
        Shape::Triangle => Mesh::from(bevy::math::primitives::RegularPolygon::new(
            shape_size / 2.0,
            3,
        )),
        Shape::Hexagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(
            shape_size / 2.0,
            6,
        )),
        Shape::Pentagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(
            shape_size / 2.0,
            5,
        )),
        Shape::Octagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(
            shape_size / 2.0,
            8,
        )),
        Shape::Quad => Mesh::from(bevy::math::primitives::Rectangle::new(
            shape_size,
            shape_size * 0.75,
        )),
    }
}

fn handle_welcome_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

fn handle_game_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    clickable_shapes: Query<(Entity, &Transform, &Shape), With<ClickableShape>>,
    mut score: ResMut<Score>,
    mut click_counter_query: Query<&mut Text, With<ClickCounterText>>,
    asset_server: Res<AssetServer>,
    target_shape: Res<TargetShape>,
    shape_pool: Res<ShapePool>,
) {
    let Some(screen_position) =
        just_pressed_screen_position(&mouse_button_input, &touch_input, &windows)
    else {
        return;
    };

    let Some(world_position) =
        just_pressed_world_position(&mouse_button_input, &touch_input, &windows, &camera)
    else {
        return;
    };

    let mut clicked_shape = None;
    for (entity, transform, shape) in clickable_shapes.iter() {
        if is_point_in_shape(
            world_position,
            transform.translation.truncate(),
            SHAPE_SIZE / 2.0,
        ) {
            clicked_shape = Some((entity, *shape));
            break;
        }
    }

    if let Some((_, shape)) = clicked_shape {
        // Despawn all shapes
        for (entity, _, _) in clickable_shapes.iter() {
            commands.entity(entity).despawn();
        }

        // Respawn all shapes
        spawn_all_shapes(
            &mut commands,
            &mut meshes,
            &mut materials,
            &target_shape,
            &shape_pool,
        );

        if shape == target_shape.shape {
            // Clicked the correct shape
            score.0 += 1;
            spawn_floating_score(&mut commands, screen_position, "+1", GREEN, &asset_server);
        } else {
            // Clicked the wrong shape
            score.0 = score.0.saturating_sub(1);
            spawn_floating_score(&mut commands, screen_position, "-1", RED, &asset_server);
        }
    } else {
        // Clicked outside any shape
        score.0 = score.0.saturating_sub(1);
        spawn_floating_score(&mut commands, screen_position, "-1", RED, &asset_server);
    }

    // Update the click counter text
    if let Ok(mut text) = click_counter_query.get_single_mut() {
        text.0 = format!("Score: {}", score.0);
    }
}

fn update_timer(
    mut game_timer: ResMut<GameTimer>,
    mut query: Query<&mut Text, With<TimerText>>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    game_timer.0 -= time.delta_secs();
    game_timer.0 = game_timer.0.max(0.0);

    if let Ok(mut text) = query.get_single_mut() {
        text.0 = format!("Time: {:.1}", game_timer.0);
    }

    if game_timer.0 <= 0.0 {
        next_state.set(GameState::GameOver);
    }
}

fn animate_shape(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PulseAnimation), With<ClickableShape>>,
) {
    for (mut transform, mut animation) in &mut query {
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if animation.growing {
                animation.scale += 0.05;
                if animation.scale >= 1.1 {
                    animation.growing = false;
                }
            } else {
                animation.scale -= 0.05;
                if animation.scale <= 0.9 {
                    animation.growing = true;
                }
            }
        }

        transform.scale = Vec3::splat(animation.scale);
    }
}

fn animate_click_feedback(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ClickFeedback)>,
) {
    for (entity, mut transform, mut feedback) in &mut query {
        feedback.timer.tick(time.delta());
        let progress = feedback.timer.fraction();
        let scale = feedback.original_scale * 0.2f32.mul_add(1.0 - progress, 1.0);
        transform.scale = Vec3::splat(scale);

        if feedback.timer.finished() {
            commands.entity(entity).remove::<ClickFeedback>();
        }
    }
}

fn animate_spawn(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut SpawnAnimation)>,
) {
    for (entity, mut transform, mut spawn_anim) in &mut query {
        spawn_anim.timer.tick(time.delta());
        let progress = spawn_anim.timer.fraction();
        transform.scale = Vec3::splat(progress);

        if spawn_anim.timer.finished() {
            commands.entity(entity).remove::<SpawnAnimation>();
        }
    }
}

fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::from("Game Over!"),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::Srgba(WHITE)),
            ));
            parent.spawn((
                Text::new(format!("Score: {}", score.0)),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::Srgba(WHITE)),
            ));
        });
}

fn is_point_in_shape(point: Vec2, shape_center: Vec2, size: f32) -> bool {
    let expanded_size = size * 1.4; // Increase the size
    let half_size = expanded_size / 2.0;
    let min_x = shape_center.x - half_size;
    let max_x = shape_center.x + half_size;
    let min_y = shape_center.y - half_size;
    let max_y = shape_center.y + half_size;

    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}
