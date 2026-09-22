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
#[doc(hidden)]
pub mod source_paint;
pub mod style;
// Slice A of `.scratch/theming/issues/06`: the resolved Theme model and pure
// inheritance resolver. Nothing shipped calls into it yet — slice B wires
// Source painting to read a resolved Theme, and slice C adds the fixed
// display-point stroke widths. `not(test)` scopes the suppression to the
// plain library build; the crate's own unit tests inside `theme::tests`
// exercise every item, so the lint genuinely does not fire there.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by theming/06 slice B (Source painting) and slice C (stroke widths); \
                   only this module's own tests call it for now"
    )
)]
mod theme;
#[cfg(target_arch = "wasm32")]
pub mod web_startup;

#[doc(inline)]
pub use paint::{BackgroundRun, FramePaint, Paint, VisiblePositions};
