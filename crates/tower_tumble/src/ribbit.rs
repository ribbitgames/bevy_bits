use bevy::state::state::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::game::{GameProgress, GameState};

#[derive(Default, Clone, Copy)]
pub struct TowerTumble;

impl RibbitMessageHandler for TowerTumble {
    fn restart(world: &mut bevy::prelude::World) {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);

        let mut progress = world.resource_mut::<GameProgress>();
        *progress = GameProgress::default();
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        // Return final score
        let progress = world.resource::<GameProgress>();
        BitResult::HighestScore(progress.score.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        // Adding 3 seconds for the splash screen.
        BitDuration::max_duration()
    }
}
