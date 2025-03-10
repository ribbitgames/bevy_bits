use bevy::prelude::*;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::{GameProgress, GameState};

#[derive(Default, Clone, Copy)]
pub struct WhackAMole;

impl RibbitMessageHandler for WhackAMole {
    fn restart(world: &mut bevy::prelude::World) {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Reset);
    }

    fn end(world: &mut World) -> BitResult {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Result);

        let progress = world.resource::<GameProgress>();
        BitResult::HighestScore(progress.score.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
