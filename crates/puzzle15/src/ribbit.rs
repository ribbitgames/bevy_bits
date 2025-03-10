use bevy::log::info;
use bevy::prelude::*;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::GameState;

#[derive(Default, Clone, Copy)]
pub struct Puzzle15;

impl RibbitMessageHandler for Puzzle15 {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting Puzzle15");

        let mut next_state: Mut<'_, NextState<GameState>> =
            world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Reset);
    }

    fn end(world: &mut World) -> BitResult {
        info!("Ending Puzzle15");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Result);

        // Player did not complete the game
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
