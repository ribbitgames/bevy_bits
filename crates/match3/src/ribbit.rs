use bevy::log::info;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

#[derive(Default, Clone, Copy)]
pub struct RibbitMatch3;

impl RibbitMessageHandler for RibbitMatch3 {
    fn restart(_world: &mut bevy::prelude::World) {
        info!("Restarting Match3 : this game has no restart state.");
    }

    fn end(_world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending Match3 : There's no end game.");

        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
