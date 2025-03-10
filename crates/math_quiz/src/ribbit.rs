use bevy::log::info;
use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::{GameData, GameState};

#[derive(Default, Clone, Copy)]
pub struct MathQuiz;

impl RibbitMessageHandler for MathQuiz {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting MathQuiz");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);

        let mut game_data = world.resource_mut::<GameData>();
        *game_data = GameData::default();
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending MathQuiz");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        // Player did not complete the game
        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
