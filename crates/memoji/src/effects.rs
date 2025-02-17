use bevy::prelude::*;

use crate::game::{GameState, StageState};

/// Component to mark celebration particle entities
#[derive(Component)]
pub struct CelebrationParticle {
    /// Lifetime of the particle in seconds
    lifetime: Timer,
    /// Velocity vector for particle movement
    velocity: Vec2,
    /// Starting scale for size animation
    initial_scale: f32,
}

/// Resource to control the celebration effect
#[derive(Resource, Default)]
pub struct CelebrationState {
    /// Whether celebration is currently active
    pub is_celebrating: bool,
    /// How long to show celebration before next stage
    pub transition_timer: Option<Timer>,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CelebrationState>().add_systems(
            Update,
            (spawn_celebration_particles, update_celebration_particles)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Spawns celebration particles when stage is completed
fn spawn_celebration_particles(
    mut commands: Commands,
    mut celebration_state: ResMut<CelebrationState>,
    stage_state: Res<StageState>,
    time: Res<Time>,
) {
    // Start celebration when stage is complete
    if stage_state.stage_complete && !celebration_state.is_celebrating {
        celebration_state.is_celebrating = true;
        celebration_state.transition_timer = Some(Timer::from_seconds(2.0, TimerMode::Once));

        // Spawn more particles for a more noticeable effect
        for _ in 0..100 {
            let angle = fastrand::f32() * std::f32::consts::TAU;
            let speed = fastrand::f32().mul_add(200.0, 100.0);
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

            // Randomize starting positions around the center
            let offset = Vec2::new(
                fastrand::f32().mul_add(300.0, -150.0),
                fastrand::f32().mul_add(300.0, -150.0),
            );

            commands.spawn((
                CelebrationParticle {
                    lifetime: Timer::from_seconds(1.5, TimerMode::Once),
                    velocity,
                    initial_scale: fastrand::f32().mul_add(1.5, 0.5),
                },
                Sprite {
                    color: Color::hsla(fastrand::f32() * 360.0, 0.8, 0.8, 1.0),
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                Transform::from_xyz(offset.x, offset.y, 10.0),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
    }

    // Handle transition timer
    if let Some(timer) = &mut celebration_state.transition_timer {
        if timer.tick(time.delta()).just_finished() {
            celebration_state.is_celebrating = false;
            celebration_state.transition_timer = None;
        }
    }
}

/// Updates celebration particle positions and lifetimes
fn update_celebration_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(
        Entity,
        &mut Transform,
        &mut Sprite,
        &mut CelebrationParticle,
    )>,
) {
    for (entity, mut transform, mut sprite, mut particle) in &mut particles {
        // Tick the particle's timer using the current delta time.
        particle.lifetime.tick(time.delta());

        // Update position based on velocity.
        let delta = particle.velocity * time.delta_secs();
        transform.translation += Vec3::new(delta.x, delta.y, 0.0);

        // Fade out and scale down based on remaining lifetime.
        let life_factor = 1.0 - particle.lifetime.fraction();
        sprite.color = sprite.color.with_alpha(life_factor);
        transform.scale = Vec3::splat(particle.initial_scale * life_factor);

        // Remove particle when lifetime is over.
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
