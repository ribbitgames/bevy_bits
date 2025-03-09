use bevy::log::info;
use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::gameplay::{GameState, ScoreInfo};

#[derive(Default, Clone, Copy)]
pub struct FlappyGun;

impl RibbitMessageHandler for FlappyGun {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting FlappyGun");

        world.insert_resource(ScoreInfo::default());

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Dead);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending FlappyGun");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Dead);

        let score = world.resource::<ScoreInfo>();
        BitResult::HighestScore(score.current_score.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
