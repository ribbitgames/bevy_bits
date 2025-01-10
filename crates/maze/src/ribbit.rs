use bevy::log::info;
use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::GameState;

#[derive(Default, Clone, Copy)]
pub struct Maze;

impl RibbitMessageHandler for Maze {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting Maze");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Game);
    }

    fn end(_world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending Maze");

        // Player did not complete the game
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
