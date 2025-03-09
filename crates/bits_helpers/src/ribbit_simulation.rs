use std::time::Duration;

use bevy::prelude::*;
use ribbit_bits::{BitDuration, BitMessage, BitResult, RibbitMessage};

use crate::{BIT_MESSAGE_QUEUE, FONT, RIBBIT_MESSAGE_QUEUE, WINDOW_HEIGHT, WINDOW_WIDTH};

/// Event sent when the game over screen should be cleaned up
#[derive(Event)]
pub struct CleanupGameOverEvent;

pub struct RibbitSimulation;

impl Plugin for RibbitSimulation {
    fn build(&self, app: &mut App) {
        app.add_event::<CleanupGameOverEvent>();
        app.add_systems(Startup, ribbit_simulation_init);
        app.add_systems(Update, ribbit_simulation);
        app.add_systems(Update, update_timer_display);
        app.add_systems(Update, cleanup_game_over);
    }
}

/// Component tag for the timer progress bar
#[derive(Component)]
pub struct TimerProgressBar;

/// Component marker for game over screen entities
#[derive(Component)]
pub struct GameOverScreen;

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
    asset_server: Res<AssetServer>,
    mut event_writer: EventWriter<CleanupGameOverEvent>,
) {
    if keycode.just_pressed(KeyCode::KeyR) {
        RIBBIT_MESSAGE_QUEUE.lock().push(RibbitMessage::Restart);
        game_timer.timer.reset();
        event_writer.send(CleanupGameOverEvent);
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
            BitMessage::End(bit_result) => {
                spawn_game_over_screen(&mut commands, &asset_server, bit_result);
            }
        }
    }
}

/// Spawns the game over screen with final score
pub fn spawn_game_over_screen(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    bit_result: BitResult,
) {
    // Create a semi-transparent overlay
    commands.spawn((
        GameOverScreen,
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.8),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Visible,
    ));

    // Game Over text
    commands.spawn((
        GameOverScreen,
        Text::new("Game Over!"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 48.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(5.0),
            width: Val::Percent(100.0),
            ..default()
        },
    ));

    // Final score
    let text = match bit_result {
        BitResult::LowestScore(score) | BitResult::HighestScore(score) => {
            format!("Final score: {score}")
        }
        BitResult::LongestDuration(duration) | BitResult::FastestDuration(duration) => {
            format!("Time: {:1}", duration.as_secs_f32())
        }
        BitResult::Success => "You won!".to_string(),
        BitResult::Failure => "You lost! Try again!".to_string(),
    };

    commands.spawn((
        GameOverScreen,
        Text::new(text),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(45.0),
            width: Val::Percent(100.0),
            ..default()
        },
    ));

    commands.spawn((
        GameOverScreen,
        Text::new("Press 'R' to restart"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(90.0),
            width: Val::Percent(100.0),
            ..default()
        },
    ));
}

fn cleanup_game_over(
    mut commands: Commands,
    mut event_reader: EventReader<CleanupGameOverEvent>,
    query: Query<Entity, With<GameOverScreen>>,
) {
    for _ in event_reader.read() {
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    }
}
