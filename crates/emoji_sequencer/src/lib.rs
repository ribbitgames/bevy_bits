use bevy::prelude::*;
use bits_helpers::emoji::EmojiPlugin;

use crate::game::GameState;

mod animations;
mod cards;
mod effects;
mod game;
mod input;
mod ribbit;
mod screen;
mod variables;

use animations::AnimationPlugin;
use cards::CardPlugin;
use effects::EffectsPlugin;
use game::GamePlugin;
use input::InputPlugin;
use screen::ScreenPlugin;
use variables::GameVariablesPlugin;

pub fn run() {
    bits_helpers::get_default_app::<ribbit::EmojiSequencer>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .add_plugins(EmojiPlugin)
    .add_plugins(GameVariablesPlugin)
    .add_plugins(CardPlugin)
    .add_plugins(InputPlugin)
    .add_plugins(GamePlugin)
    .add_plugins(EffectsPlugin)
    .add_plugins(AnimationPlugin)
    .init_state::<GameState>()
    .add_plugins(ScreenPlugin)
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
