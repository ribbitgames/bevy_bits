use bevy::prelude::*;

mod components;
mod input;
mod matching;
mod sliding;
mod spawning;

pub use components::*;
use input::handle_input;
use matching::{check_matches, handle_cascading};
use sliding::update_sliding;
use spawning::spawn_grid;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridState>().add_systems(
            Update,
            (
                spawn_grid,
                handle_input,
                update_sliding,
                check_matches,
                handle_cascading,
            )
                .chain()
                .run_if(in_state(crate::game::GameState::Playing)),
        );
    }
}
