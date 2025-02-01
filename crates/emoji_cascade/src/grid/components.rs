use bevy::prelude::*;

pub const SLIDE_SPEED: f32 = 500.0; // pixels per second
pub const MINIMUM_DRAG_DISTANCE: f32 = 20.0; // minimum pixels to trigger a slide
pub const SLIDE_THRESHOLD: f32 = 0.1; // distance threshold to snap to grid

#[derive(Component)]
pub struct GridTile {
    pub emoji_index: usize,
    pub grid_pos: IVec2,
    pub is_sliding: bool,
    pub target_pos: Vec2,
}

#[derive(Resource, Default)]
pub struct GridState {
    pub drag_start: Option<Vec2>,
    pub sliding_active: bool,
    pub grid_positions: Vec<Vec<Option<Entity>>>, // 2D grid of entity references
}
