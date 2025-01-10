#[cfg(target_arch = "wasm32")]
pub fn handle_browser_resize(
    mut primary_query: bevy::ecs::system::Query<
        &mut bevy::window::Window,
        bevy::ecs::query::With<bevy::window::PrimaryWindow>,
    >,
) {
    let Some(wasm_window) = web_sys::window() else {
        return;
    };
    let Ok(inner_width) = wasm_window.inner_width() else {
        return;
    };
    let Ok(inner_height) = wasm_window.inner_height() else {
        return;
    };
    let Some(target_width) = inner_width.as_f64() else {
        return;
    };
    let Some(target_height) = inner_height.as_f64() else {
        return;
    };
    let target_width = target_width as f32;
    let target_height = target_height as f32;

    const MAX_WIDTH: f32 = 2048.0;
    const MAX_HEIGHT: f32 = 2048.0;

    for mut window in &mut primary_query {
        if (window.resolution.width() - target_width).abs() > f32::EPSILON
            || (window.resolution.height() - target_height).abs() > f32::EPSILON
        {
            /* panicked at wgpu-0.20.1\src\backend\wgpu_core.rs:751:18:
            Error in Surface::configure: Validation Error

            Caused by:
                `Surface` width and height must be within the maximum supported texture size. Requested was (1284, 2418), maximum extent for either dimension is 2048.
             */

            let width = target_width.min(MAX_WIDTH);
            let height = target_height.min(MAX_HEIGHT);
            window.resolution.set(width, height);
        }
    }
}
