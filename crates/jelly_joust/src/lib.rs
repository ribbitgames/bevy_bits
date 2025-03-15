use bevy::prelude::*;
use bits_helpers::emoji::EmojiPlugin;

use crate::game::GameState;

mod game;
mod ribbit;

pub fn run() {
    bits_helpers::get_default_app::<ribbit::JellyJoust>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .add_plugins(EmojiPlugin)
    .add_plugins(game::GamePlugin)
    .init_state::<GameState>()
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
