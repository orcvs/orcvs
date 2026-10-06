//! A Source Tick as a performer observes it: the Grid after the Tick, and
//! each diagnostic with the Grid column and row of its first Cell.

use lang::Tick;

use crate::grid::Grid;
use crate::source::{Source, TickPlan};

/// One observed Tick.
#[derive(Debug, PartialEq)]
pub(super) struct Observed {
    pub(super) rows: Vec<String>,
    pub(super) diagnostics: Vec<(usize, usize, String)>,
}

/// A Source holding `rows`, each padded to the Grid's width, with every Cell
/// of them set one at a time.
pub(super) fn source_of(grid: Grid, rows: &[&str]) -> Source {
    let mut source = Source::new(grid);
    let width = grid.columns();
    let text: String = rows.iter().map(|row| format!("{row:width$}")).collect();
    assert!(text.len() <= grid.count(), "the rows fit the Grid");
    for (index, byte) in text.bytes().enumerate() {
        source
            .set(
                grid.cell_index(index).expect("inside the Grid"),
                &char::from(byte).to_string(),
            )
            .unwrap();
    }
    source
}

/// Every row of `source`'s Grid as it stands.
pub(super) fn rows_of(source: &Source) -> Vec<String> {
    source
        .snapshot()
        .into_bytes()
        .chunks(source.grid().columns())
        .map(|row| String::from_utf8(row.to_vec()).expect("ASCII Source"))
        .collect()
}

/// `source` after it applied `plan`.
pub(super) fn observed(source: &Source, plan: &TickPlan) -> Observed {
    Observed {
        rows: rows_of(source),
        diagnostics: plan
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let anchor = diagnostic.anchor();
                (anchor.x(), anchor.y(), diagnostic.message.clone())
            })
            .collect(),
    }
}

/// Runs the Ticks in `ticks` against `rows` through [`Source::execute`].
pub(super) fn observe_at(
    grid: Grid,
    rows: &[&str],
    ticks: impl IntoIterator<Item = u64>,
) -> Vec<Observed> {
    let mut source = source_of(grid, rows);
    ticks
        .into_iter()
        .map(|tick| {
            let plan = source.execute(Tick::new(tick));
            observed(&source, &plan)
        })
        .collect()
}
