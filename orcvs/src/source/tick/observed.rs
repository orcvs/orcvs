//! A Source Tick as a performer observes it: the Grid after the Tick, and
//! each diagnostic with the Grid column and row of its first Cell.

use std::collections::BTreeMap;

use lang::{Function, MidiChannel, Note, PlayCommand, Tick, Velocity};

use super::portals::{PortalEvent, PortalOrigin, TickPortals, WriteRefusal};
use super::{PortalError, plan, shared_schedule};
use crate::grid::Grid;
use crate::opts::{CursorBloomRadius, SectorSeamSpacing};
use crate::region::Region;
use crate::render_frame::{PortalRole, RenderFrame, RenderFrameConfig};
use crate::source::{Cells, Source, SourceCommander, Span, TickPlan};

/// The diagnostic for a Bang that reaches a root after its Turn.
pub(super) const MISSED_BANG: &str = "Bang reached a root that has taken its Turn";

/// One Raw Play Command, stated as the three Numbers a Source writes.
pub(in crate::source) fn raw(channel: u8, velocity: u8, note: u8) -> PlayCommand {
    PlayCommand::Raw {
        channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
        velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
        note: Note::try_from(note).expect("a MIDI note"),
    }
}

/// The Raw Play Command `!>007FC4` answers: C4 on channel 0 at full
/// velocity.
pub(in crate::source) fn raw_c4() -> PlayCommand {
    raw(0, 0x7F, 0x3C)
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
    let (_, _, states) = plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
    shared_schedule(grid, &map)
        .lookup
        .nodes()
        .iter()
        .zip(&states)
        .map(|(node, state)| ((node.anchor.x(), node.anchor.y()), state.turn()))
        .collect()
}

/// One Portal read Tick zero of `rows` recorded: its owner's anchor and
/// Function, its origin, and the Cells it read or why it read none.
pub(super) type ObservedRead = (
    (usize, usize),
    Function,
    PortalOrigin,
    Result<Vec<(usize, usize)>, PortalError>,
);

/// Every read the Tick Portals of Tick zero of `rows` recorded, in order.
pub(super) fn reads(grid: Grid, rows: &[&str]) -> Vec<ObservedRead> {
    let source = source_of(grid, rows);
    let map = source.shared_language_map();
    let bytes = source.snapshot();
    let (_, portals, _) = plan(grid, Cells::of(bytes.as_bytes()), &map, Tick::ZERO);
    portals
        .uses()
        .iter()
        .filter_map(|access| {
            let PortalEvent::Read { origin, site } = access.event() else {
                return None;
            };
            let anchor = access.anchor();
            Some((
                (anchor.x(), anchor.y()),
                access.function(),
                *origin,
                site.map(|span| cells_of(span.span())),
            ))
        })
        .collect()
}

/// The Cells of `span`, by column and row.
fn cells_of(span: Span) -> Vec<(usize, usize)> {
    span.positions()
        .map(|position| (position.x(), position.y()))
        .collect()
}

/// The record the last Tick `source` committed left, while `source` is still
/// at the revision that commit left.
pub(super) fn committed_portals(source: &Source) -> TickPortals {
    TickPortals::clone(
        &source
            .current_tick_portals()
            .expect("a Tick committed a record at this revision"),
    )
}

/// One Portal write a Tick recorded: its owner's anchor and Function, its
/// origin, and the Cells it was admitted to or why it was refused.
pub(super) type ObservedWrite = (
    (usize, usize),
    Function,
    PortalOrigin,
    Result<Vec<(usize, usize)>, WriteRefusal>,
);

/// Every write `record` holds, in order.
pub(super) fn writes(record: &TickPortals) -> Vec<ObservedWrite> {
    record
        .uses()
        .iter()
        .filter_map(|access| {
            let PortalEvent::Write { origin, outcome } = access.event() else {
                return None;
            };
            let anchor = access.anchor();
            Some((
                (anchor.x(), anchor.y()),
                access.function(),
                *origin,
                outcome.map(|span| cells_of(span.span())),
            ))
        })
        .collect()
}

/// Which access each entry of `record` is, with its owner's anchor and
/// Function, in the order execution made them.
pub(super) fn accesses(record: &TickPortals) -> Vec<((usize, usize), Function, &'static str)> {
    record
        .uses()
        .iter()
        .map(|access| {
            let anchor = access.anchor();
            let kind = match access.event() {
                PortalEvent::Read { .. } => "read",
                PortalEvent::Write { .. } => "write",
            };
            ((anchor.x(), anchor.y()), access.function(), kind)
        })
        .collect()
}

/// The Cells a Render Frame derived from `source`'s current revision paints
/// in `role`, by column and row.
pub(super) fn painted(source: Source, role: PortalRole) -> Vec<(usize, usize)> {
    let grid = source.grid();
    let frame = RenderFrame::derive(
        SourceCommander::with_source(source).read_revision(),
        Region::at(grid, grid.origin()),
        false,
        RenderFrameConfig {
            sector_seam_spacing: SectorSeamSpacing::new(2).expect("a seam spacing"),
            cursor_bloom_radius: CursorBloomRadius::new(1).expect("a bloom radius"),
        },
    );
    frame
        .cells()
        .iter()
        .filter(|cell| cell.portal_roles().contains(role))
        .map(|cell| (cell.position().x(), cell.position().y()))
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
