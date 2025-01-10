use bevy::log::info;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::ResetGame;

#[derive(Default, Clone, Copy)]
pub struct MasterMind;

// This trait implements  is used by Ribbit to control
impl RibbitMessageHandler for MasterMind {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting MasterMind");

        let mut event_writer = world.resource_mut::<bevy::ecs::event::Events<ResetGame>>();
        event_writer.send(ResetGame);
    }

    // Nothing to do other than return the failure state.
    fn end(_world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending MasterMind");

        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
