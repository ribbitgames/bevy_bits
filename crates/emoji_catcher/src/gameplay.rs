use bevy::prelude::*;
use bits_helpers::floating_score::{spawn_floating_score, FloatingScore};
use bits_helpers::input::pressed_world_position;
use bits_helpers::{emoji, WINDOW_HEIGHT, WINDOW_WIDTH};
use rand::Rng;

use crate::core::config::*;
use crate::core::{Catcher, FallingEmoji, GameState, Score, SpawnTimer};

/// Component to wrap Timer for game over delay
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

/// Updates falling emoji positions and handles collisions
pub fn move_emojis(
    mut commands: Commands,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut emoji_query: Query<(Entity, &mut Transform, &FallingEmoji)>,
    catcher_query: Query<(&Transform, &Catcher), Without<FallingEmoji>>, // Added Without to make queries disjoint
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((catcher_transform, catcher)) = catcher_query.get_single() else {
        return;
    };

    let catcher_pos = catcher_transform.translation.truncate();
    let catcher_y = catcher_pos.y;

    for (entity, mut transform, emoji) in emoji_query.iter_mut() {
        // Move emoji down
        transform.translation.y -= emoji.speed * time.delta_secs();

        // Check for collision with catcher
        if (transform.translation.y - catcher_y).abs() < CATCHER_SIZE.y
            && (transform.translation.x - catcher_pos.x).abs() < (catcher.width + emoji.size) / 2.0
        {
            commands.entity(entity).despawn();

            if emoji.is_bad {
                // Hit bad emoji - show effect and game over
                spawn_floating_score(
                    &mut commands,
                    Vec2::new(transform.translation.x, transform.translation.y),
                    "GAME OVER!",
                    bevy::color::palettes::css::RED,
                    &asset_server,
                );
                // Add small delay before game over
                commands.spawn((GameOverDelay(Timer::from_seconds(0.5, TimerMode::Once)),));
                next_state.set(GameState::GameOver);
                return;
            }

            // Caught good emoji
            score.0 += 10;
            spawn_floating_score(
                &mut commands,
                Vec2::new(transform.translation.x, transform.translation.y),
                "+10",
                bevy::color::palettes::css::GREEN,
                &asset_server,
            );
        }

        // Remove if passed bottom of screen
        if transform.translation.y < -WINDOW_HEIGHT / 2.0 - emoji.size {
            commands.entity(entity).despawn();

            if !emoji.is_bad {
                // Missed good emoji - lose points
                score.0 -= 5;
                spawn_floating_score(
                    &mut commands,
                    Vec2::new(transform.translation.x, -WINDOW_HEIGHT / 2.0),
                    "-5",
                    bevy::color::palettes::css::RED,
                    &asset_server,
                );
            }
        }
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
) {
    // Update spawn timer
    spawn_timer.timer.tick(time.delta());

    // Increase difficulty
    spawn_timer.current_speed =
        (spawn_timer.current_speed + SPEED_INCREASE_RATE * time.delta_secs()).min(MAX_FALL_SPEED);
    spawn_timer.spawn_rate =
        (spawn_timer.spawn_rate - SPAWN_RATE_DECREASE * time.delta_secs()).max(MIN_SPAWN_INTERVAL);

    // Update score display
    if let Some(mut score_text) = score_query.iter_mut().next() {
        *score_text = Text2d::new(format!("Score: {}", score.0));
    }

    // Spawn new emoji if timer finished
    if spawn_timer.timer.just_finished() {
        let mut rng = rand::rng();

        // Determine if this should be a bad emoji
        let is_bad = rng.gen_bool(BAD_EMOJI_PROBABILITY.into());

        // Get random emoji index
        let indices = emoji::get_random_emojis(&atlas, &validation, 1);
        let Some(&index) = indices.first() else {
            return;
        };

        // Random size and position
        let size = rng.gen_range(MIN_EMOJI_SIZE..MAX_EMOJI_SIZE);
        let x = rng.gen_range(-WINDOW_WIDTH / 2.0 + size..WINDOW_WIDTH / 2.0 - size);

        // Create transform for emoji
        let emoji_transform = Transform::from_xyz(x, WINDOW_HEIGHT / 2.0 + size, 0.0)
            .with_scale(Vec3::splat(size / 64.0));

        // Spawn the emoji
        if let Some(emoji_entity) =
            emoji::spawn_emoji(&mut commands, &atlas, &validation, index, emoji_transform)
        {
            commands.entity(emoji_entity).insert(FallingEmoji {
                speed: spawn_timer.current_speed,
                is_bad,
                size,
            });
        }

        // Reset timer with new spawn rate
        spawn_timer.timer = Timer::from_seconds(spawn_timer.spawn_rate, TimerMode::Once);
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
