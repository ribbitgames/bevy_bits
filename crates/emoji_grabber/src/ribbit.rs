use bevy::log::info;
use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::{GameState, Score};

#[derive(Default, Clone, Copy)]
pub struct EmojiGrabber;

impl RibbitMessageHandler for EmojiGrabber {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting EmojiGrabber");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending EmojiGrabber");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        let score = world.resource::<Score>();

        BitResult::HighestScore(score.0.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
