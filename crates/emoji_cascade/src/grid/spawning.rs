use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use rand::prelude::*;

use super::components::{GridState, GridTile};

pub fn spawn_grid(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    config: Res<crate::game::LevelConfig>,
    query: Query<Entity, With<GridTile>>,
    mut grid_state: ResMut<GridState>,
) {
    // Only spawn if grid is empty
    if !query.is_empty() || !emoji::is_emoji_system_ready(&validation) {
        return;
    }

    let (rows, cols) = config.grid_size;
    let spacing = config.grid_spacing;

    // Calculate grid dimensions
    let grid_width = cols as f32 * spacing;
    let grid_height = rows as f32 * spacing;
    let start_x = -(grid_width / 2.0);
    let start_y = grid_height / 2.0;

    // Initialize grid positions
    grid_state.grid_positions = vec![vec![None; cols as usize]; rows as usize];

    // Get random emoji selection
    let emoji_indices = emoji::get_random_emojis(&atlas, &validation, config.num_emoji_types);
    let mut rng = rand::thread_rng();

    for row in 0..rows {
        for col in 0..cols {
            let x = (col as f32 * spacing) + start_x + spacing / 2.0;
            let y = -(row as f32) * spacing + start_y - spacing / 2.0; // Convert row to f32 before negation
            let position = Vec2::new(x, y);

            let emoji_idx = emoji_indices[rng.gen_range(0..emoji_indices.len())];

            if let Some(entity) =
                emoji::spawn_emoji(&mut commands, &atlas, &validation, emoji_idx, position, 0.5)
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

                grid_state.grid_positions[row as usize][col as usize] = Some(tile_entity);
            }
        }
    }
}
