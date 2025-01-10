use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::utils::Duration;
use bits_helpers::WINDOW_HEIGHT;

const FRAME_OFFSET: f32 = 64.;
const FRAME_SIZE: f32 = 62.;
const FRAME_THICKNESS: f32 = 4.;
const CURSOR_SIZE: f32 = 48.;

const FRAME_COLOR_DEFAULT: Color = Color::Srgba(Srgba {
    red: 0.5,
    green: 0.5,
    blue: 0.5,
    alpha: 1.,
});

const FRAME_COLOR_HIGHLIGHT: Color = Color::Srgba(Srgba {
    red: 1.,
    green: 1.,
    blue: 1.,
    alpha: 1.,
});

pub const FRAME_COLOR_HIT: Color = Color::Srgba(Srgba {
    red: 0.,
    green: 1.,
    blue: 0.,
    alpha: 1.,
});

pub const FRAME_COLOR_BLOW: Color = Color::Srgba(Srgba {
    red: 1.,
    green: 1.,
    blue: 0.,
    alpha: 1.,
});

pub const FRAME_COLOR_MISS: Color = Color::Srgba(Srgba {
    red: 1.,
    green: 0.,
    blue: 0.,
    alpha: 1.,
});

const FRAME_COLOR_INSIDE: Color = Color::Srgba(Srgba {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 1.,
});

const CURSOR_COLOR: Color = Color::Srgba(Srgba {
    red: 0.5,
    green: 0.5,
    blue: 0.5,
    alpha: 0.5,
});

#[derive(Event)]
pub struct ResetBoard;

#[derive(Event)]
pub struct AddCharacterAt {
    pub x: usize,
    pub y: usize,
    pub c: char,
}

#[derive(Event)]
pub struct RemoveCharacterAt {
    pub x: usize,
    pub y: usize,
}

#[derive(Event)]
pub struct SetColorsAt {
    pub row: usize,
    pub colors: Vec<Color>,
}

#[derive(Component)]
struct Frame {
    x: i32,
    y: i32,
}

#[derive(Component)]
pub struct Cursor {
    pub x: i32,
    pub y: i32,
    pub timer: Timer,
}

#[derive(Component)]
struct Character;

#[derive(Resource)]
struct BoardSize {
    row: usize,
    column: usize,
}

#[derive(Resource)]
struct CurrentRow {
    chars: Vec<Entity>,
}

pub struct DecodingBoardPlugin {
    pub row: usize,
    pub column: usize,
}

impl Plugin for DecodingBoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoardSize {
            row: self.row,
            column: self.column,
        })
        .insert_resource(CurrentRow {
            chars: vec![Entity::PLACEHOLDER; self.column],
        })
        .add_event::<ResetBoard>()
        .add_event::<AddCharacterAt>()
        .add_event::<RemoveCharacterAt>()
        .add_event::<SetColorsAt>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                reset_board,
                animate_cursor,
                add_character_at,
                remove_character_at,
                set_colors_at,
            ),
        );
    }
}

fn setup(mut commands: Commands, board_size: Res<BoardSize>) {
    for y in 0..board_size.row {
        for x in 0..board_size.column {
            spawn_frame(&mut commands, x as i32, y as i32, board_size.column);
        }
    }

    let position = index_to_position(0, 0, board_size.column);
    commands
        .spawn((
            Sprite::from_color(CURSOR_COLOR, Vec2::new(CURSOR_SIZE, CURSOR_SIZE)),
            Transform::from_xyz(position.x, position.y, 0.2),
        ))
        .insert(Cursor {
            x: 0,
            y: 0,
            timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
        });
}

fn spawn_frame(commands: &mut Commands, x: i32, y: i32, column: usize) {
    let position = index_to_position(x, y, column);

    let color = if y == 0 {
        FRAME_COLOR_HIGHLIGHT
    } else {
        FRAME_COLOR_DEFAULT
    };
    commands
        .spawn((
            Sprite::from_color(color, Vec2::new(FRAME_SIZE, FRAME_SIZE)),
            Transform::from_xyz(position.x, position.y, 0.),
        ))
        .insert(Frame { x, y });
    commands.spawn((
        Sprite::from_color(
            FRAME_COLOR_INSIDE,
            Vec2::new(
                FRAME_THICKNESS.mul_add(-2., FRAME_SIZE),
                FRAME_THICKNESS.mul_add(-2., FRAME_SIZE),
            ),
        ),
        Transform::from_xyz(position.x, position.y, 0.1),
    ));
}

fn index_to_position(x: i32, y: i32, column: usize) -> Vec2 {
    Vec2::new(
        (x as f32).mul_add(
            FRAME_OFFSET,
            FRAME_OFFSET * 0.5f32.mul_add(((column - 1) % 2) as f32, -((column / 2) as f32)),
        ),
        (y as f32).mul_add(-FRAME_OFFSET, WINDOW_HEIGHT.mul_add(0.5, -FRAME_OFFSET)),
    )
}

fn reset_board(
    mut commands: Commands,
    mut frame_query: Query<(&mut Sprite, &Frame)>,
    mut cursor_query: Query<(&mut Cursor, &mut Transform)>,
    character_query: Query<(&Character, Entity)>,
    mut events: EventReader<ResetBoard>,
    board_size: Res<BoardSize>,
) {
    for _event in events.read() {
        for (mut sprite, _frame) in &mut frame_query {
            sprite.color = FRAME_COLOR_DEFAULT;
        }
        let (mut cursor, mut transform) = cursor_query.single_mut();
        cursor.x = 0;
        cursor.y = 0;
        let position = index_to_position(0, 0, board_size.column);
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        for (_character, entity) in &character_query {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_cursor(mut cursor: Query<(&mut Cursor, &mut Sprite)>, time: Res<Time>) {
    let (mut c, mut sprite) = cursor.single_mut();
    c.timer.tick(time.delta());
    sprite
        .color
        .set_alpha((f32::sin(c.timer.elapsed_secs() * core::f32::consts::PI) + 1.) * 0.25);
}

fn add_character_at(
    mut commands: Commands,
    mut query: Query<(&mut Cursor, &mut Transform)>,
    mut events: EventReader<AddCharacterAt>,
    mut entities: ResMut<CurrentRow>,
    board_size: Res<BoardSize>,
) {
    for event in events.read() {
        let (mut cursor, mut transform) = query.single_mut();
        // add the character at the current cursor position first
        let position = index_to_position(cursor.x, cursor.y, board_size.column);
        let entity = commands
            .spawn((
                Text2d::new(event.c.to_string()),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(position.x, position.y, 0.2),
                Character,
            ))
            .id();
        if let Some(e) = entities.chars.get_mut(cursor.x as usize) {
            *e = entity;
        }
        // then move the cursor at the new position
        cursor.x = event.x as i32;
        cursor.y = event.y as i32;
        let new_pos = index_to_position(cursor.x, cursor.y, board_size.column);
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

fn remove_character_at(
    mut commands: Commands,
    mut query: Query<(&mut Cursor, &mut Transform)>,
    mut events: EventReader<RemoveCharacterAt>,
    entities: Res<CurrentRow>,
    board_size: Res<BoardSize>,
) {
    for event in events.read() {
        let (mut cursor, mut transform) = query.single_mut();
        cursor.x = event.x as i32;
        cursor.y = event.y as i32;
        let new_pos = index_to_position(cursor.x, cursor.y, board_size.column);
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
        let entity = entities.chars.get(event.x).expect("");
        commands.entity(*entity).despawn();
    }
}

fn set_colors_at(mut query: Query<(&Frame, &mut Sprite)>, mut events: EventReader<SetColorsAt>) {
    for event in events.read() {
        for (frame, mut sprite) in &mut query {
            if frame.y != event.row as i32 {
                continue;
            }
            if let Some(color) = event.colors.get(frame.x as usize) {
                sprite.color = *color;
            }
        }
    }
}
