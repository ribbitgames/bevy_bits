#![allow(clippy::type_complexity)]

mod audio;
mod gameplay;
mod player;
mod ribbit;
mod scene;
mod ui;

use avian3d::prelude::*;
use ribbit::FlappyGun;

pub fn run() {
    bits_helpers::get_default_app::<FlappyGun>(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default()) // Activate when you need to debug physics
        .add_plugins(player::PlayerPlugin)
        .add_plugins(scene::ScenePlugin)
        .add_plugins(gameplay::StateTransitionPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(audio::GameAudioPlugin)
        .run();
}
