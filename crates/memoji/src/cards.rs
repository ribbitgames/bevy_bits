use bevy::prelude::{ParamSet, *};
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use rand::prelude::*;

use crate::game::{FlipState, GameDifficulty, GameProgress, GameState, StageState};

pub const CARD_BACK: &str = "card_back.png";
const MISMATCH_COLOR: Color = Color::srgb(0.5, 0.0, 0.0);
const DEFAULT_COLOR: Color = Color::WHITE;

#[derive(Component, Debug, Default)]
pub struct Card {
    pub emoji_index: usize,
    pub face_up: bool,
    pub locked: bool,
}

#[derive(Component, Default)]
pub struct FeedbackTimer;

// Components for the two types of sprites
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
        app.init_resource::<FlipState>()
            .init_resource::<GameDifficulty>()
            .insert_resource(GameState::default())
            .add_systems(Startup, setup_cards)
            .add_systems(
                Update,
                (
                    spawn_emoji_grid,
                    handle_card_flipping,
                    update_card_visibility,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_cards(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the card back texture
    let card_back = asset_server.load(CARD_BACK);
    commands.insert_resource(CardBackTexture(card_back));
}

fn spawn_emoji_grid(
    mut commands: Commands,
    atlas: Res<EmojiAtlas>,
    validation: Res<AtlasValidation>,
    card_back: Res<CardBackTexture>,
    difficulty: Res<GameDifficulty>,
    query: Query<Entity, With<emoji::EmojiSprite>>,
) {
    if !emoji::is_emoji_system_ready(&validation) || !query.is_empty() {
        return;
    }

    let selected_indices = emoji::get_random_emojis(&atlas, &validation, difficulty.num_pairs);
    let mut all_indices = Vec::with_capacity(difficulty.num_pairs * 2);
    for &idx in &selected_indices {
        all_indices.extend([idx, idx]);
    }
    all_indices.shuffle(&mut rand::thread_rng());

    let grid_width = difficulty.grid_cols as f32 * difficulty.grid_spacing;
    let grid_height = difficulty.grid_rows as f32 * difficulty.grid_spacing;
    let start_x = -grid_width / 2.0;
    let start_y = grid_height / 2.0;

    for row in 0..difficulty.grid_rows {
        for col in 0..difficulty.grid_cols {
            let index = (row * difficulty.grid_cols + col) as usize;
            if let Some(&sprite_index) = all_indices.get(index) {
                let x = (col as f32).mul_add(difficulty.grid_spacing, start_x)
                    + difficulty.grid_spacing / 2.0;
                let y = (-(row as f32)).mul_add(difficulty.grid_spacing, start_y)
                    - difficulty.grid_spacing / 2.0;
                let _position = Vec2::new(x + 0.5, y + 0.5);

                // Spawn card parent entity
                let card_entity = commands
                    .spawn((
                        Card {
                            emoji_index: sprite_index,
                            ..Default::default()
                        },
                        Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(0.5)),
                        Visibility::Inherited,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                    ))
                    .id();

                // Spawn emoji sprite (face up side)
                if let Some(emoji_entity) = emoji::spawn_emoji(
                    &mut commands,
                    &atlas,
                    &validation,
                    sprite_index,
                    Vec2::ZERO, // Relative to parent
                    0.5,
                ) {
                    commands
                        .entity(emoji_entity)
                        .insert(CardFace)
                        .insert(Visibility::Hidden)
                        .insert(Transform::from_translation(Vec3::ZERO));
                    commands.entity(card_entity).add_child(emoji_entity);
                }

                // Spawn card back sprite
                let card_back_entity = commands
                    .spawn(CardBackBundle {
                        sprite: Sprite {
                            image: card_back.0.clone(),
                            custom_size: Some(Vec2::splat(difficulty.grid_spacing * 1.5)), // card_back size doesn't match the atlas emojis unfortunately
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
            // Update back sprite visibility
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

fn handle_card_flipping(
    mut card_queries: ParamSet<(
        Query<(Entity, &mut Card, &Children)>,
        Query<(Entity, &Card)>,
    )>,
    mut sprite_query: Query<&mut Sprite>,
    mut flip_state: ResMut<FlipState>,
    difficulty: Res<GameDifficulty>,
    mut stage_state: ResMut<StageState>,
    mut game_progress: ResMut<GameProgress>,
    time: Res<Time>,
) {
    if game_progress.is_interaction_blocked() {
        return;
    }

    // Handle unmatch timer if it exists
    if let Some(timer) = &mut flip_state.unmatch_timer {
        if timer.tick(time.delta()).just_finished() {
            let mut cards = card_queries.p0();
            for &entity in &flip_state.face_up_cards {
                let Ok((_, mut card, children)) = cards.get_mut(entity) else {
                    continue;
                };

                if card.locked {
                    continue;
                }

                card.face_up = false;
                for &child in children {
                    if let Ok(mut sprite) = sprite_query.get_mut(child) {
                        sprite.color = DEFAULT_COLOR;
                    }
                }
            }
            flip_state.face_up_cards.clear();
            flip_state.unmatch_timer = None;
        }
        return;
    }

    // Check if we have a pair to evaluate
    if flip_state.face_up_cards.len() != 2 {
        return;
    }

    let card_refs: Vec<Entity> = flip_state.face_up_cards.clone();
    let cards = card_queries.p1();

    let is_match = if let (Some(&first), Some(&second)) = (card_refs.first(), card_refs.get(1)) {
        check_for_match(&cards, first, second)
    } else {
        flip_state.face_up_cards.clear();
        return;
    };

    let mut cards = card_queries.p0();
    if is_match {
        for entity in card_refs {
            if let Ok((_, mut card, children)) = cards.get_mut(entity) {
                card.locked = true;
                // Matches stay default color
                for &child in children {
                    if let Ok(mut sprite) = sprite_query.get_mut(child) {
                        sprite.color = DEFAULT_COLOR;
                    }
                }
            }
        }
        flip_state.face_up_cards.clear();

        let cards = card_queries.p1();
        if check_all_matched(&cards) {
            stage_state.stage_complete = true;
            stage_state.transition_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
        }
    } else {
        if game_progress.record_mistake() {
            return;
        }
        // Apply mismatch color
        for entity in &card_refs {
            if let Ok((_, _, children)) = cards.get_mut(*entity) {
                for &child in children {
                    if let Ok(mut sprite) = sprite_query.get_mut(child) {
                        sprite.color = MISMATCH_COLOR;
                    }
                }
            }
        }
        flip_state.unmatch_timer = Some(Timer::from_seconds(
            difficulty.mismatch_delay,
            TimerMode::Once,
        ));
    }
}

fn check_for_match(cards: &Query<(Entity, &Card)>, card1: Entity, card2: Entity) -> bool {
    match (cards.get(card1), cards.get(card2)) {
        (Ok((_, card1)), Ok((_, card2))) => card1.emoji_index == card2.emoji_index,
        _ => false,
    }
}

fn check_all_matched(cards: &Query<(Entity, &Card)>) -> bool {
    cards.iter().all(|(_, card)| card.face_up && card.locked)
}
