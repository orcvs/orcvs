//! A Source Tick as a performer observes it: the Grid after the Tick, and
//! each diagnostic with the Grid column and row of its first Cell.

use std::collections::{BTreeMap, BTreeSet};

use lang::{MidiChannel, Note, PlayCommand, Tick, Velocity};

use super::plan;
use crate::grid::Grid;
use crate::source::{Cells, Source, TickPlan};

/// The diagnostic for a Bang that reaches a root after its Turn.
pub(super) const MISSED_BANG: &str = "Bang reached a root that has taken its Turn";

/// The Play Command a Raw Play spelled `!>007F` answers for `note`.
pub(super) fn raw_play(note: u8) -> PlayCommand {
    PlayCommand::Raw {
        channel: MidiChannel::try_from(0x00).expect("a MIDI channel"),
        velocity: Velocity::try_from(0x7F).expect("a MIDI data byte"),
        note: Note::try_from(note).expect("a MIDI note"),
    }
}

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

/// The first Tick of `rows`, observed.
pub(super) fn first(grid: Grid, rows: &[&str]) -> Observed {
    observe_at(grid, rows, [0]).remove(0)
}

/// The Grid rows after `ticks` consecutive Ticks from Tick zero, one entry per
/// Tick, for a Source whose Ticks diagnose nothing.
pub(super) fn quiet_rows(grid: Grid, rows: &[&str], ticks: u64) -> Vec<Vec<String>> {
    observe_at(grid, rows, 0..ticks)
        .into_iter()
        .map(|tick| {
            quiet(&tick);
            tick.rows
        })
        .collect()
}

/// Asserts that `tick` diagnosed nothing.
pub(super) fn quiet(tick: &Observed) {
    assert!(tick.diagnostics.is_empty(), "{:?}", tick.diagnostics);
}

/// A diagnostic as [`Observed`] records it.
pub(super) fn diagnostic(x: usize, y: usize, message: &str) -> (usize, usize, String) {
    (x, y, message.to_string())
}

/// The Turn each computation took in Tick zero of `rows`, by anchor.
pub(super) fn turns(grid: Grid, rows: &[&str]) -> BTreeMap<(usize, usize), Option<usize>> {
    let source = source_of(grid, rows);
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (_, states) = plan(
        grid,
        Cells::of(bytes.as_bytes()),
        &map,
        &BTreeSet::new(),
        Tick::ZERO,
    );
    map.schedule_cache()
        .schedule(grid, &map)
        .lookup
        .nodes()
        .iter()
        .zip(&states)
        .map(|(node, state)| ((node.anchor.x(), node.anchor.y()), state.turn()))
        .collect()
}

/// Asserts that the computation at `earlier` took its Turn before the one at
/// `later`, both having taken one.
pub(super) fn turn_before(
    turns: &BTreeMap<(usize, usize), Option<usize>>,
    earlier: (usize, usize),
    later: (usize, usize),
) {
    let turn =
        |anchor| turns[&anchor].unwrap_or_else(|| panic!("{anchor:?} took no Turn: {turns:?}"));
    assert!(turn(earlier) < turn(later), "{turns:?}");
}
