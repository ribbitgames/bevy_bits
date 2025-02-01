use bevy::prelude::*;

pub fn just_pressed_screen_position(
    button_input: &Res<ButtonInput<MouseButton>>,
    touch_input: &Res<Touches>,
    windows: &Query<&Window>,
) -> Option<Vec2> {
    if button_input.just_pressed(MouseButton::Left) {
        let cursor_position = windows.single().cursor_position()?;
        Some(cursor_position)
    } else if touch_input.any_just_pressed() {
        let touch = touch_input.iter_just_pressed().next()?;
        Some(touch.position())
    } else {
        None
    }
}

pub fn just_pressed_world_position(
    button_input: &Res<ButtonInput<MouseButton>>,
    touch_input: &Res<Touches>,
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let position = just_pressed_screen_position(button_input, touch_input, windows)?;

    let (camera, camera_transform) = camera.single();

    camera.viewport_to_world_2d(camera_transform, position).ok()
}

pub fn just_released_screen_position(
    button_input: &Res<ButtonInput<MouseButton>>,
    touch_input: &Res<Touches>,
    windows: &Query<&Window>,
) -> Option<Vec2> {
    if button_input.just_released(MouseButton::Left) {
        let cursor_position = windows.single().cursor_position()?;
        Some(cursor_position)
    } else if touch_input.any_just_released() {
        let touch = touch_input.iter_just_released().next()?;
        Some(touch.position())
    } else if touch_input.any_just_canceled() {
        let touch = touch_input.iter_just_canceled().next()?;
        Some(touch.position())
    } else {
        None
    }
}

pub fn just_released_world_position(
    button_input: &Res<ButtonInput<MouseButton>>,
    touch_input: &Res<Touches>,
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let position = just_released_screen_position(button_input, touch_input, windows)?;

    let (camera, camera_transform) = camera.single();

    camera.viewport_to_world_2d(camera_transform, position).ok()
}

pub fn pressed_screen_position(
    button_input: &Res<ButtonInput<MouseButton>>,
    touch_input: &Res<Touches>,
    windows: &Query<&Window>,
) -> Option<Vec2> {
    if button_input.pressed(MouseButton::Left) {
        let cursor_position = windows.single().cursor_position()?;
        Some(cursor_position)
    } else {
        touch_input.first_pressed_position()
    }
}

pub fn pressed_world_position(
    button_input: &Res<ButtonInput<MouseButton>>,
    touch_input: &Res<Touches>,
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let position = pressed_screen_position(button_input, touch_input, windows)?;

    let (camera, camera_transform) = camera.single();

    camera.viewport_to_world_2d(camera_transform, position).ok()
}
