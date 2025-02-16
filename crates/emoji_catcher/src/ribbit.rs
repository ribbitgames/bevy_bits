use bevy::log::info;
use bevy::prelude::NextState;
use bits_helpers::RibbitMessageHandler;
use ribbit_bits::{BitDuration, BitResult};

use crate::core::{GameState, Score};

#[derive(Default, Clone, Copy)]
pub struct EmojiCatcher;

impl RibbitMessageHandler for EmojiCatcher {
    fn restart(world: &mut bevy::prelude::World) {
        info!("Restarting EmojiCatcher");

        // Reset score
        if let Some(mut score) = world.get_resource_mut::<Score>() {
            *score = Score::default();
        }

        // Return to welcome screen
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Welcome);
    }

    fn end(world: &mut bevy::prelude::World) -> BitResult {
        info!("Ending EmojiCatcher");

        // Ensure we're in game over state
        let mut next_state = world.resource_mut::<NextState<GameState>>();
        next_state.set(GameState::GameOver);

        // Return final score
        let score = world.resource::<Score>();
        BitResult::HighestScore(score.0.into())
    }

    fn duration(_world: &mut bevy::prelude::World) -> BitDuration {
        // Game can run indefinitely
        BitDuration::max_duration()
    }
}
