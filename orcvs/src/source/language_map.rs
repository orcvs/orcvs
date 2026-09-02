use lang::{
    Activation, Atom, Atoms, Expression, Function, Parser, SourceAnalysis, Token, to_atom_note,
    to_atom_num,
};

use crate::{
    glyph::Glyph,
    grid::{Grid, Position},
};

use super::Diagnostic;

const SPACE_BYTE: u8 = b' ';

/// The semantic information derived from one complete Source revision.
///
/// This is the single owner of expression extents, parsed expressions, Glyph
/// classifications, and diagnostics. It deliberately exposes only the
/// semantics the current parser and row-local partition can establish.
#[derive(Clone)]
pub struct LanguageMap {
    grid: Grid,
    units: Vec<LanguageUnit>,
    expressions: Vec<ExpressionEntry>,
    glyphs: Vec<Option<Glyph>>,
    lexical_diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LanguageUnitKind {
    OperandLiteral,
    Atom(Atom),
    Function(Function),
    Bang,
    Activation(Activation),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanguageUnit {
    kind: LanguageUnitKind,
    anchor: Position,
    footprint: Footprint,
}

impl LanguageUnit {
    pub fn kind(&self) -> LanguageUnitKind {
        self.kind
    }

    pub fn anchor(&self) -> Position {
        self.anchor
    }

    pub fn footprint(&self) -> &Footprint {
        &self.footprint
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Footprint {
    positions: Vec<Position>,
}

impl Footprint {
    pub fn positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.positions.iter().copied()
    }

    pub(super) fn from_indices(grid: Grid, indices: impl IntoIterator<Item = usize>) -> Self {
        Self {
            positions: indices
                .into_iter()
                .filter_map(|idx| grid.position_at(idx))
                .collect(),
        }
    }
}

#[derive(Clone)]
pub struct ExpressionEntry {
    atoms: Option<Atoms>,
    diagnostic: Option<Diagnostic>,
    root: Option<Position>,
    footprint: Footprint,
    units: Vec<LanguageUnit>,
}

impl ExpressionEntry {
    /// The first Function anchor when this is a complete executable Expression.
    pub fn root(&self) -> Option<Position> {
        self.root
    }

    pub fn footprint(&self) -> &Footprint {
        &self.footprint
    }

    pub fn units(&self) -> impl Iterator<Item = &LanguageUnit> {
        self.units.iter()
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
    /// Derives the semantic view of one complete Source revision.
    ///
    /// Returns `None` when `source` is not exactly one printable-ASCII Cell per
    /// Position in `grid`.
    pub fn derive(grid: Grid, source: &str) -> Option<Self> {
        (source.len() == grid.count() && source.bytes().all(|byte| (0x20..=0x7e).contains(&byte)))
            .then(|| Self::build(grid, source.as_bytes()))
    }

    pub(super) fn build(grid: Grid, bytes: &[u8]) -> Self {
        assert_eq!(
            bytes.len(),
            grid.count(),
            "LanguageMap Source length must match its Grid"
        );
        let (units, lexical_diagnostics) = partition_units(grid, bytes);
        let expression_map = ExpressionMap::build(grid, bytes);
        let ranges = expression_map.ranges().collect::<Vec<_>>();
        let mut map = Self {
            grid,
            units,
            expressions: Vec::new(),
            glyphs: vec![None; bytes.len()],
            lexical_diagnostics,
        };

        for range in ranges {
            map.parse_range(grid, bytes, range);
        }
        for expression in &map.expressions {
            for parsed in &expression.units {
                if let Some(unit) = map
                    .units
                    .iter_mut()
                    .find(|unit| unit.anchor == parsed.anchor)
                {
                    unit.kind = parsed.kind;
                }
            }
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

    pub fn expressions(&self) -> impl Iterator<Item = &ExpressionEntry> {
        self.expressions.iter()
    }

    pub fn units(&self) -> impl Iterator<Item = &LanguageUnit> {
        self.units.iter()
    }

    /// Whether a Source-resident Bang activates the root anchored at `root`.
    ///
    /// An ordinary root Expression is inert until a Bang activates it, and the
    /// geometry deciding that is a question about where things sit rather than
    /// about what any Function means. Keeping it here is what lets Source
    /// interpretation stay a question about Atoms: the Interpreter is never
    /// told where anything sits, and the MIDI path never learns what a Bang is.
    ///
    /// Bangs are partitioned independently of Expressions, so this reads the
    /// Language Unit partition rather than any Expression's contents.
    ///
    /// This answers the geometry alone: whether a Bang is cardinally aligned
    /// with `root`, not whether a complete root sits there. ADR 0006 requires
    /// both, and the caller supplies the second half by passing an Expression's
    /// own root anchor. A Position holding no root answers `true` just as
    /// readily, so a future caller delivering activation to arbitrary Positions
    /// owes its own root check.
    pub fn is_root_active(&self, root: Position) -> bool {
        self.units()
            .filter(|unit| matches!(unit.kind(), LanguageUnitKind::Bang))
            .any(|unit| {
                activated_root_anchors(self.grid, unit.anchor()).any(|anchor| anchor == root)
            })
    }

    /// Every parser and unmatched-character diagnostic in this revision.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.expressions
            .iter()
            .filter_map(|expression| expression.diagnostic.as_ref())
            .chain(self.lexical_diagnostics.iter())
    }

    #[cfg(test)]
    pub(super) fn expression_diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.expressions
            .iter()
            .filter_map(|expression| expression.diagnostic.as_ref())
    }

    /// The semantic Glyph for the Cell at `position`, when the revision gives
    /// that Cell a language classification.
    pub fn glyph_at(&self, position: Position) -> Option<Glyph> {
        self.glyphs
            .get(self.grid.index(position))
            .copied()
            .flatten()
    }

    pub(super) fn glyph_at_index(&self, idx: usize) -> Option<Glyph> {
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
        let analysis = match standalone_run
            .map(SourceAnalysis::Complete)
            .map_or_else(|| Parser::from(&mut source).analyze(), Ok)
        {
            Ok(analysis) => analysis,
            Err(error) => {
                self.expressions.push(ExpressionEntry {
                    atoms: None,
                    diagnostic: Some(Diagnostic::for_range(grid, start, end, error.to_string())),
                    root: None,
                    footprint: footprint_for_range(grid, range),
                    units: units_in_range(&self.units, grid, range),
                });
                return;
            }
        };

        let executable = matches!(analysis, SourceAnalysis::Complete(_));
        let diagnostic = analysis
            .error()
            .map(|error| Diagnostic::for_range(grid, start, end, error.to_string()));
        let expression = analysis.into_expression();
        let mut expression_units = units_in_range(&self.units, grid, range);
        if executable {
            for (unit, (_, atom)) in expression_units.iter_mut().zip(expression.entries()) {
                unit.kind = match atom {
                    Atom::Function(function) => LanguageUnitKind::Function(function),
                    Atom::Bang => LanguageUnitKind::Bang,
                    Atom::Activation(activation) => LanguageUnitKind::Activation(activation),
                    atom => LanguageUnitKind::Atom(atom),
                };
            }
        }
        let root = executable
            .then(|| {
                expression_units.iter().find_map(|unit| {
                    matches!(unit.kind, LanguageUnitKind::Function(_)).then_some(unit.anchor)
                })
            })
            .flatten();
        let (atoms, mut glyphs) = expression_parts(expression, executable);
        if !executable
            && matches!(
                expression_units.as_slice(),
                [LanguageUnit {
                    kind: LanguageUnitKind::OperandLiteral,
                    ..
                }]
            )
        {
            // A standalone Operand Literal has no contextual Number or Note
            // type. Preserve the existing raw-character presentation while
            // retaining its invalid-expression diagnostic.
            glyphs.clear();
        }
        self.set_glyphs(grid, start, glyphs);
        self.expressions.push(ExpressionEntry {
            atoms,
            diagnostic,
            root,
            footprint: footprint_for_range(grid, range),
            units: expression_units,
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

fn partition_units(grid: Grid, bytes: &[u8]) -> (Vec<LanguageUnit>, Vec<Diagnostic>) {
    let mut units = Vec::new();
    let mut diagnostics = Vec::new();

    for row in grid.rows() {
        let positions = row.collect::<Vec<_>>();
        let mut column = 0;
        while column < positions.len() {
            let anchor = positions[column];
            let idx = grid.index(anchor);
            if bytes[idx] == SPACE_BYTE {
                column += 1;
                continue;
            }
            if bytes[idx] == b'#' && bytes.get(idx + 1) == Some(&b'#') && grid.fits(anchor, 2) {
                break;
            }

            let Some(spelling) = bytes.get(idx..idx + 2).filter(|_| grid.fits(anchor, 2)) else {
                diagnostics.push(invalid_unit_diagnostic(grid, idx, bytes[idx]));
                column += 1;
                continue;
            };
            let kind = match spelling {
                b"**" => Some(LanguageUnitKind::Bang),
                _ => std::str::from_utf8(spelling).ok().and_then(|spelling| {
                    Activation::try_from(spelling)
                        .map(LanguageUnitKind::Activation)
                        .ok()
                        .or_else(|| {
                            Function::try_from(spelling)
                                .map(LanguageUnitKind::Function)
                                .ok()
                        })
                        .or_else(|| {
                            (to_atom_num(spelling).is_ok() || to_atom_note(spelling).is_ok())
                                .then_some(LanguageUnitKind::OperandLiteral)
                        })
                }),
            };

            if let Some(kind) = kind {
                units.push(LanguageUnit {
                    kind,
                    anchor,
                    footprint: Footprint {
                        positions: positions[column..column + 2].to_vec(),
                    },
                });
                column += 2;
            } else {
                diagnostics.push(invalid_unit_diagnostic(grid, idx, bytes[idx]));
                column += 1;
            }
        }
    }

    (units, diagnostics)
}

/// The root anchors a Bang anchored at `bang` activates.
///
/// ADR 0006 states the geometry from the Bang outward: north `(x, y-1)`, south
/// `(x, y+1)`, west `(x-2, y)`, and east `(x+2, y)`. The horizontal step is two
/// Cells because every Language Unit is two Cells wide, so a horizontal
/// neighbour's anchor sits two columns away rather than one. An anchor outside
/// the Grid is not a Position at all and simply does not appear.
///
/// The west and east anchors are stated here because ADR 0006 states them, but
/// no Source can reach them today: `row_extents` splits Expression runs only on
/// spaces and `##`, so a horizontally adjacent Bang either merges into the
/// root's own run and forms no root at all, or is separated by a space that
/// puts its anchor three or more columns away. `spatial-tick-planning/01` owns
/// the Expression partition that makes them reachable;
/// `test_a_horizontally_adjacent_bang_does_not_activate_a_terminal_root` pins
/// the present behaviour until then.
fn activated_root_anchors(grid: Grid, bang: Position) -> impl Iterator<Item = Position> {
    let (x, y) = (bang.x(), bang.y());

    [
        y.checked_sub(1).map(|north| (x, north)),
        Some((x, y + 1)),
        x.checked_sub(2).map(|west| (west, y)),
        Some((x + 2, y)),
    ]
    .into_iter()
    .flatten()
    .filter_map(move |(x, y)| grid.position(x, y))
}

fn invalid_unit_diagnostic(grid: Grid, idx: usize, byte: u8) -> Diagnostic {
    Diagnostic::for_range(
        grid,
        idx,
        idx,
        format!("invalid Language Unit character {:?}", char::from(byte)),
    )
}

fn footprint_for_range(grid: Grid, range: Range) -> Footprint {
    Footprint::from_indices(grid, range.start()..=range.end())
}

fn units_in_range(units: &[LanguageUnit], grid: Grid, range: Range) -> Vec<LanguageUnit> {
    units
        .iter()
        .filter(|unit| {
            let idx = grid.index(unit.anchor);
            (range.start()..=range.end()).contains(&idx)
        })
        .cloned()
        .collect()
}

fn parse_standalone_run(source: &str) -> Option<Expression> {
    if source.len() < 4 || !source.len().is_multiple_of(2) {
        return None;
    }

    let mut expression = Expression::new();
    for spelling in source.as_bytes().as_chunks::<2>().0 {
        let (token, atom) = match spelling {
            b"**" => (Token::Bang, Atom::Bang),
            _ => {
                let activation = std::str::from_utf8(spelling)
                    .ok()
                    .and_then(|spelling| Activation::try_from(spelling).ok())?;
                (Token::Activation, Atom::Activation(activation))
            }
        };
        expression.add(token, atom).ok()?;
    }
    Some(expression)
}

fn expression_parts(expression: Expression, executable: bool) -> (Option<Atoms>, Vec<Glyph>) {
    let glyphs = Glyph::to_glyphs(expression.tokens().collect());
    let atoms = executable.then(|| expression.atoms()).flatten();
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

fn row_extents(row_start: usize, row: &[u8]) -> Vec<Option<Range>> {
    let mut extents = vec![None; row.len()];
    let mut local_start = 0;

    while local_start < row.len() {
        if row[local_start..].starts_with(b"##") {
            break;
        }
        if row[local_start] == SPACE_BYTE {
            local_start += 1;
            continue;
        }

        let local_end = (local_start..row.len())
            .find(|&idx| row[idx] == SPACE_BYTE || row[idx..].starts_with(b"##"))
            .map_or(row.len() - 1, |idx| idx - 1);
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

    use super::{ExpressionMap, LanguageMap, LanguageUnitKind, Range};

    #[test]
    fn public_language_map_expression_exposes_root_nested_functions_and_footprints() {
        let grid = Grid::new(10, 1);
        let map = LanguageMap::derive(grid, ".+.x010203").unwrap();
        let expression = map.expressions().next().unwrap();

        assert_eq!(expression.root().unwrap().x(), 0);
        assert_eq!(expression.root().unwrap().y(), 0);
        assert_eq!(
            expression
                .units()
                .filter_map(|unit| match unit.kind() {
                    LanguageUnitKind::Function(function) => Some(function),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![lang::Function::Add, lang::Function::Multiply]
        );
        assert_eq!(
            expression
                .units()
                .flat_map(|unit| unit.footprint().positions())
                .map(|position| (position.x(), position.y()))
                .collect::<Vec<_>>(),
            vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (5, 0),
                (6, 0),
                (7, 0),
                (8, 0),
                (9, 0),
            ]
        );
    }

    #[test]
    fn expression_and_diagnostic_positions_belong_to_the_derived_revision_grid() {
        let grid = Grid::new(4, 2);
        let map = LanguageMap::derive(grid, ".+01xxxx").unwrap();
        let first = map.expressions().next().unwrap();
        let diagnostic = map.diagnostics().next().unwrap();

        assert_eq!(first.footprint().positions().count(), 4);
        assert_eq!(diagnostic.anchor(), grid.position(0, 0));
        assert_eq!(
            diagnostic
                .footprint()
                .positions()
                .map(|position| (position.x(), position.y()))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 0), (2, 0), (3, 0)]
        );
        assert!(map.expressions().all(|expression| {
            expression
                .footprint()
                .positions()
                .all(|position| position.y() < 2)
        }));
    }

    fn unit_spellings(map: &LanguageMap) -> Vec<(usize, Vec<usize>)> {
        map.units()
            .map(|unit| {
                (
                    unit.anchor().x(),
                    unit.footprint()
                        .positions()
                        .map(|position| position.x())
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn language_map_partitions_complete_units_left_to_right_without_overlap() {
        let bangs = LanguageMap::build(Grid::new(3, 1), b"***");
        let west = LanguageMap::build(Grid::new(3, 1), b"<<<");
        let north = LanguageMap::build(Grid::new(4, 1), b"^^^^");

        assert_eq!(unit_spellings(&bangs), vec![(0, vec![0, 1])]);
        assert_eq!(unit_spellings(&west), vec![(0, vec![0, 1])]);
        assert_eq!(
            unit_spellings(&north),
            vec![(0, vec![0, 1]), (2, vec![2, 3])]
        );
        assert!(bangs.diagnostics().any(|diagnostic| diagnostic.start == 2));
        assert!(west.diagnostics().any(|diagnostic| diagnostic.start == 2));
        assert_eq!(north.diagnostics().count(), 0);
    }

    #[test]
    fn language_map_recognizes_every_current_unit_kind_on_a_rectangular_grid() {
        let map = LanguageMap::build(Grid::new(12, 2), b".+C4**>>    ^^vv<<00    ");

        assert_eq!(
            map.units().map(|unit| unit.kind()).collect::<Vec<_>>(),
            vec![
                LanguageUnitKind::Function(lang::Function::Add),
                LanguageUnitKind::OperandLiteral,
                LanguageUnitKind::Bang,
                LanguageUnitKind::Activation(Activation::East),
                LanguageUnitKind::Activation(Activation::North),
                LanguageUnitKind::Activation(Activation::South),
                LanguageUnitKind::Activation(Activation::West),
                LanguageUnitKind::OperandLiteral,
            ]
        );

        assert_eq!(
            unit_spellings(&map),
            vec![
                (0, vec![0, 1]),
                (2, vec![2, 3]),
                (4, vec![4, 5]),
                (6, vec![6, 7]),
                (0, vec![0, 1]),
                (2, vec![2, 3]),
                (4, vec![4, 5]),
                (6, vec![6, 7]),
            ]
        );
    }

    #[test]
    fn a_bang_aligns_with_the_root_anchor_at_each_of_its_four_cardinal_positions() {
        // The geometry filter alone: this Grid holds no root, because the
        // horizontal anchors are unreachable from any Source that parses one
        // (see `activated_root_anchors`). What a real root does with an
        // aligned Bang is pinned end-to-end in `source::model`'s Tick tests.
        let grid = Grid::new(6, 3);
        let map = LanguageMap::build(grid, b"        **        ");
        let at = |x, y| grid.position(x, y).expect("inside the Grid");

        // The Bang is anchored at (2, 1).
        for (x, y) in [(2, 0), (2, 2), (0, 1), (4, 1)] {
            assert!(map.is_root_active(at(x, y)), "({x}, {y})");
        }
        // One column off an aligned anchor, diagonally placed, or the Bang's
        // own anchor: only complete cardinal alignment activates.
        for (x, y) in [(1, 1), (3, 1), (1, 0), (3, 2), (2, 1)] {
            assert!(!map.is_root_active(at(x, y)), "({x}, {y})");
        }
    }

    #[test]
    fn a_source_without_a_bang_activates_no_root() {
        let grid = Grid::new(6, 1);
        let map = LanguageMap::build(grid, b".+0102");

        assert!(!map.is_root_active(grid.position(0, 0).unwrap()));
    }

    #[test]
    fn language_map_never_forms_a_unit_across_a_row_edge() {
        let map = LanguageMap::build(Grid::new(3, 2), b"  **  ");

        assert!(map.units().next().is_none());
        assert_eq!(
            map.diagnostics()
                .filter(|diagnostic| diagnostic.message.starts_with("invalid Language Unit"))
                .map(|diagnostic| diagnostic.start)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[test]
    fn comments_and_live_edit_fragments_do_not_form_language_units() {
        let comment = LanguageMap::build(Grid::new(8, 1), b"**##**00");
        let fragment = LanguageMap::build(Grid::new(8, 1), b".+# **  ");

        assert_eq!(unit_spellings(&comment), vec![(0, vec![0, 1])]);
        assert_eq!(
            comment
                .expressions()
                .next()
                .unwrap()
                .footprint()
                .positions()
                .count(),
            2
        );
        assert_eq!(comment.diagnostics().count(), 0);
        assert_eq!(
            unit_spellings(&fragment),
            vec![(0, vec![0, 1]), (4, vec![4, 5])]
        );
        assert!(
            fragment
                .diagnostics()
                .any(|diagnostic| diagnostic.start == 2)
        );
    }

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
        let map = ExpressionMap::build(Grid::new(5, 1), b" .+1 ");
        assert_eq!(map.inner[0], None);
        assert_range(&map, 1, 3);
        assert_eq!(map.inner[4], None);
    }

    #[test]
    fn build_separates_multiple_runs_in_one_row() {
        let map = ExpressionMap::build(Grid::new(8, 1), b".+  .-  ");
        assert_range(&map, 0, 1);
        assert_eq!(map.inner[2], None);
        assert_eq!(map.inner[3], None);
        assert_range(&map, 4, 5);
        assert_eq!(map.inner[6], None);
        assert_eq!(map.inner[7], None);
    }

    #[test]
    fn build_keeps_edge_touching_runs_inside_their_rows() {
        let map = ExpressionMap::build(Grid::new(4, 2), b"  .+.-  ");
        assert_range(&map, 2, 3);
        assert_range(&map, 4, 5);
        assert_ne!(map.inner[3], map.inner[4]);
    }

    #[test]
    fn prospective_range_scans_only_the_edited_row() {
        let grid = Grid::new(5, 2);
        let bytes = b".+   .-   ";
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
        assert_eq!(expressions[0].footprint().positions().count(), 6);
        assert!(expressions[0].atoms().is_some());
        assert_eq!(map.glyph_at_index(0), Some(Glyph::Function));
        assert_eq!(map.glyph_at_index(7), Some(Glyph::Char));
        assert_eq!(map.expression_diagnostics().count(), 1);
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
        assert!((0..4).all(|idx| bangs.glyph_at_index(idx) == Some(Glyph::Bang)));
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
