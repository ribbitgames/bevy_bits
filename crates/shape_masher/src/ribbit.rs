use std::time::Duration;

use bevy::log::info;
use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::{GameState, Score, GAME_DURATION};

#[derive(Default, Clone, Copy)]
pub struct ShapeMasher;

impl RibbitMessageHandler for ShapeMasher {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting ShapeMasher");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending ShapeMasher");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        let score = world.resource::<Score>();

        BitResult::HighestScore(score.0.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        // Adding 3 seconds for the splash screen.
        BitDuration::new(Duration::from_secs_f32(GAME_DURATION) + Duration::from_secs(3))
    }
}
