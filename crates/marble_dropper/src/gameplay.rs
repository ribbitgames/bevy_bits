use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use bits_helpers::floating_score::spawn_floating_score;
use bits_helpers::input::pressed_world_position;
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH, send_bit_message};
use ribbit_bits::{BitMessage, BitResult};

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
    // in case we want to restore 3 paddles //
    //let platform_spacing = 150.0;
    //let base_y = -WINDOW_HEIGHT / 2.0 + 100.0;

    //for i in 0..3 {
    //    let y = base_y + (i as f32 * platform_spacing);
    //    commands.spawn((
    //        Sprite {
    //            color: Color::srgb(0.5, 0.5, 0.5),
    //            custom_size: Some(config::PLATFORM_SIZE),
    //            ..default()
    //        },
    //        Transform::from_xyz(0.0, y, 0.0),
    //        Platform {
    //            width: config::PLATFORM_SIZE.x,
    //        },
    //        RigidBody::Static,
    //        Collider::rectangle(config::PLATFORM_SIZE.x, config::PLATFORM_SIZE.y),
    //        Friction::new(0.05),
    //        LockedAxes::new().lock_rotation(),
    //    ));
    //}

    // Single paddle closer to buckets //
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
        Friction::new(0.05),
        LockedAxes::new().lock_rotation(),
    ));

    let colors = [
        Color::srgb(1.0, 0.0, 0.0),
        Color::srgb(0.0, 1.0, 0.0),
        Color::srgb(0.0, 0.0, 1.0),
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

    commands.insert_resource(SpawnTimer::default());
    commands.insert_resource(GameTimer::default());
}

pub const fn handle_input() {
    // No marble spawning here anymore; input only handles paddle movement
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

// in case we want to restore multiple paddles
// pub fn move_platforms(
//     mut query_set: ParamSet<(
//         Query<(&mut Transform, &Platform), (With<Platform>, Without<Marble>)>,
//         Query<
//             (&Transform, &mut LinearVelocity, &Circle, &Marble),
//             (With<Marble>, Without<Platform>),
//         >,
//     )>,
//     mouse_input: Res<ButtonInput<MouseButton>>,
//     touch_input: Res<Touches>,
//     windows: Query<&Window>,
//     camera: Query<(&Camera, &GlobalTransform)>,
// ) {
//     if let Some(world_position) =
//         pressed_world_position(&mouse_input, &touch_input, &windows, &camera)
//     {
//         // Get a reference to p0 first and keep it
//         let mut platforms = query_set.p0();

//         // Find the minimum distance to any platform
//         let min_distance = platforms
//             .iter()
//             .map(|(transform, _)| (transform.translation.y - world_position.y).abs())
//             .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
//             .unwrap_or(f32::MAX);

//         // Find the closest platform
//         let closest_platform = platforms.iter_mut().find(|(transform, _)| {
//             (transform.translation.y - world_position.y).abs() == min_distance
//         });

//         // Move only the closest platform
//         if let Some((mut transform, platform)) = closest_platform {
//             let platform_radius = platform.width / 2.0;
//             let new_x = world_position.x.clamp(
//                 -WINDOW_WIDTH / 2.0 + platform_radius,
//                 WINDOW_WIDTH / 2.0 - platform_radius,
//             );
//             transform.translation.x = new_x;
//         }

//         // Collect platform data for updating marbles
//         let platform_data: Vec<(f32, f32, f32)> = platforms
//             .iter()
//             .map(|(transform, platform)| {
//                 (
//                     transform.translation.x,
//                     transform.translation.y,
//                     platform.width,
//                 )
//             })
//             .collect();

//         // Now access p1
//         let mut marbles = query_set.p1();

//         for (marble_transform, mut velocity, circle, _marble) in marbles.iter_mut() {
//             for &(platform_x, platform_y, platform_width) in &platform_data {
//                 if (marble_transform.translation.y - platform_y).abs() < 10.0
//                     && (marble_transform.translation.x - platform_x).abs()
//                         < platform_width / 2.0 + circle.radius
//                 {
//                     let dx = world_position.x - platform_x;
//                     velocity.x += dx * 0.1; // Gentle push
//                 }
//             }
//         }
//     }
// }

pub fn update_game(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut score: ResMut<Score>,
    mut score_display: Query<&mut Text2d, With<ScoreDisplay>>,
    bucket_query: Query<(&Transform, &Bucket), With<Bucket>>,
    asset_server: Res<AssetServer>,
    mut marble_query: Query<(Entity, &Transform, &Circle, &mut Resting, &Marble), With<Marble>>,
) {
    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished() {
        // Randomly decide if this marble is colored (75% chance) or grey (25% chance)
        let is_colored = fastrand::f32() < 0.75;
        let color = if is_colored {
            match fastrand::u32(0..3) {
                0 => Color::srgb(1.0, 0.0, 0.0),
                1 => Color::srgb(0.0, 1.0, 0.0),
                _ => Color::srgb(0.0, 0.0, 1.0),
            }
        } else {
            Color::srgb(0.5, 0.5, 0.5)
        };

        let horizontal_velocity = fastrand::f32().mul_add(200.0, -100.0);
        commands.spawn((
            Transform::from_xyz(
                fastrand::f32().mul_add(WINDOW_WIDTH - config::MARBLE_SIZE, -(WINDOW_WIDTH / 2.0)),
                WINDOW_HEIGHT / 2.0,
                0.0,
            ),
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
            Restitution::new(1.75),
            Friction::new(0.05),
            LinearDamping(0.1),
            AngularDamping(0.1),
            GravityScale(1.0),
            LinearVelocity(Vec2::new(horizontal_velocity, 0.0)),
            Mass(1.0),
            Resting(Timer::new(Duration::from_secs(1), TimerMode::Once)),
        ));
        spawn_timer.timer.reset();
    }

    // Update score display
    if let Some(mut score_text) = score_display.iter_mut().next() {
        *score_text = Text2d::new(format!("Score: {}", score.0));
    }

    // Collect bucket data to avoid borrow conflicts
    let buckets: Vec<(Vec2, Color, f32)> = bucket_query
        .iter()
        .map(|(transform, bucket)| (transform.translation.truncate(), bucket.color, bucket.width))
        .collect();

    // Process marbles
    for (marble_entity, transform, circle, mut resting, marble) in &mut marble_query {
        resting.0.tick(time.delta());
        let marble_pos = transform.translation.truncate();
        let marble_radius = circle.radius;

        for (bucket_pos, bucket_color, bucket_width) in &buckets {
            let bucket_half_width = *bucket_width / 2.0;
            let bucket_half_height = config::BUCKET_SIZE.y / 2.0;

            // Check if the marble's bounding box overlaps with the bucket's bounding box
            let dx = (marble_pos.x - bucket_pos.x).abs();
            let dy = (marble_pos.y - bucket_pos.y).abs();
            if dx < (bucket_half_width + marble_radius) && dy < (bucket_half_height + marble_radius)
            {
                commands.entity(marble_entity).despawn();
                if marble.is_target && circle.color == *bucket_color {
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
                break; // Exit the bucket loop since the marble has been handled
            }
        }

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

pub fn update_game_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    score: Res<Score>,
    mut timer_display: Query<&mut Text2d, With<TimerDisplay>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    game_timer.timer.tick(time.delta());

    // Update timer display
    if let Some(mut timer_text) = timer_display.iter_mut().next() {
        let remaining = game_timer.timer.remaining_secs().ceil() as i32;
        *timer_text = Text2d::new(format!("Time: {remaining}"));
    }

    if game_timer.timer.just_finished() {
        next_state.set(GameState::GameOver);
        send_bit_message(BitMessage::End(BitResult::HighestScore(score.0.into())));
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
