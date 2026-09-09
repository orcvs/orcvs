#![warn(clippy::all)]

pub mod console;
pub mod diagnostics;
mod grid_viewport;
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
mod midi;
mod persistence;
mod report;
pub mod style;
#[cfg(target_arch = "wasm32")]
pub mod web_startup;
