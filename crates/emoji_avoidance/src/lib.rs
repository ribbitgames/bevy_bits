use bevy::prelude::*;
use bits_helpers::emoji::EmojiPlugin;

mod game;
mod obstacles;
mod player;
mod ribbit;

use game::GamePlugin;
use obstacles::ObstaclesPlugin;
use player::PlayerPlugin;

pub fn run() {
    bits_helpers::get_default_app::<ribbit::EmojiAvoidance>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .add_plugins(EmojiPlugin)
    .add_plugins(GamePlugin)
    .add_plugins(ObstaclesPlugin)
    .add_plugins(PlayerPlugin)
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
