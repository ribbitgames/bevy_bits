use bevy::log::info;
use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::{GameState, GameTimer};

#[derive(Default, Clone, Copy)]
pub struct ShapeRacer;

impl RibbitMessageHandler for ShapeRacer {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting ShapeRacer");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending ShapeRacer");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        let timer = world.resource::<GameTimer>();

        BitResult::LongestDuration(timer.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
