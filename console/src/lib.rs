#![warn(clippy::all)]

pub mod console;
#[doc(hidden)]
pub mod cursor_effects;
pub mod diagnostics;
mod function_reference;
mod grid_viewport;
mod marks;
mod midi;
pub mod native_midi;
mod paint;
pub mod persistence;
mod readout_deadline;
mod report;
pub mod style;
// The resolved Theme model and pure inheritance resolver
// (`.scratch/theming/issues/06`). Slice B wires Source painting and Cursor
// effects to read it; slice C adds the fixed display-point stroke widths.
// `pub` and `doc(hidden)` for the same reason `cursor_effects` is:
// `console/benches/paint.rs` is a separate crate and needs `Theme`/
// `okabe_ito` to build a Paint's Theme argument.
#[doc(hidden)]
pub mod theme;
#[cfg(target_arch = "wasm32")]
pub mod web_startup;

#[doc(inline)]
pub use paint::{BackgroundRun, FramePaint, Paint, VisiblePositions};
