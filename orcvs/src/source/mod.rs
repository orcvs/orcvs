mod cell;
pub use cell::CellContent;
mod encoding;
pub mod error;
pub mod file;
mod language_map;
pub use lang::Token;
// `Claim::atom`'s type: re-exported so a caller naming a `Claim` value — the
// console's colour tests among them (`syntax-highlighting/09`) — can spell
// `Atom` without adding a direct dependency on `lang`.
pub use lang::Atom;
pub use language_map::{Claim, ExpressionEntry, LanguageMap, LanguageUnit, LanguageUnitKind, Span};
use language_map::{
    OUTPUT_PORTAL_SCALAR_WIDTH, OUTPUT_PORTAL_SEQUENCE_MINIMUM_WIDTH, OutputPortalReservation,
};
mod model;
mod portal;
mod tick;
use crate::grid::{CellIndex, Grid, Position};
pub use error::SourceError;
pub use lang::Tick;
pub use model::{
    BendLsb, BendMsb, CellWrite, ControlValue, Controller, Diagnostic, Length, MidiChannel, Note,
    Performance, PlayCommand, RevisionId, Source, TickPlan, Velocity,
};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The language fact the console paints for one Source Cell.
///
/// This is deliberately colour-free. A Render Frame answers the semantic
/// distinction once; the console's theme decides how to present it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SourcePaint {
    Unclaimed,
    Function,
    Bang,
    Comment,
    Operand { token: Token, state: OperandState },
}

/// Whether a declared operand slot is waiting, valid, or contains content
/// that did not bind as its declared Token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperandState {
    Pending,
    Valid,
    Invalid,
}

fn read_recover<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write_recover<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Clone)]
pub struct SourceCommander {
    inner: Arc<RwLock<Source>>,
}

/// Character Cells and semantic language information observed from exactly one
/// Source revision.
#[derive(Clone)]
pub struct SourceRevision {
    grid: Grid,
    source: String,
    language_map: Arc<LanguageMap>,
}

impl SourceRevision {
    pub fn grid(&self) -> Grid {
        self.grid
    }

    pub fn content_at(&self, position: Position) -> Option<char> {
        self.grid.assert_owns(position);
        let byte = self.source.as_bytes()[self.grid.index(position).get()];
        (byte != b' ').then_some(char::from(byte))
    }

    pub fn language_map(&self) -> &LanguageMap {
        &self.language_map
    }

    ///
    /// The parser's claim on each Cell of this revision, in the Grid's
    /// row-major order: [`LanguageMap::claims_by_cell`] read against this
    /// revision's own Cell contents, which is what lets each claim answer
    /// [`Claim::written`] as it is built.
    ///
    pub(crate) fn claims_by_cell(&self) -> Vec<Option<Arc<Claim>>> {
        self.language_map.claims_by_cell(self.source.as_bytes())
    }

    ///
    /// The Token this revision answers at `position`.
    ///
    /// The Language Map's claim first; leftover `Char` when the Cell has
    /// content no Expression covered; `None` when the Cell is empty and
    /// unclaimed.
    ///
    pub fn token_at(&self, position: Position) -> Option<Token> {
        self.language_map
            .token_at(position)
            .or_else(|| self.content_at(position).map(|_| Token::Char))
    }

    ///
    /// Whether each Cell of this revision draws as a root Function's Output
    /// Portal, in the Grid's row-major order.
    ///
    /// The highlight, not the Reservation. Tick scheduling still reserves a
    /// Sequence-capable root the rest of its destination row (ADR 0036) and
    /// [`LanguageMap::output_portal_reservations`] still answers exactly that;
    /// this narrows the Sequence-capable ones to the answer they hold, which
    /// is `.scratch/syntax-highlighting/issues/12`. A whole-row highlight ran
    /// under the neighbouring column's Expressions in a two-column layout,
    /// which is the defect that ticket records.
    ///
    /// The fit lives here because it needs both inputs at once: the
    /// Reservations, which only the Language Map derives, and the Cell
    /// contents of this revision, which the Language Map deliberately does not
    /// retain. A Source revision is the one value that holds both.
    ///
    /// A scalar root keeps its Cell pair untouched. A Sequence-capable root
    /// takes at least [`OUTPUT_PORTAL_SEQUENCE_MINIMUM_WIDTH`] Cells, written
    /// or not — a Function that never answered more than two Cells would be
    /// declared scalar, so the minimum is what tells the two apart before any
    /// Tick. Past the minimum it follows the run of written Cells, one Cell
    /// pair at a time, and stops at the first blank Cell. Every step is
    /// clipped to the Reservation, so a root whose row edge leaves fewer Cells
    /// than the minimum takes the Cells that are there and no more.
    ///
    /// **Written** is [`Self::content_at`]'s question, so a Cell holding
    /// [`CellContent::SPACE`] is blank: a space reads back identically to a
    /// Cell never written, and the highlight has no other fact to tell them
    /// apart. The run is what the extension follows, and the pair it stops
    /// inside is taken whole: stopping mid-pair would draw a written Cell
    /// outside the highlight that covers the answer it belongs to.
    ///
    /// A run can end mid-pair because the Cell, not the Atom, is the unit
    /// here: `lang`'s `Atom::Char` spells one Cell where every other Atom
    /// spells two, and a Sequence admits a Char as a member. Nothing answers
    /// one today — no Function signature declares a Char operand, no
    /// Sequence-producing Function can introduce one, and the Parser refuses
    /// to read one out of Source — so every answer the language can currently
    /// deliver is pair-aligned from the Portal and holds no blank Cell. A
    /// Function that answered a Char would break that alignment, and a Char
    /// holding a space would put a blank Cell inside an answer, which this
    /// rule reads as its end.
    ///
    /// A **blank Cell**, not a wholly blank pair, is what ends the run. A
    /// pair counted by either of its Cells stepped over a gutter narrower
    /// than an aligned blank pair: the neighbouring column's first glyph wrote
    /// the far Cell of the pair the gutter fell in, the extension resumed
    /// through that column's Expression, and the fit degenerated to the
    /// whole-row tint this derivation exists to remove. One blank Cell ends
    /// the answer, however the columns happen to be aligned.
    ///
    pub(crate) fn output_portal_highlight(&self) -> Vec<bool> {
        let mut covered = vec![false; self.grid.count()];
        for reservation in self.language_map.output_portal_reservations() {
            for index in self.fitted(&reservation) {
                covered[index] = true;
            }
        }
        covered
    }

    /// [`Self::output_portal_highlight`]'s rule for one Reservation.
    fn fitted(&self, reservation: &OutputPortalReservation) -> std::ops::Range<usize> {
        let std::ops::Range { start, end } = reservation.range;
        if !reservation.sequence_capable {
            return start..end;
        }
        let mut fitted = end.min(start + OUTPUT_PORTAL_SEQUENCE_MINIMUM_WIDTH);
        while fitted < end && self.written(fitted) {
            fitted = end.min(fitted + OUTPUT_PORTAL_SCALAR_WIDTH);
        }
        start..fitted
    }

    ///
    /// Whether the Cell at `index` holds content: [`Self::content_at`]'s own
    /// question, asked of the Position that index names.
    ///
    /// Asked through the Grid rather than by indexing the Source bytes
    /// directly, so the doc above naming `content_at` as the authority on
    /// written has one definition to name and no second space test to drift
    /// from. [`Grid::cell_index`] is also the only thing that turns a bare
    /// number into an index this Grid can address: every index reaching here
    /// comes from a Reservation this revision's own Language Map derived and
    /// so is always in range, and one outside it is answered rather than
    /// panicked on.
    ///
    fn written(&self, index: usize) -> bool {
        self.grid
            .cell_index(index)
            .is_some_and(|cell| self.content_at(self.grid.position_at(cell)).is_some())
    }
}

impl SourceCommander {
    pub fn new(grid: Grid) -> Self {
        Self::with_source(Source::new(grid))
    }

    ///
    /// Commands `source`: a Source built elsewhere, such as one read back from
    /// persistence. The Grid is the one that Source was built from, so it is
    /// the only Grid that mints an index addressing one of its Cells.
    ///
    /// ```
    /// use orcvs::grid::Grid;
    /// use orcvs::source::{Source, SourceCommander};
    ///
    /// let mut built = Source::new(Grid::new());
    /// let cell = built.grid().cell_index(0).expect("inside the Grid");
    /// built.set(cell, "1").expect("a Cell the Source accepts");
    ///
    /// let source = SourceCommander::with_source(built);
    ///
    /// // the Grid comes with the Source, and it is the Grid that mints the
    /// // index naming one of its Cells
    /// assert_eq!(source.grid().count(), 256 * 256);
    /// assert_eq!(source.get(cell), Some("1".to_owned()));
    /// ```
    ///
    pub fn with_source(source: Source) -> Self {
        Self {
            inner: Arc::new(RwLock::new(source)),
        }
    }

    ///
    /// Hands the current revision to `read` as the Source root: what
    /// persistence stores and what a Source File is written from.
    ///
    /// The Source stays behind the lock: it is the live state every reader of
    /// this handle shares, not a value to hand out.
    ///
    /// ```
    /// use orcvs::grid::Grid;
    /// use orcvs::source::SourceCommander;
    ///
    /// let source = SourceCommander::new(Grid::new());
    /// let cell = source.grid().cell_index(0).expect("inside the Grid");
    /// source.set(cell, "1").expect("a Cell the Source accepts");
    ///
    /// // the Source is read where it lives; what the reader takes from it is
    /// // its own to keep
    /// let mut stored = String::new();
    /// source.read_source(|source| stored = source.snapshot());
    ///
    /// assert_eq!(&stored[..1], "1");
    /// ```
    ///
    pub fn read_source(&self, read: impl FnOnce(&Source)) {
        read(&read_recover(&self.inner));
    }

    ///
    /// The identity of the revision the Source is now at
    /// ([`Source::revision`]), without copying a Cell: cheap enough to ask
    /// every frame.
    ///
    pub fn revision(&self) -> RevisionId {
        read_recover(&self.inner).revision()
    }

    ///
    /// The shape this Source was built from, and so the only Grid that can
    /// mint an index addressing one of its Cells.
    ///
    /// A caller editing the Source needs it: a Cell is named by an index, and
    /// only this Grid mints one. `read_revision` also answers, but copies a
    /// whole revision to do it.
    ///
    pub fn grid(&self) -> Grid {
        read_recover(&self.inner).grid()
    }

    ///
    /// Synchronous edit: when this returns, every observable part of the
    /// Source describes the new revision.
    ///
    pub fn set(&self, cell: CellIndex, s: &str) -> Result<(), SourceError> {
        write_recover(&self.inner).set(cell, s)
    }

    ///
    /// Synchronous block edit: every Cell of `writes` lands in one revision,
    /// so no reader of this Source — a Tick among them — observes part of it.
    ///
    pub fn write_cells(&self, writes: &[CellWrite]) {
        write_recover(&self.inner).write_cells(writes);
    }

    ///
    /// Synchronous delete: when this returns, every observable part of the
    /// Source describes the new revision.
    ///
    pub fn unset(&self, cell: CellIndex) {
        write_recover(&self.inner).unset(cell);
    }

    /// What `cell` holds at the current revision, or `None` when it is empty.
    pub fn get(&self, cell: CellIndex) -> Option<String> {
        read_recover(&self.inner).get(cell)
    }

    ///
    /// The full grid contents, read consistently at one revision.
    ///
    pub fn snapshot(&self) -> String {
        read_recover(&self.inner).snapshot()
    }

    /// Every character Cell and its Language Map from one Source revision.
    pub fn read_revision(&self) -> SourceRevision {
        let source = read_recover(&self.inner);
        SourceRevision {
            grid: source.grid(),
            source: source.snapshot(),
            language_map: source.shared_language_map(),
        }
    }

    ///
    /// Runs one Tick against the current Source revision at absolute Tick
    /// `tick`.
    ///
    /// The Tick is supplied rather than counted here: the Playback Engine owns
    /// musical time, and a counter living beside the Source would be language
    /// state outside the Source Snapshot.
    ///
    pub(crate) fn execute(&self, tick: Tick) -> TickPlan {
        write_recover(&self.inner).execute(tick)
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceCommander, SourceError, Tick, Token};
    use crate::grid::Grid;

    #[test]
    fn source_access_recovers_after_the_lock_is_poisoned() {
        let grid = Grid::with_shape(2, 1);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        let poisoned = source.clone();

        assert!(
            std::thread::spawn(move || {
                let _guard = poisoned.inner.write().unwrap();
                panic!("poison the Source lock");
            })
            .join()
            .is_err()
        );

        assert_eq!(source.snapshot(), "  ");
        source.set(cell(0), "x").unwrap();
        assert_eq!(source.get(cell(0)).as_deref(), Some("x"));
    }

    #[test]
    fn the_commander_answers_the_revision_its_source_is_at() {
        let grid = Grid::with_shape(2, 1);
        let source = SourceCommander::new(grid);
        let before = source.revision();
        source.set(grid.cell_index(0).unwrap(), "x").unwrap();
        let mut held = None;
        source.read_source(|source| held = Some(source.revision()));
        assert_ne!(source.revision(), before);
        assert_eq!(Some(source.revision()), held);
    }

    #[test]
    fn tick_writes_into_a_long_expression() {
        let grid = Grid::with_shape(100, 3);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        // Row 1 holds the chain, which claims Cells 100 to 161.
        for (offset, content) in ".+".repeat(15).chars().enumerate() {
            source
                .set(cell(100 + offset), &content.to_string())
                .unwrap();
        }
        // One producer writes inside that claim and one outside it.
        for (offset, content) in ".+0102".chars().enumerate() {
            source.set(cell(30 + offset), &content.to_string()).unwrap();
            source.set(cell(70 + offset), &content.to_string()).unwrap();
        }

        source.execute(Tick::ZERO);

        assert_eq!(source.get(cell(130)), Some("0".to_string()));
        assert_eq!(source.get(cell(131)), Some("3".to_string()));
        assert_eq!(source.get(cell(170)), Some("0".to_string()));
        assert_eq!(source.get(cell(171)), Some("3".to_string()));

        source.execute(Tick::ZERO);
        source.set(cell(199), "x").unwrap();

        assert_eq!(source.get(cell(199)), Some("x".to_string()));
        assert_eq!(source.get(cell(170)), Some("0".to_string()));
        assert_eq!(source.get(cell(171)), Some("3".to_string()));
    }

    #[test]
    fn setting_clearing_and_reading_a_cell_all_take_a_grid_minted_index() {
        // The whole editing seam in one place, now that it has one shape.
        // Addressing is settled before the Source is asked anything, so the
        // only rules left are about content.
        let grid = Grid::with_shape(4, 2);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");

        source.set(cell(0), ".").unwrap();
        source.set(cell(1), "+").unwrap();

        assert_eq!(source.get(cell(0)).as_deref(), Some("."));
        assert_eq!(source.get(cell(1)).as_deref(), Some("+"));

        // A Cell it cannot store is still refused, and refusing it leaves the
        // Cell as it was. That rule is about content, and it is a separate rule
        // that keeps its own error.
        assert_eq!(
            source.set(cell(2), "ab"),
            Err(SourceError::InvalidCell {
                content: "ab".to_string()
            })
        );
        assert_eq!(source.get(cell(2)), None);

        source.unset(cell(0));

        assert_eq!(source.get(cell(0)), None);
        assert_eq!(source.get(cell(1)).as_deref(), Some("+"));
    }

    #[test]
    #[should_panic(expected = "CellIndex belongs to another Grid")]
    fn the_editing_seam_refuses_an_index_minted_by_another_grid() {
        // What the seam has instead of an out-of-range error: a Cell this
        // Source does not have cannot be presented to it at all, and an index
        // from a Grid of the same shape is still not one of this Source's.
        let source = SourceCommander::new(Grid::with_shape(4, 2));
        let foreign = Grid::with_shape(4, 2)
            .cell_index(0)
            .expect("inside the Grid");

        let _ = source.set(foreign, "x");
    }

    #[test]
    fn coherent_read_pairs_every_cell_with_its_source_derived_token() {
        let grid = Grid::with_shape(4, 2);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        source.set(cell(0), ".").unwrap();
        source.set(cell(1), "+").unwrap();

        let read = source.read_revision();

        assert_eq!(read.grid(), grid);
        assert_eq!(read.content_at(grid.position(0, 0).unwrap()), Some('.'));
        assert_eq!(
            read.token_at(grid.position(0, 0).unwrap()),
            Some(Token::Function)
        );
        assert_eq!(read.content_at(grid.position(1, 0).unwrap()), Some('+'));
        assert!(
            grid.positions_by_row()
                .flatten()
                .skip(2)
                .all(|position| read.content_at(position).is_none())
        );
    }

    #[test]
    fn unchanged_revision_reads_share_the_language_map() {
        let grid = Grid::with_shape(4, 2);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        source.set(cell(0), ".").unwrap();
        source.set(cell(1), "+").unwrap();

        let first = source.read_revision();
        let second = source.read_revision();

        assert!(std::ptr::eq(first.language_map(), second.language_map()));
    }

    #[test]
    fn token_at_answers_none_when_the_cell_is_empty_and_unclaimed() {
        let grid = Grid::with_shape(16, 1);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        for (index, content) in ".+0102  .-0304  ".chars().enumerate() {
            source.set(cell(index), &content.to_string()).unwrap();
        }

        let read = source.read_revision();
        let claimed = grid.position(0, 0).unwrap();
        let gap = grid.position(6, 0).unwrap();

        assert_eq!(read.token_at(claimed), Some(Token::Function));
        assert_eq!(read.language_map().token_at(claimed), Some(Token::Function));
        assert_eq!(read.content_at(gap), None);
        assert_eq!(read.language_map().token_at(gap), None);
        assert_eq!(read.token_at(gap), None);
    }

    ///
    /// `.scratch/syntax-highlighting/issues/12`: the Output Portal highlight
    /// fitted to the answer a Sequence-capable root holds.
    ///
    /// Every case is written straight into Source with no Tick, because the
    /// highlight reads the current revision alone (`05`'s Answer). The
    /// Reservation these are fitted inside stays whole, and
    /// `LanguageMap::output_portal_cells`'s own tests and `tick.rs`'s
    /// agreement test still pin it.
    ///
    mod output_portal_highlight {
        use super::{Grid, SourceCommander};
        use crate::source::SourceRevision;

        /// One revision written from `rows`, each padded to the Grid's width.
        fn revision(grid: Grid, rows: &[&str]) -> SourceRevision {
            let source = SourceCommander::new(grid);
            assert!(rows.len() <= grid.rows(), "more rows than the Grid holds");
            for (y, row) in rows.iter().enumerate() {
                assert!(
                    row.len() <= grid.columns(),
                    "{row:?} does not fit {} columns",
                    grid.columns()
                );
                for (x, character) in row.chars().enumerate() {
                    if character == ' ' {
                        continue;
                    }
                    let position = grid.position(x, y).expect("inside the Grid");
                    source
                        .set(grid.index(position), &character.to_string())
                        .expect("a Cell the Source accepts");
                }
            }
            source.read_revision()
        }

        /// Row `y` of the highlight, `#` per covered Cell and `.` per bare
        /// one, so a failure reads as the row it draws.
        fn row(revision: &SourceRevision, grid: Grid, y: usize) -> String {
            let covered = revision.output_portal_highlight();
            (0..grid.columns())
                .map(|x| {
                    let position = grid.position(x, y).expect("inside the Grid");
                    if covered[grid.index(position).get()] {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect()
        }

        #[test]
        fn a_sequence_answer_of_four_cells_or_more_is_covered_exactly() {
            // `12`'s four worked examples, each answer written south of the
            // root that would produce it.
            let wide = Grid::with_shape(10, 2);
            let range = revision(wide, &[":-0104", "01020304"]);
            assert_eq!(row(&range, wide, 1), "########..");

            let notes = revision(wide, &[":#C4D4", "C4c4D4"]);
            assert_eq!(row(&notes, wide, 1), "######....");

            let reversed = revision(wide, &[":<:-0104", "04030201"]);
            assert_eq!(row(&reversed, wide, 1), "########..");

            let widest = Grid::with_shape(16, 2);
            let concatenated = revision(widest, &[":&.+0001:-0203", "010203"]);
            assert_eq!(row(&concatenated, widest, 1), "######..........");
        }

        #[test]
        fn an_empty_sequence_capable_output_portal_shows_four_cells() {
            // Before any Tick, with nothing written south of it at all: the
            // four-Cell minimum is what tells a Sequence-capable root from a
            // scalar one on sight.
            let grid = Grid::with_shape(10, 2);
            let empty = revision(grid, &[":-0104"]);

            assert_eq!(row(&empty, grid, 1), "####......");
        }

        #[test]
        fn a_one_atom_answer_shows_four_cells() {
            // The answer is narrower than the minimum, and the minimum wins:
            // the two Cells past it are covered although they are blank.
            let grid = Grid::with_shape(10, 2);
            let one_atom = revision(grid, &[":-0101", "01"]);

            assert_eq!(row(&one_atom, grid, 1), "####......");
        }

        #[test]
        fn the_highlight_stops_at_the_first_blank_cell_past_the_minimum() {
            // Six written Cells, a blank pair, then four more written Cells
            // the highlight never reaches: the first blank Cell ends it,
            // whatever lies beyond.
            let grid = Grid::with_shape(12, 2);
            let gapped = revision(grid, &[":-0104", "010203  0405"]);

            assert_eq!(row(&gapped, grid, 1), "######......");
        }

        #[test]
        fn the_pair_the_written_run_stops_inside_is_covered_whole() {
            // The run reaches column 4 and ends there. Stopping mid-pair
            // would draw column 4's glyph outside the highlight that covers
            // the Atom it belongs to, so the pair it stopped inside is taken
            // whole.
            let grid = Grid::with_shape(10, 2);
            let half = revision(grid, &[":-0104", "01020"]);

            assert_eq!(row(&half, grid, 1), "######....");
        }

        #[test]
        fn a_blank_gutter_cell_stops_the_highlight_whatever_follows_it() {
            // A neighbouring column's Expression is not this root's answer,
            // however narrow the gutter between them. One blank Cell is
            // enough to end the answer, even where completing the pair the
            // run stopped inside covers the gutter's own first Cell.
            let grid = Grid::with_shape(20, 2);

            // A one-column gutter: `.` at column 9 is the right column's
            // Expression, and the blank at column 8 ends the answer.
            let narrow = revision(grid, &[":-0104", "01020304 .+0304"]);
            assert_eq!(row(&narrow, grid, 1), "########............");

            // A two-column gutter an odd written run reaches into: the run
            // ends at column 9, the pair it stopped in is completed, and the
            // right column's Expression at column 11 is left alone.
            let straddled = revision(grid, &[":-0104", "010203040  .+0304"]);
            assert_eq!(row(&straddled, grid, 1), "##########..........");
        }

        #[test]
        fn the_highlight_is_clipped_to_a_reservation_the_row_edge_cuts_short() {
            // `:-` with no operands is still a Function candidate, so it
            // reserves. Anchored at column 5 of an 8-wide row, its Reservation
            // is the three Cells to the row's end, and the four-Cell minimum
            // does not reach past them.
            let grid = Grid::with_shape(8, 2);
            let clipped = revision(grid, &["     :-", "     01"]);

            assert_eq!(row(&clipped, grid, 1), ".....###");
        }

        #[test]
        fn a_scalar_root_keeps_its_cell_pair() {
            // Neither widened to the minimum nor extended by the written
            // Cells that follow: the fit applies to Sequence-capable roots
            // alone.
            let grid = Grid::with_shape(10, 2);
            let scalar = revision(grid, &[".+0304", "07"]);
            assert_eq!(row(&scalar, grid, 1), "##........");

            let trailed = revision(grid, &[".+0304", "0708090A"]);
            assert_eq!(row(&trailed, grid, 1), "##........");
        }
    }
}
