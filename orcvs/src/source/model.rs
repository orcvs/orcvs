use lang::Tick;
use std::{fmt, sync::Arc};
use tracing::debug;

use crate::grid::{CellIndex, Grid};

use std::collections::BTreeSet;

use super::language_map::{LanguageMap, Span};
use super::tick;
use super::{CellContent, SourceError};

pub const SPACE: &str = " ";
const SPACE_BYTE: u8 = b' ';

///
/// A problem with the Expression occupying `start..=end` in the current
/// Source revision. Diagnostics describe accepted user content; they never
/// reject an incomplete or invalid Live Edit.
///
/// Every Diagnostic has both an anchor and a Span, and they answer two
/// different questions. The Span is the run of Cells the problem is about. The
/// anchor is the one Position responsible for it — where a reader is sent to
/// fix it. The anchor lies within the Span but need not be its first Cell: a
/// producer whose effect falls elsewhere than where it sits diagnoses at its
/// own Position while describing the Cells its Expression occupies.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub message: String,
    anchor: crate::grid::Position,
    span: Span,
}

impl Diagnostic {
    /// A Diagnostic about a run of Cells, anchored at the first of them.
    ///
    /// This is the shape for a problem no single Position is more responsible
    /// for than any other: an unmatched character, or a parse of a whole Span.
    pub(super) fn for_range(
        grid: Grid,
        start: crate::grid::CellIndex,
        end: crate::grid::CellIndex,
        message: String,
    ) -> Self {
        Self {
            message,
            anchor: grid.position_at(start),
            span: Span::new(grid, start, end),
        }
    }

    /// A Diagnostic about the Expression covering `span`, anchored at the
    /// Position that produced it.
    ///
    /// The anchor is supplied rather than derived because a producer's own
    /// Position is not always its Expression's first Cell. It still has to be
    /// one of the Cells the Diagnostic describes, which is the half of the
    /// relationship a supplied anchor can get wrong.
    pub(super) fn for_expression(
        anchor: crate::grid::Position,
        span: Span,
        message: String,
    ) -> Self {
        debug_assert!(
            span.positions().any(|position| position == anchor),
            "a Diagnostic's anchor is one of the Cells it describes"
        );
        Self {
            message,
            anchor,
            span,
        }
    }

    /// The first Cell this Diagnostic covers, as a Source index.
    pub fn start(&self) -> usize {
        self.span.start().get()
    }

    /// The last Cell this Diagnostic covers, as a Source index. Inclusive.
    pub fn end(&self) -> usize {
        self.span.end().get()
    }

    /// The Position this Diagnostic sends a reader to. Always inside `span()`.
    pub fn anchor(&self) -> crate::grid::Position {
        self.anchor
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

/// One Cell a Tick Plan commits, and what it receives.
///
/// The Cell is named by the index its Grid minted rather than by a number,
/// so a planned write cannot address a Cell the Source has no room for and
/// nothing downstream re-checks the bound.
#[derive(Clone, Debug, PartialEq)]
pub struct CellWrite {
    pub cell: CellIndex,
    pub content: CellContent,
}

/// One interpreted MIDI instruction emitted by an active Terminal Output
/// Function, and the ordered group of them one Expression performs. Tick
/// planning decides which terminal roots are active and in what order their
/// commands appear; the output adapter turns each one into MIDI. Per ADR 0030
/// one Expression can perform many commands, ordered by element index, so a
/// Performance crosses the seam and a Tick Plan holds the flattened list.
pub use lang::{
    BendLsb, BendMsb, ControlValue, Controller, Length, MidiChannel, Note, Performance,
    PlayCommand, Velocity,
};

#[derive(Clone, Debug, PartialEq)]
pub struct TickPlan {
    pub writes: Vec<CellWrite>,
    pub play_commands: Vec<PlayCommand>,
    pub diagnostics: Vec<Diagnostic>,
}

///
/// The Cells of one Orca program. The Source is the contents; the Grid it is
/// built from is the shape, and answers every question about that shape.
///
pub struct Source {
    grid: Grid,
    inner: String,
    language_map: Arc<LanguageMap>,
}

#[cfg(feature = "persistence")]
#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedSource {
    grid: Grid,
    inner: String,
}

#[cfg(feature = "persistence")]
impl serde::Serialize for Source {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(
            &PersistedSource {
                grid: self.grid,
                inner: self.inner.clone(),
            },
            serializer,
        )
    }
}

#[cfg(feature = "persistence")]
impl<'de> serde::Deserialize<'de> for Source {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let persisted = <PersistedSource as serde::Deserialize>::deserialize(deserializer)?;
        if persisted.inner.len() != persisted.grid.count() {
            return Err(D::Error::custom(
                "persisted Source Cell count does not match its Grid",
            ));
        }
        if !persisted
            .inner
            .bytes()
            .all(|byte| CellContent::new(byte).is_some())
        {
            return Err(D::Error::custom(
                "persisted Source contains a non-Cell character",
            ));
        }

        let mut source = Source::new(persisted.grid);
        source.inner = persisted.inner;
        // A Source read back from persistence has no previous revision to
        // carry rows over from, so every row is parsed.
        source.language_map = Arc::new(LanguageMap::build(source.grid, source.inner.as_bytes()));
        Ok(source)
    }
}

impl Source {
    pub fn grid(&self) -> Grid {
        self.grid
    }

    ///
    /// A Source of empty Cells, one per Position of `grid`. A Grid has at
    /// least one column and one row, so a Source always has Cells.
    ///
    pub fn new(grid: Grid) -> Self {
        let size = grid.count();
        let inner = SPACE.to_string().repeat(size);
        let language_map = Arc::new(LanguageMap::build(grid, inner.as_bytes()));

        Self {
            grid,
            inner,
            language_map,
        }
    }

    ///
    /// Sets `cell` and recalculates the affected Expressions. A space empties
    /// the Cell, equivalent to `unset`.
    ///
    /// The Cell is named by an index its Grid minted, so there is no index to
    /// refuse: this weighs the content and the Expression the edit would make,
    /// and nothing else.
    ///
    /// ```
    /// use orcvs::{grid::Grid, source::Source};
    ///
    /// let grid = Grid::new(10, 10);
    /// let mut source = Source::new(grid);
    /// let cell = grid.cell_index(33).expect("inside the Grid");
    /// source.set(cell, "!").unwrap();
    ///
    /// assert_eq!(source.get(cell), Some("!".to_string()));
    /// ```
    ///
    pub fn set(&mut self, cell: CellIndex, s: &str) -> Result<(), SourceError> {
        self.grid.assert_owns_index(cell);
        debug!("set {}: {s}", cell.get());
        let byte = Self::check_content(s)?;

        self.edit(cell, byte);
        Ok(())
    }

    ///
    /// Empties `cell` and recalculates the affected Expressions.
    ///
    /// Nothing can refuse this. A Cell the Grid minted exists, and emptying a
    /// Cell can only shorten an Expression.
    ///
    pub fn unset(&mut self, cell: CellIndex) {
        self.grid.assert_owns_index(cell);
        self.edit(cell, CellContent::SPACE);
    }

    ///
    /// Applies one already-validated edit. The revision it produces is what
    /// the console observes, so the edit reports nothing of its own.
    ///
    fn edit(&mut self, cell: CellIndex, byte: CellContent) {
        self.set_source(cell, byte);
        // One Cell changes one row, and a row is the largest thing a Cell can
        // change: a run never crosses the row edge.
        self.rebuild_rows(&BTreeSet::from([self.grid.position_at(cell).y()]));
    }

    ///
    /// The full grid contents at the current revision.
    ///
    pub fn snapshot(&self) -> String {
        self.inner.clone()
    }

    /// The semantic view derived from this exact Source revision.
    pub fn language_map(&self) -> &LanguageMap {
        &self.language_map
    }

    pub(super) fn shared_language_map(&self) -> Arc<LanguageMap> {
        Arc::clone(&self.language_map)
    }

    ///
    /// The one byte `s` holds, when `s` is one printable single-byte ASCII
    /// character. This is the only rule the editing seam has left: addressing
    /// is settled by the index, so content is all a Cell can be refused for.
    ///
    fn check_content(s: &str) -> Result<CellContent, SourceError> {
        let content = match s.as_bytes() {
            [byte] => CellContent::new(*byte),
            _ => None,
        };
        content.ok_or_else(|| SourceError::InvalidCell {
            content: s.to_string(),
        })
    }

    ///
    /// What `cell` holds, or `None` when it is empty.
    ///
    /// Total for the Cells this Source has: a Grid-minted index addresses one
    /// of them, so the only `None` here is an empty Cell.
    ///
    pub fn get(&self, cell: CellIndex) -> Option<String> {
        self.grid.assert_owns_index(cell);

        match self.inner.as_bytes()[cell.get()] {
            SPACE_BYTE => None,
            byte => Some((byte as char).to_string()),
        }
    }

    /// Schedules and runs one Tick, then atomically commits the final writes.
    /// Function outputs are visible to their current-Tick dependents; MIDI
    /// commands carry the operands those scheduled evaluations observed.
    /// `tick` is the absolute musical Tick supplied by the Playback Engine.
    pub fn execute(&mut self, tick: Tick) -> TickPlan {
        let plan = self.plan_tick(tick);
        self.commit_tick(&plan);
        plan
    }

    #[cfg(test)]
    pub(super) fn execute_configured(
        &mut self,
        tick: Tick,
        configuration: &super::tick::Configuration,
    ) -> TickPlan {
        let plan = super::tick::plan_configured(
            self.grid,
            self.inner.as_bytes(),
            &self.language_map,
            tick,
            configuration,
        );
        self.commit_tick(&plan);
        plan
    }

    fn plan_tick(&self, tick: Tick) -> TickPlan {
        tick::plan(self.grid, self.inner.as_bytes(), &self.language_map, tick)
    }

    /// Visible to the Tick module so a test that plans a Tick without going
    /// through [`Source::execute`] still commits it the one way a Tick is
    /// committed: every planned Cell first, then one rebuild of the rows they
    /// touched.
    pub(in crate::source) fn commit_tick(&mut self, plan: &TickPlan) {
        // Commit every planned Cell before rebuilding any derived state.
        let mut written = BTreeSet::new();
        for write in &plan.writes {
            self.set_source(write.cell, write.content);
            written.insert(self.grid.position_at(write.cell).y());
        }
        self.rebuild_rows(&written);
    }

    ///
    /// Rebuilds the Language Map, parsing only the rows that were written to.
    ///
    /// The rows are the ones this revision wrote, not the ones whose bytes
    /// differ from the last revision. A Tick that writes a Cell the same value
    /// it already held has still written it, and a Map that skipped the row on
    /// that ground would be reporting on how little a fixture changes rather
    /// than on how much work a revision costs.
    ///
    fn rebuild_rows(&mut self, written: &BTreeSet<usize>) {
        self.language_map = Arc::new(LanguageMap::rebuild(
            &self.language_map,
            self.grid,
            self.inner.as_bytes(),
            written,
        ));
    }

    ///
    /// Writes one already-validated ASCII byte at `cell` without
    /// recalculating Expressions.
    ///
    fn set_source(&mut self, cell: CellIndex, content: CellContent) {
        // What makes `cell` address a byte of *this* Source. `Grid::cell_index`
        // and `Grid::index` are the only minters and both bound their answer by
        // the Grid's Cell count; `inner` is that many bytes from `Source::new`
        // onward, and nothing below changes its length. A Grid of the same
        // shape is still a different Grid, which is why identity is what is
        // asked rather than a number compared.
        self.grid.assert_owns_index(cell);
        // SAFETY: Source construction and deserialization establish one
        // printable ASCII byte per Cell. Every subsequent write takes a
        // CellContent, whose private byte is printable ASCII by construction.
        // Replacing one such byte preserves UTF-8 validity and String length.
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            bytes[cell.get()] = content.byte();
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod test {

    use lang::{Atom, Function, Interpretation, Sequence, Value};
    use std::ops::{Deref, DerefMut};

    use crate::{
        glyph::Glyph,
        grid::{CellIndex, Grid, Position},
        source::{
            BendLsb, BendMsb, CellWrite, ControlValue, Controller, Length, MidiChannel, Note,
            PlayCommand, Source, SourceError, Tick, TickPlan, Velocity,
            encoding::{Encoding, Rendered},
            portal::Portal,
            tick::{Effect, resolve},
        },
        test::trace,
    };

    #[test]
    fn editing_and_derivation_agree_on_every_byte_character() {
        let grid = Grid::new(1, 1);
        let cell = grid.cell_index(0).unwrap();
        let mut source = Source::new(grid);
        for byte in u8::MIN..=u8::MAX {
            let input = char::from(byte).to_string();
            let accepted = byte == b' ' || byte.is_ascii_graphic();
            let before = source.snapshot();
            assert_eq!(source.set(cell, &input).is_ok(), accepted);
            assert_eq!(
                crate::source::LanguageMap::derive(grid, &input).is_some(),
                accepted,
            );
            assert_eq!(source.snapshot(), if accepted { input } else { before });
        }
    }

    #[test]
    fn every_printable_character_survives_portal_commit() {
        let encoding: String = (u8::MIN..=u8::MAX)
            .filter(|byte| *byte == b' ' || byte.is_ascii_graphic())
            .map(char::from)
            .collect();
        let grid = Grid::new(encoding.len(), 2);
        let mut source = Source::new(grid);
        let root = grid.position(0, 0).unwrap();
        source.commit_tick(&plan_result(
            grid,
            root,
            Interpretation::Sequence(Sequence::new(encoding.chars().map(Atom::Char)).unwrap()),
        ));
        assert_eq!(
            source.snapshot(),
            format!("{}{encoding}", " ".repeat(encoding.len()))
        );

        #[cfg(feature = "persistence")]
        {
            let encoded = serde_json::to_string(&source).unwrap();
            let restored: Source = serde_json::from_str(&encoded).unwrap();
            assert_eq!(restored.snapshot(), source.snapshot());
        }
    }

    ///
    /// The default shape for tests in this module. Rectangular on purpose: a
    /// Source that derived a dimension of its own, or read the two the wrong
    /// way round, addresses different Cells here than it does on a square one.
    ///
    fn grid() -> Grid {
        Grid::new(10, 6)
    }

    ///
    /// The Glyph a Cell presents at the current revision, read the way the
    /// console reads it.
    ///
    fn glyph_at(source: &Source, idx: usize) -> Option<Glyph> {
        let cell = source.grid.cell_index(idx)?;
        source.language_map.glyph_at(source.grid.position_at(cell))
    }

    ///
    /// What a Cell presents at the current revision: its content and its Glyph.
    /// An edit is observed by reading the revision it produced, so this is what
    /// the console sees after one.
    ///
    fn cell(source: &Source, idx: usize) -> (Option<char>, Option<Glyph>) {
        let content = source
            .grid
            .cell_index(idx)
            .and_then(|cell| source.get(cell))
            .and_then(|s| s.chars().next())
            .filter(|c| *c != ' ');
        (content, glyph_at(source, idx))
    }

    ///
    /// Every Cell's Glyph at the current revision, in Source order.
    ///
    fn glyphs(source: &Source) -> Vec<Option<Glyph>> {
        (0..source.grid.count())
            .map(|idx| glyph_at(source, idx))
            .collect()
    }

    ///
    /// What the current revision diagnoses, as the Cells each problem covers
    /// and what it says. Two Sources are built on Grids of their own, so their
    /// Diagnostics are never equal as values however alike they are; this is
    /// the part of one that two Sources can share.
    ///
    fn reported(source: &Source) -> Vec<(usize, usize, String)> {
        diagnostics(source)
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.start(),
                    diagnostic.end(),
                    diagnostic.message.clone(),
                )
            })
            .collect()
    }

    fn diagnostics(source: &Source) -> Vec<super::Diagnostic> {
        source
            .language_map
            .expression_diagnostics()
            .cloned()
            .collect()
    }

    ///
    /// A Source under test, together with the Grid it was built from. Every
    /// test obtains its Source here, so the shape a helper reads is always the
    /// shape the Source was built on: this owns the only constructor, mints
    /// the Source from the Grid it keeps, and hands neither out for
    /// replacement. Two different shapes are not expressible, and a Source
    /// built outside it has no row helper to call.
    ///
    /// It also owns the Playback run's absolute Tick, so a test that Ticks
    /// twice describes a Playback run that ADR 0012 admits without restating
    /// the counter. See `execute`.
    ///
    /// It derefs to the Source so a test still speaks to a Source directly.
    ///
    struct SourceUnderTest {
        grid: Grid,
        src: Source,
        tick: Tick,
    }

    impl SourceUnderTest {
        fn new(grid: Grid) -> Self {
            Self {
                grid,
                src: Source::new(grid),
                // ADR 0012: the first Tick of a Playback run is absolute Tick
                // `0`. A Source that has not run yet is a run about to begin.
                tick: Tick::ZERO,
            }
        }

        ///
        /// Runs the next Tick of this Source's Playback run.
        ///
        /// ADR 0012 numbers the first Playback Tick `0` and increments that
        /// counter by one for each Tick after it, so a test that executes
        /// twice is describing Ticks `0` and `1`. Counting here rather than at
        /// every call site is what makes the alternative — a run that executes
        /// two consecutive Ticks at one absolute Tick, which no Playback run
        /// does — unwritable: a test cannot pass a Tick it does not name.
        ///
        /// Inherent, so it wins over the `DerefMut` fall-through to
        /// `Source::execute` and a plain `src.execute()` reaches this counter
        /// rather than an unnumbered Tick.
        ///
        fn execute(&mut self) -> TickPlan {
            self.execute_at(self.tick)
        }

        ///
        /// Runs one Tick at the absolute Tick `tick`, and resumes the run from
        /// the Tick after it.
        ///
        /// For the tests that are *about* a particular absolute Tick — a Tick
        /// Plan pinned as a function of the Snapshot and the Tick together —
        /// rather than about a Playback run's ordinary progress.
        ///
        fn execute_at(&mut self, tick: Tick) -> TickPlan {
            self.tick = tick.next();
            self.src.execute(tick)
        }

        ///
        /// The absolute Tick the next execution interprets at.
        ///
        /// This is the Tick itself, not a second count of it: `execute_at` is
        /// the only writer, and it writes what it just handed to
        /// interpretation.
        ///
        fn tick(&self) -> Tick {
            self.tick
        }

        ///
        /// Types `s` into consecutive Cells starting at `start`, one accepted
        /// edit per Cell, exactly as a user would.
        ///
        fn write(&mut self, start: CellIndex, s: &str) {
            // Typing walks the Source's Cells in order, so each Cell after the
            // first is a number away from `start` and has to be minted again.
            // Refusing a foreign `start` is what keeps that arithmetic from
            // beginning somewhere this Grid never named.
            self.grid.assert_owns_index(start);
            for (offset, c) in s.chars().enumerate() {
                let cell = self.cell_index(start.get() + offset);
                self.src.set(cell, &c.to_string()).unwrap();
            }
        }

        ///
        /// The Cells of row `row`, spaces included. The Grid names the Cells
        /// of a row and where each one sits, so no test restates a width of
        /// its own.
        ///
        fn row(&self, row: usize) -> String {
            let cells: Vec<char> = self.src.snapshot().chars().collect();

            self.grid
                .rows()
                .nth(row)
                .expect("a row of the grid")
                .map(|position| cells[self.grid.index(position).get()])
                .collect()
        }

        ///
        /// How many Cells this Source has, asked of the Grid it was built
        /// from.
        ///
        fn count(&self) -> usize {
            self.grid.count()
        }

        ///
        /// How many rows this Source has, asked of the Grid it was built from.
        ///
        fn row_count(&self) -> usize {
            self.grid.rows().count()
        }

        ///
        /// The index this Source's Grid mints for `idx`, so a test states an
        /// expected planned write in the same terms a Tick Plan carries.
        ///
        fn cell_index(&self, idx: usize) -> CellIndex {
            self.grid.cell_index(idx).expect("inside the Grid")
        }

        ///
        /// A function that mints this Source's Cells, so a test can name a
        /// Cell inside the same call that edits it. It holds a copy of the
        /// Grid rather than a borrow of the Source, which is what lets
        /// `src.set(at(5), ..)` be written at all.
        ///
        fn cells(&self) -> impl Fn(usize) -> CellIndex + use<> {
            let grid = self.grid;
            move |idx| grid.cell_index(idx).expect("inside the Grid")
        }
    }

    impl Deref for SourceUnderTest {
        type Target = Source;

        fn deref(&self) -> &Source {
            &self.src
        }
    }

    impl DerefMut for SourceUnderTest {
        fn deref_mut(&mut self) -> &mut Source {
            &mut self.src
        }
    }

    fn source() -> SourceUnderTest {
        SourceUnderTest::new(grid())
    }

    ///
    /// Supply an evaluation answer at the seam production delivers one
    /// through: encode it, admit the whole encoding through the ordinary
    /// result Portal, and resolve the Effect. No parseable Function answers
    /// with a Sequence yet, so a Sequence is stated here rather than spelled
    /// in Source; what it exercises is the Portal and the commit, both of
    /// which are the same ones a Tick uses. Commit remains Source's.
    ///
    fn plan_result(grid: Grid, root: Position, result: Interpretation) -> TickPlan {
        let value = match result {
            Interpretation::Play(command) => return resolve(vec![Effect::Play(command)]),
            Interpretation::Cell(atom) => Value::from(atom),
            Interpretation::Sequence(sequence) => Value::from(sequence),
        };
        // The rule for what an answer becomes in Cells is production's, called
        // here rather than restated: these tests state an answer because no
        // parseable Function answers with a Sequence yet, and what they
        // exercise is the commit, not a second encoding.
        let rendered = Encoding::render(&value).expect("these answers are stated as Source Cells");
        let Rendered::Cells(encoding) = rendered else {
            return resolve(Vec::new());
        };
        let write = Portal::ordinary_result(grid, root)
            .and_then(|portal| portal.admit(&encoding))
            .expect("these answers are stated to fit their destination");
        resolve(vec![Effect::Write(write)])
    }

    fn numbers(values: &[u8]) -> Interpretation {
        Interpretation::Sequence(Sequence::new(values.iter().copied().map(Atom::Number)).unwrap())
    }

    #[test]
    fn test_row_reads_the_shape_of_the_source_under_test() {
        trace();

        // A Source built on a shape other than this module's default. The row
        // helper must read the Cells of *this* Source, not the ones a Grid it
        // was never built from would name.
        let mut src = SourceUnderTest::new(Grid::new(8, 4));
        let at = src.cells();

        src.write(at(0), ".+0102");
        src.execute();

        assert_eq!(src.row(1), "03      ");
    }

    #[test]
    fn test_the_source_under_test_executes_consecutive_ticks_of_one_playback_run() {
        trace();

        // Every multi-Tick test in this module reads its Tick numbering from
        // the helper rather than stating one, so this is where that numbering
        // is pinned. ADR 0012: the first Playback Tick is absolute Tick `0`,
        // and each Tick after it is one on from the last. A helper that handed
        // interpretation the same Tick twice would describe a Playback run
        // that cannot exist, and would silently stop a test that Ticks twice
        // from exercising a second Tick once a Function reads the Tick.
        let mut src = source();

        assert_eq!(src.tick(), Tick::ZERO, "a run begins at absolute Tick 0");
        for expected in 1..=4 {
            src.execute();
            assert_eq!(
                src.tick(),
                Tick::new(expected),
                "execution {expected} left the run on the wrong absolute Tick"
            );
        }

        // A pinned Tick is a Tick of the same run: `execute_at` names the Tick
        // it interprets at, and the run carries on from the Tick after it.
        src.execute_at(Tick::new(7));

        assert_eq!(src.tick(), Tick::new(8));
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn test_source_round_trip_restores_shape_contents_and_derived_state() {
        let grid = Grid::new(10, 3);
        let mut source = Source::new(grid);
        for (idx, content) in ".+0102".chars().enumerate() {
            source
                .set(
                    grid.cell_index(idx).expect("inside the Grid"),
                    &content.to_string(),
                )
                .unwrap();
        }
        source
            .set(grid.cell_index(15).expect("inside the Grid"), "x")
            .unwrap();

        let encoded = serde_json::to_string(&source).unwrap();
        let mut restored: Source = serde_json::from_str(&encoded).unwrap();

        assert_eq!(restored.snapshot(), source.snapshot());
        assert_eq!(restored.grid.count(), 30);
        assert!(restored.grid.position(9, 2).is_some());
        assert!(restored.grid.position(10, 2).is_none());
        assert_eq!(
            (0..grid.count())
                .map(|idx| glyph_at(&restored, idx))
                .collect::<Vec<_>>(),
            (0..grid.count())
                .map(|idx| glyph_at(&source, idx))
                .collect::<Vec<_>>()
        );

        restored.execute(Tick::ZERO);
        // A restored Source is built from a Grid of its own: persistence
        // carries the shape, not the identity, so the Cells of the Source that
        // was written are not the Cells of the Source that was read back.
        let restored_cell = |idx| {
            restored
                .grid()
                .cell_index(idx)
                .expect("inside the restored Grid")
        };
        assert_eq!(restored.get(restored_cell(10)), Some("0".to_string()));
        assert_eq!(restored.get(restored_cell(11)), Some("3".to_string()));
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn test_source_deserialization_rejects_an_empty_grid() {
        let encoded = r#"{"grid":{"cols":0,"rows":3},"inner":""}"#;

        assert!(serde_json::from_str::<Source>(encoded).is_err());
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn test_source_deserialization_rejects_overflowing_grid_dimensions() {
        let encoded = format!(
            r#"{{"grid":{{"cols":{},"rows":2}},"inner":""}}"#,
            usize::MAX / 2 + 1
        );

        assert!(serde_json::from_str::<Source>(&encoded).is_err());
    }

    #[test]
    fn test_set_rejects_invalid_content_without_mutation() {
        trace();

        let mut src = source();

        let at = src.cells();
        src.set(at(5), "x").unwrap();
        let before = src.snapshot();

        for content in ["", "ab", "é", "\n"] {
            let err = src.set(at(5), content).unwrap_err();
            assert_eq!(
                err,
                SourceError::InvalidCell {
                    content: content.to_string()
                }
            );
            assert_eq!(src.snapshot(), before);
            assert_eq!(src.get(at(5)), Some("x".to_string()));
        }
    }

    #[test]
    fn edits_accept_long_expressions() {
        let mut src = SourceUnderTest::new(Grid::new(140, 1));
        let at = src.cells();
        src.write(at(0), &(".+".repeat(33) + &"01".repeat(34)));
        assert!(diagnostics(&src).is_empty());
    }

    #[test]
    fn test_an_edit_classifies_the_cells_it_affects() {
        trace();

        let mut src = source();

        let at = src.cells();

        // A `.` alone is already read where a Function goes: `". "` is not a
        // spelling the table holds, so it is refused there and classified
        // there, two Cells wide.
        src.set(at(0), ".").unwrap();
        assert_eq!(cell(&src, 0), (Some('.'), Some(Glyph::Function)));

        // completing the `.+` Function reclassifies Cell 0 and marks the four
        // empty operand-slot Cells (two 2-wide Numbers) as Number
        src.set(at(1), "+").unwrap();

        let function = |content: char| (Some(content), Some(Glyph::Function));
        let operand_slot = (None, Some(Glyph::Number));
        assert_eq!(cell(&src, 0), function('.'));
        assert_eq!(cell(&src, 1), function('+'));
        assert_eq!(cell(&src, 2), operand_slot);
        assert_eq!(cell(&src, 3), operand_slot);
        assert_eq!(cell(&src, 4), operand_slot);
        assert_eq!(cell(&src, 5), operand_slot);
    }

    #[test]
    fn test_deleting_half_a_function_clears_the_hints_it_placed() {
        trace();

        let mut src = source();

        let at = src.cells();
        src.set(at(0), ".").unwrap();
        src.set(at(1), "+").unwrap();

        // deleting half the Function leaves a spelling the table does not
        // hold, so Cell 0 is still read where a Function goes and the operand
        // hints it placed are gone
        src.unset(at(1));

        assert_eq!(cell(&src, 0), (Some('.'), Some(Glyph::Function)));
        for idx in 2..=5 {
            assert_eq!(cell(&src, idx), (None, None), "Cell {idx} was not cleared");
        }
    }

    #[test]
    fn test_set_near_grid_end_truncates_operand_hints() {
        trace();

        let mut src = source();

        let at = src.cells();

        // `.+` at the last two Cells wants four more operand-slot glyphs
        // than the Source has room for
        src.set(at(58), ".").unwrap();
        src.set(at(59), "+").unwrap();

        assert_eq!(glyph_at(&src, 58), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 59), Some(Glyph::Function));
        // the Function sits in the last two Cells, so its operand-slot hints
        // have nowhere to go: nothing past the row edge is classified
        assert_eq!(glyph_at(&src, 60), None);
    }

    #[test]
    fn test_editing_an_operand_slot_hint_restores_the_current_glyphs() {
        let mut src = SourceUnderTest::new(Grid::new(10, 1));
        let at = src.cells();
        src.set(at(0), ".").unwrap();
        src.set(at(1), "+").unwrap();
        assert_eq!(glyph_at(&src, 5), Some(Glyph::Number));

        // The Cell is inside the Addition's claim, so writing to it fills part
        // of an operand rather than standing outside the Expression.
        src.set(at(5), "x").unwrap();
        assert_eq!(cell(&src, 5), (Some('x'), Some(Glyph::Number)));

        src.unset(at(5));
        assert_eq!(cell(&src, 5), (None, Some(Glyph::Number)));
    }

    #[test]
    fn test_editing_an_operand_slot_matches_a_source_rebuilt_from_its_snapshot() {
        let grid = Grid::new(10, 2);
        let mut src = SourceUnderTest::new(grid);
        let at = src.cells();
        src.write(at(0), ".+");
        src.write(at(10), ".+0102");
        src.set(at(5), "x").unwrap();

        let mut rebuilt = Source::new(grid);
        for (idx, content) in src.snapshot().chars().enumerate() {
            if content != ' ' {
                let cell = grid.cell_index(idx).expect("inside the Grid");
                rebuilt.set(cell, &content.to_string()).unwrap();
            }
        }

        assert_eq!(rebuilt.snapshot(), src.snapshot());
        assert_eq!(
            (0..grid.count())
                .map(|idx| glyph_at(&rebuilt, idx))
                .collect::<Vec<_>>(),
            (0..grid.count())
                .map(|idx| glyph_at(&src, idx))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_operand_hints_and_invalidation_stop_at_the_row_edge() {
        let mut src = SourceUnderTest::new(Grid::new(10, 2));
        let at = src.cells();
        src.write(at(10), ".+0102");
        src.set(at(8), ".").unwrap();

        src.set(at(9), "+").unwrap();
        assert_eq!(cell(&src, 8), (Some('.'), Some(Glyph::Function)));
        assert_eq!(cell(&src, 9), (Some('+'), Some(Glyph::Function)));
        assert_eq!(glyph_at(&src, 10), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 11), Some(Glyph::Function));

        // The refused `.` owns one Cell. The empty Cell beside it belongs
        // to no Expression, and neither hint reaches the next row.
        src.unset(at(9));
        assert_eq!(cell(&src, 8), (Some('.'), Some(Glyph::Function)));
        assert_eq!(cell(&src, 9), (None, None));
        assert_eq!(glyph_at(&src, 10), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 11), Some(Glyph::Function));
    }

    #[test]
    fn test_set_classifies_expression_immediately() {
        trace();

        let mut src = source();

        let at = src.cells();

        for (i, c) in ".+0101".chars().enumerate() {
            src.set(at(i), &c.to_string()).unwrap();
        }

        let glyphs: Vec<_> = (0..6).map(|i| glyph_at(&src, i)).collect();
        assert_eq!(
            glyphs,
            vec![
                Some(Glyph::Function),
                Some(Glyph::Function),
                Some(Glyph::Number),
                Some(Glyph::Number),
                Some(Glyph::Number),
                Some(Glyph::Number),
            ]
        );
        assert_eq!(src.row(0), ".+0101    ");
    }

    #[test]
    fn test_expressions_do_not_join_across_a_row_edge() {
        trace();

        let mut src = source();

        let at = src.cells();

        src.set(at(9), ".").unwrap();
        src.set(at(10), "+").unwrap();

        // The two Cells are adjacent by index but sit in different rows, so
        // neither reads the other: each is the start of a Function spelling
        // its own row cannot complete, and `.+` is nowhere.
        assert_eq!(cell(&src, 9), (Some('.'), Some(Glyph::Function)));
        assert_eq!(cell(&src, 10), (Some('+'), Some(Glyph::Function)));
        assert!(
            src.language_map()
                .expressions()
                .all(|expression| expression.root().is_none())
        );
    }

    #[test]
    fn test_diagnostics_follow_the_visible_expression_revision() {
        trace();

        let mut src = source();

        let at = src.cells();

        // The half-typed Function remains visible and is diagnosed
        // immediately. Its Span is six Cells rather than four: an Addition
        // claims two operands whether or not anyone has written into them, so
        // the diagnostic covers the Cells the Function is asking for.
        src.write(at(0), ".+01");
        assert_eq!(src.row(0), ".+01      ");
        assert_eq!(diagnostics(&src).len(), 1);
        assert_eq!(diagnostics(&src)[0].start(), 0);
        assert_eq!(diagnostics(&src)[0].end(), 5);

        // Completing it removes the cause and therefore the diagnostic in the
        // same accepted edit.
        src.write(at(4), "02");
        assert!(diagnostics(&src).is_empty());

        // Source after a complete Expression belongs to the next Expression
        // rather than to this one. The Addition stays whole and the `Z` is
        // refused where it stands, one Cell of its own.
        src.set(at(6), "Z").unwrap();
        assert_eq!(diagnostics(&src)[0].message, "unknown function \"Z \"");
        assert_eq!(diagnostics(&src)[0].start(), 6);
        src.unset(at(6));
        assert!(diagnostics(&src).is_empty());

        // Replacing a valid operand with invalid content creates a fresh
        // diagnostic for the current Expression, without rejecting the edit.
        src.set(at(4), "X").unwrap();
        assert_eq!(src.get(at(4)), Some("X".to_string()));
        assert_eq!(diagnostics(&src).len(), 1);
        assert_eq!(
            diagnostics(&src)[0].message,
            "expected a number, found \"X2\""
        );

        // Removing the Expression removes its diagnostic rather than leaving
        // stale state attached to empty Cells.
        for idx in 0..6 {
            src.unset(at(idx));
        }
        assert!(diagnostics(&src).is_empty());
    }

    #[test]
    fn test_retired_id_receives_the_unknown_function_diagnostic() {
        trace();

        let mut src = source();

        let at = src.cells();

        // ADR 0015 retired `id` from the Function vocabulary, so Source
        // containing it no longer parses as a Function and diagnoses like any
        // other unknown spelling.
        src.write(at(0), "id");

        assert_eq!(src.row(0), "id        ");
        // Two, because ADR 0018 resumes one Cell after a refused spelling:
        // `id` is refused at Cell 0, and `d ` is refused at Cell 1. Each is an
        // Expression of one Cell.
        assert_eq!(diagnostics(&src).len(), 2);
        assert_eq!(diagnostics(&src)[0].start(), 0);
        assert_eq!(diagnostics(&src)[0].end(), 0);
        assert_eq!(diagnostics(&src)[0].message, "unknown function \"id\"");
        // Classification is unaffected: an unrecognized spelling standing where
        // a Function is expected keeps the Function Glyph, because a Record
        // that failed to parse reports the Token its position expected. That is
        // the same operand-slot hint the editing tests cover, not a claim that
        // `id` is still a Function.
        assert_eq!(glyph_at(&src, 0), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 1), Some(Glyph::Function));
    }

    #[test]
    fn test_join_discards_stale_expression_state() {
        trace();

        let mut src = source();

        let at = src.cells();

        // `.+0101` starting at Cell 4; a Tick would write its result one row below
        for (i, c) in ".+0101".chars().enumerate() {
            src.set(at(i + 4), &c.to_string()).unwrap();
        }

        // Prepending `.+00` joins everything into one Expression at Cell 0;
        // the old Expression starting at Cell 4 no longer exists.
        src.write(at(0), ".+00");
        src.execute();

        // `.+00.+0101` commits `02` across Cells 10 and 11. The stale Expression
        // at Cell 4 would commit its own `02` over Cells 14 and 15, so the row
        // is asserted whole: only the joined Expression's result may appear.
        assert_eq!(src.row(1), "02        ");
    }

    #[test]
    fn test_split_reclassifies_and_evaluates_fresh_expressions() {
        trace();

        let mut src = source();

        let at = src.cells();

        for (i, c) in "xx.+0101".chars().enumerate() {
            src.set(at(i), &c.to_string()).unwrap();
        }

        // deleting Cell 1 splits off a complete `.+0101` Expression at Cell 2
        src.unset(at(1));

        // The refused `x` owns only Cell 0; the empty Cell has no hint.
        assert_eq!(glyph_at(&src, 1), None);
        let glyphs: Vec<_> = (2..8).map(|i| glyph_at(&src, i)).collect();
        assert_eq!(
            glyphs,
            vec![
                Some(Glyph::Function),
                Some(Glyph::Function),
                Some(Glyph::Number),
                Some(Glyph::Number),
                Some(Glyph::Number),
                Some(Glyph::Number),
            ]
        );

        // the split-off Expression evaluates on the next Tick and commits its
        // whole two-Cell result; the lone `x` left at Cell 0 is a literal and
        // commits nothing
        src.execute();
        assert_eq!(src.row(1), "  02      ");
    }

    #[test]
    fn test_set_space_empties_cell() {
        trace();

        let mut src = source();

        let at = src.cells();
        src.set(at(5), "x").unwrap();

        src.set(at(5), " ").unwrap();

        assert_eq!(src.get(at(5)), None);
        assert_eq!(src.snapshot(), " ".repeat(src.count()));
    }

    #[test]
    fn test_result_commits_its_complete_encoding_across_consecutive_cells() {
        trace();

        let mut src = source();

        let at = src.cells();

        // The README example: `.+0102` is 1 + 2, and a Number is two Cells wide
        src.write(at(0), ".+0102");

        let tick = src.execute();

        assert_eq!(src.row(1), "03        ");
        assert_eq!(src.get(at(10)), Some("0".to_string()));
        assert_eq!(src.get(at(11)), Some("3".to_string()));
        assert!(tick.play_commands.is_empty());
        assert_eq!(tick.writes.len(), 2);
        assert_eq!(tick.writes[0].cell, at(10));
        assert_eq!(tick.writes[0].content.as_char(), '0');
        assert_eq!(tick.writes[1].cell, at(11));
        assert_eq!(tick.writes[1].content.as_char(), '3');
        // The result is ordinary Source on the next Tick, and ordinary Source
        // beginning with `03` is a Function spelling the table does not hold.
        // ADR 0020 expects a written result to be readable as Source rather
        // than privileged, and this is what that reads as.
        assert_eq!(cell(&src, 10), (Some('0'), Some(Glyph::Function)));
        assert_eq!(cell(&src, 11), (Some('3'), Some(Glyph::Function)));
    }

    fn assert_only_bang_display(plan: &TickPlan, grid: Grid, anchors: &[usize]) {
        let expected: Vec<_> = anchors
            .iter()
            .flat_map(|anchor| [*anchor, anchor + 1])
            .map(|index| CellWrite {
                cell: grid.cell_index(index).unwrap(),
                content: crate::source::CellContent::new(b'*').unwrap(),
            })
            .collect();
        assert_eq!(plan.writes, expected);
    }

    #[test]
    fn a_note_and_bang_produced_this_tick_play_the_new_note_this_tick() {
        for initial_call in ["!>007FD4", "!>007F", "!>007FXX"] {
            let mut src = SourceUnderTest::new(Grid::new(16, 4));
            let at = src.cells();
            src.write(at(0), ".=0101");
            src.write(at(22), ".^3C");
            src.write(at(32), initial_call);

            let tick = src.execute();

            assert_eq!(
                tick.play_commands,
                vec![PlayCommand::Raw {
                    channel: MidiChannel::try_from(0).unwrap(),
                    velocity: Velocity::try_from(0x7F).unwrap(),
                    note: Note::try_from(60).unwrap(),
                }],
                "{initial_call}: C4 and its Bang belong to this Tick"
            );
            assert!(tick.diagnostics.is_empty());
            assert_eq!(src.row(2), "!>007FC4        ");
        }
    }

    #[test]
    fn a_generated_pulse_does_not_replay_on_the_next_tick() {
        let mut src = SourceUnderTest::new(Grid::new(16, 4));
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(22), ".^3C");
        src.write(at(32), "!>007F");

        let first = src.execute();
        assert_eq!(first.play_commands.len(), 1);
        assert_eq!(src.row(1), "**    .^3C      ");
        // Stop the producer, leaving the generated Bang visible in Source.
        src.write(at(4), "02");

        let second = src.execute();
        assert!(second.play_commands.is_empty());
        assert_eq!(src.row(1), "      .^3C      ");
        assert!(second.diagnostics.is_empty());
    }

    #[test]
    fn manually_entered_bang_is_display_only_and_never_activates_midi() {
        let mut src = SourceUnderTest::new(Grid::new(10, 3));
        let at = src.cells();
        src.write(at(10), "!>007FC4");
        src.write(at(20), "**");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert_eq!(src.row(2), "          ");
        assert!(tick.diagnostics.is_empty());
    }

    #[test]
    fn a_later_calculation_reads_an_operand_written_this_tick() {
        let mut src = SourceUnderTest::new(Grid::new(12, 3));
        let at = src.cells();
        src.write(at(2), ".+0203");
        src.write(at(12), ".+0102");

        let tick = src.execute();

        assert_eq!(src.row(1), ".+0502      ");
        assert_eq!(src.row(2), "07          ");
        assert!(tick.diagnostics.is_empty());
    }

    #[test]
    fn a_rejected_bang_operand_neither_activates_nor_erases() {
        let mut src = SourceUnderTest::new(Grid::new(16, 3));
        let at = src.cells();
        src.write(at(0), "!>00**C4");
        src.write(at(20), "!>007FC4");
        let before = src.snapshot();
        assert!(
            src.language_map()
                .diagnostics()
                .any(|d| d.message.contains("expected a number"))
        );

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert!(tick.writes.is_empty());
        assert_eq!(src.snapshot(), before);
        assert!(
            src.language_map()
                .diagnostics()
                .any(|d| d.message.contains("expected a number"))
        );
    }

    #[test]
    fn test_root_play_function_emits_one_play_command_without_a_cell_write() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(20), "!>007FC4");

        let tick = src.execute();

        assert_eq!(
            tick.play_commands,
            vec![PlayCommand::Raw {
                channel: MidiChannel::try_from(0).unwrap(),
                velocity: Velocity::try_from(0x7F).unwrap(),
                note: Note::try_from(60).unwrap()
            }]
        );
        assert_only_bang_display(&tick, src.grid, &[10]);
        assert_eq!(src.row(1), "**        ");
    }

    #[test]
    fn test_root_timed_play_function_emits_one_command_carrying_its_whole_lifetime() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(20), "!~017FC403");

        let tick = src.execute();

        // ADR 0016 puts the lifetime in the Tick Plan rather than resolving it
        // here: a Tick plans one Tick, and the Note Off this command owes is
        // due at another one. Each operand also differs from the others, so a
        // transposed declaration changes this answer rather than diagnosing.
        assert_eq!(
            tick.play_commands,
            vec![PlayCommand::Timed {
                channel: MidiChannel::try_from(1).unwrap(),
                velocity: Velocity::try_from(0x7F).unwrap(),
                note: Note::try_from(60).unwrap(),
                length: Length::from(3),
            }]
        );
        assert_only_bang_display(&tick, src.grid, &[10]);
        assert!(tick.diagnostics.is_empty());
        assert_eq!(src.row(1), "**        ");
    }

    #[test]
    fn an_unactivated_terminal_root_does_not_report_a_dependency_it_never_read() {
        // A terminal root with no Bang takes no turn at all. Asking about its
        // suppliers before asking whether it was ever going to run made it
        // complain, every Tick, about an operand it was never going to read.
        let mut src = source();
        let at = src.cells();
        src.write(at(2), ".+0102Z");
        // Its channel operand is the destination above, and nothing activates
        // it, so this root is inert whatever that operand ends up holding.
        src.write(at(10), "!>007FC4");

        let tick = src.execute();

        assert!(
            !tick
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("data dependency")),
            "an inert root reported a dependency: {:?}",
            tick.diagnostics
        );
        assert!(tick.play_commands.is_empty());
    }

    #[test]
    fn a_half_typed_function_claims_every_cell_its_arity_declares() {
        // A Raw Play takes three operands, so `!>00` claims eight Cells from
        // its anchor whether or not anyone has written into them. The Bang the
        // Equality produces lands on the last two of them, which makes it that
        // Function's Note operand and not an activation — and the Play root
        // beneath it takes no turn.
        //
        // This is the geometry `8e7bdce` worked around by filtering the slots
        // a whitespace run could not reach. There is no run to reach past now:
        // the claim is arity, one derivation, and the two readings that used to
        // disagree about this Cell agree that it is a slot.
        //
        // What is owed here is the diagnostic, not a different claim. ADR 0032
        // requires a result covering an operand slot to be told apart from an
        // activation *before* the Tick publishes, and reported either way;
        // `cell-indexed-parse/03` is where that verdict is drawn, and until it
        // lands this Source is refused silently rather than loudly.
        let mut src = SourceUnderTest::new(Grid::new(16, 4));
        let at = src.cells();
        src.write(at(6), ".=0101");
        // Velocity at columns 4-5, Note at columns 6-7. The Bang lands on the
        // Note, inside the claim.
        src.write(at(16), "!>00");
        src.write(at(38), "!>007FC4");

        let tick = src.execute();

        assert_eq!(
            src.language_map()
                .expressions()
                .find(|expression| expression.span().start().get() == 16)
                .map(|expression| expression.span().end().get()),
            Some(23),
            "the half-typed Play claimed something other than its arity",
        );
        assert!(
            tick.play_commands.is_empty(),
            "the Bang was delivered as an activation from inside an operand slot: {:?}",
            tick.play_commands,
        );
    }

    #[test]
    fn a_truncated_spatial_supplier_preserves_surviving_operands() {
        let mut src = source();
        let at = src.cells();
        src.write(at(6), ".+01");
        src.write(at(14), ".+9902");
        let tick = src.execute();
        assert!(
            tick.diagnostics
                .iter()
                .any(|d| d.message.contains("crosses the row edge"))
        );
        assert_eq!(src.row(2), "    9B    ");
    }

    #[test]
    fn a_result_beside_a_root_preserves_the_root_and_activates_it_each_tick() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(12), "!>007FC4");
        for tick in 0..3 {
            let plan = src.execute();
            assert_eq!(plan.play_commands.len(), 1, "tick {tick}");
            assert!(plan.diagnostics.is_empty());
            assert_eq!(src.row(1), "**!>007FC4", "tick {tick}");
        }
    }

    #[test]
    fn test_root_monophonic_play_function_emits_one_command_of_its_own_kind() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(20), "!%017FC403");

        let tick = src.execute();

        // `!%` shares Timed Play's operand shape, so what a Tick Plan carries
        // is the variant rather than the values: the Playback Engine owns a
        // Mono voice by its channel and a Timed voice by its channel and note,
        // and only the spelling that arrived says which of the two this is.
        assert_eq!(
            tick.play_commands,
            vec![PlayCommand::Mono {
                channel: MidiChannel::try_from(1).unwrap(),
                velocity: Velocity::try_from(0x7F).unwrap(),
                note: Note::try_from(60).unwrap(),
                length: Length::from(3),
            }]
        );
        assert_only_bang_display(&tick, src.grid, &[10]);
        assert!(tick.diagnostics.is_empty());
        assert_eq!(src.row(1), "**        ");
    }

    #[test]
    fn test_root_control_change_and_pitch_bend_emit_the_command_their_operands_name() {
        // Every operand of each spelling differs from the others, and all
        // three are legal in all three positions, so a transposition inside
        // the `define_functions!` declaration changes this answer rather than
        // diagnosing its way out.
        for (expression, expected) in [
            (
                "!c010203",
                PlayCommand::ControlChange {
                    channel: MidiChannel::try_from(1).unwrap(),
                    controller: Controller::try_from(2).unwrap(),
                    value: ControlValue::try_from(3).unwrap(),
                },
            ),
            (
                "!b010203",
                PlayCommand::PitchBend {
                    channel: MidiChannel::try_from(1).unwrap(),
                    lsb: BendLsb::try_from(2).unwrap(),
                    msb: BendMsb::try_from(3).unwrap(),
                },
            ),
        ] {
            let mut src = source();
            let at = src.cells();
            src.write(at(0), ".=0101");
            src.write(at(20), expression);

            let tick = src.execute();

            assert_eq!(tick.play_commands, vec![expected], "{expression}");
            assert_only_bang_display(&tick, src.grid, &[10]);
            assert!(tick.diagnostics.is_empty(), "{expression}");
            assert_eq!(src.row(1), "**        ", "{expression}");
        }
    }

    #[test]
    fn test_control_change_and_pitch_bend_operands_outside_their_domains_emit_nothing() {
        // One out-of-range operand per role, including both halves of each
        // pair that shares a domain: the message names which operand the
        // Source wrote out of range, which is what the role types buy at the
        // Source rather than in the code that reads the command.
        for (expression, message) in [
            (
                "!c100203",
                "MIDI channel 10 is outside the range 00\u{2013}0F",
            ),
            (
                "!c018003",
                "MIDI controller 80 is outside the range 00\u{2013}7F",
            ),
            (
                "!c010280",
                "MIDI value 80 is outside the range 00\u{2013}7F",
            ),
            (
                "!b100203",
                "MIDI channel 10 is outside the range 00\u{2013}0F",
            ),
            ("!b018003", "MIDI lsb 80 is outside the range 00\u{2013}7F"),
            ("!b010280", "MIDI msb 80 is outside the range 00\u{2013}7F"),
        ] {
            let mut src = source();
            let at = src.cells();
            src.write(at(0), ".=0101");
            src.write(at(20), expression);

            let tick = src.execute();

            assert!(tick.play_commands.is_empty(), "{expression}");
            assert_only_bang_display(&tick, src.grid, &[10]);
            assert_eq!(tick.diagnostics.len(), 1, "{expression}");
            assert_eq!(tick.diagnostics[0].message, message, "{expression}");
        }
    }

    #[test]
    fn test_lifetime_play_operands_outside_their_domains_diagnose_and_emit_nothing() {
        for (expression, message) in [
            (
                "!~107FC403",
                "MIDI channel 10 is outside the range 00\u{2013}0F",
            ),
            (
                "!~0080C403",
                "MIDI velocity 80 is outside the range 00\u{2013}7F",
            ),
            (
                "!%107FC403",
                "MIDI channel 10 is outside the range 00\u{2013}0F",
            ),
            (
                "!%0080C403",
                "MIDI velocity 80 is outside the range 00\u{2013}7F",
            ),
        ] {
            let mut src = source();
            let at = src.cells();
            src.write(at(0), ".=0101");
            src.write(at(20), expression);

            let tick = src.execute();

            assert!(tick.play_commands.is_empty(), "{expression}");
            assert_only_bang_display(&tick, src.grid, &[10]);
            assert_eq!(tick.diagnostics.len(), 1, "{expression}");
            assert_eq!(tick.diagnostics[0].message, message, "{expression}");
        }
    }

    #[test]
    fn test_play_preserves_zero_velocity_as_an_explicit_command() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(20), "!>0F00A0");

        let tick = src.execute();

        assert_eq!(
            tick.play_commands,
            vec![PlayCommand::Raw {
                channel: MidiChannel::try_from(0xF).unwrap(),
                velocity: Velocity::try_from(0).unwrap(),
                note: Note::try_from(21).unwrap()
            }]
        );
        assert!(tick.diagnostics.is_empty());
    }

    #[test]
    fn test_play_velocity_above_midi_range_is_diagnosed() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(20), "!>0080C4");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert_only_bang_display(&tick, src.grid, &[10]);
        assert_eq!(tick.diagnostics.len(), 1);
        assert_eq!(tick.diagnostics[0].start(), 20);
        assert_eq!(tick.diagnostics[0].end(), 27);
        assert_eq!(
            tick.diagnostics[0].message,
            "MIDI velocity 80 is outside the range 00–7F"
        );
    }

    #[test]
    fn test_play_channel_above_midi_range_is_diagnosed() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(20), "!>107FC4");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert_only_bang_display(&tick, src.grid, &[10]);
        assert_eq!(tick.diagnostics.len(), 1);
        assert_eq!(
            tick.diagnostics[0].message,
            "MIDI channel 10 is outside the range 00–0F"
        );
    }

    #[test]
    fn test_nested_play_is_diagnosed_without_emitting_a_command() {
        let mut src = SourceUnderTest::new(Grid::new(12, 3));
        let at = src.cells();
        src.write(at(0), ".+!>007FC401");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert!(tick.writes.is_empty());
        assert!(
            tick.diagnostics
                .iter()
                .any(|d| d.message == lang::InterpretationError::NestedEffectFunction.to_string())
        );
        assert!(
            tick.diagnostics
                .iter()
                .any(|d| d.message.contains("supplied no typed result"))
        );
    }

    #[test]
    fn test_nested_evaluation_cannot_change_play_operand_types() {
        for (expression, expected) in [
            ("!>.^007FC4", "expected a number, found \"C/\""),
            ("!>00.^7FC4", "expected a number, found \"G9\""),
            ("!>007F.vC4", "expected a note, found \"3C\""),
        ] {
            let mut src = SourceUnderTest::new(Grid::new(expression.len(), 3));
            let at = src.cells();
            src.write(at(0), ".=0101");
            src.write(at(expression.len() * 2), expression);

            let tick = src.execute();

            assert!(tick.play_commands.is_empty(), "{expression}");
            assert_only_bang_display(&tick, src.grid, &[expression.len()]);
            assert_eq!(tick.diagnostics.len(), 1, "{expression}");
            assert_eq!(tick.diagnostics[0].message, expected, "{expression}");
        }
    }

    #[test]
    fn test_play_commands_retain_expression_order_and_repeat_on_every_tick() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(20), "!>0001C4");
        src.write(at(30), ".=0101");
        src.write(at(50), "!>017FA4");

        // Two successive Ticks of one Playback run, which is what "every Tick"
        // means: the same commands at Tick `0` and again at Tick `1`.
        let first = src.execute();
        let second = src.execute();
        let expected = vec![
            PlayCommand::Raw {
                channel: MidiChannel::try_from(0).unwrap(),
                velocity: Velocity::try_from(1).unwrap(),
                note: Note::try_from(60).unwrap(),
            },
            PlayCommand::Raw {
                channel: MidiChannel::try_from(1).unwrap(),
                velocity: Velocity::try_from(0x7F).unwrap(),
                note: Note::try_from(69).unwrap(),
            },
        ];

        assert_eq!(first.play_commands, expected);
        assert_eq!(second.play_commands, expected);
        assert_only_bang_display(&first, src.grid, &[10, 40]);
        assert_only_bang_display(&second, src.grid, &[10, 40]);
    }

    #[test]
    fn test_inactive_terminal_root_emits_neither_a_command_nor_a_diagnostic() {
        let mut src = source();
        let at = src.cells();
        // Every operand of this Raw Play is outside its MIDI domain. An
        // inactive terminal root is never evaluated, so not even the domain
        // diagnostics it would produce reach the Tick Plan.
        src.write(at(0), "!>1080C4");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert!(tick.diagnostics.is_empty());
        assert!(tick.writes.is_empty());
    }

    #[test]
    fn test_manual_bang_is_inert_at_either_vertical_position() {
        for (bang, root) in [(0, 10), (10, 0)] {
            let mut src = source();
            let at = src.cells();
            src.write(at(bang), "**");
            src.write(at(root), "!>007FC4");

            let tick = src.execute();

            assert!(
                tick.play_commands.is_empty(),
                "Bang at {bang}, root at {root}"
            );
            assert!(tick.diagnostics.is_empty(), "Bang at {bang}");
            assert_eq!(&src.snapshot()[bang..bang + 2], "  ");
        }
    }

    #[test]
    fn test_one_bang_producer_activates_one_terminal_root_once() {
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(20), "!>007FC4");

        let tick = src.execute();

        assert_eq!(
            tick.play_commands,
            vec![PlayCommand::Raw {
                channel: MidiChannel::try_from(0).unwrap(),
                velocity: Velocity::try_from(0x7F).unwrap(),
                note: Note::try_from(60).unwrap()
            }]
        );
        assert_eq!(src.row(1), "**        ");
    }

    #[test]
    fn test_a_horizontally_adjacent_bang_does_not_activate_a_terminal_root() {
        // Pins the limitation `spatial-tick-planning/02` inherits. ADR 0006's
        // west and east anchors sit two Cells from the Bang, but a Raw Play's
        // operands occupy those Cells, and the walk partitions a row by parse,
        // so a Bang beside a Function is that Function's operand Source. The
        // contiguous spellings form no root at all, and the space-separated
        // ones put the Bang anchor three or more columns away from the root
        // anchor. Every horizontal placement is inert; the day the partition
        // Bang activation reads changes, this test says so.
        for expression in ["**!>007FC4", "!>007FC4**", "** !>007FC4", "!>007FC4 **"] {
            // The Grid is as wide as the spelling it holds. The geometry under
            // test is horizontal, so a spelling that outran the row would wrap
            // onto the next one and pin nothing.
            let mut src = SourceUnderTest::new(Grid::new(expression.len(), 6));
            let at = src.cells();
            src.write(at(0), expression);

            assert_eq!(src.row(0), expression, "{expression:?} did not fit one row");

            let tick = src.execute();

            assert!(
                tick.play_commands.is_empty(),
                "{expression:?} emitted a command"
            );
        }
    }

    #[test]
    fn test_a_value_producing_root_evaluates_without_a_bang() {
        // Gating every root behind activation belongs to spatial Tick
        // planning. Until then only terminal roots consult the Bang.
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".+0102");

        let tick = src.execute();

        assert_eq!(tick.writes.len(), 2);
    }

    #[test]
    fn an_equal_comparison_commits_a_bang_and_an_unequal_one_commits_nothing() {
        // Equality answers a pulse, so its two answers reach the Source by two
        // different paths: the equal case is an ordinary two-Cell result write
        // that must render as `**`, and the unequal case rides the existing
        // Empty signal and must leave the result row exactly as it found it.
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".=0303");

        let tick = src.execute();

        assert_eq!(src.row(1), "**        ");
        assert_eq!(tick.writes.len(), 2);

        let mut src = source();

        let at = src.cells();
        src.write(at(0), ".=0304");

        let tick = src.execute();

        assert_eq!(src.row(1), "          ");
        assert!(tick.writes.is_empty());
        assert!(tick.diagnostics.is_empty());
    }

    #[test]
    fn a_zero_divisor_diagnoses_and_commits_nothing() {
        // The ticket pairs "diagnoses" with "produces no result", and only the
        // Source can show the second half: an Interpreter error has to reach
        // the Tick Plan as a diagnostic AND leave the result row untouched,
        // rather than committing a Cell the next Tick would read as an operand.
        let mut src = source();
        let at = src.cells();
        src.write(at(0), ".%0A00");

        let tick = src.execute();

        assert_eq!(src.row(1), "          ");
        assert!(tick.writes.is_empty());
        assert_eq!(tick.diagnostics.len(), 1);
        assert_eq!(tick.diagnostics[0].message, "cannot modulo by zero");
    }

    #[test]
    fn test_result_above_nine_commits_both_hexadecimal_cells() {
        trace();

        let mut src = source();

        let at = src.cells();

        // Numbers are hexadecimal, so 5 + 5 is `0A` — a Cell holding only the
        // leading `0` would be a truncated, and wrong, result
        src.write(at(0), ".+0505");

        src.execute();

        assert_eq!(src.row(1), "0A        ");
    }

    #[test]
    fn test_result_below_the_bottom_row_is_discarded() {
        trace();

        let mut src = source();

        let at = src.cells();

        // An Expression in the bottom row has nowhere to write: its result
        // falls outside the Source and is discarded, never clamped onto a Cell
        // the user owns
        src.write(at(50), ".+0102");
        src.write(at(59), "Z");

        let tick = src.execute();

        assert_eq!(src.row(5), ".+0102   Z");
        assert_eq!(src.get(at(59)), Some("Z".to_string()));
        assert!(tick.writes.is_empty());
        assert_eq!(tick.diagnostics.len(), 1);
        assert_eq!(tick.diagnostics[0].start(), 50);
        assert_eq!(tick.diagnostics[0].end(), 55);
        assert_eq!(
            tick.diagnostics[0].message,
            "result \"03\" falls below the Source"
        );
        assert!(tick.writes.is_empty());
    }

    #[test]
    fn test_row_confined_expressions_do_not_produce_a_wrapped_result() {
        trace();

        let mut src = source();

        let at = src.cells();

        // The last-column `.` is one incomplete Expression and `+0102` is an
        // invalid Expression in the next row. Neither can produce the `03`
        // that their formerly wrapped `.+0102` run produced.
        src.write(at(9), ".+0102");

        src.execute();

        assert_eq!(src.row(1), "+0102     ");
        assert_eq!(src.row(2), "          ");
    }

    #[test]
    fn test_operand_slot_hints_do_not_cross_a_row_edge() {
        trace();

        let mut src = source();

        let at = src.cells();

        // The incomplete `.+` occupies the last two Cells of row 0. Its four
        // operand-slot hints have no Cells left in that row, so they must not
        // classify Cells at the beginning of row 1.
        src.write(at(8), ".+");

        assert_eq!(glyph_at(&src, 8), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 9), Some(Glyph::Function));
        for idx in 10..14 {
            assert_eq!(glyph_at(&src, idx), None);
        }
    }

    #[test]
    fn test_tick_does_not_evaluate_an_expression_across_a_row_edge() {
        trace();

        let mut src = source();

        let at = src.cells();

        // This formerly parsed as one wrapped `.+0102` Expression. It is now
        // an incomplete `.+` followed by a separate literal `0102`, neither
        // of which can produce the old `03` result.
        src.write(at(8), ".+0102");

        src.execute();

        assert_eq!(src.row(1), "0102      ");
        assert_eq!(src.row(2), "          ");
    }

    #[test]
    fn test_expression_without_a_function_commits_nothing() {
        trace();

        let mut src = source();

        let at = src.cells();

        // A bare Number is not a computation: the Interpreter has nothing to
        // apply, so the Expression has no result to commit
        src.write(at(0), "03");

        src.execute();

        assert_eq!(src.row(1), "          ");
    }

    #[test]
    fn test_repeated_ticks_do_not_cascade_results_down_the_grid() {
        trace();

        let mut src = source();

        let at = src.cells();
        src.write(at(0), ".+0102");

        src.execute();
        let after_first_tick = src.snapshot();

        // A committed result is not itself a computation, so re-Ticking the
        // same Source re-commits the same Cells and never marches down the
        // grid. Ticks `1` through `4` of the same Playback run, not Tick `0`
        // four times: the helper counts, so what is re-Ticked here is a run
        // ADR 0012 admits.
        for _ in 0..4 {
            src.execute();
            assert_eq!(src.snapshot(), after_first_tick);
        }

        assert_eq!(src.row(0), ".+0102    ");
        assert_eq!(src.row(1), "03        ");
        // and every row below the one it wrote is still empty
        for r in 2..src.row_count() {
            assert_eq!(src.row(r), "          ", "row {r} is untouched");
        }
    }

    #[test]
    fn test_empty_result_of_an_incomplete_function_commits_nothing() {
        trace();

        let mut src = source();

        let at = src.cells();

        // `.+` with no operands contains a Function, but the Interpreter has
        // no value to add. An empty result must never reach a Cell.
        src.write(at(0), ".+");

        src.execute();

        assert_eq!(src.row(1), "          ");
    }

    #[test]
    fn test_function_over_a_literal_still_commits_its_result() {
        trace();

        let mut src = source();

        let at = src.cells();

        // `.+0102` is a computation — suppressing literals must not suppress
        // a Function applied to them.
        src.write(at(0), ".+0102");

        src.execute();

        assert_eq!(src.row(1), "03        ");
    }

    #[test]
    fn test_incomplete_expression_is_not_evaluated_and_suppresses_only_its_own_result() {
        trace();

        // Wide enough to hold both, because `.+` with no operands still claims
        // the six Cells its arity declares. The second Addition begins after
        // them, so the two are unrelated — which is the whole of what this
        // test is about, and now a fact about arity rather than about the
        // space between them.
        let mut src = SourceUnderTest::new(Grid::new(16, 2));

        let at = src.cells();

        src.write(at(0), ".+");
        src.write(at(6), ".+0102");

        let tick = src.execute();

        assert_eq!(src.row(1), "      03        ");
        assert_eq!(tick.writes.len(), 2);
        assert!(tick.diagnostics.is_empty());
    }

    #[test]
    fn test_writes_play_commands_and_diagnostics_follow_one_producer_order() {
        let mut src = SourceUnderTest::new(Grid::new(20, 9));
        let at = src.cells();
        src.write(at(0), ".=0101");
        src.write(at(40), "!>0001C4");
        src.write(at(50), "./0100");
        src.write(at(60), ".=0202");
        src.write(at(100), "!>027FA4");
        src.write(at(130), ".^80");
        src.write(at(140), ".+0102");

        let tick = src.execute();

        assert_eq!(
            tick.play_commands,
            vec![
                PlayCommand::Raw {
                    channel: MidiChannel::try_from(0).unwrap(),
                    velocity: Velocity::try_from(1).unwrap(),
                    note: Note::try_from(60).unwrap(),
                },
                PlayCommand::Raw {
                    channel: MidiChannel::try_from(2).unwrap(),
                    velocity: Velocity::try_from(0x7F).unwrap(),
                    note: Note::try_from(69).unwrap(),
                },
            ]
        );
        assert_eq!(
            tick.diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.start(), diagnostic.message.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (50, "cannot divide by zero"),
                (130, "Number 80 cannot be converted to a Note"),
            ]
        );
        assert_eq!(
            tick.writes
                .iter()
                .map(|write| (write.cell.get(), write.content.as_char()))
                .collect::<Vec<_>>(),
            vec![
                (20, '*'),
                (21, '*'),
                (80, '*'),
                (81, '*'),
                (160, '0'),
                (161, '3'),
            ]
        );
        assert_eq!(src.row(8), "03                  ");
    }

    #[test]
    fn test_a_computation_completed_by_a_write_evaluates_this_tick() {
        let mut src = source();
        let at = src.cells();
        // The original Function retains its turn while an earlier producer
        // supplies its missing operand.
        src.write(at(4), ".+0002");
        src.write(at(10), ".+01");

        let first = src.execute();

        assert_eq!(src.row(1), ".+0102    ");
        assert_eq!(src.row(2), "03        ");
        assert_eq!(
            first
                .writes
                .iter()
                .map(|write| (write.cell.get(), write.content.as_char()))
                .collect::<Vec<_>>(),
            vec![(14, '0'), (15, '2'), (20, '0'), (21, '3')]
        );
        assert!(first.diagnostics.is_empty());
    }

    #[test]
    fn a_failed_spatial_supplier_preserves_original_operand_cells() {
        let mut src = SourceUnderTest::new(Grid::new(8, 3));
        let at = src.cells();
        src.write(at(2), ".+01");
        src.write(at(8), ".+0502");
        let tick = src.execute();
        assert_eq!(&src.snapshot()[16..18], "07");
        assert!(tick.play_commands.is_empty());
    }

    #[test]
    fn test_one_source_snapshot_at_one_tick_plans_one_tick_plan() {
        trace();

        // ADR 0012 makes the Tick an explicit input so that the Tick Plan stays
        // a function of the Source Snapshot and the Tick together. Two Sources
        // typed identically on one Grid and interpreted at the same absolute
        // Tick must plan the same writes, Play Commands, and diagnostics.
        //
        // What that pins is determinism at a fixed Tick: interpretation of one
        // Source Snapshot carries nothing over from an earlier interpretation
        // of it, so a Tick Plan is reproducible from the Snapshot and the Tick
        // alone. It does not pin that a Function reads time from its Tick
        // input rather than from a clock — two executions this close together
        // read the same coarse clock and agree anyway — and it does not pin
        // that a different Tick plans a different Tick Plan. Those are the
        // other half of ADR 0012, and they need a test that varies the Tick.
        //
        // One Grid for both, because a Tick Plan names Cells by index and an
        // index belongs to the Grid that minted it: two Grids of one shape
        // would differ here without either Snapshot differing.
        let grid = Grid::new(10, 9);
        let plan_at = |tick| {
            let mut src = SourceUnderTest::new(grid);
            let at = src.cells();
            // Arithmetic that writes, an activated Raw Play that commands, and
            // a result with no row beneath it to land in, which diagnoses: one
            // of each part of a Tick Plan.
            src.write(at(0), ".+0102");
            src.write(at(30), ".=0101");
            src.write(at(50), "!>007FC4");
            src.write(at(80), ".+0304");
            src.execute_at(tick)
        };

        let first = plan_at(Tick::new(7));
        let second = plan_at(Tick::new(7));

        assert!(!first.writes.is_empty());
        assert!(!first.play_commands.is_empty());
        assert!(!first.diagnostics.is_empty());
        assert_eq!(first, second);
    }

    #[test]
    fn test_an_overwritten_root_loses_its_reserved_turn() {
        trace();

        let mut src = source();

        let at = src.cells();

        // Row 0 replaces the row 1 Function spelling before its reserved turn.
        // The old root can no longer evaluate.
        src.write(at(0), ".+0101");
        src.write(at(10), ".+0304");

        let tick = src.execute();

        assert_eq!(src.row(1), "020304    ");
        assert_eq!(src.row(2), "          ");
        assert!(tick.diagnostics.is_empty());
    }

    #[test]
    fn a_shorter_result_leaves_the_earlier_results_tail_standing() {
        // ADR 0007: an ordinary result "writes exactly its current encoding and
        // never infers or clears Cells beyond that Span from an earlier, longer
        // result". Three Atoms at one Tick and one Atom at the next is the case
        // that catches the two ways that goes wrong — clearing the destination
        // row before writing, or remembering how wide the last result was — and
        // a same-width pair of results catches neither. The Cells the shorter
        // result does not reach still hold the earlier Sequence's characters,
        // not spaces.
        //
        // The results are planned through the Portal an evaluated Function's
        // answer passes through, and committed by the Source's own commit, so
        // what is read back is what two Ticks of a Playback run would leave.
        let mut src = source();
        let grid = src.grid;
        let root = grid.position(0, 0).expect("inside the Grid");

        src.commit_tick(&plan_result(grid, root, numbers(&[0x0A, 0x0B, 0x0C])));
        let shorter = plan_result(grid, root, Interpretation::Cell(Atom::Number(0x0D)));
        src.commit_tick(&shorter);

        assert_eq!(shorter.writes.len(), 2, "a result plans only its own Cells");
        assert_eq!(src.row(1), "0D0B0C    ");
    }

    #[test]
    fn cells_generated_by_a_sequence_result_are_read_as_ordinary_source() {
        // ADR 0007: successfully encoded Cells become ordinary Source content
        // under the same parsing, diagnostic, and generated-code rules as a
        // single Atom, "without a privileged literal-Sequence interpretation".
        // The way to state that is a comparison rather than a list of expected
        // Glyphs: a Source that was written by a Sequence result and a Source
        // the same characters were typed into are indistinguishable afterwards,
        // Cell for Cell, Glyph for Glyph, and diagnostic for diagnostic.
        //
        // Three adjacent Numbers read as one Expression whose head is an
        // unknown Function, so this pair shares a syntax diagnostic. That is
        // the point rather than a flaw in the case: ADR 0020 says the Source a
        // Tick writes "may intentionally contain an alignment or syntax
        // diagnostic on the next Tick", and a result that suppressed it would
        // be the privileged interpretation ADR 0007 rules out. A case whose
        // characters happened to parse cleanly could not tell the two apart.
        //
        // Diagnostics are compared as their Cells and their message because a
        // Diagnostic carries the Grid that minted its Span, and these two
        // Sources are built on Grids of their own.
        let mut generated = source();
        let grid = generated.grid;
        let root = grid.position(0, 0).expect("inside the Grid");
        generated.commit_tick(&plan_result(grid, root, numbers(&[0x0A, 0x0B, 0x0C])));

        let mut typed = source();
        let at = typed.cells();
        typed.write(at(10), "0A0B0C");

        assert_eq!(generated.snapshot(), typed.snapshot());
        assert_eq!(glyphs(&generated), glyphs(&typed));
        assert_eq!(reported(&generated), reported(&typed));
        // Six diagnostics rather than one: ADR 0033 resumes one Cell after a
        // refused spelling, so each Cell of the run is refused on its own and
        // says so. The last is the row's final Cell, where no spelling can be
        // read at all.
        assert_eq!(
            reported(&generated),
            vec![
                (10, 10, "unknown function \"0A\"".to_string()),
                (11, 11, "unknown function \"A0\"".to_string()),
                (12, 12, "unknown function \"0B\"".to_string()),
                (13, 13, "unknown function \"B0\"".to_string()),
                (14, 14, "unknown function \"0C\"".to_string()),
                (15, 15, "unknown function \"C \"".to_string()),
            ]
        );
    }

    #[test]
    fn a_committed_number_sequence_plans_nothing_of_its_own_on_the_next_tick() {
        // The other half of ADR 0007's generated-code rule, and the half a
        // comparison cannot make: what a Tick does when it meets the Cells an
        // earlier one wrote. These Number encodings contain no Function, so
        // the next Tick plans no writes or commands. A Sequence containing a
        // Function spelling can compute, as the adjacent test demonstrates.
        let mut src = source();
        let grid = src.grid;
        let root = grid.position(0, 0).expect("inside the Grid");
        src.commit_tick(&plan_result(grid, root, numbers(&[0x0A, 0x0B, 0x0C])));

        let tick = src.execute();

        assert!(tick.writes.is_empty());
        assert!(tick.play_commands.is_empty());
        assert_eq!(src.row(1), "0A0B0C    ");
        for row in 2..src.row_count() {
            assert_eq!(src.row(row), "          ", "row {row} is untouched");
        }
    }

    #[test]
    fn a_generated_function_sequence_computes_on_the_next_tick() {
        let mut generated = source();
        let grid = generated.grid;
        let root = grid.position(0, 0).unwrap();
        let result = Interpretation::Sequence(
            Sequence::new([
                Atom::Function(Function::Add),
                Atom::Number(1),
                Atom::Number(2),
            ])
            .unwrap(),
        );
        generated.commit_tick(&plan_result(grid, root, result));
        assert_eq!(generated.row(1), ".+0102    ");
        assert_eq!(generated.row(2), "          ");

        let mut typed = source();
        let at = typed.cells();
        typed.write(at(10), ".+0102");
        assert_eq!(glyphs(&generated), glyphs(&typed));
        assert_eq!(reported(&generated), reported(&typed));

        let plan = generated.execute();
        typed.execute();
        assert_eq!(generated.snapshot(), typed.snapshot());
        assert_eq!(generated.row(2), "03        ");
        assert_eq!(plan.writes.len(), 2);
        assert!(plan.diagnostics.is_empty());
        assert!(plan.play_commands.is_empty());
    }

    #[test]
    fn an_empty_sequence_needs_no_destination_and_preserves_source() {
        let mut src = source();
        let grid = src.grid;
        let root = grid.position(0, src.row_count() - 1).unwrap();
        let before = src.snapshot();
        let plan = plan_result(grid, root, Interpretation::Sequence(Sequence::empty()));
        src.commit_tick(&plan);
        assert_eq!(src.snapshot(), before);
        assert!(plan.writes.is_empty());
        assert!(plan.diagnostics.is_empty());
        assert!(plan.play_commands.is_empty());
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn a_persisted_source_carries_only_its_grid_and_its_character_cells() {
        // ADR 0009: a Portal is internal destination state, never a persisted
        // object. Most of that argument is one the compiler makes — `Portal` is
        // internal to the Source module, absent from the language crate, and
        // derives no `Serialize` — and this is the half a test can hold: the
        // persisted form has exactly two members, so no destination, and
        // nothing else one Tick resolved, can have joined it unnoticed.
        let source = Source::new(Grid::new(4, 2));

        let persisted = serde_json::to_value(&source).unwrap();

        assert_eq!(
            persisted
                .as_object()
                .expect("a persisted Source is a JSON object")
                .keys()
                .collect::<Vec<_>>(),
            vec!["grid", "inner"]
        );
    }
}
