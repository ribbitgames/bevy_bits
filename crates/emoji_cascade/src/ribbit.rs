use bevy::log::info;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

#[derive(Default, Clone, Copy)]
pub struct EmojiCascade;

impl RibbitMessageHandler for EmojiCascade {
    fn restart(_world: &mut bevy::prelude::World) {
        info!("Restarting EmojiCascade");
    }

    fn end(_world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending EmojiCascade");
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
