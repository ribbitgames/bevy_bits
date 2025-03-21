#![allow(
    clippy::allow_attributes,
    reason = "allow attributes are needed for wasm"
)]

use std::time::Duration;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{WgpuSettings, WgpuSettingsPriority};
use bevy::window::{WindowMode, WindowResolution};
use bevy_framepace::{FramepaceSettings, Limiter};

#[cfg(not(target_arch = "wasm32"))]
use crate::ribbit_simulation::RibbitSimulation;
#[cfg(target_arch = "wasm32")]
use crate::window_resizing::handle_browser_resize;
use crate::{RibbitCommunicationPlugin, RibbitMessageHandler};

#[cfg(not(target_arch = "wasm32"))]
pub const FONT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraSans-Bold.ttf"
);
#[cfg(target_arch = "wasm32")]
pub const FONT: &str = concat!(
    "../../bits_helpers-",
    env!("CARGO_PKG_VERSION"),
    "/assets/fonts/FiraSans-Bold.ttf"
);

// typical smartphone screen ratio (9:16)
pub const WINDOW_WIDTH: f32 = 360.0;
pub const WINDOW_HEIGHT: f32 = 640.0;
pub const UI_MARGIN: f32 = 60.0;

// Creates a Bevy app with default settings to make Ribbit work
// This prevent duplication / errors accross different bits
#[allow(unused_variables, reason = "bit_version is used in wasm")]
#[allow(clippy::extra_unused_type_parameters)]
pub fn get_default_app<T: RibbitMessageHandler>(bit_name: &str, bit_version: &str) -> App {
    let mut app = App::new();

    let asset_plugin = bevy::asset::AssetPlugin {
        mode: bevy::asset::AssetMode::Unprocessed,

        #[cfg(not(target_arch = "wasm32"))]
        file_path: "assets".to_string(),
        #[cfg(target_arch = "wasm32")]
        file_path: format!("bits/{bit_name}-{bit_version}/assets"),
        processed_file_path: "imported_assets/Default".to_string(),
        watch_for_changes_override: None,
        meta_check: AssetMetaCheck::Never,
    };

    let resolution = WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT);

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: bit_name.to_string(),
            present_mode: bevy::window::PresentMode::Fifo,
            resolution,
            canvas: Some("#bit".into()),
            fit_canvas_to_parent: true,
            mode: WindowMode::Windowed,
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    };

    let render_plugin = RenderPlugin {
        render_creation: bevy::render::settings::RenderCreation::Automatic(WgpuSettings {
            backends: Some(
                bevy::render::settings::Backends::PRIMARY
                    | bevy::render::settings::Backends::SECONDARY,
            ),
            power_preference: bevy::render::settings::PowerPreference::HighPerformance,
            priority: WgpuSettingsPriority::Functionality,
            ..Default::default()
        }),
        ..Default::default()
    };

    app.add_plugins(
        DefaultPlugins
            .set(asset_plugin)
            .set(window_plugin)
            .set(render_plugin),
    );

    // This plugin is useful to preserve battery life on mobile.
    // https://github.com/aevyrie/bevy_framepace
    app.add_plugins(bevy_framepace::FramepacePlugin);
    app.add_systems(Startup, framepace_plugin_setup);

    // Add this new code to set the clear color to black
    app.insert_resource(ClearColor(Color::BLACK));

    app.add_plugins(RibbitCommunicationPlugin::<T>::default());

    #[cfg(target_arch = "wasm32")]
    {
        app.add_systems(PreUpdate, handle_browser_resize);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(RibbitSimulation);
    }

    app
}

fn framepace_plugin_setup(mut framepace: ResMut<FramepaceSettings>) {
    framepace.limiter = Limiter::Manual(Duration::from_secs_f64(1.0 / 60.0));
}
