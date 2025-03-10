use bevy::state::state::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::game::{GameDifficulty, GameProgress, GameState, ScoreState};

#[derive(Default, Clone, Copy)]
pub struct EmojiSequencer;

impl RibbitMessageHandler for EmojiSequencer {
    fn restart(world: &mut bevy::prelude::World) {
        world.insert_resource(GameDifficulty::default());
        world.insert_resource(GameProgress::default());
        world.insert_resource(ScoreState::default());

        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        let score = world.resource::<ScoreState>();

        BitResult::HighestScore(score.total_score.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        BitDuration::max_duration()
    }
}
