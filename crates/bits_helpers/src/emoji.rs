use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::utils::default;
use thiserror::Error;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum EmojiSystemSet {
    Analyze,
    Validate,
}

pub struct EmojiPlugin;

impl Plugin for EmojiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AtlasValidation>()
            .configure_sets(
                Update,
                (
                    EmojiSystemSet::Analyze,
                    EmojiSystemSet::Validate.after(EmojiSystemSet::Analyze),
                ),
            )
            .add_systems(Startup, setup_emoji_atlas)
            .add_systems(Update, analyze_emoji_atlas.in_set(EmojiSystemSet::Analyze))
            .add_systems(
                Update,
                validate_emoji_atlas.in_set(EmojiSystemSet::Validate),
            );
    }
}

pub const ATLAS_SIZE: UVec2 = UVec2::new(4096, 4096);
pub const EMOJI_SIZE: UVec2 = UVec2::new(64, 64);

#[cfg(not(target_arch = "wasm32"))]
pub const ATLAS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "../../bits_helpers/assets/emojis/EmojiAtlas.ktx2"
);

#[cfg(target_arch = "wasm32")]
pub const ATLAS_PATH: &str = concat!(
    "../../bits_helpers-",
    env!("CARGO_PKG_VERSION"),
    "/assets/emojis/EmojiAtlas.ktx2"
);

#[derive(Error, Debug)]
pub enum AtlasError {
    #[error("Failed to load atlas texture: {0}")]
    TextureLoadError(String),

    #[error("Atlas dimensions mismatch - expected {expected:?}, got {actual:?}")]
    DimensionMismatch { expected: UVec2, actual: UVec2 },

    #[error("No valid emoji cells found in atlas")]
    NoValidCells,

    #[error("Invalid texture format: {0:?}")]
    InvalidFormat(TextureFormat),
}

#[derive(Resource)]
pub struct EmojiAtlas {
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    valid_indices: Vec<usize>,
}

#[derive(Component)]
pub struct EmojiSprite;

#[derive(Resource, Default)]
pub struct AtlasValidation {
    is_analyzed: bool,
    is_loaded: bool,
    total_emojis: usize,
}

fn setup_emoji_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load(ATLAS_PATH);
    let cols = ATLAS_SIZE.x / EMOJI_SIZE.x;
    let rows = ATLAS_SIZE.y / EMOJI_SIZE.y;
    let layout = TextureAtlasLayout::from_grid(EMOJI_SIZE, cols, rows, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    commands.insert_resource(EmojiAtlas {
        texture: texture_handle,
        layout: layout_handle,
        valid_indices: Vec::new(),
    });
}

fn analyze_emoji_atlas(
    mut atlas: ResMut<EmojiAtlas>,
    mut validation: ResMut<AtlasValidation>,
    images: Res<Assets<Image>>,
) {
    if validation.is_analyzed {
        return;
    }

    let Some(texture) = images.get(&atlas.texture) else {
        return;
    };

    let bytes_per_pixel = match texture.texture_descriptor.format {
        TextureFormat::Rgba8UnormSrgb
        | TextureFormat::Rgba8Unorm
        | TextureFormat::Bgra8UnormSrgb => 4,
        format => {
            error!("Invalid texture format: {:?}", format);
            return;
        }
    };

    let cols = ATLAS_SIZE.x / EMOJI_SIZE.x;
    let rows = ATLAS_SIZE.y / EMOJI_SIZE.y;
    let mut valid_indices = Vec::new();

    let is_cell_valid = |image: &Image, cell_x: u32, cell_y: u32| -> bool {
        for y in 0..EMOJI_SIZE.y {
            for x in 0..EMOJI_SIZE.x {
                let pixel_x = cell_x * EMOJI_SIZE.x + x;
                let pixel_y = cell_y * EMOJI_SIZE.y + y;

                if pixel_x >= image.width() || pixel_y >= image.height() {
                    continue;
                }

                let idx = ((pixel_y * image.width() + pixel_x) * bytes_per_pixel) as usize;
                let pixel: Option<&[u8]> = image.data.get(idx..idx + 4);

                if let Some(&[r, g, b, a]) =
                    pixel.and_then(|window| <&[u8] as TryInto<&[u8; 4]>>::try_into(window).ok())
                {
                    if a > 0 || r > 0 || g > 0 || b > 0 {
                        return true;
                    }
                }
            }
        }
        false
    };

    for row in 0..rows {
        for col in 0..cols {
            let index = (row * cols + col) as usize;
            if is_cell_valid(texture, col, row) {
                valid_indices.push(index);
            }
        }
    }

    if valid_indices.is_empty() {
        error!("No valid emoji cells found in atlas!");
        return;
    }

    atlas.valid_indices = valid_indices;
    validation.is_analyzed = true;
    info!(
        "Atlas analyzed: found {} valid emoji cells",
        atlas.valid_indices.len()
    );
}

fn validate_emoji_atlas(
    atlas: Res<EmojiAtlas>,
    mut validation: ResMut<AtlasValidation>,
    atlas_images: Res<Assets<Image>>,
) {
    if validation.is_loaded {
        return;
    }

    let Some(texture) = atlas_images.get(&atlas.texture) else {
        return;
    };

    if texture.width() != ATLAS_SIZE.x || texture.height() != ATLAS_SIZE.y {
        error!(
            "Emoji atlas dimensions mismatch! Expected {}x{}, got {}x{}",
            ATLAS_SIZE.x,
            ATLAS_SIZE.y,
            texture.width(),
            texture.height()
        );
        return;
    }

    if !validation.is_analyzed {
        return;
    }

    validation.is_loaded = true;
    validation.total_emojis = atlas.valid_indices.len();
    info!(
        "Emoji atlas validated! Total positions: {}",
        validation.total_emojis
    );
}

/// Creates a new emoji sprite entity with the specified transform
///
/// # Parameters
/// * `commands` - Mutable commands resource for entity spawning
/// * `atlas` - Reference to the emoji atlas resource
/// * `validation` - Reference to the atlas validation state
/// * `index` - The index of the emoji in the atlas
/// * `transform` - The transform defining the emoji's position, rotation, and scale
///
/// # Returns
/// * `Option<Entity>` - The spawned entity ID if successful, None if the emoji index is invalid
pub fn spawn_emoji(
    commands: &mut Commands,
    atlas: &Res<EmojiAtlas>,
    validation: &Res<AtlasValidation>,
    index: usize,
    transform: Transform,
) -> Option<Entity> {
    if !validation.is_loaded || !atlas.valid_indices.contains(&index) {
        return None;
    }

    Some(
        commands
            .spawn((
                Sprite {
                    image: atlas.texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas.layout.clone(),
                        index,
                    }),
                    ..default()
                },
                transform,
                Visibility::Visible,
                EmojiSprite,
            ))
            .id(),
    )
}

/// Gets a random selection of emoji indices suitable for creating pairs
pub fn get_random_emojis(
    atlas: &Res<EmojiAtlas>,
    validation: &Res<AtlasValidation>,
    count: usize,
) -> Vec<usize> {
    if !validation.is_loaded {
        return Vec::new();
    }

    let valid_indices = &atlas.valid_indices;
    let mut indices: Vec<usize> = valid_indices.clone();
    let mut result = Vec::with_capacity(count);

    // Take random indices until we have enough or run out
    while result.len() < count && !indices.is_empty() {
        let idx = fastrand::usize(..indices.len());
        result.push(indices.swap_remove(idx));
    }

    result
}

/// Returns whether the emoji system is ready for use
#[must_use]
pub fn is_emoji_system_ready(validation: &Res<AtlasValidation>) -> bool {
    validation.is_loaded
}

/// Gets the total number of valid emojis available
#[must_use]
pub fn get_emoji_count(validation: &Res<AtlasValidation>) -> usize {
    validation.total_emojis
}

/// Returns whether the index is valid
#[must_use]
pub fn is_valid_emoji_index(atlas: &Res<EmojiAtlas>, index: usize) -> bool {
    atlas.valid_indices.contains(&index)
}
