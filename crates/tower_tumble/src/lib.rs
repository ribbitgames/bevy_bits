use bevy::prelude::*;
use bits_helpers::emoji::EmojiPlugin;

use crate::game::GameState;

mod blocks;
mod game;
mod input;
mod physics;
mod ribbit;
mod screen;

use blocks::BlocksPlugin;
use game::GamePlugin;
use input::InputPlugin;
use physics::PhysicsPlugin;
use screen::ScreenPlugin;

pub fn run() {
    bits_helpers::get_default_app::<ribbit::TowerTumble>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .add_plugins(EmojiPlugin)
    .add_plugins(PhysicsPlugin)
    .add_plugins(BlocksPlugin)
    .add_plugins(InputPlugin)
    .add_plugins(GamePlugin)
    .init_state::<GameState>()
    .add_plugins(ScreenPlugin)
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    // Add a 2D camera for the gameplay
    commands.spawn(Camera2d);
}
