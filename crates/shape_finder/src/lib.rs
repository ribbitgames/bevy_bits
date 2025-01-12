use bevy::color::palettes::css::{GREEN, RED};
use bevy::prelude::*;
use bits_helpers::emoji::{self, EmojiPlugin};
use bits_helpers::floating_score::{animate_floating_scores, spawn_floating_score};
use bits_helpers::{FONT, WINDOW_HEIGHT, WINDOW_WIDTH};
use rand::seq::SliceRandom;
use rand::Rng;
use ribbit::ShapeFinder;

mod ribbit;

// Game constants
const GAME_DURATION: f32 = 20.0;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
    StageTransition,
}

#[derive(Component)]
struct MovingEmoji {
    index: usize,
    size: f32,
}

#[derive(Resource, Default)]
struct CorrectEmojisFound(usize);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Resource)]
struct GameTimer(Timer);

impl Default for GameTimer {
    fn default() -> Self {
        // We'll update this in a moment to use stage_config
        Self(Timer::new(
            std::time::Duration::from_secs_f32(20.0), // Default fallback value
            TimerMode::Once,
        ))
    }
}

#[derive(Resource, Default)]
struct Score(i32);

#[derive(Resource, Default)]
struct TargetEmojiInfo {
    index: usize,
}

#[derive(Event)]
struct EmojiClickedEvent {
    entity: Entity,
    position: Vec2,
    is_correct: bool,
}

#[derive(Component)]
struct WelcomeScreen;

#[derive(Clone)]
pub struct Stage {
    pub total_emojis: usize,
    pub correct_emojis: usize,
    pub emoji_speed: f32,
    pub time_limit: f32,
}

#[derive(Resource)]
pub struct StageConfig {
    pub stage: Stage,
    pub current_stage_number: usize,
}

impl Default for StageConfig {
    fn default() -> Self {
        Self {
            stage: Stage {
                total_emojis: 30,
                correct_emojis: 5,
                emoji_speed: 100.0,
                time_limit: 20.0,
            },
            current_stage_number: 1,
        }
    }
}

#[derive(Component)]
struct StageTransitionTimer(Timer);

impl Default for StageTransitionTimer {
    fn default() -> Self {
        Self(Timer::new(
            std::time::Duration::from_secs_f32(2.0),
            TimerMode::Once,
        ))
    }
}

pub fn run() {
    let mut app = bits_helpers::get_default_app::<ShapeFinder>(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );

    app.add_plugins(EmojiPlugin)
        .init_state::<GameState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        .init_resource::<TargetEmojiInfo>()
        .init_resource::<CorrectEmojisFound>()
        .init_resource::<StageConfig>()
        .add_event::<EmojiClickedEvent>()
        .add_systems(Startup, setup)
        .add_systems(OnExit(GameState::Welcome), despawn_welcome_screen)
        .add_systems(OnEnter(GameState::Playing), (spawn_emojis, spawn_timer))
        // Add cleanup system for Playing state
        .add_systems(OnExit(GameState::Playing), cleanup_stage)
        // Add systems for StageTransition
        .add_systems(OnEnter(GameState::StageTransition), handle_stage_transition)
        .add_systems(Update, handle_emoji_clicked.after(handle_playing_input))
        .add_systems(
            Update,
            (
                try_spawn_welcome_screen.run_if(in_state(GameState::Welcome)),
                handle_welcome_input.run_if(in_state(GameState::Welcome)),
                (
                    handle_playing_input,
                    move_emojis,
                    update_timer,
                    animate_floating_scores,
                )
                    .run_if(in_state(GameState::Playing)),
            ),
        );

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn try_spawn_welcome_screen(
    mut commands: Commands,
    atlas: Option<Res<emoji::EmojiAtlas>>,
    validation: Option<Res<emoji::AtlasValidation>>,
    mut target_info: ResMut<TargetEmojiInfo>,
    asset_server: Res<AssetServer>,
    welcome_screen: Query<&WelcomeScreen>,
) {
    // If we already have a welcome screen, don't spawn another
    if !welcome_screen.is_empty() {
        return;
    }

    let (Some(atlas), Some(validation)) = (atlas, validation) else {
        return;
    };

    if !emoji::is_emoji_system_ready(&validation) {
        return;
    }

    // Select random emoji index for target
    let indices = emoji::get_random_emojis(&atlas, &validation, 1);
    let Some(&index) = indices.first() else {
        return;
    };

    target_info.index = index;

    // Spawn welcome screen entity
    let welcome_screen_entity = commands
        .spawn((WelcomeScreen, Transform::default(), Visibility::default()))
        .id();

    // Spawn text as child
    commands
        .spawn((
            Text2d::new("Find this emoji!"),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 32.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(0.0, WINDOW_HEIGHT / 4.0, 0.0)),
        ))
        .set_parent(welcome_screen_entity);

    // Spawn emoji and make it a child of welcome screen
    if let Some(emoji_entity) = emoji::spawn_emoji(
        &mut commands,
        &atlas,
        &validation,
        index,
        Vec2::new(0.0, 0.0),
        1.0,
    ) {
        commands
            .entity(emoji_entity)
            .set_parent(welcome_screen_entity);
    }
}

fn handle_welcome_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_timer: ResMut<GameTimer>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        game_timer.0.reset();
        next_state.set(GameState::Playing);
    }
}

fn spawn_emojis(
    mut commands: Commands,
    atlas: Res<emoji::EmojiAtlas>,
    validation: Res<emoji::AtlasValidation>,
    target_info: Res<TargetEmojiInfo>,
    stage_config: Res<StageConfig>,
) {
    if !emoji::is_emoji_system_ready(&validation) {
        return;
    }

    let mut rng = rand::thread_rng();
    let mut emojis = Vec::new();

    // Add correct emojis
    for _ in 0..stage_config.stage.correct_emojis {
        emojis.push(target_info.index);
    }

    // Add other random emojis
    let other_indices = emoji::get_random_emojis(
        &atlas,
        &validation,
        stage_config.stage.total_emojis - stage_config.stage.correct_emojis,
    );
    emojis.extend(other_indices);
    emojis.shuffle(&mut rng);

    // Spawn all emojis
    for &index in &emojis {
        let size = rng.gen_range(40.0..80.0);
        let x = rng.gen_range(-WINDOW_WIDTH / 2.0 + size..WINDOW_WIDTH / 2.0 - size);
        let y = rng.gen_range(-WINDOW_HEIGHT / 2.0 + size..WINDOW_HEIGHT / 2.0 - size);
        let velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize()
            * stage_config.stage.emoji_speed;

        if let Some(entity) = emoji::spawn_emoji(
            &mut commands,
            &atlas,
            &validation,
            index,
            Vec2::new(x, y),
            size / 128.0,
        ) {
            commands
                .entity(entity)
                .insert((MovingEmoji { index, size }, Velocity(velocity)));
        }
    }
}

fn spawn_timer(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stage_config: Res<StageConfig>,
) {
    commands.spawn((
        Text2d::new(format!("Time: {:.1}", stage_config.stage.time_limit)),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Right),
        Transform::from_translation(Vec3::new(WINDOW_WIDTH / 2.2, WINDOW_HEIGHT / 2.2, 0.0)),
    ));
}

fn update_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut timer_text: Query<&mut Text2d>,
    stage_config: Res<StageConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    correct_emojis_found: Res<CorrectEmojisFound>,
) {
    game_timer.0.tick(time.delta());
    let remaining_time = stage_config.stage.time_limit - game_timer.0.elapsed_secs();

    if let Ok(mut text) = timer_text.get_single_mut() {
        *text = Text2d::new(format!("Time: {:.1}", remaining_time.max(0.0)));
    }

    // Check if timer is finished or all emojis are found
    if game_timer.0.just_finished() || correct_emojis_found.0 >= stage_config.stage.correct_emojis {
        next_state.set(GameState::StageTransition);
    }
}

fn handle_playing_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    emojis: Query<(Entity, &Transform, &MovingEmoji)>,
    target_info: Res<TargetEmojiInfo>,
    mut emoji_clicked_events: EventWriter<EmojiClickedEvent>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera_q.single();
        let window = windows.single();

        let Some(cursor_position) = window.cursor_position() else {
            return;
        };

        if let Some(world_position) = camera
            .viewport_to_world(camera_transform, cursor_position)
            .ok()
            .map(|ray| ray.origin.truncate())
        {
            for (entity, transform, emoji) in emojis.iter() {
                if transform.translation.truncate().distance(world_position) < emoji.size / 2.0 {
                    let is_correct = emoji.index == target_info.index;
                    emoji_clicked_events.send(EmojiClickedEvent {
                        entity,
                        position: cursor_position,
                        is_correct,
                    });
                    break;
                }
            }
        }
    }
}

fn handle_emoji_clicked(
    mut commands: Commands,
    mut emoji_clicked_events: EventReader<EmojiClickedEvent>,
    mut score: ResMut<Score>,
    mut correct_emojis_found: ResMut<CorrectEmojisFound>,
    stage_config: Res<StageConfig>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in emoji_clicked_events.read() {
        if event.is_correct {
            score.0 += 1;
            correct_emojis_found.0 += 1;
            spawn_floating_score(&mut commands, event.position, "+1", GREEN, &asset_server);
            if correct_emojis_found.0 >= stage_config.stage.correct_emojis {
                next_state.set(GameState::StageTransition);
            }
        } else {
            score.0 -= 1;
            spawn_floating_score(&mut commands, event.position, "-1", RED, &asset_server);
        }
        commands.entity(event.entity).despawn();
    }
}

fn move_emojis(mut query: Query<(&mut Transform, &mut Velocity, &MovingEmoji)>, time: Res<Time>) {
    // Move emojis and handle wall collisions only
    for (mut transform, mut velocity, emoji) in &mut query {
        let mut new_pos = transform.translation + velocity.0.extend(0.0) * time.delta_secs();

        // Handle wall collisions
        if new_pos.x - emoji.size / 2.0 < -WINDOW_WIDTH / 2.0
            || new_pos.x + emoji.size / 2.0 > WINDOW_WIDTH / 2.0
        {
            new_pos.x = new_pos.x.clamp(
                -WINDOW_WIDTH / 2.0 + emoji.size / 2.0,
                WINDOW_WIDTH / 2.0 - emoji.size / 2.0,
            );
            velocity.0.x *= -1.0;
        }
        if new_pos.y - emoji.size / 2.0 < -WINDOW_HEIGHT / 2.0
            || new_pos.y + emoji.size / 2.0 > WINDOW_HEIGHT / 2.0
        {
            new_pos.y = new_pos.y.clamp(
                -WINDOW_HEIGHT / 2.0 + emoji.size / 2.0,
                WINDOW_HEIGHT / 2.0 - emoji.size / 2.0,
            );
            velocity.0.y *= -1.0;
        }

        transform.translation = new_pos;
    }
}

fn spawn_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
) {
    commands.spawn((
        Text2d::new(format!("Game Over!\nYour score: {}", score.0)),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::from_translation(Vec3::ZERO),
    ));

    // Spawn the transition timer
    commands.spawn(StageTransitionTimer::default());
}

fn despawn_welcome_screen(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Text2d>, With<emoji::EmojiSprite>)>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_stage(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Text2d>, With<MovingEmoji>, With<emoji::EmojiSprite>)>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// Modify handle_stage_transition to be an immediate transition system
fn handle_stage_transition(
    mut commands: Commands,
    mut stage_config: ResMut<StageConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Update stage config
    stage_config.current_stage_number += 1;
    stage_config.stage.emoji_speed *= 1.2; // Increase speed by 20%
    stage_config.stage.total_emojis += 5; // Add 5 more emojis
    stage_config.stage.time_limit *= 0.9; // Reduce time by 10%

    // Reset correct emojis found
    commands.insert_resource(CorrectEmojisFound(0));

    // Reset the game timer
    commands.insert_resource(GameTimer(Timer::new(
        std::time::Duration::from_secs_f32(stage_config.stage.time_limit),
        TimerMode::Once,
    )));

    // Move to Playing state immediately
    next_state.set(GameState::Playing);
}
