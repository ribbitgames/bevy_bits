use bevy::prelude::*;

/// Plugin that manages all game-related constants and configuration values
pub struct GameVariablesPlugin;

impl Plugin for GameVariablesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameVariables::default());
    }
}

/// Resource containing all game-related constants and configuration values
#[derive(Resource)]
pub struct GameVariables {
    // Card-related constants
    /// Size of each card in pixels (width and height as cards are square)
    pub card_size: f32,
    /// Vertical position for the bottom sequence cards
    pub sequence_card_y: f32,
    /// Default color for card sprites
    pub default_color: Color,
    /// Color used to indicate incorrect selection
    pub wrong_color: Color,
    /// Color used to indicate correct selection
    pub correct_color: Color,
    /// Path to the card back image asset
    pub card_back_path: &'static str,

    // Timing constants
    /// Duration to show each emoji in the sequence (seconds)
    pub reveal_time_per_emoji: f32,
    /// Duration to show completed sequence before hiding (seconds)
    pub sequence_complete_delay: f32,
    /// Duration to show mismatch feedback (seconds)
    pub mismatch_delay: f32,

    // Game rules
    /// Maximum number of mistakes allowed before game over
    pub max_mistakes: u32,
    /// Base score awarded for completing a stage
    pub stage_completion_score: u32,
    /// Maximum bonus score for quick completion
    pub max_speed_bonus: u32,
    /// Time threshold for achieving maximum speed bonus (seconds)
    pub speed_bonus_threshold: f32,

    // Initial difficulty settings
    /// Starting sequence length
    pub initial_sequence_length: u32,
    /// Starting grid columns
    pub initial_grid_cols: u32,
    /// Starting grid rows
    pub initial_grid_rows: u32,
    /// Starting grid spacing
    pub initial_grid_spacing: f32,
    /// Starting total emojis
    pub initial_total_emojis: usize,

    // Game over settings
    /// Duration to show all cards after game over (seconds)
    pub game_over_reveal_duration: f32,
    /// Duration for stage transition (seconds)
    pub stage_transition_duration: f32,

    // Effect settings
    /// Number of celebration particles to spawn
    pub celebration_particle_count: u32,
    /// Duration of celebration effect (seconds)
    pub celebration_duration: f32,
    /// Base size of celebration particles
    pub celebration_particle_size: f32,
    /// Number of feedback particles per card
    pub feedback_particle_count: u32,
    /// Duration of feedback particles (seconds)
    pub feedback_particle_duration: f32,
    /// Size of feedback particles
    pub feedback_particle_size: f32,

    // Ring effect settings
    /// Duration of ring effect (seconds)
    pub ring_effect_duration: f32,
    /// Ring effect margin beyond grid
    pub ring_effect_margin: f32,
    /// Ring effect rotation speed (radians per second)
    pub ring_effect_rotation_speed: f32,

    // UI settings
    /// Font size for welcome screen
    pub welcome_font_size: f32,
    /// Font size for game over screen
    pub game_over_font_size: f32,
    /// Font size for stage transition screen
    pub stage_transition_font_size: f32,
    /// Font size for score display
    pub score_font_size: f32,

    // Interaction settings
    /// Click detection radius for cards
    pub card_click_radius: f32,
}

impl Default for GameVariables {
    fn default() -> Self {
        Self {
            // Card constants
            card_size: 140.0,
            sequence_card_y: -200.0,
            default_color: Color::WHITE,
            wrong_color: Color::srgb(1.0, 0.0, 0.0),
            correct_color: Color::srgb(0.0, 1.0, 0.0),
            card_back_path: "card_back.png",

            // Timing constants
            reveal_time_per_emoji: 1.0,
            sequence_complete_delay: 1.0,
            mismatch_delay: 0.5,

            // Game rules
            max_mistakes: 3,
            stage_completion_score: 100,
            max_speed_bonus: 50,
            speed_bonus_threshold: 5.0,

            // Initial difficulty settings
            initial_sequence_length: 3,
            initial_grid_cols: 3,
            initial_grid_rows: 2,
            initial_grid_spacing: 80.0,
            initial_total_emojis: 6,

            // Game over settings
            game_over_reveal_duration: 3.0,
            stage_transition_duration: 2.0,

            // Effect settings
            celebration_particle_count: 30,
            celebration_duration: 1.5,
            celebration_particle_size: 10.0,
            feedback_particle_count: 10,
            feedback_particle_duration: 0.75,
            feedback_particle_size: 5.0,

            // Ring effect settings
            ring_effect_duration: 2.0,
            ring_effect_margin: 30.0,
            ring_effect_rotation_speed: 1.0,

            // UI settings
            welcome_font_size: 32.0,
            game_over_font_size: 32.0,
            stage_transition_font_size: 32.0,
            score_font_size: 24.0,

            // Interaction settings
            card_click_radius: 35.0,
        }
    }
}
