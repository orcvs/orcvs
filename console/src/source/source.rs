use lang::{
    Atom, Atoms, Error as LangError, Expression, Interpretation, Interpreter, Parser, SyntaxError,
    EXP_LEN,
};
use std::{collections::BTreeMap, fmt};
use tracing::debug;

use crate::glyph::Glyph;
use crate::grid::Grid;

use super::expression_map::{ExpressionMap, Range};
use super::SourceError;

pub const SPACE: &str = " ";
const SPACE_BYTE: u8 = b' ';

///
/// One Cell as observable at a Source revision: its content and its
/// glyph classification.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub idx: usize,
    pub content: Option<char>,
    pub glyph: Option<Glyph>,
}

///
/// The observable outcome of one accepted edit: every Cell whose content or
/// glyph classification differs from the previous revision, described at the
/// revision the edit produced.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub idx: usize,
    pub cells: Vec<Cell>,
}

///
/// A problem with the Expression occupying `start..=end` in the current
/// Source revision. Diagnostics describe accepted user content; they never
/// reject an incomplete or invalid Live Edit.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub start: usize,
    pub end: usize,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CellWrite {
    pub idx: usize,
    pub content: char,
}

/// One MIDI Note On instruction. Issue 04 teaches Source interpretation to
/// populate these; issue 03 establishes their ordered place in every Tick Plan.
pub use lang::PlayCommand;

#[derive(Clone, Debug, PartialEq)]
pub struct TickPlan {
    pub writes: Vec<CellWrite>,
    pub play_commands: Vec<PlayCommand>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TickResult {
    pub plan: TickPlan,
    pub snapshot: String,
    pub changes: Vec<Cell>,
}

///
/// The Cells of one Orca program. The Source is the contents; the Grid it is
/// built from is the shape, and answers every question about that shape.
///
pub struct Source {
    grid: Grid,
    inner: String,
    map: ExpressionMap,
    glyphs: Vec<Option<Glyph>>,
    parsed: Vec<Option<Atoms>>,
    diagnostics: Vec<Option<Diagnostic>>,
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
    ///
    /// A Source of empty Cells, one per Position of `grid`. A Grid has at
    /// least one column and one row, so a Source always has Cells.
    ///
    pub fn new(grid: Grid) -> Self {
        let size = grid.count();
        let inner = SPACE.to_string().repeat(size);
        let map = ExpressionMap::new(grid);

        let glyphs = vec![None; size];
        let parsed = vec![None; size];
        let diagnostics = vec![None; size];

        Self {
            grid,
            inner,
            map,
            glyphs,
            parsed,
            diagnostics,
        }
    }

    ///
    /// Sets the Cell at `idx` and recalculates the affected Expressions.
    /// A space empties the Cell, equivalent to `unset`.
    ///
    /// ```
    /// use console::{grid::Grid, source::Source};
    ///
    /// let mut source = Source::new(Grid::new(10, 10));
    /// source.set(33, "!").unwrap();
    ///
    /// assert_eq!(source.get(33), Some("!".to_string()));
    /// ```
    ///
    pub fn set(&mut self, idx: usize, s: &str) -> Result<Change, SourceError> {
        debug!("set {idx}: {s}");
        self.check_idx(idx)?;
        let byte = Self::check_content(s)?;
        self.check_expression_capacity(idx, byte)?;

        Ok(self.edit(idx, byte))
    }

    ///
    /// Empties the Cell at `idx` and recalculates the affected Expressions.
    ///
    pub fn unset(&mut self, idx: usize) -> Result<Change, SourceError> {
        self.check_idx(idx)?;

        Ok(self.edit(idx, SPACE_BYTE))
    }

    ///
    /// Applies one already-validated edit and describes the Cells it changed
    /// at the new revision.
    ///
    fn edit(&mut self, idx: usize, byte: u8) -> Change {
        let before_cells = self.inner.as_bytes().to_vec();
        let before_glyphs = self.glyphs.clone();

        self.set_source(idx, byte);
        self.rebuild_derived_state();

        let cells = self.changed_cells(&before_cells, &before_glyphs);

        Change { idx, cells }
    }

    ///
    /// The full grid contents at the current revision.
    ///
    pub fn snapshot(&self) -> String {
        self.inner.clone()
    }

    ///
    /// Whether `idx` names a Cell. The Grid decides: an index it cannot
    /// address is one this Source has no Cell for.
    ///
    fn check_idx(&self, idx: usize) -> Result<(), SourceError> {
        match self.grid.position_at(idx) {
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

        let mut prospective = self.inner.as_bytes().to_vec();
        prospective[idx] = byte;

        let mut start = idx;
        while start > 0
            && prospective[start - 1] != SPACE_BYTE
            && self.indices_share_a_row(idx, start - 1)
        {
            start -= 1;
        }
        let mut end = idx;
        while end + 1 < prospective.len()
            && prospective[end + 1] != SPACE_BYTE
            && self.indices_share_a_row(idx, end + 1)
        {
            end += 1;
        }

        let mut expression = String::from_utf8(prospective[start..=end].to_vec())
            .expect("Source Cells contain ASCII");
        match Parser::from(&mut expression).parse() {
            Err(LangError::Syntax(SyntaxError::ExpressionTooLong { .. })) => {
                Err(SourceError::ExpressionTooLong {
                    start,
                    end,
                    capacity: EXP_LEN,
                })
            }
            _ => Ok(()),
        }
    }

    ///
    /// The unique Expression start positions within `from..=to`.
    ///
    fn expression_starts(&self, from: usize, to: usize) -> Vec<usize> {
        let mut starts = Vec::new();

        let mut i = from;
        while i <= to {
            if let Some(range) = self.map.get(i) {
                if !starts.contains(&range.start) {
                    starts.push(range.start);
                }
                i = range.end + 1;
            } else {
                i += 1;
            }
        }

        starts
    }

    pub fn get(&self, idx: usize) -> Option<String> {
        let b = *self.inner.as_bytes().get(idx)?;

        match b {
            SPACE_BYTE => None,
            _ => Some((b as char).to_string()),
        }
    }

    ///
    /// Problems with Expressions in the current revision, in Source order.
    ///
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.iter().flatten().cloned().collect()
    }

    ///
    /// Whether an Expression computes anything.
    ///
    /// An Expression with no Function is a literal — the Interpreter has no
    /// Function to apply, so a Tick produces no result to commit for it. This
    /// is what stops a committed result from feeding itself: the value a Tick
    /// writes is valid Source, but on the next Tick it parses to a literal and
    /// so commits nothing of its own.
    ///
    fn is_computation(atoms: &Atoms) -> bool {
        atoms.iter().any(|a| matches!(a, Atom::Function(_)))
    }

    ///
    /// Runs one Tick: evaluates every Expression against the current Source
    /// snapshot, then commits the resulting Cell changes.
    ///
    /// Every Expression is interpreted before any Cell is written, so a result
    /// committed by one Expression can never become another Expression's input
    /// within the same Tick.
    ///
    pub fn execute(&mut self) -> TickResult {
        let plan = self.plan_tick();
        let changes = self.commit_tick(&plan);

        TickResult {
            plan,
            snapshot: self.snapshot(),
            changes,
        }
    }

    fn plan_tick(&self) -> TickPlan {
        let mut writes = BTreeMap::new();
        let mut play_commands = Vec::new();
        let mut diagnostics = Vec::new();

        for (start, atoms) in self.parsed.iter().enumerate() {
            let Some(atoms) = atoms.as_ref().filter(|atoms| Self::is_computation(atoms)) else {
                continue;
            };
            let range = self
                .map
                .get(start)
                .expect("parsed Expressions have a Source range");

            let result = match Interpreter::execute(atoms) {
                Ok(Interpretation::Cell(Atom::Empty)) => continue,
                Ok(Interpretation::Cell(result)) => result,
                Ok(Interpretation::Play(command)) => {
                    play_commands.push(command);
                    continue;
                }
                Err(error) => {
                    diagnostics.push(Diagnostic {
                        start: range.start,
                        end: range.end,
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            let encoded = result.to_string();
            let origin = self
                .grid
                .position_at(start)
                .expect("parsed holds one entry per Cell");
            let Some(target) = self.grid.below(origin) else {
                diagnostics.push(Diagnostic {
                    start: range.start,
                    end: range.end,
                    message: format!("result {encoded:?} falls below the Source"),
                });
                continue;
            };
            if !self.grid.fits(target, encoded.chars().count()) {
                diagnostics.push(Diagnostic {
                    start: range.start,
                    end: range.end,
                    message: format!("result {encoded:?} crosses the row edge"),
                });
                continue;
            }

            let target_idx = self.grid.index(target);
            for (offset, content) in encoded.chars().enumerate() {
                // Expressions are visited in Source order, so insertion gives
                // a later Expression ownership of only the Cells it overlaps.
                writes.insert(
                    target_idx + offset,
                    CellWrite {
                        idx: target_idx + offset,
                        content,
                    },
                );
            }
        }

        TickPlan {
            writes: writes.into_values().collect(),
            play_commands,
            diagnostics,
        }
    }

    fn commit_tick(&mut self, plan: &TickPlan) -> Vec<Cell> {
        let before_cells = self.inner.as_bytes().to_vec();
        let before_glyphs = self.glyphs.clone();

        // Commit every planned Cell before rebuilding any derived state.
        for write in &plan.writes {
            self.set_source(write.idx, write.content as u8);
        }
        self.rebuild_derived_state();

        self.changed_cells(&before_cells, &before_glyphs)
    }

    fn changed_cells(&self, before_cells: &[u8], before_glyphs: &[Option<Glyph>]) -> Vec<Cell> {
        self.inner
            .bytes()
            .enumerate()
            .filter(|&(idx, byte)| {
                byte != before_cells[idx] || self.glyphs[idx] != before_glyphs[idx]
            })
            .map(|(idx, byte)| Cell {
                idx,
                content: (byte != SPACE_BYTE).then_some(byte as char),
                glyph: self.glyphs[idx],
            })
            .collect()
    }

    fn rebuild_derived_state(&mut self) {
        self.map = ExpressionMap::new(self.grid);
        self.glyphs.fill(None);
        self.parsed.fill(None);
        self.diagnostics.fill(None);

        for (idx, byte) in self.inner.bytes().enumerate() {
            if byte != SPACE_BYTE {
                self.map.set(idx);
            }
        }
        for start in self.expression_starts(0, self.grid.count() - 1) {
            let range = self.map.get(start).expect("Expression starts have a range");
            self.parse_range(range);
        }
        for (idx, byte) in self.inner.bytes().enumerate() {
            if byte != SPACE_BYTE && self.glyphs[idx].is_none() {
                self.glyphs[idx] = Some(Glyph::Char);
            }
        }
    }

    fn get_exp_src(&self, range: Range) -> String {
        // Ranges come from the ExpressionMap, which only holds in-bounds indices
        self.inner[range.start..=range.end].to_owned()
    }

    ///
    /// Writes one already-validated ASCII byte at `idx` without
    /// recalculating Expressions.
    ///
    fn set_source(&mut self, idx: usize, byte: u8) {
        // SAFETY: `byte` is validated as ASCII and `idx` is bounds-checked
        // before any mutation, so the String stays valid UTF-8
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            bytes[idx] = byte;
        }
    }

    fn parse_range(&mut self, exp_range: Range) {
        let start = exp_range.start;
        let end = exp_range.end;
        let mut src = self.get_exp_src(exp_range);
        let mut strict_src = src.clone();
        let diagnostic = Parser::from(&mut strict_src)
            .try_parse()
            .err()
            .map(|error| Diagnostic {
                start,
                end: start + src.len() - 1,
                message: error.to_string(),
            });
        let parsed = Parser::from(&mut src).parse();

        let mut parsed: Expression = match parsed {
            Ok(parsed) => parsed,
            Err(error) => {
                self.parsed[start] = None;
                self.diagnostics[start] = Some(Diagnostic {
                    start,
                    end,
                    message: error.to_string(),
                });
                self.glyphs[start..=end].fill(None);
                return;
            }
        };

        let glyphs = Glyph::to_glyphs(parsed.take_tokens());
        let atoms = parsed.take_atoms();

        self.parsed[start] = Some(atoms);
        self.diagnostics[start] = diagnostic;
        self.glyphs[start..=end].fill(None);
        self.set_glyphs(start, glyphs);
    }

    pub fn get_glyph_at(&self, idx: usize) -> Option<Glyph> {
        self.glyphs.get(idx).copied().flatten()
    }

    fn set_glyphs(&mut self, start: usize, glyphs: Vec<Glyph>) {
        for (i, g) in glyphs.iter().enumerate() {
            let idx = start + i;
            // Operand-slot hints can extend beyond their Expression, but an
            // Expression is horizontal: hints stop at the same row edge.
            if !self.indices_share_a_row(start, idx) {
                break;
            }
            self.glyphs[idx] = Some(*g);
        }
    }

    fn indices_share_a_row(&self, first: usize, second: usize) -> bool {
        match (self.grid.position_at(first), self.grid.position_at(second)) {
            (Some(first), Some(second)) => first.y() == second.y(),
            _ => false,
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
        source::{source::Cell, PlayCommand, Source, SourceError},
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
                .map(|position| cells[self.grid.index(position)])
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

        src.write(0, "++0102");
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
        for (idx, content) in "++0102".chars().enumerate() {
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
                .map(|idx| restored.get_glyph_at(idx))
                .collect::<Vec<_>>(),
            (0..grid.count())
                .map(|idx| source.get_glyph_at(idx))
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
        src.write(0, &"id".repeat(31));
        src.set(62, "i").unwrap();
        let before = src.snapshot();
        let before_diagnostics = src.diagnostics();
        let before_glyphs = (0..src.count())
            .map(|idx| src.get_glyph_at(idx))
            .collect::<Vec<_>>();

        let result = src.set(63, "d");

        assert_eq!(
            result,
            Err(SourceError::ExpressionTooLong {
                start: 0,
                end: 63,
                capacity: 32,
            })
        );
        assert_eq!(src.snapshot(), before);
        assert_eq!(src.diagnostics(), before_diagnostics);
        assert_eq!(
            (0..src.count())
                .map(|idx| src.get_glyph_at(idx))
                .collect::<Vec<_>>(),
            before_glyphs
        );
    }

    #[test]
    fn test_set_returns_change_set_of_affected_cells() {
        trace();

        let mut src = source();

        let change = src.set(0, "+").unwrap();
        assert_eq!(
            change.cells,
            vec![Cell {
                idx: 0,
                content: Some('+'),
                glyph: Some(Glyph::Char),
            }]
        );

        // completing the `++` Function reclassifies Cell 0 in the same change and
        // marks the four empty operand-slot Cells (two 2-wide Numbers) as Number
        let change = src.set(1, "+").unwrap();

        let function = |idx: usize| Cell {
            idx,
            content: Some('+'),
            glyph: Some(Glyph::Function),
        };
        let operand_slot = |idx: usize| Cell {
            idx,
            content: None,
            glyph: Some(Glyph::Number),
        };
        assert_eq!(
            change.cells,
            vec![
                function(0),
                function(1),
                operand_slot(2),
                operand_slot(3),
                operand_slot(4),
                operand_slot(5),
            ]
        );
    }

    #[test]
    fn test_unset_returns_change_set_of_affected_cells() {
        trace();

        let mut src = source();
        src.set(0, "+").unwrap();
        src.set(1, "+").unwrap();

        // deleting half the Function restores its raw character classification and
        // clears the operand-slot hints
        let change = src.unset(1).unwrap();

        let cleared = |idx: usize, content: Option<char>| Cell {
            idx,
            content,
            glyph: None,
        };
        assert_eq!(
            change.cells,
            vec![
                Cell {
                    idx: 0,
                    content: Some('+'),
                    glyph: Some(Glyph::Char),
                },
                cleared(1, None),
                cleared(2, None),
                cleared(3, None),
                cleared(4, None),
                cleared(5, None),
            ]
        );
    }

    #[test]
    fn test_set_near_grid_end_truncates_operand_hints() {
        trace();

        let mut src = source();

        // `++` at the last two Cells wants four more operand-slot glyphs
        // than the Source has room for
        src.set(58, "+").unwrap();
        let change = src.set(59, "+").unwrap();

        assert_eq!(change.cells.len(), 2);
        assert_eq!(src.get_glyph_at(58), Some(Glyph::Function));
        assert_eq!(src.get_glyph_at(59), Some(Glyph::Function));
    }

    #[test]
    fn test_editing_an_operand_slot_hint_restores_the_current_glyphs() {
        let mut src = SourceUnderTest::new(Grid::new(10, 1));
        src.set(0, "+").unwrap();
        src.set(1, "+").unwrap();
        assert_eq!(src.get_glyph_at(5), Some(Glyph::Number));

        let change = src.set(5, "x").unwrap();
        assert_eq!(src.get(5), Some("x".to_string()));
        assert_eq!(src.get_glyph_at(5), Some(Glyph::Char));
        assert_eq!(
            change.cells,
            vec![Cell {
                idx: 5,
                content: Some('x'),
                glyph: Some(Glyph::Char),
            }]
        );

        let change = src.unset(5).unwrap();
        assert_eq!(src.get(5), None);
        assert_eq!(src.get_glyph_at(5), Some(Glyph::Number));
        assert_eq!(
            change.cells,
            vec![Cell {
                idx: 5,
                content: None,
                glyph: Some(Glyph::Number),
            }]
        );
    }

    #[test]
    fn test_editing_an_operand_slot_matches_a_source_rebuilt_from_its_snapshot() {
        let grid = Grid::new(10, 2);
        let mut src = SourceUnderTest::new(grid);
        src.write(0, "++");
        src.write(10, "++0102");
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
                .map(|idx| rebuilt.get_glyph_at(idx))
                .collect::<Vec<_>>(),
            (0..grid.count())
                .map(|idx| src.get_glyph_at(idx))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_operand_hints_and_invalidation_stop_at_the_row_edge() {
        let mut src = SourceUnderTest::new(Grid::new(10, 2));
        src.write(10, "++0102");
        src.set(8, "+").unwrap();

        let change = src.set(9, "+").unwrap();
        assert_eq!(
            change.cells,
            vec![
                Cell {
                    idx: 8,
                    content: Some('+'),
                    glyph: Some(Glyph::Function),
                },
                Cell {
                    idx: 9,
                    content: Some('+'),
                    glyph: Some(Glyph::Function),
                },
            ]
        );
        assert_eq!(src.get_glyph_at(10), Some(Glyph::Function));
        assert_eq!(src.get_glyph_at(11), Some(Glyph::Function));

        let change = src.unset(9).unwrap();
        assert_eq!(
            change.cells,
            vec![
                Cell {
                    idx: 8,
                    content: Some('+'),
                    glyph: Some(Glyph::Char),
                },
                Cell {
                    idx: 9,
                    content: None,
                    glyph: None,
                },
            ]
        );
        assert_eq!(src.get_glyph_at(10), Some(Glyph::Function));
        assert_eq!(src.get_glyph_at(11), Some(Glyph::Function));
    }

    #[test]
    fn test_set_classifies_expression_immediately() {
        trace();

        let mut src = source();

        for (i, c) in "++0101".chars().enumerate() {
            src.set(i, &c.to_string()).unwrap();
        }

        let glyphs: Vec<_> = (0..6).map(|i| src.get_glyph_at(i)).collect();
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
        assert_eq!(src.row(0), "++0101    ");
    }

    #[test]
    fn test_expressions_do_not_join_across_a_row_edge() {
        trace();

        let mut src = source();

        src.set(9, "+").unwrap();
        let change = src.set(10, "+").unwrap();

        assert_eq!(src.get_glyph_at(9), Some(Glyph::Char));
        assert_eq!(src.get_glyph_at(10), Some(Glyph::Char));
        assert_eq!(
            change.cells,
            vec![Cell {
                idx: 10,
                content: Some('+'),
                glyph: Some(Glyph::Char),
            }]
        );
    }

    #[test]
    fn test_diagnostics_follow_the_visible_expression_revision() {
        trace();

        let mut src = source();

        // The incomplete Function remains visible and is diagnosed immediately.
        src.write(0, "++01");
        assert_eq!(src.row(0), "++01      ");
        assert_eq!(src.diagnostics().len(), 1);
        assert_eq!(src.diagnostics()[0].start, 0);
        assert_eq!(src.diagnostics()[0].end, 3);
        assert_eq!(src.diagnostics()[0].message, "expected a token");

        // Completing it removes the cause and therefore the diagnostic in the
        // same accepted edit.
        src.write(4, "02");
        assert!(src.diagnostics().is_empty());

        // A valid prefix does not make trailing content disappear from the
        // Expression's diagnostic state.
        src.set(6, "Z").unwrap();
        assert_eq!(
            src.diagnostics()[0].message,
            "unexpected trailing content \"Z\""
        );
        src.unset(6).unwrap();
        assert!(src.diagnostics().is_empty());

        // Replacing a valid operand with invalid content creates a fresh
        // diagnostic for the current Expression, without rejecting the edit.
        src.set(4, "X").unwrap();
        assert_eq!(src.get(4), Some("X".to_string()));
        assert_eq!(src.diagnostics().len(), 1);
        assert_eq!(
            src.diagnostics()[0].message,
            "expected a number, found \"X2\""
        );

        // Removing the Expression removes its diagnostic rather than leaving
        // stale state attached to empty Cells.
        for idx in 0..6 {
            src.unset(idx).unwrap();
        }
        assert!(src.diagnostics().is_empty());
    }

    #[test]
    fn test_join_discards_stale_expression_state() {
        trace();

        let mut src = source();

        // `++0101` starting at Cell 2; a Tick would write its result one row below
        for (i, c) in "++0101".chars().enumerate() {
            src.set(i + 2, &c.to_string()).unwrap();
        }

        // prepending joins everything into one Expression starting at Cell 0;
        // the old Expression starting at Cell 2 no longer exists
        src.set(1, "d").unwrap();
        src.set(0, "i").unwrap();
        src.execute();

        // `id++0101` commits `02` across Cells 10 and 11. The stale Expression
        // at Cell 2 would commit its own `02` over Cells 12 and 13, so the row
        // is asserted whole: only the joined Expression's result may appear.
        assert_eq!(src.row(1), "02        ");
    }

    #[test]
    fn test_split_reclassifies_and_evaluates_fresh_expressions() {
        trace();

        let mut src = source();

        for (i, c) in "xx++0101".chars().enumerate() {
            src.set(i, &c.to_string()).unwrap();
        }

        // deleting Cell 1 splits off a complete `++0101` Expression at Cell 2
        src.unset(1).unwrap();

        assert_eq!(src.get_glyph_at(1), None);
        let glyphs: Vec<_> = (2..8).map(|i| src.get_glyph_at(i)).collect();
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

        // The README example: `++0102` is 1 + 2, and a Number is two Cells wide
        src.write(0, "++0102");

        let tick = src.execute();

        assert_eq!(src.row(1), "03        ");
        assert_eq!(src.get(10), Some("0".to_string()));
        assert_eq!(src.get(11), Some("3".to_string()));
        assert_eq!(tick.snapshot, src.snapshot());
        assert!(tick.plan.play_commands.is_empty());
        assert_eq!(tick.plan.writes.len(), 2);
        assert_eq!(tick.plan.writes[0].idx, 10);
        assert_eq!(tick.plan.writes[0].content, '0');
        assert_eq!(tick.plan.writes[1].idx, 11);
        assert_eq!(tick.plan.writes[1].content, '3');
        assert_eq!(
            tick.changes,
            vec![
                Cell {
                    idx: 10,
                    content: Some('0'),
                    glyph: Some(Glyph::Char),
                },
                Cell {
                    idx: 11,
                    content: Some('3'),
                    glyph: Some(Glyph::Char),
                },
            ]
        );
    }

    #[test]
    fn test_root_play_function_emits_one_play_command_without_a_cell_write() {
        let mut src = source();
        src.write(0, ">>07FC4");

        let tick = src.execute();

        assert_eq!(
            tick.plan.play_commands,
            vec![PlayCommand {
                channel: 0,
                velocity: 0x7F,
                note: 60,
            }]
        );
        assert!(tick.plan.writes.is_empty());
        assert!(tick.changes.is_empty());
        assert_eq!(src.row(1), "          ");
    }

    #[test]
    fn test_play_preserves_zero_velocity_as_an_explicit_command() {
        let mut src = source();
        src.write(0, ">>F00A0");

        let tick = src.execute();

        assert_eq!(
            tick.plan.play_commands,
            vec![PlayCommand {
                channel: 0xF,
                velocity: 0,
                note: 21,
            }]
        );
        assert!(tick.plan.diagnostics.is_empty());
    }

    #[test]
    fn test_play_velocity_above_midi_range_is_diagnosed() {
        let mut src = source();
        src.write(0, ">>080C4");

        let tick = src.execute();

        assert!(tick.plan.play_commands.is_empty());
        assert!(tick.plan.writes.is_empty());
        assert_eq!(tick.plan.diagnostics.len(), 1);
        assert_eq!(tick.plan.diagnostics[0].start, 0);
        assert_eq!(tick.plan.diagnostics[0].end, 6);
        assert_eq!(
            tick.plan.diagnostics[0].message,
            "Play velocity 80 is outside the MIDI range 00–7F"
        );
    }

    #[test]
    fn test_nested_play_is_diagnosed_without_emitting_a_command() {
        let mut src = SourceUnderTest::new(Grid::new(12, 3));
        src.write(0, "++>>07FC401");

        let tick = src.execute();

        assert!(tick.plan.play_commands.is_empty());
        assert!(tick.plan.writes.is_empty());
        assert_eq!(tick.plan.diagnostics.len(), 1);
        assert_eq!(
            tick.plan.diagnostics[0].message,
            "a Play Function is valid only at the root of an Expression"
        );
    }

    #[test]
    fn test_play_commands_retain_expression_order_and_repeat_on_every_tick() {
        let mut src = source();
        src.write(0, ">>001C4");
        src.write(10, ">>17FA4");

        let first = src.execute();
        let second = src.execute();
        let expected = vec![
            PlayCommand {
                channel: 0,
                velocity: 1,
                note: 60,
            },
            PlayCommand {
                channel: 1,
                velocity: 0x7F,
                note: 69,
            },
        ];

        assert_eq!(first.plan.play_commands, expected);
        assert_eq!(second.plan.play_commands, expected);
        assert!(first.changes.is_empty());
        assert!(second.changes.is_empty());
    }

    #[test]
    fn test_result_above_nine_commits_both_hexadecimal_cells() {
        trace();

        let mut src = source();

        // Numbers are hexadecimal, so 5 + 5 is `0A` — a Cell holding only the
        // leading `0` would be a truncated, and wrong, result
        src.write(0, "++0505");

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
        src.write(50, "++0102");
        src.write(59, "Z");

        let tick = src.execute();

        assert_eq!(src.row(5), "++0102   Z");
        assert_eq!(src.get(59), Some("Z".to_string()));
        assert!(tick.plan.writes.is_empty());
        assert_eq!(tick.plan.diagnostics.len(), 1);
        assert_eq!(tick.plan.diagnostics[0].start, 50);
        assert_eq!(tick.plan.diagnostics[0].end, 55);
        assert_eq!(
            tick.plan.diagnostics[0].message,
            "result \"03\" falls below the Source"
        );
        assert!(tick.changes.is_empty());
    }

    #[test]
    fn test_row_confined_expressions_do_not_produce_a_wrapped_result() {
        trace();

        let mut src = source();

        // The last-column `+` is one incomplete Expression and `+0102` is an
        // invalid Expression in the next row. Neither can produce the `03`
        // that their formerly wrapped `++0102` run produced.
        src.write(9, "++0102");

        src.execute();

        assert_eq!(src.row(1), "+0102     ");
        assert_eq!(src.row(2), "          ");
    }

    #[test]
    fn test_operand_slot_hints_do_not_cross_a_row_edge() {
        trace();

        let mut src = source();

        // The incomplete `++` occupies the last two Cells of row 0. Its four
        // operand-slot hints have no Cells left in that row, so they must not
        // classify Cells at the beginning of row 1.
        src.write(8, "++");

        assert_eq!(src.get_glyph_at(8), Some(Glyph::Function));
        assert_eq!(src.get_glyph_at(9), Some(Glyph::Function));
        for idx in 10..14 {
            assert_eq!(src.get_glyph_at(idx), None);
        }
    }

    #[test]
    fn test_tick_does_not_evaluate_an_expression_across_a_row_edge() {
        trace();

        let mut src = source();

        // This formerly parsed as one wrapped `++0102` Expression. It is now
        // an incomplete `++` followed by a separate literal `0102`, neither
        // of which can produce the old `03` result.
        src.write(8, "++0102");

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
        src.write(0, "++0102");

        src.execute();
        let after_first_tick = src.snapshot();

        // A committed result is not itself a computation, so re-Ticking the
        // same Source re-commits the same Cells and never marches down the grid
        for _ in 0..4 {
            src.execute();
            assert_eq!(src.snapshot(), after_first_tick);
        }

        assert_eq!(src.row(0), "++0102    ");
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

        // `id` with no operand does contain a Function, but the Interpreter
        // has nothing to identify: the empty result is the absence of a value
        // and must never reach a Cell
        src.write(0, "id");

        src.execute();

        assert_eq!(src.row(1), "          ");
    }

    #[test]
    fn test_function_over_a_literal_still_commits_its_result() {
        trace();

        let mut src = source();

        // `id1` is a computation — suppressing literals must not suppress a
        // Function applied to one
        src.write(0, "id1");

        src.execute();

        assert_eq!(src.row(1), "1         ");
    }

    #[test]
    fn test_failed_expression_suppresses_only_its_own_result() {
        trace();

        let mut src = source();

        // `++` with no operands fails to evaluate; the `++0102` beside it is
        // unrelated and must still commit its `03` in the same Tick
        src.write(0, "++");
        src.write(3, "++0102");

        let tick = src.execute();

        assert_eq!(src.row(1), "   03     ");
        assert_eq!(tick.plan.writes.len(), 2);
        assert_eq!(tick.plan.diagnostics.len(), 1);
        assert_eq!(tick.plan.diagnostics[0].start, 0);
        assert_eq!(tick.plan.diagnostics[0].end, 1);
        assert_eq!(
            tick.plan.diagnostics[0].message,
            "expected a number, found \"_\""
        );
    }

    #[test]
    fn test_every_expression_evaluates_from_the_same_pre_tick_snapshot() {
        trace();

        let mut src = source();

        // The row 0 Expression commits `02` over the first two Cells of the
        // row 1 Expression. Row 1 must still evaluate the `++0304` that was
        // there when the Tick began, not the `020304` the write leaves behind.
        src.write(0, "++0101");
        src.write(10, "++0304");

        src.execute();

        assert_eq!(src.row(1), "020304    ");
        assert_eq!(src.row(2), "07        ");
    }
}
