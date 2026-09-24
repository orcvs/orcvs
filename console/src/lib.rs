#![warn(clippy::all)]

// The settings `~/.orcvs/config.toml` holds, read once at native startup
// (`.scratch/menu-structure/issues/04`).
mod config;
pub mod console;
// Validates a resolved Theme's composited text contrast
// (`.scratch/theming/issues/08`), reusing `style`'s own composition
// functions so painting and validation cannot independently drift.
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
mod contrast;
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
// Decodes a TOML, JSON or YAML Theme document's bytes into the unresolved
// document `theme::resolve` takes (`.scratch/theming/issues/07`). Pure:
// `theme_registry`'s native discovery owns the I/O.
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
mod theme_document;
// The Themes a console can select — built-ins, loaded Theme documents and
// their load failures — with native discovery (`.scratch/theming/issues/07`).
mod theme_registry;
// The dark and light Theme selections and the Theme each presents
// (`.scratch/theming/issues/04`).
mod theme_selection;
#[cfg(target_arch = "wasm32")]
pub mod web_startup;

#[doc(inline)]
pub use paint::{BackgroundRun, FramePaint, Paint, VisiblePositions};
