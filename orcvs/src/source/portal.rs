//! ADR 0009's Portal: where one interpreted result becomes Cells.
//!
//! A Portal is one Cell destination resolved while interpreting a Source
//! Snapshot. It lives here rather than beside the producers in `tick` because
//! destination resolution is the question ADR 0009 expects to change: a
//! future Cell-addressing model, an infinite canvas among them, moves a
//! result somewhere else without touching Function evaluation, effect
//! ordering, or Tick Plan commit. Keeping resolution in its own module is what
//! makes that a change to one file rather than a change threaded through the
//! producer that happened to hardcode "the row below the root".
//!
//! Nothing in this module is reachable from the language crate, and nothing in
//! it is serialized. That is the whole of CONTEXT.md's "a Portal is neither a
//! language value, Source content, nor persistent state": a Portal cannot be
//! spelled in Source because no Atom carries one, cannot survive a Tick
//! because it is a local of the pass that resolved it, and cannot reach a save
//! file because `PersistedSource` carries a Grid and character Cells and this
//! type is not among them.

use crate::grid::{CellIndex, Grid, Position};

use super::CellContent;
use super::language_map::Span;

///
/// One Cell destination resolved while interpreting a Source Snapshot.
///
/// A Portal names where a result begins, not how wide it is: an ordinary Atom
/// and an intact Sequence pass through the same one, and the encoding decides
/// how many Cells follow along the row. That is why resolution and fit are
/// separate steps here — a destination exists or it does not, independently of
/// what any particular result would spell there — and why the Sequence case
/// needs no destination rule of its own.
///
/// ADR 0009 also lets a Source Function resolve several Portals as one effect
/// bundle, validated complete before any of its writes is admitted. Nothing
/// builds a bundle yet, because ADR 0005 defers the Source-addressing
/// Functions that would resolve one and a bundle API with no caller would be
/// shaped by guesswork. The shape it needs is already here: every Portal
/// answers with a whole [`SpanWrite`] or a refusal, so a bundle is those
/// answers collected with `?` and is admitted whole or not at all, with no
/// validation pass of its own to write.
#[derive(Clone, Copy, Debug)]
pub(super) struct Portal {
    grid: Grid,
    destination: Position,
}

///
/// Why a Portal refused a whole destination.
///
/// Failing to resolve a destination and failing to fit an encoding into one
/// are different questions, and they answer into one type because every
/// producer treats them identically: ADR 0004 admits no partial write, so
/// either one costs the whole write and yields a diagnostic instead. A
/// producer distinguishes them only to say which it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PortalError {
    /// There is no row below the producer's root, so no ordinary result
    /// destination exists at all.
    BelowSource,
    /// The encoding is wider than the destination row's remaining Cells.
    CrossesRowEdge,
}

impl Portal {
    ///
    /// The Portal an ordinary result passes through: the Cell below `root`, in
    /// `root`'s own column.
    ///
    /// A root in the last row resolves no Portal rather than a clamped one.
    /// Clamping would put a result in a Cell the Source never asked for, and a
    /// destination that does not exist is exactly what ADR 0009 wants reported
    /// — the caller diagnoses at the root and plans nothing.
    ///
    pub(super) fn ordinary_result(grid: Grid, root: Position) -> Result<Self, PortalError> {
        grid.below(root)
            .map(|destination| Self::at(grid, destination))
            .ok_or(PortalError::BelowSource)
    }

    ///
    /// The Portal onto `destination`.
    ///
    /// Infallible because a Position can only be obtained from the Grid that
    /// contains it: a destination outside the Grid cannot be presented here.
    /// A Position minted by another Grid can be, and is refused, because that
    /// Grid's coordinates name different Cells of this one.
    ///
    pub(super) fn at(grid: Grid, destination: Position) -> Self {
        grid.assert_owns(destination);
        Self { grid, destination }
    }

    ///
    /// The write that places `encoding` at this Portal and along its row, or
    /// the reason the whole destination was refused.
    ///
    /// The whole destination is decided before a [`SpanWrite`] exists, so
    /// ADR 0004's "validate the complete effect before admitting any of it" is
    /// a property of this constructor rather than a discipline every caller
    /// keeps. A refused encoding leaves nothing behind to emit half of: there
    /// is no value describing part of a write.
    ///
    /// `encoding` contains one or more printable ASCII Cells. Invalid content
    /// is an internal defect, so it asserts even in release builds. The write
    /// retains validated CellContent values through resolution and commit.
    ///
    /// An empty Sequence encodes to the empty string and plans no writes at
    /// all, so it never reaches a Portal; that is a rule about results, and it
    /// belongs where results are read.
    ///
    pub(super) fn admit(&self, encoding: &str) -> Result<SpanWrite, PortalError> {
        assert!(!encoding.is_empty(), "a write places at least one Cell");
        let content: Vec<CellContent> = encoding
            .bytes()
            .map(|byte| CellContent::new(byte).expect("a write contains printable ASCII Cells"))
            .collect();
        let width = content.len();
        let last = self
            .grid
            .offset_in_row(self.destination, width - 1)
            .ok_or(PortalError::CrossesRowEdge)?;

        Ok(SpanWrite {
            span: Span::new(self.grid, self.grid.index(self.destination), last),
            content,
        })
    }
}

///
/// A validated write of one encoding to a contiguous run of Cells.
///
/// The two ways a whole destination is refused are the two variants of
/// [`PortalError`]; each producer turns them into its own diagnostic, and
/// neither yields a `SpanWrite`. [`Portal::admit`] is the only constructor, so
/// a `SpanWrite` exists only because some Portal accepted its whole
/// destination, and a partial write is unrepresentable rather than merely
/// avoided. Cells are addressed only when the Tick Plan resolves, so no
/// producer can emit half of one either.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SpanWrite {
    span: Span,
    content: Vec<CellContent>,
}

impl SpanWrite {
    ///
    /// Each Cell this write covers, paired with what it receives.
    ///
    /// One encoding fans out into Cells here and nowhere earlier, which is
    /// what keeps ADR 0020's conflict resolution per Cell: an intact Sequence
    /// is one validated write until the Tick Plan resolves, and then it is as
    /// many independently contested Cells as it has characters.
    ///
    pub(super) fn cells(&self) -> impl Iterator<Item = (CellIndex, CellContent)> + '_ {
        self.span.indices().zip(self.content.iter().copied())
    }
}

#[cfg(test)]
mod test {
    use super::{Portal, PortalError};
    use crate::grid::{CellIndex, Grid};

    #[test]
    #[should_panic(expected = "printable ASCII")]
    fn generated_control_characters_are_internal_defects() {
        let grid = Grid::new(4, 2);
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        let _ = portal.admit("A\n");
    }

    ///
    /// The index `grid` mints for `idx`, so a test states a destination Cell
    /// in the same terms a Tick Plan carries.
    ///
    fn cell(grid: Grid, idx: usize) -> CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    ///
    /// Where `portal` places `encoding`, Cell by Cell. A Portal has no
    /// destination to read back on its own: what it resolved is observable
    /// only as the write it admits, which is the same thing a producer sees.
    ///
    fn placed(portal: &Portal, encoding: &str) -> Vec<(CellIndex, char)> {
        portal
            .admit(encoding)
            .expect("the encoding fits its row")
            .cells()
            .map(|(cell, content)| (cell, content.as_char()))
            .collect()
    }

    #[test]
    fn an_ordinary_result_passes_through_the_portal_below_its_root() {
        // ADR 0009's ordinary result destination, stated as the Cells it
        // reaches. The Grid is ten wide and the root sits at column 3 of row
        // 0, so the destination Cells are asymmetric in both coordinates: a
        // Portal that kept the root's own Cell, dropped to the row below but
        // reset to column 0, or transposed the two coordinates lands somewhere
        // other than 13 and 14.
        let grid = Grid::new(10, 3);
        let root = grid.position(3, 0).expect("inside the Grid");

        let portal = Portal::ordinary_result(grid, root).expect("a row below the root");

        assert_eq!(
            placed(&portal, "03"),
            vec![(cell(grid, 13), '0'), (cell(grid, 14), '3')]
        );
    }

    #[test]
    fn a_root_in_the_last_row_resolves_no_portal_at_all() {
        // ADR 0009 answers a missing destination with the resolution failure
        // rather than with a destination the caller then has to re-check. The
        // last row has no row below it, so there is no Portal to admit
        // anything through — not one whose writes would be refused later.
        let grid = Grid::new(10, 2);
        let last_row = grid.position(0, 1).expect("inside the Grid");

        assert_eq!(
            Portal::ordinary_result(grid, last_row).err(),
            Some(PortalError::BelowSource)
        );
    }

    #[test]
    fn a_portal_refuses_an_encoding_wider_than_the_cells_left_in_its_row() {
        // ADR 0004: the whole destination is validated before any Cell of it
        // exists. A row is the whole horizontal extent there is, so an
        // encoding running past its end is refused entire — the two Cells that
        // would have fitted are not admitted on their own — while the widest
        // encoding the row does hold is accepted.
        let grid = Grid::new(4, 2);
        let near_edge = grid.position(2, 1).expect("inside the Grid");
        let portal = Portal::at(grid, near_edge);

        assert_eq!(portal.admit("ABC").err(), Some(PortalError::CrossesRowEdge));
        assert_eq!(
            placed(&portal, "AB"),
            vec![(cell(grid, 6), 'A'), (cell(grid, 7), 'B')]
        );
    }

    #[test]
    fn a_whole_sequence_encoding_passes_through_one_portal_as_one_write() {
        // ADR 0007: an intact Sequence is one ordinary result through one
        // Portal, not a batch of Cell writes. A six-Cell encoding is therefore
        // admitted by the same call a two-Cell Atom uses, and lands on six
        // consecutive Cells of the destination row in encoding order.
        let grid = Grid::new(10, 3);
        let root = grid.position(0, 0).expect("inside the Grid");
        let portal = Portal::ordinary_result(grid, root).expect("a row below the root");

        assert_eq!(
            placed(&portal, "0A0B0C"),
            vec![
                (cell(grid, 10), '0'),
                (cell(grid, 11), 'A'),
                (cell(grid, 12), '0'),
                (cell(grid, 13), 'B'),
                (cell(grid, 14), '0'),
                (cell(grid, 15), 'C'),
            ]
        );
    }

    #[test]
    fn a_sequence_too_wide_for_its_destination_row_admits_nothing() {
        // The complete-fit rule ADR 0007 states for Sequences is the rule
        // ADR 0004 already states for any write, which is why a Sequence needs
        // no fit check of its own. Five Atoms need ten Cells; the destination
        // row has eight left, and the refusal costs the whole Sequence rather
        // than its first four Atoms.
        let grid = Grid::new(10, 3);
        let root = grid.position(2, 0).expect("inside the Grid");
        let portal = Portal::ordinary_result(grid, root).expect("a row below the root");

        assert_eq!(
            portal.admit("0A0B0C0D0E").err(),
            Some(PortalError::CrossesRowEdge)
        );
        assert_eq!(
            portal.admit("0A0B0C0D").map(|write| write.cells().count()),
            Ok(8)
        );
    }
}
