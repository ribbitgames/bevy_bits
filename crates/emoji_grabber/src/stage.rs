use bevy::prelude::*;
use bits_helpers::{emoji, FONT};

use crate::core::{CorrectEmojisFound, GameState, GameTimer, Score, StageConfig, TargetEmojiInfo};

/// Component marker for stage complete screen entities
#[derive(Component)]
pub struct StageCompleteScreen;

/// Spawns the stage completion screen
pub fn spawn_stage_complete_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
    stage_config: Res<StageConfig>,
    correct_emojis_found: Res<CorrectEmojisFound>,
) {
    let completion_text = if correct_emojis_found.0 >= stage_config.stage.correct_emojis {
        format!(
            "Stage {} Complete!\n\nYou found all {}\nTotal Score: {}\n\nClick to continue",
            stage_config.current_stage_number, stage_config.stage.correct_emojis, score.0
        )
    } else {
        format!(
            "Time's Up!\n\nYou found {} of {}\nTotal Score: {}\n\nClick to continue",
            correct_emojis_found.0, stage_config.stage.correct_emojis, score.0
        )
    };

    commands.spawn((
        StageCompleteScreen,
        Text2d::new(completion_text),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::default(),
    ));
}

/// Handles input on the stage completion screen
pub fn handle_stage_complete_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stage_config: ResMut<StageConfig>,
    mut target_info: ResMut<TargetEmojiInfo>,
    atlas: Res<emoji::EmojiAtlas>,
    validation: Res<emoji::AtlasValidation>,
    mut commands: Commands,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        // Update stage configuration
        stage_config.current_stage_number += 1;
        stage_config.stage.emoji_speed *= 1.2; // Increase speed by 20%
        stage_config.stage.total_emojis += 5; // Add 5 more emojis
        stage_config.stage.time_limit *= 0.9; // Reduce time by 10%

        // Pick a new target emoji
        if let Some(&index) = emoji::get_random_emojis(&atlas, &validation, 1).first() {
            target_info.index = index;
        }

        // Reset stage progress
        commands.insert_resource(CorrectEmojisFound(0));
        commands.insert_resource(GameTimer(Timer::new(
            std::time::Duration::from_secs_f32(stage_config.stage.time_limit),
            TimerMode::Once,
        )));

        // Transition to welcome screen
        next_state.set(GameState::Welcome);
    }
}

/// Cleans up the stage completion screen
pub fn cleanup_stage_complete(
    mut commands: Commands,
    query: Query<Entity, With<StageCompleteScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
