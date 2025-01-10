use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use match3::prelude::*;
use ribbit::RibbitMatch3;

mod match3;
mod ribbit;

const GEM_SIDE_LENGTH: f32 = 50.0;

pub fn run() {
    bits_helpers::get_default_app::<RibbitMatch3>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .insert_resource(Selection::default())
    .insert_resource(Match3Config {
        gem_types: 5,
        board_dimensions: [8, 8].into(),
    })
    .add_plugins(Match3Plugin)
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            move_to,
            consume_events,
            input,
            visualize_selection,
            control,
            animate_once,
            shuffle,
        ),
    )
    .run();
}

#[derive(Component, Clone)]
struct VisibleBoard(HashMap<UVec2, Entity>);

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands, board: Res<Board>, asset_server: Res<AssetServer>) {
    let board_side_length = GEM_SIDE_LENGTH * 10.0;
    let centered_offset_x = board_side_length / 2.0 - GEM_SIDE_LENGTH / 2.0;
    let centered_offset_y = board_side_length / 2.0 - GEM_SIDE_LENGTH / 2.0;

    let camera = (
        Camera2d,
        Transform::from_xyz(centered_offset_x, 0.0 - centered_offset_y, 0.0),
    );
    commands.spawn(camera).insert(MainCamera);

    let mut gems = HashMap::default();

    let vis_board = commands
        .spawn((Transform::default(), Visibility::default()))
        .id();

    board.iter().for_each(|(position, typ)| {
        let transform = Transform::from_xyz(
            position.x as f32 * GEM_SIDE_LENGTH,
            position.y as f32 * -GEM_SIDE_LENGTH,
            0.0,
        );

        let child = commands
            .spawn((
                Sprite {
                    image: asset_server.load(map_type_to_path(*typ)),
                    custom_size: Some(Vec2::new(GEM_SIDE_LENGTH, GEM_SIDE_LENGTH)),
                    ..default()
                },
                transform,
            ))
            .insert(Name::new(format!("{};{}", position.x, position.y)))
            .id();
        gems.insert(*position, child);
        commands.entity(vis_board).add_child(child);
    });

    let board = VisibleBoard(gems);

    commands.entity(vis_board).insert(board);
}

fn map_type_to_path(typ: u32) -> String {
    format!("{typ}.png")
}

#[derive(Component)]
struct MoveTo(Vec2);

fn move_to(
    mut commands: Commands,
    time: Res<Time>,
    mut moves: Query<(Entity, &mut Transform, &MoveTo)>,
) {
    for (entity, mut transform, MoveTo(move_to)) in &mut moves {
        if transform.translation == Vec3::new(move_to.x, move_to.y, transform.translation.z) {
            commands.entity(entity).remove::<MoveTo>();
        } else {
            let mut movement = *move_to - transform.translation.xy();
            movement = // Multiplying the move by GEM_SIDE_LENGTH as well as delta seconds means the animation takes exactly 1 second
                (movement.normalize() * time.delta_secs() * GEM_SIDE_LENGTH * 5.0).clamp_length_max(movement.length());
            let movement = movement.extend(transform.translation.z);
            transform.translation += movement;
        }
    }
}

fn consume_events(
    mut commands: Commands,
    mut events: ResMut<BoardEvents>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut board_commands: ResMut<BoardCommands>,
    asset_server: Res<AssetServer>,
    mut board: Query<(Entity, &mut VisibleBoard)>,
    animations: Query<(), With<MoveTo>>,
) {
    if animations.iter().count() == 0 {
        if let Ok(event) = events.pop() {
            let Ok((board_entity, mut board)) = board.get_single_mut() else {
                error!("Could not find board");
                return;
            };

            match event {
                BoardEvent::Swapped(pos1, pos2) => {
                    let Some(gem1) = board.0.get(&pos1).copied() else {
                        warn!("Could not find gem at {pos1}");
                        return;
                    };
                    let Some(gem2) = board.0.get(&pos2).copied() else {
                        warn!("Could not find gem at {pos2}");
                        return;
                    };
                    commands
                        .entity(gem1)
                        .insert(MoveTo(board_pos_to_world_pos(pos2)));

                    commands
                        .entity(gem2)
                        .insert(MoveTo(board_pos_to_world_pos(pos1)));

                    board.0.insert(pos2, gem1);
                    board.0.insert(pos1, gem2);
                }
                BoardEvent::Popped(pos) => {
                    let Some(gem) = board.0.get(&pos).copied() else {
                        warn!("Could not find gem at pos {} {}", pos.x, pos.y);
                        return;
                    };
                    board.0.remove(&pos);
                    commands.entity(gem).despawn_recursive();
                    spawn_explosion(
                        &asset_server,
                        &mut texture_atlases,
                        &mut commands,
                        board_pos_to_world_pos(pos),
                    );
                }
                BoardEvent::Matched(matches) => {
                    if let Err(err) = board_commands.push(BoardCommand::Pop(
                        matches.without_duplicates().iter().copied().collect(),
                    )) {
                        error!("{err}");
                    }
                }
                BoardEvent::Dropped(drops) => {
                    // Need to keep a buffered board clone because we read and write at the same time
                    let mut new_board = board.clone();
                    for Drop { from, to } in drops {
                        let Some(gem) = board.0.get(&from).copied() else {
                            warn!("Could not find gem at pos {} {}", from.x, from.y);
                            return;
                        };
                        new_board.0.insert(to, gem);
                        new_board.0.remove(&from);
                        commands
                            .entity(gem)
                            .insert(MoveTo(board_pos_to_world_pos(to)));
                    }
                    // And copy the buffer to the resource
                    *board = new_board;
                }
                BoardEvent::Spawned(spawns) => {
                    let mut new_board = board.clone();

                    for (pos, typ) in spawns {
                        let world_pos = board_pos_to_world_pos(pos);
                        let gem = commands
                            .spawn((
                                Sprite {
                                    image: asset_server.load(map_type_to_path(typ)),
                                    custom_size: Some([50.0, 50.0].into()),
                                    ..Sprite::default()
                                },
                                Transform::from_xyz(world_pos.x, 200.0, 0.0),
                            ))
                            .insert(MoveTo(world_pos))
                            .id();
                        new_board.0.insert(pos, gem);
                        commands.entity(board_entity).add_child(gem);
                    }
                    *board = new_board;
                }
                BoardEvent::Shuffled(moves) => {
                    let mut temp_board = board.clone();
                    for (from, to) in moves {
                        let Some(gem) = board.0.get(&from).copied() else {
                            warn!("Could not find gem at pos {from}");
                            return;
                        };

                        commands
                            .entity(gem)
                            .insert(MoveTo(board_pos_to_world_pos(to)));

                        temp_board.0.insert(to, gem);
                    }
                    *board = temp_board;
                }
                BoardEvent::FailedSwap(pos1, pos2) => {
                    info!("Failed to swap elements {pos1} and {pos2}");
                }
            }
        }
    }
}

fn board_pos_to_world_pos(pos: UVec2) -> Vec2 {
    Vec2::new(
        pos.x as f32 * GEM_SIDE_LENGTH,
        -(pos.y as f32) * GEM_SIDE_LENGTH,
    )
}

#[derive(Default, Clone, Copy, Resource)]
struct Selection(Option<Entity>);

fn input(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut selection: ResMut<Selection>,
    mut button_events: EventReader<MouseButtonInput>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    board: Query<&VisibleBoard>,
) {
    for event in button_events.read() {
        if let MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            ..
        } = event
        {
            let window = window_query.single();
            let (camera, camera_transform) = camera.single();
            if let Some(world_position) = window
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
                .map(|ray| ray.origin.truncate())
            {
                // round down to the gem coordinate
                let coordinates: IVec2 = (
                    ((world_position.x + GEM_SIDE_LENGTH / 2.0) / GEM_SIDE_LENGTH) as i32,
                    ((GEM_SIDE_LENGTH / 2.0 - world_position.y) / GEM_SIDE_LENGTH) as i32,
                )
                    .into();

                if coordinates.x >= 0 && coordinates.y >= 0 {
                    selection.0 = board
                        .single()
                        .0
                        .get::<UVec2>(&UVec2::new(coordinates.x as u32, coordinates.y as u32))
                        .copied();
                }
            }
        }
    }
}

#[derive(Component)]
struct SelectionRectangle;

fn visualize_selection(
    mut commands: Commands,
    selection: Res<Selection>,
    asset_server: Res<AssetServer>,
    g_transforms: Query<&GlobalTransform>,
    mut rectangle: Query<(Entity, &mut Transform), With<SelectionRectangle>>,
) {
    if selection.is_changed() {
        if let Some(selected_gem) = selection.0 {
            let Ok(transform) = g_transforms.get(selected_gem) else {
                error!("Could not find {:?}", selected_gem);
                return;
            };

            if let Ok((_, mut old_transform)) = rectangle.get_single_mut() {
                *old_transform = (*transform).into();
            } else {
                commands
                    .spawn((
                        Sprite {
                            image: asset_server.load("rectangle.png"),
                            custom_size: Some([50.0, 50.0].into()),
                            ..default()
                        },
                        *transform,
                    ))
                    .insert(SelectionRectangle);
            }
        } else if let Ok((entity, _)) = rectangle.get_single_mut() {
            commands.entity(entity).despawn();
        }
    }
}

fn control(
    mut board_commands: ResMut<BoardCommands>,
    mut selection: ResMut<Selection>,
    mut last_selection: Local<Selection>,
    transforms: Query<&Transform>,
) {
    if selection.is_changed() {
        if let Some(selected_gem) = selection.0 {
            if let Some(last_selected_gem) = last_selection.0 {
                let Ok(selected_gem_transform) = transforms.get(selected_gem) else {
                    error!("Could not find transform of {:?}", selected_gem);
                    return;
                };
                let selected_pos = selected_gem_transform.translation.xy() / 50.0;

                let Ok(last_selected_transform) = transforms.get(last_selected_gem as Entity)
                else {
                    error!("Could not find transform of {:?}", selected_gem);
                    return;
                };

                let last_selected_pos = last_selected_transform.translation.xy() / 50.0;

                if let Err(err) = board_commands.push(BoardCommand::Swap(
                    [selected_pos.x as u32, -selected_pos.y as u32].into(),
                    [last_selected_pos.x as u32, -last_selected_pos.y as u32].into(),
                )) {
                    error!("{err}");
                    return;
                }

                selection.0 = None;
                last_selection.0 = None;
            } else {
                *last_selection = *selection;
            }
        } else {
            last_selection.0 = None;
        }
    }
}

#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_once(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut AnimationTimer, &mut Sprite)>,
) {
    for (entity, mut timer, mut sprite) in &mut timers {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let Some(texture_atlas) = &mut sprite.texture_atlas else {
                continue;
            };

            if texture_atlas.index == 3 {
                commands.entity(entity).despawn_recursive();
            } else {
                texture_atlas.index += 1;
            }
        }
    }
}

fn spawn_explosion(
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    commands: &mut Commands,
    pos: Vec2,
) {
    let texture = asset_server.load("explosion.png");
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(49, 50), 4, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_handle,
                    index: 0,
                },
            ),
            Transform::from_translation(pos.extend(0.0)),
        ))
        .insert(AnimationTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )));
}

fn shuffle(
    mut board_commands: ResMut<BoardCommands>,
    mut key_event: EventReader<KeyboardInput>,
    animations: Query<(), With<MoveTo>>,
) {
    if animations.iter().count() == 0 {
        for event in key_event.read() {
            if let KeyboardInput {
                key_code: KeyCode::KeyS,
                state: ButtonState::Pressed,
                ..
            } = event
            {
                if let Err(err) = board_commands.push(BoardCommand::Shuffle) {
                    error!("{err}");
                }
            }
        }
    }
}
