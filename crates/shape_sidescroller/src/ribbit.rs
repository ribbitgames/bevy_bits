use bevy::log::info;
use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::{GameData, GameState};

#[derive(Default, Clone, Copy)]
pub struct ShapeSideScroller;

impl RibbitMessageHandler for ShapeSideScroller {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting ShapeSideScroller");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending ShapeSideScroller");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        let game_data = world.resource::<GameData>();

        BitResult::HighestScore(game_data.score.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
