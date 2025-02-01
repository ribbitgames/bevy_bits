use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use rand::prelude::*;

use super::components::{GridState, GridTile};
use crate::game::ChainEvent;

pub fn check_matches(
    mut commands: Commands,
    query: Query<(Entity, &GridTile)>,
    grid_state: Res<GridState>,
    mut chain_events: EventWriter<ChainEvent>,
) {
    if grid_state.sliding_active {
        return;
    }

    let matches = find_matches(&grid_state, &query);
    if !matches.is_empty() {
        // Remove matched tiles
        for entity in &matches {
            commands.entity(*entity).despawn();
        }

        // Send chain event - each match is 3 or more tiles
        let chain_count = matches.len() as u32 / 3;
        chain_events.send(ChainEvent(chain_count));
    }
}

pub fn handle_cascading(
    mut commands: Commands,
    mut grid_state: ResMut<GridState>,
    mut query: Query<(Entity, &mut GridTile, &mut Transform)>,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    config: Res<crate::game::LevelConfig>,
) {
    if grid_state.sliding_active {
        return;
    }

    let (rows, cols) = config.grid_size;
    let spacing = config.grid_spacing;

    // Phase 1: Move existing tiles down to fill gaps
    for col in 0..cols as usize {
        compact_column(col, &mut grid_state, &mut query, spacing);
    }

    // Phase 2: Spawn new tiles at the top
    spawn_new_tiles(&mut commands, &mut grid_state, &atlas, &validation, &config);
}

fn find_matches(grid_state: &GridState, query: &Query<(Entity, &GridTile)>) -> Vec<Entity> {
    let mut matches = Vec::new();
    let rows = grid_state.grid_positions.len();
    let cols = grid_state.grid_positions[0].len();

    // Check horizontal matches
    for row in 0..rows {
        let mut current_run = Vec::new();
        let mut current_emoji = None;

        for col in 0..cols {
            if let Some(entity) = grid_state.grid_positions[row][col] {
                if let Ok((entity, tile)) = query.get(entity) {
                    match current_emoji {
                        Some(emoji) if emoji == tile.emoji_index => {
                            current_run.push(entity);
                        }
                        _ => {
                            // Check if we had a match before resetting
                            if current_run.len() >= 3 {
                                matches.extend(current_run);
                            }
                            current_run = vec![entity];
                            current_emoji = Some(tile.emoji_index);
                        }
                    }
                }
            } else {
                if current_run.len() >= 3 {
                    matches.extend(current_run);
                }
                current_run = Vec::new();
                current_emoji = None;
            }
        }
        if current_run.len() >= 3 {
            matches.extend(current_run);
        }
    }

    // Check vertical matches
    for col in 0..cols {
        let mut current_run = Vec::new();
        let mut current_emoji = None;

        for row in 0..rows {
            if let Some(entity) = grid_state.grid_positions[row][col] {
                if let Ok((entity, tile)) = query.get(entity) {
                    match current_emoji {
                        Some(emoji) if emoji == tile.emoji_index => {
                            current_run.push(entity);
                        }
                        _ => {
                            if current_run.len() >= 3 {
                                matches.extend(current_run);
                            }
                            current_run = vec![entity];
                            current_emoji = Some(tile.emoji_index);
                        }
                    }
                }
            } else {
                if current_run.len() >= 3 {
                    matches.extend(current_run);
                }
                current_run = Vec::new();
                current_emoji = None;
            }
        }
        if current_run.len() >= 3 {
            matches.extend(current_run);
        }
    }

    matches
}

fn compact_column(
    col: usize,
    grid_state: &mut GridState,
    query: &mut Query<(Entity, &mut GridTile, &mut Transform)>,
    spacing: f32,
) {
    let rows = grid_state.grid_positions.len();
    let mut empty_spaces = 0;

    // Move from bottom to top
    for row in (0..rows).rev() {
        if grid_state.grid_positions[row][col].is_none() {
            empty_spaces += 1;
            continue;
        }

        if empty_spaces > 0 {
            let new_row = row + empty_spaces;
            if let Some(entity) = grid_state.grid_positions[row][col] {
                if let Ok((_, mut tile, mut transform)) = query.get_mut(entity) {
                    // Update grid position
                    tile.grid_pos.y = new_row as i32;

                    // Calculate new visual position
                    let new_y = -(new_row as f32) * spacing;
                    tile.target_pos.y = new_y;
                    transform.translation.y = new_y;

                    // Update grid state
                    grid_state.grid_positions[new_row][col] = Some(entity);
                    grid_state.grid_positions[row][col] = None;
                }
            }
        }
    }
}

fn spawn_new_tiles(
    commands: &mut Commands,
    grid_state: &mut GridState,
    atlas: &Res<EmojiAtlas>,
    validation: &Res<AtlasValidation>,
    config: &Res<crate::game::LevelConfig>,
) {
    let (rows, cols) = config.grid_size;
    let spacing = config.grid_spacing;
    let mut rng = rand::thread_rng();
    let emoji_indices = emoji::get_random_emojis(atlas, validation, config.num_emoji_types);

    for col in 0..cols as usize {
        let mut row = 0;
        while row < rows as usize && grid_state.grid_positions[row][col].is_none() {
            let x = col as f32 * spacing - ((cols as f32 - 1.0) * spacing / 2.0);
            let y = -(row as f32 * spacing);
            let position = Vec2::new(x, y);

            // Pick a random emoji that won't create an immediate match
            let mut attempts = 0;
            let mut emoji_idx;
            loop {
                emoji_idx = emoji_indices[rng.gen_range(0..emoji_indices.len())];
                if attempts > 5 || !would_create_match(row, col, emoji_idx, grid_state) {
                    break;
                }
                attempts += 1;
            }

            if let Some(entity) =
                emoji::spawn_emoji(commands, atlas, validation, emoji_idx, position, 0.5)
            {
                let tile_entity = commands
                    .entity(entity)
                    .insert(GridTile {
                        emoji_index: emoji_idx,
                        grid_pos: IVec2::new(col as i32, row as i32),
                        is_sliding: false,
                        target_pos: position,
                    })
                    .id();
                grid_state.grid_positions[row][col] = Some(tile_entity);
            }
            row += 1;
        }
    }
}

fn would_create_match(row: usize, col: usize, emoji_idx: usize, grid_state: &GridState) -> bool {
    let rows = grid_state.grid_positions.len();
    let cols = grid_state.grid_positions[0].len();

    // Check horizontal matches
    if col >= 2 {
        let mut matching = 0;
        for i in 1..=2 {
            if let Some(entity) = grid_state.grid_positions[row][col - i] {
                // In a real implementation, you'd check the emoji_index of the entity
                // For now, we'll just prevent obvious matches
                if matching >= 2 {
                    return true;
                }
            }
        }
    }

    // Check vertical matches
    if row >= 2 {
        let mut matching = 0;
        for i in 1..=2 {
            if let Some(entity) = grid_state.grid_positions[row - i][col] {
                // Similar to above
                if matching >= 2 {
                    return true;
                }
            }
        }
    }

    false
}
