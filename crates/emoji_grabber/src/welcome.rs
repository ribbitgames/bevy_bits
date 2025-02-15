use bevy::prelude::*;
use bits_helpers::{emoji, FONT, WINDOW_HEIGHT};

use crate::core::{GameState, GameTimer, StageConfig, TargetEmojiInfo};

/// Component marker for welcome screen entities
#[derive(Component)]
pub struct WelcomeScreen;

/// Attempts to spawn the welcome screen once emoji assets are loaded
pub fn try_spawn_welcome_screen(
    mut commands: Commands,
    atlas: Option<Res<emoji::EmojiAtlas>>,
    validation: Option<Res<emoji::AtlasValidation>>,
    stage_config: Res<StageConfig>,
    mut target_info: ResMut<TargetEmojiInfo>,
    asset_server: Res<AssetServer>,
    welcome_screen: Query<&WelcomeScreen>,
) {
    // Don't spawn if we already have a welcome screen
    if !welcome_screen.is_empty() {
        return;
    }

    let (Some(atlas), Some(validation)) = (atlas, validation) else {
        return;
    };

    if !emoji::is_emoji_system_ready(&validation) {
        return;
    }

    // Select random emoji index for target
    let indices = emoji::get_random_emojis(&atlas, &validation, 1);
    let Some(&index) = indices.first() else {
        return;
    };

    target_info.index = index;

    // Spawn welcome screen entity
    let welcome_screen_entity = commands
        .spawn((WelcomeScreen, Transform::default(), Visibility::default()))
        .id();

    // Spawn welcome text
    commands
        .spawn((
            Text2d::new(format!(
                "Stage {}\nFind {} of me!",
                stage_config.current_stage_number, stage_config.stage.correct_emojis
            )),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 32.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(0.0, WINDOW_HEIGHT / 4.0, 0.0)),
        ))
        .set_parent(welcome_screen_entity);

    // Spawn target emoji preview
    let emoji_transform = Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.0));

    if let Some(emoji_entity) =
        emoji::spawn_emoji(&mut commands, &atlas, &validation, index, emoji_transform)
    {
        commands
            .entity(emoji_entity)
            .set_parent(welcome_screen_entity);
    }
}

/// Handles input on the welcome screen
pub fn handle_welcome_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_timer: ResMut<GameTimer>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        game_timer.0.reset();
        next_state.set(GameState::Playing);
    }
}

/// Cleans up the welcome screen when leaving the state
pub fn despawn_welcome_screen(mut commands: Commands, query: Query<Entity, With<WelcomeScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
