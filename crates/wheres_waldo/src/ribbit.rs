use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::GameState;

#[derive(Default, Clone, Copy)]
pub struct WheresWaldo;

impl RibbitMessageHandler for WheresWaldo {
    fn restart(world: &mut bevy::prelude::World) {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Reset);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Result);

        // Player did not complete the game
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
