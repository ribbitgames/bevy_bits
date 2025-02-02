use bevy::prelude::*;
use bits_helpers::input::{just_pressed_world_position, just_released_world_position};

use super::components::{GridState, GridTile, MINIMUM_DRAG_DISTANCE};
use super::sliding::{slide_column, slide_row};

/// System for handling grid input interactions including drag operations for sliding rows and columns.
/// Uses the input helper functions to handle both mouse and touch input uniformly.
pub fn handle_input(
    mut grid_state: ResMut<GridState>,
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    config: Res<crate::game::LevelConfig>,
    mut query: Query<(&mut GridTile, &Transform)>,
) {
    // Skip input handling if a slide animation is active
    if grid_state.sliding_active {
        return;
    }

    // Handle drag start
    if grid_state.drag_start.is_none() {
        if let Some(world_pos) =
            just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
        {
            grid_state.drag_start = Some(world_pos);
            return;
        }
    }

    // Handle drag end
    if let Some(start_pos) = grid_state.drag_start {
        if let Some(end_pos) =
            just_released_world_position(&buttons, &touch_input, &windows, &camera_q)
        {
            let drag_vector = end_pos - start_pos;

            // Reset drag if minimum distance not met
            if drag_vector.length() < MINIMUM_DRAG_DISTANCE {
                grid_state.drag_start = None;
                return;
            }

            let spacing = config.grid_spacing;
            let (rows, cols) = config.grid_size;

            // Convert world position to grid coordinates
            let grid_x = ((start_pos.x + (cols as f32 * spacing / 2.0)) / spacing).floor() as i32;
            let grid_y = ((-start_pos.y + (rows as f32 * spacing / 2.0)) / spacing).floor() as i32;

            // Validate grid coordinates
            if grid_x >= 0 && grid_x < cols as i32 && grid_y >= 0 && grid_y < rows as i32 {
                // Determine slide direction based on dominant drag axis
                if drag_vector.x.abs() > drag_vector.y.abs() {
                    // Horizontal slide
                    let direction = if drag_vector.x > 0.0 { 1 } else { -1 };
                    slide_row(&mut grid_state, grid_y as usize, direction, &mut query);
                } else {
                    // Vertical slide
                    let direction = if drag_vector.y > 0.0 { -1 } else { 1 };
                    slide_column(&mut grid_state, grid_x as usize, direction, &mut query);
                }
                grid_state.sliding_active = true;
            }

            grid_state.drag_start = None;
        }
    }
}
