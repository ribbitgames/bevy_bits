use bevy::prelude::*;
use bits_helpers::emoji::EmojiPlugin;
use bits_helpers::floating_score::animate_floating_scores;
use ribbit::ShapeFinder;

mod core;
mod gameplay;
mod ribbit;
mod stage;
mod welcome;

use core::*;

use crate::gameplay::*;
use crate::stage::*;
use crate::welcome::*;

/// Entry point for the game
pub fn run() {
    let mut app = bits_helpers::get_default_app::<ShapeFinder>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    app.add_plugins(EmojiPlugin)
        // Initialize core resources
        .init_state::<GameState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        .init_resource::<TargetEmojiInfo>()
        .init_resource::<CorrectEmojisFound>()
        .init_resource::<StageConfig>()
        .add_event::<EmojiClickedEvent>()
        // Add systems by state
        .add_systems(Startup, setup_camera)
        // Welcome state
        .add_systems(
            Update,
            try_spawn_welcome_screen.run_if(in_state(GameState::Welcome)),
        )
        .add_systems(
            Update,
            handle_welcome_input.run_if(in_state(GameState::Welcome)),
        )
        .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
        // Playing state
        .add_systems(OnEnter(GameState::Playing), (spawn_emojis, spawn_timer))
        .add_systems(
            Update,
            (
                handle_playing_input,
                move_emojis,
                update_timer,
                handle_emoji_clicked,
                animate_floating_scores,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_playing_state)
        // Stage complete state
        .add_systems(
            OnEnter(GameState::StageComplete),
            spawn_stage_complete_screen,
        )
        .add_systems(
            Update,
            handle_stage_complete_input.run_if(in_state(GameState::StageComplete)),
        )
        .add_systems(OnExit(GameState::StageComplete), cleanup_stage_complete);

    app.run();
}

/// Sets up the main 2D camera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
