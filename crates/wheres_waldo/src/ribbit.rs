use bevy::log::info;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::ClearPuzzle;

#[derive(Default, Clone, Copy)]
pub struct WheresWaldo;

impl RibbitMessageHandler for WheresWaldo {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting WheresWaldo");

        let mut event_writer = world.resource_mut::<bevy::ecs::event::Events<ClearPuzzle>>();
        event_writer.send(ClearPuzzle);
    }

    fn end(_world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending WheresWaldo");

        // Player did not complete the game
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
