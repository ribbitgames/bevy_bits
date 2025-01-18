use bevy::prelude::*;
use bits_helpers::emoji::EmojiPlugin;

mod cards;
mod input;
mod ribbit;

use cards::CardPlugin;
use input::InputPlugin;

pub fn run() {
    bits_helpers::get_default_app::<ribbit::Memoji>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .add_plugins(EmojiPlugin)
    .add_plugins(CardPlugin)
    .add_plugins(InputPlugin) // Include the InputPlugin
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
