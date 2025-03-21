use bevy::prelude::*;
use bits_helpers::FONT;

use crate::game::{GameDifficulty, GameProgress, GameState, REVEAL_TIME_PER_CARD};

#[derive(Component)]
pub struct WelcomeScreen;

#[derive(Component)]
pub struct StageTransitionScreen;

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
            try_spawn_stage_transition.run_if(in_state(GameState::StageComplete)),
        )
        .add_systems(
            Update,
            handle_stage_transition_input.run_if(in_state(GameState::StageComplete)),
        )
        .add_systems(
            OnExit(GameState::StageComplete),
            despawn_screen::<StageTransitionScreen>,
        );
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
        Text2d::new("Memoji\nClick to Start".to_string()),
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

fn try_spawn_stage_transition(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    difficulty: Res<GameDifficulty>,
    game_progress: Res<GameProgress>,
    query: Query<&StageTransitionScreen>,
) {
    if !query.is_empty() {
        return;
    }

    commands.spawn((
        StageTransitionScreen,
        Text2d::new(format!(
            "Stage {} Complete!\n\nMistakes: {}/{}\nNext Stage: {}x{} Grid\nReveal Time: {:.1}s\n\nClick to Continue",
            difficulty.stage,
            game_progress.mistakes,
            game_progress.max_mistakes,
            difficulty.grid_rows,
            difficulty.grid_cols,
            (difficulty.grid_rows * difficulty.grid_cols) as f32 * REVEAL_TIME_PER_CARD
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
