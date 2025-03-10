use bevy::log::info;
use bevy::state::state::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::game::GameState;

#[derive(Default, Clone, Copy)]
pub struct EmojiAvoidance;

impl RibbitMessageHandler for EmojiAvoidance {
    fn restart(world: &mut bevy::prelude::World) {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);

        info!("Restarting EmojiAvoidance");
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);
        info!("Ending EmojiAvoidance");
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
