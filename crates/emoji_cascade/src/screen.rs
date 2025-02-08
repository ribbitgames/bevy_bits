use bevy::color::palettes::css::{GREEN, YELLOW};
use bevy::prelude::*;
use bits_helpers::input::just_pressed_world_position;
use bits_helpers::FONT;

use crate::game::{GameProgress, GameState};

/// Plugin for handling various screens in the game (Welcome, Game Over, and Score Display).
pub struct ScreenPlugin;

/// Component marker for welcome screen entities
#[derive(Component)]
struct WelcomeScreen;

/// Component marker for game over screen entities
#[derive(Component)]
struct GameOverScreen;

/// Component marker for score display entities
#[derive(Component)]
struct ScoreDisplay;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
            .add_systems(
                Update,
                handle_welcome_input.run_if(in_state(GameState::Welcome)),
            )
            .add_systems(OnExit(GameState::Welcome), despawn_screen::<WelcomeScreen>)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(
                Update,
                handle_game_over_input.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(
                OnExit(GameState::GameOver),
                despawn_screen::<GameOverScreen>,
            )
            .add_systems(OnEnter(GameState::Playing), spawn_score_display)
            .add_systems(
                Update,
                update_score_display.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_screen::<ScoreDisplay>);
    }
}

fn spawn_welcome_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(FONT);
    let base_text_font = TextFont {
        font,
        font_size: 50.0,
        ..default()
    };

    commands
        .spawn((WelcomeScreen, Transform::default(), Visibility::default()))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text2d::new("Emoji Cascade"),
                base_text_font.clone().with_font_size(64.0),
                TextLayout::new_with_justify(JustifyText::Center),
                Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
            ));

            // Instructions
            parent.spawn((
                Text2d::new("Match 3 or more emojis\nby sliding rows and columns"),
                base_text_font.clone(),
                TextLayout::new_with_justify(JustifyText::Center),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ));

            // Start prompt
            parent.spawn((
                Text2d::new("Click or tap to start"),
                base_text_font.with_font_size(40.0),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::Srgba(YELLOW)),
                Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
            ));
        });
}

fn spawn_game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    progress: Res<GameProgress>,
) {
    let font = asset_server.load(FONT);
    let base_text_font = TextFont {
        font,
        font_size: 50.0,
        ..default()
    };

    commands
        .spawn((GameOverScreen, Transform::default(), Visibility::default()))
        .with_children(|parent| {
            // Game Over text
            parent.spawn((
                Text2d::new("Game Over!"),
                base_text_font.clone().with_font_size(64.0),
                TextLayout::new_with_justify(JustifyText::Center),
                Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
            ));

            // Final Score
            parent.spawn((
                Text2d::new(format!("Final Score: {}", progress.score)),
                base_text_font.clone(),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::Srgba(YELLOW)),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ));

            // Level Reached
            parent.spawn((
                Text2d::new(format!("Level: {}", progress.level)),
                base_text_font.clone().with_font_size(40.0),
                TextLayout::new_with_justify(JustifyText::Center),
                Transform::from_translation(Vec3::new(0.0, -50.0, 0.0)),
            ));

            // Restart prompt
            parent.spawn((
                Text2d::new("Click or tap to play again"),
                base_text_font.with_font_size(40.0),
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::Srgba(GREEN)),
                Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
            ));
        });
}

fn spawn_score_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(FONT);
    let score_text_font = TextFont {
        font,
        font_size: 32.0,
        ..default()
    };

    commands
        .spawn((
            ScoreDisplay,
            Transform::from_translation(Vec3::new(-350.0, 250.0, 0.0)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new("Score: 0\nMoves: 15\nLevel: 1"),
                score_text_font,
                TextLayout::new(JustifyText::Left, LineBreak::WordBoundary),
            ));
        });
}

fn handle_welcome_input(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q).is_some() {
        next_state.set(GameState::Playing);
    }
}

fn handle_game_over_input(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q).is_some() {
        commands.insert_resource(GameProgress::default());
        next_state.set(GameState::Welcome);
    }
}

fn update_score_display(
    mut query: Query<&mut Text2d, With<ScoreDisplay>>,
    progress: Res<GameProgress>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        text.0 = format!(
            "Score: {}\nMoves: {}\nLevel: {}",
            progress.score, progress.moves_remaining, progress.level
        );
    }
}

fn despawn_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
