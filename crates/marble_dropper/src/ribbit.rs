use bevy::log::info;
use bevy::prelude::{NextState, World};
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::core::{GameState, Score};

#[derive(Default, Clone, Copy)]
pub struct MarbleDropper;

impl RibbitMessageHandler for MarbleDropper {
    fn restart(world: &mut World) {
        info!("Restarting Marble Dropper");
        if let Some(mut score) = world.get_resource_mut::<Score>() {
            *score = Score::default();
        }
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut World) -> BitResult {
        info!("Ending Marble Dropper");
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);
        let score = world.resource::<Score>();
        BitResult::HighestScore(score.0.into())
    }

    fn duration(_world: &mut World) -> BitDuration {
        BitDuration::max_duration() // Game runs until player fails
    }
}