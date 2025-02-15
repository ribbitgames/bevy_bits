use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use bits_helpers::WINDOW_WIDTH;
use rand::prelude::*;

use crate::game::{GameDifficulty, GameProgress, GameState, SequenceState, SequenceStep};
use crate::variables::GameVariables;

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
fn setup_cards(mut commands: Commands, asset_server: Res<AssetServer>, vars: Res<GameVariables>) {
    let card_back = asset_server.load(vars.card_back_path);
    commands.insert_resource(CardBackTexture(card_back));
}

/// Handles initial spawning of sequence cards.
///
/// Cards are arranged in rows at the bottom, with each row centered horizontally.
fn handle_sequence_spawn(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    card_back: Res<CardBackTexture>,
    mut sequence_state: ResMut<SequenceState>,
    mut game_progress: ResMut<GameProgress>,
    difficulty: Res<GameDifficulty>,
    vars: Res<GameVariables>,
    query: Query<(Entity, &Card)>,
) {
    if game_progress.sequence_step != SequenceStep::SpawningSequence {
        return;
    }

    if !query
        .iter()
        .any(|(_, card)| card.sequence_position.is_some())
    {
        let sequence_length = difficulty.sequence_length as usize;
        let target_sequence = emoji::get_random_emojis(&atlas, &validation, sequence_length);

        let card_width = vars.card_size;
        let card_height = vars.card_size;
        let sequence_spacing = 10.0;
        let vertical_spacing = 10.0;

        let max_columns = (WINDOW_WIDTH / (card_width + sequence_spacing)).floor() as usize;
        let row_limit = sequence_length.min(max_columns);
        let num_rows = sequence_length.div_ceil(row_limit);

        let mut current_index = 0;
        for row in 0..num_rows {
            let cards_in_row = if row == num_rows - 1 && sequence_length % row_limit != 0 {
                sequence_length % row_limit
            } else {
                row_limit
            };

            let row_spacing = if cards_in_row > 1 {
                sequence_spacing
            } else {
                0.0
            };

            let row_width = (cards_in_row as f32)
                .mul_add(card_width, ((cards_in_row as f32) - 1.0) * row_spacing);
            let start_x = -row_width / 2.0 + card_width / 2.0;
            let y = (row as f32).mul_add(-(card_height + vertical_spacing), vars.sequence_card_y);

            for col in 0..cards_in_row {
                let x = (col as f32).mul_add(card_width + row_spacing, start_x);
                let emoji_index = *target_sequence
                    .get(current_index)
                    .expect("Index out-of-bound: target_sequence has fewer elements than expected");
                current_index += 1;

                let card_entity = commands
                    .spawn((
                        Card {
                            emoji_index,
                            face_up: false,
                            sequence_position: Some(row * row_limit + col),
                            locked: false,
                        },
                        Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(0.5)),
                        Visibility::Inherited,
                    ))
                    .id();

                // Create emoji transform relative to card
                let emoji_transform =
                    Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(2.0));

                if let Some(emoji_entity) = emoji::spawn_emoji(
                    &mut commands,
                    &atlas,
                    &validation,
                    emoji_index,
                    emoji_transform,
                ) {
                    commands
                        .entity(emoji_entity)
                        .insert(CardFace)
                        .insert(Visibility::Hidden);
                    commands.entity(card_entity).add_child(emoji_entity);
                }

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

        sequence_state.target_sequence = target_sequence;
        game_progress.sequence_step = SequenceStep::RevealingSequence;
        game_progress.step_timer = Some(Timer::from_seconds(
            vars.reveal_time_per_emoji,
            TimerMode::Once,
        ));
    }
}

/// Handles revealing sequence cards one by one.
fn handle_sequence_reveal(
    time: Res<Time>,
    vars: Res<GameVariables>,
    difficulty: Res<GameDifficulty>,
    mut game_progress: ResMut<GameProgress>,
    mut cards: Query<&mut Card>,
) {
    if game_progress.sequence_step != SequenceStep::RevealingSequence {
        return;
    }

    if let Some(timer) = &mut game_progress.step_timer {
        if timer.tick(time.delta()).just_finished() {
            if game_progress.current_reveal_index < difficulty.sequence_length as usize {
                for mut card in &mut cards {
                    if card.sequence_position == Some(game_progress.current_reveal_index) {
                        card.face_up = true;
                    }
                }

                game_progress.current_reveal_index += 1;
                game_progress.step_timer = Some(Timer::from_seconds(
                    vars.reveal_time_per_emoji,
                    TimerMode::Once,
                ));
            } else {
                game_progress.sequence_step = SequenceStep::HidingSequence;
                game_progress.step_timer = Some(Timer::from_seconds(
                    vars.sequence_complete_delay,
                    TimerMode::Once,
                ));
            }
        }
    }
}

/// Handles hiding all sequence cards after reveal.
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

/// Spawns the grid of face-up cards in the center of the screen.
fn handle_grid_spawn(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    difficulty: Res<GameDifficulty>,
    sequence_state: Res<SequenceState>,
    vars: Res<GameVariables>,
    mut game_progress: ResMut<GameProgress>,
    query: Query<(Entity, &Card)>,
) {
    if game_progress.sequence_step != SequenceStep::SpawningGrid {
        return;
    }

    if !query
        .iter()
        .any(|(_, card)| card.sequence_position.is_none())
    {
        let mut grid_indices = sequence_state.target_sequence.clone();
        let extra_emojis = emoji::get_random_emojis(
            &atlas,
            &validation,
            difficulty.total_emojis - difficulty.sequence_length as usize,
        );
        grid_indices.extend(extra_emojis);
        grid_indices.shuffle(&mut rand::rng());

        let grid_width = difficulty.grid_cols as f32 * difficulty.grid_spacing;
        let grid_height = difficulty.grid_rows as f32 * difficulty.grid_spacing;
        let start_x = -grid_width / 2.0;
        let start_y = grid_height / 2.0;

        for row in 0..difficulty.grid_rows {
            for col in 0..difficulty.grid_cols {
                let index = (row * difficulty.grid_cols + col) as usize;
                if let Some(&emoji_index) = grid_indices.get(index) {
                    let x = (col as f32).mul_add(difficulty.grid_spacing, start_x)
                        + vars.card_size / 2.0;
                    let y = (-(row as f32)).mul_add(difficulty.grid_spacing, start_y)
                        - vars.card_size / 2.0;

                    let card_entity = commands
                        .spawn((
                            Card {
                                emoji_index,
                                face_up: true,
                                sequence_position: None,
                                locked: false,
                            },
                            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(2.0)),
                            Visibility::Inherited,
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                        ))
                        .id();

                    // Create transform for emoji relative to card
                    let emoji_transform =
                        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(2.0));

                    if let Some(emoji_entity) = emoji::spawn_emoji(
                        &mut commands,
                        &atlas,
                        &validation,
                        emoji_index,
                        emoji_transform,
                    ) {
                        commands
                            .entity(emoji_entity)
                            .insert(CardFace)
                            .insert(Visibility::Visible);
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
            if let Ok(mut visibility) = face_sprites.get_mut(child) {
                *visibility = if card.face_up {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
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
