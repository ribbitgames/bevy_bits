use bevy::color::palettes::css;
use bevy::prelude::*;
use bits_helpers::FONT;

use crate::game::{GameProgress, GameState, LevelSettings};

#[derive(Component)]
pub struct WelcomeScreen;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct LevelCompleteScreen;

#[derive(Component)]
pub struct GameUI;

#[derive(Component)]
pub enum UITextType {
    Score,
    Level,
    Timer,
    Blocks,
}

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
            try_spawn_level_complete.run_if(in_state(GameState::LevelComplete)),
        )
        .add_systems(
            Update,
            handle_level_complete_input.run_if(in_state(GameState::LevelComplete)),
        )
        .add_systems(
            OnExit(GameState::LevelComplete),
            despawn_screen::<LevelCompleteScreen>,
        )
        .add_systems(OnEnter(GameState::Playing), spawn_game_ui)
        .add_systems(Update, update_game_ui.run_if(in_state(GameState::Playing)))
        .add_systems(OnExit(GameState::Playing), despawn_screen::<GameUI>);
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

    // Create a UI root node
    let ui_root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            WelcomeScreen,
        ))
        .id();

    // Spawn the welcome text as a child of the root
    let welcome_text = commands
        .spawn((
            Text::new("Tower Tumble\n\nCarefully remove\nblocks without\ntoppling the tower\n\nClick to Start"),
            TextColor(Color::WHITE),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 28.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
        ))
        .id();

    // Build the hierarchy
    commands.entity(ui_root).add_child(welcome_text);
}

fn handle_welcome_input(
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if buttons.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

fn try_spawn_game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_settings: Res<LevelSettings>,
    game_progress: Res<GameProgress>,
    query: Query<&GameOverScreen>,
) {
    if !query.is_empty() {
        return;
    }

    // Create a semi-transparent overlay with a UI root node
    let overlay = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,

                ..default()
            },
            GameOverScreen,
            Name::new("Game Over Screen"),
        ))
        .id();

    // Add the game over text as a child
    let gameover_text = commands
        .spawn((
            Text::new(format!(
                "Game Over!\n\nTower Collapsed\nReached Level {}\nFinal Score: {}\n\nClick to Restart",
                level_settings.level, game_progress.score
            )),
            TextColor(Color::WHITE),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 28.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
        ))
        .id();

    // Build the hierarchy
    commands.entity(overlay).add_child(gameover_text);
}

fn handle_game_over_input(
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if buttons.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        commands.insert_resource(LevelSettings::default());
        commands.insert_resource(GameProgress::default());
        next_state.set(GameState::Welcome);
    }
}

fn try_spawn_level_complete(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_settings: Res<LevelSettings>,
    mut game_progress: ResMut<GameProgress>,
    query: Query<&LevelCompleteScreen>,
) {
    if !query.is_empty() {
        return;
    }

    let time_bonus = (game_progress.level_timer.remaining_secs() as u32) * 5;
    game_progress.add_time_bonus();

    // Create a semi-transparent overlay with a UI root node
    let overlay = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,

                ..default()
            },
            LevelCompleteScreen,
            Name::new("Level Complete Screen"),
        ))
        .id();

    // Add the level complete text as a child
    let complete_text = commands
        .spawn((
            Text::new(format!(
                "Level {} Complete!\n\nScore: {}\nTime Bonus: +{}\nTotal Score: {}\n\nClick to Continue",
                level_settings.level,
                game_progress.score - time_bonus,
                time_bonus,
                game_progress.score
            )),
            TextColor(Color::WHITE),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 28.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
        ))
        .id();

    // Build the hierarchy
    commands.entity(overlay).add_child(complete_text);
}

fn handle_level_complete_input(
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if buttons.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

fn spawn_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create UI root node
    let ui_root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GameUI,
            Name::new("Game UI"),
        ))
        .id();

    // Score - top left
    let score_text = commands
        .spawn((
            Text::new("Score: 0"),
            UITextType::Score,
            TextFont {
                font: asset_server.load(FONT),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            // Position manually within the node
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                ..default()
            },
        ))
        .id();

    // Level - top right
    let level_text = commands
        .spawn((
            Text::new("Level: 1"),
            UITextType::Level,
            TextFont {
                font: asset_server.load(FONT),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                ..default()
            },
        ))
        .id();

    // Timer - below level
    let timer_text = commands
        .spawn((
            Text::new("Time: 90"),
            UITextType::Timer,
            TextFont {
                font: asset_server.load(FONT),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(40.0),
                ..default()
            },
        ))
        .id();

    // Blocks - below score
    let blocks_text = commands
        .spawn((
            Text::new("Blocks: 0/15"),
            UITextType::Blocks,
            TextFont {
                font: asset_server.load(FONT),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(40.0),
                ..default()
            },
        ))
        .id();

    // Build the hierarchy
    commands
        .entity(ui_root)
        .add_child(score_text)
        .add_child(level_text)
        .add_child(timer_text)
        .add_child(blocks_text);
}

fn update_game_ui(
    mut text_query: Query<(&mut Text, &UITextType), With<GameUI>>,
    game_progress: Res<GameProgress>,
    level_settings: Res<LevelSettings>,
) {
    for (mut text, text_type) in &mut text_query {
        match text_type {
            UITextType::Score => {
                text.0 = format!("Score: {}", game_progress.score);
            }
            UITextType::Level => {
                text.0 = format!("Level: {}", level_settings.level);
            }
            UITextType::Timer => {
                let time_remaining = game_progress.level_timer.remaining_secs() as u32;
                text.0 = format!("Time: {}", time_remaining);

                // Change color based on time remaining
                // Can't change text color dynamically in Bevy 0.15.3
                // We only update the text content
            }
            UITextType::Blocks => {
                text.0 = format!("Blocks: {}/15", game_progress.blocks_removed);
            }
        }
    }
}
