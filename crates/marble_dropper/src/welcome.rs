use bevy::prelude::*;
use bits_helpers::input::just_pressed_world_position;
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};

use crate::core::GameState;

#[derive(Component)]
pub struct WelcomeScreen;

pub fn spawn_welcome_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        WelcomeScreen,
    ));

    commands.spawn((
        Text2d::new("Marble Dropper"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, WINDOW_HEIGHT / 4.0, 1.0),
        WelcomeScreen,
    ));

    commands.spawn((
        Text2d::new("Tap to Start"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 30.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -WINDOW_HEIGHT / 4.0, 1.0),
        WelcomeScreen,
    ));
}

pub fn handle_welcome_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if just_pressed_world_position(&mouse_input, &touch_input, &windows, &camera).is_some() {
        next_state.set(GameState::Playing);
    }
}

pub fn despawn_welcome_screen(mut commands: Commands, query: Query<Entity, With<WelcomeScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
