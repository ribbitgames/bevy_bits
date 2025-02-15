use bevy::prelude::*;

use crate::cards::{Card, CardBack, CardFace};

/// Component for flip animation on a card.
/// - `target`: desired final face state (true = face up, false = face down).
/// - `timer`: controls the duration of the flip animation.
/// - `swapped`: flag indicating whether the face/back swap has been done.
#[derive(Component)]
pub struct FlipAnimation {
    pub timer: Timer,
    pub target: bool,
    pub swapped: bool,
}

/// Component for shake animation on a card.
/// - `timer`: duration of the shake effect.
/// - `original_translation`: the card’s original translation.
/// - `amplitude`: maximum offset applied during the shake.
#[derive(Component)]
pub struct ShakeAnimation {
    pub timer: Timer,
    pub original_translation: Vec3,
    pub amplitude: f32,
}

/// Component for click (scale) animation on a card.
/// - `timer`: controls the duration of the scale effect.
/// - `original_scale`: the card’s scale before the effect.
/// - `scale_factor`: the temporary scale multiplier.
#[derive(Component)]
pub struct ClickAnimation {
    pub timer: Timer,
    pub original_scale: Vec3,
    pub scale_factor: f32,
}

/// Plugin that registers animation systems.
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_flip)
            .add_systems(Update, animate_shake)
            .add_systems(Update, animate_click);
    }
}

/// Animates the flip effect by rotating the card along the Y axis.
/// At halfway (progress ≥ 0.5), swaps the child visibilities.
fn animate_flip(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FlipAnimation, &mut Transform, &Children), With<Card>>,
    mut visibility_set: ParamSet<(
        Query<&mut Visibility, With<CardFace>>,
        Query<&mut Visibility, With<CardBack>>,
    )>,
) {
    for (entity, mut flip, mut transform, children) in &mut query {
        flip.timer.tick(time.delta());
        let progress = flip.timer.elapsed_secs() / flip.timer.duration().as_secs_f32();
        let angle = if flip.target {
            progress * std::f32::consts::PI
        } else {
            std::f32::consts::PI * (1.0 - progress)
        };
        transform.rotation = Quat::from_rotation_y(angle);

        // At halfway point, swap child visibilities if not already done.
        if !flip.swapped && progress >= 0.5 {
            for &child in children {
                if let Ok(mut vis) = visibility_set.p0().get_mut(child) {
                    *vis = if flip.target {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
                if let Ok(mut vis) = visibility_set.p1().get_mut(child) {
                    *vis = if flip.target {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    };
                }
            }
            flip.swapped = true;
        }

        if flip.timer.finished() {
            // Finalize rotation and remove the animation component.
            transform.rotation = if flip.target {
                Quat::from_rotation_y(std::f32::consts::PI)
            } else {
                Quat::IDENTITY
            };
            commands.entity(entity).remove::<FlipAnimation>();
        }
    }
}

/// Animates a shake effect by oscillating the card’s translation.
fn animate_shake(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShakeAnimation, &mut Transform)>,
) {
    for (entity, mut shake, mut transform) in &mut query {
        shake.timer.tick(time.delta());
        let progress = shake.timer.elapsed_secs() / shake.timer.duration().as_secs_f32();
        let dampening = 1.0 - progress;
        let offset = shake.amplitude * dampening * (shake.timer.elapsed_secs() * 20.0).sin();
        transform.translation.x = shake.original_translation.x + offset;
        if shake.timer.finished() {
            transform.translation.x = shake.original_translation.x;
            commands.entity(entity).remove::<ShakeAnimation>();
        }
    }
}

/// Animates a click (scale) effect by temporarily increasing and then restoring scale.
fn animate_click(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ClickAnimation, &mut Transform)>,
) {
    for (entity, mut click, mut transform) in &mut query {
        click.timer.tick(time.delta());
        let progress = click.timer.elapsed_secs() / click.timer.duration().as_secs_f32();
        let scale = if progress < 0.5 {
            // Scaling up phase.
            (click.scale_factor - 1.0).mul_add(progress / 0.5, 1.0)
        } else {
            // Scaling back down phase.
            (click.scale_factor - 1.0).mul_add(-((progress - 0.5) / 0.5), click.scale_factor)
        };
        transform.scale = click.original_scale * scale;
        if click.timer.finished() {
            transform.scale = click.original_scale;
            commands.entity(entity).remove::<ClickAnimation>();
        }
    }
}
