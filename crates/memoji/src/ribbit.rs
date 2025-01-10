use bevy::log::info;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

#[derive(Default, Clone, Copy)]
pub struct Memoji;

impl RibbitMessageHandler for Memoji {
    fn restart(_world: &mut bevy::prelude::World) {
        info!("Restarting Memoji");
    }

    fn end(_world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending Memoji");
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        // Adding 3 seconds for the splash screen.
        BitDuration::max_duration()
    }
}
