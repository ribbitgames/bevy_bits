use bevy::prelude::*;
use bits_helpers::input::just_pressed_world_position;

use crate::cards::Card;
use crate::game::{FlipState, GameProgress, GameState};

#[derive(Resource, Default)]
pub struct InputState {
    pub _enabled: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>().add_systems(
            Update,
            handle_card_clicks.run_if(in_state(GameState::Playing)),
        );
    }
}

fn handle_card_clicks(
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut cards: Query<(Entity, &Transform, &GlobalTransform, &mut Card)>,
    mut flip_state: ResMut<FlipState>,
    game_progress: Res<GameProgress>,
) {
    if game_progress.is_interaction_blocked() || flip_state.unmatch_timer.is_some() {
        return;
    }

    let Some(world_position) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    else {
        return;
    };

    for (entity, _local_transform, global_transform, mut card) in &mut cards {
        let distance = world_position.distance(global_transform.translation().truncate());

        if distance < 35.0 && !card.face_up && !card.locked && flip_state.face_up_cards.len() < 2 {
            card.face_up = true;
            flip_state.face_up_cards.push(entity);
            break;
        }
    }
}
