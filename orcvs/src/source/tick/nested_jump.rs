//! Characterization of nested Jumps, whose Output Portal is not one row
//! south of their anchor.
//!
//! A nested Function writes through its own Output Portal as a root would
//! (ADR 0061). A nested `&>` or `&<` therefore reads and writes Cells of the
//! Expression it stands in. These tests state what a performer sees for each
//! layout, Tick after Tick: the Grid, the diagnostics and where they anchor,
//! and whether unrelated Expressions still publish.

mod horizontal;
mod root_parity;
mod vertical_and_map;

use super::observed::{Observed, observe_at};
use crate::grid::Grid;

///
/// Runs `ticks` consecutive Ticks against `rows`, each padded to the Grid's
/// width, through [`crate::source::Source::execute`].
///
fn observe(grid: Grid, rows: &[&str], ticks: u64) -> Vec<Observed> {
    observe_at(grid, rows, 0..ticks)
}
