use bevy::log::info;
use bevy::prelude::*;
use bevy::state::state::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::cards::Card;
use crate::game::{GameDifficulty, GameProgress, GameState};

#[derive(Default, Clone, Copy)]
pub struct Memoji;

impl RibbitMessageHandler for Memoji {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting Memoji");

        world.insert_resource(GameDifficulty::default());
        world.insert_resource(GameProgress::default());

        let cards = world
            .query_filtered::<Entity, With<Card>>()
            .iter(world)
            .collect::<Vec<_>>();

        for card in cards {
            world.entity_mut(card).despawn_recursive();
        }

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending Memoji");

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        BitResult::Failure
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
