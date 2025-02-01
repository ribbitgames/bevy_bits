use bevy::log::info;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

#[derive(Default, Clone, Copy)]
pub struct EmojiAvoidance;

impl RibbitMessageHandler for EmojiAvoidance {
    fn restart(_world: &mut bevy::prelude::World) {
        info!("Restarting EmojiAvoidance");
    }

    fn end(_world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending EmojiAvoidance");
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
