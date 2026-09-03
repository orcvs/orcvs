use lang::{EXP_LEN, Error as LangError, Parser, SyntaxError};
use std::{fmt, sync::Arc};
use tracing::debug;

use crate::grid::Grid;

use super::SourceError;
use super::language_map::{LanguageMap, Span};
use super::tick;

pub const SPACE: &str = " ";
const SPACE_BYTE: u8 = b' ';

///
/// A problem with the Expression occupying `start..=end` in the current
/// Source revision. Diagnostics describe accepted user content; they never
/// reject an incomplete or invalid Live Edit.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub message: String,
    anchor: Option<crate::grid::Position>,
    span: Span,
}

impl Diagnostic {
    pub(super) fn for_range(
        grid: Grid,
        start: crate::grid::CellIndex,
        end: crate::grid::CellIndex,
        message: String,
    ) -> Self {
        Self {
            message,
            anchor: Some(grid.position_at(start)),
            span: Span::new(grid, start, end),
        }
    }

    pub(super) fn for_expression(
        anchor: crate::grid::Position,
        span: Span,
        message: String,
    ) -> Self {
        Self {
            message,
            anchor: Some(anchor),
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

    pub fn anchor(&self) -> Option<crate::grid::Position> {
        self.anchor
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CellWrite {
    pub idx: usize,
    pub content: char,
}

/// One interpreted MIDI instruction emitted by an active Terminal Output
/// Function. Tick planning decides which terminal roots are active and in what
/// order their commands appear; the output adapter turns each one into MIDI.
pub use lang::PlayCommand;

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
            .all(|byte| (0x20..=0x7e).contains(&byte))
        {
            return Err(D::Error::custom(
                "persisted Source contains a non-Cell character",
            ));
        }

        let mut source = Source::new(persisted.grid);
        source.inner = persisted.inner;
        source.rebuild_derived_state();
        Ok(source)
    }
}

impl Source {
    pub(crate) fn grid(&self) -> Grid {
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
    /// Sets the Cell at `idx` and recalculates the affected Expressions.
    /// A space empties the Cell, equivalent to `unset`.
    ///
    /// ```
    /// use orcvs::{grid::Grid, source::Source};
    ///
    /// let mut source = Source::new(Grid::new(10, 10));
    /// source.set(33, "!").unwrap();
    ///
    /// assert_eq!(source.get(33), Some("!".to_string()));
    /// ```
    ///
    pub fn set(&mut self, idx: usize, s: &str) -> Result<(), SourceError> {
        debug!("set {idx}: {s}");
        self.check_idx(idx)?;
        let byte = Self::check_content(s)?;
        self.check_expression_capacity(idx, byte)?;

        self.edit(idx, byte);
        Ok(())
    }

    ///
    /// Empties the Cell at `idx` and recalculates the affected Expressions.
    ///
    pub fn unset(&mut self, idx: usize) -> Result<(), SourceError> {
        self.check_idx(idx)?;

        self.edit(idx, SPACE_BYTE);
        Ok(())
    }

    ///
    /// Applies one already-validated edit. The revision it produces is what
    /// the console observes, so the edit reports nothing of its own.
    ///
    fn edit(&mut self, idx: usize, byte: u8) {
        self.set_source(idx, byte);
        self.rebuild_derived_state();
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
    /// Whether `idx` names a Cell. The Grid decides: an index it cannot
    /// address is one this Source has no Cell for.
    ///
    fn check_idx(&self, idx: usize) -> Result<(), SourceError> {
        match self.grid.cell_index(idx) {
            Some(_) => Ok(()),
            None => Err(SourceError::OutOfRange {
                idx,
                len: self.grid.count(),
            }),
        }
    }

    fn check_content(s: &str) -> Result<u8, SourceError> {
        match s.as_bytes() {
            [b] if (0x20..=0x7e).contains(b) => Ok(*b),
            _ => Err(SourceError::InvalidCell {
                content: s.to_string(),
            }),
        }
    }

    fn check_expression_capacity(&self, idx: usize, byte: u8) -> Result<(), SourceError> {
        if byte == SPACE_BYTE {
            return Ok(());
        }

        let range =
            LanguageMap::prospective_expression_range(self.grid, self.inner.as_bytes(), idx, byte)
                .expect("an occupied prospective Cell belongs to one Expression");
        let start = range.start();
        let end = range.end();

        let mut expression =
            String::from_utf8(self.inner.as_bytes()[start.get()..=end.get()].to_vec())
                .expect("Source Cells contain ASCII");
        let offset = idx - start.get();
        expression.replace_range(offset..=offset, &(byte as char).to_string());
        match Parser::from(&mut expression).analyze() {
            Err(LangError::Syntax(SyntaxError::ExpressionTooLong { .. })) => {
                Err(SourceError::ExpressionTooLong {
                    start: start.get(),
                    end: end.get(),
                    capacity: EXP_LEN,
                })
            }
            _ => Ok(()),
        }
    }

    pub fn get(&self, idx: usize) -> Option<String> {
        let b = *self.inner.as_bytes().get(idx)?;

        match b {
            SPACE_BYTE => None,
            _ => Some((b as char).to_string()),
        }
    }

    ///
    /// Runs one Tick: evaluates every Expression against the current Source
    /// snapshot, then commits the resulting Cell changes.
    ///
    /// Every Expression is interpreted before any Cell is written, so a result
    /// committed by one Expression can never become another Expression's input
    /// within the same Tick.
    ///
    pub fn execute(&mut self) -> TickPlan {
        let plan = self.plan_tick();
        self.commit_tick(&plan);

        plan
    }

    ///
    /// Interprets one Source Snapshot as ADR 0020's single row-major pass:
    /// every producer takes its turn in anchor order and emits its effects,
    /// then resolution folds those effects into the Tick Plan.
    ///
    /// Nothing here reads the Source between two turns, so a planned write
    /// gains no turn of its own and a Function a write generates first becomes
    /// actionable in the next Source Snapshot.
    ///
    fn plan_tick(&self) -> TickPlan {
        let mut effects = Vec::new();
        for turn in tick::turns(self.grid, &self.language_map) {
            turn.emit(self.grid, &self.language_map, &mut effects);
        }

        tick::resolve(effects)
    }

    fn commit_tick(&mut self, plan: &TickPlan) {
        // Commit every planned Cell before rebuilding any derived state.
        for write in &plan.writes {
            self.set_source(write.idx, write.content as u8);
        }
        self.rebuild_derived_state();
    }

    fn rebuild_derived_state(&mut self) {
        self.language_map = Arc::new(LanguageMap::build(self.grid, self.inner.as_bytes()));
    }

    ///
    /// Writes one already-validated ASCII byte at `idx` without
    /// recalculating Expressions.
    ///
    fn set_source(&mut self, idx: usize, byte: u8) {
        // SAFETY: The `edit` caller receives `byte` only after `Source::set`
        // validates it with `check_content`. The `commit_tick` caller receives
        // bytes from a Tick Plan built by `plan_tick`, whose ASCII assertion
        // also makes its character offsets valid byte offsets. Both callers
        // bounds-check `idx` before mutation, so the String stays valid UTF-8.
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            bytes[idx] = byte;
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

    use std::ops::{Deref, DerefMut};

    use crate::{
        glyph::Glyph,
        grid::Grid,
        source::{CellWrite, PlayCommand, Source, SourceError},
        test::trace,
    };

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
            .get(idx)
            .and_then(|s| s.chars().next())
            .filter(|c| *c != ' ');
        (content, glyph_at(source, idx))
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
    /// It derefs to the Source so a test still speaks to a Source directly.
    ///
    struct SourceUnderTest {
        grid: Grid,
        src: Source,
    }

    impl SourceUnderTest {
        fn new(grid: Grid) -> Self {
            Self {
                grid,
                src: Source::new(grid),
            }
        }

        ///
        /// Types `s` into consecutive Cells starting at `idx`, one accepted
        /// edit per Cell, exactly as a user would.
        ///
        fn write(&mut self, idx: usize, s: &str) {
            for (i, c) in s.chars().enumerate() {
                self.src.set(idx + i, &c.to_string()).unwrap();
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

    #[test]
    fn test_row_reads_the_shape_of_the_source_under_test() {
        trace();

        // A Source built on a shape other than this module's default. The row
        // helper must read the Cells of *this* Source, not the ones a Grid it
        // was never built from would name.
        let mut src = SourceUnderTest::new(Grid::new(8, 4));

        src.write(0, ".+0102");
        src.execute();

        assert_eq!(src.row(1), "03      ");
    }

    #[test]
    fn test_set_rejects_out_of_range_without_mutation() {
        trace();

        let mut src = source();
        let before = src.snapshot();

        // one past the last Cell the Grid addresses
        let past_the_end = src.count();
        let err = src.set(past_the_end, "x").unwrap_err();

        assert_eq!(
            err,
            SourceError::OutOfRange {
                idx: past_the_end,
                len: src.count()
            }
        );
        assert_eq!(src.snapshot(), before);
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn test_source_round_trip_restores_shape_contents_and_derived_state() {
        let grid = Grid::new(10, 3);
        let mut source = Source::new(grid);
        for (idx, content) in ".+0102".chars().enumerate() {
            source.set(idx, &content.to_string()).unwrap();
        }
        source.set(15, "x").unwrap();

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

        restored.execute();
        assert_eq!(restored.get(10), Some("0".to_string()));
        assert_eq!(restored.get(11), Some("3".to_string()));
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
    fn test_unset_rejects_out_of_range_without_mutation() {
        trace();

        let mut src = source();
        let before = src.snapshot();

        let err = src.unset(200).unwrap_err();

        assert_eq!(
            err,
            SourceError::OutOfRange {
                idx: 200,
                len: src.count()
            }
        );
        assert_eq!(src.snapshot(), before);
    }

    #[test]
    fn test_set_rejects_invalid_content_without_mutation() {
        trace();

        let mut src = source();
        src.set(5, "x").unwrap();
        let before = src.snapshot();

        for content in ["", "ab", "é", "\n"] {
            let err = src.set(5, content).unwrap_err();
            assert_eq!(
                err,
                SourceError::InvalidCell {
                    content: content.to_string()
                }
            );
            assert_eq!(src.snapshot(), before);
            assert_eq!(src.get(5), Some("x".to_string()));
        }
    }

    #[test]
    fn test_set_rejects_an_expression_beyond_parser_capacity_without_mutation() {
        let mut src = SourceUnderTest::new(Grid::new(80, 1));
        // Fifteen nested additions plus sixteen operands occupy 31 parser
        // atoms. Prefixing one more binary Function would also require an
        // empty second operand, exceeding the 32-atom parser capacity.
        src.write(2, &(".+".repeat(15) + &"00".repeat(16)));
        src.set(1, "+").unwrap();
        let before = src.snapshot();
        let before_diagnostics = diagnostics(&src);
        let before_glyphs = (0..src.count())
            .map(|idx| glyph_at(&src, idx))
            .collect::<Vec<_>>();

        let result = src.set(0, ".");

        assert_eq!(
            result,
            Err(SourceError::ExpressionTooLong {
                start: 0,
                end: 63,
                capacity: 32,
            })
        );
        assert_eq!(src.snapshot(), before);
        assert_eq!(diagnostics(&src), before_diagnostics);
        assert_eq!(
            (0..src.count())
                .map(|idx| glyph_at(&src, idx))
                .collect::<Vec<_>>(),
            before_glyphs
        );
    }

    #[test]
    fn test_an_edit_classifies_the_cells_it_affects() {
        trace();

        let mut src = source();

        src.set(0, ".").unwrap();
        assert_eq!(cell(&src, 0), (Some('.'), Some(Glyph::Char)));

        // completing the `.+` Function reclassifies Cell 0 and marks the four
        // empty operand-slot Cells (two 2-wide Numbers) as Number
        src.set(1, "+").unwrap();

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
        src.set(0, ".").unwrap();
        src.set(1, "+").unwrap();

        // deleting half the Function restores its raw character classification and
        // clears the operand-slot hints
        src.unset(1).unwrap();

        assert_eq!(cell(&src, 0), (Some('.'), Some(Glyph::Char)));
        for idx in 1..=5 {
            assert_eq!(cell(&src, idx), (None, None), "Cell {idx} was not cleared");
        }
    }

    #[test]
    fn test_set_near_grid_end_truncates_operand_hints() {
        trace();

        let mut src = source();

        // `.+` at the last two Cells wants four more operand-slot glyphs
        // than the Source has room for
        src.set(58, ".").unwrap();
        src.set(59, "+").unwrap();

        assert_eq!(glyph_at(&src, 58), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 59), Some(Glyph::Function));
        // the Function sits in the last two Cells, so its operand-slot hints
        // have nowhere to go: nothing past the row edge is classified
        assert_eq!(glyph_at(&src, 60), None);
    }

    #[test]
    fn test_editing_an_operand_slot_hint_restores_the_current_glyphs() {
        let mut src = SourceUnderTest::new(Grid::new(10, 1));
        src.set(0, ".").unwrap();
        src.set(1, "+").unwrap();
        assert_eq!(glyph_at(&src, 5), Some(Glyph::Number));

        src.set(5, "x").unwrap();
        assert_eq!(cell(&src, 5), (Some('x'), Some(Glyph::Char)));

        src.unset(5).unwrap();
        assert_eq!(cell(&src, 5), (None, Some(Glyph::Number)));
    }

    #[test]
    fn test_editing_an_operand_slot_matches_a_source_rebuilt_from_its_snapshot() {
        let grid = Grid::new(10, 2);
        let mut src = SourceUnderTest::new(grid);
        src.write(0, ".+");
        src.write(10, ".+0102");
        src.set(5, "x").unwrap();

        let mut rebuilt = Source::new(grid);
        for (idx, content) in src.snapshot().chars().enumerate() {
            if content != ' ' {
                rebuilt.set(idx, &content.to_string()).unwrap();
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
        src.write(10, ".+0102");
        src.set(8, ".").unwrap();

        src.set(9, "+").unwrap();
        assert_eq!(cell(&src, 8), (Some('.'), Some(Glyph::Function)));
        assert_eq!(cell(&src, 9), (Some('+'), Some(Glyph::Function)));
        assert_eq!(glyph_at(&src, 10), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 11), Some(Glyph::Function));

        src.unset(9).unwrap();
        assert_eq!(cell(&src, 8), (Some('.'), Some(Glyph::Char)));
        assert_eq!(cell(&src, 9), (None, None));
        assert_eq!(glyph_at(&src, 10), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 11), Some(Glyph::Function));
    }

    #[test]
    fn test_set_classifies_expression_immediately() {
        trace();

        let mut src = source();

        for (i, c) in ".+0101".chars().enumerate() {
            src.set(i, &c.to_string()).unwrap();
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

        src.set(9, ".").unwrap();
        src.set(10, "+").unwrap();

        // the two Cells are adjacent by index but sit in different rows, so
        // neither is classified as part of a Function
        assert_eq!(cell(&src, 9), (Some('.'), Some(Glyph::Char)));
        assert_eq!(cell(&src, 10), (Some('+'), Some(Glyph::Char)));
    }

    #[test]
    fn test_diagnostics_follow_the_visible_expression_revision() {
        trace();

        let mut src = source();

        // The incomplete Function remains visible and is diagnosed immediately.
        src.write(0, ".+01");
        assert_eq!(src.row(0), ".+01      ");
        assert_eq!(diagnostics(&src).len(), 1);
        assert_eq!(diagnostics(&src)[0].start(), 0);
        assert_eq!(diagnostics(&src)[0].end(), 3);
        assert_eq!(diagnostics(&src)[0].message, "expected a token");

        // Completing it removes the cause and therefore the diagnostic in the
        // same accepted edit.
        src.write(4, "02");
        assert!(diagnostics(&src).is_empty());

        // A valid prefix does not make trailing content disappear from the
        // Expression's diagnostic state.
        src.set(6, "Z").unwrap();
        assert_eq!(
            diagnostics(&src)[0].message,
            "unexpected trailing content \"Z\""
        );
        src.unset(6).unwrap();
        assert!(diagnostics(&src).is_empty());

        // Replacing a valid operand with invalid content creates a fresh
        // diagnostic for the current Expression, without rejecting the edit.
        src.set(4, "X").unwrap();
        assert_eq!(src.get(4), Some("X".to_string()));
        assert_eq!(diagnostics(&src).len(), 1);
        assert_eq!(
            diagnostics(&src)[0].message,
            "expected a number, found \"X2\""
        );

        // Removing the Expression removes its diagnostic rather than leaving
        // stale state attached to empty Cells.
        for idx in 0..6 {
            src.unset(idx).unwrap();
        }
        assert!(diagnostics(&src).is_empty());
    }

    #[test]
    fn test_retired_id_receives_the_unknown_function_diagnostic() {
        trace();

        let mut src = source();

        // ADR 0015 retired `id` from the Function vocabulary, so Source
        // containing it no longer parses as a Function and diagnoses like any
        // other unknown spelling.
        src.write(0, "id");

        assert_eq!(src.row(0), "id        ");
        assert_eq!(diagnostics(&src).len(), 1);
        assert_eq!(diagnostics(&src)[0].start(), 0);
        assert_eq!(diagnostics(&src)[0].end(), 1);
        assert_eq!(diagnostics(&src)[0].message, "unknown function \"id\"");
        // Classification is unaffected: an unrecognized run standing where a
        // Function is expected keeps the Function Glyph, because a Record that
        // failed to parse reports the Token its position expected. That is the
        // same operand-slot hint the editing tests cover, not a claim that `id`
        // is still a Function.
        assert_eq!(glyph_at(&src, 0), Some(Glyph::Function));
        assert_eq!(glyph_at(&src, 1), Some(Glyph::Function));
    }

    #[test]
    fn test_join_discards_stale_expression_state() {
        trace();

        let mut src = source();

        // `.+0101` starting at Cell 4; a Tick would write its result one row below
        for (i, c) in ".+0101".chars().enumerate() {
            src.set(i + 4, &c.to_string()).unwrap();
        }

        // Prepending `.+00` joins everything into one Expression at Cell 0;
        // the old Expression starting at Cell 4 no longer exists.
        src.write(0, ".+00");
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

        for (i, c) in "xx.+0101".chars().enumerate() {
            src.set(i, &c.to_string()).unwrap();
        }

        // deleting Cell 1 splits off a complete `.+0101` Expression at Cell 2
        src.unset(1).unwrap();

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
        src.set(5, "x").unwrap();

        src.set(5, " ").unwrap();

        assert_eq!(src.get(5), None);
        assert_eq!(src.snapshot(), " ".repeat(src.count()));
    }

    #[test]
    fn test_result_commits_its_complete_encoding_across_consecutive_cells() {
        trace();

        let mut src = source();

        // The README example: `.+0102` is 1 + 2, and a Number is two Cells wide
        src.write(0, ".+0102");

        let tick = src.execute();

        assert_eq!(src.row(1), "03        ");
        assert_eq!(src.get(10), Some("0".to_string()));
        assert_eq!(src.get(11), Some("3".to_string()));
        assert!(tick.play_commands.is_empty());
        assert_eq!(tick.writes.len(), 2);
        assert_eq!(tick.writes[0].idx, 10);
        assert_eq!(tick.writes[0].content, '0');
        assert_eq!(tick.writes[1].idx, 11);
        assert_eq!(tick.writes[1].content, '3');
        assert_eq!(cell(&src, 10), (Some('0'), Some(Glyph::Char)));
        assert_eq!(cell(&src, 11), (Some('3'), Some(Glyph::Char)));
    }

    #[test]
    fn test_root_play_function_emits_one_play_command_without_a_cell_write() {
        let mut src = source();
        // The Bang sits north of the root anchor, leaving the row below the
        // Play free to show that a terminal Function writes no result Cell.
        src.write(0, "**");
        src.write(10, "!>007FC4");

        let tick = src.execute();

        assert_eq!(
            tick.play_commands,
            vec![PlayCommand::Raw {
                channel: 0,
                velocity: 0x7F,
                note: 60,
            }]
        );
        assert!(tick.writes.is_empty());
        assert_eq!(src.row(2), "          ");
    }

    #[test]
    fn test_play_preserves_zero_velocity_as_an_explicit_command() {
        let mut src = source();
        src.write(0, "**");
        src.write(10, "!>0F00A0");

        let tick = src.execute();

        assert_eq!(
            tick.play_commands,
            vec![PlayCommand::Raw {
                channel: 0xF,
                velocity: 0,
                note: 21,
            }]
        );
        assert!(tick.diagnostics.is_empty());
    }

    #[test]
    fn test_play_velocity_above_midi_range_is_diagnosed() {
        let mut src = source();
        src.write(0, "**");
        src.write(10, "!>0080C4");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert!(tick.writes.is_empty());
        assert_eq!(tick.diagnostics.len(), 1);
        assert_eq!(tick.diagnostics[0].start(), 10);
        assert_eq!(tick.diagnostics[0].end(), 17);
        assert_eq!(
            tick.diagnostics[0].message,
            "MIDI velocity 80 is outside the range 00–7F"
        );
    }

    #[test]
    fn test_play_channel_above_midi_range_is_diagnosed() {
        let mut src = source();
        src.write(0, "**");
        src.write(10, "!>107FC4");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert!(tick.writes.is_empty());
        assert_eq!(tick.diagnostics.len(), 1);
        assert_eq!(
            tick.diagnostics[0].message,
            "MIDI channel 10 is outside the range 00–0F"
        );
    }

    #[test]
    fn test_nested_play_is_diagnosed_without_emitting_a_command() {
        let mut src = SourceUnderTest::new(Grid::new(12, 3));
        src.write(0, ".+!>007FC401");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert!(tick.writes.is_empty());
        assert_eq!(tick.diagnostics.len(), 1);
        assert_eq!(
            tick.diagnostics[0].message,
            "a terminal Function is valid only at the root of an Expression"
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
            src.write(0, expression);
            // A terminal root diagnoses only when it is evaluated, and it is
            // evaluated only when a Bang activates it.
            src.write(expression.len(), "**");

            let tick = src.execute();

            assert!(tick.play_commands.is_empty(), "{expression}");
            assert!(tick.writes.is_empty(), "{expression}");
            assert_eq!(tick.diagnostics.len(), 1, "{expression}");
            assert_eq!(tick.diagnostics[0].message, expected, "{expression}");
        }
    }

    #[test]
    fn test_play_commands_retain_expression_order_and_repeat_on_every_tick() {
        let mut src = source();
        // One Bang between the two roots activates both: the row above it is
        // its north anchor and the row below it is its south anchor.
        src.write(0, "!>0001C4");
        src.write(10, "**");
        src.write(20, "!>017FA4");

        let first = src.execute();
        let second = src.execute();
        let expected = vec![
            PlayCommand::Raw {
                channel: 0,
                velocity: 1,
                note: 60,
            },
            PlayCommand::Raw {
                channel: 1,
                velocity: 0x7F,
                note: 69,
            },
        ];

        assert_eq!(first.play_commands, expected);
        assert_eq!(second.play_commands, expected);
        assert!(first.writes.is_empty());
        assert!(second.writes.is_empty());
    }

    #[test]
    fn test_inactive_terminal_root_emits_neither_a_command_nor_a_diagnostic() {
        let mut src = source();
        // Every operand of this Raw Play is outside its MIDI domain. An
        // inactive terminal root is never evaluated, so not even the domain
        // diagnostics it would produce reach the Tick Plan.
        src.write(0, "!>1080C4");

        let tick = src.execute();

        assert!(tick.play_commands.is_empty());
        assert!(tick.diagnostics.is_empty());
        assert!(tick.writes.is_empty());
    }

    #[test]
    fn test_a_bang_north_or_south_of_a_terminal_root_activates_it() {
        // The Bang carries the geometry: at `(0, 0)` it activates the root
        // one row south, and at `(0, 1)` the root one row north.
        for (bang, root) in [(0, 10), (10, 0)] {
            let mut src = source();
            src.write(bang, "**");
            src.write(root, "!>007FC4");

            let tick = src.execute();

            assert_eq!(
                tick.play_commands,
                vec![PlayCommand::Raw {
                    channel: 0,
                    velocity: 0x7F,
                    note: 60,
                }],
                "Bang at {bang}, root at {root}"
            );
            assert!(tick.diagnostics.is_empty(), "Bang at {bang}");
        }
    }

    #[test]
    fn test_two_bangs_around_one_terminal_root_emit_one_command() {
        // ADR 0006: multiple Bangs do not make one root evaluate twice. The
        // root is aligned north of one Bang and south of the other, so both
        // reach it and it still has exactly one turn.
        let mut src = source();
        src.write(0, "**");
        src.write(10, "!>007FC4");
        src.write(20, "**");

        let tick = src.execute();

        assert_eq!(
            tick.play_commands,
            vec![PlayCommand::Raw {
                channel: 0,
                velocity: 0x7F,
                note: 60,
            }]
        );
    }

    #[test]
    fn test_a_horizontally_adjacent_bang_does_not_activate_a_terminal_root() {
        // Pins the limitation `spatial-tick-planning/02` inherits. ADR 0006's
        // west and east anchors sit two Cells from the Bang, but a Raw Play's
        // operands occupy those Cells, and `row_extents` splits Expression runs
        // only on spaces and `##`. So the contiguous spellings form no root at
        // all, and the space-separated ones put the Bang anchor three or more
        // columns away from the root anchor. Every horizontal placement is
        // inert; the day the partition Bang activation reads changes, this
        // test says so.
        for expression in ["**!>007FC4", "!>007FC4**", "** !>007FC4", "!>007FC4 **"] {
            // The Grid is as wide as the spelling it holds. The geometry under
            // test is horizontal, so a spelling that outran the row would wrap
            // onto the next one and pin nothing.
            let mut src = SourceUnderTest::new(Grid::new(expression.len(), 6));
            src.write(0, expression);

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
        src.write(0, ".+0102");

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
        src.write(0, ".=0303");

        let tick = src.execute();

        assert_eq!(src.row(1), "**        ");
        assert_eq!(tick.plan.writes.len(), 2);

        let mut src = source();
        src.write(0, ".=0304");

        let tick = src.execute();

        assert_eq!(src.row(1), "          ");
        assert!(tick.plan.writes.is_empty());
        assert!(tick.plan.diagnostics.is_empty());
    }

    #[test]
    fn a_zero_divisor_diagnoses_and_commits_nothing() {
        // The ticket pairs "diagnoses" with "produces no result", and only the
        // Source can show the second half: an Interpreter error has to reach
        // the Tick Plan as a diagnostic AND leave the result row untouched,
        // rather than committing a Cell the next Tick would read as an operand.
        let mut src = source();
        src.write(0, ".%0A00");

        let tick = src.execute();

        assert_eq!(src.row(1), "          ");
        assert!(tick.plan.writes.is_empty());
        assert_eq!(tick.plan.diagnostics.len(), 1);
        assert_eq!(tick.plan.diagnostics[0].message, "cannot modulo by zero");
    }

    #[test]
    fn test_result_above_nine_commits_both_hexadecimal_cells() {
        trace();

        let mut src = source();

        // Numbers are hexadecimal, so 5 + 5 is `0A` — a Cell holding only the
        // leading `0` would be a truncated, and wrong, result
        src.write(0, ".+0505");

        src.execute();

        assert_eq!(src.row(1), "0A        ");
    }

    #[test]
    fn test_result_below_the_bottom_row_is_discarded() {
        trace();

        let mut src = source();

        // An Expression in the bottom row has nowhere to write: its result
        // falls outside the Source and is discarded, never clamped onto a Cell
        // the user owns
        src.write(50, ".+0102");
        src.write(59, "Z");

        let tick = src.execute();

        assert_eq!(src.row(5), ".+0102   Z");
        assert_eq!(src.get(59), Some("Z".to_string()));
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

        // The last-column `.` is one incomplete Expression and `+0102` is an
        // invalid Expression in the next row. Neither can produce the `03`
        // that their formerly wrapped `.+0102` run produced.
        src.write(9, ".+0102");

        src.execute();

        assert_eq!(src.row(1), "+0102     ");
        assert_eq!(src.row(2), "          ");
    }

    #[test]
    fn test_operand_slot_hints_do_not_cross_a_row_edge() {
        trace();

        let mut src = source();

        // The incomplete `.+` occupies the last two Cells of row 0. Its four
        // operand-slot hints have no Cells left in that row, so they must not
        // classify Cells at the beginning of row 1.
        src.write(8, ".+");

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

        // This formerly parsed as one wrapped `.+0102` Expression. It is now
        // an incomplete `.+` followed by a separate literal `0102`, neither
        // of which can produce the old `03` result.
        src.write(8, ".+0102");

        src.execute();

        assert_eq!(src.row(1), "0102      ");
        assert_eq!(src.row(2), "          ");
    }

    #[test]
    fn test_expression_without_a_function_commits_nothing() {
        trace();

        let mut src = source();

        // A bare Number is not a computation: the Interpreter has nothing to
        // apply, so the Expression has no result to commit
        src.write(0, "03");

        src.execute();

        assert_eq!(src.row(1), "          ");
    }

    #[test]
    fn test_repeated_ticks_do_not_cascade_results_down_the_grid() {
        trace();

        let mut src = source();
        src.write(0, ".+0102");

        src.execute();
        let after_first_tick = src.snapshot();

        // A committed result is not itself a computation, so re-Ticking the
        // same Source re-commits the same Cells and never marches down the grid
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

        // `.+` with no operands contains a Function, but the Interpreter has
        // no value to add. An empty result must never reach a Cell.
        src.write(0, ".+");

        src.execute();

        assert_eq!(src.row(1), "          ");
    }

    #[test]
    fn test_function_over_a_literal_still_commits_its_result() {
        trace();

        let mut src = source();

        // `.+0102` is a computation — suppressing literals must not suppress
        // a Function applied to them.
        src.write(0, ".+0102");

        src.execute();

        assert_eq!(src.row(1), "03        ");
    }

    #[test]
    fn test_incomplete_expression_is_not_evaluated_and_suppresses_only_its_own_result() {
        trace();

        let mut src = source();

        // `.+` with no operands is an analysis-only record; the `.+0102` beside
        // it is unrelated and must still commit its `03` in the same Tick.
        src.write(0, ".+");
        src.write(3, ".+0102");

        let tick = src.execute();

        assert_eq!(src.row(1), "   03     ");
        assert_eq!(tick.writes.len(), 2);
        assert!(tick.diagnostics.is_empty());
    }

    #[test]
    fn test_writes_play_commands_and_diagnostics_follow_one_producer_order() {
        trace();

        // ADR 0020 orders every effect kind by the same row-major producer
        // Position. This Tick emits all three kinds from five producers whose
        // anchors interleave across rows and columns: two Play Commands, two
        // diagnostics, and one result write. Each kind must come out in the
        // order its producers took their turns — row first, then column —
        // rather than in an order of its own.
        let mut src = SourceUnderTest::new(Grid::new(20, 6));
        src.write(0, "**");
        src.write(20, "!>0001C4");
        src.write(30, "**");
        src.write(40, "./0100");
        src.write(50, "!>027FA4");
        src.write(60, ".^80");
        src.write(70, ".+0102");

        let tick = src.execute();

        // (0, 1) then (10, 2): the second Play's anchor is further right and
        // one row lower.
        assert_eq!(
            tick.play_commands,
            vec![
                PlayCommand::Raw {
                    channel: 0,
                    velocity: 1,
                    note: 60,
                },
                PlayCommand::Raw {
                    channel: 2,
                    velocity: 0x7F,
                    note: 69,
                },
            ]
        );
        // (0, 2) then (0, 3), interleaved between the two Play producers.
        assert_eq!(
            tick.diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.start(), diagnostic.message.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (40, "cannot divide by zero"),
                (60, "Number 80 cannot be converted to a Note"),
            ]
        );
        // The last producer's write, from the anchor (10, 3).
        assert_eq!(
            tick.writes,
            vec![
                CellWrite {
                    idx: 90,
                    content: '0',
                },
                CellWrite {
                    idx: 91,
                    content: '3',
                },
            ]
        );
        assert_eq!(src.row(4), "          03        ");
    }

    #[test]
    fn test_a_computation_completed_by_a_write_waits_for_the_next_snapshot() {
        trace();

        // A planned write gains no turn in the Tick that plans it. The row 0
        // Expression writes the `02` that completes the row 1 `.+01` into the
        // computation `.+0102`. That completed Expression is not part of this
        // Tick's Source Snapshot, so row 2 stays empty until the next Tick
        // reads the Source the write left behind.
        //
        // No Function spelling can be written today: every result is a Number
        // or a Note. Completing an Expression is as close as this Source gets
        // to generating one, and it pins the same rule.
        let mut src = source();
        src.write(4, ".+0002");
        src.write(10, ".+01");

        let first = src.execute();

        assert_eq!(
            first.writes,
            vec![
                CellWrite {
                    idx: 14,
                    content: '0',
                },
                CellWrite {
                    idx: 15,
                    content: '2',
                },
            ]
        );
        assert_eq!(src.row(1), ".+0102    ");
        assert_eq!(src.row(2), "          ");

        let second = src.execute();

        assert_eq!(src.row(2), "03        ");
        assert!(
            second
                .writes
                .iter()
                .any(|write| write.idx == 20 && write.content == '0')
        );
    }

    #[test]
    fn test_every_expression_evaluates_from_the_same_pre_tick_snapshot() {
        trace();

        let mut src = source();

        // The row 0 Expression commits `02` over the first two Cells of the
        // row 1 Expression. Row 1 must still evaluate the `.+0304` that was
        // there when the Tick began, not the `020304` the write leaves behind.
        src.write(0, ".+0101");
        src.write(10, ".+0304");

        src.execute();

        assert_eq!(src.row(1), "020304    ");
        assert_eq!(src.row(2), "07        ");
    }
}
