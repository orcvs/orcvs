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

use crate::grid::{Grid, Position};

use super::language_map::{ExpressionEntry, LanguageMap};
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
    /// One Cell of a complete write. ADR 0004 and ADR 0009 validate a write's
    /// whole destination before any of its Cells is emitted, so an invalid or
    /// out-of-Grid destination diagnoses and contributes no partial write.
    ///
    Write(CellWrite),

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
pub(super) fn resolve(effects: Vec<Effect>) -> TickPlan {
    let mut writes = BTreeMap::new();
    let mut play_commands = Vec::new();
    let mut diagnostics = Vec::new();

    for effect in effects {
        match effect {
            Effect::Write(write) => {
                writes.insert(write.idx, write);
            }
            Effect::Play(command) => play_commands.push(command),
            Effect::Diagnose(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    TickPlan {
        writes: writes.into_values().collect(),
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
                grid,
                root,
                expression.footprint(),
                error.to_string(),
            )));
            return;
        }
    };
    assert!(
        encoded.is_ascii(),
        "Interpreter results must preserve the Source ASCII invariant"
    );

    // The whole destination validates before any Cell of this write is
    // emitted, so a rejected destination contributes no partial write.
    let Some(target) = grid.below(root) else {
        effects.push(Effect::Diagnose(Diagnostic::for_expression(
            grid,
            root,
            expression.footprint(),
            format!("result {encoded:?} falls below the Source"),
        )));
        return;
    };
    if !grid.fits(target, encoded.chars().count()) {
        effects.push(Effect::Diagnose(Diagnostic::for_expression(
            grid,
            root,
            expression.footprint(),
            format!("result {encoded:?} crosses the row edge"),
        )));
        return;
    }

    let target_idx = grid.index(target);
    for (offset, content) in encoded.chars().enumerate() {
        effects.push(Effect::Write(CellWrite {
            idx: target_idx + offset,
            content,
        }));
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
    use super::{Effect, Turn, order_by_anchor, resolve, turns};
    use crate::{
        grid::Grid,
        source::{CellWrite, Diagnostic, PlayCommand, language_map::LanguageMap},
    };

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

    fn write(idx: usize, content: char) -> Effect {
        Effect::Write(CellWrite { idx, content })
    }

    fn diagnostic(grid: Grid, start: usize, end: usize, message: &str) -> Effect {
        Effect::Diagnose(Diagnostic::for_range(grid, start, end, message.to_string()))
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

        assert_eq!(first, vec![write(10, '0'), write(11, '3')]);
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
        let plan = resolve(vec![
            write(10, 'A'),
            write(11, 'B'),
            write(12, 'C'),
            write(12, 'X'),
            write(13, 'Y'),
        ]);

        assert_eq!(
            plan.writes,
            vec![
                CellWrite {
                    idx: 10,
                    content: 'A'
                },
                CellWrite {
                    idx: 11,
                    content: 'B'
                },
                CellWrite {
                    idx: 12,
                    content: 'X'
                },
                CellWrite {
                    idx: 13,
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
            write(10, '0'),
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
                idx: 10,
                content: '0'
            }]
        );
    }
}
