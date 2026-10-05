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

use lang::Tick;

use crate::grid::Grid;
use crate::source::{Source, TickPlan};

///
/// One Tick as a performer observes it: the Grid after the Tick, and each
/// diagnostic with the Grid column and row of its first Cell.
///
#[derive(Debug, PartialEq)]
struct Observed {
    rows: Vec<String>,
    diagnostics: Vec<(usize, usize, String)>,
}

///
/// Runs `ticks` consecutive Ticks against `rows`, each padded to the Grid's
/// width, through [`Source::execute`].
///
fn observe(grid: Grid, rows: &[&str], ticks: u64) -> Vec<Observed> {
    let mut source = Source::new(grid);
    let width = grid.columns();
    let text: String = rows.iter().map(|row| format!("{row:width$}")).collect();
    for (index, byte) in text.bytes().enumerate() {
        source
            .set(
                grid.cell_index(index).expect("inside the Grid"),
                &char::from(byte).to_string(),
            )
            .unwrap();
    }
    (0..ticks)
        .map(|tick| {
            let plan: TickPlan = source.execute(Tick::new(tick));
            Observed {
                rows: source
                    .snapshot()
                    .into_bytes()
                    .chunks(width)
                    .map(|row| String::from_utf8(row.to_vec()).expect("ASCII Source"))
                    .collect(),
                diagnostics: plan
                    .diagnostics
                    .iter()
                    .map(|diagnostic| {
                        let anchor = diagnostic.anchor();
                        (anchor.x(), anchor.y(), diagnostic.message.clone())
                    })
                    .collect(),
            }
        })
        .collect()
}
