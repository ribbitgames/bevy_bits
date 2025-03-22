use std::time::Duration;

use avian2d::collision::contact_reporting::CollisionStarted; // Corrected import path
use avian2d::prelude::{
    AngularDamping, Collider, Friction, GravityScale, LinearDamping, LinearVelocity, LockedAxes,
    Mass, Restitution, RigidBody,
};
use bevy::prelude::*;
use bits_helpers::floating_score::spawn_floating_score;
use bits_helpers::input::pressed_world_position;
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};

use crate::core::{Bucket, GameState, GameTimer, Marble, Platform, Score, SpawnTimer, config};

// Component for rendering circles
#[derive(Component)]
pub struct Circle {
    pub radius: f32,
    pub color: Color,
}

#[derive(Component)]
pub struct ScoreDisplay;

#[derive(Component)]
pub struct TimerDisplay;

#[derive(Component)]
pub struct Resting(pub Timer); // Tracks if a marble is resting on a platform

pub fn spawn_game_elements(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a single platform
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(config::PLATFORM_SIZE),
            ..default()
        },
        Transform::from_xyz(
            0.0,
            -WINDOW_HEIGHT / 2.0 + config::BUCKET_SIZE.y + 30.0,
            0.0,
        ),
        Platform {
            width: config::PLATFORM_SIZE.x,
        },
        RigidBody::Static,
        Collider::rectangle(config::PLATFORM_SIZE.x, config::PLATFORM_SIZE.y),
        Friction::new(0.0), // No friction to prevent velocity loss
        Restitution::new(0.5),
        LockedAxes::new().lock_rotation(),
    ));

    // Spawn buckets
    let colors = [
        Color::srgb(1.0, 0.0, 0.0), // Red
        Color::srgb(0.0, 1.0, 0.0), // Green
        Color::srgb(0.0, 0.0, 1.0), // Blue
    ];
    for (i, &color) in colors.iter().enumerate() {
        let x = (i as f32).mul_add(120.0, -WINDOW_WIDTH / 2.0 + 60.0);
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(config::BUCKET_SIZE),
                ..default()
            },
            Transform::from_xyz(x, -WINDOW_HEIGHT / 2.0 + config::BUCKET_SIZE.y / 2.0, 0.0),
            Bucket {
                color,
                width: config::BUCKET_SIZE.x,
            },
            RigidBody::Static,
            Collider::rectangle(config::BUCKET_SIZE.x, config::BUCKET_SIZE.y),
            Friction::new(0.1),
        ));
    }

    // Spawn score display
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::default(),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + 60.0, WINDOW_HEIGHT / 2.0 - 60.0, 1.0),
        ScoreDisplay,
    ));

    // Spawn timer display
    commands.spawn((
        Text2d::new("Time: 30"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - 60.0, WINDOW_HEIGHT / 2.0 - 60.0, 1.0),
        TimerDisplay,
    ));

    // Initialize resources
    commands.insert_resource(SpawnTimer::default());
    commands.insert_resource(GameTimer::default());
}

pub fn move_platforms(
    mut platform_query: Query<(&mut Transform, &Platform), With<Platform>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    if let Some(world_position) =
        pressed_world_position(&mouse_input, &touch_input, &windows, &camera)
    {
        if let Ok((mut transform, platform)) = platform_query.get_single_mut() {
            let platform_radius = platform.width / 2.0;
            let new_x = world_position.x.clamp(
                -WINDOW_WIDTH / 2.0 + platform_radius,
                WINDOW_WIDTH / 2.0 - platform_radius,
            );
            transform.translation.x = new_x;
        }
    }
}

pub fn update_game(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
    score: ResMut<Score>,
    mut score_display: Query<&mut Text2d, With<ScoreDisplay>>,
    asset_server: Res<AssetServer>,
    mut marble_query: Query<(Entity, &Transform, &Circle, &mut Resting, &Marble), With<Marble>>,
    game_timer: Res<GameTimer>,
) {
    spawn_timer.timer.tick(time.delta());

    // Adjust spawn rate based on game progression
    let total_duration = game_timer.timer.duration().as_secs_f32();
    let elapsed = total_duration - game_timer.timer.remaining_secs();
    let chunk_duration = total_duration / config::TIME_CHUNKS as f32;
    let current_chunk = (elapsed / chunk_duration).floor() as usize;

    let new_spawn_rate = (current_chunk as f32).mul_add(-0.5, 1.5).max(0.5);
    const EPSILON: f32 = 0.001;
    if (new_spawn_rate - spawn_timer.spawn_rate).abs() > EPSILON {
        spawn_timer.spawn_rate = new_spawn_rate;
        spawn_timer
            .timer
            .set_duration(Duration::from_secs_f32(new_spawn_rate));
        spawn_timer.timer.reset();
    }

    // Spawn marbles
    if spawn_timer.timer.just_finished() {
        let is_colored = fastrand::f32() < 0.75;
        let color: Color;
        let target_bucket_x: f32;

        if is_colored {
            let random_color = fastrand::u32(0..3);
            if random_color == 0 {
                color = Color::srgb(1.0, 0.0, 0.0); // Red
                target_bucket_x = -WINDOW_WIDTH / 2.0 + 60.0; // Red bucket position
            } else if random_color == 1 {
                color = Color::srgb(0.0, 1.0, 0.0); // Green
                target_bucket_x = -WINDOW_WIDTH / 2.0 + 180.0; // Green bucket position
            } else {
                color = Color::srgb(0.0, 0.0, 1.0); // Blue
                target_bucket_x = -WINDOW_WIDTH / 2.0 + 300.0; // Blue bucket position
            }
        } else {
            color = Color::srgb(0.5, 0.5, 0.5); // Grey
            let random_bucket = fastrand::u32(0..3);
            target_bucket_x = (random_bucket as f32).mul_add(120.0, -WINDOW_WIDTH / 2.0 + 60.0);
        }

        let spawn_x =
            fastrand::f32().mul_add(WINDOW_WIDTH - config::MARBLE_SIZE, -(WINDOW_WIDTH / 2.0));
        let clamped_spawn_x = spawn_x.clamp(
            -WINDOW_WIDTH / 2.0 + config::MARBLE_SIZE / 2.0,
            WINDOW_WIDTH / 2.0 - config::MARBLE_SIZE / 2.0,
        );

        // Calculate direction towards target bucket
        let start_position = Vec2::new(clamped_spawn_x, WINDOW_HEIGHT / 2.0);
        let target_position = Vec2::new(
            target_bucket_x,
            -WINDOW_HEIGHT / 2.0 + config::BUCKET_SIZE.y / 2.0,
        );
        let direction_vector = target_position - start_position;
        let normalized_direction = direction_vector.normalize();

        // Set initial velocity towards bucket with some randomness
        let base_speed = fastrand::f32().mul_add(200.0, 300.0); // Speed between 300 and 500
        let velocity_x = normalized_direction.x * base_speed;
        let velocity_y = normalized_direction.y * base_speed;

        commands.spawn((
            Transform::from_xyz(clamped_spawn_x, WINDOW_HEIGHT / 2.0, 0.0),
            Marble {
                size: config::MARBLE_SIZE,
                is_target: is_colored,
            },
            Circle {
                radius: config::MARBLE_SIZE / 2.0,
                color,
            },
            RigidBody::Dynamic,
            Collider::circle(config::MARBLE_SIZE / 2.0),
            Restitution::new(0.5),
            Friction::new(0.0),
            LinearDamping(0.0),
            AngularDamping(0.0),
            GravityScale(1.0),
            LinearVelocity(Vec2::new(velocity_x, velocity_y)),
            Mass(1.0),
            Resting(Timer::new(Duration::from_secs(1), TimerMode::Once)),
        ));
        spawn_timer.timer.reset();
    }

    // Update score display
    if let Ok(mut score_text) = score_display.get_single_mut() {
        *score_text = Text2d::new(format!("Score: {}", score.0));
    }

    // Handle marbles that fall off-screen
    for (marble_entity, transform, _circle, mut resting, marble) in &mut marble_query {
        resting.0.tick(time.delta());
        let marble_pos = transform.translation.truncate();

        if marble_pos.y < -WINDOW_HEIGHT / 2.0 - marble.size && resting.0.finished() {
            commands.entity(marble_entity).despawn();
            if marble.is_target {
                spawn_floating_score(
                    &mut commands,
                    Vec2::new(marble_pos.x, -WINDOW_HEIGHT / 2.0),
                    "MISSED!",
                    Color::srgb(1.0, 0.0, 0.0).into(),
                    &asset_server,
                );
            }
        }
    }
}

pub fn handle_marble_bucket_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    marble_query: Query<(&Circle, &Marble)>,
    bucket_query: Query<&Bucket>,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
    transform_query: Query<&Transform>,
) {
    for CollisionStarted(entity1, entity2) in collision_events.read() {
        let (marble_entity, bucket_entity) = if marble_query.get(*entity1).is_ok() {
            (*entity1, *entity2)
        } else if marble_query.get(*entity2).is_ok() {
            (*entity2, *entity1)
        } else {
            continue; // Neither entity is a marble
        };

        if let (Ok((circle, marble)), Ok(bucket)) = (
            marble_query.get(marble_entity),
            bucket_query.get(bucket_entity),
        ) {
            let marble_pos = transform_query
                .get(marble_entity)
                .map_or(Vec2::ZERO, |t| t.translation.truncate());

            commands.entity(marble_entity).despawn();

            if marble.is_target && circle.color == bucket.color {
                score.0 += 5;
                spawn_floating_score(
                    &mut commands,
                    marble_pos,
                    "+5",
                    Color::srgb(0.0, 1.0, 0.0).into(),
                    &asset_server,
                );
            } else {
                score.0 -= 2;
                spawn_floating_score(
                    &mut commands,
                    marble_pos,
                    "-2",
                    Color::srgb(1.0, 0.0, 0.0).into(),
                    &asset_server,
                );
            }
        }
    }
}

pub fn update_game_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    score: Res<Score>,
    mut timer_display: Query<&mut Text2d, With<TimerDisplay>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    game_timer.timer.tick(time.delta());

    if let Ok(mut timer_text) = timer_display.get_single_mut() {
        let remaining = game_timer.timer.remaining_secs().ceil() as i32;
        *timer_text = Text2d::new(format!("Time: {remaining}"));
    }

    if game_timer.timer.just_finished() {
        next_state.set(GameState::GameOver);
        bits_helpers::send_bit_message(ribbit_bits::BitMessage::End(
            ribbit_bits::BitResult::HighestScore(score.0.into()),
        ));
    }
}

pub fn render_circles(
    query: Query<(&Transform, &Circle), (With<Circle>, Without<Platform>, Without<Bucket>)>,
    mut gizmos: Gizmos,
) {
    for (transform, circle) in query.iter() {
        gizmos.circle_2d(
            transform.translation.truncate(),
            circle.radius,
            circle.color,
        );
    }
}

pub fn cleanup_game(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<Marble>,
            With<Platform>,
            With<Bucket>,
            With<ScoreDisplay>,
            With<TimerDisplay>,
        )>,
    >,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
