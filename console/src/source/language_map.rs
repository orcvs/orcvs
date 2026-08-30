use lang::{Activation, Atom, Atoms, Expression, Parser, Token};

use crate::{glyph::Glyph, grid::Grid};

use super::Diagnostic;

const SPACE_BYTE: u8 = b' ';

/// The semantic information derived from one complete Source revision.
///
/// This is the single owner of expression extents, parsed expressions, Glyph
/// classifications, and diagnostics. It deliberately exposes only the
/// semantics the current parser can establish; Language Unit queries described
/// by ADR 0018 belong here once their lexical prerequisites are implemented.
pub(super) struct LanguageMap {
    expressions: Vec<ExpressionEntry>,
    glyphs: Vec<Option<Glyph>>,
}

pub(super) struct ExpressionEntry {
    range: Range,
    atoms: Option<Atoms>,
    diagnostic: Option<Diagnostic>,
}

impl ExpressionEntry {
    pub(super) fn range(&self) -> Range {
        self.range
    }

    pub(super) fn atoms(&self) -> Option<&Atoms> {
        self.atoms.as_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Range {
    start: usize,
    end: usize,
}

impl Range {
    fn new(start: usize, end: usize) -> Self {
        assert!(
            start <= end,
            "Expression range start must not exceed its end"
        );
        Self { start, end }
    }

    pub(super) fn start(self) -> usize {
        self.start
    }

    pub(super) fn end(self) -> usize {
        self.end
    }
}

impl LanguageMap {
    pub(super) fn build(grid: Grid, bytes: &[u8]) -> Self {
        let expression_map = ExpressionMap::build(grid, bytes);
        let ranges = expression_map.ranges().collect::<Vec<_>>();
        let mut map = Self {
            expressions: Vec::new(),
            glyphs: vec![None; bytes.len()],
        };

        for range in ranges {
            map.parse_range(grid, bytes, range);
        }
        for (idx, byte) in bytes.iter().copied().enumerate() {
            if byte != SPACE_BYTE && map.glyphs[idx].is_none() {
                map.glyphs[idx] = Some(Glyph::Char);
            }
        }

        map
    }

    /// Answers the Expression extent that would contain `idx` after replacing
    /// that Cell with `byte`, without scanning any other row.
    pub(super) fn prospective_expression_range(
        grid: Grid,
        bytes: &[u8],
        idx: usize,
        byte: u8,
    ) -> Option<Range> {
        ExpressionMap::prospective_range(grid, bytes, idx, byte)
    }

    pub(super) fn expressions(&self) -> impl Iterator<Item = &ExpressionEntry> {
        self.expressions.iter()
    }

    pub(super) fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.expressions
            .iter()
            .filter_map(|expression| expression.diagnostic.as_ref())
    }

    pub(super) fn glyph_at(&self, idx: usize) -> Option<Glyph> {
        self.glyphs.get(idx).copied().flatten()
    }

    pub(super) fn presentation_differs_at(&self, previous: &Self, idx: usize) -> bool {
        debug_assert_eq!(self.glyphs.len(), previous.glyphs.len());
        self.glyphs[idx] != previous.glyphs[idx]
    }

    fn parse_range(&mut self, grid: Grid, bytes: &[u8], range: Range) {
        let start = range.start();
        let end = range.end();
        // A later Expression owns its occupied Cells over any operand-slot
        // hints emitted by an earlier Expression.
        self.glyphs[start..=end].fill(None);
        let mut source =
            String::from_utf8(bytes[start..=end].to_vec()).expect("Source Cells contain ASCII");
        let standalone_run = parse_standalone_run(&source);
        let strict_diagnostic = if standalone_run.is_some() {
            None
        } else {
            let mut strict_source = source.clone();
            Parser::from(&mut strict_source)
                .try_parse()
                .err()
                .map(|error| Diagnostic {
                    start,
                    end,
                    message: error.to_string(),
                })
        };

        let parsed = match standalone_run.map_or_else(|| Parser::from(&mut source).parse(), Ok) {
            Ok(parsed) => parsed,
            Err(error) => {
                self.expressions.push(ExpressionEntry {
                    range,
                    atoms: None,
                    diagnostic: Some(Diagnostic {
                        start,
                        end,
                        message: error.to_string(),
                    }),
                });
                return;
            }
        };

        let (atoms, glyphs) = expression_parts(parsed);
        self.set_glyphs(grid, start, glyphs);
        self.expressions.push(ExpressionEntry {
            range,
            atoms: Some(atoms),
            diagnostic: strict_diagnostic,
        });
    }

    fn set_glyphs(&mut self, grid: Grid, start: usize, glyphs: Vec<Glyph>) {
        for (offset, glyph) in glyphs.into_iter().enumerate() {
            let idx = start + offset;
            // Operand-slot hints can extend beyond their Expression, but an
            // Expression is horizontal: hints stop at the same row edge.
            if !indices_share_a_row(grid, start, idx) {
                break;
            }
            self.glyphs[idx] = Some(glyph);
        }
    }
}

fn parse_standalone_run(source: &str) -> Option<Expression> {
    if source.len() < 4 || !source.len().is_multiple_of(2) {
        return None;
    }

    let mut expression = Expression::new();
    for spelling in source.as_bytes().as_chunks::<2>().0 {
        let (token, atom) = match spelling {
            b"**" => (Token::Bang, Atom::Bang),
            b">>" => (Token::Activation, Atom::Activation(Activation::East)),
            _ => return None,
        };
        expression.add(token, atom).ok()?;
    }
    Some(expression)
}

fn expression_parts(mut expression: Expression) -> (Atoms, Vec<Glyph>) {
    let glyphs = Glyph::to_glyphs(expression.take_tokens());
    let atoms = expression.take_atoms();
    (atoms, glyphs)
}

fn indices_share_a_row(grid: Grid, first: usize, second: usize) -> bool {
    match (grid.position_at(first), grid.position_at(second)) {
        (Some(first), Some(second)) => first.y() == second.y(),
        _ => false,
    }
}

#[derive(Debug)]
struct ExpressionMap {
    inner: Vec<Option<Range>>,
}

impl ExpressionMap {
    fn build(grid: Grid, bytes: &[u8]) -> Self {
        assert_eq!(
            bytes.len(),
            grid.count(),
            "LanguageMap Source length must match its Grid"
        );

        let mut inner = Vec::with_capacity(bytes.len());
        for (row_number, row) in bytes.chunks_exact(grid.cols()).enumerate() {
            inner.extend(row_extents(row_number * grid.cols(), row));
        }
        Self { inner }
    }

    fn prospective_range(grid: Grid, bytes: &[u8], idx: usize, byte: u8) -> Option<Range> {
        assert_eq!(
            bytes.len(),
            grid.count(),
            "LanguageMap Source length must match its Grid"
        );
        assert!(
            idx < bytes.len(),
            "prospective Cell must belong to the Source"
        );

        let cols = grid.cols();
        let row_start = (idx / cols) * cols;
        let mut row = bytes[row_start..row_start + cols].to_vec();
        let row_idx = idx - row_start;
        row[row_idx] = byte;
        row_extents(row_start, &row)[row_idx]
    }

    fn ranges(&self) -> impl Iterator<Item = Range> + '_ {
        self.inner
            .iter()
            .enumerate()
            .filter_map(|(idx, range)| range.filter(|range| range.start() == idx))
    }
}

/// The one rule for Expression extent. A future Comment implementation belongs
/// here and must exclude the entire `#` suffix of a row from Expressions.
fn row_extents(row_start: usize, row: &[u8]) -> Vec<Option<Range>> {
    let mut extents = vec![None; row.len()];
    let mut local_start = 0;

    while local_start < row.len() {
        if row[local_start] == SPACE_BYTE {
            local_start += 1;
            continue;
        }

        let local_end = row[local_start..]
            .iter()
            .position(|&byte| byte == SPACE_BYTE)
            .map_or(row.len() - 1, |offset| local_start + offset - 1);
        let range = Range::new(row_start + local_start, row_start + local_end);
        extents[local_start..=local_end].fill(Some(range));
        local_start = local_end + 1;
    }
    extents
}

#[cfg(test)]
mod tests {
    use crate::{glyph::Glyph, grid::Grid};

    use lang::{Activation, Atom};

    use super::{ExpressionMap, LanguageMap, Range};

    fn assert_range(map: &ExpressionMap, start: usize, end: usize) {
        for idx in start..=end {
            assert_eq!(map.inner[idx], Some(Range::new(start, end)));
        }
    }

    #[test]
    fn build_leaves_empty_rows_without_expressions() {
        let map = ExpressionMap::build(Grid::new(5, 1), b"     ");
        assert!(map.inner.iter().all(Option::is_none));
    }

    #[test]
    fn build_maps_every_cell_in_one_run_to_its_inclusive_extent() {
        let map = ExpressionMap::build(Grid::new(5, 1), b" id1 ");
        assert_eq!(map.inner[0], None);
        assert_range(&map, 1, 3);
        assert_eq!(map.inner[4], None);
    }

    #[test]
    fn build_separates_multiple_runs_in_one_row() {
        let map = ExpressionMap::build(Grid::new(8, 1), b"id  .+  ");
        assert_range(&map, 0, 1);
        assert_eq!(map.inner[2], None);
        assert_eq!(map.inner[3], None);
        assert_range(&map, 4, 5);
        assert_eq!(map.inner[6], None);
        assert_eq!(map.inner[7], None);
    }

    #[test]
    fn build_keeps_edge_touching_runs_inside_their_rows() {
        let map = ExpressionMap::build(Grid::new(4, 2), b"  id.+  ");
        assert_range(&map, 2, 3);
        assert_range(&map, 4, 5);
        assert_ne!(map.inner[3], map.inner[4]);
    }

    #[test]
    fn prospective_range_scans_only_the_edited_row() {
        let grid = Grid::new(5, 2);
        let bytes = b"id   .+   ";
        assert_eq!(
            ExpressionMap::prospective_range(grid, bytes, 2, b'1'),
            Some(Range::new(0, 2))
        );
        assert_eq!(
            ExpressionMap::prospective_range(grid, bytes, 7, b'0'),
            Some(Range::new(5, 7))
        );
    }

    #[test]
    #[should_panic(expected = "LanguageMap Source length must match its Grid")]
    fn build_rejects_source_content_with_the_wrong_length() {
        let _ = ExpressionMap::build(Grid::new(5, 1), b"    ");
    }

    #[test]
    fn language_map_builds_cohesive_expression_state() {
        let map = LanguageMap::build(Grid::new(8, 1), b".+0102 x");
        let expressions = map.expressions().collect::<Vec<_>>();
        assert_eq!(expressions.len(), 2);
        assert_eq!(expressions[0].range(), Range::new(0, 5));
        assert!(expressions[0].atoms().is_some());
        assert_eq!(map.glyph_at(0), Some(Glyph::Function));
        assert_eq!(map.glyph_at(7), Some(Glyph::Char));
        assert_eq!(map.diagnostics().count(), 1);
    }

    #[test]
    fn language_map_partitions_adjacent_standalone_units() {
        let bangs = LanguageMap::build(Grid::new(4, 1), b"****");
        let activations = LanguageMap::build(Grid::new(4, 1), b">>>>");

        assert_eq!(
            bangs
                .expressions()
                .next()
                .unwrap()
                .atoms()
                .unwrap()
                .as_slice(),
            &[Atom::Bang, Atom::Bang]
        );
        assert!((0..4).all(|idx| bangs.glyph_at(idx) == Some(Glyph::Bang)));
        assert_eq!(bangs.diagnostics().count(), 0);
        assert_eq!(
            activations
                .expressions()
                .next()
                .unwrap()
                .atoms()
                .unwrap()
                .as_slice(),
            &[
                Atom::Activation(Activation::East),
                Atom::Activation(Activation::East)
            ]
        );
        assert_eq!(activations.diagnostics().count(), 0);
    }

    #[test]
    fn presentation_comparison_stays_behind_the_language_map() {
        let grid = Grid::new(5, 1);
        let before = LanguageMap::build(grid, b"x    ");
        let after = LanguageMap::build(grid, b"x y  ");
        assert!(!after.presentation_differs_at(&before, 0));
        assert!(after.presentation_differs_at(&before, 2));
    }
}
