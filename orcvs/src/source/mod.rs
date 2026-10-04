mod buffer;
mod cell;
pub use cell::CellContent;
mod encoding;
pub mod error;
pub mod file;
mod language_map;
pub use lang::Token;
// `Claim::atom`'s type: re-exported so a caller naming a `Claim` value — the
// console's colour tests among them — can spell `Atom` without adding a direct
// dependency on `lang`.
pub use lang::Atom;
pub use language_map::{Claim, ExpressionEntry, LanguageMap, LanguageUnit, LanguageUnitKind, Span};
mod model;
mod planning;
mod portal;
mod tick;
use crate::grid::{CellIndex, Grid, Position};
use buffer::{Cells, SourceBuffer};
pub use error::SourceError;
pub use lang::Tick;
pub use model::{
    BendLsb, BendMsb, CellWrite, ControlValue, Controller, Diagnostic, Length, MidiChannel, Note,
    PlayCommand, RevisionId, Source, TickPlan, Velocity,
};
use planning::{PlannedTick, PlanningSnapshot, StalePlan, TickCommit};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

///
/// How many times [`SourceCommander::execute`] plans a Tick with no Source lock
/// before it plans one under the write lock.
///
/// Each refused attempt means an edit landed inside one planning window, which
/// lasts a fraction of the Tick period, so a second refusal in a row is already
/// an editor writing faster than a Tick plans. The bound is what ends that: the
/// locked attempt cannot be refused, so a Tick always commits, and an editor it
/// holds off waits for one planning and commit.
///
const OPTIMISTIC_TICK_ATTEMPTS: usize = 3;

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
    /// A List Item: Source inside a List Function's claim that the Parser
    /// never decodes, whatever its characters spell.
    Item,
    Operand {
        token: Token,
        state: OperandState,
    },
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
///
/// The Cells are shared with the Source that revision was read from, not
/// copied, and a later write to that Source leaves them as they are.
#[derive(Clone)]
pub struct SourceRevision {
    grid: Grid,
    cells: SourceBuffer,
    language_map: Arc<LanguageMap>,
}

impl SourceRevision {
    pub fn grid(&self) -> Grid {
        self.grid
    }

    pub fn content_at(&self, position: Position) -> Option<char> {
        self.grid.assert_owns(position);
        let byte = self.cells.bytes()[self.grid.index(position).get()];
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
        self.language_map.claims_by_cell(self.cells.cells())
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
    /// Portal, in the Grid's row-major order: the Cell pair each root's
    /// Reservation covers, which [`LanguageMap::output_portal_reservations`]
    /// answers.
    ///
    pub(crate) fn output_portal_highlight(&self) -> Vec<bool> {
        let mut covered = vec![false; self.grid.count()];
        for reservation in self.language_map.output_portal_reservations() {
            for index in reservation.range {
                covered[index] = true;
            }
        }
        covered
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
    /// The identity of the revision the Source is at ([`Source::revision`]),
    /// read without copying a Cell.
    ///
    pub fn revision(&self) -> RevisionId {
        read_recover(&self.inner).revision()
    }

    ///
    /// The shape this Source was built from, and so the only Grid that can
    /// mint an index addressing one of its Cells. `read_revision` also
    /// answers, but holds on to a whole revision to do it.
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
            cells: source.shared_cells(),
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
    /// Planning holds no Source lock, so an edit and a revision read are
    /// never kept waiting on it. Only the commit takes the write lock, and it
    /// commits only when the Source is still at the revision the plan was made
    /// from: an edit made while planning is never overwritten by Cells derived
    /// without it. A refused plan is dropped unpublished, and the same Tick is
    /// planned again from the revision holding the edit, so a refusal neither
    /// consumes a musical Tick nor repeats an effect.
    ///
    /// After `OPTIMISTIC_TICK_ATTEMPTS` refusals the Tick is planned and
    /// committed under one write guard, which nothing can refuse.
    ///
    pub fn execute(&self, tick: Tick) -> TickPlan {
        TickCommit::new(self, tick).run()
    }

    ///
    /// The current revision's planning inputs. The read guard is released
    /// when this returns, before anything is planned from them.
    ///
    fn planning_snapshot(&self) -> PlanningSnapshot {
        PlanningSnapshot::capture(&read_recover(&self.inner))
    }

    /// Validates `planned` against the current revision and commits it, under
    /// one write guard.
    fn commit_planned(&self, planned: PlannedTick) -> Result<TickPlan, StalePlan> {
        planned.commit(&mut write_recover(&self.inner))
    }

    /// Plans and commits the Tick under one write guard.
    fn execute_locked(&self, tick: Tick) -> TickPlan {
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
        // The whole editing seam in one place. Addressing is settled before
        // the Source is asked anything, so the only rules left are about
        // content.
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
        // The seam has no out-of-range error: a Cell this Source does not
        // have cannot be presented to it at all, and an index from a Grid of
        // the same shape is still not one of this Source's.
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
    fn a_revision_read_shares_the_cells_and_keeps_them_through_a_later_write() {
        let grid = Grid::with_shape(4, 2);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        source.set(cell(0), ".").unwrap();

        let read = source.read_revision();
        let held = source.inner.read().unwrap().shared_cells();
        assert_eq!(read.cells.bytes().as_ptr(), held.bytes().as_ptr());
        drop(held);

        source.set(cell(0), "x").unwrap();

        assert_eq!(read.content_at(grid.position(0, 0).unwrap()), Some('.'));
        assert_eq!(source.get(cell(0)).as_deref(), Some("x"));
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
    /// The Output Portal highlight, written straight into Source with no Tick
    /// because the highlight reads the current revision alone.
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
        fn a_root_keeps_its_cell_pair() {
            // Not extended by the written Cells that follow: every root
            // reserves one Atom's Cell pair.
            let grid = Grid::with_shape(10, 2);
            let scalar = revision(grid, &[".+0304", "07"]);
            assert_eq!(row(&scalar, grid, 1), "##........");

            let trailed = revision(grid, &[".+0304", "0708090A"]);
            assert_eq!(row(&trailed, grid, 1), "##........");
        }

        #[test]
        fn a_retired_colon_spelling_highlights_nothing() {
            let grid = Grid::with_shape(10, 2);
            let retired = revision(grid, &[":-0104", "01020304"]);
            assert_eq!(row(&retired, grid, 1), "..........");
        }
    }
}
