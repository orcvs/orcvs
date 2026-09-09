use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::atomic::{AtomicU64, Ordering};

use lang::{Activation, Atom, Atoms, Expression, Function, Parser, SourceAnalysis, Token};

use crate::{
    glyph::Glyph,
    grid::{CellIndex, Grid, Position},
};

use super::{CellContent, Diagnostic};

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
    /// Established once by `build` and never changed afterwards. An
    /// `ExpressionEntry` records where its own units sit here as a range, so
    /// anything that reordered or resized this would silently re-point every
    /// Expression in the Map.
    units: Vec<LanguageUnit>,
    expressions: Vec<ExpressionEntry>,
    glyphs: Vec<Option<Glyph>>,
    lexical_diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// A complete unit established by the Parser. Literal types remain on the
/// positioned Expression; this presentation view groups literal units.
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
    expression: Expression,
    atoms: Option<Atoms>,
    diagnostic: Option<Diagnostic>,
    root: Option<Position>,
    function_candidate: Option<(Position, Function)>,
    span: Span,
    /// Where this Expression's Language Units sit in its Map's partition,
    /// established when the Expression was built.
    ///
    /// A range into the partition rather than the units themselves: the Map
    /// owns them, an Expression is one contiguous run of them, and a Map
    /// outlives every question asked of the Expressions it holds.
    units: std::ops::Range<usize>,
}

impl ExpressionEntry {
    /// The first Function anchor when this is a complete executable Expression.
    pub fn root(&self) -> Option<Position> {
        self.root
    }

    /// The parsed leading Function, even when its operands are not yet valid.
    /// A Tick reserves its turn so earlier writes can complete those operands.
    pub(super) fn function_candidate(&self) -> Option<(Position, Function)> {
        self.function_candidate
    }

    pub(super) fn positioned(&self) -> impl Iterator<Item = &lang::PositionedEntry> {
        self.expression.positioned()
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
        (source.len() == grid.count()
            && source.bytes().all(|byte| CellContent::new(byte).is_some()))
        .then(|| Self::build(grid, source.as_bytes()))
    }

    ///
    /// Rebuilds the Map, parsing only the rows whose Cells changed.
    ///
    /// The row is the unit of work because the row is already the unit of
    /// meaning: `walk_row` reads exactly one row's Cells and carries nothing
    /// across the boundary, and an Expression Span is a run inside one row. A
    /// row's Language Units, Expressions, Glyphs and lexical diagnostics are
    /// therefore functions of that row's bytes alone, and a row nobody wrote
    /// to answers this revision exactly as it answered the last one.
    ///
    /// Everything such a row contributed is carried over rather than parsed
    /// again, which is the whole point: parsing is most of what building a Map
    /// costs, and walking is the small remainder. One field stands in the way.
    /// An `ExpressionEntry` locates its units as a range into the Map's
    /// partition rather than into the Grid, so a dirty row that now holds a
    /// different number of units moves every later Expression's range. Each
    /// carried entry is re-pointed by its offset inside its own row, which
    /// needs no arithmetic across rows and cannot go negative.
    ///
    /// The Map's identity is new either way. An `ExpressionEntry` from the
    /// previous revision addresses the previous partition, and carrying one
    /// forward does not make it answerable there.
    ///
    pub(super) fn rebuild(
        previous: &Self,
        grid: Grid,
        bytes: &[u8],
        dirty: &BTreeSet<usize>,
    ) -> Self {
        assert_eq!(
            bytes.len(),
            grid.count(),
            "LanguageMap Source length must match its Grid"
        );
        assert_eq!(
            previous.grid, grid,
            "a LanguageMap is rebuilt on the Grid that built it"
        );

        let cols = grid.cols();
        let row_count = bytes.len() / cols;

        // Everything the previous Map holds is in row-major order, so one pass
        // over each collection names the run belonging to each row.
        let previous_units = row_runs(row_count, previous.units.len(), |index| {
            grid.index(previous.units[index].anchor).get() / cols
        });
        let previous_expressions = row_runs(row_count, previous.expressions.len(), |index| {
            previous.expressions[index].span.start().get() / cols
        });
        let previous_diagnostics =
            row_runs(row_count, previous.lexical_diagnostics.len(), |index| {
                previous.lexical_diagnostics[index].start() / cols
            });

        // The partition is assembled first and in full, because
        // `record_expression` searches it for the units of the Span it is
        // given.
        let mut units = Vec::with_capacity(previous.units.len());
        let mut walks = Vec::with_capacity(row_count);
        let mut row_units = Vec::with_capacity(row_count);
        for row in 0..row_count {
            let start = units.len();
            if dirty.contains(&row) {
                let mut walk = RowWalk::default();
                walk_row(
                    grid,
                    row * cols,
                    &bytes[row * cols..(row + 1) * cols],
                    &mut walk,
                );
                units.extend_from_slice(&walk.units);
                walks.push(Some(walk));
            } else {
                units.extend_from_slice(&previous.units[previous_units[row].clone()]);
                walks.push(None);
            }
            row_units.push(start..units.len());
        }

        let mut map = Self {
            id: LanguageMapId::new(),
            grid,
            units,
            expressions: Vec::with_capacity(previous.expressions.len()),
            glyphs: vec![None; bytes.len()],
            lexical_diagnostics: Vec::with_capacity(previous.lexical_diagnostics.len()),
        };

        for (row, walk) in walks.into_iter().enumerate() {
            let cells = row * cols..(row + 1) * cols;
            match walk {
                Some(walk) => {
                    map.lexical_diagnostics.extend(walk.diagnostics);
                    for parse in walk.parses {
                        map.record_expression(grid, parse);
                    }
                    for index in cells {
                        if bytes[index] != SPACE_BYTE && map.glyphs[index].is_none() {
                            map.glyphs[index] = Some(Glyph::Char);
                        }
                    }
                }
                None => {
                    map.lexical_diagnostics.extend_from_slice(
                        &previous.lexical_diagnostics[previous_diagnostics[row].clone()],
                    );
                    map.glyphs[cells.clone()].copy_from_slice(&previous.glyphs[cells]);
                    let carried = row_units[row].start;
                    let held = previous_units[row].start;
                    for entry in &previous.expressions[previous_expressions[row].clone()] {
                        let offset = entry.units.start - held;
                        let length = entry.units.len();
                        map.expressions.push(ExpressionEntry {
                            map_id: map.id,
                            units: carried + offset..carried + offset + length,
                            ..entry.clone()
                        });
                    }
                }
            }
        }

        map
    }

    pub(super) fn build(grid: Grid, bytes: &[u8]) -> Self {
        let walk = walk_source(grid, bytes);
        let mut map = Self {
            id: LanguageMapId::new(),
            grid,
            units: walk.units,
            expressions: Vec::new(),
            glyphs: vec![None; bytes.len()],
            lexical_diagnostics: walk.diagnostics,
        };

        for parse in walk.parses {
            map.record_expression(grid, parse);
        }
        for (idx, byte) in bytes.iter().copied().enumerate() {
            if byte != SPACE_BYTE && map.glyphs[idx].is_none() {
                map.glyphs[idx] = Some(Glyph::Char);
            }
        }

        map
    }

    pub fn expressions(&self) -> impl Iterator<Item = &ExpressionEntry> {
        self.expressions.iter()
    }

    pub fn units(&self) -> impl Iterator<Item = &LanguageUnit> {
        self.units.iter()
    }

    /// Bang values from complete standalone Expressions, paired with their
    /// spelling Spans. The parsed Atoms decide meaning; units supply geometry.
    pub(super) fn bangs(&self) -> impl Iterator<Item = (Position, Span)> + '_ {
        self.expressions().flat_map(move |expression| {
            let atoms = expression.atoms().filter(|atoms| {
                atoms
                    .as_slice()
                    .iter()
                    .all(|atom| matches!(atom, Atom::Bang | Atom::Activation(_)))
            });
            atoms.into_iter().flat_map(move |atoms| {
                atoms
                    .as_slice()
                    .iter()
                    .zip(self.expression_units(expression))
                    .filter_map(|(atom, unit)| {
                        matches!(atom, Atom::Bang).then_some((unit.anchor(), unit.span()))
                    })
            })
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
        &self.units[expression.units.clone()]
    }

    ///
    /// Records the Expression one parse established.
    ///
    /// The Span came from the analysis, so nothing here re-reads the Source to
    /// find out where the Expression ends or what it holds. There is no
    /// trailing content to restore either: an Expression claims exactly the
    /// Cells the Parser read, and whatever follows them is the next
    /// Expression's, which is the partition ADR 0033 records.
    ///
    fn record_expression(&mut self, grid: Grid, parse: Parse) {
        let Parse { span, analysis } = parse;
        let start = span.start();
        let end = span.end();
        // Where this Span's units sit in the partition, searched for once here
        // and then recorded on the Expression, so nothing asks again.
        let units = units_range(&self.units, grid, span);
        // A later Expression owns its occupied Cells over any operand-slot
        // hints emitted by an earlier Expression.
        self.glyphs[start.get()..=end.get()].fill(None);

        let executable = analysis.is_complete();
        let diagnostic = analysis
            .error()
            .map(|error| Diagnostic::for_range(grid, start, end, error.to_string()));
        let expression = analysis.into_expression();
        let function_candidate = match expression.entries().next() {
            Some((Token::Function, Atom::Function(function))) => {
                Some((grid.position_at(start), function))
            }
            _ => None,
        };
        let root = executable
            .then_some(function_candidate)
            .flatten()
            .map(|(anchor, _)| anchor);
        let atoms = executable.then(|| expression.atoms()).flatten();
        for entry in expression.positioned() {
            for cell in entry.cells.clone() {
                self.glyphs[cell] = Some(Glyph::from(entry.token));
            }
        }
        self.expressions.push(ExpressionEntry {
            map_id: self.id,
            expression,
            atoms,
            diagnostic,
            root,
            function_candidate,
            span,
            units,
        });
    }
}

///
/// One Expression the walk established: the Cells it occupies, and what the
/// Parser made of them.
///
/// The analysis travels with the Span because the Span came from it. Under the
/// partition ADR 0033 describes there is no second way to decide where an
/// Expression ends, so re-reading the Source to find out what a Span holds
/// would be asking the same question twice and inviting two answers.
///
struct Parse {
    span: Span,
    analysis: SourceAnalysis,
}

///
/// Everything one walk of a row establishes.
///
/// A row decides three things at once — which Expressions it holds, which
/// Language Units they are spelled from, and which characters diagnose — and
/// they are decided by the same rules. Collecting them together is what keeps
/// the rules from being written twice: ADR 0018 has Expression construction
/// operate on the partition rather than reinterpret overlapping character
/// pairs, and one walk is how that is true rather than merely intended.
///
/// Spans cannot be derived afterwards from the units: a Span covers Cells that
/// produce no unit, so a lone unrecognized character is its own Span and no
/// arrangement of units says so. The walk has to emit both.
///
#[derive(Default)]
struct RowWalk {
    units: Vec<LanguageUnit>,
    parses: Vec<Parse>,
    diagnostics: Vec<Diagnostic>,
}

///
/// Where the Source of `row` ends: the first `##`, or the row's own edge.
///
/// This is the one place the Comment rule is stated. Nothing at or after the
/// introducer is Source, so the Parser is never shown it and no Language Unit
/// and no Expression can be spelled across it. The row edge needs no rule of
/// its own — it is where the slice the walk was handed stops.
///
/// The first `##` in the row is always the one the walk means. No Language
/// Unit spelling holds a `#`, so nothing the walk reads can begin before the
/// introducer and end after it.
///
fn source_end(row: &[u8]) -> usize {
    row.windows(2)
        .position(|pair| pair == b"##")
        .unwrap_or(row.len())
}

///
/// Walks one row left to right, appending everything it establishes to `walk`.
///
/// `row` holds exactly the Cells of one row and `row_start` is the Cell index
/// of its first column, so no byte of another row is reachable from here: an
/// Expression cannot straddle the row edge, a `##` cannot be spelled across
/// one, and a Language Unit ends where the slice does. The row edge needs no
/// rule of its own for the same reason — a two-Cell spelling that would cross
/// it simply is not there to read.
///
/// **A row is partitioned by parse.** The walk hands the Parser the row's
/// remaining Cells and the Cell they begin at, takes the Expression it
/// establishes, and resumes at the Cell after it. ADR 0033 records what that
/// makes of an empty Cell: one between Expressions is skipped rather than
/// named, and one inside an Expression's arity-determined claim is an operand
/// Cell that fails to bind, because a space no longer terminates anything.
///
fn walk_row(grid: Grid, row_start: usize, row: &[u8], walk: &mut RowWalk) {
    let cell = |idx: usize| {
        grid.cell_index(idx)
            .expect("a row's Cells lie inside the Grid that owns the row")
    };
    let source = &row[..source_end(row)];
    let text = std::str::from_utf8(source).expect("Source Cells contain ASCII");

    let mut idx = row_start;
    let end_of_source = row_start + source.len();
    while idx < end_of_source {
        if source[idx - row_start] == SPACE_BYTE {
            // An empty Cell between Expressions is not Source, so it is not
            // diagnosed and starts nothing. This is the whole of what a space
            // does now.
            idx += 1;
            continue;
        }

        let analysis = Parser::at(&text[idx - row_start..], idx).analyze();
        let cells = analysis.cells();
        name_units(grid, row_start, source, &analysis, walk);
        walk.parses.push(Parse {
            span: Span::new(grid, cell(cells.start), cell(cells.end - 1)),
            analysis,
        });
        idx = cells.end;
    }
}

///
/// Names complete Parser entries and reports the occupied Cells of invalid
/// entries. Missing ranges are empty; no unit is guessed from their spelling.
fn name_units(
    grid: Grid,
    row_start: usize,
    source: &[u8],
    analysis: &SourceAnalysis,
    walk: &mut RowWalk,
) {
    for entry in analysis.expression().positioned() {
        if entry.cells.is_empty() {
            continue;
        }
        let start = grid
            .cell_index(entry.cells.start)
            .expect("parsed Cell inside Grid");
        let end = grid
            .cell_index(entry.cells.end - 1)
            .expect("parsed Cell inside Grid");
        let kind = match entry.atom {
            Some(Atom::Function(function)) => Some(LanguageUnitKind::Function(function)),
            Some(Atom::Bang) => Some(LanguageUnitKind::Bang),
            Some(Atom::Activation(activation)) => Some(LanguageUnitKind::Activation(activation)),
            Some(Atom::Number(_) | Atom::Note(_) | Atom::Char(_)) => {
                Some(LanguageUnitKind::OperandLiteral)
            }
            _ => None,
        };
        if let Some(kind) = kind {
            walk.units.push(LanguageUnit {
                kind,
                anchor: grid.position_at(start),
                span: Span::new(grid, start, end),
            });
        } else {
            for index in entry.cells.clone() {
                let byte = source[index - row_start];
                if byte != SPACE_BYTE {
                    walk.diagnostics.push(invalid_unit_diagnostic(
                        grid,
                        grid.cell_index(index).expect("parsed Cell inside Grid"),
                        byte,
                    ));
                }
            }
        }
    }
}

///
/// The run of `len` row-major items belonging to each of `rows` rows.
///
/// `row_of` answers which row an item sits in. Items ascend by row, so one
/// pass names every run; a row holding nothing gets an empty one.
///
fn row_runs(rows: usize, len: usize, row_of: impl Fn(usize) -> usize) -> Vec<Range<usize>> {
    let mut runs = Vec::with_capacity(rows);
    let mut start = 0;
    for row in 0..rows {
        let mut end = start;
        while end < len && row_of(end) == row {
            end += 1;
        }
        runs.push(start..end);
        start = end;
    }
    runs
}

///
/// Walks a whole Source revision, row by row, in row-major order.
///
fn walk_source(grid: Grid, bytes: &[u8]) -> RowWalk {
    assert_eq!(
        bytes.len(),
        grid.count(),
        "LanguageMap Source length must match its Grid"
    );

    let cols = grid.cols();
    let mut walk = RowWalk::default();
    for (row_number, row) in bytes.chunks_exact(cols).enumerate() {
        walk_row(grid, row_number * cols, row, &mut walk);
    }

    // Rows are walked top to bottom and each row's column only ever advances,
    // so anchors ascend strictly and an Expression Span names a contiguous
    // run of this partition. Nothing downstream may reorder it.
    debug_assert!(
        walk.units.is_sorted_by_key(|unit| grid.index(unit.anchor)),
        "Language Units are partitioned in ascending anchor order"
    );

    walk
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
/// `walk_source` establishes its units in ascending anchor order, so an
/// Expression Span names a contiguous run of them and both ends are found by
/// search rather than by testing every unit against every Span. The returned
/// bounds are positions in `units`, a different index space from the Cell
/// indices the Span carries; the Span's own bounds are inclusive at both ends.
fn units_range(units: &[LanguageUnit], grid: Grid, span: Span) -> std::ops::Range<usize> {
    let first = units.partition_point(|unit| grid.index(unit.anchor) < span.start());
    let past_last = units.partition_point(|unit| grid.index(unit.anchor) <= span.end());
    first..past_last
}

#[cfg(test)]
mod tests {
    use crate::{glyph::Glyph, grid::Grid};

    use lang::{Activation, Atom};

    use super::{LanguageMap, LanguageUnitKind, Span, walk_source};

    #[test]
    fn invalid_operand_bang_spellings_are_not_parsed_bang_values() {
        // A `**` inside a Function's arity-determined claim is that slot's
        // Source and not a Bang, whether the slot is reached with characters
        // to spare or with the Source running out under it. Both Expressions
        // report rather than answer, so neither contributes a Bang.
        for source in ["!>00**C4", "!>**7F  "] {
            let grid = Grid::new(8, 2);
            let map = LanguageMap::build(grid, format!("{source}        ").as_bytes());
            assert_eq!(map.bangs().count(), 0, "{source}");
            assert!(
                !map.units()
                    .any(|unit| unit.kind() == LanguageUnitKind::Bang),
                "{source}"
            );
        }
    }

    #[test]
    fn a_bang_outside_every_claim_is_a_bang_however_the_source_around_it_reads() {
        // The other side of the rule above, and the one ADR 0033 changed: a
        // `**` no Function claims is a Bang, and the invalid Source beside it
        // costs only its own Cells. `**X0**` was one refused six-Cell run
        // before the partition was decided by the parse.
        let grid = Grid::new(6, 1);
        let map = LanguageMap::build(grid, b"**X0**");

        assert_eq!(
            map.bangs()
                .map(|(anchor, span)| (anchor.x(), span.start().get(), span.end().get()))
                .collect::<Vec<_>>(),
            vec![(0, 0, 1), (4, 4, 5)],
        );
    }

    #[test]
    fn parsed_function_candidates_survive_missing_or_invalid_operands() {
        for source in ["!>", "!>007F", "!>00**C4"] {
            let grid = Grid::new(source.len(), 1);
            let map = LanguageMap::build(grid, source.as_bytes());
            let expression = map.expressions().next().unwrap();
            assert_eq!(
                expression.function_candidate(),
                Some((
                    grid.position(0, 0).unwrap(),
                    lang::Function::try_from("!>").unwrap()
                )),
                "{source}",
            );
            assert!(expression.root().is_none(), "{source}");
        }
    }

    #[test]
    fn expression_layout_retains_slots_beyond_invalid_and_missing_source() {
        for source in ["!>**7F", "!>00  "] {
            let grid = Grid::new(12, 1);
            let map = LanguageMap::build(grid, format!("{source}      ").as_bytes());
            let expression = map.expressions().next().unwrap();
            assert_eq!(
                expression
                    .positioned()
                    .map(|entry| (entry.cells.start, entry.token))
                    .collect::<Vec<_>>(),
                vec![
                    (0, lang::Token::Function),
                    (2, lang::Token::Number),
                    (4, lang::Token::Number),
                    (6, lang::Token::Note)
                ]
            );
            assert!(expression.atoms().is_none());
        }
    }

    #[test]
    fn a_malformed_prefix_costs_its_own_cells_and_leaves_the_function_after_it_recognised() {
        // The Source before a Function no longer decides whether that Function
        // is read. Each prefix here is refused a Cell at a time and the Play
        // that follows it is a candidate anchored where it is spelled, which
        // is what ADR 0033's partition buys a person mid-keystroke: a mistyped
        // Cell costs that Cell rather than the rest of the row.
        for (source, column) in [("XX!>007FC4", 2), ("**!>007FC4", 2), ("0!>007FC4", 1)] {
            let grid = Grid::new(source.len(), 1);
            let map = LanguageMap::build(grid, source.as_bytes());
            assert_eq!(
                map.expressions()
                    .filter_map(|expression| expression.function_candidate())
                    .map(|(anchor, function)| (anchor.x(), function))
                    .collect::<Vec<_>>(),
                vec![(column, lang::Function::try_from("!>").unwrap())],
                "{source}",
            );
        }
    }

    #[test]
    fn adjacent_standalone_bangs_have_distinct_parsed_spans() {
        let grid = Grid::new(6, 1);
        let map = LanguageMap::build(grid, b"**>>**");
        assert_eq!(
            map.bangs()
                .map(|(anchor, span)| (anchor.x(), span.start().get(), span.end().get()))
                .collect::<Vec<_>>(),
            vec![(0, 0, 1), (4, 4, 5)],
        );
    }

    ///
    /// The Expression Spans of a whole Source revision, in row-major order.
    ///
    fn expression_spans(grid: Grid, bytes: &[u8]) -> Vec<Span> {
        walk_source(grid, bytes)
            .parses
            .into_iter()
            .map(|parse| parse.span)
            .collect()
    }

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
        let grid = Grid::new(14, 1);
        let map = LanguageMap::derive(grid, ".+0102 .-0304 ").unwrap();
        let expressions = map.expressions().collect::<Vec<_>>();

        let anchors = |expression| {
            map.expression_units(expression)
                .iter()
                .map(|unit| (unit.anchor().x(), unit.anchor().y()))
                .collect::<Vec<_>>()
        };

        assert_eq!(expressions.len(), 2);
        assert_eq!(anchors(expressions[0]), vec![(0, 0), (2, 0), (4, 0)]);
        assert_eq!(anchors(expressions[1]), vec![(7, 0), (9, 0), (11, 0)]);
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
        // Every unit kind, and each one inside the Expression that claims it:
        // an Operand Literal is spelled in a Function's slot, because a pair
        // of hexadecimal characters standing on its own is no longer part of
        // any Expression for a unit to belong to.
        let map = LanguageMap::build(Grid::new(12, 2), b".+C4**>>    ^^vv<<.+00  ");

        assert_eq!(
            map.units().map(|unit| unit.kind()).collect::<Vec<_>>(),
            vec![
                LanguageUnitKind::Function(lang::Function::Add),
                LanguageUnitKind::OperandLiteral,
                LanguageUnitKind::Activation(Activation::East),
                LanguageUnitKind::Activation(Activation::North),
                LanguageUnitKind::Activation(Activation::South),
                LanguageUnitKind::Activation(Activation::West),
                LanguageUnitKind::Function(lang::Function::Add),
                LanguageUnitKind::OperandLiteral,
            ]
        );

        assert_eq!(
            unit_spellings(&map),
            vec![
                (0, vec![0, 1]),
                (2, vec![2, 3]),
                (6, vec![6, 7]),
                (0, vec![0, 1]),
                (2, vec![2, 3]),
                (4, vec![4, 5]),
                (6, vec![6, 7]),
                (8, vec![8, 9]),
            ]
        );
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
        assert_eq!(unit_spellings(&fragment), vec![(0, vec![0, 1])]);
        assert!(
            fragment
                .diagnostics()
                .any(|diagnostic| diagnostic.message.contains("#"))
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
    fn build_names_one_span_per_expression_covering_it_inclusively() {
        // An Addition takes two operands, so it claims six Cells from its
        // anchor and the Span says so. The `1` and the space after it are the
        // first operand, which is why they are inside the Span rather than
        // beside it: a space no longer ends anything, so what bounds this
        // Expression is arity.
        let grid = Grid::new(5, 1);
        let spans = expression_spans(grid, b" .+1 ");

        assert_eq!(spans, vec![span(grid, 1, 4)]);
        assert_eq!(
            spans[0].indices().map(|idx| idx.get()).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
    }

    #[test]
    fn a_cell_that_produces_no_language_unit_is_still_its_own_span() {
        // Spans are not recoverable from the units, which is why the walk emits
        // both rather than deriving one from the other: neither of these Cells
        // is half of a two-Cell spelling, so the partition names no unit for
        // either, and each is still a Span in its own right.
        let grid = Grid::new(5, 1);
        let map = LanguageMap::build(grid, b" x  x");

        assert!(map.units().next().is_none());
        assert_eq!(
            expression_spans(grid, b" x  x"),
            vec![span(grid, 1, 1), span(grid, 4, 4)]
        );
    }

    #[test]
    fn build_separates_the_expressions_in_one_row() {
        let grid = Grid::new(16, 1);

        // asserting the whole list, not Cell by Cell: a spurious extra Span
        // shows up here and would not show up in per-Cell probing. Both
        // Additions are spelled whole, so each ends where its arity says and
        // the spaces between them belong to neither.
        assert_eq!(
            expression_spans(grid, b".+0102  .-0304  "),
            vec![span(grid, 0, 5), span(grid, 8, 13)]
        );
    }

    #[test]
    fn a_space_inside_an_expressions_claim_is_an_operand_cell_and_not_a_boundary() {
        // ADR 0033's space rule, and the one behaviour change a reader is most
        // likely to be surprised by. `.+` claims six Cells whatever they hold,
        // so the two spaces and the `.-` after them are its operands rather
        // than the next Expression: one Span, not two.
        let grid = Grid::new(8, 1);

        assert_eq!(expression_spans(grid, b".+  .-  "), vec![span(grid, 0, 7)]);
    }

    #[test]
    fn build_keeps_edge_touching_expressions_inside_their_rows() {
        let grid = Grid::new(4, 2);

        // the Expressions touch across the row edge but are two Spans, not
        // one: each claims what is left of its own row and stops there
        assert_eq!(
            expression_spans(grid, b"  .+.-  "),
            vec![span(grid, 2, 3), span(grid, 4, 7)]
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
        // The `x` is where the next Expression begins, and a Function is what
        // begins one.
        assert_eq!(at(7), Some(Glyph::Function));
        assert_eq!(map.expression_diagnostics().count(), 1);
    }

    #[test]
    fn a_run_of_standalone_atoms_is_as_many_expressions_as_the_parser_finds() {
        // ADR 0018 already said `**^^` is a Bang and then a Self-Banging
        // Function; ADR 0033 makes the partition say it too. Each is a whole
        // Expression with a Span of its own and Atoms of its own, where the
        // assembly path this replaces reported one four-Cell Expression that
        // no single parse ever produced.
        let grid = Grid::new(4, 1);
        let map = LanguageMap::build(grid, b"**^^");

        assert_eq!(
            map.expressions()
                .map(|expression| (
                    expression.span().start().get(),
                    expression.span().end().get(),
                    expression.atoms().map(|atoms| atoms.as_slice().to_vec()),
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, 1, Some(vec![Atom::Bang])),
                (2, 3, Some(vec![Atom::Activation(Activation::North)])),
            ],
        );
        assert_eq!(map.diagnostics().count(), 0);
    }

    #[test]
    fn an_odd_standalone_run_costs_one_cell_and_leaves_the_rest_readable() {
        // ADR 0018's `***`: a Bang and one invalid `*`. The Bang is a whole
        // Expression that answers with Atoms, so a Tick can execute it — the
        // stray character costs the Cell it occupies and nothing more. The
        // trailing-content verdict this replaces refused the Bang along with
        // it.
        let map = LanguageMap::build(Grid::new(3, 1), b"***");

        assert_eq!(
            map.expressions()
                .map(|expression| expression.atoms().map(|atoms| atoms.as_slice().to_vec()))
                .collect::<Vec<_>>(),
            vec![Some(vec![Atom::Bang]), None],
        );
        assert_eq!(map.bangs().count(), 1);
    }

    #[test]
    fn language_map_partitions_adjacent_standalone_units() {
        let bang_grid = Grid::new(4, 1);
        let bangs = LanguageMap::build(bang_grid, b"****");
        let activations = LanguageMap::build(Grid::new(4, 1), b">>>>");

        // Two Bangs, and two Expressions: a standalone Atom is one whole
        // Expression, so a run of them is a run of Expressions rather than one
        // Expression holding several Atoms.
        assert_eq!(
            bangs
                .expressions()
                .map(|expression| expression.atoms().unwrap().as_slice().to_vec())
                .collect::<Vec<_>>(),
            vec![vec![Atom::Bang], vec![Atom::Bang]]
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
                .map(|expression| expression.atoms().unwrap().as_slice().to_vec())
                .collect::<Vec<_>>(),
            vec![
                vec![Atom::Activation(Activation::East)],
                vec![Atom::Activation(Activation::East)],
            ]
        );
        assert_eq!(activations.diagnostics().count(), 0);
    }
}

///
/// Permissive Source analysis is total over printable ASCII.
///
/// A Language Map is derived from whatever the Grid holds when it is read, so
/// Live Editing puts every revision between two keystrokes through this path:
/// half-typed Functions, operands with one Cell written, a `#` that is not yet
/// a Comment. Deriving one has to answer for all of them rather than panic,
/// and the answer has to keep the rules `CONTEXT.md` states — a row is
/// partitioned left to right into non-overlapping complete Language Units, an
/// unmatched character diagnoses without participating in an overlapping unit,
/// and incomplete or invalid Source contributes no runtime Atoms.
///
/// The Source is generated as raw text rather than as Expressions. What makes
/// this path worth a property is exactly the input a grammar-shaped generator
/// would never produce, so the space that ends a run, the `#` that is
/// incomplete Source, and the `##` Comment introducer are drawn as characters
/// like everything else.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {
    use super::{LanguageMap, Parser, SPACE_BYTE};
    use crate::grid::Grid;
    use lang::{Activation, Atom, Function, Note, Token};
    use proptest::prelude::*;
    use proptest::sample::select;
    use proptest::test_runner::{Config, TestRunner};
    // Aliased because `Cell` is a domain noun throughout this file and in
    // CONTEXT.md. What `std::cell::Cell` holds here is a draw count.
    use std::cell::Cell as Counter;

    /// The widest and tallest Grid a case is derived over. Wide enough to hold
    /// several runs and a Comment in one row, and short enough that a shrunk
    /// counterexample is still readable as Source.
    const COLS: usize = 12;
    const ROWS: usize = 3;

    /// The standalone Language Unit spellings: the Bang and every Activation,
    /// read from the Atoms themselves rather than restated, and from
    /// `Activation::ALL` so a fifth Activation is drawn the day it is
    /// declared.
    fn standalone_spellings() -> Vec<String> {
        std::iter::once(Atom::Bang)
            .chain(Activation::ALL.iter().copied().map(Atom::Activation))
            .map(|atom| atom.to_string())
            .collect()
    }

    /// One Function spelled with a literal in each operand position it
    /// declares: the shape the walk reads as a whole Expression.
    ///
    /// This is the minority branch, and it is here because the coverage guard
    /// below proved it was needed — without it, no revision in 256 cases held
    /// a Function the walk read whole, so both properties' Expression arms
    /// were reached only by standalone runs, which `standalone_run` answers
    /// without ever calling the Parser.
    ///
    /// `lang` keeps a Function's signature crate-private, so the operand count
    /// and types are not restated here: a Function is drawn with between one
    /// and four literals after it and the draws that do not spell a whole
    /// Expression are filtered out. Strict parsing is what selects them, and
    /// the properties state what permissive analysis and the walk do with
    /// them, so the generator is not deciding the question it feeds.
    fn complete_expression() -> BoxedStrategy<String> {
        (
            select(Function::ALL),
            prop::collection::vec(
                prop_oneof![literal_source(Token::Number), literal_source(Token::Note)],
                1..=4,
            ),
        )
            .prop_map(|(function, operands)| format!("{function}{}", operands.concat()))
            .prop_filter("one whole Expression", |source| {
                let mut source = source.clone();
                Parser::from(&mut source).try_parse().is_ok()
            })
            .boxed()
    }

    /// Source text for one Operand Literal of the type its position declares.
    ///
    /// ADR 0021 makes an Operand Literal's type the consuming Function's
    /// rather than the Source's, so a literal is spelled against the `Token`
    /// the slot declares.
    fn literal_source(token: Token) -> BoxedStrategy<String> {
        match token {
            Token::Number => any::<u8>()
                .prop_map(|number| Atom::Number(number).to_string())
                .boxed(),
            Token::Note => (0x00u8..=0x7F)
                .prop_map(|note| Atom::Note(Note::try_from(note).expect("a MIDI Note")).to_string())
                .boxed(),
            other => panic!("no operand is declared as {other:?}"),
        }
    }

    /// One piece of generated Source text.
    ///
    /// Most of the weight is one arbitrary printable character, which is what
    /// keeps the whole range a Cell can hold in reach. The rest are the pieces
    /// the walk gives meaning to, so that a row is more often a near miss than
    /// noise: the space and the `##` introducer that end a run, the `#` that is
    /// incomplete Source rather than a Comment, a Function spelling, a
    /// standalone Atom, an Operand Literal, and — as the minority branch — a
    /// whole Function with its operands. They are concatenated in whatever
    /// order they are drawn and then cut to the Grid, so what reaches the walk
    /// is raw text rather than a grammar.
    ///
    /// `lang::parser`'s `mod property` has a fragment generator of the same
    /// shape, and the two are deliberately separate: `orcvs` depends on `lang`,
    /// so sharing one would mean a test-support module in `lang` compiled into
    /// a dependency for the sake of a test. The duplication is recorded here
    /// rather than left to be discovered — a new run boundary or a second
    /// Comment form has to be taught to both, and neither coverage guard
    /// notices if only one learns it.
    fn fragment() -> BoxedStrategy<String> {
        prop_oneof![
            8 => proptest::char::range(' ', '~').prop_map(String::from),
            2 => Just(" ".to_owned()),
            2 => Just("#".to_owned()),
            2 => Just("##".to_owned()),
            2 => select(Function::ALL).prop_map(|function| function.to_string()),
            1 => select(standalone_spellings()),
            2 => prop_oneof![literal_source(Token::Number), literal_source(Token::Note)],
            3 => complete_expression(),
        ]
        .boxed()
    }

    /// Exactly `cells` printable ASCII characters: the fragments drawn, cut to
    /// the Grid's Cell count and padded with the empty Cell. Cutting is what
    /// makes a fragment's own shape unreliable, which is the point — a `##`
    /// severed by the row edge is Source a Live Edit reaches.
    fn source_text(cells: usize) -> BoxedStrategy<String> {
        prop::collection::vec(fragment(), 1..=cells)
            .prop_map(move |fragments| {
                let mut text = fragments.concat();
                text.truncate(cells);
                while text.len() < cells {
                    text.push(char::from(SPACE_BYTE));
                }
                text
            })
            .boxed()
    }

    /// One Source revision: a Grid's shape, and exactly one printable ASCII
    /// Cell per Position in it.
    fn revision() -> BoxedStrategy<(usize, usize, String)> {
        (1usize..=COLS, 1usize..=ROWS)
            .prop_flat_map(|(cols, rows)| (Just(cols), Just(rows), source_text(cols * rows)))
            .boxed()
    }

    proptest! {
        ///
        /// Deriving a Language Map from any Source answers, and answers with
        /// the partition `CONTEXT.md` describes: each row left to right into
        /// non-overlapping two-Cell units, with an unmatched character
        /// diagnosed on its own Cell and the walk resuming one Cell later.
        ///
        /// The Comment introducer is the one place the walk stops early, and
        /// this reads it back with a plain search for `##` rather than by
        /// re-walking the row. That is an independent statement of the rule
        /// rather than a copy of the implementation: it holds only because no
        /// Language Unit spelling contains a `#`, so no unit can step over the
        /// introducer and the first `##` in a row is always the one the walk
        /// meets.
        ///
        #[test]
        fn deriving_a_language_map_partitions_every_row_at_the_cell_recovery_resumes_from(
            (cols, rows, source) in revision(),
        ) {
            let grid = Grid::new(cols, rows);
            let map = LanguageMap::derive(grid, &source)
                .expect("one printable ASCII Cell per Position");
            let bytes = source.as_bytes();

            // Which Language Unit claims each Cell, if any.
            let mut claimed: Vec<Option<usize>> = vec![None; bytes.len()];
            let mut previous_anchor: Option<usize> = None;
            for (ordinal, unit) in map.units().enumerate() {
                let span = unit.span();

                prop_assert_eq!(span.end().get(), span.start().get() + 1);
                prop_assert_eq!(unit.anchor(), grid.position_at(span.start()));
                prop_assert_eq!(unit.anchor().y(), grid.position_at(span.end()).y());
                prop_assert!(previous_anchor < Some(span.start().get()), "{:?}", source);
                previous_anchor = Some(span.start().get());

                for idx in span.indices() {
                    prop_assert!(
                        claimed[idx.get()].is_none(),
                        "{:?} gave Cell {} to two Language Units",
                        source,
                        idx.get(),
                    );
                    claimed[idx.get()] = Some(ordinal);
                }
            }

            // How many times each Cell was diagnosed as an unmatched character.
            let mut diagnosed = vec![0usize; bytes.len()];
            for diagnostic in &map.lexical_diagnostics {
                prop_assert_eq!(diagnostic.start(), diagnostic.end());
                diagnosed[diagnostic.start()] += 1;
            }

            for row in 0..rows {
                let row_start = row * cols;
                let row_bytes = &bytes[row_start..row_start + cols];
                let comment = row_bytes
                    .windows(2)
                    .position(|pair| pair == b"##")
                    .unwrap_or(cols);

                for (column, byte) in row_bytes.iter().copied().enumerate() {
                    let idx = row_start + column;
                    let claims = usize::from(claimed[idx].is_some()) + diagnosed[idx];

                    if column >= comment || byte == SPACE_BYTE {
                        // A Comment is not Source and an empty Cell spells
                        // nothing, so neither is named or diagnosed.
                        prop_assert_eq!(
                            claims,
                            0,
                            "{:?} named Cell {} of a Comment or an empty Cell",
                            source,
                            idx,
                        );
                    } else {
                        // Every other Cell is named exactly once: by the one
                        // unit that covers it, or by the one diagnostic
                        // recovery left behind before advancing past it.
                        prop_assert_eq!(
                            claims,
                            1,
                            "{:?} named Cell {} {} times",
                            source,
                            idx,
                            claims,
                        );
                    }
                }
            }
        }

        ///
        /// Every Expression and every Diagnostic a revision holds answers to
        /// that revision, and incomplete or invalid Source contributes no
        /// runtime Atoms.
        ///
        /// A Span is Cell numbers and the Grid that minted them, and two
        /// revisions of one Source share a Grid, so the two halves of "the same
        /// revision" are asserted together: a Diagnostic's Positions come from
        /// the Grid this Map was derived over, and the Expression carrying it
        /// is one this Map can answer for. `expression_units` is what asks the
        /// second question, and it refuses a foreign Expression rather than
        /// answering with the wrong units.
        ///
        #[test]
        fn every_expression_and_diagnostic_answers_to_the_revision_that_derived_it(
            (cols, rows, source) in revision(),
        ) {
            let grid = Grid::new(cols, rows);
            let map = LanguageMap::derive(grid, &source)
                .expect("one printable ASCII Cell per Position");

            for expression in map.expressions() {
                prop_assert_eq!(expression.map_id, map.id);

                let span = expression.span();
                let units = map.expression_units(expression);

                // Every named unit lies wholly inside the Expression's Span,
                // and the named units run in Source order without a gap. This
                // is containment of the whole unit rather than of its anchor,
                // which is what `units_range` searches on, so it states what
                // the range is for instead of re-deriving it the way the
                // implementation does: a unit that began inside the Span and
                // ran past its end would satisfy the anchor search and fail
                // here.
                let mut previous_end: Option<usize> = None;
                for unit in units {
                    let unit_span = unit.span();
                    prop_assert!(
                        span.start() <= unit_span.start() && unit_span.end() <= span.end(),
                        "{:?} named a unit at {}..={} outside the Span {}..={}",
                        source,
                        unit_span.start().get(),
                        unit_span.end().get(),
                        span.start().get(),
                        span.end().get(),
                    );
                    prop_assert!(previous_end < Some(unit_span.start().get()), "{:?}", source);
                    previous_end = Some(unit_span.end().get());
                }

                // A value never stands in for Source that was not read: an
                // Expression answers with Atoms exactly when it has nothing to
                // report, and a root is only ever the anchor of one that does.
                prop_assert_eq!(
                    expression.atoms().is_some(),
                    expression.diagnostic.is_none(),
                    "{:?}",
                    source,
                );
                // And the Atoms answer for the very Cells they were read
                // from. `standalone_run` builds an Expression straight from
                // the partition's unit kinds without going through the Parser,
                // so this is the one check that its Atoms spell the Source
                // they claim rather than merely being present.
                if let Some(atoms) = expression.atoms() {
                    let rendered: String = atoms.iter().map(|atom| atom.to_string()).collect();
                    prop_assert_eq!(
                        rendered.as_str(),
                        &source[span.start().get()..=span.end().get()],
                        "{:?}",
                        source,
                    );
                }
                if let Some(root) = expression.root() {
                    prop_assert!(expression.atoms().is_some(), "{:?}", source);
                    prop_assert!(span.positions().any(|position| position == root));
                }
                if let Some(diagnostic) = &expression.diagnostic {
                    prop_assert_eq!(diagnostic.span(), span, "{:?}", source);
                }
            }

            for diagnostic in map.diagnostics() {
                prop_assert_eq!(diagnostic.span().grid, grid);
                prop_assert!(diagnostic.start() <= diagnostic.end());
                // On the raw Cell numbers rather than on `positions()`:
                // `Span::indices` filters out any index the Grid cannot answer
                // for, so a Position sweep holds vacuously for a Span that
                // runs off the end of the Grid — exactly the Span it should
                // reject.
                prop_assert!(
                    diagnostic.end() < grid.count(),
                    "{:?} diagnosed Cell {} of a {}-Cell Grid",
                    source,
                    diagnostic.end(),
                    grid.count(),
                );
                prop_assert!(
                    diagnostic
                        .span()
                        .positions()
                        .any(|position| position == diagnostic.anchor()),
                );
                // An Expression never wraps, so neither does a Diagnostic about
                // one, and a Position of another row would be a Position of
                // another revision's reading of the same Cells.
                prop_assert_eq!(diagnostic.start() / cols, diagnostic.end() / cols);
            }
        }

        ///
        /// The Expression Spans of one revision are disjoint: no Cell belongs
        /// to two of them, their start Cells ascend, none crosses a row edge,
        /// and every one names Cells the Grid can answer for. Every Cell that
        /// is not empty is left with a Glyph.
        ///
        /// Disjoint, not a partition. The unit property above proves cover as
        /// well, because every Cell carries a Language Unit claim. Expression
        /// Spans leave gaps by design, and the `Glyph::Char` fallback exists to
        /// classify exactly those Cells no Span claimed, so there is no cover
        /// law here to assert.
        ///
        /// The unit partition property above proves the same law one index
        /// space down, and cannot stand in for this one. A Language Unit is
        /// always two Cells and is always spelled; an Expression Span is
        /// whatever the Parser claimed, which includes Cells that spell no unit
        /// at all — `a_cell_that_produces_no_language_unit_is_still_its_own_span`
        /// is one. ADR 0033 is what makes the difference worth a property:
        /// `walk_row` no longer decides where an Expression stops, it resumes
        /// at the Cell the Parser reports, so a Span is the Parser's answer
        /// taken on trust and this is where the trust is checked.
        ///
        /// The Grid bounds are asserted on the raw Cell numbers rather than
        /// through `positions()`, for the reason the Diagnostic arm above
        /// gives: `Span::indices` drops every index the Grid cannot answer for,
        /// so a Span running past the last Cell passes a Position sweep without
        /// reporting anything.
        ///
        /// The Glyph arm is the other half of "every Cell is accounted for".
        /// The Map gives a Cell an Expression claimed the Glyph of its parsed
        /// Token, and `Glyph::Char` to every other non-empty Cell, so only an
        /// empty Cell can be left unclassified. The converse is false and is
        /// not asserted: an empty Cell inside an Expression's arity-determined
        /// claim is an operand Cell and answers with that operand's Glyph.
        ///
        #[test]
        fn expression_spans_are_disjoint_and_name_cells_the_grid_can_answer_for(
            (cols, rows, source) in revision(),
        ) {
            let grid = Grid::new(cols, rows);
            let map = LanguageMap::derive(grid, &source)
                .expect("one printable ASCII Cell per Position");
            let bytes = source.as_bytes();

            // Which Expression claims each Cell, if any.
            let mut claimed: Vec<Option<usize>> = vec![None; bytes.len()];
            let mut previous_start: Option<usize> = None;
            for (ordinal, expression) in map.expressions().enumerate() {
                let span = expression.span();
                let (start, end) = (span.start().get(), span.end().get());

                // The raw Cell numbers below are compared against this Grid's
                // count, and `Span::new` does not check that its `CellIndex`
                // arguments were minted by its own Grid. The Diagnostic arm
                // above pins the Grid down for the same reason.
                prop_assert_eq!(span.grid, grid, "{:?}", source);
                prop_assert!(start <= end, "{:?}", source);
                // On the raw Cell numbers, so a Span past the end of the Grid
                // is caught here rather than filtered away by `Span::indices`.
                prop_assert!(
                    end < grid.count(),
                    "{:?} spanned Cell {} of a {}-Cell Grid",
                    source,
                    end,
                    grid.count(),
                );
                // A row is the whole horizontal run there is, so a Span's first
                // and last Cell are in the same one.
                prop_assert_eq!(
                    start / cols,
                    end / cols,
                    "{:?} spanned {}..={} across a row edge",
                    source,
                    start,
                    end,
                );
                prop_assert!(
                    previous_start < Some(start),
                    "{:?} named an Expression Span starting at Cell {} out of order",
                    source,
                    start,
                );
                previous_start = Some(start);

                // Every pair rather than only the neighbouring one: two Spans
                // that are not adjacent in this order still may not overlap.
                for (offset, claim) in claimed[start..=end].iter_mut().enumerate() {
                    prop_assert!(
                        claim.is_none(),
                        "{:?} gave Cell {} to two Expressions",
                        source,
                        start + offset,
                    );
                    *claim = Some(ordinal);
                }
            }

            for (position, byte) in grid.rows().flatten().zip(bytes.iter().copied()) {
                if byte != SPACE_BYTE {
                    prop_assert!(
                        map.glyph_at(position).is_some(),
                        "{:?} left the non-empty Cell {} unclassified",
                        source,
                        grid.index(position).get(),
                    );
                }
            }
        }
    }

    ///
    /// The generated revisions reach the Source that makes the properties above
    /// worth stating: an empty Cell, a `#` that is incomplete Source, a `##`
    /// Comment introducer, a character no Language Unit spelling matches, and
    /// an Expression complete enough to answer with Atoms.
    ///
    /// Neither property can tell Source it never saw from Source it saw and
    /// handled, and the partition property's Comment rule in particular is a
    /// claim about a branch a grammar-shaped generator would never take.
    /// Driving the runner directly is what lets the draws be counted across
    /// cases and asserted afterwards, where `proptest!` would have had nowhere
    /// to put the count.
    ///
    /// The case count is pinned rather than taken from `PROPTEST_CASES`,
    /// because this claim is about the generator rather than about the
    /// derivation. It does derive a Language Map from each draw — that is how
    /// the last two of the five counts are taken — so the fixed 256 cases are
    /// 256 derivations that neither verification tier can dial down. That is
    /// the cost of the claim rather than an oversight: a coverage guard that
    /// weakened with the tier would stop guarding exactly where the tier is
    /// cheapest.
    ///
    #[test]
    fn generated_revisions_cover_empty_cells_comments_and_complete_expressions() {
        let config = Config {
            cases: 256,
            source_file: Some(file!()),
            ..Config::default()
        };
        let empty = Counter::new(0usize);
        let incomplete = Counter::new(0usize);
        let comment = Counter::new(0usize);
        let unmatched = Counter::new(0usize);
        let complete = Counter::new(0usize);

        TestRunner::new(config)
            .run(&revision(), |(cols, rows, source)| {
                let map = LanguageMap::derive(Grid::new(cols, rows), &source)
                    .expect("one printable ASCII Cell per Position");
                let bytes = source.as_bytes();

                if bytes.contains(&SPACE_BYTE) {
                    empty.set(empty.get() + 1);
                }
                if bytes
                    .chunks_exact(cols)
                    .any(|row| row.windows(2).any(|pair| pair == b"##"))
                {
                    comment.set(comment.get() + 1);
                }
                // A `#` with no `#` beside it in its own row: incomplete Source
                // rather than the introducer, which is the distinction
                // CONTEXT.md draws.
                if bytes.chunks_exact(cols).any(|row| {
                    row.iter().enumerate().any(|(column, byte)| {
                        *byte == b'#'
                            && row.get(column + 1) != Some(&b'#')
                            && (column == 0 || row[column - 1] != b'#')
                    })
                }) {
                    incomplete.set(incomplete.get() + 1);
                }
                if !map.lexical_diagnostics.is_empty() {
                    unmatched.set(unmatched.get() + 1);
                }
                // A Function among the Atoms rather than merely an Expression
                // that answers: a lone `**` or `<<` is answered by
                // `standalone_run` without the Parser ever being called, so
                // counting any answering Expression would let this guard pass
                // while the Function-bearing arms of both properties above had
                // never once run.
                if map.expressions().any(|expression| {
                    expression.atoms().is_some_and(|atoms| {
                        atoms.iter().any(|atom| matches!(atom, Atom::Function(_)))
                    })
                }) {
                    complete.set(complete.get() + 1);
                }
                Ok(())
            })
            .unwrap_or_else(|error| panic!("{error}"));

        assert!(empty.get() > 0, "no generated revision held an empty Cell");
        assert!(
            incomplete.get() > 0,
            "no generated revision held an incomplete `#`",
        );
        assert!(
            comment.get() > 0,
            "no generated revision held the `##` Comment introducer",
        );
        assert!(
            unmatched.get() > 0,
            "no generated revision held an unmatched character",
        );
        assert!(
            complete.get() > 0,
            "no generated revision held a Function the walk read as a whole Expression",
        );
    }
}

///
/// A rebuilt Map must be the Map a full build would have produced.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod rebuild_property {
    use super::{Glyph, Grid, LanguageMap, LanguageUnit};
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    /// The Cells a Source is built from: Function spellings, operands, Bang,
    /// the comment that ends a row, and the space that separates runs.
    const ALPHABET: &[u8] = b".+=x><0123456789ABCDEF*# ";

    ///
    /// Everything a Map holds except its identity, in a form two Maps can be
    /// compared by.
    ///
    /// The identity is left out on purpose: a rebuilt Map is a new revision
    /// and says so, exactly as a built one does. Everything else has to agree
    /// Cell for Cell, unit for unit, and diagnostic for diagnostic.
    ///
    /// One lexical diagnostic, as its Cells and its message.
    type ReportedDiagnostic = (usize, usize, String);

    /// One Expression, as its Span, its diagnostic, and the units its range
    /// resolves to.
    type ReportedExpression = (usize, usize, Option<String>, Vec<LanguageUnit>);

    /// Everything two Maps are compared by.
    type Contents = (
        Vec<LanguageUnit>,
        Vec<Option<Glyph>>,
        Vec<ReportedDiagnostic>,
        Vec<ReportedExpression>,
    );

    fn contents(map: &LanguageMap) -> Contents {
        (
            map.units.clone(),
            map.glyphs.clone(),
            map.lexical_diagnostics
                .iter()
                .map(|diagnostic| {
                    (
                        diagnostic.start(),
                        diagnostic.end(),
                        diagnostic.message.clone(),
                    )
                })
                .collect(),
            map.expressions
                .iter()
                .map(|entry| {
                    (
                        entry.span.start().get(),
                        entry.span.end().get(),
                        entry
                            .diagnostic
                            .as_ref()
                            .map(|diagnostic| diagnostic.message.clone()),
                        // Resolved through the Map that owns them, so a range
                        // carried across a rebuild is checked by what it
                        // actually points at rather than by its numbers.
                        map.expression_units(entry).to_vec(),
                    )
                })
                .collect(),
        )
    }

    proptest! {
        ///
        /// Write to some rows, then rebuild only those rows.
        ///
        /// The written rows are the only ones whose Cells change, which is the
        /// contract `rebuild` is given. Rows are written with fresh content
        /// rather than mutated, so a row can gain or lose Language Units and
        /// move every later Expression's range in the partition — the case the
        /// carried ranges exist to survive.
        ///
        #[test]
        fn a_rebuilt_map_equals_the_map_a_full_build_would_have_made(
            cols in 4usize..14,
            rows in 1usize..7,
            // Sized to the largest Grid the dimensions above can name, then
            // cut to the one they did, so no case is generated only to be
            // rejected for being too short.
            before in prop::collection::vec(0usize..ALPHABET.len(), 13 * 6),
            after in prop::collection::vec(0usize..ALPHABET.len(), 13 * 6),
            written in prop::collection::vec(any::<bool>(), 6),
        ) {
            let count = cols * rows;

            let grid = Grid::new(cols, rows);
            let mut bytes: Vec<u8> = before[..count].iter().map(|i| ALPHABET[*i]).collect();
            let previous = LanguageMap::build(grid, &bytes);

            let dirty: BTreeSet<usize> = (0..rows)
                .filter(|row| *written.get(*row).unwrap_or(&false))
                .collect();
            for row in &dirty {
                for column in 0..cols {
                    let cell = row * cols + column;
                    bytes[cell] = ALPHABET[after[cell]];
                }
            }

            let rebuilt = LanguageMap::rebuild(&previous, grid, &bytes, &dirty);
            let built = LanguageMap::build(grid, &bytes);

            prop_assert_eq!(contents(&rebuilt), contents(&built));
        }
    }
}
