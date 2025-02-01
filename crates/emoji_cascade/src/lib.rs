use bevy::prelude::*;
use bits_helpers::emoji::EmojiPlugin;

mod game;
mod grid;
mod ribbit;

use game::GamePlugin;
use grid::GridPlugin;

pub fn run() {
    bits_helpers::get_default_app::<ribbit::EmojiCascade>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .add_plugins(EmojiPlugin)
    .add_plugins(GamePlugin)
    .add_plugins(GridPlugin)
    .init_state::<game::GameState>()
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
