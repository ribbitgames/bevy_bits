use bevy::prelude::*;
use rand::prelude::*;

use crate::game::{GameState, SequenceState, StageState};

/// Particle effect for successful sequence completion
#[derive(Component)]
pub struct CelebrationParticle {
    /// Lifetime of the particle in seconds
    lifetime: Timer,
    /// Velocity vector for particle movement
    velocity: Vec2,
    /// Starting scale for size animation
    initial_scale: f32,
}

/// Particle effect for sequence feedback
#[derive(Component)]
pub struct FeedbackParticle {
    /// Lifetime of the particle in seconds
    lifetime: Timer,
    /// Velocity vector for particle movement
    velocity: Vec2,
    /// Whether this particle is for correct (true) or incorrect (false) feedback
    is_correct: bool,
}

/// Resource to control celebration effects
#[derive(Resource, Default)]
pub struct CelebrationState {
    /// Whether celebration is currently active
    pub is_celebrating: bool,
    /// Timer for transitioning after celebration
    pub transition_timer: Option<Timer>,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CelebrationState>().add_systems(
            Update,
            (
                spawn_celebration_particles,
                update_celebration_particles,
                spawn_sequence_feedback,
                update_feedback_particles,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

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

        let mut rng = rand::thread_rng();

        // Spawn burst of celebratory particles
        for _ in 0..100 {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(100.0..300.0);
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
            let offset = Vec2::new(rng.gen_range(-150.0..150.0), rng.gen_range(-150.0..150.0));

            commands.spawn((
                CelebrationParticle {
                    lifetime: Timer::from_seconds(1.5, TimerMode::Once),
                    velocity,
                    initial_scale: rng.gen_range(0.5..2.0),
                },
                Sprite {
                    color: Color::hsla(rng.gen_range(0.0..360.0), 0.8, 0.8, 1.0),
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

fn spawn_sequence_feedback(
    mut commands: Commands,
    sequence_state: Res<SequenceState>,
    cards: Query<(&Transform, &Children)>,
) {
    // Only spawn feedback when sequence is complete
    if sequence_state.player_sequence.len() != sequence_state.target_sequence.len() {
        return;
    }

    let mut rng = rand::thread_rng();

    // Spawn feedback particles for each card in sequence
    for (i, &player_idx) in sequence_state.player_sequence.iter().enumerate() {
        let is_correct = sequence_state.target_sequence.get(i) == Some(&player_idx);

        // Find card position for this emoji
        for (transform, _) in &cards {
            let pos = transform.translation.truncate();

            // Spawn particles around the card
            for _ in 0..10 {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let speed = rng.gen_range(50.0..150.0);
                let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
                let offset = Vec2::new(rng.gen_range(-20.0..20.0), rng.gen_range(-20.0..20.0));

                commands.spawn((
                    FeedbackParticle {
                        lifetime: Timer::from_seconds(0.75, TimerMode::Once),
                        velocity,
                        is_correct,
                    },
                    Sprite {
                        color: if is_correct {
                            Color::rgb(0.0, 0.8, 0.0)
                        } else {
                            Color::rgb(0.8, 0.0, 0.0)
                        },
                        custom_size: Some(Vec2::splat(5.0)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x + offset.x, pos.y + offset.y, 5.0),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));
            }
        }
    }
}

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
        particle.lifetime.tick(time.delta());

        let delta = particle.velocity * time.delta_secs();
        transform.translation += Vec3::new(delta.x, delta.y, 0.0);

        let life_factor = 1.0 - particle.lifetime.fraction();
        sprite.color = sprite.color.with_alpha(life_factor);
        transform.scale = Vec3::splat(particle.initial_scale * life_factor);

        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_feedback_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut Sprite, &mut FeedbackParticle)>,
) {
    for (entity, mut transform, mut sprite, mut particle) in &mut particles {
        particle.lifetime.tick(time.delta());

        let delta = particle.velocity * time.delta_secs();
        transform.translation += Vec3::new(delta.x, delta.y, 0.0);

        let life_factor = 1.0 - particle.lifetime.fraction();
        sprite.color = sprite.color.with_alpha(life_factor);

        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
