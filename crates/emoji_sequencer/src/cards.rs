use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use rand::prelude::*;

use crate::game::{
    GameDifficulty, GameProgress, GameState, SequenceState, SequenceStep, StageState,
};

pub const CARD_BACK: &str = "card_back.png";
pub const DEFAULT_COLOR: Color = Color::WHITE;
pub const WRONG_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
pub const CORRECT_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
const SEQUENCE_CARD_Y: f32 = -200.0; // Position for bottom sequence cards
const SEQUENCE_CARD_SPACING: f32 = 100.0; // Horizontal spacing between sequence cards

#[derive(Component, Debug)]
pub struct Card {
    pub emoji_index: usize,
    pub face_up: bool,
    pub sequence_position: Option<usize>, // None for grid cards, Some(index) for sequence cards
    pub locked: bool,                     // Track if this card has been correctly selected
}

#[derive(Component)]
pub struct CardFace;

#[derive(Component)]
pub struct CardBack;

#[derive(Resource)]
pub struct CardBackTexture(Handle<Image>);

#[derive(Bundle)]
struct CardBackBundle {
    sprite: Sprite,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    card_back: CardBack,
}

pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SequenceState>()
            .init_resource::<GameDifficulty>()
            .insert_resource(GameState::default())
            .add_systems(Startup, setup_cards)
            .add_systems(
                Update,
                (
                    handle_sequence_spawn,
                    handle_sequence_reveal,
                    handle_sequence_hide,
                    handle_grid_spawn,
                    update_card_visibility,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_stage);
    }
}

fn cleanup_stage(mut commands: Commands, cards: Query<Entity, With<Card>>) {
    for card_entity in cards.iter() {
        commands.entity(card_entity).despawn_recursive();
    }
}

fn setup_cards(mut commands: Commands, asset_server: Res<AssetServer>) {
    let card_back = asset_server.load(CARD_BACK);
    commands.insert_resource(CardBackTexture(card_back));
}

/// Handles initial spawning of sequence cards
fn handle_sequence_spawn(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    card_back: Res<CardBackTexture>,
    mut sequence_state: ResMut<SequenceState>,
    mut game_progress: ResMut<GameProgress>,
    difficulty: Res<GameDifficulty>,
    query: Query<(Entity, &Card)>,
) {
    if game_progress.sequence_step != SequenceStep::SpawningSequence {
        return;
    }

    // Only spawn if no sequence cards exist
    if !query
        .iter()
        .any(|(_, card)| card.sequence_position.is_some())
    {
        let sequence_length = difficulty.sequence_length as usize;

        // Get random emojis for the sequence
        let available_indices = emoji::get_random_emojis(&atlas, &validation, sequence_length);
        sequence_state.target_sequence = available_indices.clone();

        // Calculate total width of all cards
        let total_width = SEQUENCE_CARD_SPACING * (sequence_length as f32 - 1.0);
        let start_x = -total_width / 2.0;

        // Spawn sequence cards
        for (i, &emoji_index) in available_indices.iter().enumerate() {
            let x = start_x + (i as f32 * SEQUENCE_CARD_SPACING);

            // Spawn card parent
            let card_entity = commands
                .spawn((
                    Card {
                        emoji_index,
                        face_up: false,
                        sequence_position: Some(i),
                        locked: false,
                    },
                    Transform::from_xyz(x, SEQUENCE_CARD_Y, 0.0).with_scale(Vec3::splat(0.5)),
                    Visibility::Inherited,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();

            // Spawn emoji sprite (face)
            if let Some(emoji_entity) = emoji::spawn_emoji(
                &mut commands,
                &atlas,
                &validation,
                emoji_index,
                Vec2::ZERO,
                0.5,
            ) {
                commands
                    .entity(emoji_entity)
                    .insert(CardFace)
                    .insert(Visibility::Hidden)
                    .insert(Transform::from_translation(Vec3::ZERO));
                commands.entity(card_entity).add_child(emoji_entity);
            }

            // Spawn card back
            let card_back_entity = commands
                .spawn(CardBackBundle {
                    sprite: Sprite {
                        image: card_back.0.clone(),
                        custom_size: Some(Vec2::splat(70.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::ZERO),
                    global_transform: GlobalTransform::default(),
                    visibility: Visibility::Visible,
                    inherited_visibility: InheritedVisibility::default(),
                    view_visibility: ViewVisibility::default(),
                    card_back: CardBack,
                })
                .id();

            commands.entity(card_entity).add_child(card_back_entity);
        }

        game_progress.sequence_step = SequenceStep::RevealingSequence;
        game_progress.step_timer = Some(Timer::from_seconds(
            crate::game::REVEAL_TIME_PER_EMOJI,
            TimerMode::Once,
        ));
    }
}

/// Handles revealing sequence cards one by one
fn handle_sequence_reveal(
    time: Res<Time>,
    mut game_progress: ResMut<GameProgress>,
    mut cards: Query<&mut Card>,
) {
    if game_progress.sequence_step != SequenceStep::RevealingSequence {
        return;
    }

    if let Some(timer) = &mut game_progress.step_timer {
        if timer.tick(time.delta()).just_finished() {
            // Hide previous card if not first card
            if game_progress.current_reveal_index > 0 {
                for mut card in &mut cards {
                    if card.sequence_position == Some(game_progress.current_reveal_index - 1) {
                        card.face_up = false;
                    }
                }
            }

            // Show next card if available
            if game_progress.current_reveal_index < 3 {
                for mut card in &mut cards {
                    if card.sequence_position == Some(game_progress.current_reveal_index) {
                        card.face_up = true;
                    }
                }
                game_progress.current_reveal_index += 1;
                game_progress.step_timer = Some(Timer::from_seconds(
                    crate::game::REVEAL_TIME_PER_EMOJI,
                    TimerMode::Once,
                ));
            } else {
                // All cards revealed, transition to hiding phase
                game_progress.sequence_step = SequenceStep::HidingSequence;
                game_progress.step_timer = Some(Timer::from_seconds(
                    crate::game::SEQUENCE_COMPLETE_DELAY,
                    TimerMode::Once,
                ));
            }
        }
    }
}

/// Handles hiding all sequence cards after reveal
fn handle_sequence_hide(
    time: Res<Time>,
    mut game_progress: ResMut<GameProgress>,
    mut cards: Query<&mut Card>,
) {
    if game_progress.sequence_step != SequenceStep::HidingSequence {
        return;
    }

    if let Some(timer) = &mut game_progress.step_timer {
        if timer.tick(time.delta()).just_finished() {
            // Hide all sequence cards
            for mut card in &mut cards {
                if card.sequence_position.is_some() {
                    card.face_up = false;
                }
            }
            game_progress.sequence_step = SequenceStep::SpawningGrid;
            game_progress.step_timer = None;
        }
    }
}

/// Spawns the grid of face-up cards in the center of the screen
fn handle_grid_spawn(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    difficulty: Res<GameDifficulty>,
    sequence_state: Res<SequenceState>,
    mut game_progress: ResMut<GameProgress>,
    query: Query<(Entity, &Card)>,
) {
    if game_progress.sequence_step != SequenceStep::SpawningGrid {
        return;
    }

    // Only spawn grid cards when no grid cards exist
    if !query
        .iter()
        .any(|(_, card)| card.sequence_position.is_none())
    {
        // Get emoji indices for grid (including sequence emojis plus additional random ones)
        let mut grid_indices = sequence_state.target_sequence.clone();
        let extra_emojis = emoji::get_random_emojis(
            &atlas,
            &validation,
            difficulty.total_emojis - difficulty.sequence_length as usize,
        );
        grid_indices.extend(extra_emojis);
        grid_indices.shuffle(&mut rand::thread_rng());

        // Calculate grid layout
        let grid_width = difficulty.grid_cols as f32 * difficulty.grid_spacing;
        let grid_height = difficulty.grid_rows as f32 * difficulty.grid_spacing;
        let start_x = -grid_width / 2.0;
        let start_y = grid_height / 2.0;

        // Spawn grid cards
        for row in 0..difficulty.grid_rows {
            for col in 0..difficulty.grid_cols {
                let index = (row * difficulty.grid_cols + col) as usize;
                if let Some(&emoji_index) = grid_indices.get(index) {
                    let x = (col as f32).mul_add(difficulty.grid_spacing, start_x)
                        + difficulty.grid_spacing / 2.0;
                    let y = (-(row as f32)).mul_add(difficulty.grid_spacing, start_y)
                        - difficulty.grid_spacing / 2.0;

                    // Spawn card with face up
                    let card_entity = commands
                        .spawn((
                            Card {
                                emoji_index,
                                face_up: true,
                                sequence_position: None,
                                locked: false,
                            },
                            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(0.5)),
                            Visibility::Inherited,
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ))
                        .id();

                    // Spawn emoji sprite
                    if let Some(emoji_entity) = emoji::spawn_emoji(
                        &mut commands,
                        &atlas,
                        &validation,
                        emoji_index,
                        Vec2::ZERO,
                        0.5,
                    ) {
                        commands
                            .entity(emoji_entity)
                            .insert(CardFace)
                            .insert(Visibility::Visible)
                            .insert(Transform::from_translation(Vec3::ZERO));
                        commands.entity(card_entity).add_child(emoji_entity);
                    }
                }
            }
        }

        game_progress.sequence_step = SequenceStep::Ready;
    }
}

fn update_card_visibility(
    cards: Query<(&Card, &Children)>,
    mut face_sprites: Query<&mut Visibility, (With<CardFace>, Without<CardBack>)>,
    mut back_sprites: Query<&mut Visibility, (With<CardBack>, Without<CardFace>)>,
) {
    for (card, children) in &cards {
        for &child in children {
            // Update face sprite visibility
            if let Ok(mut visibility) = face_sprites.get_mut(child) {
                *visibility = if card.face_up {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            // Update back sprite visibility (only for sequence cards)
            if let Ok(mut visibility) = back_sprites.get_mut(child) {
                *visibility = if card.face_up {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
}
