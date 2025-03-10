use bevy::color::palettes::css;
use bevy::prelude::*;
use bits_helpers::{FONT, send_bit_message};
use ribbit_bits::BitMessage;

pub struct GamePlugin;

/// Time to wait before allowing block interaction (seconds)
const INITIAL_WAIT_TIME: f32 = 1.0;
/// Maximum number of blocks that can be removed before tower collapse
const MAX_BLOCKS_REMOVED: u32 = 15;
/// Time limit for each level in seconds
const LEVEL_TIME_LIMIT: f32 = 90.0;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameProgress>()
            .init_resource::<LevelSettings>()
            .add_systems(Update, handle_level_transition)
            .add_systems(
                Update,
                (update_game_timer, check_level_complete)
                    .run_if(in_state(GameState::Playing))
                    .chain(),
            );
    }
}

#[derive(Component)]
pub struct InteractionStateText;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Welcome,
    Playing,
    LevelComplete,
    GameOver,
}

#[derive(Resource)]
pub struct GameProgress {
    /// Current score
    pub score: u32,

    /// Timer for current level
    pub level_timer: Timer,

    /// Number of blocks safely removed
    pub blocks_removed: u32,

    /// Whether tower has collapsed
    pub tower_collapsed: bool,

    /// Whether level is completed
    pub level_complete: bool,

    /// Timer before allowing block interactions
    pub initial_wait_timer: Option<Timer>,
}

impl Default for GameProgress {
    fn default() -> Self {
        Self {
            score: 0,
            level_timer: Timer::from_seconds(LEVEL_TIME_LIMIT, TimerMode::Once),
            blocks_removed: 0,
            tower_collapsed: false,
            level_complete: false,
            initial_wait_timer: Some(Timer::from_seconds(INITIAL_WAIT_TIME, TimerMode::Once)),
        }
    }
}

impl GameProgress {
    /// Records a successful block removal and returns whether level is complete
    pub fn record_block_removal(&mut self) -> bool {
        self.blocks_removed += 1;
        self.score += 10; // Basic score per block

        // Level is complete when certain number of blocks are removed
        if self.blocks_removed >= MAX_BLOCKS_REMOVED {
            self.level_complete = true;
        }

        self.level_complete
    }

    /// Records tower collapse and game over
    pub fn record_tower_collapse(&mut self) {
        self.tower_collapsed = true;
    }

    /// Returns true if block interaction should be blocked
    pub const fn is_interaction_blocked(&self) -> bool {
        self.initial_wait_timer.is_some() || self.level_complete || self.tower_collapsed
    }

    /// Adds time bonus to score based on remaining time
    pub fn add_time_bonus(&mut self) {
        let remaining_time = self.level_timer.remaining_secs();
        let bonus = (remaining_time as u32) * 5; // 5 points per second remaining
        self.score += bonus;
    }
}

#[derive(Resource, Debug)]
pub struct LevelSettings {
    /// Current level number (starts at 1)
    pub level: u32,

    /// Number of blocks in tower
    pub num_blocks: u32,

    /// Tower height (number of rows)
    pub tower_height: u32,

    /// Tower width (blocks per row)
    pub tower_width: u32,

    /// Block size in pixels
    pub block_size: f32,

    /// Gravity strength
    pub gravity: f32,
}

impl Default for LevelSettings {
    fn default() -> Self {
        Self {
            level: 1,
            num_blocks: 12,
            tower_height: 4,
            tower_width: 3,
            block_size: 50.0,
            gravity: 3.0,
        }
    }
}

impl LevelSettings {
    /// Progress to next level and recalculate settings
    pub fn advance_level(&mut self) {
        self.level += 1;
        self.recalculate_settings();
    }

    /// Calculate level settings based on current level
    fn recalculate_settings(&mut self) {
        // Start small and gradually increase height
        self.tower_height = 4 + self.level;

        // Keep width at 3 for early levels, then start increasing
        if self.level > 3 {
            self.tower_width = 3 + (self.level - 3) / 2;
        }

        // Adjust block count based on dimensions
        self.num_blocks = self.tower_height * self.tower_width;

        // Adjust gravity more gradually
        self.gravity = (self.level as f32).mul_add(0.3, 3.0);
    }
}

/// System to update the level timer
fn update_game_timer(
    time: Res<Time>,
    mut game_progress: ResMut<GameProgress>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut text_query: Query<(Entity, &mut Text), With<InteractionStateText>>,
) {
    // Update initial wait timer
    if let Some(timer) = &mut game_progress.initial_wait_timer {
        if timer.tick(time.delta()).just_finished() {
            game_progress.initial_wait_timer = None;

            // Add visual indicator that interactions are now enabled
            if text_query.is_empty() {
                // Create a new UI root for the interaction state text
                let ui_root = commands
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        top: Val::Px(200.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .id();

                // Add the text as a child
                let text_entity = commands
                    .spawn((
                        Text::new("INTERACTIONS ENABLED"),
                        TextColor(css::GREEN.into()),
                        TextFont {
                            font: asset_server.load(FONT),
                            font_size: 24.0,
                            ..default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        InteractionStateText,
                    ))
                    .id();

                commands.entity(ui_root).add_child(text_entity);
            } else if let Ok((_, mut text)) = text_query.get_single_mut() {
                text.0 = "INTERACTIONS ENABLED".to_string();
            }
        } else {
            // Show countdown
            let remaining = timer.remaining_secs().ceil();
            let message = format!("WAITING: {:.0} seconds", remaining);

            if text_query.is_empty() {
                // Create a new UI root for the interaction state text
                let ui_root = commands
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        top: Val::Px(200.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .id();

                // Add the text as a child
                let text_entity = commands
                    .spawn((
                        Text::new(message),
                        TextColor(css::YELLOW.into()),
                        TextFont {
                            font: asset_server.load(FONT),
                            font_size: 24.0,
                            ..default()
                        },
                        TextLayout::new_with_justify(JustifyText::Center),
                        InteractionStateText,
                    ))
                    .id();

                commands.entity(ui_root).add_child(text_entity);
            } else if let Ok((_, mut text)) = text_query.get_single_mut() {
                text.0 = message;
            }
        }
        return;
    }

    // Update the interaction state text if it exists
    if let Ok((_, mut text)) = text_query.get_single_mut() {
        if game_progress.is_interaction_blocked() {
            text.0 = "INTERACTIONS BLOCKED".to_string();
        } else {
            text.0 = "INTERACTIONS ENABLED".to_string();
        }
    }

    // Update level timer if game is active
    if !game_progress.level_complete
        && !game_progress.tower_collapsed
        && game_progress.level_timer.tick(time.delta()).just_finished()
    {
        // Time's up - set game over
        game_progress.tower_collapsed = true;
    }
}

/// System to check if the level is complete or game is over
fn check_level_complete(
    game_progress: Res<GameProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if game_progress.level_complete {
        next_state.set(GameState::LevelComplete);
    } else if game_progress.tower_collapsed {
        send_bit_message(BitMessage::End(ribbit_bits::BitResult::HighestScore(
            game_progress.score.into(),
        )));
        next_state.set(GameState::GameOver);
    }
}

/// System to handle transition between levels
fn handle_level_transition(
    mut level_settings: ResMut<LevelSettings>,
    mut game_progress: ResMut<GameProgress>,
    game_state: Res<State<GameState>>,
) {
    if *game_state.get() == GameState::LevelComplete {
        // Advance to next level
        level_settings.advance_level();

        // Reset game progress for next level
        *game_progress = GameProgress {
            score: game_progress.score, // Keep the score
            level_timer: Timer::from_seconds(LEVEL_TIME_LIMIT, TimerMode::Once),
            blocks_removed: 0,
            tower_collapsed: false,
            level_complete: false,
            initial_wait_timer: Some(Timer::from_seconds(INITIAL_WAIT_TIME, TimerMode::Once)),
        };
    }
}
