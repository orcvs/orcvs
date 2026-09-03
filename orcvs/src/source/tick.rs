//! ADR 0020's producer and effect model for one Tick.
//!
//! Tick planning is one row-major pass over the Language Map derived from one
//! Source Snapshot. Every actionable Language Unit and Expression root has one
//! producer Position — its anchor — and takes at most one turn. A producer
//! emits an ordered sequence of effects, and resolution folds those effects,
//! in producer-then-emission order, into the Tick Plan.
//!
//! Only the Expression root has a producer today. The rest of
//! `spatial-tick-planning` attaches here rather than beside here: the
//! Source-resident Bang (02), the Self-Banging Function (03), the Jump chain
//! head (04), and Halt (05) each add a `Producer` variant, an `emit` arm, and
//! where they need one an `Effect` variant with its `resolve` arm. None of them
//! adds a second ordering pass.

use lang::{Atom, Atoms, Interpretation, Interpreter};
use std::collections::BTreeMap;

use crate::grid::{CellIndex, Grid, Position};

use super::language_map::{ExpressionEntry, LanguageMap, Span};
use super::{CellWrite, Diagnostic, PlayCommand, TickPlan};

///
/// One producer's turn in a Tick, taken at its anchor Position.
///
pub(super) struct Turn<'map> {
    anchor: Position,
    producer: Producer<'map>,
}

///
/// The kinds of producer that take a turn from one Source Snapshot.
///
/// An ordinary Expression uses its root anchor. The producers ADR 0020 also
/// names — the Source-resident Bang, the Self-Banging Function, the Jump chain
/// head, and Halt — arrive with issues 02 to 05 as further variants here.
///
enum Producer<'map> {
    ExpressionRoot(&'map ExpressionEntry),
}

///
/// One thing a producer contributes to the Tick Plan.
///
/// Every effect kind passes through the same ordering model, so a new kind is
/// a variant here and an arm in `resolve` rather than an ordering path of its
/// own. Issue 02's activation delivery and issue 05's root lock are the next
/// two variants; neither exists yet, because no producer emits them yet.
///
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Effect {
    ///
    /// One complete write. ADR 0004 and ADR 0009 validate a write's whole
    /// destination before any of its Cells is emitted; a `SpanWrite` exists
    /// only when its whole destination was accepted, so a partial write is
    /// unrepresentable rather than merely avoided.
    ///
    Write(SpanWrite),

    /// One Play Command from an active Terminal Output Function.
    Play(PlayCommand),

    /// One diagnostic about this producer's turn.
    Diagnose(Diagnostic),
}

///
/// Every turn this Source Snapshot grants, in ADR 0020's row-major producer
/// order.
///
pub(super) fn turns(grid: Grid, language_map: &LanguageMap) -> Vec<Turn<'_>> {
    // A root anchor is the turn's selector, not a lookup after one. An
    // `ExpressionEntry` carries a root only when it is executable and holds a
    // Function unit, so selecting on the root admits exactly the Expressions
    // that compute — the same set the pre-ADR-0020 loop reached by testing for
    // a Function first and then asserting a root had to exist.
    let mut turns: Vec<Turn<'_>> = language_map
        .expressions()
        .filter_map(|expression| {
            expression.root().map(|anchor| Turn {
                anchor,
                producer: Producer::ExpressionRoot(expression),
            })
        })
        .collect();

    order_by_anchor(grid, &mut turns);
    turns
}

///
/// Orders producers by row-major anchor Position: row first, then column.
///
/// The Language Map happens to yield Expression roots this way already —
/// Expression extents are collected in index order and never overlap, so an
/// earlier Expression's root anchor always precedes a later one's. That is a
/// property of one producer kind reading one partition, not of the ordering
/// model. Stating the order here is what keeps a producer kind reading a
/// different partition, such as the Language Unit partition issue 02 reads,
/// from silently taking its turn out of order.
///
/// The sort is stable, so producers sharing an anchor keep the order they were
/// collected in.
///
/// This guard is deliberately not pinnable end to end yet: with one producer
/// kind reading one already-ordered partition, deleting the `order_by_anchor`
/// call from `turns` leaves the whole suite green. Nothing can feed `turns` an
/// out-of-order producer until `spatial-tick-planning/02` adds a producer over
/// the Language Unit partition, which is the ticket that owes the end-to-end
/// test. `test_producers_arriving_out_of_source_order_still_take_row_major_turns`
/// pins the ordering itself in the meantime.
///
fn order_by_anchor(grid: Grid, turns: &mut [Turn<'_>]) {
    turns.sort_by_key(|turn| grid.index(turn.anchor()));
}

impl Turn<'_> {
    pub(super) fn anchor(&self) -> Position {
        self.anchor
    }

    ///
    /// Appends every effect this producer contributes, in the order it emits
    /// them.
    ///
    pub(super) fn emit(&self, grid: Grid, language_map: &LanguageMap, effects: &mut Vec<Effect>) {
        match self.producer {
            Producer::ExpressionRoot(expression) => {
                emit_expression_root(grid, language_map, self.anchor, expression, effects);
            }
        }
    }
}

///
/// Folds one Tick's effects, collected in producer order, into its Tick Plan.
///
/// Writes resolve Cell-wise: a later producer wins each Cell it overlaps and
/// leaves every other Cell of an earlier producer's write standing. Play
/// Commands and diagnostics keep producer-then-emission order. The resolved
/// writes are ordered by Cell so a Tick Plan describes its Source changes in
/// one predictable order, which is a separate question from which producer
/// owns each Cell.
///
///
/// A validated write of one encoding to a contiguous run of Cells.
///
/// The two ways a whole destination is refused are the two variants of
/// `SpanWriteError`; each producer turns them into its own diagnostic, and
/// neither yields a `SpanWrite`. Cells are addressed only when the Tick Plan
/// resolves, so no producer can emit half of one.
///
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SpanWrite {
    span: Span,
    content: String,
}

///
/// Why a whole destination was refused.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpanWriteError {
    /// There is no row below the producer's root.
    BelowSource,
    /// The encoding is wider than the destination row's remaining Cells.
    CrossesRowEdge,
}

impl SpanWrite {
    ///
    /// The write that places `content` in the row below `root`, starting at
    /// `root`'s column. `content` is one or more ASCII Cells.
    ///
    pub(super) fn below(grid: Grid, root: Position, content: &str) -> Result<Self, SpanWriteError> {
        let target = grid.below(root).ok_or(SpanWriteError::BelowSource)?;
        Self::at(grid, target, content)
    }

    ///
    /// The write that places `content` at `start` and along its row.
    /// `content` is one or more ASCII Cells.
    ///
    pub(super) fn at(grid: Grid, start: Position, content: &str) -> Result<Self, SpanWriteError> {
        debug_assert!(!content.is_empty(), "a write places at least one Cell");
        debug_assert!(
            content.is_ascii(),
            "a write preserves the Source ASCII invariant"
        );

        let width = content.chars().count();
        let last = grid
            .offset_in_row(start, width - 1)
            .ok_or(SpanWriteError::CrossesRowEdge)?;

        Ok(Self {
            span: Span::new(grid, grid.index(start), last),
            content: content.to_string(),
        })
    }

    ///
    /// Each Cell this write covers, paired with what it receives.
    ///
    fn cells(&self) -> impl Iterator<Item = (CellIndex, char)> + '_ {
        self.span.indices().zip(self.content.chars())
    }
}

pub(super) fn resolve(effects: Vec<Effect>) -> TickPlan {
    let mut writes: BTreeMap<CellIndex, char> = BTreeMap::new();
    let mut play_commands = Vec::new();
    let mut diagnostics = Vec::new();

    for effect in effects {
        match effect {
            Effect::Write(write) => {
                // A validated write fans out Cell-wise here, so ADR 0020's
                // per-Cell conflict resolution is unchanged: a later producer
                // still wins each Cell it overlaps, independently.
                for (cell, content) in write.cells() {
                    writes.insert(cell, content);
                }
            }
            Effect::Play(command) => play_commands.push(command),
            Effect::Diagnose(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    TickPlan {
        // Cell indices order row-major within one Grid, so draining the map in
        // key order is the Cell order a Tick Plan describes its changes in.
        writes: writes
            .into_iter()
            .map(|(cell, content)| CellWrite { cell, content })
            .collect(),
        play_commands,
        diagnostics,
    }
}

///
/// The effects of one Expression root's turn.
///
fn emit_expression_root(
    grid: Grid,
    language_map: &LanguageMap,
    root: Position,
    expression: &ExpressionEntry,
    effects: &mut Vec<Effect>,
) {
    let Some(atoms) = expression.atoms().filter(|atoms| is_computation(atoms)) else {
        return;
    };

    // A Terminal Output Function performs only when its root is active, so an
    // inactive terminal root is never evaluated at all: it contributes neither
    // a command nor a diagnostic, exactly as an absent Function would.
    // Value-producing roots still evaluate on every Tick; gating those is not
    // asked for by ADR 0020 and belongs to whichever issue states it.
    if is_terminal_root(atoms) && !language_map.is_root_active(root) {
        return;
    }

    let encoded = match Interpreter::execute(atoms) {
        Ok(Interpretation::Cell(Atom::Empty)) => return,
        Ok(Interpretation::Cell(result)) => result.to_string(),
        // Per ADR 0007 an empty Sequence emits no Cell writes, exactly as the
        // absence marker above emits none. A non-empty one is encoded and
        // routed through the same below-root, complete-fit, horizontal-write
        // path a single Atom takes: an intact Sequence is one ordinary result,
        // not a batch of Cell writes. Resolving a destination other than the
        // Cell below the root, and clearing a stale tail, belong to ADR 0009
        // and issue 04.
        //
        // No Source-parseable Function returns a Sequence yet, so no Source
        // text reaches this arm; issues 02 and 03 add the producers that
        // exercise it.
        Ok(Interpretation::Sequence(sequence)) if sequence.is_empty() => return,
        Ok(Interpretation::Sequence(sequence)) => sequence.to_string(),
        Ok(Interpretation::Play(command)) => {
            effects.push(Effect::Play(command));
            return;
        }
        Err(error) => {
            effects.push(Effect::Diagnose(Diagnostic::for_expression(
                root,
                expression.span(),
                error.to_string(),
            )));
            return;
        }
    };
    assert!(
        encoded.is_ascii(),
        "Interpreter results must preserve the Source ASCII invariant"
    );

    match SpanWrite::below(grid, root, &encoded) {
        Ok(write) => effects.push(Effect::Write(write)),
        Err(reason) => effects.push(Effect::Diagnose(Diagnostic::for_expression(
            root,
            expression.span(),
            match reason {
                SpanWriteError::BelowSource => {
                    format!("result {encoded:?} falls below the Source")
                }
                SpanWriteError::CrossesRowEdge => {
                    format!("result {encoded:?} crosses the row edge")
                }
            },
        ))),
    }
}

///
/// Whether an Expression computes anything.
///
/// An Expression with no Function is a literal — the Interpreter has no
/// Function to apply, so a Tick produces no result to commit for it. This is
/// what stops a committed result from feeding itself: the value a Tick writes
/// is valid Source, but on the next Tick it parses to a literal and so commits
/// nothing of its own.
///
fn is_computation(atoms: &Atoms) -> bool {
    atoms.iter().any(|a| matches!(a, Atom::Function(_)))
}

///
/// Whether this Expression's root is a Terminal Output Function.
///
/// The Interpreter accepts a terminal Function only as the first Atom, so that
/// is the one position where a terminal root can be. Asking the Function's own
/// classification rather than naming `!>` keeps this in step with the canonical
/// Function definitions as the family grows.
///
fn is_terminal_root(atoms: &Atoms) -> bool {
    matches!(atoms.first(), Some(Atom::Function(function)) if function.is_terminal())
}

#[cfg(test)]
mod test {
    use super::{Effect, SpanWrite, SpanWriteError, Turn, order_by_anchor, resolve, turns};
    use crate::{
        grid::{CellIndex, Grid},
        source::{CellWrite, Diagnostic, PlayCommand, language_map::LanguageMap},
    };

    ///
    /// The index `grid` mints for `idx`, so a test states an expected planned
    /// write in the same terms a Tick Plan carries.
    ///
    fn cell(grid: Grid, idx: usize) -> CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    ///
    /// A Language Map derived from `rows`, each padded to the Grid's width.
    /// Tests state the Source they mean as Cells, exactly as it is seen.
    ///
    fn language_map(grid: Grid, rows: &[&str]) -> LanguageMap {
        let mut source = String::with_capacity(grid.count());
        for row in 0..grid.rows().count() {
            let cells = rows.get(row).copied().unwrap_or("");
            assert!(
                cells.len() <= grid.cols(),
                "row {row} is wider than the Grid"
            );
            source.push_str(cells);
            source.push_str(&" ".repeat(grid.cols() - cells.len()));
        }

        LanguageMap::derive(grid, &source).expect("Source Cells are printable ASCII")
    }

    fn anchors(turns: &[Turn<'_>]) -> Vec<(usize, usize)> {
        turns
            .iter()
            .map(|turn| (turn.anchor().x(), turn.anchor().y()))
            .collect()
    }

    fn emitted(grid: Grid, language_map: &LanguageMap) -> Vec<Effect> {
        let mut effects = Vec::new();
        for turn in turns(grid, language_map) {
            turn.emit(grid, language_map, &mut effects);
        }
        effects
    }

    ///
    /// A complete write of `content` starting at `idx`. One Effect covers the
    /// whole run, which is the shape a producer emits.
    ///
    fn write(grid: Grid, idx: usize, content: &str) -> Effect {
        let start = grid.position_at(grid.cell_index(idx).expect("inside the Grid"));
        Effect::Write(SpanWrite::at(grid, start, content).expect("the write fits its row"))
    }

    fn diagnostic(grid: Grid, start: usize, end: usize, message: &str) -> Effect {
        let cell = |idx: usize| grid.cell_index(idx).expect("inside the Grid");
        Effect::Diagnose(Diagnostic::for_range(
            grid,
            cell(start),
            cell(end),
            message.to_string(),
        ))
    }

    #[test]
    fn test_turns_are_ordered_by_row_major_anchor_position() {
        // Row-major is row first, then column: the row 1 root takes its turn
        // after both row 0 roots even though it sits further left than one of
        // them.
        let grid = Grid::new(20, 3);
        let map = language_map(grid, &[".+0102 .+0304", "  .-0504"]);

        assert_eq!(anchors(&turns(grid, &map)), vec![(0, 0), (7, 0), (2, 1)]);
    }

    #[test]
    fn test_producers_arriving_out_of_source_order_still_take_row_major_turns() {
        // The ordering must be a property of this model rather than something
        // inherited from the order the Language Map happens to hand producers
        // over in. A producer kind reading a different partition can arrive in
        // any order; its turn still lands where its anchor says.
        let grid = Grid::new(20, 3);
        let map = language_map(grid, &[".+0102 .+0304", "  .-0504"]);

        let mut arrivals = turns(grid, &map);
        arrivals.reverse();
        order_by_anchor(grid, &mut arrivals);

        assert_eq!(anchors(&arrivals), vec![(0, 0), (7, 0), (2, 1)]);
    }

    #[test]
    fn test_each_expression_root_takes_exactly_one_turn() {
        // ADR 0006: multiple Bangs never give one root a second turn, and a
        // root whose turn has passed is never revisited. Both Bangs are
        // aligned with the root, and the pass still grants exactly one turn.
        // The Bangs themselves take no turn until issue 02 gives them one.
        let grid = Grid::new(10, 3);
        let map = language_map(grid, &["**", "!>007FC4", "**"]);

        assert_eq!(anchors(&turns(grid, &map)), vec![(0, 1)]);
    }

    #[test]
    fn test_a_producer_emits_its_effects_in_a_stable_local_order() {
        // One producer, two Cells of one result: the encoding is emitted left
        // to right, and emitting the same turn again produces the same
        // sequence.
        let grid = Grid::new(10, 3);
        let map = language_map(grid, &[".+0102"]);
        let turns = turns(grid, &map);
        let turn = turns.first().expect("the root takes a turn");

        let mut first = Vec::new();
        turn.emit(grid, &map, &mut first);
        let mut second = Vec::new();
        turn.emit(grid, &map, &mut second);

        assert_eq!(first, vec![write(grid, 10, "03")]);
        assert_eq!(first, second);
    }

    #[test]
    fn test_a_write_whose_destination_leaves_the_grid_emits_no_partial_write() {
        // ADR 0004: a complete write validates its whole destination before
        // any Cell of it enters the Tick Plan. This root's result has nowhere
        // below it to go, so the turn contributes a diagnostic and nothing
        // else — not the first Cell of a result that could not be placed.
        let grid = Grid::new(10, 2);
        let map = language_map(grid, &["", ".+0102"]);

        assert_eq!(
            emitted(grid, &map),
            vec![diagnostic(
                grid,
                10,
                15,
                "result \"03\" falls below the Source"
            )]
        );
    }

    #[test]
    fn test_a_refused_destination_yields_no_write_at_all() {
        // The property ADR 0004 states — validate the whole destination before
        // emitting any Cell of it — is now a property of the constructor, so it
        // is checked here without a Source, a producer, or a Tick.
        let grid = Grid::new(4, 2);
        let bottom_row = grid.position(0, 1).expect("inside the Grid");
        let near_edge = grid.position(2, 0).expect("inside the Grid");

        assert_eq!(
            SpanWrite::below(grid, bottom_row, "03"),
            Err(SpanWriteError::BelowSource),
            "there is no row below the last one"
        );
        assert_eq!(
            SpanWrite::at(grid, near_edge, "ABC"),
            Err(SpanWriteError::CrossesRowEdge),
            "three Cells do not fit in the two remaining"
        );

        // the widest write the row does hold is accepted whole
        assert!(SpanWrite::at(grid, near_edge, "AB").is_ok());
    }

    #[test]
    fn test_later_effects_win_cell_conflicts_independently() {
        // An earlier producer writes three Cells and a later one writes two,
        // overlapping the third. Resolution is Cell-wise: the later producer
        // takes only the Cell they share, and every other Cell of the earlier
        // producer's complete write still stands.
        //
        // No Source can express this yet, because every result today is one
        // two-Cell Atom written below a root, and two roots in one row sit at
        // least three columns apart. The Source-writing Functions of issues 03
        // and 04 emit exactly this shape of overlapping bundle, so the
        // resolution they rely on is pinned here at the seam that owns it.
        let grid = Grid::new(20, 3);
        let plan = resolve(vec![write(grid, 10, "ABC"), write(grid, 12, "XY")]);

        assert_eq!(
            plan.writes,
            vec![
                CellWrite {
                    cell: cell(grid, 10),
                    content: 'A'
                },
                CellWrite {
                    cell: cell(grid, 11),
                    content: 'B'
                },
                CellWrite {
                    cell: cell(grid, 12),
                    content: 'X'
                },
                CellWrite {
                    cell: cell(grid, 13),
                    content: 'Y'
                },
            ]
        );
    }

    #[test]
    fn test_play_commands_and_diagnostics_keep_producer_and_emission_order() {
        // Play Commands and diagnostics are ordered, never merged: unlike a
        // Cell, which one producer can take from another, each command and
        // each diagnostic keeps the place its producer's turn gave it.
        let grid = Grid::new(10, 3);
        let first = PlayCommand::Raw {
            channel: 0,
            velocity: 1,
            note: 60,
        };
        let second = PlayCommand::Raw {
            channel: 1,
            velocity: 2,
            note: 61,
        };
        let earlier = diagnostic(grid, 0, 5, "earlier producer");
        let later = diagnostic(grid, 20, 25, "later producer");

        let plan = resolve(vec![
            Effect::Play(first),
            earlier.clone(),
            write(grid, 10, "0"),
            Effect::Play(second),
            later.clone(),
        ]);

        assert_eq!(plan.play_commands, vec![first, second]);
        assert_eq!(
            plan.diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec!["earlier producer", "later producer"]
        );
        assert_eq!(
            plan.writes,
            vec![CellWrite {
                cell: cell(grid, 10),
                content: '0'
            }]
        );
    }
}
