use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiPlugin};
use bits_helpers::floating_score::animate_floating_scores;
use ribbit::EmojiCatcher;

mod core;
mod gameplay;
mod ribbit;
mod welcome;

use core::{GameState, GameTimer, Score, TargetEmojiIndex};

use gameplay::{
    cleanup_game, handle_input, move_emojis, render_circles, spawn_game_elements, update_game,
    update_game_timer,
};
use welcome::{
    add_emoji_to_welcome_screen, despawn_welcome_screen, handle_welcome_input, spawn_welcome_screen,
};

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
        .init_resource::<TargetEmojiIndex>()
        // Add startup systems
        .add_systems(Startup, setup_camera)
        // Welcome state
        .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
        .add_systems(
            Update,
            (
                add_emoji_to_welcome_screen.run_if(emoji_system_ready),
                handle_welcome_input,
            )
                .run_if(in_state(GameState::Welcome)),
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
                update_game_timer,
                animate_floating_scores,
                render_circles,
            )
                .run_if(in_state(GameState::Playing))
                .run_if(emoji_system_ready),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_game);

    app.run();
}

/// Condition system that checks if emoji system is ready
fn emoji_system_ready(validation: Res<AtlasValidation>) -> bool {
    emoji::is_emoji_system_ready(&validation)
}

/// Sets up the main 2D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
