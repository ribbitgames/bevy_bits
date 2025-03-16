use avian2d::prelude::*;
use bevy::prelude::*;

use crate::core::config;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Gravity(Vec2::new(0.0, -config::GRAVITY)))
            .add_plugins(PhysicsPlugins::default());
    }
}
