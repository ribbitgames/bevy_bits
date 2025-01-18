use bevy::prelude::*;
use bits_helpers::emoji::{self, AtlasValidation, EmojiAtlas};
use rand::prelude::*;

// Game constants
pub const DISPLAY_COLS: u32 = 4;
pub const DISPLAY_ROWS: u32 = 2;
pub const GRID_SPACING: f32 = 70.0;
pub const CARD_BACK: &str = "card_back.png";

#[derive(Component, Debug, Default)]
pub struct Card {
    pub emoji_index: usize,
    pub face_up: bool,
    pub locked: bool,
}

// Components for the two types of sprites
#[derive(Component)]
pub struct CardFace;

#[derive(Component)]
pub struct CardBack;

// Game state resource
#[derive(Resource, Default)]
pub struct FlipState {
    /// Currently face-up cards that aren't locked
    pub face_up_cards: Vec<Entity>,
    /// Timer for automatic flip-down of unmatched pairs
    pub unmatch_timer: Option<Timer>,
}

#[derive(Resource)]
pub struct CardBackTexture(Handle<Image>);

#[derive(Bundle)]
struct CardBackBundle {
    sprite: Sprite,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    card_back: CardBack,
}

// Game state to track reveal sequence
#[derive(Resource, Default)]
pub struct GameState {
    /// Timer for initial face-down state
    pub initial_wait_timer: Option<Timer>,
    /// Timer for how long cards stay revealed
    pub reveal_timer: Option<Timer>,
    /// Whether we're in the initial reveal phase
    pub cards_revealed: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            initial_wait_timer: Some(Timer::from_seconds(1.0, TimerMode::Once)),
            reveal_timer: Some(Timer::from_seconds(
                (DISPLAY_COLS * DISPLAY_ROWS) as f32 * 0.5,
                TimerMode::Once,
            )),
            cards_revealed: false,
        }
    }
}

pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlipState>()
            .insert_resource(GameState::new())
            .add_systems(Startup, setup_cards)
            .add_systems(
                Update,
                (
                    spawn_emoji_grid,
                    handle_reveal_sequence,
                    handle_card_flipping,
                    update_card_visibility,
                )
                    .chain(),
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
    query: Query<Entity, With<emoji::EmojiSprite>>,
) {
    if !emoji::is_emoji_system_ready(&validation) || !query.is_empty() {
        return;
    }

    let selected_indices = emoji::get_random_emojis(&atlas, &validation, 4);
    let mut all_indices = Vec::with_capacity(8);
    for &idx in &selected_indices {
        all_indices.extend([idx, idx]);
    }
    all_indices.shuffle(&mut rand::thread_rng());

    let grid_width = DISPLAY_COLS as f32 * GRID_SPACING;
    let grid_height = DISPLAY_ROWS as f32 * GRID_SPACING;
    let start_x = -grid_width / 2.0;
    let start_y = grid_height / 2.0;

    for row in 0..DISPLAY_ROWS {
        for col in 0..DISPLAY_COLS {
            let index = (row * DISPLAY_COLS + col) as usize;
            if let Some(&sprite_index) = all_indices.get(index) {
                let x = (col as f32).mul_add(GRID_SPACING, start_x) + GRID_SPACING / 2.0;
                let y = (-(row as f32)).mul_add(GRID_SPACING, start_y) - GRID_SPACING / 2.0;
                let position = Vec2::new(x + 0.5, y + 0.5);

                // Spawn card parent entity
                let card_entity = commands
                    .spawn((
                        Card {
                            emoji_index: sprite_index,
                            ..Default::default()
                        },
                        Transform::default(),
                        Visibility::default(),
                    ))
                    .id();

                // Spawn emoji sprite (face up side)
                if let Some(emoji_entity) = emoji::spawn_emoji(
                    &mut commands,
                    &atlas,
                    &validation,
                    sprite_index,
                    position,
                    0.5,
                ) {
                    commands
                        .entity(emoji_entity)
                        .insert(CardFace)
                        .insert(Visibility::Hidden);
                    commands.entity(card_entity).add_child(emoji_entity);
                }

                // Spawn card back sprite
                let card_back_entity = commands
                    .spawn(CardBackBundle {
                        sprite: Sprite {
                            image: card_back.0.clone(),
                            custom_size: Some(Vec2::splat(GRID_SPACING)),
                            ..default()
                        },
                        transform: Transform::from_xyz(position.x, position.y, 0.0)
                            .with_scale(Vec3::splat(0.5)),
                        global_transform: GlobalTransform::default(),
                        visibility: Visibility::Visible,
                        inherited_visibility: InheritedVisibility::default(),
                        card_back: CardBack,
                    })
                    .id();

                commands.entity(card_entity).add_child(card_back_entity);
            }
        }
    }
}

fn handle_reveal_sequence(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    mut cards: Query<&mut Card>,
) {
    // Handle initial wait timer
    if let Some(timer) = &mut game_state.initial_wait_timer {
        if timer.tick(time.delta()).just_finished() {
            // Initial wait is over, reveal all cards
            for mut card in &mut cards {
                card.face_up = true;
            }
            game_state.cards_revealed = true;
            game_state.initial_wait_timer = None;
        }
        return;
    }

    // Handle reveal timer
    if game_state.cards_revealed {
        if let Some(timer) = &mut game_state.reveal_timer {
            if timer.tick(time.delta()).just_finished() {
                // Reveal time is over, hide all cards
                for mut card in &mut cards {
                    card.face_up = false;
                }
                game_state.cards_revealed = false;
                game_state.reveal_timer = None;
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
    mut cards: Query<(Entity, &mut Card)>,
    mut flip_state: ResMut<FlipState>,
    time: Res<Time>,
) {
    // Handle unmatch timer
    if let Some(timer) = &mut flip_state.unmatch_timer {
        if timer.tick(time.delta()).just_finished() {
            for entity in flip_state.face_up_cards.drain(..) {
                if let Ok((_, mut card)) = cards.get_mut(entity) {
                    if !card.locked {
                        card.face_up = false;
                    }
                }
            }
            flip_state.unmatch_timer = None;
            return;
        }
    }

    // Early return if we don't have exactly 2 cards
    if flip_state.face_up_cards.len() != 2 {
        return;
    }

    // Get indices for the two cards
    let indices: Vec<_> = flip_state
        .face_up_cards
        .iter()
        .filter_map(|&entity| cards.get(entity).ok())
        .map(|(_, card)| card.emoji_index)
        .collect();

    // If we don't have exactly 2 indices, something's wrong
    if indices.len() != 2 {
        flip_state.face_up_cards.clear();
        return;
    }

    // Check if we have a match
    let is_match = indices
        .first()
        .zip(indices.get(1))
        .is_some_and(|(a, b)| a == b);

    if is_match {
        // Lock the matched cards
        for &entity in &flip_state.face_up_cards {
            if let Ok((_, mut card)) = cards.get_mut(entity) {
                card.locked = true;
            }
        }
        flip_state.face_up_cards.clear();
    } else {
        // Start timer to flip unmatched cards
        flip_state.unmatch_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
    }
}
