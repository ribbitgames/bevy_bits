use bevy::color::palettes::css::GOLD;
use bevy::prelude::*;

use crate::gameplay::{GameState, ScoreInfo, ScoredEvent};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update_score)
            .add_systems(OnEnter(GameState::Ready), update_high_score);
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct HighScoreText;

fn setup(mut commands: Commands) {
    commands.spawn((
        Text::new("0"),
        TextFont {
            font_size: 100.0,
            ..default()
        },
        TextColor(GOLD.into()),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            justify_self: JustifySelf::Center,
            ..default()
        },
        ScoreText,
    ));

    // todo: Also need to disable file creation etc
    //if !cfg!(target_arch = "wasm32") {
    commands.spawn((
        Text::new("0"),
        TextFont {
            font_size: 50.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            justify_self: JustifySelf::End,
            ..default()
        },
        HighScoreText,
    ));
}

fn update_score(
    mut score_query: Query<&mut Text, With<ScoreText>>,
    mut scored_event: EventReader<ScoredEvent>,
    score_info: Res<ScoreInfo>,
) {
    for _ in scored_event.read() {
        for mut text in &mut score_query {
            text.0 = format!("{num}", num = score_info.current_score);
        }
    }
}

fn update_high_score(
    mut high_score_query: Query<&mut Text, With<HighScoreText>>,
    score_info: Res<ScoreInfo>,
) {
    for mut text in &mut high_score_query {
        text.0 = format!("{num} ", num = score_info.high_score);
    }
}
