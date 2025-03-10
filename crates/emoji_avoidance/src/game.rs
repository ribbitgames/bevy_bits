use std::time::Duration;

use bevy::prelude::*;
use bits_helpers::FONT;
use bits_helpers::welcome_screen::{WelcomeScreenElement, despawn_welcome_screen};

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

#[derive(Component)]
pub struct TimerText;

#[derive(Resource, Default)]
pub struct GameTimer(pub f32);

impl From<&GameTimer> for Duration {
    fn from(value: &GameTimer) -> Self {
        let secs = value.0.trunc() as u64;
        let nanos = (value.0.fract() * 1e9) as u32;
        Self::new(secs, nanos)
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<GameTimer>()
            .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
            .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
            .add_systems(OnEnter(GameState::Playing), init_timer)
            .add_systems(
                Update,
                (
                    handle_welcome_input.run_if(in_state(GameState::Welcome)),
                    update_timer.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

fn init_timer(mut game_timer: ResMut<GameTimer>) {
    game_timer.0 = 0.0;
}

fn spawn_welcome_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load(FONT);
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            WelcomeScreenElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Emoji Avoidance"),
                TextFont {
                    font: font.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Avoid the falling emojis!"),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Tap to start"),
                TextFont {
                    font,
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn handle_welcome_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) || touch_input.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}

fn update_timer(
    mut game_timer: ResMut<GameTimer>,
    mut query: Query<&mut Text, With<TimerText>>,
    time: Res<Time>,
) {
    game_timer.0 += time.delta_secs();

    if let Ok(mut text) = query.get_single_mut() {
        text.0 = format!("Time: {:.1}", game_timer.0);
    }
}
