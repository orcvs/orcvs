use lang::{Atom, Atoms, Expression, Interpreter, Parser};
use std::fmt;
use tracing::{debug, warn};

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
/// The Cells of one Orca program. The Source is the contents; the Grid it is
/// built from is the shape, and answers every question about that shape.
///
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Source {
    grid: Grid,
    inner: String,
    map: ExpressionMap,
    glyphs: Vec<Option<Glyph>>,
    parsed: Vec<Option<Atoms>>,
}

impl Source {
    ///
    /// A Source of empty Cells, one per Position of `grid`. A Grid has at
    /// least one column and one row, so a Source always has Cells.
    ///
    pub fn new(grid: Grid) -> Self {
        let size = grid.count();
        let inner = SPACE.to_string().repeat(size);
        let map = ExpressionMap::new(size);

        let glyphs = vec![None; size];
        let parsed = vec![None; size];

        Self {
            grid,
            inner,
            map,
            glyphs,
            parsed,
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

        let span = self.unparse_around(idx);
        self.set_source(idx, byte);
        if byte == SPACE_BYTE {
            self.map.unset(idx);
        } else {
            self.map.set(idx);
        }
        self.reparse_span(span);

        let cells = self
            .inner
            .bytes()
            .enumerate()
            .filter(|&(i, b)| b != before_cells[i] || self.glyphs[i] != before_glyphs[i])
            .map(|(i, b)| Cell {
                idx: i,
                content: (b != SPACE_BYTE).then_some(b as char),
                glyph: self.glyphs[i],
            })
            .collect();

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

    ///
    /// Clears the derived state (glyphs, parsed Atoms) of every Expression
    /// adjacent to `idx`, and returns the span of Cells that must be
    /// reparsed once the edit is applied.
    ///
    /// An edit at `idx` can join, split, extend, or shrink the Expressions
    /// beside it, so all of them are invalidated before the map changes.
    ///
    fn unparse_around(&mut self, idx: usize) -> Range {
        // A Grid has at least one Cell, so the last index is not an underflow
        let last = self.grid.count() - 1;
        let mut span = Range {
            start: idx.saturating_sub(1),
            end: (idx + 1).min(last),
        };

        for start in self.expression_starts(span.start, span.end) {
            let cleared_to = self.unset_glyphs(start);
            self.parsed[start] = None;

            span.start = span.start.min(start);
            span.end = span.end.max(cleared_to);
        }

        span
    }

    ///
    /// Reparses every Expression that intersects `span`, restoring glyphs and
    /// parsed Atoms for the current revision. This covers the edited
    /// Expression and any neighbour whose derived state was cleared with it.
    ///
    fn reparse_span(&mut self, span: Range) {
        for start in self.expression_starts(span.start, span.end) {
            if let Some(range) = self.map.get(start) {
                self.parse_range(range);
            }
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
    pub fn execute(&mut self) {
        // The whole snapshot is interpreted before the first write
        let results: Vec<_> = self
            .parsed
            .iter()
            .map(|o| {
                o.as_ref()
                    .filter(|atoms| Self::is_computation(atoms))
                    .map(Interpreter::execute)
            })
            .collect();

        for (start, o) in results.iter().enumerate() {
            let a = match o {
                // Not the start of an Expression, or an Expression that
                // computes nothing
                None => continue,
                // A failing Expression suppresses only its own result and
                // leaves every other result in this Tick untouched.
                // TODO(issue 03): this diagnostic belongs in the Tick Plan so
                // a failure is reportable rather than only loggable.
                // See .scratch/source-playback-engine/issues/03-commit-atomic-cell-results-through-tick-plans.md
                Some(Err(e)) => {
                    warn!("Expression at {start} failed to evaluate: {e}");
                    continue;
                }
                Some(Ok(a)) => a,
            };

            // `Atom::Empty` is the absence of a result, not a value, so it is
            // never written — an Expression whose operands are missing leaves
            // the Cells below it alone
            if matches!(a, Atom::Empty) {
                debug!("Expression at {start} produced no result");
                continue;
            }

            let encoded = a.to_string();

            // Total: `parsed` holds one entry per Cell, so `start` is an index
            // this Grid addresses
            let origin = self
                .grid
                .position_at(start)
                .expect("parsed holds one entry per Cell");

            // An Expression writes its result into the row below its own
            // start. In the bottom row there is no such row, so the result
            // falls outside the Source and is discarded, never clamped onto a
            // Cell the Expression does not own.
            // TODO(issue 03): a discard is a diagnostic the Tick Plan should
            // report rather than only log.
            // See .scratch/source-playback-engine/issues/03-commit-atomic-cell-results-through-tick-plans.md
            let Some(target) = self.grid.below(origin) else {
                debug!("discarded {encoded:?} from Expression at {start}: below the bottom row");
                continue;
            };

            // An Expression never wraps across rows, and neither does its
            // result: a value too wide for the Cells left in the row is
            // discarded whole rather than split across two rows
            if !self.grid.fits(target, encoded.chars().count()) {
                debug!(
                    "discarded {encoded:?} from Expression at {start}: does not fit before the row edge"
                );
                continue;
            }

            // A result is written as its complete encoding, one Cell per
            // character, left to right from the target Cell
            let idx = self.grid.index(target);
            for (i, c) in encoded.chars().enumerate() {
                if let Err(e) = self.set(idx + i, &c.to_string()) {
                    debug!("discarded result at {}: {e}", idx + i);
                }
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
        let mut src = self.get_exp_src(exp_range);
        let mut parsed: Expression = Parser::from(&mut src).parse();

        let glyphs = Glyph::to_glyphs(parsed.take_tokens());
        let atoms = parsed.take_atoms();

        self.parsed[start] = Some(atoms);
        self.set_glyphs(start, glyphs);
    }

    pub fn get_glyph_at(&self, idx: usize) -> Option<Glyph> {
        self.glyphs.get(idx).copied().flatten()
    }

    fn set_glyphs(&mut self, start: usize, glyphs: Vec<Glyph>) {
        for (i, g) in glyphs.iter().enumerate() {
            let pos = start + i;
            // Operand-slot hints can extend past the last Cell; drop those
            if pos >= self.glyphs.len() {
                break;
            }
            self.glyphs[pos] = Some(*g);
        }
    }

    ///
    /// Clears the contiguous run of glyphs beginning at `start` and returns
    /// the last Cell cleared. Operand-slot hints painted past an Expression's
    /// end are contiguous with it, so they are cleared by the same walk.
    ///
    fn unset_glyphs(&mut self, start: usize) -> usize {
        let mut last = start;

        for i in start..self.glyphs.len() {
            match self.glyphs[i] {
                Some(_) => {
                    self.glyphs[i] = None;
                    last = i;
                }
                None => break,
            };
        }

        last
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod test {

    use crate::{
        glyph::Glyph,
        grid::Grid,
        source::{source::Cell, Source, SourceError},
        test::trace,
    };

    ///
    /// Rectangular on purpose: a Source that derived a dimension of its own,
    /// or read the two the wrong way round, addresses different Cells here
    /// than it does on a square one.
    ///
    fn source() -> Source {
        Source::new(Grid::new(10, 6))
    }

    ///
    /// Types `s` into consecutive Cells starting at `idx`, one accepted edit
    /// per Cell, exactly as a user would.
    ///
    fn write(src: &mut Source, idx: usize, s: &str) {
        for (i, c) in s.chars().enumerate() {
            src.set(idx + i, &c.to_string()).unwrap();
        }
    }

    ///
    /// The Cells of row `row`, spaces included.
    ///
    fn row(src: &Source, row: usize) -> String {
        src.snapshot()[row * 10..(row + 1) * 10].to_string()
    }

    #[test]
    fn test_set_rejects_out_of_range_without_mutation() {
        trace();

        let mut src = source();
        let before = src.snapshot();

        let err = src.set(60, "x").unwrap_err();

        assert_eq!(err, SourceError::OutOfRange { idx: 60, len: 60 });
        assert_eq!(src.snapshot(), before);
    }

    #[test]
    fn test_unset_rejects_out_of_range_without_mutation() {
        trace();

        let mut src = source();
        let before = src.snapshot();

        let err = src.unset(200).unwrap_err();

        assert_eq!(err, SourceError::OutOfRange { idx: 200, len: 60 });
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
    fn test_set_returns_change_set_of_affected_cells() {
        trace();

        let mut src = source();

        let change = src.set(0, "+").unwrap();
        assert_eq!(
            change.cells,
            vec![Cell {
                idx: 0,
                content: Some('+'),
                glyph: None,
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

        // deleting half the Function unclassifies it and clears the operand-slot hints
        let change = src.unset(1).unwrap();

        let cleared = |idx: usize, content: Option<char>| Cell {
            idx,
            content,
            glyph: None,
        };
        assert_eq!(
            change.cells,
            vec![
                cleared(0, Some('+')),
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
        assert_eq!(&src.snapshot()[0..10], "++0101    ");
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
        assert_eq!(row(&src, 1), "02        ");
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
        assert_eq!(row(&src, 1), "  02      ");
    }

    #[test]
    fn test_set_space_empties_cell() {
        trace();

        let mut src = source();
        src.set(5, "x").unwrap();

        src.set(5, " ").unwrap();

        assert_eq!(src.get(5), None);
        assert_eq!(src.snapshot(), " ".repeat(60));
    }

    #[test]
    fn test_result_commits_its_complete_encoding_across_consecutive_cells() {
        trace();

        let mut src = source();

        // The README example: `++0102` is 1 + 2, and a Number is two Cells wide
        write(&mut src, 0, "++0102");

        src.execute();

        assert_eq!(row(&src, 1), "03        ");
        assert_eq!(src.get(10), Some("0".to_string()));
        assert_eq!(src.get(11), Some("3".to_string()));
    }

    #[test]
    fn test_result_above_nine_commits_both_hexadecimal_cells() {
        trace();

        let mut src = source();

        // Numbers are hexadecimal, so 5 + 5 is `0A` — a Cell holding only the
        // leading `0` would be a truncated, and wrong, result
        write(&mut src, 0, "++0505");

        src.execute();

        assert_eq!(row(&src, 1), "0A        ");
    }

    #[test]
    fn test_result_below_the_bottom_row_is_discarded() {
        trace();

        let mut src = source();

        // An Expression in the bottom row has nowhere to write: its result
        // falls outside the Source and is discarded, never clamped onto a Cell
        // the user owns
        write(&mut src, 50, "++0102");
        write(&mut src, 59, "Z");

        src.execute();

        assert_eq!(row(&src, 5), "++0102   Z");
        assert_eq!(src.get(59), Some("Z".to_string()));
    }

    #[test]
    fn test_result_that_cannot_fit_before_the_row_edge_is_discarded() {
        trace();

        let mut src = source();

        // An Expression starting in the last column: its two-Cell result would
        // have to wrap onto the following row, so the whole result is discarded
        // rather than split. (The Expression itself still wraps — issue 02.)
        write(&mut src, 9, "++0102");

        src.execute();

        assert_eq!(row(&src, 1), "+0102     ");
        assert_eq!(row(&src, 2), "          ");
    }

    #[test]
    fn test_expression_without_a_function_commits_nothing() {
        trace();

        let mut src = source();

        // A bare Number is not a computation: the Interpreter has nothing to
        // apply, so the Expression has no result to commit
        write(&mut src, 0, "03");

        src.execute();

        assert_eq!(row(&src, 1), "          ");
    }

    #[test]
    fn test_repeated_ticks_do_not_cascade_results_down_the_grid() {
        trace();

        let mut src = source();
        write(&mut src, 0, "++0102");

        src.execute();
        let after_first_tick = src.snapshot();

        // A committed result is not itself a computation, so re-Ticking the
        // same Source re-commits the same Cells and never marches down the grid
        for _ in 0..4 {
            src.execute();
            assert_eq!(src.snapshot(), after_first_tick);
        }

        assert_eq!(row(&src, 0), "++0102    ");
        assert_eq!(row(&src, 1), "03        ");
        assert_eq!(&src.snapshot()[20..], &" ".repeat(40));
    }

    #[test]
    fn test_empty_result_of_an_incomplete_function_commits_nothing() {
        trace();

        let mut src = source();

        // `id` with no operand does contain a Function, but the Interpreter
        // has nothing to identify: the empty result is the absence of a value
        // and must never reach a Cell
        write(&mut src, 0, "id");

        src.execute();

        assert_eq!(row(&src, 1), "          ");
    }

    #[test]
    fn test_function_over_a_literal_still_commits_its_result() {
        trace();

        let mut src = source();

        // `id1` is a computation — suppressing literals must not suppress a
        // Function applied to one
        write(&mut src, 0, "id1");

        src.execute();

        assert_eq!(row(&src, 1), "1         ");
    }

    #[test]
    fn test_failed_expression_suppresses_only_its_own_result() {
        trace();

        let mut src = source();

        // `++` with no operands fails to evaluate; the `++0102` beside it is
        // unrelated and must still commit its `03` in the same Tick
        write(&mut src, 0, "++");
        write(&mut src, 3, "++0102");

        src.execute();

        assert_eq!(row(&src, 1), "   03     ");
    }

    #[test]
    fn test_every_expression_evaluates_from_the_same_pre_tick_snapshot() {
        trace();

        let mut src = source();

        // The row 0 Expression commits `02` over the first two Cells of the
        // row 1 Expression. Row 1 must still evaluate the `++0304` that was
        // there when the Tick began, not the `020304` the write leaves behind.
        write(&mut src, 0, "++0101");
        write(&mut src, 10, "++0304");

        src.execute();

        assert_eq!(row(&src, 1), "020304    ");
        assert_eq!(row(&src, 2), "07        ");
    }
}
