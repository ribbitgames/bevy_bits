use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::gameplay::{GameState, JumpedEvent};
use crate::player::inputs::Action;
use crate::player::PlayerSettings;

pub fn jump(
    mut query: Query<(&ActionState<Action>, &mut LinearVelocity)>,
    player_settings: Res<PlayerSettings>,
    touch_input: Res<Touches>,
    mut jumped_event: EventWriter<JumpedEvent>,
) {
    let (action_state, mut velocity) = query.single_mut();

    // Leafwing Input Manager doesn't support touch input, so we need to check for it here
    if action_state.just_pressed(&Action::Jump) || touch_input.any_just_pressed() {
        velocity.y = player_settings.jump_velocity;
        jumped_event.send(JumpedEvent);
    }
}

pub fn check_for_game_start(
    query: Query<&ActionState<Action>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let action_state = query.single();

    // Leafwing Input Manager doesn't support touch input, so we need to check for it here
    if action_state.just_pressed(&Action::Jump) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}
