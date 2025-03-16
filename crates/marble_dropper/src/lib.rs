use bevy::prelude::*;
use bits_helpers::emoji::EmojiPlugin;

pub mod core;
pub mod gameplay;
pub mod physics;
pub mod ribbit;
pub mod welcome;

use core::{GameState, GameTimer, Score};

use gameplay::{
    cleanup_game, handle_input, move_platforms, render_circles, spawn_game_elements, update_game,
    update_game_timer,
};
use physics::PhysicsPlugin;
use ribbit::MarbleDropper;
use welcome::{despawn_welcome_screen, handle_welcome_input, spawn_welcome_screen};

pub fn run() {
    let mut app = bits_helpers::get_default_app::<MarbleDropper>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    app.add_plugins(EmojiPlugin)
        .add_plugins(PhysicsPlugin)
        .init_state::<GameState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        .add_systems(Startup, setup_camera)
        // Welcome state
        .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
        .add_systems(
            Update,
            handle_welcome_input.run_if(in_state(GameState::Welcome)),
        )
        .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
        // Playing state
        .add_systems(OnEnter(GameState::Playing), spawn_game_elements)
        .add_systems(
            Update,
            (handle_input, move_platforms, update_game, update_game_timer)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            render_circles
                .run_if(in_state(GameState::Playing))
                .after(move_platforms)
                .after(update_game),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_game);
    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
