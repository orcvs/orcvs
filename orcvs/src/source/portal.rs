//! ADR 0009's Portal: where one interpreted result becomes Cells.
//!
//! A Portal is one Cell destination resolved during a Tick. [`PortalAccess`] is
//! how one computation interacts with Portals: whether it writes any, and which
//! extra Cells it reads. Both live here rather than beside the producers in
//! `tick` because destination resolution is the question ADR 0009 expects to
//! change: a future Cell-addressing model, an infinite canvas among them, moves
//! a result somewhere else without touching Function evaluation, effect
//! ordering, or Tick Plan commit. Keeping resolution in its own module is what
//! makes that a change to one file rather than a change threaded through the
//! producer that happened to hardcode "the row below the root".
//! Reads, write admission, Reservations, occupancy, and Jump's Language Unit
//! share its row-fit calculation; each caller retains the policy deciding how
//! much coverage it needs, and whether an occupied Portal diagnoses, activates,
//! locks, copies, or stays silent.
//!
//! Nothing in this module is reachable from the language crate, and nothing in
//! it is serialized. That is the whole of CONTEXT.md's "a Portal is neither a
//! language value, Source content, nor persistent state": a Portal cannot be
//! spelled in Source because no Atom carries one, cannot survive a Tick
//! because it is a local of the pass that resolved it, and cannot reach a save
//! file because `PersistedSource` carries a Grid and character Cells and this
//! type is not among them.

use std::ops::Range;

use lang::{Function, PortalCoords, SourceBundle};

use crate::grid::{CellIndex, Grid, Position};

use super::CellContent;
use super::encoding::Encoding;
use super::language_map::{LanguageMap, LanguageUnitKind, Span};

/// The Cell pair an Atom occupies. Jump reads that pair at the opposite
/// Portal; Tick reservations use the same width as `SCALAR_WIDTH`.
const PAIR_WIDTH: usize = 2;

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
/// What Language Units occupy Cells at a resolved destination.
///
/// Occupancy is a fact about the Source Snapshot's Language Map, not about
/// working Source: Bang cleanup clears a standalone `**` before any Turn, and
/// a Comment never writes, so the Map still names an occupied non-root after
/// those Cells look empty. Working Source vacancy is [`Portal::occupied_in`].
///
/// ADR 0006 and CONTEXT.md keep diagnose-versus-silent with the producer.
/// Halt, Jump Bang, and a blocked Self-Banging move each read this answer and
/// apply their own policy.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Occupancy {
    /// No Language Unit overlaps the Cells.
    Empty,
    /// One Language Unit covers every Cell, and it is an Expression root.
    Root(usize),
    /// One Language Unit covers every Cell, and it is not a root.
    NonRoot,
    /// A Language Unit is met across its edge.
    Partial,
}

///
/// The Language Unit occupying a Portal's two Cells, as Jump copies it.
///
/// Occupancy is the Snapshot Map. This is working Source at the Portal,
/// aligned against that Map and against admitted Sequence writes: Empty and
/// Bang are values even when the Map still names a cleaned unit; a complete
/// aligned unit is one too; a partial pair, a slice across two units, a
/// Comment, or a Sequence member is not. Decoding the admitted spelling is
/// the Interpreter's.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PortalUnit {
    /// Two space Cells. A cleaned standalone Bang is Empty here.
    Empty,
    /// Working Source holds `**`.
    Bang,
    /// One complete aligned unit Jump may copy. The Interpreter decodes it.
    Unit,
    /// Not one Language Unit Jump may copy.
    Invalid,
}

///
/// Why a Portal refused a whole destination.
///
/// All three are questions about the destination, and they answer into one
/// type because every producer treats them identically: ADR 0004 admits no
/// partial write, so any refusal costs the whole write and yields a diagnostic
/// instead. A producer distinguishes them only to say which it was.
///
/// Whether the content can be Cells at all is not among them. That is true of
/// a value wherever it lands and is settled by [`Encoding`] before a
/// destination is asked, which is why this type no longer carries a content
/// refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PortalError {
    /// There is no row below the producer's root, so no default Portal
    /// exists at all.
    BelowSource,
    /// The declared displacement lands outside the Grid, so the destination
    /// this Function asked for does not exist. It is [`PortalError::BelowSource`]
    /// generalised: that one is this refusal for the one displacement every
    /// Function used to take, and it is kept apart because its diagnostic names
    /// the row below rather than a displacement the Source never wrote.
    OutsideGrid,
    /// The encoding is wider than the destination row's remaining Cells.
    CrossesRowEdge,
}

impl Portal {
    ///
    /// The Portal `coords` name relative to `root`.
    ///
    /// The row below is the default Portal: leaving the Grid there is the row
    /// below, not a displacement the Source wrote. Any other coordinates use
    /// the same displaced resolution Jump already takes.
    ///
    pub(super) fn named(
        grid: Grid,
        root: Position,
        coords: PortalCoords,
    ) -> Result<Self, PortalError> {
        if coords == PortalCoords::SOUTH {
            Self::below(grid, root)
        } else {
            Self::displaced(grid, root, coords.columns, coords.rows)
        }
    }

    ///
    /// The default Portal: the row below `root`, in `root`'s own column.
    ///
    /// A root in the last row resolves no Portal rather than a clamped one.
    /// Clamping would put a result in a Cell the Source never asked for, and a
    /// destination that does not exist is exactly what ADR 0009 wants reported
    /// — the caller diagnoses at the root and plans nothing.
    ///
    pub(super) fn below(grid: Grid, root: Position) -> Result<Self, PortalError> {
        grid.below(root)
            .map(|destination| Self::at(grid, destination))
            .ok_or(PortalError::BelowSource)
    }

    ///
    /// The Portal `columns` Cells east and `rows` Cells south of `root`.
    ///
    /// The destination a Function declaring its own Portal offset resolves,
    /// where [`Portal::below`] is the destination every other Function
    /// takes. ADR 0006 states this geometry in coordinates — north
    /// `(x, y-1)`, west `(x-2, y)` — and ADR 0009 keeps the resolution here,
    /// so the language crate answers a displacement and never a Position.
    ///
    /// A displacement leaving the Grid resolves no Portal, for the reason
    /// `below` resolves none below the last row: the caller diagnoses at
    /// the root and plans nothing.
    ///
    pub(super) fn displaced(
        grid: Grid,
        root: Position,
        columns: i16,
        rows: i16,
    ) -> Result<Self, PortalError> {
        grid.displaced(root, columns, rows)
            .map(|destination| Self::at(grid, destination))
            .ok_or(PortalError::OutsideGrid)
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

    /// The first Cell reached by this Portal.
    pub(super) fn destination(self) -> Position {
        self.destination
    }

    /// Complete coverage of a nonempty read or write, without leaving this row.
    /// Callers supply widths from nonempty encodings, declared input types,
    /// or Reservations. Empty results never reach a Portal.
    pub(super) fn span(self, width: usize) -> Result<Span, PortalError> {
        let offset = width.checked_sub(1).expect("Portal coverage is nonempty");
        let last = self
            .grid
            .offset_in_row(self.destination, offset)
            .ok_or(PortalError::CrossesRowEdge)?;
        Ok(Span::new(
            self.grid,
            self.grid.index(self.destination),
            last,
        ))
    }

    ///
    /// All Cells from this destination through the end of its own row.
    /// A resolved destination always has at least its own Cell remaining.
    pub(super) fn remaining_span(self) -> Span {
        self.span(self.grid.columns() - self.destination.x())
            .expect("the remaining Cells of a resolved Portal fit its row")
    }

    ///
    /// Language-Map occupancy of up to `width` Cells from this destination.
    ///
    /// A destination in the last column still answers the one Cell it holds,
    /// so occupancy is not refused at the row edge the way a two-Cell write
    /// is. Halt asks this of a Portal that resolved; the root it locks is a
    /// separate question about whether an Expression root is anchored at the
    /// destination itself.
    ///
    pub(super) fn occupancy(
        self,
        map: &LanguageMap,
        width: usize,
        root_at: impl Fn(Position) -> Option<usize>,
    ) -> Occupancy {
        occupancy_of(map, &self.cells_along_row(width), root_at)
    }

    ///
    /// Whether any of `width` Cells from this destination holds a non-space
    /// in working Source.
    ///
    /// A span that cannot fit answers false: the write path refuses the row
    /// edge itself. Jump Bang asks this after a root at the destination has
    /// already been offered activation, which is why a cleaned standalone
    /// Bang — empty in working Source, still a unit on the Map — writes
    /// rather than diagnosing.
    ///
    pub(super) fn occupied_in(self, working: &[u8], width: usize) -> bool {
        self.span(width)
            .ok()
            .is_some_and(|span| working[span.range()].iter().any(|&byte| byte != b' '))
    }

    ///
    /// The Language Unit Jump may copy from `width` Cells of working Source.
    ///
    /// `sequence_covers` is the Tick's admitted Sequence writes: membership
    /// is the write, not the reservation. A same-Tick scalar that lands in a
    /// reserved tail is a unit of its own; a Sequence member is not.
    ///
    pub(super) fn language_unit(
        self,
        working: &[u8],
        map: &LanguageMap,
        width: usize,
        sequence_covers: impl Fn(std::ops::Range<usize>) -> bool,
    ) -> PortalUnit {
        let Ok(span) = self.span(width) else {
            return PortalUnit::Invalid;
        };
        let range = span.range();
        let cells = &working[range.clone()];
        if cells.iter().all(|&byte| byte == b' ') {
            return PortalUnit::Empty;
        }
        if cells == b"**" {
            return PortalUnit::Bang;
        }
        if cells.contains(&b' ') || sequence_covers(range.clone()) {
            return PortalUnit::Invalid;
        }
        let covering: Vec<_> = map
            .units()
            .filter(|unit| spans_overlap(unit.span().range(), range.clone()))
            .collect();
        match covering.as_slice() {
            [unit] if unit.span().range() == range => match unit.kind() {
                LanguageUnitKind::Comment => PortalUnit::Invalid,
                LanguageUnitKind::Bang
                | LanguageUnitKind::Function(_)
                | LanguageUnitKind::OperandLiteral => PortalUnit::Unit,
            },
            [] => PortalUnit::Unit,
            _ => PortalUnit::Invalid,
        }
    }

    /// Cells from this destination along its row, at most `width` of them.
    fn cells_along_row(self, width: usize) -> Vec<usize> {
        let start = self.grid.index(self.destination).get();
        let available = self.grid.columns() - self.destination.x();
        (start..start + width.min(available)).collect()
    }

    /// The write that places `encoding` at this Portal and along its row, or
    /// the reason the whole destination was refused.
    ///
    /// The whole destination is decided before a [`SpanWrite`] exists, so
    /// ADR 0004's "validate the complete effect before admitting any of it" is
    /// a property of this constructor rather than a discipline every caller
    /// keeps. A refused encoding leaves nothing behind to emit half of: there
    /// is no value describing part of a write.
    ///
    /// An [`Encoding`] places at least one printable Cell by construction, so
    /// the only question left here is how far along the row those Cells reach.
    /// The write retains validated CellContent values through resolution and
    /// commit.
    ///
    /// A value that plans no write renders to [`super::encoding::Rendered::Nothing`]
    /// and never reaches a Portal; that is a rule about results, and it belongs
    /// where results are read.
    ///
    pub(super) fn admit(&self, encoding: &Encoding) -> Result<SpanWrite, PortalError> {
        let content: Vec<CellContent> = encoding.content();
        Ok(SpanWrite {
            span: self.span(content.len())?,
            content,
        })
    }
}

///
/// Occupancy of arbitrary Cells, as a blocked move's newly entered Cells.
///
/// ADR 0006 classifies contact against every Language Unit rather than
/// against the computations alone: a Comment and a standalone Bang each
/// occupy Cells and neither is scheduled. `root_at` is the Lookup question
/// that tells a complete covering unit from an Expression root; this module
/// holds no Lookup.
///
pub(super) fn occupancy_of(
    map: &LanguageMap,
    cells: &[usize],
    root_at: impl Fn(Position) -> Option<usize>,
) -> Occupancy {
    let mut partial = false;
    for unit in map.units() {
        let covered = unit.span().start().get()..=unit.span().end().get();
        if !cells.iter().any(|cell| covered.contains(cell)) {
            continue;
        }
        if !cells.iter().all(|cell| covered.contains(cell)) {
            partial = true;
            continue;
        }
        return match root_at(unit.anchor()) {
            Some(root) => Occupancy::Root(root),
            None => Occupancy::NonRoot,
        };
    }
    if partial {
        Occupancy::Partial
    } else {
        Occupancy::Empty
    }
}

fn spans_overlap(left: std::ops::Range<usize>, right: std::ops::Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

///
/// A validated write of one encoding to a contiguous run of Cells.
///
/// The ways a whole write is refused are the variants of
/// [`PortalError`]; each producer turns them into its own diagnostic, and
/// none yields a `SpanWrite`. [`Portal::admit`] is the only constructor, so
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
    /// Coverage already validated by admission, independent of the Reservation.
    pub(super) fn span(&self) -> Span {
        self.span
    }

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

///
/// How one computation interacts with Portals: Cell writes or a root lock,
/// and the extra Cells it reads.
///
/// A Portal is one Cell. [`PortalOutput`] distinguishes a Cell write from
/// a root lock, so a lock never supplies an Input Portal's characters. Reads
/// are independent of that: a nested Jump writes nothing and still reads the
/// opposite Portal, so a producer of those Cells is ordered first.
///
/// Terminal Output answers Play, not a Cell, so its writes are [`PortalOutput::None`]
/// — a kind, so [`Self::carry`] cannot mint a site the resolve step refused.
/// Empty-vec silence was the leak: a test helper could stuff a Portal onto
/// `!>`. Play stays an Effect; it is not a Portal.
///
#[derive(Clone, Debug, PartialEq)]
pub(super) struct PortalAccess {
    output: PortalOutput,
    reads: Vec<Range<usize>>,
}

///
/// What a computation delivers at its output: Cell writes or a root lock.
///
/// [`PortalOutput::None`] is a kind, not an empty site list. An empty list can
/// be stuffed; none cannot.
///
#[derive(Clone, Debug, PartialEq)]
enum PortalOutput {
    None,
    Writes(Vec<Result<Position, PortalError>>),
    Lock(Result<Position, PortalError>),
}

impl PortalAccess {
    ///
    /// The output kind, destination sites and extra reads demanded at `anchor`.
    ///
    /// Nested computations hand a typed value to a parent. Terminal Output
    /// answers Play. Neither demands a write Portal. A locking root and every
    /// other Value name their Output Portal on the Function; Jump and the
    /// feedback Functions also name an Input Portal. A nested Jump still
    /// reads that Input Portal. A Source write states its declared bundle.
    ///
    pub(super) fn resolve(grid: Grid, anchor: Position, function: Function, nested: bool) -> Self {
        let reads = function
            .input_portal()
            .map(|coords| Self::portal_reads(grid, anchor, coords))
            .unwrap_or_default();
        if nested {
            return Self {
                output: PortalOutput::None,
                reads,
            };
        }
        if function.performs_terminal_output() {
            return Self {
                output: PortalOutput::None,
                reads: Vec::new(),
            };
        }
        if let Some(effect) = function.source_effect() {
            // ADR 0004's effect bundle, in the emission order ADR 0020
            // resolves it by. An `Advance` clears the Cells the producer
            // stands in before it writes; an `Emit` plans nothing at its own
            // Cells. The displacement is read from the declaration here and
            // answered again by the Interpreter at the Turn.
            let destination = Portal::displaced(grid, anchor, effect.columns, effect.rows)
                .map(|portal| portal.destination());
            let writes = match effect.bundle {
                SourceBundle::Advance => vec![Ok(anchor), destination],
                SourceBundle::Emit => vec![destination],
            };
            return Self {
                output: PortalOutput::Writes(writes),
                reads: Vec::new(),
            };
        }
        if let Some(coords) = function.output_portal() {
            let site = Portal::named(grid, anchor, coords).map(|portal| portal.destination());
            let output = if function.locks_root() {
                PortalOutput::Lock(site)
            } else {
                PortalOutput::Writes(vec![site])
            };
            return Self { output, reads };
        }
        Self {
            output: PortalOutput::None,
            reads,
        }
    }

    fn portal_reads(grid: Grid, anchor: Position, coords: PortalCoords) -> Vec<Range<usize>> {
        Portal::named(grid, anchor, coords)
            .ok()
            .and_then(|portal| portal.span(PAIR_WIDTH).ok())
            .map(|span| vec![span.range()])
            .unwrap_or_default()
    }

    pub(super) fn writes_cells(&self) -> bool {
        matches!(self.output, PortalOutput::Writes(_))
    }

    pub(super) fn write_sites(&self) -> &[Result<Position, PortalError>] {
        match &self.output {
            PortalOutput::None | PortalOutput::Lock(_) => &[],
            PortalOutput::Writes(writes) => writes,
        }
    }

    /// The lock's destination, including failure to resolve outside the Grid.
    pub(super) fn lock_site(&self) -> Option<Result<Position, PortalError>> {
        match self.output {
            PortalOutput::Lock(site) => Some(site),
            PortalOutput::None | PortalOutput::Writes(_) => None,
        }
    }

    pub(super) fn read_spans(&self) -> &[Range<usize>] {
        &self.reads
    }

    ///
    /// Restates write sites a test named, only when this value already demanded
    /// some.
    ///
    /// [`PortalOutput::None`] stays none. That is the whole of the helper: it
    /// cannot attach a write to Terminal Output, a nested Jump, or a root lock.
    ///
    #[cfg(test)]
    pub(super) fn carry(&mut self, grid: Grid, writes: &[Position]) {
        match &mut self.output {
            PortalOutput::None | PortalOutput::Lock(_) => {}
            PortalOutput::Writes(sites) => {
                *sites = writes
                    .iter()
                    .map(|output| {
                        grid.assert_owns(*output);
                        Ok(*output)
                    })
                    .collect();
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn remaining_portal_coverage_stops_at_its_own_row_end() {
        for (columns, column, first, last) in
            [(5, 0, 5, 9), (5, 2, 7, 9), (5, 4, 9, 9), (1, 0, 1, 1)]
        {
            let grid = Grid::with_shape(columns, 3);
            let portal = Portal::at(grid, grid.position(column, 1).unwrap());
            let span = portal.remaining_span();
            assert_eq!(span.start(), cell(grid, first));
            assert_eq!(span.end(), cell(grid, last));
        }
    }

    #[test]
    fn portal_coverage_fits_the_complete_width_or_refuses_it() {
        let grid = Grid::with_shape(5, 3);
        let portal = Portal::at(grid, grid.position(3, 1).unwrap());
        let span = portal.span(2).unwrap();
        assert_eq!(span.start(), cell(grid, 8));
        assert_eq!(span.end(), cell(grid, 9));
        assert_eq!(portal.span(3), Err(PortalError::CrossesRowEdge));
    }

    use super::{
        Encoding, LanguageMap, Occupancy, Portal, PortalAccess, PortalError, PortalUnit,
        occupancy_of,
    };
    use crate::grid::{CellIndex, Grid};

    #[test]
    fn every_printable_cell_reaches_its_destination() {
        // What a Portal is left holding once content validity belongs to
        // `Encoding`: the Cells arrive as given, and the refusal that used to
        // sit here is tested where the rule now lives.
        let grid = Grid::with_shape(4, 2);
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        assert_eq!(
            placed(&portal, " ~"),
            vec![(cell(grid, 0), ' '), (cell(grid, 1), '~')]
        );
    }

    ///
    /// The index `grid` mints for `idx`, so a test states a destination Cell
    /// in the same terms a Tick Plan carries.
    ///
    fn cell(grid: Grid, idx: usize) -> CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    fn unit(portal: Portal, working: &[u8], map: &LanguageMap, sequence: bool) -> PortalUnit {
        portal.language_unit(working, map, 2, |_| sequence)
    }

    ///
    /// What `portal` answers for `encoding`, stated as Source text so a test
    /// says what it means. Content validity belongs to [`Encoding`], so every
    /// refusal reaching here is one about the destination.
    ///
    fn admitted(portal: &Portal, encoding: &str) -> Result<super::SpanWrite, PortalError> {
        portal.admit(&Encoding::literal(encoding).expect("printable test content"))
    }

    ///
    /// Where `portal` places `encoding`, Cell by Cell. A Portal has no
    /// destination to read back on its own: what it resolved is observable
    /// only as the write it admits, which is the same thing a producer sees.
    ///
    fn placed(portal: &Portal, encoding: &str) -> Vec<(CellIndex, char)> {
        admitted(portal, encoding)
            .expect("the encoding fits its row")
            .cells()
            .map(|(cell, content)| (cell, content.as_char()))
            .collect()
    }

    #[test]
    fn the_default_portal_is_below_the_root() {
        // The default Portal: the row below the root, in the root's own
        // column. The Grid is ten wide and the root sits at
        // column 3 of row 0, so the destination Cells are asymmetric in both
        // coordinates: a Portal that kept the root's own Cell, dropped to the
        // row below but reset to column 0, or transposed the two coordinates
        // lands somewhere other than 13 and 14.
        let grid = Grid::with_shape(10, 3);
        let root = grid.position(3, 0).expect("inside the Grid");

        let portal = Portal::below(grid, root).expect("a row below the root");

        assert_eq!(
            placed(&portal, "03"),
            vec![(cell(grid, 13), '0'), (cell(grid, 14), '3')]
        );
    }

    #[test]
    fn a_declared_displacement_resolves_the_cell_it_names_in_both_axes() {
        // ADR 0006's geometry, stated as the Cells it reaches. The producer
        // sits at column 3 of row 1 in a Grid ten wide, so each of the four
        // displacements lands on a different index and a transposed or
        // sign-flipped offset lands on none of them.
        let grid = Grid::with_shape(10, 3);
        let root = grid.position(3, 1).expect("inside the Grid");

        for (columns, rows, first) in [(0, -1, 3), (0, 1, 23), (-1, 0, 12), (1, 0, 14)] {
            let portal = Portal::displaced(grid, root, columns, rows).expect("inside the Grid");
            assert_eq!(
                placed(&portal, "><"),
                vec![(cell(grid, first), '>'), (cell(grid, first + 1), '<')],
                "({columns}, {rows})",
            );
        }
    }

    #[test]
    fn a_displacement_off_the_grid_resolves_no_portal_at_all() {
        // The refusal `below` gives the last row, generalised to
        // every edge a declared offset can reach. Each of these is outside the
        // Grid in one coordinate, and none of them is clamped to the edge Cell
        // beside it: a destination that does not exist is what ADR 0009 wants
        // reported.
        let grid = Grid::with_shape(4, 2);
        let corner = grid.position(0, 0).expect("inside the Grid");

        for (columns, rows) in [(0, -1), (-1, 0), (4, 0), (0, 2)] {
            assert_eq!(
                Portal::displaced(grid, corner, columns, rows).err(),
                Some(PortalError::OutsideGrid),
                "({columns}, {rows})",
            );
        }
    }

    #[test]
    fn a_displacement_inside_the_grid_is_still_refused_past_its_row_edge() {
        // The two refusals are separate questions and this is the Cell that
        // proves it: displacing east from the last column pair of row 0 names
        // a Position the Grid holds, and the two-Cell encoding placed there
        // runs into row 1. Resolution succeeds and admission does not.
        let grid = Grid::with_shape(4, 2);
        let root = grid.position(2, 0).expect("inside the Grid");
        let portal = Portal::displaced(grid, root, 1, 0).expect("inside the Grid");

        assert_eq!(
            admitted(&portal, "><").err(),
            Some(PortalError::CrossesRowEdge)
        );
    }

    #[test]
    fn a_root_in_the_last_row_resolves_no_portal_at_all() {
        // ADR 0009 answers a missing destination with the resolution failure
        // rather than with a destination the caller then has to re-check. The
        // last row has no row below it, so there is no Portal to admit
        // anything through — not one whose writes would be refused later.
        let grid = Grid::with_shape(10, 2);
        let last_row = grid.position(0, 1).expect("inside the Grid");

        assert_eq!(
            Portal::below(grid, last_row).err(),
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
        let grid = Grid::with_shape(4, 2);
        let near_edge = grid.position(2, 1).expect("inside the Grid");
        let portal = Portal::at(grid, near_edge);

        assert_eq!(
            admitted(&portal, "ABC").err(),
            Some(PortalError::CrossesRowEdge)
        );
        assert_eq!(
            placed(&portal, "AB"),
            vec![(cell(grid, 6), 'A'), (cell(grid, 7), 'B')]
        );
    }

    #[test]
    fn a_whole_sequence_encoding_passes_through_one_portal_as_one_write() {
        // ADR 0007: an intact Sequence passes through one Portal, not a batch
        // of Cell writes. A six-Cell encoding is therefore admitted by the
        // same call a two-Cell Atom uses, and lands on six consecutive Cells
        // of the destination row in encoding order.
        let grid = Grid::with_shape(10, 3);
        let root = grid.position(0, 0).expect("inside the Grid");
        let portal = Portal::below(grid, root).expect("a row below the root");

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
        let grid = Grid::with_shape(10, 3);
        let root = grid.position(2, 0).expect("inside the Grid");
        let portal = Portal::below(grid, root).expect("a row below the root");

        assert_eq!(
            admitted(&portal, "0A0B0C0D0E").err(),
            Some(PortalError::CrossesRowEdge)
        );
        assert_eq!(
            admitted(&portal, "0A0B0C0D").map(|write| write.cells().count()),
            Ok(8)
        );
    }

    #[test]
    fn a_root_terminal_output_function_resolves_silent_after_carry() {
        // Terminal Output answers Play, not a Cell write. PortalAccess is how
        // a computation interacts with Portals: a root `!>` never demands a
        // write site, and carry cannot mint one. Empty-vec silence was the
        // leak — it could be stuffed.
        let grid = Grid::with_shape(8, 2);
        let anchor = grid.position(0, 0).expect("inside the Grid");
        let elsewhere = grid.position(0, 1).expect("inside the Grid");
        let mut access = PortalAccess::resolve(grid, anchor, lang::Function::RawPlay, false);
        access.carry(grid, &[elsewhere]);
        assert!(!access.writes_cells());
        assert_eq!(access.write_sites().len(), 0);
    }

    #[test]
    fn a_nested_jump_keeps_its_input_portal_read_and_writes_no_cell() {
        // Nested Jump answers a value to its parent and writes no Cell. The
        // opposite Portal is still a read, so a producer of those Cells is
        // ordered first. Carry cannot mint a write the resolve step refused.
        let grid = Grid::with_shape(8, 2);
        let anchor = grid.position(2, 0).expect("inside the Grid");
        let input = grid.position(2, 1).expect("inside the Grid");
        let expected = Portal::at(grid, input)
            .span(2)
            .expect("the input Portal fits the row")
            .range();
        let mut access = PortalAccess::resolve(grid, anchor, lang::Function::JumpNorth, true);
        access.carry(grid, &[input]);
        assert!(!access.writes_cells());
        assert!(access.write_sites().is_empty());
        assert_eq!(access.read_spans(), &[expected]);
    }

    #[test]
    fn occupancy_of_empty_cells_is_empty() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b".+0102  ");
        let portal = Portal::at(grid, grid.position(6, 0).unwrap());
        assert_eq!(portal.occupancy(&map, 2, |_| None), Occupancy::Empty);
    }

    #[test]
    fn occupancy_of_a_complete_non_root_is_non_root() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b".+0102  ");
        let portal = Portal::at(grid, grid.position(2, 0).unwrap());
        assert_eq!(portal.occupancy(&map, 2, |_| None), Occupancy::NonRoot);
    }

    #[test]
    fn occupancy_of_a_complete_root_is_root() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b".+0102  ");
        let root = grid.position(0, 0).unwrap();
        let portal = Portal::at(grid, root);
        assert_eq!(
            portal.occupancy(&map, 2, |anchor| (anchor == root).then_some(0)),
            Occupancy::Root(0)
        );
    }

    #[test]
    fn occupancy_across_two_units_is_partial() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b".+0102  ");
        let portal = Portal::at(grid, grid.position(3, 0).unwrap());
        assert_eq!(portal.occupancy(&map, 2, |_| None), Occupancy::Partial);
    }

    #[test]
    fn one_entered_cell_of_a_root_is_complete_root_contact() {
        // A horizontal move enters one Cell. The root whose Span holds that
        // Cell is anchored two columns west, which is ADR 0006's east-anchor
        // geometry rather than a root that begins at the entered Cell.
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b".+0102  ");
        let root = grid.position(0, 0).unwrap();
        assert_eq!(
            occupancy_of(&map, &[1], |anchor| (anchor == root).then_some(0)),
            Occupancy::Root(0)
        );
    }

    #[test]
    fn last_column_occupancy_covers_the_one_cell_it_holds() {
        let grid = Grid::with_shape(4, 1);
        let map = LanguageMap::build(grid, b"  >>");
        let root = grid.position(2, 0).unwrap();
        let portal = Portal::at(grid, grid.position(3, 0).unwrap());
        assert_eq!(
            portal.occupancy(&map, 2, |anchor| (anchor == root).then_some(0)),
            Occupancy::Root(0)
        );
    }

    #[test]
    fn occupied_in_answers_working_source_spaces() {
        let grid = Grid::with_shape(4, 1);
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        assert!(!portal.occupied_in(b"    ", 2));
        assert!(portal.occupied_in(b"x   ", 2));
        assert!(portal.occupied_in(b" x  ", 2));
        let last = Portal::at(grid, grid.position(3, 0).unwrap());
        assert!(!last.occupied_in(b"   x", 2));
    }

    #[test]
    fn language_unit_of_empty_working_cells_is_empty() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b".+0102  ");
        let portal = Portal::at(grid, grid.position(6, 0).unwrap());
        assert_eq!(unit(portal, b".+0102  ", &map, false), PortalUnit::Empty);
    }

    #[test]
    fn language_unit_of_a_cleaned_bang_is_empty() {
        // Occupancy still names the Map unit. Jump copies working Source, so
        // a cleaned standalone Bang is Empty rather than Invalid.
        let grid = Grid::with_shape(4, 1);
        let map = LanguageMap::build(grid, b"**  ");
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        assert_eq!(portal.occupancy(&map, 2, |_| None), Occupancy::NonRoot);
        assert_eq!(unit(portal, b"    ", &map, false), PortalUnit::Empty);
    }

    #[test]
    fn language_unit_of_working_bang_is_bang() {
        let grid = Grid::with_shape(4, 1);
        let map = LanguageMap::build(grid, b"**  ");
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        assert_eq!(unit(portal, b"**  ", &map, false), PortalUnit::Bang);
    }

    #[test]
    fn language_unit_of_a_complete_aligned_unit_is_unit() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b".+0102  ");
        let portal = Portal::at(grid, grid.position(2, 0).unwrap());
        assert_eq!(unit(portal, b".+0102  ", &map, false), PortalUnit::Unit);
    }

    #[test]
    fn language_unit_across_two_units_is_invalid() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b".+0102  ");
        let portal = Portal::at(grid, grid.position(3, 0).unwrap());
        assert_eq!(unit(portal, b".+0102  ", &map, false), PortalUnit::Invalid);
    }

    #[test]
    fn language_unit_of_a_partial_pair_is_invalid() {
        let grid = Grid::with_shape(6, 1);
        let map = LanguageMap::build(grid, b"0 &>xx");
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        assert_eq!(unit(portal, b"0 &>xx", &map, false), PortalUnit::Invalid);
    }

    #[test]
    fn language_unit_inside_a_sequence_write_is_invalid() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b"        ");
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        assert_eq!(unit(portal, b"0001    ", &map, true), PortalUnit::Invalid);
    }

    #[test]
    fn language_unit_past_a_sequence_write_is_the_working_cells() {
        let grid = Grid::with_shape(8, 1);
        let map = LanguageMap::build(grid, b"        ");
        let empty = Portal::at(grid, grid.position(4, 0).unwrap());
        assert_eq!(unit(empty, b"0001    ", &map, false), PortalUnit::Empty);
        let number = Portal::at(grid, grid.position(4, 0).unwrap());
        assert_eq!(unit(number, b"000101  ", &map, false), PortalUnit::Unit);
    }

    #[test]
    fn language_unit_of_a_comment_is_invalid() {
        let grid = Grid::with_shape(2, 1);
        let map = LanguageMap::build(grid, b"||");
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        assert_eq!(unit(portal, b"||", &map, false), PortalUnit::Invalid);
    }

    #[test]
    fn language_unit_alignment_does_not_decode_the_spelling() {
        // "xx" is not an Atom. The Portal still admits the aligned pair; jump()
        // is the decoder that diagnoses it.
        let grid = Grid::with_shape(4, 1);
        let map = LanguageMap::build(grid, b"    ");
        let portal = Portal::at(grid, grid.position(0, 0).unwrap());
        assert_eq!(unit(portal, b"xx  ", &map, false), PortalUnit::Unit);
    }

    #[test]
    fn language_unit_that_cannot_fit_is_invalid() {
        let grid = Grid::with_shape(4, 1);
        let map = LanguageMap::build(grid, b"   x");
        let portal = Portal::at(grid, grid.position(3, 0).unwrap());
        assert_eq!(unit(portal, b"   x", &map, false), PortalUnit::Invalid);
    }
}
