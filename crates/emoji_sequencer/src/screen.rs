use bevy::prelude::*;
use bits_helpers::FONT;

use crate::game::{GameDifficulty, GameProgress, GameState, ScoreState, REVEAL_TIME_PER_EMOJI};

#[derive(Component)]
pub struct WelcomeScreen;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct StageTransitionScreen;

#[derive(Component)]
pub struct ScoreDisplay;

pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            try_spawn_welcome_screen.run_if(in_state(GameState::Welcome)),
        )
        .add_systems(
            Update,
            handle_welcome_input.run_if(in_state(GameState::Welcome)),
        )
        .add_systems(OnExit(GameState::Welcome), despawn_screen::<WelcomeScreen>)
        .add_systems(
            Update,
            try_spawn_game_over.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            Update,
            handle_game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            OnExit(GameState::GameOver),
            despawn_screen::<GameOverScreen>,
        )
        .add_systems(
            Update,
            try_spawn_stage_transition.run_if(in_state(GameState::StageComplete)),
        )
        .add_systems(
            Update,
            handle_stage_transition_input.run_if(in_state(GameState::StageComplete)),
        )
        .add_systems(
            OnExit(GameState::StageComplete),
            despawn_screen::<StageTransitionScreen>,
        )
        .add_systems(
            Update,
            (spawn_score_display, update_score_display).run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_screen::<ScoreDisplay>);
    }
}

fn despawn_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn try_spawn_welcome_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&WelcomeScreen>,
) {
    if !query.is_empty() {
        return;
    }

    commands.spawn((
        WelcomeScreen,
        Text2d::new(
            "Emoji Sequencer\n\nMemorize the sequence of emojis\nand recreate it in order!\n\nClick to Start"
                .to_string(),
        ),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::default(),
    ));
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

fn try_spawn_game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    difficulty: Res<GameDifficulty>,
    game_progress: Res<GameProgress>,
    score_state: Res<ScoreState>,
    query: Query<&GameOverScreen>,
) {
    if !query.is_empty() {
        return;
    }

    commands.spawn((
        GameOverScreen,
        Text2d::new(format!(
            "Game Over!\n\nReached Stage {}\nFinal Score: {}\nMistakes: {}/{}\n\nClick to Restart",
            difficulty.stage,
            score_state.total_score,
            game_progress.mistakes,
            game_progress.max_mistakes
        )),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::default(),
    ));
}

fn handle_game_over_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        commands.insert_resource(GameDifficulty::default());
        commands.insert_resource(GameProgress::default());
        commands.insert_resource(ScoreState::default());
        next_state.set(GameState::Welcome);
    }
}

fn try_spawn_stage_transition(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    difficulty: Res<GameDifficulty>,
    game_progress: Res<GameProgress>,
    score_state: Res<ScoreState>,
    query: Query<&StageTransitionScreen>,
) {
    if !query.is_empty() {
        return;
    }

    commands.spawn((
        StageTransitionScreen,
        Text2d::new(format!(
            "Stage {} Complete!\n\nScore: +{}\nTotal Score: {}\nMistakes: {}/{}\n\nNext Stage:\nSequence Length: {}\nReveal Time: {:.1}s\n\nClick to Continue",
            difficulty.stage,
            score_state.stage_score,
            score_state.total_score,
            game_progress.mistakes,
            game_progress.max_mistakes,
            difficulty.sequence_length,
            difficulty.sequence_length as f32 * REVEAL_TIME_PER_EMOJI
        )),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::default(),
    ));
}

fn handle_stage_transition_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

fn spawn_score_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&ScoreDisplay>,
) {
    if !query.is_empty() {
        return;
    }

    commands.spawn((
        ScoreDisplay,
        Text2d::new(String::new()),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Left),
        Transform::from_xyz(-380.0, 280.0, 0.0),
    ));
}

fn update_score_display(
    mut query: Query<&mut Text2d, With<ScoreDisplay>>,
    difficulty: Res<GameDifficulty>,
    game_progress: Res<GameProgress>,
    score_state: Res<ScoreState>,
) {
    for mut text in &mut query {
        text.0 = format!(
            "Stage: {}\nScore: {}\nMistakes: {}/{}",
            difficulty.stage,
            score_state.total_score,
            game_progress.mistakes,
            game_progress.max_mistakes
        );
    }
}
