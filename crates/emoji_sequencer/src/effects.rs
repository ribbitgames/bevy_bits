use bevy::prelude::*;
use rand::prelude::*;
use rand::rng;

use crate::game::{GameDifficulty, GameState, SequenceState, StageState};

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
}

/// New component for an encircling ring effect that highlights the grid
#[derive(Component)]
pub struct EncirclingRingEffect {
    /// Timer for how long the ring effect lasts
    timer: Timer,
    /// Initial scale of the ring effect
    initial_scale: f32,
    /// Target scale to reach by the end of the effect
    target_scale: f32,
    /// Rotation speed in radians per second
    rotation_speed: f32,
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
        app.init_resource::<CelebrationState>()
            .add_systems(
                Update,
                (
                    spawn_celebration_particles,
                    update_celebration_particles,
                    spawn_sequence_feedback,
                    update_feedback_particles,
                    spawn_encircling_ring_effect,
                    update_encircling_ring_effect,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // When leaving the Playing state, clean up all lingering effects.
            .add_systems(OnExit(GameState::Playing), cleanup_effects);
    }
}

/// Spawns celebration particles when a stage is successfully completed.
/// Reduced the spawn count from 100 to 30 to tone down the effect.
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

        let mut rng = rng();

        // Spawn a reduced burst of celebratory particles (30 instead of 100)
        for _ in 0..30 {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = rng.random_range(100.0..300.0);
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
            let offset = Vec2::new(
                rng.random_range(-150.0..150.0),
                rng.random_range(-150.0..150.0),
            );

            commands.spawn((
                CelebrationParticle {
                    lifetime: Timer::from_seconds(1.5, TimerMode::Once),
                    velocity,
                    initial_scale: rng.random_range(0.5..2.0),
                },
                Sprite {
                    color: Color::hsla(rng.random_range(0.0..360.0), 0.8, 0.8, 1.0),
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

/// Spawns feedback particles for each card in the sequence.
fn spawn_sequence_feedback(
    mut commands: Commands,
    sequence_state: Res<SequenceState>,
    cards: Query<(&Transform, &Children)>,
) {
    // Only spawn feedback when the player's sequence is complete
    if sequence_state.player_sequence.len() != sequence_state.target_sequence.len() {
        return;
    }

    let mut rng = rng();

    // Spawn feedback particles for each card in the sequence
    for (i, &player_idx) in sequence_state.player_sequence.iter().enumerate() {
        let is_correct = sequence_state.target_sequence.get(i) == Some(&player_idx);

        // Find card position for this emoji
        for (transform, _) in &cards {
            let pos = transform.translation.truncate();

            // Spawn particles around the card
            for _ in 0..10 {
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let speed = rng.random_range(50.0..150.0);
                let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
                let offset =
                    Vec2::new(rng.random_range(-20.0..20.0), rng.random_range(-20.0..20.0));

                commands.spawn((
                    FeedbackParticle {
                        lifetime: Timer::from_seconds(0.75, TimerMode::Once),
                        velocity,
                    },
                    Sprite {
                        color: if is_correct {
                            Color::srgb(0.0, 0.8, 0.0)
                        } else {
                            Color::srgb(0.8, 0.0, 0.0)
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

/// Updates celebration particles: moves, fades, scales, and despawns them when lifetime is over.
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

/// Updates feedback particles: moves, fades, and despawns them when lifetime is over.
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

/// Spawns an encircling ring effect behind the grid when the stage is complete.
/// The ring expands, rotates, and fades out over its duration.
fn spawn_encircling_ring_effect(
    mut commands: Commands,
    stage_state: Res<StageState>,
    difficulty: Res<GameDifficulty>,
    ring_query: Query<&EncirclingRingEffect>,
) {
    // Only spawn if the stage is complete and no ring effect exists
    if stage_state.stage_complete && ring_query.is_empty() {
        // Compute grid bounds based on difficulty parameters
        let grid_width = difficulty.grid_cols as f32 * difficulty.grid_spacing;
        let grid_height = difficulty.grid_rows as f32 * difficulty.grid_spacing;
        // The grid is centered at (0,0)
        let center = Vec2::ZERO;
        // Compute a radius based on half the diagonal plus a margin (e.g., 30.0)
        let radius = (grid_width.hypot(grid_height) / 2.0) + 30.0;

        // Spawn the ring effect entity
        commands.spawn((
            EncirclingRingEffect {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
                // Assuming the base sprite size is 50.0; adjust scales accordingly.
                initial_scale: radius / 50.0,
                target_scale: (radius * 1.5) / 50.0,
                rotation_speed: 1.0, // radians per second
            },
            Sprite {
                // If you have a ring texture, you can use it here.
                // For now, we use a simple white color with some transparency.
                color: Color::srgba(1.0, 1.0, 1.0, 0.8),
                custom_size: Some(Vec2::splat(50.0)),
                ..default()
            },
            Transform {
                // Position the ring at the grid center; use a z value behind the cards.
                translation: Vec3::new(center.x, center.y, -1.0),
                scale: Vec3::splat(1.0),
                ..default()
            },
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
    }
}

/// Updates the encircling ring effect by interpolating its scale, rotating it,
/// and fading out its opacity over time.
fn update_encircling_ring_effect(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut EncirclingRingEffect,
        &mut Transform,
        &mut Sprite,
    )>,
) {
    for (entity, mut ring, mut transform, mut sprite) in &mut query {
        ring.timer.tick(time.delta());
        let progress = ring.timer.elapsed_secs() / ring.timer.duration().as_secs_f32();
        // Lerp the scale between initial and target values.
        let scale_factor =
            (ring.target_scale - ring.initial_scale).mul_add(progress, ring.initial_scale);
        transform.scale = Vec3::splat(scale_factor);
        // Apply continuous rotation.
        transform.rotation *= Quat::from_rotation_z(ring.rotation_speed * time.delta_secs());
        // Fade out the ring (reduce alpha) over time.
        let alpha = 1.0 - progress;
        sprite.color = sprite.color.with_alpha(alpha);

        // Despawn the ring effect when its timer finishes.
        if ring.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Cleanup system that removes all lingering effects (celebration, feedback, and ring)
/// when transitioning out of the Playing state.
fn cleanup_effects(
    mut commands: Commands,
    effects: Query<
        Entity,
        Or<(
            With<CelebrationParticle>,
            With<FeedbackParticle>,
            With<EncirclingRingEffect>,
        )>,
    >,
) {
    for entity in effects.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
