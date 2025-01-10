use bevy::color::palettes::css::{
    BLUE, CORAL, GREEN, LIME, MAGENTA, ORANGE, PINK, RED, TEAL, YELLOW,
};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use bits_helpers::floating_score::{animate_floating_scores, spawn_floating_score};
use bits_helpers::restart::{
    cleanup_marked_entities, handle_restart, CleanupMarker, RestartButton,
};
use bits_helpers::welcome_screen::{despawn_welcome_screen, WelcomeScreenElement};
use bits_helpers::FONT;
use rand::seq::SliceRandom;
use rand::Rng;
use ribbit::ShapeMatcher;

mod ribbit;

const COUNTDOWN_DURATION: f32 = 5.0;
const MAX_MISTAKES: usize = 3;
const CELL_SIZE: f32 = 60.0;
const SHAPE_SCALE_FACTOR: f32 = 0.9;

const FONT_SIZE_LARGE: f32 = 60.0;
const FONT_SIZE_MEDIUM: f32 = 40.0;

const STAGE_COMPLETE_DELAY: f32 = 2.0;
const HIDE_CELLS_DELAY: f32 = 0.5;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Stage {
    grid_cols: usize,
    grid_rows: usize,
}

const STAGES: [Stage; 3] = [
    Stage {
        grid_cols: 2,
        grid_rows: 4,
    },
    Stage {
        grid_cols: 3,
        grid_rows: 4,
    },
    Stage {
        grid_cols: 4,
        grid_rows: 4,
    },
];

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    StageSetup,
    Countdown,
    Playing,
    StageComplete,
    GameOver,
}

#[derive(Component, Clone, Copy, PartialEq)]
enum Shape {
    Circle,
    Square,
    Triangle,
    Hexagon,
    Pentagon,
    Octagon,
    Quad,
}

#[derive(Component, Clone)]
struct GridCell {
    row: usize,
    col: usize,
    shape: Shape,
    color: Color,
    is_revealed: bool,
    is_matched: bool,
    shape_entity: Entity,
    question_mark_entity: Entity,
    is_locked: bool,
}

#[derive(Resource)]
struct GameData {
    mistakes: usize,
    matched_pairs: usize,
    total_time: f32,
    first_revealed: Option<Entity>,
    input_cooldown: Timer,
    grid_entity: Option<Entity>,
    stage_text_entity: Option<Entity>,
    stopwatch_text_entity: Option<Entity>,
    stage_complete_timer: Option<Timer>,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            mistakes: 0,
            matched_pairs: 0,
            total_time: 0.0,
            first_revealed: None,
            input_cooldown: Timer::from_seconds(0.0, TimerMode::Once),
            grid_entity: None,
            stage_text_entity: None,
            stopwatch_text_entity: None,
            stage_complete_timer: None,
        }
    }
}

use bits_helpers::restart::Restartable;

impl Restartable for GameData {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn initial_state() -> Self::State {
        GameState::Welcome
    }

    type State = GameState;
}

impl Restartable for CurrentStage {
    fn reset(&mut self) {
        *self = Self::new();
    }

    fn initial_state() -> Self::State {
        GameState::Welcome
    }

    type State = GameState;
}

#[derive(Resource)]
struct CountdownTimer(Timer);

#[derive(Component)]
struct CountdownText;

#[derive(Component)]
struct StopwatchText;

#[derive(Component)]
struct StageText;

#[derive(Component)]
struct StageCompleteOverlay;

#[derive(Component)]
struct GameOverElement;

const SHAPES: [Shape; 7] = [
    Shape::Circle,
    Shape::Square,
    Shape::Triangle,
    Shape::Hexagon,
    Shape::Pentagon,
    Shape::Octagon,
    Shape::Quad,
];

const COLORS: [Color; 10] = [
    Color::Srgba(RED),
    Color::Srgba(BLUE),
    Color::Srgba(GREEN),
    Color::Srgba(YELLOW),
    Color::Srgba(ORANGE),
    Color::Srgba(LIME),
    Color::Srgba(MAGENTA),
    Color::Srgba(TEAL),
    Color::Srgba(PINK),
    Color::Srgba(CORAL),
];

#[derive(Resource)]
struct CurrentStage {
    index: usize,
    config: Stage,
}

impl CurrentStage {
    const fn new() -> Self {
        Self {
            index: 0,
            config: STAGES[0],
        }
    }

    fn advance(&mut self) -> bool {
        if self.index + 1 < STAGES.len() {
            self.index += 1;
            self.config = *STAGES.get(self.index).expect("Stage index out of bounds");
            true
        } else {
            false
        }
    }

    const fn is_final(&self) -> bool {
        self.index == STAGES.len() - 1
    }
}

pub fn run() {
    bits_helpers::get_default_app::<ShapeMatcher>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .init_state::<GameState>()
    .insert_resource(CurrentStage::new())
    .insert_resource(GameData::default())
    .add_systems(Startup, setup)
    .add_systems(
        OnEnter(GameState::Welcome),
        (cleanup_marked_entities, spawn_welcome_screen),
    )
    .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
    .add_systems(OnEnter(GameState::StageSetup), setup_stage)
    .add_systems(OnEnter(GameState::Countdown), start_countdown)
    .add_systems(OnEnter(GameState::Playing), start_stopwatch)
    .add_systems(OnEnter(GameState::StageComplete), handle_stage_complete)
    .add_systems(
        Update,
        (
            handle_welcome_input.run_if(in_state(GameState::Welcome)),
            update_countdown.run_if(in_state(GameState::Countdown)),
            (handle_game_input, check_game_over, update_stopwatch)
                .run_if(in_state(GameState::Playing)),
            animate_floating_scores,
            handle_timers,
            handle_restart::<GameData>,
            handle_restart::<CurrentStage>,
        ),
    )
    .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
    .add_systems(OnExit(GameState::GameOver), despawn_game_over_screen)
    .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_welcome_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new("Match the shapes!"),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.0, 0.0),
        WelcomeScreenElement,
    ));
}

fn handle_welcome_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        next_state.set(GameState::StageSetup);
    }
}

fn setup_stage(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    current_stage: Res<CurrentStage>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
    stage_complete_query: Query<Entity, With<StageCompleteOverlay>>,
) {
    // Despawn the stage complete overlay
    for entity in stage_complete_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Clear previous stage entities
    if let Some(grid_entity) = game_data.grid_entity {
        commands.entity(grid_entity).despawn_recursive();
    }

    // Despawn stage text and stopwatch text
    if let Some(stage_text_entity) = game_data.stage_text_entity {
        commands.entity(stage_text_entity).despawn();
    }
    if let Some(stopwatch_text_entity) = game_data.stopwatch_text_entity {
        commands.entity(stopwatch_text_entity).despawn();
    }

    // Reset stage data
    game_data.mistakes = 0;
    game_data.matched_pairs = 0;
    game_data.first_revealed = None;
    game_data.input_cooldown = Timer::from_seconds(0.0, TimerMode::Once);

    let mut rng = rand::thread_rng();
    let mut available_colors = COLORS.to_vec();
    let mut available_shapes = SHAPES.to_vec();

    let total_cells = current_stage.config.grid_cols * current_stage.config.grid_rows;
    let pairs_needed = total_cells / 2;

    // Create all necessary shape-color combinations
    let mut combinations = Vec::new();
    for i in 0..pairs_needed {
        if available_colors.is_empty() {
            available_colors = COLORS.to_vec();
        }
        if available_shapes.is_empty() {
            available_shapes = SHAPES.to_vec();
        }

        let shape = if i < SHAPES.len() {
            // Ensure each shape is used at least once
            available_shapes.remove(0)
        } else {
            available_shapes.remove(rng.gen_range(0..available_shapes.len()))
        };

        let color = available_colors.remove(rng.gen_range(0..available_colors.len()));
        combinations.push((shape, color));
    }

    // Duplicate combinations to create pairs
    combinations.extend(combinations.clone());

    // Shuffle the combinations
    combinations.shuffle(&mut rng);

    // Use all combinations to fill the grid
    let shapes = combinations;

    let grid_entity = commands
        .spawn((Transform::default(), Visibility::default(), CleanupMarker))
        .with_children(|parent| {
            for (i, (shape, color)) in shapes.into_iter().enumerate() {
                let row = i / current_stage.config.grid_cols;
                let col = i % current_stage.config.grid_cols;
                let position = calculate_grid_position(
                    row,
                    col,
                    current_stage.config.grid_cols,
                    current_stage.config.grid_rows,
                );

                let mesh = create_shape_mesh(shape, CELL_SIZE);
                let shape_entity = parent
                    .spawn((
                        Mesh2d(meshes.add(mesh)),
                        MeshMaterial2d(materials.add(ColorMaterial::from(color))),
                        Transform::from_translation(position.extend(0.0)),
                        Visibility::Visible,
                    ))
                    .id();

                let question_mark_entity = parent
                    .spawn((
                        Text2d::new("?"),
                        TextFont {
                            font: asset_server.load(bits_helpers::FONT),
                            font_size: CELL_SIZE * 0.8,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_translation(position.extend(0.0)),
                        Visibility::Hidden,
                    ))
                    .id();

                parent.spawn(GridCell {
                    row,
                    col,
                    shape,
                    color,
                    is_revealed: true,
                    is_matched: false,
                    shape_entity,
                    question_mark_entity,
                    is_locked: false,
                });
            }
        })
        .id();

    game_data.grid_entity = Some(grid_entity);

    // Spawn or update stage text
    let stage_text_entity = commands
        .spawn((
            Text::new(format!("Stage {}", current_stage.index + 1)),
            TextFont {
                font: asset_server.load(bits_helpers::FONT),
                font_size: FONT_SIZE_MEDIUM,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(10.0, 10.0, 0.0),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            StageText,
            CleanupMarker,
        ))
        .id();
    game_data.stage_text_entity = Some(stage_text_entity);

    // Spawn stopwatch text (hidden initially)
    let stopwatch_text_entity = commands
        .spawn((
            Text::new("0.0"),
            TextFont {
                font: asset_server.load(bits_helpers::FONT),
                font_size: FONT_SIZE_MEDIUM,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(10.0, 60.0, 0.0),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                left: Val::Px(10.0),
                ..default()
            },
            StopwatchText,
            CleanupMarker,
        ))
        .id();
    game_data.stopwatch_text_entity = Some(stopwatch_text_entity);

    next_state.set(GameState::Countdown);
}

fn start_countdown(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_stage: Res<CurrentStage>,
) {
    commands.insert_resource(CountdownTimer(Timer::from_seconds(
        COUNTDOWN_DURATION,
        TimerMode::Once,
    )));

    commands.spawn((
        Text::new(format!(
            "Stage {}\n{:.0}",
            current_stage.index + 1,
            COUNTDOWN_DURATION
        )),
        TextFont {
            font: asset_server.load(bits_helpers::FONT),
            font_size: FONT_SIZE_LARGE,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(10.0, 10.0, 0.0),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        CountdownText,
        CleanupMarker,
    ));
}

fn update_countdown(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<CountdownTimer>,
    mut countdown_query: Query<(Entity, &mut Text), With<CountdownText>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cell_query: Query<&mut GridCell>,
    mut visibility_query: Query<&mut Visibility>,
) {
    timer.0.tick(time.delta());

    if let Ok((_, mut text)) = countdown_query.get_single_mut() {
        text.0 = format!("{:.0}", timer.0.remaining_secs().max(0.0));
    }

    if timer.0.finished() {
        for mut cell in &mut cell_query {
            cell.is_revealed = false;
            if let Ok(mut shape_visibility) = visibility_query.get_mut(cell.shape_entity) {
                *shape_visibility = Visibility::Hidden;
            }
            if let Ok(mut question_mark_visibility) =
                visibility_query.get_mut(cell.question_mark_entity)
            {
                *question_mark_visibility = Visibility::Visible;
            }
        }

        // Remove the countdown text entity
        if let Ok((countdown_entity, _)) = countdown_query.get_single_mut() {
            commands.entity(countdown_entity).despawn();
        }

        commands.remove_resource::<CountdownTimer>();
        next_state.set(GameState::Playing);
    }
}

// Replace the TimerBundle usage with a custom component
#[derive(Component)]
struct HideCellsTimer(Timer);

fn handle_game_input(
    mut commands: Commands,
    windows: Query<&Window>,
    mut cell_query: Query<(Entity, &mut GridCell)>,
    mut game_data: ResMut<GameData>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut visibility_query: Query<&mut Visibility>,
    mut current_stage: ResMut<CurrentStage>,
) {
    // Tick the cooldown timer
    game_data.input_cooldown.tick(time.delta());

    // Only process input if the cooldown has finished
    if game_data.input_cooldown.finished() && mouse_button_input.just_pressed(MouseButton::Left) {
        let window = windows.single();
        let Some(position) = window.cursor_position() else {
            return;
        };

        let grid_width = current_stage.config.grid_cols as f32 * CELL_SIZE;
        let grid_height = current_stage.config.grid_rows as f32 * CELL_SIZE;
        let start_x = -grid_width / 2.0;
        let start_y = grid_height / 2.0;

        let x = position.x - window.width() / 2.0;
        let y = window.height() / 2.0 - position.y;

        if x < start_x || x >= start_x + grid_width || y < start_y - grid_height || y >= start_y {
            return;
        }

        let col = ((x - start_x) / CELL_SIZE) as usize;
        let row = ((start_y - y) / CELL_SIZE) as usize;

        let mut revealed_entity = None;
        for (entity, mut cell) in &mut cell_query {
            if cell.row != row || cell.col != col || cell.is_revealed || cell.is_locked {
                continue;
            }
            cell.is_revealed = true;
            revealed_entity = Some(entity);

            // Reveal the shape
            if let Ok(mut shape_visibility) = visibility_query.get_mut(cell.shape_entity) {
                *shape_visibility = Visibility::Visible;
            }
            // Hide the question mark
            if let Ok(mut question_mark_visibility) =
                visibility_query.get_mut(cell.question_mark_entity)
            {
                *question_mark_visibility = Visibility::Hidden;
            }
            break;
        }

        if let Some(revealed_entity) = revealed_entity {
            if let Some(first_revealed) = game_data.first_revealed {
                let [(_, mut first_cell), (_, mut second_cell)] = cell_query
                    .get_many_mut([first_revealed, revealed_entity])
                    .expect("Failed to get both revealed cells");
                if first_cell.shape == second_cell.shape && first_cell.color == second_cell.color {
                    // Match found
                    first_cell.is_matched = true;
                    second_cell.is_matched = true;
                    game_data.matched_pairs += 1;
                    spawn_floating_score(
                        &mut commands,
                        calculate_grid_position(
                            second_cell.row,
                            second_cell.col,
                            current_stage.config.grid_cols,
                            current_stage.config.grid_rows,
                        ),
                        "+1",
                        GREEN,
                        &asset_server,
                    );
                    // Remove the input cooldown for correct matches
                    game_data.input_cooldown = Timer::from_seconds(0.0, TimerMode::Once);
                } else {
                    // No match
                    game_data.mistakes += 1;
                    spawn_floating_score(
                        &mut commands,
                        calculate_grid_position(
                            second_cell.row,
                            second_cell.col,
                            current_stage.config.grid_cols,
                            current_stage.config.grid_rows,
                        ),
                        "-1",
                        RED,
                        &asset_server,
                    );

                    // Set up timer to hide cells
                    commands.spawn((
                        HideCellsTimer(Timer::from_seconds(HIDE_CELLS_DELAY, TimerMode::Once)),
                        HideCellsData {
                            first: first_revealed,
                            second: revealed_entity,
                        },
                    ));

                    // Lock the cells
                    first_cell.is_locked = true;
                    second_cell.is_locked = true;

                    // Apply input cooldown only for mistakes
                    game_data.input_cooldown =
                        Timer::from_seconds(HIDE_CELLS_DELAY, TimerMode::Once);
                }
                game_data.first_revealed = None;
            } else {
                game_data.first_revealed = Some(revealed_entity);
            }
        }

        // Check if the stage is complete
        if game_data.is_stage_complete(current_stage.config) {
            if current_stage.advance() {
                game_data.stage_complete_timer =
                    Some(Timer::from_seconds(STAGE_COMPLETE_DELAY, TimerMode::Once));
                next_state.set(GameState::StageComplete);
            } else {
                next_state.set(GameState::GameOver);
            }
        }
    }
}

// Add this new component to store the entities to hide
#[derive(Component)]
struct HideCellsData {
    first: Entity,
    second: Entity,
}

fn handle_stage_complete(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_stage: Res<CurrentStage>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            StageCompleteOverlay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!(
                    "Stage {} Complete!\nPreparing next stage...",
                    current_stage.index
                )),
                TextFont {
                    font: asset_server.load(bits_helpers::FONT),
                    font_size: FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn handle_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut hide_cells_query: Query<(Entity, &mut HideCellsTimer, &HideCellsData)>,
    mut cell_query: Query<&mut GridCell>,
    mut visibility_query: Query<&mut Visibility>,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>, // Add this line to get the current game state
) {
    // Only process hide cells timers if we're not in the GameOver state
    if *current_state.get() != GameState::GameOver {
        for (entity, mut timer, hide_cells_data) in &mut hide_cells_query {
            timer.0.tick(time.delta());
            if timer.0.finished() {
                handle_hide_cells(hide_cells_data, &mut cell_query, &mut visibility_query);
                commands.entity(entity).despawn();
            }
        }
    } else {
        // If we're in GameOver state, despawn all hide cells timers without hiding the cells
        for (entity, _, _) in hide_cells_query.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Handle stage complete timer
    if let Some(ref mut timer) = game_data.stage_complete_timer {
        timer.tick(time.delta());
        if timer.finished() {
            next_state.set(GameState::StageSetup);
            game_data.stage_complete_timer = None;
        }
    }
}

fn handle_hide_cells(
    hide_cells_data: &HideCellsData,
    cell_query: &mut Query<&mut GridCell>,
    visibility_query: &mut Query<&mut Visibility>,
) {
    for &cell_entity in &[hide_cells_data.first, hide_cells_data.second] {
        if let Ok(mut cell) = cell_query.get_mut(cell_entity) {
            if !cell.is_matched {
                cell.is_revealed = false;
                cell.is_locked = false;
                if let Ok(mut shape_visibility) = visibility_query.get_mut(cell.shape_entity) {
                    *shape_visibility = Visibility::Hidden;
                }
                if let Ok(mut question_mark_visibility) =
                    visibility_query.get_mut(cell.question_mark_entity)
                {
                    *question_mark_visibility = Visibility::Visible;
                }
            }
        }
    }
}

impl GameData {
    const fn is_stage_complete(&self, stage: Stage) -> bool {
        self.matched_pairs * 2 == stage.grid_cols * stage.grid_rows
    }
}

fn check_game_over(
    game_data: Res<GameData>,
    current_stage: Res<CurrentStage>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cell_query: Query<&mut GridCell>,
    mut visibility_query: Query<&mut Visibility>,
) {
    if game_data.mistakes >= MAX_MISTAKES {
        // Reveal all shapes
        for mut cell in &mut cell_query {
            cell.is_revealed = true;
            if let Ok(mut shape_visibility) = visibility_query.get_mut(cell.shape_entity) {
                *shape_visibility = Visibility::Visible;
            }
            if let Ok(mut question_mark_visibility) =
                visibility_query.get_mut(cell.question_mark_entity)
            {
                *question_mark_visibility = Visibility::Hidden;
            }
        }
        next_state.set(GameState::GameOver);
    } else if current_stage.is_final() && game_data.is_stage_complete(current_stage.config) {
        next_state.set(GameState::GameOver);
    }
}

fn update_stopwatch(
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    mut query: Query<&mut Text, With<StopwatchText>>,
    game_state: Res<State<GameState>>,
) {
    if *game_state.get() == GameState::Playing {
        game_data.total_time += time.delta_secs();
        if let Ok(mut text) = query.get_single_mut() {
            text.0 = format!("{:.1}", game_data.total_time);
        }
    }
}

fn start_stopwatch(mut query: Query<&mut Visibility, With<StopwatchText>>) {
    if let Ok(mut visibility) = query.get_single_mut() {
        *visibility = Visibility::Visible;
    }
}

fn despawn_game_over_screen(
    mut commands: Commands,
    game_over_elements: Query<Entity, With<GameOverElement>>,
) {
    for entity in game_over_elements.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_data: Res<GameData>,
    current_stage: Res<CurrentStage>,
) {
    let message = if game_data.mistakes >= MAX_MISTAKES {
        format!(
            "Game Over!\nToo many mistakes.\nReached Stage {}",
            current_stage.index + 1
        )
    } else {
        format!(
            "Congratulations!\nYou completed all stages!\nTotal time: {:.1} seconds",
            game_data.total_time
        )
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GameOverElement,
            CleanupMarker,
        ))
        .with_children(|parent| {
            // Game over text
            parent.spawn((
                Text::new(message),
                TextFont {
                    font: asset_server.load(bits_helpers::FONT),
                    font_size: FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    margin: UiRect::top(Val::Px(100.0)),
                    ..default()
                },
            ));

            // Restart button
            parent
                .spawn((
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(50.0)),
                        ..default()
                    },
                    BackgroundColor::from(Color::BLACK),
                    RestartButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Restart"),
                        TextFont {
                            font: asset_server.load(bits_helpers::FONT),
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn calculate_grid_position(row: usize, col: usize, grid_cols: usize, grid_rows: usize) -> Vec2 {
    if grid_cols == 0 || grid_rows == 0 {
        return Vec2::ZERO;
    }
    let grid_width = grid_cols as f32 * CELL_SIZE;
    let grid_height = grid_rows as f32 * CELL_SIZE;
    let start_x = -grid_width / 2.0 + CELL_SIZE / 2.0;
    let start_y = grid_height / 2.0 - CELL_SIZE / 2.0;
    Vec2::new(
        (col as f32).mul_add(CELL_SIZE, start_x),
        (row as f32).mul_add(-CELL_SIZE, start_y),
    )
}

fn create_shape_mesh(shape_type: Shape, cell_size: f32) -> Mesh {
    let shape_size = cell_size * SHAPE_SCALE_FACTOR;

    match shape_type {
        Shape::Circle => Mesh::from(bevy::math::primitives::Circle::new(shape_size / 2.0)),
        Shape::Square => Mesh::from(bevy::math::primitives::Rectangle::new(
            shape_size, shape_size,
        )),
        Shape::Triangle => Mesh::from(bevy::math::primitives::RegularPolygon::new(
            shape_size / 2.0,
            3,
        )),
        Shape::Hexagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(
            shape_size / 2.0,
            6,
        )),
        Shape::Pentagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(
            shape_size / 2.0,
            5,
        )),
        Shape::Octagon => Mesh::from(bevy::math::primitives::RegularPolygon::new(
            shape_size / 2.0,
            8,
        )),
        Shape::Quad => Mesh::from(bevy::math::primitives::Rectangle::new(
            shape_size,
            shape_size * 0.75,
        )),
    }
}
