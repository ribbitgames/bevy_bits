use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::gameplay::{GameState, JumpedEvent};
use crate::player::inputs::Action;
use crate::player::PlayerSettings;

pub fn jump(
    mut query: Query<(&ActionState<Action>, &mut LinearVelocity)>,
    player_settings: Res<PlayerSettings>,
    mut jumped_event: EventWriter<JumpedEvent>,
) {
    let (action_state, mut velocity) = query.single_mut();

    if action_state.just_pressed(&Action::Jump) {
        velocity.y = player_settings.jump_velocity;
        jumped_event.send(JumpedEvent);
    }
}

pub fn check_for_game_start(
    query: Query<&ActionState<Action>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let action_state = query.single();

    if action_state.just_pressed(&Action::Jump) {
        next_state.set(GameState::Playing);
    }
}
