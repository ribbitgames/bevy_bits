use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use bits_helpers::input::{just_pressed_world_position, pressed_world_position};
use bits_helpers::FONT;

use crate::game::{GameState, TimerText, WINDOW_HEIGHT};
use crate::obstacles::{check_collision, Obstacle};

pub const PLAYER_WIDTH: f32 = 40.0;
pub const PLAYER_HEIGHT: f32 = 60.0;

#[derive(Component)]
pub struct Player {
    pub radius: f32,
}

#[derive(Resource, Default)]
pub struct DragState {
    pub is_dragging: bool,
    pub drag_start: Option<Vec2>,
    pub initial_player_pos: Option<Vec2>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (handle_drag_input, player_movement, check_collisions)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    asset_server: Res<AssetServer>,
) {
    let player_radius = PLAYER_WIDTH.min(PLAYER_HEIGHT) / 2.0;

    // Create transform for player emoji
    let player_transform =
        Transform::from_xyz(0.0, -WINDOW_HEIGHT / 2.0 + PLAYER_HEIGHT + 10.0, 0.0)
            .with_scale(Vec3::splat(1.0));

    // Spawn player emoji (using a specific emoji index for the player)
    if let Some(player_entity) = emoji::spawn_emoji(
        &mut commands,
        &atlas,
        &validation,
        0, // Using first emoji for player - you might want to choose a specific one
        player_transform,
    ) {
        commands.entity(player_entity).insert(Player {
            radius: player_radius,
        });
    }

    // Spawn timer text
    commands.spawn((
        Text::new("Time: 0.0"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TimerText,
    ));
}

fn handle_drag_input(
    mut drag_state: ResMut<DragState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    windows: Query<&Window>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let player_transform = player_query.single();

    if let Some(world_position) =
        just_pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
    {
        if is_point_in_rect(
            world_position,
            player_transform.translation.truncate(),
            Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
        ) {
            drag_state.is_dragging = true;
            drag_state.drag_start = Some(world_position);
            drag_state.initial_player_pos = Some(player_transform.translation.truncate());
        }
    } else if mouse_input.just_released(MouseButton::Left)
        || touch_input.any_just_released()
        || touch_input.any_just_canceled()
    {
        drag_state.is_dragging = false;
        drag_state.drag_start = None;
        drag_state.initial_player_pos = None;
    }
}

fn player_movement(
    drag_state: ResMut<DragState>,
    mut player_query: Query<&mut Transform, With<Player>>,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touch_input: Res<Touches>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if !drag_state.is_dragging {
        return;
    }

    let Some(world_position) =
        pressed_world_position(&mouse_input, &touch_input, &windows, &camera_query)
    else {
        return;
    };

    let mut player_transform = player_query.single_mut();
    player_transform.translation.x = world_position.x;
}

fn check_collisions(
    player_query: Query<(&Transform, &Player)>,
    obstacle_query: Query<(&Transform, &Obstacle)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (player_transform, player) = player_query.single();
    let player_pos = player_transform.translation.truncate();

    for (obstacle_transform, obstacle) in obstacle_query.iter() {
        let obstacle_pos = obstacle_transform.translation.truncate();

        if check_collision(player_pos, player.radius, obstacle_pos, obstacle.radius) {
            next_state.set(GameState::GameOver);
            return;
        }
    }
}

fn is_point_in_rect(point: Vec2, rect_center: Vec2, rect_size: Vec2) -> bool {
    let half_size = rect_size / 2.0;
    point.x >= rect_center.x - half_size.x
        && point.x <= rect_center.x + half_size.x
        && point.y >= rect_center.y - half_size.y
        && point.y <= rect_center.y + half_size.y
}
