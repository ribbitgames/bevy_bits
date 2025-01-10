use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct CleanupMarker;

pub trait Restartable: Resource {
    fn reset(&mut self);
    fn initial_state() -> Self::State;
    type State: States + FreelyMutableState;
}

pub fn handle_restart<T: Restartable>(
    mut next_state: ResMut<NextState<T::State>>,
    mut restartable: ResMut<T>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            restartable.reset();
            next_state.set(T::initial_state());
        }
    }
}

pub fn cleanup_marked_entities(mut commands: Commands, query: Query<Entity, With<CleanupMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
