#![warn(clippy::all)]

// The settings `~/.orcvs/config.toml` holds, read once at native startup.
mod config;
pub mod console;
pub mod console_midi;
// Validates a resolved Theme's composited text contrast, reusing `style`'s
// own composition functions so painting and validation cannot independently
// drift.
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
mod paint;
pub mod persistence;
mod readout_deadline;
mod report;
// The Source File the native console has open and whether the Source has
// changed since. The web opens no file.
#[cfg(not(target_arch = "wasm32"))]
mod source_file;
pub mod style;
// The resolved Theme model and pure inheritance resolver.
// `pub` and `doc(hidden)` for the same reason `cursor_effects` is:
// `console/benches/paint.rs` is a separate crate and needs `Theme`/
// `okabe_ito` to build a Paint's Theme argument.
#[doc(hidden)]
pub mod theme;
// Decodes a TOML Theme document's bytes into the unresolved
// document `theme::resolve` takes. Pure:
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
// their load failures — with native discovery.
mod theme_registry;
// The dark and light Theme selections and the Theme each presents.
mod theme_selection;
// The browser's MIDI backend over the Web MIDI API. Its logic is compiled
// for tests on every target, driven by a fake of the Web MIDI access it reads.
#[cfg(any(target_arch = "wasm32", test))]
mod web_midi;
#[cfg(target_arch = "wasm32")]
pub mod web_startup;

#[doc(inline)]
pub use paint::{BackgroundRun, FramePaint, Paint, VisiblePositions};
