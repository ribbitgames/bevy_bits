// use bevy::prelude::*;
// use bits_helpers::FONT;

// use crate::game::{GameProgress, GameState};

// /// Plugin for handling various screens in the game (Welcome, Game Over, and Score Display).
// pub struct ScreenPlugin;

// #[derive(Component)]
// struct WelcomeScreen;

// #[derive(Component)]
// struct GameOverScreen;

// #[derive(Component)]
// struct ScoreDisplay;

// impl Plugin for ScreenPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(
//             Update,
//             spawn_welcome_screen.run_if(in_state(GameState::Welcome)),
//         )
//         .add_systems(
//             Update,
//             handle_welcome_input.run_if(in_state(GameState::Welcome)),
//         )
//         .add_systems(OnExit(GameState::Welcome), despawn_screen::<WelcomeScreen>)
//         .add_systems(
//             Update,
//             spawn_game_over.run_if(in_state(GameState::GameOver)),
//         )
//         .add_systems(
//             Update,
//             handle_game_over_input.run_if(in_state(GameState::GameOver)),
//         )
//         .add_systems(
//             OnExit(GameState::GameOver),
//             despawn_screen::<GameOverScreen>,
//         )
//         .add_systems(
//             Update,
//             update_score_display.run_if(in_state(GameState::Playing)),
//         )
//         .add_systems(OnEnter(GameState::Playing), spawn_score_display)
//         .add_systems(OnExit(GameState::Playing), despawn_screen::<ScoreDisplay>);
//     }
// }

// /// Spawns the welcome screen with centered text if it does not already exist.
// ///
// /// This system is only active when the game state is `GameState::Welcome`.
// fn spawn_welcome_screen(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     query: Query<&WelcomeScreen>,
// ) {
//     if !query.is_empty() {
//         return;
//     }

//     commands.spawn((
//         WelcomeScreen,
//         Text2dBundle {
//             // Build the text from one section.
//             text: Text::from_section(
//                 "Emoji Cascade\nClick to Start",
//                 TextStyle {
//                     font: asset_server.load(FONT),
//                     font_size: 32.0,
//                     color: Color::WHITE,
//                 },
//             ),
//             // Position the text.
//             transform: Transform::from_xyz(0.0, 50.0, 1.0),
//             ..default()
//         },
//         // Center the text relative to its transform.
//         Text2dAlignment {
//             alignment: Vec2::new(0.5, 0.5),
//         },
//     ));
// }

// /// Handles input on the welcome screen, transitioning to `GameState::Playing` when a click or touch is detected.
// fn handle_welcome_input(
//     mouse_input: Res<Input<MouseButton>>,
//     touch_input: Res<Touches>,
//     mut next_state: ResMut<NextState<GameState>>,
// ) {
//     // Check if the left mouse button was just pressed or if any touch was just started.
//     if mouse_input.just_pressed(MouseButton::Left)
//         || touch_input.iter_just_pressed().next().is_some()
//     {
//         next_state.set(GameState::Playing);
//     }
// }

// /// Spawns the game over screen with centered text showing the final score and restart prompt.
// ///
// /// This system is only active when the game state is `GameState::GameOver`.
// fn spawn_game_over(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     progress: Res<GameProgress>,
//     query: Query<&GameOverScreen>,
// ) {
//     if !query.is_empty() {
//         return;
//     }

//     commands.spawn((
//         GameOverScreen,
//         Text2dBundle {
//             text: Text::from_section(
//                 format!(
//                     "Game Over!\nFinal Score: {}\nClick to Restart",
//                     progress.score
//                 ),
//                 TextStyle {
//                     font: asset_server.load(FONT),
//                     font_size: 32.0,
//                     color: Color::WHITE,
//                 },
//             ),
//             transform: Transform::from_xyz(0.0, 0.0, 1.0),
//             ..default()
//         },
//         // Center the text relative to its transform.
//         Text2dAlignment {
//             alignment: Vec2::new(0.5, 0.5),
//         },
//     ));
// }

// /// Handles input on the game over screen, resetting the game progress and transitioning to `GameState::Welcome` on click or touch.
// fn handle_game_over_input(
//     mouse_input: Res<Input<MouseButton>>,
//     touch_input: Res<Touches>,
//     mut next_state: ResMut<NextState<GameState>>,
//     mut commands: Commands,
// ) {
//     // Check if the left mouse button was just pressed or if any touch was just started.
//     if mouse_input.just_pressed(MouseButton::Left)
//         || touch_input.iter_just_pressed().next().is_some()
//     {
//         // Reset the game progress.
//         commands.insert_resource(GameProgress::default());
//         next_state.set(GameState::Welcome);
//     }
// }

// /// Spawns the score display with a top–left anchored text for the current score and moves remaining.
// ///
// /// This system is triggered upon entering `GameState::Playing`.
// fn spawn_score_display(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn((
//         ScoreDisplay,
//         Text2dBundle {
//             text: Text::from_section(
//                 "Score: 0\nMoves: 0",
//                 TextStyle {
//                     font: asset_server.load(FONT),
//                     font_size: 24.0,
//                     color: Color::WHITE,
//                 },
//             ),
//             // Position the score display.
//             transform: Transform::from_xyz(-350.0, 250.0, 1.0),
//             ..default()
//         },
//         // Align so that (-350, 250) corresponds to the top–left corner of the text.
//         Text2dAlignment {
//             alignment: Vec2::new(0.0, 1.0),
//         },
//     ));
// }

// /// Updates the score display text based on the current game progress.
// ///
// /// This system is only active when the game state is `GameState::Playing`.
// fn update_score_display(
//     mut query: Query<&mut Text, With<ScoreDisplay>>,
//     progress: Res<GameProgress>,
// ) {
//     if let Ok(mut text) = query.get_single_mut() {
//         text.sections[0].value = format!(
//             "Score: {}\nMoves: {}",
//             progress.score, progress.moves_remaining
//         );
//     }
// }

// /// Despawns all entities associated with a given screen marker component.
// ///
// /// This generic system is used to clean up UI entities when exiting a game state.
// fn despawn_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
//     for entity in &query {
//         commands.entity(entity).despawn_recursive();
//     }
// }
