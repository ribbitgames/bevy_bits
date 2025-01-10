use bevy::log::info;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

#[derive(Default, Clone, Copy)]
pub struct BitTemplate;

impl RibbitMessageHandler for BitTemplate {
    fn restart(_world: &mut bevy::prelude::World) {
        info!("Restarting BitTemplate");
    }

    fn end(_world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending BitTemplate");
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        // Adding 3 seconds for the splash screen.
        BitDuration::max_duration()
    }
}
