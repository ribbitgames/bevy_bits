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

    // Create the welcome screen with better text fitting
    commands.spawn((
        WelcomeScreen,
        Text2d::new("Tower Tumble\n\nCarefully remove\nblocks without\ntoppling the tower\n\nClick to Start".to_string()),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 28.0, // Reduced font size for better fitting
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Transform::default(),
    ));
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

    commands
        .spawn((
            GameOverScreen,
            Transform::from_xyz(0.0, 0.0, 1.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                custom_size: Some(Vec2::new(800.0, 600.0)),
                ..default()
            },
            Name::new("Game Over Screen"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text(format!(
                    "Game Over!\n\nTower Collapsed\nReached Level {}\nFinal Score: {}\n\nClick to Restart",
                    level_settings.level, game_progress.score
                )),
                Transform::from_xyz(0.0, 0.0, 0.1),
                GlobalTransform::default(),
            ));
        });
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

    commands
        .spawn((
            LevelCompleteScreen,
            Transform::from_xyz(0.0, 0.0, 1.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                custom_size: Some(Vec2::new(800.0, 600.0)),
                ..default()
            },
            Name::new("Level Complete Screen"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text(format!(
                    "Level {} Complete!\n\nScore: {}\nTime Bonus: +{}\nTotal Score: {}\n\nClick to Continue",
                    level_settings.level,
                    game_progress.score - time_bonus,
                    time_bonus,
                    game_progress.score
                )),
                Transform::from_xyz(0.0, 0.0, 0.1),
                GlobalTransform::default(),
            ));
        });
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
    commands
        .spawn((
            GameUI,
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new("Game UI"),
        ))
        .with_children(|parent| {
            // Score
            parent.spawn((
                UITextType::Score,
                Text("Score: 0".to_string()),
                Transform::from_xyz(-250.0, 280.0, 10.0),
                GlobalTransform::default(),
            ));

            // Level
            parent.spawn((
                UITextType::Level,
                Text("Level: 1".to_string()),
                Transform::from_xyz(250.0, 280.0, 10.0),
                GlobalTransform::default(),
            ));

            // Timer
            parent.spawn((
                UITextType::Timer,
                Text("Time: 90".to_string()),
                Transform::from_xyz(250.0, 250.0, 10.0),
                GlobalTransform::default(),
            ));

            // Blocks
            parent.spawn((
                UITextType::Blocks,
                Text("Blocks: 0/15".to_string()),
                Transform::from_xyz(-250.0, 250.0, 10.0),
                GlobalTransform::default(),
            ));
        });
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
                // Note: Color cannot be changed here with 2D Text; requires UI text or custom sprite
            }
            UITextType::Blocks => {
                text.0 = format!("Blocks: {}/15", game_progress.blocks_removed);
            }
        }
    }
}
