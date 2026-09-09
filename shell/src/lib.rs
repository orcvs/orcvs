#![warn(clippy::all)]

pub mod console;
mod diagnostics;
mod grid_viewport;
mod midi;
mod persistence;
pub mod style;
#[cfg(target_arch = "wasm32")]
pub mod web_startup;
