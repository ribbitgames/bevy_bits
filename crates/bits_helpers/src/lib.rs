mod bit;
pub use bit::*;

pub mod emoji;
pub mod floating_score;
pub mod input;
pub mod restart;
pub mod welcome_screen;

mod ribbit_communication;
pub use ribbit_communication::*;

#[cfg(target_arch = "wasm32")]
mod window_resizing;

#[cfg(not(target_arch = "wasm32"))]
mod ribbit_simulation;
