use lang::{Atoms, Expression, Interpreter, Parser};
use std::fmt;
use tracing::debug;

use crate::glyph::Glyph;
use crate::opts::Opts;

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

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Source {
    opts: Opts,
    inner: String,
    map: ExpressionMap,
    glyphs: Vec<Option<Glyph>>,
    parsed: Vec<Option<Atoms>>,
}

impl Source {
    pub fn new(opts: Opts) -> Self {
        let size = opts.count();
        let inner = SPACE.to_string().repeat(size);
        let map = ExpressionMap::new(size);

        let glyphs = vec![None; size];
        let parsed = vec![None; size];

        Self {
            opts,
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
    /// use console::{opts::Opts, source::Source};
    ///
    /// let mut source = Source::new(Opts::new(10, 10));
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

    fn check_idx(&self, idx: usize) -> Result<(), SourceError> {
        let len = self.opts.count();
        if idx >= len {
            return Err(SourceError::OutOfRange { idx, len });
        }
        Ok(())
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
        let last = self.opts.count() - 1;
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

    pub fn execute(&mut self) {
        let results: Vec<_> = self
            .parsed
            .iter()
            .map(|o| o.as_ref().map(Interpreter::execute))
            .collect();

        debug!("{:?}", results);

        for (idx, o) in results.iter().enumerate() {
            let idx = (idx + self.opts.cols).clamp(0, self.opts.count() - 1);

            if let Some(Ok(a)) = o {
                // A result may encode to more than one character; only the first
                // Cell's worth is written for now (multi-Cell commits are Tick Plan work)
                if let Some(c) = a.to_string().chars().next() {
                    if let Err(e) = self.set(idx, &c.to_string()) {
                        debug!("discarded result at {idx}: {e}");
                    }
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
        opts::Opts,
        source::{source::Cell, Source, SourceError},
        test::trace,
    };

    fn source() -> Source {
        Source::new(Opts::new(10, 10))
    }

    #[test]
    fn test_set_rejects_out_of_range_without_mutation() {
        trace();

        let mut src = source();
        let before = src.snapshot();

        let err = src.set(100, "x").unwrap_err();

        assert_eq!(err, SourceError::OutOfRange { idx: 100, len: 100 });
        assert_eq!(src.snapshot(), before);
    }

    #[test]
    fn test_unset_rejects_out_of_range_without_mutation() {
        trace();

        let mut src = source();
        let before = src.snapshot();

        let err = src.unset(200).unwrap_err();

        assert_eq!(err, SourceError::OutOfRange { idx: 200, len: 100 });
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
        src.set(98, "+").unwrap();
        let change = src.set(99, "+").unwrap();

        assert_eq!(change.cells.len(), 2);
        assert_eq!(src.get_glyph_at(98), Some(Glyph::Function));
        assert_eq!(src.get_glyph_at(99), Some(Glyph::Function));
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

        // prepending at Cell 1 joins into one Expression starting at Cell 1;
        // the old Expression starting at Cell 2 no longer exists
        src.set(1, "+").unwrap();
        src.execute();

        // no result may appear below Cell 2 — that would be the stale Expression
        assert_eq!(src.get(12), None);
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

        // the split-off Expression evaluates on the next Tick
        src.execute();
        assert_eq!(src.get(12), Some("0".to_string()));
    }

    #[test]
    fn test_set_space_empties_cell() {
        trace();

        let mut src = source();
        src.set(5, "x").unwrap();

        src.set(5, " ").unwrap();

        assert_eq!(src.get(5), None);
        assert_eq!(src.snapshot(), " ".repeat(100));
    }

}
