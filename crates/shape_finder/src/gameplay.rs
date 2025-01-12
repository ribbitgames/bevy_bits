use bevy::color::palettes::css::{GREEN, RED};
use bevy::prelude::*;
use bits_helpers::floating_score::{spawn_floating_score, FloatingScore};
use bits_helpers::{emoji, FONT, WINDOW_HEIGHT, WINDOW_WIDTH};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::core::{
    CorrectEmojisFound, EmojiClickedEvent, GameState, GameTimer, MovingEmoji, Score, StageConfig,
    TargetEmojiInfo, Velocity,
};

const UI_MARGIN: f32 = 50.0; // Height of the UI safe zone from top

/// Spawns all emojis for the current stage
pub fn spawn_emojis(
    mut commands: Commands,
    atlas: Res<emoji::EmojiAtlas>,
    validation: Res<emoji::AtlasValidation>,
    target_info: Res<TargetEmojiInfo>,
    stage_config: Res<StageConfig>,
) {
    if !emoji::is_emoji_system_ready(&validation) {
        return;
    }

    let mut rng = rand::thread_rng();
    let mut emojis = Vec::new();

    // Add correct emojis
    for _ in 0..stage_config.stage.correct_emojis {
        emojis.push(target_info.index);
    }

    // Add other random emojis
    let other_indices = emoji::get_random_emojis(
        &atlas,
        &validation,
        stage_config.stage.total_emojis - stage_config.stage.correct_emojis,
    );
    emojis.extend(other_indices);
    emojis.shuffle(&mut rng);

    // Spawn all emojis with random positions and velocities
    for &index in &emojis {
        let size = rng.gen_range(40.0..80.0);
        let x = rng.gen_range(-WINDOW_WIDTH / 2.0 + size..WINDOW_WIDTH / 2.0 - size);
        let y = rng.gen_range(-WINDOW_HEIGHT / 2.0 + size..WINDOW_HEIGHT / 2.0 - UI_MARGIN - size);
        let velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize()
            * stage_config.stage.emoji_speed;

        if let Some(entity) = emoji::spawn_emoji(
            &mut commands,
            &atlas,
            &validation,
            index,
            Vec2::new(x, y),
            size / 128.0,
        ) {
            commands
                .entity(entity)
                .insert((MovingEmoji { index, size }, Velocity(velocity)));
        }
    }
}

/// Spawns the timer UI elements for the game stage
///
/// # Parameters
/// * `commands` - Command buffer to spawn entities
/// * `asset_server` - Resource for loading assets like fonts
/// * `stage_config` - Current stage configuration containing time limit and other settings
pub fn spawn_timer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stage_config: Res<StageConfig>,
) {
    // Timer text - positioned in top right
    commands.spawn((
        Text2d::new(format!("Time: {:.1}", stage_config.stage.time_limit)),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Right),
        Transform::from_translation(Vec3::new(
            WINDOW_WIDTH / 2.2 - 80.0,
            WINDOW_HEIGHT / 2.2 - 20.0,
            0.0,
        )),
    ));

    // Score and progress text - positioned in top left
    commands.spawn((
        Text2d::new(format!("Found: 0/{}", stage_config.stage.correct_emojis)),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Left),
        Transform::from_translation(Vec3::new(
            -WINDOW_WIDTH / 2.2 + 80.0,
            WINDOW_HEIGHT / 2.2 - 20.0,
            0.0,
        )),
    ));
}

/// Updates the timer and checks for stage completion
pub fn update_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut timer_text: Query<&mut Text2d>,
    stage_config: Res<StageConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    correct_emojis_found: Res<CorrectEmojisFound>,
) {
    game_timer.0.tick(time.delta());
    let remaining_time = stage_config.stage.time_limit - game_timer.0.elapsed_secs();

    // Update timer text
    if let Some(mut text) = timer_text.iter_mut().next() {
        *text = Text2d::new(format!("Time: {:.1}", remaining_time.max(0.0)));
    }

    // Update progress text
    if let Some(mut text) = timer_text.iter_mut().nth(1) {
        *text = Text2d::new(format!(
            "Found: {}/{}",
            correct_emojis_found.0, stage_config.stage.correct_emojis
        ));
    }

    // Check for stage completion
    if game_timer.0.just_finished() || correct_emojis_found.0 >= stage_config.stage.correct_emojis {
        next_state.set(GameState::StageComplete);
    }
}

/// Handles mouse input during gameplay
pub fn handle_playing_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    emojis: Query<(Entity, &Transform, &MovingEmoji)>,
    target_info: Res<TargetEmojiInfo>,
    mut emoji_clicked_events: EventWriter<EmojiClickedEvent>,
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
            for (entity, transform, emoji) in emojis.iter() {
                if transform.translation.truncate().distance(world_position) < emoji.size / 2.0 {
                    let is_correct = emoji.index == target_info.index;
                    emoji_clicked_events.send(EmojiClickedEvent {
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

/// Processes emoji click events and updates score
pub fn handle_emoji_clicked(
    mut commands: Commands,
    mut emoji_clicked_events: EventReader<EmojiClickedEvent>,
    mut score: ResMut<Score>,
    mut correct_emojis_found: ResMut<CorrectEmojisFound>,
    asset_server: Res<AssetServer>,
) {
    for event in emoji_clicked_events.read() {
        if event.is_correct {
            score.0 += 1;
            correct_emojis_found.0 += 1;
            spawn_floating_score(&mut commands, event.position, "+1", GREEN, &asset_server);
        } else {
            score.0 -= 1;
            spawn_floating_score(&mut commands, event.position, "-1", RED, &asset_server);
        }
        commands.entity(event.entity).despawn();
    }
}

/// Updates emoji positions and handles collisions while respecting UI safe zone
pub fn move_emojis(
    mut query: Query<(&mut Transform, &mut Velocity, &MovingEmoji)>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, emoji) in &mut query {
        let mut new_pos = transform.translation + velocity.0.extend(0.0) * time.delta_secs();

        // Handle wall collisions
        if new_pos.x - emoji.size / 2.0 < -WINDOW_WIDTH / 2.0
            || new_pos.x + emoji.size / 2.0 > WINDOW_WIDTH / 2.0
        {
            velocity.0.x *= -1.0;
            new_pos.x = new_pos.x.clamp(
                -WINDOW_WIDTH / 2.0 + emoji.size / 2.0,
                WINDOW_WIDTH / 2.0 - emoji.size / 2.0,
            );
        }

        // Handle vertical boundaries with UI safe zone
        let top_boundary = WINDOW_HEIGHT / 2.0 - UI_MARGIN;
        if new_pos.y - emoji.size / 2.0 < -WINDOW_HEIGHT / 2.0
            || new_pos.y + emoji.size / 2.0 > top_boundary
        {
            velocity.0.y *= -1.0;
            new_pos.y = new_pos.y.clamp(
                -WINDOW_HEIGHT / 2.0 + emoji.size / 2.0,
                top_boundary - emoji.size / 2.0,
            );
        }

        transform.translation = new_pos;
    }
}

/// Cleans up all gameplay entities when leaving the Playing state
pub fn cleanup_playing_state(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Text2d>, With<MovingEmoji>, With<FloatingScore>)>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
