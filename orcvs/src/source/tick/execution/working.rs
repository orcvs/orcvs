//! The working Source of one Tick, with the generated placements it holds.
//!
//! Three facts change together as a Tick writes: the working Cells, the
//! geometry of each intact generated placement, and the Snapshot Cells a
//! Source effect has obscured. [`WorkingSource::apply`] is the only way any of
//! them changes, so every admitted write updates all three at once, and a
//! refused write, which never becomes a [`SpanWrite`], updates none.
//!
//! Contact is classified here because it is the one question that reads all
//! three: a placed unit is met by its geometry, and a Snapshot Language Unit
//! is met only through the Cells no Source effect has obscured. Placement
//! geometry is kept rather than recovered by reparsing working Source,
//! because a generated Function is Source content for the rest of this Tick:
//! it takes no Turn until a later Tick's Snapshot parses it.
//!
//! Which Cells a Portal reaches is the caller's to resolve. This module reads
//! the Cells at a resolved [`Portal`] and does not choose its offset. Nothing
//! here survives the Tick.

use std::ops::Range;

use super::{Cells, LanguageMap, Occupancy, Portal, PortalUnit, Position, SpanWrite};
use crate::source::buffer::WorkingCells;

/// What an admitted write does to placement geometry besides its Cells.
#[derive(Clone, Copy)]
pub(super) enum WriteKind {
    /// Ordinary Source content: a value, a Copy's write, or the cleanup of a
    /// prior Bang display. It obscures nothing, and an intact placement it
    /// reaches stops being one unit.
    Output,
    /// A Source effect clearing the Span a moving Function leaves. The
    /// Snapshot Cells it covers no longer hold the unit the Snapshot parsed.
    Vacate,
    /// A Source effect writing one whole generated unit: a moved or emitted
    /// Function, or the Bang a refused move leaves. It obscures the Snapshot
    /// Cells it covers and is an intact placement until a later write reaches
    /// it.
    Place,
}

pub(super) struct WorkingSource<'a> {
    cells: WorkingCells,
    /// The Language Units of the Source Snapshot, against which unobscured
    /// Cells are classified. A Comment and a standalone Bang occupy Cells and
    /// take no Turn, so contact reads every Language Unit rather than the
    /// computations alone.
    map: &'a LanguageMap,
    /// Generated units placed this Tick that no later write has reached.
    placed: Vec<Range<usize>>,
    /// Cells a Source effect has written. They stay obscured after a later
    /// ordinary write replaces what was placed there, because Snapshot
    /// ownership of those Cells ended with the effect.
    obscured: Vec<Range<usize>>,
}

impl<'a> WorkingSource<'a> {
    /// The working Source a Tick starts from: the Snapshot's Cells, with
    /// nothing placed and nothing obscured.
    pub(super) fn new(cells: Cells<'_>, map: &'a LanguageMap) -> Self {
        Self {
            cells: WorkingCells::new(cells),
            map,
            placed: Vec::new(),
            obscured: Vec::new(),
        }
    }

    ///
    /// Applies one admitted write to the Cells and to the placement geometry
    /// together.
    ///
    /// Any write overlapping an intact placement ends it, whatever its kind:
    /// the Cells there are no longer the unit that was placed. Later-write-wins
    /// settles a Cell two writes share.
    ///
    pub(super) fn apply(&mut self, kind: WriteKind, write: &SpanWrite) {
        let written = write.span().range();
        self.placed
            .retain(|placed| written.end <= placed.start || placed.end <= written.start);
        for (cell, content) in write.cells() {
            self.cells.write(cell, content);
        }
        match kind {
            WriteKind::Output => {}
            WriteKind::Vacate => self.obscured.push(written),
            WriteKind::Place => {
                self.obscured.push(written.clone());
                self.placed.push(written);
            }
        }
    }

    /// The working Cells in `range` as text. Panics when `range` runs past the
    /// last Cell.
    pub(super) fn text(&self, range: Range<usize>) -> &str {
        self.cells.text(range)
    }

    /// Whether every Cell in `cells` is empty in working Source.
    pub(super) fn vacant(&self, cells: &[usize]) -> bool {
        cells.iter().all(|&cell| self.cells.is_empty_at(cell))
    }

    /// Whether the pair one Atom occupies at `portal` holds a non-space in
    /// working Source.
    pub(super) fn occupied(&self, portal: Portal) -> bool {
        portal.occupied_in(self.cells.cells())
    }

    /// The `width` Cells at `portal`, or `None` where the row edge cuts them.
    pub(super) fn portal_cells(&self, portal: Portal, width: usize) -> Option<&str> {
        let span = portal.span(width).ok()?;
        Some(self.cells.text(span.range()))
    }

    ///
    /// The pair at `portal` when it is one complete aligned unit a Copy may
    /// copy, or `None` where it is not.
    ///
    /// Invalid and partial input stay absent so the Interpreter diagnoses
    /// rather than answering an Atom that was never a Language Unit.
    ///
    pub(super) fn portal_unit(&self, portal: Portal) -> Option<&str> {
        match portal.language_unit(self.cells.cells(), self.map) {
            PortalUnit::Invalid => None,
            PortalUnit::Empty | PortalUnit::Bang | PortalUnit::Unit => {
                let span = portal
                    .reservation()
                    .expect("an admitted unit fitted its row");
                Some(self.cells.text(span.range()))
            }
        }
    }

    ///
    /// What `cells` contact: an intact placement or a Snapshot Language Unit
    /// whose Cells no Source effect has obscured.
    ///
    /// A placed unit has no computation this Tick, so covering one completely
    /// is a non-root. `root_at` tells a completely covered Snapshot unit that
    /// is an Expression root from one that is not.
    ///
    pub(super) fn contact(
        &self,
        cells: &[usize],
        root_at: impl Fn(Position) -> Option<usize>,
    ) -> Occupancy {
        let mut partial = false;
        for placed in &self.placed {
            if cells.iter().any(|cell| placed.contains(cell)) {
                if cells.iter().all(|cell| placed.contains(cell)) {
                    return Occupancy::NonRoot;
                }
                partial = true;
            }
        }
        for unit in self.map.units() {
            let span = unit.span().range();
            let visible = |cell: &usize| {
                span.contains(cell) && !self.obscured.iter().any(|write| write.contains(cell))
            };
            if !cells.iter().any(visible) {
                continue;
            }
            if cells.iter().all(visible) {
                return root_at(unit.anchor()).map_or(Occupancy::NonRoot, Occupancy::Root);
            }
            partial = true;
        }
        if partial {
            Occupancy::Partial
        } else {
            Occupancy::Empty
        }
    }
}
