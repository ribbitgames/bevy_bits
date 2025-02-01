use bevy::prelude::*;

use super::components::{GridState, GridTile, SLIDE_SPEED, SLIDE_THRESHOLD};

pub fn slide_row(
    grid_state: &mut GridState,
    row: usize,
    direction: i32,
    query: &mut Query<(&mut GridTile, &Transform)>,
) {
    let cols = grid_state.grid_positions[row].len();
    let mut new_row = vec![None; cols];

    // Calculate new positions
    for col in 0..cols {
        if let Some(entity) = grid_state.grid_positions[row][col] {
            let new_col = (col as i32 + direction).rem_euclid(cols as i32) as usize;
            new_row[new_col] = Some(entity);

            if let Ok((mut tile, transform)) = query.get_mut(entity) {
                // Update grid position
                tile.grid_pos.x = new_col as i32;

                // Calculate new target position while keeping y constant
                let x_offset = transform.translation.x.signum() * tile.target_pos.x.abs();
                tile.target_pos.x = x_offset + (direction as f32 * tile.target_pos.x.abs());
                tile.is_sliding = true;
            }
        }
    }

    grid_state.grid_positions[row] = new_row;
}

pub fn slide_column(
    grid_state: &mut GridState,
    col: usize,
    direction: i32,
    query: &mut Query<(&mut GridTile, &Transform)>,
) {
    let rows = grid_state.grid_positions.len();
    let mut new_col = vec![None; rows];

    // Calculate new positions
    for row in 0..rows {
        if let Some(entity) = grid_state.grid_positions[row][col] {
            let new_row = (row as i32 + direction).rem_euclid(rows as i32) as usize;
            new_col[new_row] = Some(entity);

            if let Ok((mut tile, transform)) = query.get_mut(entity) {
                // Update grid position
                tile.grid_pos.y = new_row as i32;

                // Calculate new target position while keeping x constant
                let y_offset = transform.translation.y.signum() * tile.target_pos.y.abs();
                tile.target_pos.y = y_offset + (direction as f32 * tile.target_pos.y.abs());
                tile.is_sliding = true;
            }
        }
    }

    // Update grid positions
    for (row, &entity) in new_col.iter().enumerate() {
        grid_state.grid_positions[row][col] = entity;
    }
}

pub fn update_sliding(
    mut grid_state: ResMut<GridState>,
    mut query: Query<(&mut Transform, &mut GridTile)>,
    time: Res<Time>,
) {
    if !grid_state.sliding_active {
        return;
    }

    let mut all_tiles_in_position = true;

    for (mut transform, mut tile) in query.iter_mut() {
        if !tile.is_sliding {
            continue;
        }

        let current_pos = transform.translation.truncate();
        let target_pos = tile.target_pos;
        let diff = target_pos - current_pos;

        if diff.length() > SLIDE_THRESHOLD {
            all_tiles_in_position = false;
            let move_delta = diff.normalize() * SLIDE_SPEED * time.delta_secs();

            // Check if we would overshoot
            if move_delta.length() > diff.length() {
                transform.translation = target_pos.extend(transform.translation.z);
                tile.is_sliding = false;
            } else {
                transform.translation += move_delta.extend(0.0);
            }
        } else {
            // Snap to final position
            transform.translation = target_pos.extend(transform.translation.z);
            tile.is_sliding = false;
        }
    }

    if all_tiles_in_position {
        grid_state.sliding_active = false;
    }
}
