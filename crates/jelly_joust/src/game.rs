use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use bits_helpers::emoji::{AtlasValidation, EmojiAtlas, spawn_emoji};
use bits_helpers::floating_score::spawn_floating_score;
use bits_helpers::input::{
    just_pressed_world_position, just_released_world_position, pressed_world_position,
};
use bits_helpers::welcome_screen::{
    WelcomeScreenElement, despawn_welcome_screen, spawn_welcome_screen_shape,
};
use bits_helpers::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::new(0.0, -600.0)))
            .init_state::<GameState>()
            .insert_resource(Score::default())
            .insert_resource(PlatformTiltState::default())
            .insert_resource(SwipeState::default())
            .insert_resource(AiState::default())
            .add_systems(OnEnter(GameState::Welcome), spawn_welcome_screen)
            .add_systems(Update, start_game.run_if(in_state(GameState::Welcome)))
            .add_systems(OnEnter(GameState::Playing), spawn_game_entities)
            .add_systems(
                Update,
                (player_swing, opponent_ai, check_falls, animate_platform)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::GameOver), cleanup_game_entities);
    }
}

// Game states
#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    GameOver,
}

// Components
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Opponent;

#[derive(Component)]
struct Platform;

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Sword {
    is_player: bool, // Removed unused `owner` field
}

#[derive(Resource, Default)]
pub struct Score {
    pub player: u32,
    pub opponent: u32,
}

#[derive(Resource, Default)]
struct PlatformTiltState {
    current_tilt: f32,
}

#[derive(Resource, Default)]
struct SwipeState {
    start_pos: Option<Vec2>,
}

#[derive(Resource, Default)]
struct AiState {
    start_timer: Option<Timer>,
}

const KNIGHT_SIZE: f32 = 50.0;
const PLATFORM_WIDTH: f32 = WINDOW_WIDTH * 0.8;
const PLATFORM_HEIGHT: f32 = 20.0;
const SWORD_LENGTH: f32 = 40.0;
const GROUND_HEIGHT: f32 = 20.0;

fn spawn_welcome_screen(
    commands: Commands,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_welcome_screen_shape(
        commands,
        asset_server,
        meshes,
        materials,
        "Swipe to swing your sword!",
        Mesh::from(Circle::new(KNIGHT_SIZE)),
        Color::srgb(0.0, 1.0, 0.0),
    );
}

fn start_game(
    mut next_state: ResMut<NextState<GameState>>,
    touch_input: Res<Touches>,
    button_input: Res<ButtonInput<MouseButton>>,
    welcome_query: Query<Entity, With<WelcomeScreenElement>>,
    commands: Commands,
) {
    if touch_input.any_just_pressed() || button_input.just_pressed(MouseButton::Left) {
        despawn_welcome_screen(commands, welcome_query);
        next_state.set(GameState::Playing);
    }
}

fn spawn_game_entities(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    mut ai_state: ResMut<AiState>,
) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.2),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, GROUND_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_HEIGHT * 0.45, 0.0),
        Visibility::Visible,
        RigidBody::Static,
        Collider::rectangle(WINDOW_WIDTH, GROUND_HEIGHT),
        Friction::new(1.0),
        Ground,
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.3, 0.1),
            custom_size: Some(Vec2::new(PLATFORM_WIDTH, PLATFORM_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, -WINDOW_HEIGHT * 0.3, 0.0),
        Visibility::Visible,
        RigidBody::Static,
        LockedAxes::new().lock_rotation(),
        Collider::rectangle(PLATFORM_WIDTH, PLATFORM_HEIGHT),
        Restitution::new(0.3),
        Friction::new(1.0),
        Platform,
    ));

    let emoji_indices = bits_helpers::emoji::get_random_emojis(&atlas, &validation, 2);

    let platform_y = -WINDOW_HEIGHT * 0.3;
    let player = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::circle(KNIGHT_SIZE / 2.0),
            Restitution::new(0.3),
            Friction::new(1.0),
            Mass(10.0),
            Transform::from_xyz(
                -PLATFORM_WIDTH * 0.25,
                platform_y + PLATFORM_HEIGHT / 2.0 + KNIGHT_SIZE / 2.0,
                0.0,
            ),
            Visibility::Visible,
            Player,
        ))
        .id();
    if let Some(index) = emoji_indices.first() {
        if let Some(emoji_entity) = spawn_emoji(
            &mut commands,
            &atlas,
            &validation,
            *index,
            Transform::from_scale(Vec3::splat(KNIGHT_SIZE / 64.0)),
        ) {
            commands.entity(player).add_child(emoji_entity);
        }
    }

    let opponent = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::circle(KNIGHT_SIZE / 2.0),
            Restitution::new(0.3),
            Friction::new(1.0),
            Mass(10.0),
            Transform::from_xyz(
                PLATFORM_WIDTH * 0.25,
                platform_y + PLATFORM_HEIGHT / 2.0 + KNIGHT_SIZE / 2.0,
                0.0,
            ),
            Visibility::Visible,
            Opponent,
        ))
        .id();
    if let Some(index) = emoji_indices.get(1) {
        if let Some(emoji_entity) = spawn_emoji(
            &mut commands,
            &atlas,
            &validation,
            *index,
            Transform::from_scale(Vec3::splat(KNIGHT_SIZE / 64.0)),
        ) {
            commands.entity(opponent).add_child(emoji_entity);
        }
    }

    let player_sword = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(SWORD_LENGTH, 10.0)),
                ..default()
            },
            Transform::from_xyz(
                (-PLATFORM_WIDTH).mul_add(0.25, KNIGHT_SIZE / 2.0),
                platform_y + PLATFORM_HEIGHT / 2.0 + KNIGHT_SIZE / 2.0,
                0.0,
            ),
            Visibility::Visible,
            RigidBody::Dynamic,
            Collider::rectangle(SWORD_LENGTH, 10.0),
            Mass(1.0),
            Sword { is_player: true },
        ))
        .id();
    commands.spawn(
        RevoluteJoint::new(player, player_sword)
            .with_local_anchor_1(Vec2::new(KNIGHT_SIZE / 2.0, 0.0))
            .with_local_anchor_2(Vec2::new(-SWORD_LENGTH / 2.0, 0.0))
            .with_angle_limits(-1.0, 1.0),
    );

    let opponent_sword = commands
        .spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(SWORD_LENGTH, 10.0)),
                ..default()
            },
            Transform::from_xyz(
                PLATFORM_WIDTH.mul_add(0.25, -(KNIGHT_SIZE / 2.0)),
                platform_y + PLATFORM_HEIGHT / 2.0 + KNIGHT_SIZE / 2.0,
                0.0,
            ),
            Visibility::Visible,
            RigidBody::Dynamic,
            Collider::rectangle(SWORD_LENGTH, 10.0),
            Mass(1.0),
            Sword { is_player: false },
        ))
        .id();
    commands.spawn(
        RevoluteJoint::new(opponent, opponent_sword)
            .with_local_anchor_1(Vec2::new(-KNIGHT_SIZE / 2.0, 0.0))
            .with_local_anchor_2(Vec2::new(SWORD_LENGTH / 2.0, 0.0))
            .with_angle_limits(-1.0, 1.0),
    );

    // Start AI timer
    ai_state.start_timer = Some(Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once));
}

fn player_swing(
    mut swords: Query<(&Sword, &mut AngularVelocity), With<Sword>>,
    button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    touch_input: Res<Touches>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut swipe_state: ResMut<SwipeState>,
) {
    if key_input.just_pressed(KeyCode::KeyS) {
        for (sword, mut ang_vel) in &mut swords {
            if sword.is_player {
                ang_vel.0 = 30.0;
            }
        }
        return;
    }

    if let Some(start_pos) =
        just_pressed_world_position(&button_input, &touch_input, &windows, &camera)
    {
        swipe_state.start_pos = Some(start_pos);
    }

    if let Some(start_pos) = swipe_state.start_pos {
        if let Some(current_pos) =
            pressed_world_position(&button_input, &touch_input, &windows, &camera)
        {
            let swipe = current_pos - start_pos;
            for (sword, mut ang_vel) in &mut swords {
                if sword.is_player {
                    let swing_strength = (swipe.x / 50.0).clamp(-1.0, 1.0) * 30.0;
                    ang_vel.0 = swing_strength;
                }
            }
        }

        if just_released_world_position(&button_input, &touch_input, &windows, &camera).is_some()
            || button_input.just_released(MouseButton::Left)
            || touch_input.any_just_released()
            || touch_input.any_just_canceled()
        {
            swipe_state.start_pos = None;
        }
    }
}

fn opponent_ai(
    mut swords: Query<(&Sword, &mut AngularVelocity), With<Sword>>,
    time: Res<Time>,
    mut ai_state: ResMut<AiState>,
) {
    if ai_state.start_timer.is_some() {
        if let Some(timer) = &mut ai_state.start_timer {
            timer.tick(time.delta());
            if timer.just_finished() {
                ai_state.start_timer = None;
            } else {
                return;
            }
        }
    }

    let swing = (time.elapsed_secs() * 2.0).sin() * 10.0;
    for (sword, mut ang_vel) in &mut swords {
        if !sword.is_player {
            ang_vel.0 = swing;
        }
    }
}

fn check_falls(
    mut commands: Commands,
    mut score: ResMut<Score>,
    knights: Query<
        (
            Entity,
            &Transform,
            &GlobalTransform,
            Option<&Player>,
            Option<&Opponent>,
        ),
        (With<RigidBody>, Without<Platform>),
    >,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let fall_threshold = -WINDOW_HEIGHT * 0.5;
    for (entity, transform, global_transform, player, opponent) in knights.iter() {
        if transform.translation.y < fall_threshold {
            let position = global_transform.translation().truncate();
            if player.is_some() {
                score.opponent += 1;
                spawn_floating_score(
                    &mut commands,
                    position,
                    "Opponent +1",
                    Srgba::RED,
                    &asset_server,
                );
                next_state.set(GameState::GameOver);
            } else if opponent.is_some() {
                score.player += 1;
                spawn_floating_score(
                    &mut commands,
                    position,
                    "Player +1",
                    Srgba::GREEN,
                    &asset_server,
                );
                next_state.set(GameState::GameOver);
            }
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn animate_platform(
    mut platforms: Query<&mut Transform, With<Platform>>,
    knights: Query<&Transform, (With<RigidBody>, Without<Platform>)>,
    time: Res<Time>,
    mut tilt_state: ResMut<PlatformTiltState>,
) {
    for mut platform_transform in &mut platforms {
        let mut total_offset = 0.0;
        for knight_transform in knights.iter() {
            let relative_x = knight_transform.translation.x - platform_transform.translation.x;
            total_offset += relative_x / (PLATFORM_WIDTH * 0.5);
        }
        let target_tilt = total_offset.clamp(-0.2, 0.2) * 0.15;
        let smoothing_factor = 5.0 * time.delta_secs();
        tilt_state.current_tilt = tilt_state.current_tilt.lerp(target_tilt, smoothing_factor);
        platform_transform.rotation = Quat::from_rotation_z(tilt_state.current_tilt);
    }
}

fn cleanup_game_entities(
    mut commands: Commands,
    entities: Query<
        Entity,
        Or<(
            With<Player>,
            With<Opponent>,
            With<Platform>,
            With<Sword>,
            With<Ground>,
        )>,
    >,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
