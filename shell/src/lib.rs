#![warn(clippy::all)]

pub mod console;
pub mod diagnostics;
mod grid_viewport;
mod midi;
pub mod persistence;
mod report;
pub mod style;
#[cfg(target_arch = "wasm32")]
pub mod web_startup;
