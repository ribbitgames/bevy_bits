use core::time::Duration;

use bevy::prelude::*;

use crate::FONT;

#[derive(Component)]
pub struct FloatingScore {
    timer: Timer,
    initial_position: Vec2,
}

pub fn spawn_floating_score(
    commands: &mut Commands,
    position: Vec2,
    text: &str,
    color: Srgba,
    asset_server: &Res<AssetServer>,
) {
    commands.spawn((
        Text::new(text),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::Srgba(color)),
        Node {
            position_type: PositionType::Relative,
            left: Val::Px(position.x + 20.0),
            top: Val::Px(position.y),
            ..default()
        },
        FloatingScore {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
            initial_position: position,
        },
    ));
}

pub fn animate_floating_scores(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut FloatingScore)>,
) {
    for (entity, mut transform, mut floating_score) in &mut query {
        floating_score.timer.tick(time.delta());
        let progress = floating_score.timer.fraction();

        // Move upwards and fade out
        transform.translation.y = 50.0f32.mul_add(progress, floating_score.initial_position.y);
        transform.scale = Vec3::splat(1.0 - progress);

        if floating_score.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
