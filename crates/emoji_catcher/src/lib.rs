use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiPlugin};
use bits_helpers::floating_score::animate_floating_scores;
use ribbit::EmojiCatcher;

mod core;
mod game_over;
mod gameplay;
mod ribbit;
mod welcome;

use core::{GameState, GameTimer, Score};

use game_over::{cleanup_game_over, handle_game_over_input, spawn_game_over_screen};
use gameplay::{cleanup_game, handle_input, move_emojis, spawn_game_elements, update_game};
use welcome::{despawn_welcome_screen, handle_welcome_input, spawn_welcome_screen};

/// Condition system that checks if emoji system is ready
fn emoji_system_ready(validation: Res<AtlasValidation>) -> bool {
    emoji::is_emoji_system_ready(&validation)
}

/// Entry point for the game
pub fn run() {
    let mut app = bits_helpers::get_default_app::<EmojiCatcher>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    app.add_plugins(EmojiPlugin)
        // Initialize core resources
        .init_state::<GameState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        // Add startup systems
        .add_systems(Startup, setup_camera)
        // Welcome state
        .add_systems(
            OnEnter(GameState::Welcome),
            spawn_welcome_screen.run_if(emoji_system_ready),
        )
        .add_systems(
            Update,
            handle_welcome_input
                .run_if(in_state(GameState::Welcome))
                .run_if(emoji_system_ready),
        )
        .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
        // Playing state
        .add_systems(
            OnEnter(GameState::Playing),
            spawn_game_elements.run_if(emoji_system_ready),
        )
        .add_systems(
            Update,
            (
                handle_input,
                move_emojis,
                update_game,
                animate_floating_scores,
            )
                .run_if(in_state(GameState::Playing))
                .run_if(emoji_system_ready),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_game)
        // Game over state
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        .add_systems(
            Update,
            handle_game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over);

    app.run();
}

/// Sets up the main 2D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
