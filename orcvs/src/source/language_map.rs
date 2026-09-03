use std::sync::atomic::{AtomicU64, Ordering};

use lang::{
    Activation, Atom, Atoms, Error as LangError, Expression, Function, Parser, SourceAnalysis,
    Token, to_atom_note, to_atom_num,
};

use crate::{
    glyph::Glyph,
    grid::{CellIndex, Grid, Position},
};

use super::Diagnostic;

const SPACE_BYTE: u8 = b' ';

static NEXT_LANGUAGE_MAP_ID: AtomicU64 = AtomicU64::new(1);

/// Which derivation a Map, and the Expressions it owns, came from.
///
/// Two revisions of one Source share a Grid, so a Span alone cannot say which
/// revision minted it. This is the same device `GridId` is for Positions and
/// Cell indices: an identity only the owner mints, so a value carrying it
/// names the one collection that can answer for it. Copies of a Map share its
/// identity, exactly as copies of a Grid do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LanguageMapId(u64);

impl LanguageMapId {
    fn new() -> Self {
        let id = NEXT_LANGUAGE_MAP_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("LanguageMap identity space exhausted");
        Self(id)
    }
}

/// The semantic information derived from one complete Source revision.
///
/// This is the single owner of Expression Spans, parsed expressions, Glyph
/// classifications, and diagnostics. It deliberately exposes only the
/// semantics the current parser and row-local partition can establish.
#[derive(Clone)]
pub struct LanguageMap {
    id: LanguageMapId,
    grid: Grid,
    units: Vec<LanguageUnit>,
    expressions: Vec<ExpressionEntry>,
    glyphs: Vec<Option<Glyph>>,
    lexical_diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// What the two-Cell spelling at a unit's anchor is, read from the characters
/// alone.
///
/// A literal stays an `OperandLiteral`: the same two characters spell a Number
/// in a Number slot and a Note in a Note slot, so the Atom type belongs to the
/// consuming Function's signature and not to the Source. See ADR 0021.
pub enum LanguageUnitKind {
    OperandLiteral,
    Function(Function),
    Bang,
    Activation(Activation),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanguageUnit {
    kind: LanguageUnitKind,
    anchor: Position,
    span: Span,
}

impl LanguageUnit {
    pub fn kind(&self) -> LanguageUnitKind {
        self.kind
    }

    pub fn anchor(&self) -> Position {
        self.anchor
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

/// The Cells one Language Unit, Expression, or Diagnostic occupies.
///
/// A row is the whole horizontal run there is, so a Span is a contiguous
/// run within one row and is named by its first and last Cell rather than by
/// listing what lies between them. It carries the Grid that minted those
/// Cells, so it can answer its own Positions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    grid: Grid,
    start: CellIndex,
    end: CellIndex,
}

impl Span {
    pub(super) fn new(grid: Grid, start: CellIndex, end: CellIndex) -> Self {
        assert!(start <= end, "a Span's first Cell precedes its last");
        Self { grid, start, end }
    }

    /// The first Cell of this Span.
    pub(super) fn start(self) -> CellIndex {
        self.start
    }

    /// The last Cell of this Span. Inclusive.
    pub(super) fn end(self) -> CellIndex {
        self.end
    }

    /// Every Cell index this Span covers, first to last.
    pub(super) fn indices(self) -> impl Iterator<Item = CellIndex> {
        let grid = self.grid;
        (self.start.get()..=self.end.get()).filter_map(move |idx| grid.cell_index(idx))
    }

    pub fn positions(self) -> impl Iterator<Item = Position> {
        let grid = self.grid;
        self.indices().map(move |idx| grid.position_at(idx))
    }
}

#[derive(Clone)]
pub struct ExpressionEntry {
    map_id: LanguageMapId,
    atoms: Option<Atoms>,
    diagnostic: Option<Diagnostic>,
    root: Option<Position>,
    span: Span,
}

impl ExpressionEntry {
    /// The first Function anchor when this is a complete executable Expression.
    pub fn root(&self) -> Option<Position> {
        self.root
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub(super) fn atoms(&self) -> Option<&Atoms> {
        self.atoms.as_ref()
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
        let spans = expression_spans(grid, bytes);
        let mut map = Self {
            id: LanguageMapId::new(),
            grid,
            units,
            expressions: Vec::new(),
            glyphs: vec![None; bytes.len()],
            lexical_diagnostics,
        };

        for span in spans {
            map.parse_span(grid, bytes, span);
        }
        for (idx, byte) in bytes.iter().copied().enumerate() {
            if byte != SPACE_BYTE && map.glyphs[idx].is_none() {
                map.glyphs[idx] = Some(Glyph::Char);
            }
        }

        map
    }

    /// Answers the Expression Span that would contain `idx` after replacing
    /// that Cell with `byte`, without scanning any other row.
    pub(super) fn prospective_expression_span(
        grid: Grid,
        bytes: &[u8],
        idx: usize,
        byte: u8,
    ) -> Option<Span> {
        prospective_span(grid, bytes, idx, byte)
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
            .get(self.grid.index(position).get())
            .copied()
            .flatten()
    }

    ///
    /// The Language Units this Expression is spelled from, in Source order.
    ///
    /// A foreign Expression is refused. A Span is only Cell numbers, and two
    /// revisions of one Source share a Grid, so a Span minted by another
    /// revision addresses this partition perfectly well and would be
    /// answered with the wrong units. The Expression's revision identity is
    /// what distinguishes them, the same way a Position's Grid identity names
    /// the Grid that can place it.
    ///
    pub fn expression_units(&self, expression: &ExpressionEntry) -> &[LanguageUnit] {
        assert!(
            self.id == expression.map_id,
            "ExpressionEntry belongs to another LanguageMap"
        );
        &self.units[units_range(&self.units, self.grid, expression.span)]
    }

    ///
    /// What one Expression Span analyzes to.
    ///
    /// A Span the partition has already named end to end as standalone Atoms is
    /// assembled from those units; every other Span is read by the Parser, which
    /// is the only path that touches the characters again.
    ///
    fn analyze_span(
        &self,
        bytes: &[u8],
        span: Span,
        units: std::ops::Range<usize>,
    ) -> Result<SourceAnalysis, LangError> {
        // Recomputed only to check the caller, and only in a debug build: the
        // search itself still happens once, in `parse_span`.
        debug_assert_eq!(
            units,
            units_range(&self.units, self.grid, span),
            "a Span is analyzed from the units covering it"
        );
        if let Some(expression) = standalone_run(&self.units[units], span) {
            return Ok(SourceAnalysis::Complete(expression));
        }

        let mut source = String::from_utf8(bytes[span.start().get()..=span.end().get()].to_vec())
            .expect("Source Cells contain ASCII");
        Parser::from(&mut source).analyze()
    }

    fn parse_span(&mut self, grid: Grid, bytes: &[u8], span: Span) {
        let start = span.start();
        let end = span.end();
        // The units covering this Span, named once. Held as a range rather than
        // a slice so the Span's Cells can be re-glyphed between the two places
        // that read them.
        let units = units_range(&self.units, grid, span);
        // A later Expression owns its occupied Cells over any operand-slot
        // hints emitted by an earlier Expression.
        self.glyphs[start.get()..=end.get()].fill(None);
        let analysis = match self.analyze_span(bytes, span, units.clone()) {
            Ok(analysis) => analysis,
            Err(error) => {
                self.expressions.push(ExpressionEntry {
                    map_id: self.id,
                    atoms: None,
                    diagnostic: Some(Diagnostic::for_range(grid, start, end, error.to_string())),
                    root: None,
                    span,
                });
                return;
            }
        };

        let executable = matches!(analysis, SourceAnalysis::Complete(_));
        let diagnostic = analysis
            .error()
            .map(|error| Diagnostic::for_range(grid, start, end, error.to_string()));
        let expression = analysis.into_expression();
        let expression_units = &self.units[units];
        let root = executable
            .then(|| {
                expression_units.iter().find_map(|unit| {
                    matches!(unit.kind, LanguageUnitKind::Function(_)).then_some(unit.anchor)
                })
            })
            .flatten();
        let standalone_literal = matches!(
            expression_units,
            [LanguageUnit {
                kind: LanguageUnitKind::OperandLiteral,
                ..
            }]
        );
        let (atoms, mut glyphs) = expression_parts(expression, executable);
        if !executable && standalone_literal {
            // A standalone Operand Literal has no contextual Number or Note
            // type. Preserve the existing raw-character presentation while
            // retaining its invalid-expression diagnostic.
            glyphs.clear();
        }
        self.set_glyphs(grid, start, glyphs);
        self.expressions.push(ExpressionEntry {
            map_id: self.id,
            atoms,
            diagnostic,
            root,
            span,
        });
    }

    fn set_glyphs(&mut self, grid: Grid, start: CellIndex, glyphs: Vec<Glyph>) {
        let anchor = grid.position_at(start);
        for (offset, glyph) in glyphs.into_iter().enumerate() {
            // Operand-slot hints can extend beyond their Expression, but an
            // Expression is horizontal: hints stop at the same row edge, which
            // is what `offset_in_row` answers.
            let Some(idx) = grid.offset_in_row(anchor, offset) else {
                break;
            };
            self.glyphs[idx.get()] = Some(glyph);
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
            let at = idx.get();
            if bytes[at] == SPACE_BYTE {
                column += 1;
                continue;
            }
            if bytes[at] == b'#' && bytes.get(at + 1) == Some(&b'#') && grid.fits(anchor, 2) {
                break;
            }

            let Some(spelling) = bytes.get(at..at + 2).filter(|_| grid.fits(anchor, 2)) else {
                diagnostics.push(invalid_unit_diagnostic(grid, idx, bytes[at]));
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
                let last = grid
                    .offset_in_row(anchor, 1)
                    .expect("a complete Language Unit fits its row");
                units.push(LanguageUnit {
                    kind,
                    anchor,
                    span: Span::new(grid, idx, last),
                });
                column += 2;
            } else {
                diagnostics.push(invalid_unit_diagnostic(grid, idx, bytes[at]));
                column += 1;
            }
        }
    }

    // Rows are walked top to bottom and each row's column only ever advances,
    // so anchors ascend strictly and an Expression Span names a contiguous
    // run of this partition. Nothing downstream may reorder it.
    debug_assert!(
        units.is_sorted_by_key(|unit| grid.index(unit.anchor)),
        "Language Units are partitioned in ascending anchor order"
    );

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
/// no Source can reach them today: `row_spans` splits Expression runs only on
/// spaces and `##`, so a horizontally adjacent Bang either merges into the
/// root's own run and forms no root at all, or is separated by a space that
/// puts its anchor three or more columns away. `spatial-tick-planning/02` owns
/// the Snapshot Bang activation that makes them reachable;
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

fn invalid_unit_diagnostic(grid: Grid, idx: CellIndex, byte: u8) -> Diagnostic {
    Diagnostic::for_range(
        grid,
        idx,
        idx,
        format!("invalid Language Unit character {:?}", char::from(byte)),
    )
}

/// Where the Language Units of `span` sit in `units`.
///
/// `partition_units` establishes its units in ascending anchor order, so an
/// Expression Span names a contiguous run of them and both ends are found by
/// search rather than by testing every unit against every Span. The returned
/// bounds are positions in `units`, a different index space from the Cell
/// indices the Span carries; the Span's own bounds are inclusive at both ends.
fn units_range(units: &[LanguageUnit], grid: Grid, span: Span) -> std::ops::Range<usize> {
    let first = units.partition_point(|unit| grid.index(unit.anchor) < span.start());
    let past_last = units.partition_point(|unit| grid.index(unit.anchor) <= span.end());
    first..past_last
}

///
/// Whether `units` tile `span`: the first begins where the Span begins, each
/// next begins immediately after the one before it ends, and the last ends
/// where the Span ends.
///
/// A tiled Span has every Cell claimed by a Language Unit, so the length and
/// parity a run of two-Cell spellings must have are consequences of the tiling
/// rather than separate tests: an odd Cell count cannot be tiled by two-Cell
/// units at all, and a Cell the partition diagnosed instead of naming leaves a
/// hole that no arrangement of units closes.
///
/// Adjacency is arithmetic on bare Cell numbers rather than a question for the
/// Grid, because a Span and the units within it are confined to one row and
/// share one Grid: the running number is only ever compared, never used to
/// address a Cell, so no index this Grid cannot answer for comes into being.
///
fn units_tile_span(units: &[LanguageUnit], span: Span) -> bool {
    let mut next = span.start().get();
    for unit in units {
        if unit.span().start().get() != next {
            return false;
        }
        next = unit.span().end().get() + 1;
    }
    next == span.end().get() + 1
}

///
/// The Expression a Span spells when the partition named every one of its
/// Cells as a standalone Atom — a Bang or an Activation.
///
/// The Parser accepts one Language Unit and calls whatever follows it trailing
/// content, so a run of standalone Atoms is the one shape it cannot take whole.
/// ADR 0024 makes the partition the single owner of what a two-Cell spelling
/// is, so the Expression is assembled from the kinds it established rather than
/// by reading the same characters a second time.
///
/// `None` leaves the Span to the Parser: when the units do not tile it, when
/// any of them is a Function or an Operand Literal, or when the run exceeds the
/// Expression's capacity. The Parser then answers, and answers as it did before
/// this path existed — for an over-capacity run of standalone Atoms that is
/// trailing content after the one unit it accepts, not a capacity error.
///
fn standalone_run(units: &[LanguageUnit], span: Span) -> Option<Expression> {
    if !units_tile_span(units, span) {
        return None;
    }

    let mut expression = Expression::new();
    for unit in units {
        let (token, atom) = match unit.kind() {
            LanguageUnitKind::Bang => (Token::Bang, Atom::Bang),
            LanguageUnitKind::Activation(activation) => {
                (Token::Activation, Atom::Activation(activation))
            }
            LanguageUnitKind::Function(_) | LanguageUnitKind::OperandLiteral => return None,
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

///
/// The Expression Spans of one row, left to right.
///
/// A row is the whole horizontal extent there is, so a run ends at a space, at
/// a `##` Comment, or at the row edge. Each run yields one Span; Cells between
/// runs belong to none.
///
fn row_spans(grid: Grid, row_start: usize, row: &[u8]) -> Vec<Span> {
    let mut spans = Vec::new();
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
        let cell = |idx: usize| {
            grid.cell_index(idx)
                .expect("a row's Span lies inside the Grid that owns the row")
        };
        spans.push(Span::new(
            grid,
            cell(row_start + local_start),
            cell(row_start + local_end),
        ));
        local_start = local_end + 1;
    }
    spans
}

///
/// The Expression Spans of a whole Source revision, in row-major order.
///
fn expression_spans(grid: Grid, bytes: &[u8]) -> Vec<Span> {
    assert_eq!(
        bytes.len(),
        grid.count(),
        "LanguageMap Source length must match its Grid"
    );

    let cols = grid.cols();
    let mut spans = Vec::new();
    for (row_number, row) in bytes.chunks_exact(cols).enumerate() {
        spans.extend(row_spans(grid, row_number * cols, row));
    }
    spans
}

///
/// The Span covering `cell`, when one does. Spans within a row do not overlap,
/// so at most one can answer.
///
fn span_containing(spans: &[Span], cell: CellIndex) -> Option<Span> {
    spans
        .iter()
        .copied()
        .find(|span| span.start() <= cell && cell <= span.end())
}

///
/// The Span that would cover `idx` after replacing that Cell with `byte`.
///
/// Only the edited row is rebuilt: an Expression is horizontal, so no other
/// row's Spans can change.
///
fn prospective_span(grid: Grid, bytes: &[u8], idx: usize, byte: u8) -> Option<Span> {
    assert_eq!(
        bytes.len(),
        grid.count(),
        "LanguageMap Source length must match its Grid"
    );
    let cell = grid
        .cell_index(idx)
        .expect("prospective Cell must belong to the Source");

    let cols = grid.cols();
    let row_start = (idx / cols) * cols;
    let mut row = bytes[row_start..row_start + cols].to_vec();
    row[idx - row_start] = byte;

    span_containing(&row_spans(grid, row_start, &row), cell)
}

#[cfg(test)]
mod tests {
    use crate::{glyph::Glyph, grid::Grid};

    use lang::{Activation, Atom};

    use super::{LanguageMap, LanguageUnitKind, Span, expression_spans, prospective_span};

    #[test]
    fn public_language_map_expression_exposes_root_nested_functions_and_spans() {
        let grid = Grid::new(10, 1);
        let map = LanguageMap::derive(grid, ".+.x010203").unwrap();
        let expression = map.expressions().next().unwrap();

        assert_eq!(expression.root().unwrap().x(), 0);
        assert_eq!(expression.root().unwrap().y(), 0);
        assert_eq!(
            map.expression_units(expression)
                .iter()
                .filter_map(|unit| match unit.kind() {
                    LanguageUnitKind::Function(function) => Some(function),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![lang::Function::Add, lang::Function::Multiply]
        );
        assert_eq!(
            map.expression_units(expression)
                .iter()
                .flat_map(|unit| unit.span().positions())
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
    fn each_expression_names_its_own_units_including_the_ones_at_its_edges() {
        // `units_range` locates a Span's units by search, so both bounds
        // have to be exact: a lower bound one unit too high drops the root a
        // row's first Expression is spelled from, and an exclusive upper bound
        // drops its last operand. Two Spans in one row put a neighbour on
        // each side of both edges, so either slip shows up as a missing or
        // borrowed anchor rather than as a crash.
        let grid = Grid::new(9, 1);
        let map = LanguageMap::derive(grid, ".+01 .-02").unwrap();
        let expressions = map.expressions().collect::<Vec<_>>();

        let anchors = |expression| {
            map.expression_units(expression)
                .iter()
                .map(|unit| (unit.anchor().x(), unit.anchor().y()))
                .collect::<Vec<_>>()
        };

        assert_eq!(expressions.len(), 2);
        assert_eq!(anchors(expressions[0]), vec![(0, 0), (2, 0)]);
        assert_eq!(anchors(expressions[1]), vec![(5, 0), (7, 0)]);
    }

    #[test]
    fn expression_and_diagnostic_positions_belong_to_the_derived_revision_grid() {
        let grid = Grid::new(4, 2);
        let map = LanguageMap::derive(grid, ".+01xxxx").unwrap();
        let first = map.expressions().next().unwrap();
        let diagnostic = map.diagnostics().next().unwrap();

        assert_eq!(first.span().positions().count(), 4);
        assert_eq!(
            diagnostic.anchor(),
            grid.position(0, 0).expect("inside the Grid")
        );
        assert_eq!(
            diagnostic
                .span()
                .positions()
                .map(|position| (position.x(), position.y()))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 0), (2, 0), (3, 0)]
        );
        assert!(map.expressions().all(|expression| {
            expression
                .span()
                .positions()
                .all(|position| position.y() < 2)
        }));
    }

    fn unit_spellings(map: &LanguageMap) -> Vec<(usize, Vec<usize>)> {
        map.units()
            .map(|unit| {
                (
                    unit.anchor().x(),
                    unit.span()
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
        assert!(
            bangs
                .diagnostics()
                .any(|diagnostic| diagnostic.start() == 2)
        );
        assert!(west.diagnostics().any(|diagnostic| diagnostic.start() == 2));
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
                .map(|diagnostic| diagnostic.start())
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
                .span()
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
                .any(|diagnostic| diagnostic.start() == 2)
        );
    }

    fn cell(grid: Grid, idx: usize) -> crate::grid::CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    fn span(grid: Grid, start: usize, end: usize) -> Span {
        Span::new(grid, cell(grid, start), cell(grid, end))
    }

    #[test]
    fn build_leaves_empty_rows_without_expressions() {
        assert!(expression_spans(Grid::new(5, 1), b"     ").is_empty());
    }

    #[test]
    fn build_names_one_span_per_run_covering_it_inclusively() {
        let grid = Grid::new(5, 1);
        let spans = expression_spans(grid, b" .+1 ");

        // one Span for the run, and it covers exactly the run's Cells
        assert_eq!(spans, vec![span(grid, 1, 3)]);
        assert_eq!(
            spans[0].indices().map(|idx| idx.get()).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn build_separates_multiple_runs_in_one_row() {
        let grid = Grid::new(8, 1);

        // asserting the whole list, not Cell by Cell: a spurious extra Span
        // shows up here and would not show up in per-Cell probing
        assert_eq!(
            expression_spans(grid, b".+  .-  "),
            vec![span(grid, 0, 1), span(grid, 4, 5)]
        );
    }

    #[test]
    fn build_keeps_edge_touching_runs_inside_their_rows() {
        let grid = Grid::new(4, 2);

        // the runs touch across the row edge but are two Spans, not one
        assert_eq!(
            expression_spans(grid, b"  .+.-  "),
            vec![span(grid, 2, 3), span(grid, 4, 5)]
        );
    }

    #[test]
    fn prospective_span_scans_only_the_edited_row() {
        let grid = Grid::new(5, 2);
        let bytes = b".+   .-   ";
        assert_eq!(
            prospective_span(grid, bytes, 2, b'1'),
            Some(span(grid, 0, 2))
        );
        assert_eq!(
            prospective_span(grid, bytes, 7, b'0'),
            Some(span(grid, 5, 7))
        );
    }

    #[test]
    #[should_panic(expected = "LanguageMap Source length must match its Grid")]
    fn build_rejects_source_content_with_the_wrong_length() {
        let _ = expression_spans(Grid::new(5, 1), b"    ");
    }

    #[test]
    fn language_map_builds_cohesive_expression_state() {
        let grid = Grid::new(8, 1);
        let map = LanguageMap::build(grid, b".+0102 x");
        let expressions = map.expressions().collect::<Vec<_>>();
        assert_eq!(expressions.len(), 2);
        assert_eq!(expressions[0].span().positions().count(), 6);
        assert!(expressions[0].atoms().is_some());
        let at = |idx: usize| map.glyph_at(grid.position_at(grid.cell_index(idx).unwrap()));
        assert_eq!(at(0), Some(Glyph::Function));
        assert_eq!(at(7), Some(Glyph::Char));
        assert_eq!(map.expression_diagnostics().count(), 1);
    }

    #[test]
    fn a_span_its_units_do_not_tile_is_not_a_standalone_run() {
        // The standalone-run path is selected by the units tiling their Span,
        // and that one property is what the retired recognizer's separate
        // length and parity tests came to. Both of its failures are here.
        //
        // An odd Cell count cannot be tiled by two-Cell units: the third `*`
        // is diagnosed rather than named, and the Span reaches the Parser,
        // which takes the Bang and calls the rest trailing content.
        let odd = LanguageMap::build(Grid::new(3, 1), b"***");
        // An even Cell count is not enough either. This Span is six Cells
        // holding two Bangs, and the two diagnosed Cells between them leave a
        // hole no arrangement of units closes.
        let holed = LanguageMap::build(Grid::new(6, 1), b"**X0**");

        for map in [&odd, &holed] {
            assert!(map.expressions().next().unwrap().atoms().is_none());
            assert!(map.diagnostics().any(|diagnostic| {
                diagnostic
                    .message
                    .starts_with("unexpected trailing content")
            }));
        }
    }

    #[test]
    fn language_map_partitions_adjacent_standalone_units() {
        let bang_grid = Grid::new(4, 1);
        let bangs = LanguageMap::build(bang_grid, b"****");
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
        assert!(
            bang_grid
                .rows()
                .flatten()
                .take(4)
                .all(|position| bangs.glyph_at(position) == Some(Glyph::Bang))
        );
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
}
