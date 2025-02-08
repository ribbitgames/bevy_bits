use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use bits_helpers::WINDOW_WIDTH;
use rand::prelude::*;

use crate::game::{
    GameDifficulty, GameProgress, GameState, SequenceState, SequenceStep, REVEAL_TIME_PER_EMOJI,
};

pub const CARD_BACK: &str = "card_back.png";
pub const DEFAULT_COLOR: Color = Color::WHITE;
pub const WRONG_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
pub const CORRECT_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
const SEQUENCE_CARD_Y: f32 = -200.0; // Position for bottom sequence cards
pub const CARD_SIZE: f32 = 70.0; // Assuming all cards are square

#[derive(Component, Debug)]
pub struct Card {
    /// Index of the emoji from the emoji atlas.
    pub emoji_index: usize,
    /// Whether the card is currently face up.
    pub face_up: bool,
    /// The position in the sequence, or None if this is a grid card.
    pub sequence_position: Option<usize>,
    /// Indicates if the card has been correctly selected and locked.
    pub locked: bool,
}

#[derive(Component)]
pub struct CardFace;

#[derive(Component)]
pub struct CardBack;

#[derive(Resource)]
pub struct CardBackTexture(Handle<Image>);

#[derive(Bundle)]
struct CardBackBundle {
    /// The sprite representing the card back image.
    sprite: Sprite,
    /// Local transform of the card back.
    transform: Transform,
    /// Global transform of the card back.
    global_transform: GlobalTransform,
    /// Visibility component.
    visibility: Visibility,
    /// Inherited visibility from parent.
    inherited_visibility: InheritedVisibility,
    /// Bevy's view visibility component.
    view_visibility: ViewVisibility,
    /// Marker component to identify card back entities.
    card_back: CardBack,
}

/// Plugin to manage cards (both sequence and grid) for the game.
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

/// Cleans up all card entities when exiting the playing state.
fn cleanup_stage(mut commands: Commands, cards: Query<Entity, With<Card>>) {
    for card_entity in cards.iter() {
        commands.entity(card_entity).despawn_recursive();
    }
}

/// Loads the card back texture and inserts it as a resource.
fn setup_cards(mut commands: Commands, asset_server: Res<AssetServer>) {
    let card_back = asset_server.load(CARD_BACK);
    commands.insert_resource(CardBackTexture(card_back));
}

/// Handles initial spawning of sequence cards.
/// Cards are arranged in rows at the bottom, with each row centered horizontally.
/// In this version, we use a fixed minimal spacing between cards.
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

    // Only spawn sequence cards if none currently exist.
    if !query
        .iter()
        .any(|(_, card)| card.sequence_position.is_some())
    {
        let sequence_length = difficulty.sequence_length as usize;
        let available_indices = emoji::get_random_emojis(&atlas, &validation, sequence_length);
        sequence_state.target_sequence = available_indices.clone();

        // Card dimensions.
        let card_width = CARD_SIZE;
        let card_height = CARD_SIZE;
        // Use a fixed horizontal spacing that is just enough to keep cards separated.
        let sequence_spacing = 10.0;
        let vertical_spacing = 10.0;

        // Compute the maximum number of columns that can fit without overflowing the window.
        let max_columns = (WINDOW_WIDTH / (card_width + sequence_spacing)).floor() as usize;
        // Use as many columns as possible up to the total number of sequence cards.
        let row_limit = if sequence_length < max_columns {
            sequence_length
        } else {
            max_columns
        };
        // Compute the number of rows required.
        let num_rows = (sequence_length + row_limit - 1) / row_limit;

        let mut current_index = 0;
        for row in 0..num_rows {
            // Determine how many cards are in the current row.
            let cards_in_row = if row == num_rows - 1 && sequence_length % row_limit != 0 {
                sequence_length % row_limit
            } else {
                row_limit
            };

            // Use the fixed spacing between cards.
            let row_spacing = if cards_in_row > 1 {
                sequence_spacing
            } else {
                0.0 // No spacing needed for a single card.
            };

            // Compute the total width of this row and then the starting x so the row is centered.
            let row_width =
                (cards_in_row as f32) * card_width + ((cards_in_row as f32) - 1.0) * row_spacing;
            let start_x = -row_width / 2.0 + card_width / 2.0;
            // Compute the y position for this row.
            let y = SEQUENCE_CARD_Y - row as f32 * (card_height + vertical_spacing);

            for col in 0..cards_in_row {
                let x = start_x + col as f32 * (card_width + row_spacing);
                let emoji_index = available_indices[current_index];
                current_index += 1;

                // Spawn the parent card entity.
                let card_entity = commands
                    .spawn((
                        Card {
                            emoji_index,
                            face_up: false,
                            // We use a sequential number as the sequence position.
                            sequence_position: Some(row * row_limit + col),
                            locked: false,
                        },
                        Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(0.5)),
                        Visibility::Inherited,
                    ))
                    .id();

                // Spawn the emoji sprite as a child.
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

                // Spawn the card back as a child of the card.
                let card_back_entity = commands
                    .spawn(CardBackBundle {
                        sprite: Sprite {
                            image: card_back.0.clone(),
                            custom_size: Some(Vec2::new(card_width, card_height)),
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
        }

        game_progress.sequence_step = SequenceStep::RevealingSequence;
        game_progress.step_timer =
            Some(Timer::from_seconds(REVEAL_TIME_PER_EMOJI, TimerMode::Once));
    }
}

/// Handles revealing sequence cards one by one.
/// Each card is flipped face up with a timer until all are revealed.
fn handle_sequence_reveal(
    time: Res<Time>,
    difficulty: Res<GameDifficulty>,
    mut game_progress: ResMut<GameProgress>,
    mut cards: Query<&mut Card>,
) {
    if game_progress.sequence_step != SequenceStep::RevealingSequence {
        return;
    }

    if let Some(timer) = &mut game_progress.step_timer {
        if timer.tick(time.delta()).just_finished() {
            // Reveal the next card in the sequence.
            if game_progress.current_reveal_index < difficulty.sequence_length as usize {
                for mut card in &mut cards {
                    if card.sequence_position == Some(game_progress.current_reveal_index) {
                        card.face_up = true;
                    }
                }

                game_progress.current_reveal_index += 1;
                game_progress.step_timer =
                    Some(Timer::from_seconds(REVEAL_TIME_PER_EMOJI, TimerMode::Once));
            } else {
                // All cards have been revealed; wait before hiding them.
                game_progress.sequence_step = SequenceStep::HidingSequence;
                game_progress.step_timer = Some(Timer::from_seconds(
                    crate::game::SEQUENCE_COMPLETE_DELAY,
                    TimerMode::Once,
                ));
            }
        }
    }
}

/// Handles hiding all sequence cards after reveal.
/// Once the timer finishes, all sequence cards are flipped face down.
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
            // Hide all sequence cards.
            for mut card in &mut cards {
                if card.sequence_position.is_some() {
                    card.face_up = false;
                }
            }

            // Move to grid spawning phase.
            game_progress.sequence_step = SequenceStep::SpawningGrid;
            game_progress.step_timer = None;
        }
    }
}

/// Spawns the grid of face-up cards in the center of the screen.
/// The grid includes both sequence emojis and additional random ones.
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

    // Only spawn grid cards when no grid cards exist.
    if !query
        .iter()
        .any(|(_, card)| card.sequence_position.is_none())
    {
        // Get emoji indices for grid (including sequence emojis plus additional random ones).
        let mut grid_indices = sequence_state.target_sequence.clone();
        let extra_emojis = emoji::get_random_emojis(
            &atlas,
            &validation,
            difficulty.total_emojis - difficulty.sequence_length as usize,
        );
        grid_indices.extend(extra_emojis);
        grid_indices.shuffle(&mut rand::rng());

        // Calculate grid layout.
        let grid_width = difficulty.grid_cols as f32 * difficulty.grid_spacing;
        let grid_height = difficulty.grid_rows as f32 * difficulty.grid_spacing;
        let start_x = -grid_width / 2.0;
        let start_y = grid_height / 2.0;

        // Spawn grid cards.
        for row in 0..difficulty.grid_rows {
            for col in 0..difficulty.grid_cols {
                let index = (row * difficulty.grid_cols + col) as usize;
                if let Some(&emoji_index) = grid_indices.get(index) {
                    let x =
                        (col as f32).mul_add(difficulty.grid_spacing, start_x) + CARD_SIZE / 2.0;
                    let y =
                        (-(row as f32)).mul_add(difficulty.grid_spacing, start_y) - CARD_SIZE / 2.0;

                    // Spawn card with face up.
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

                    // Spawn emoji sprite as a child.
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

/// Updates the visibility of card faces and backs based on the card's `face_up` state.
fn update_card_visibility(
    cards: Query<(&Card, &Children)>,
    mut face_sprites: Query<&mut Visibility, (With<CardFace>, Without<CardBack>)>,
    mut back_sprites: Query<&mut Visibility, (With<CardBack>, Without<CardFace>)>,
) {
    for (card, children) in &cards {
        for &child in children {
            // Update face sprite visibility.
            if let Ok(mut visibility) = face_sprites.get_mut(child) {
                *visibility = if card.face_up {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            // Update back sprite visibility (only for sequence cards).
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
