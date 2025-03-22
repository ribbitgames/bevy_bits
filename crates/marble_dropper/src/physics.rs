use avian2d::prelude::*;
use bevy::prelude::*;

use crate::core::{GameState, config};
use crate::gameplay::handle_marble_bucket_collisions;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0.0, -config::GRAVITY)))
            .add_plugins(PhysicsPlugins::default())
            .add_systems(
                Update,
                handle_marble_bucket_collisions.run_if(in_state(GameState::Playing)),
            );
    }
}
