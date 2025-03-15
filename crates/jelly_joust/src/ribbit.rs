use bevy::prelude::*;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::game::{GameState, Score};

#[derive(Default, Resource)]
pub struct JellyJoust;

impl RibbitMessageHandler for JellyJoust {
    fn restart(world: &mut World) {
        info!("Restarting Jelly Joust");

        world.insert_resource(Score::default());
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut World) -> BitResult {
        info!("Ending Jelly Joust");
        let score = world.resource::<Score>().player;
        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::GameOver);
        BitResult::HighestScore(score.into())
    }

    fn duration(_world: &mut World) -> BitDuration {
        BitDuration::max_duration()
    }
}
