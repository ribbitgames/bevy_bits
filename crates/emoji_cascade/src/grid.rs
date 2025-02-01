use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use bits_helpers::input::just_pressed_world_position;

pub struct GridPlugin;

#[derive(Component)]
pub struct GridTile {
    pub emoji_index: usize,
    pub grid_pos: IVec2,
    pub is_sliding: bool,
}

#[derive(Component)]
pub struct SlidingRow {
    pub row_index: i32,
    pub target_offset: f32,
}

#[derive(Component)]
pub struct SlidingColumn {
    pub col_index: i32,
    pub target_offset: f32,
}

#[derive(Resource)]
pub struct GridState {
    pub selected_row: Option<i32>,
    pub selected_col: Option<i32>,
    pub sliding_active: bool,
    pub drag_start: Option<Vec2>,
}

impl Default for GridState {
    fn default() -> Self {
        Self {
            selected_row: None,
            selected_col: None,
            sliding_active: false,
            drag_start: None,
        }
    }
}

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridState>().add_systems(
            Update,
            (
                spawn_grid,
                handle_input,
                update_sliding,
                check_matches,
                handle_cascading,
            )
                .chain()
                .run_if(in_state(crate::game::GameState::Playing)),
        );
    }
}

fn spawn_grid(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    config: Res<crate::game::LevelConfig>,
    query: Query<Entity, With<GridTile>>,
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
    let start_x = -grid_width / 2.0;
    let start_y = grid_height / 2.0;

    // Get random emoji selection
    let emoji_indices = emoji::get_random_emojis(&atlas, &validation, config.num_emoji_types);

    for row in 0..rows {
        for col in 0..cols {
            let x = col as f32 * spacing + start_x + spacing / 2.0;
            let y = -row as f32 * spacing + start_y - spacing / 2.0;

            // Randomly select emoji from our available types
            let emoji_idx = emoji_indices[fastrand::usize(..emoji_indices.len())];

            if let Some(entity) = emoji::spawn_emoji(
                &mut commands,
                &atlas,
                &validation,
                emoji_idx,
                Vec2::new(x, y),
                0.5,
            ) {
                commands.entity(entity).insert(GridTile {
                    emoji_index: emoji_idx,
                    grid_pos: IVec2::new(col as i32, row as i32),
                    is_sliding: false,
                });
            }
        }
    }
}

fn handle_input(
    mut grid_state: ResMut<GridState>,
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    // Handle drag start
    if let Some(world_pos) =
        just_pressed_world_position(&buttons, &touch_input, &windows, &camera_q)
    {
        grid_state.drag_start = Some(world_pos);
    }

    // Handle drag end and calculate slide direction
    if buttons.just_released(MouseButton::Left) || touch_input.any_just_released() {
        if let Some(start_pos) = grid_state.drag_start {
            // Calculate drag direction and distance
            // TODO: Implement sliding logic
        }
        grid_state.drag_start = None;
    }
}

fn update_sliding(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut GridTile)>,
    time: Res<Time>,
) {
    // TODO: Implement smooth sliding animation
}

fn check_matches(mut commands: Commands, query: Query<(Entity, &GridTile)>) -> u32 {
    // TODO: Implement match detection
    // Return number of chains for scoring
    0
}

fn handle_cascading(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut GridTile)>,
) {
    // TODO: Implement cascading after matches
}
