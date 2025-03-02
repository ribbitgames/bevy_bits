use bevy::prelude::*;
use bits_helpers::floating_score::{FloatingScore, spawn_floating_score};
use bits_helpers::input::pressed_world_position;
use bits_helpers::{WINDOW_HEIGHT, WINDOW_WIDTH, emoji};

use crate::core::config::{
    CATCHER_SIZE, MAX_EMOJI_SIZE, MAX_FALL_SPEED, MIN_EMOJI_SIZE, MIN_SPAWN_INTERVAL,
    SPAWN_RATE_DECREASE, SPEED_INCREASE_RATE,
};
use crate::core::{Catcher, FallingEmoji, GameState, Score, SpawnTimer, TargetEmojiIndex};

/// Component to wrap Timer for game over delay
#[expect(dead_code)]
#[derive(Component)]
pub struct GameOverDelay(Timer);

/// Spawns initial game elements including the catcher and UI
pub fn spawn_game_elements(
    mut commands: Commands,
    atlas: Res<emoji::EmojiAtlas>,
    validation: Res<emoji::AtlasValidation>,
    asset_server: Res<AssetServer>,
) {
    // Spawn catcher at bottom of screen
    if let Some(catcher_entity) = emoji::spawn_emoji(
        &mut commands,
        &atlas,
        &validation,
        0, // Using first emoji for catcher
        Transform::from_xyz(0.0, -WINDOW_HEIGHT / 2.0 + CATCHER_SIZE.y, 0.0)
            .with_scale(Vec3::splat(CATCHER_SIZE.x / 64.0)),
    ) {
        commands.entity(catcher_entity).insert(Catcher {
            width: CATCHER_SIZE.x,
        });
    }

    // Spawn score text
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font: asset_server.load(bits_helpers::FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Left),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + 20.0, WINDOW_HEIGHT / 2.0 - 30.0, 0.0),
    ));

    // Initialize spawn timer
    commands.insert_resource(SpawnTimer::default());
}

/// Handles catcher movement based on input
pub fn handle_input(
    mut catcher_query: Query<&mut Transform, With<Catcher>>,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(mut catcher_transform) = catcher_query.get_single_mut() else {
        return;
    };

    if let Some(world_pos) =
        pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
    {
        // Clamp position to screen bounds
        let new_x = world_pos.x.clamp(
            -WINDOW_WIDTH / 2.0 + CATCHER_SIZE.x / 2.0,
            WINDOW_WIDTH / 2.0 - CATCHER_SIZE.x / 2.0,
        );

        catcher_transform.translation.x = new_x;
    }
}

/// Updates game state and spawns new emojis
pub fn update_game(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
    atlas: Res<emoji::EmojiAtlas>,
    validation: Res<emoji::AtlasValidation>,
    mut score_query: Query<&mut Text2d>,
    score: Res<Score>,
    target_emoji: Res<TargetEmojiIndex>,
) {
    // Update spawn timer
    spawn_timer.timer.tick(time.delta());

    // Increase difficulty
    spawn_timer.current_speed = SPEED_INCREASE_RATE
        .mul_add(time.delta_secs(), spawn_timer.current_speed)
        .min(MAX_FALL_SPEED);
    spawn_timer.spawn_rate = SPAWN_RATE_DECREASE
        .mul_add(-time.delta_secs(), spawn_timer.spawn_rate)
        .max(MIN_SPAWN_INTERVAL);

    // Update score display
    if let Some(mut score_text) = score_query.iter_mut().next() {
        *score_text = Text2d::new(format!("Score: {}", score.0));
    }

    // Spawn new emoji if timer finished
    if spawn_timer.timer.just_finished() {
        // Determine if this should be the target emoji (25% chance)
        let is_target = fastrand::f32() < 0.25;

        // Get the target emoji index, or use fallback if not set
        let target_index = target_emoji.0.unwrap_or_else(|| {
            // Fallback in case target emoji is not set (should never happen)
            let indices = emoji::get_random_emojis(&atlas, &validation, 1);
            indices.first().copied().unwrap_or(0)
        });

        let emoji_index = if is_target {
            // Always use the target emoji index for target emojis
            target_index
        } else {
            // Get a random emoji that is NOT the target emoji
            let index;

            loop {
                let indices = emoji::get_random_emojis(&atlas, &validation, 1);
                if let Some(&idx) = indices.first() {
                    if idx != target_index {
                        index = idx;
                        break;
                    }
                } else {
                    // Fallback if no other emoji is available
                    index = if target_index > 0 { 0 } else { 1 };
                    break;
                }
            }

            index
        };

        // Random size and position
        let size = fastrand::f32().mul_add(MAX_EMOJI_SIZE - MIN_EMOJI_SIZE, MIN_EMOJI_SIZE);
        let x = fastrand::f32().mul_add(WINDOW_WIDTH - size, -(WINDOW_WIDTH / 2.0));

        // Create transform for emoji
        let emoji_transform = Transform::from_xyz(x, WINDOW_HEIGHT / 2.0 + size, 0.0)
            .with_scale(Vec3::splat(size / 64.0));

        // Spawn the emoji
        if let Some(emoji_entity) = emoji::spawn_emoji(
            &mut commands,
            &atlas,
            &validation,
            emoji_index,
            emoji_transform,
        ) {
            commands.entity(emoji_entity).insert(FallingEmoji {
                speed: spawn_timer.current_speed,
                is_target,
                size,
            });
        }

        // Reset timer with new spawn rate
        spawn_timer.timer = Timer::from_seconds(spawn_timer.spawn_rate, TimerMode::Once);
    }
}

/// Updates falling emoji positions and handles collisions
pub fn move_emojis(
    mut commands: Commands,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut emoji_query: Query<(Entity, &mut Transform, &FallingEmoji)>,
    catcher_query: Query<(&Transform, &Catcher), Without<FallingEmoji>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    target_emoji: Res<TargetEmojiIndex>,
) {
    let Ok((catcher_transform, catcher)) = catcher_query.get_single() else {
        return;
    };

    let catcher_pos = catcher_transform.translation.truncate();
    let catcher_y = catcher_pos.y;

    for (entity, mut transform, emoji) in &mut emoji_query {
        // Move emoji down
        transform.translation.y -= emoji.speed * time.delta_secs();

        // Check for collision with catcher
        if (transform.translation.y - catcher_y).abs() < CATCHER_SIZE.y
            && (transform.translation.x - catcher_pos.x).abs() < (catcher.width + emoji.size) / 2.0
        {
            commands.entity(entity).despawn();

            if emoji.is_target {
                // Caught target emoji - award points
                score.0 += 10;
                spawn_floating_score(
                    &mut commands,
                    Vec2::new(transform.translation.x, transform.translation.y),
                    "+10",
                    bevy::color::palettes::css::GREEN,
                    &asset_server,
                );
            } else {
                // Caught non-target emoji - small bonus
                score.0 += 2;
                spawn_floating_score(
                    &mut commands,
                    Vec2::new(transform.translation.x, transform.translation.y),
                    "+2",
                    bevy::color::palettes::css::YELLOW,
                    &asset_server,
                );
            }
        }
        // Remove if passed bottom of screen
        else if transform.translation.y < -WINDOW_HEIGHT / 2.0 - emoji.size {
            commands.entity(entity).despawn();

            if emoji.is_target {
                // Missed target emoji - game over!
                score.0 -= 10;
                spawn_floating_score(
                    &mut commands,
                    Vec2::new(transform.translation.x, -WINDOW_HEIGHT / 2.0),
                    "MISSED TARGET!",
                    bevy::color::palettes::css::RED,
                    &asset_server,
                );

                commands.spawn((GameOverDelay(Timer::from_seconds(0.5, TimerMode::Once)),));
                next_state.set(GameState::GameOver);
                return;
            }
        }
    }
}

/// Cleans up game entities when leaving the Playing state
pub fn cleanup_game(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<Catcher>,
            With<FallingEmoji>,
            With<Text2d>,
            With<FloatingScore>,
            With<GameOverDelay>,
        )>,
    >,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
