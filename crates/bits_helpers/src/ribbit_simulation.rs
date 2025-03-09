use std::time::Duration;

use bevy::prelude::*;
use ribbit_bits::{BitDuration, BitMessage, RibbitMessage};

use crate::{BIT_MESSAGE_QUEUE, RIBBIT_MESSAGE_QUEUE, WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct RibbitSimulation;

impl Plugin for RibbitSimulation {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ribbit_simulation_init);
        app.add_systems(Update, ribbit_simulation);
        app.add_systems(Update, update_timer_display);
    }
}

/// Component tag for the timer progress bar
#[derive(Component)]
pub struct TimerProgressBar;

/// Global game timer
#[derive(Resource)]
pub struct GameTimer {
    pub timer: Timer,
    pub initial_duration: Duration,
}

fn ribbit_simulation_init(mut commands: Commands) {
    RIBBIT_MESSAGE_QUEUE.lock().push(RibbitMessage::Parameters);

    let duration = BitDuration::max_duration().get_duration();
    commands.insert_resource(GameTimer {
        timer: Timer::new(duration, TimerMode::Once),
        initial_duration: duration,
    });
}

fn update_timer_display(
    mut progress_bar_query: Query<&mut Transform, With<TimerProgressBar>>,
    mut game_timer: ResMut<GameTimer>,
    time: Res<Time>,
) {
    // Tick the game timer
    game_timer.timer.tick(time.delta());

    // Update the progress bar width based on remaining time
    if let Some(mut transform) = progress_bar_query.iter_mut().next() {
        let progress =
            game_timer.timer.remaining_secs() / game_timer.initial_duration.as_secs_f32();

        transform.scale.x = progress;
        // Move the center to keep right edge fixed as we scale
        transform.translation.x = (WINDOW_WIDTH / 2.0) * (-1.0 + progress);
    }

    if game_timer.timer.just_finished() {
        RIBBIT_MESSAGE_QUEUE.lock().push(RibbitMessage::End);
    }
}

fn ribbit_simulation(
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    mut game_timer: ResMut<GameTimer>,
) {
    if keycode.just_pressed(KeyCode::KeyR) {
        RIBBIT_MESSAGE_QUEUE.lock().push(RibbitMessage::Restart);
        game_timer.timer.reset();
    } else if keycode.just_pressed(KeyCode::KeyS) {
        RIBBIT_MESSAGE_QUEUE.lock().push(RibbitMessage::Start);
    } else if keycode.just_pressed(KeyCode::KeyE) {
        RIBBIT_MESSAGE_QUEUE.lock().push(RibbitMessage::End);
        let initial_duration = game_timer.initial_duration;
        game_timer.timer.set_elapsed(initial_duration);
    }

    let messages: Vec<_> = BIT_MESSAGE_QUEUE.lock().drain(..).collect();
    for message in messages {
        match message {
            BitMessage::Parameters(parameters) => {
                let duration = parameters.duration.get_duration();
                commands.insert_resource(GameTimer {
                    timer: Timer::new(duration, TimerMode::Once),
                    initial_duration: duration,
                });

                // Spawn the progress bar
                commands.spawn((
                    Sprite {
                        color: Color::srgb(0.0, 1.0, 0.0),
                        custom_size: Some(Vec2::new(WINDOW_WIDTH, 10.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -WINDOW_HEIGHT / 2.0, 0.0),
                    TimerProgressBar,
                ));
            }
            BitMessage::Ready => {
                info!("Ready");
            }
            BitMessage::Start => {
                info!("Start");
            }
            BitMessage::End(bit_result) => info!("End {:?}", bit_result),
        }
    }
}
