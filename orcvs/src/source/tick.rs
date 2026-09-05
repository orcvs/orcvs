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

use lang::{
    Anchor, Atom, Atoms, Error as LangError, Interpretation, Interpreter, Tick, TickInputs,
};
use std::collections::BTreeMap;

use crate::grid::{CellIndex, Grid, Position};

use super::language_map::{ExpressionEntry, LanguageMap, Span};
use super::portal::{Portal, PortalError, SpanWrite};
use super::{CellContent, CellWrite, Diagnostic, PlayCommand, TickPlan};

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
    /// destination before any of its Cells is emitted; a [`SpanWrite`] exists
    /// only because a [`Portal`] accepted its whole destination, so a partial
    /// write is unrepresentable rather than merely avoided.
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
    pub(super) fn emit(
        &self,
        grid: Grid,
        language_map: &LanguageMap,
        tick: Tick,
        effects: &mut Vec<Effect>,
    ) {
        match self.producer {
            Producer::ExpressionRoot(expression) => {
                emit_expression_root(grid, language_map, self.anchor, tick, expression, effects);
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
    let mut writes: BTreeMap<CellIndex, CellContent> = BTreeMap::new();
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
/// ADR 0012's explicit inputs for one root's evaluation.
///
/// The Tick comes from the Playback Engine, which owns musical time; the
/// anchor is this root's own Grid-minted Position, converted to the plain pair
/// of coordinates the language crate carries. Only the two numbers cross the
/// boundary: a Position can be obtained solely from the Grid that contains it,
/// and that invariant belongs to this crate.
///
fn tick_inputs(tick: Tick, root: Position) -> TickInputs {
    TickInputs::new(tick, Anchor::new(root.x(), root.y()))
}

///
/// Evaluates one root's Atoms against the explicit inputs it was given.
///
/// The one place this crate calls the Interpreter, so it is the one place the
/// ADR 0012 inputs cross into evaluation. Nothing observable comes back out of
/// a Tick to say which Tick and which anchor a root was told — a result is the
/// same two Cells however it was seeded, and stays that way until
/// `tick-functions/02` gives Clock a Tick to read and `tick-functions/04`
/// gives Random an anchor. Until then the thread from the Playback Engine to
/// `Interpreter::execute` is only as good as something watching it, so under
/// `cfg(test)` this records what it was handed before delegating. A production
/// build compiles the recording out entirely: what remains is one inlineable
/// call that forwards its two arguments unchanged.
///
fn interpret(atoms: &Atoms, inputs: TickInputs) -> Result<Interpretation, LangError> {
    #[cfg(test)]
    observed::record(inputs);

    Interpreter::execute(atoms, inputs)
}

///
/// What the Interpreter was actually handed during this test.
///
/// A test-only seam, per thread and so per test: `cargo nextest` gives each
/// test its own process and `cargo test` its own thread, so no two tests can
/// see each other's Ticks. `take` both reads and clears, which is what lets a
/// test state the inputs of exactly the Playback run it drove rather than of
/// everything its thread has ever interpreted.
///
#[cfg(test)]
mod observed {
    use lang::TickInputs;
    use std::cell::RefCell;

    thread_local! {
        static INTERPRETED: RefCell<Vec<TickInputs>> = const { RefCell::new(Vec::new()) };
    }

    /// Records one evaluation's explicit inputs, in the order it was evaluated.
    pub(super) fn record(inputs: TickInputs) {
        INTERPRETED.with_borrow_mut(|interpreted| interpreted.push(inputs));
    }

    /// Every recorded input since the last `take`, clearing the record.
    pub(super) fn take() -> Vec<TickInputs> {
        INTERPRETED.with_borrow_mut(std::mem::take)
    }
}

///
/// The effects of one Expression root's turn.
///
fn emit_expression_root(
    grid: Grid,
    language_map: &LanguageMap,
    root: Position,
    tick: Tick,
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

    effects.extend(result_effect(
        grid,
        root,
        expression.span(),
        interpret(atoms, tick_inputs(tick, root)),
    ));
}

///
/// The optional Effect of one evaluation answer, delivered from `root`.
///
/// Delivery owns encoding, admission, and diagnostics. Its caller owns when
/// this Effect is emitted, resolved against other writes, and committed.
///
/// Split from the turn that drove the evaluation because the two halves are
/// answerable separately: which roots evaluate, and with what, is ADR 0020's
/// question, while what an answer becomes is ADR 0007's and ADR 0009's. A
/// producer kind that computes differently — the Source-writing Functions of
/// ADR 0004 among them — still delivers an ordinary result the same way, and a
/// test can state one answer without a Source that spells it. That second use
/// is not incidental: no Source-parseable Function returns a Sequence yet, so
/// naming this step is the only way the Sequence half of the result path is
/// reachable at all before issues 02 and 03 add the Functions that spell one.
///
pub(super) fn result_effect(
    grid: Grid,
    root: Position,
    span: Span,
    result: Result<Interpretation, LangError>,
) -> Option<Effect> {
    let encoded = match result {
        Ok(Interpretation::Cell(Atom::Empty)) => return None,
        Ok(Interpretation::Cell(result)) => result.to_string(),
        // Per ADR 0007 an empty Sequence emits no Cell writes, exactly as the
        // absence marker above emits none. A non-empty one is encoded and
        // routed through the same Portal a single Atom passes through: an
        // intact Sequence is one ordinary result, not a batch of Cell writes,
        // and the complete-fit rule ADR 0007 states for it is the rule the
        // Portal already applies to any encoding.
        Ok(Interpretation::Sequence(sequence)) if sequence.is_empty() => return None,
        Ok(Interpretation::Sequence(sequence)) => sequence.to_string(),
        Ok(Interpretation::Play(command)) => return Some(Effect::Play(command)),
        Err(error) => {
            return Some(Effect::Diagnose(Diagnostic::for_expression(
                root,
                span,
                error.to_string(),
            )));
        }
    };
    // Resolution and fit are one expression because a producer answers both
    // refusals the same way: no write at all, and a diagnostic saying which it
    // was. The encoding is fanned out into Cells only when the Tick Plan
    // resolves, so nothing between here and there holds part of a result.
    Some(
        match Portal::ordinary_result(grid, root).and_then(|portal| portal.admit(&encoded)) {
            Ok(write) => Effect::Write(write),
            Err(reason) => Effect::Diagnose(Diagnostic::for_expression(
                root,
                span,
                match reason {
                    PortalError::BelowSource => {
                        format!("result {encoded:?} falls below the Source")
                    }
                    PortalError::CrossesRowEdge => {
                        format!("result {encoded:?} crosses the row edge")
                    }
                },
            )),
        },
    )
}

///
/// Whether an Expression computes anything.
///
/// An Expression with no Function is a literal — the Interpreter has no
/// Function to apply, so a Tick produces no result to commit for it. Generated
/// Cells follow the same rule as typed Source: Number-only results do not
/// compute, while a result encoding a Function can compute on the next Tick.
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
    use super::{
        Effect, Portal, Tick, Turn, observed, order_by_anchor, resolve, result_effect, tick_inputs,
        turns,
    };
    use lang::{Atom, Interpretation, Sequence};

    use crate::{
        grid::{CellIndex, Grid},
        source::{
            CellWrite, Diagnostic, MidiChannel, Note, PlayCommand, TickPlan, Velocity,
            language_map::{LanguageMap, Span},
        },
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
            turn.emit(grid, language_map, Tick::ZERO, &mut effects);
        }
        effects
    }

    ///
    /// A complete write of `content` starting at `idx`. One Effect covers the
    /// whole run, which is the shape a producer emits, and it is admitted by a
    /// Portal because that is the only way one comes into being.
    ///
    fn write(grid: Grid, idx: usize, content: &str) -> Effect {
        let destination = grid.position_at(grid.cell_index(idx).expect("inside the Grid"));
        Effect::Write(
            Portal::at(grid, destination)
                .admit(content)
                .expect("the encoding fits its row"),
        )
    }

    ///
    /// One Sequence of Numbers, as evaluation would answer with it. Numbers
    /// are used throughout because their encoding is two Cells wide and
    /// self-evident in an expected row, so a test states what it means about
    /// destinations rather than about Atom spellings.
    ///
    fn sequence(numbers: &[u8]) -> Interpretation {
        Interpretation::Sequence(
            Sequence::new(numbers.iter().copied().map(Atom::Number))
                .expect("a Number is a Sequence member"),
        )
    }

    ///
    /// The Cells `plan` writes, as plain numbers and characters, so an
    /// expected Source row reads as one.
    ///
    fn planned(plan: &TickPlan) -> Vec<(usize, char)> {
        plan.writes
            .iter()
            .map(|write| (write.cell.get(), write.content.as_char()))
            .collect()
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
    fn test_each_root_is_told_the_shared_tick_and_its_own_anchor_position() {
        // ADR 0012 gives one Tick to the whole Source Snapshot, and ADR 0013
        // seeds Random from the Function's own column and row. So the two
        // explicit inputs a root is evaluated with differ in exactly one way:
        // every root of a Tick is told the same Tick, and each is told its own
        // anchor.
        //
        // The expected anchors are written out rather than read back from the
        // same turns, so the assertion cannot be satisfied by the code it is
        // testing. They are asymmetric for the same reason: a transposed
        // column and row would otherwise pass. `.-0504` sits at column 2 of
        // row 1, and is the one root whose column and row differ.
        //
        // This is about the conversion alone — the Grid-minted Position of a
        // turn becoming the plain column and row that cross the crate
        // boundary. That the Interpreter is then handed these same inputs is a
        // separate claim, pinned by
        // `test_the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor`.
        let grid = Grid::new(20, 3);
        let map = language_map(grid, &[".+0102 .+0304", "  .-0504"]);
        let tick = Tick::new(11);

        let inputs: Vec<_> = turns(grid, &map)
            .iter()
            .map(|turn| tick_inputs(tick, turn.anchor()))
            .collect();

        assert!(inputs.iter().all(|inputs| inputs.tick() == tick));
        assert_eq!(
            inputs
                .iter()
                .map(|inputs| (inputs.anchor().column(), inputs.anchor().row()))
                .collect::<Vec<_>>(),
            vec![(0, 0), (7, 0), (2, 1)],
        );
    }

    #[test]
    fn test_the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor() {
        // The claim the conversion test above cannot make: that the inputs
        // `emit_expression_root` builds are the inputs evaluation actually
        // receives. This drives a real Tick of a Source Snapshot through
        // `turns` and `Turn::emit` and reads back what reached the Interpreter,
        // so severing the thread — passing a fixed Tick and anchor at the call
        // site instead of this root's own — fails here rather than passing
        // unnoticed until ADR 0013's Random seeds every Position identically.
        //
        // The expected anchors are literals rather than anything read back from
        // the turns, so the assertion cannot be satisfied by the code under
        // test. They are asymmetric so that a transposed column and row is
        // visible: `.-0504` sits at column 2 of row 1. The Tick is not
        // `Tick::ZERO`, so a hardcoded first Tick is visible too.
        let grid = Grid::new(20, 3);
        let map = language_map(grid, &[".+0102 .+0304", "  .-0504"]);
        let tick = Tick::new(11);

        // whatever an earlier Playback run on this thread interpreted is not
        // part of this Tick
        let _ = observed::take();

        let mut effects = Vec::new();
        for turn in turns(grid, &map) {
            turn.emit(grid, &map, tick, &mut effects);
        }
        let interpreted = observed::take();

        assert_eq!(interpreted.len(), 3, "each of the three roots is evaluated");
        assert!(
            interpreted.iter().all(|inputs| inputs.tick() == tick),
            "one Tick is given to the whole Source Snapshot"
        );
        assert_eq!(
            interpreted
                .iter()
                .map(|inputs| (inputs.anchor().column(), inputs.anchor().row()))
                .collect::<Vec<_>>(),
            vec![(0, 0), (7, 0), (2, 1)],
            "each root is told its own anchor, in row-major turn order"
        );
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
        turn.emit(grid, &map, Tick::ZERO, &mut first);
        let mut second = Vec::new();
        turn.emit(grid, &map, Tick::ZERO, &mut second);

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
                    content: crate::source::CellContent::new(b'A').unwrap()
                },
                CellWrite {
                    cell: cell(grid, 11),
                    content: crate::source::CellContent::new(b'B').unwrap()
                },
                CellWrite {
                    cell: cell(grid, 12),
                    content: crate::source::CellContent::new(b'X').unwrap()
                },
                CellWrite {
                    cell: cell(grid, 13),
                    content: crate::source::CellContent::new(b'Y').unwrap()
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
            channel: MidiChannel::try_from(0).unwrap(),
            velocity: Velocity::try_from(1).unwrap(),
            note: Note::try_from(60).unwrap(),
        };
        let second = PlayCommand::Raw {
            channel: MidiChannel::try_from(1).unwrap(),
            velocity: Velocity::try_from(2).unwrap(),
            note: Note::try_from(61).unwrap(),
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
                content: crate::source::CellContent::new(b'0').unwrap()
            }]
        );
    }

    #[test]
    fn a_sequence_result_is_delivered_through_one_portal_below_its_root() {
        // ADR 0007: a non-empty Sequence encodes horizontally from the
        // ordinary result Position through one Portal carrying the intact
        // Sequence. Three Atoms are one Effect, not three, and become six
        // Cells only when the Tick Plan resolves — which is what makes the
        // whole Sequence validated before any Cell of it exists.
        //
        // No Source text reaches this arm yet, because no Source-parseable
        // Function returns a Sequence until issues 02 and 03 add Range,
        // Reverse, and Concatenate. The result is stated directly instead, at
        // the seam a Function's answer arrives through.
        let grid = Grid::new(10, 3);
        let root = grid.position(0, 0).expect("inside the Grid");
        let span = Span::new(grid, cell(grid, 0), cell(grid, 5));

        let mut effects = Vec::new();
        effects.extend(result_effect(
            grid,
            root,
            span,
            Ok(sequence(&[0x0A, 0x0B, 0x0C])),
        ));

        assert_eq!(effects, vec![write(grid, 10, "0A0B0C")]);
        assert_eq!(
            planned(&resolve(effects)),
            vec![
                (10, '0'),
                (11, 'A'),
                (12, '0'),
                (13, 'B'),
                (14, '0'),
                (15, 'C'),
            ]
        );
    }

    #[test]
    fn a_sequence_wider_than_its_destination_row_plans_no_partial_write() {
        // ADR 0007: if the complete encoding cannot fit, interpretation reports
        // a diagnostic and plans no partial write. Five Atoms need ten Cells
        // and the destination row has eight left, so the whole Sequence is
        // refused — the four Atoms that would have fitted are not admitted on
        // their own, which is the failure this pins.
        let grid = Grid::new(10, 3);
        let root = grid.position(2, 0).expect("inside the Grid");
        let span = Span::new(grid, cell(grid, 2), cell(grid, 7));

        let mut effects = Vec::new();
        effects.extend(result_effect(
            grid,
            root,
            span,
            Ok(sequence(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0E])),
        ));

        assert_eq!(
            effects,
            vec![diagnostic(
                grid,
                2,
                7,
                "result \"0A0B0C0D0E\" crosses the row edge"
            )]
        );
    }

    #[test]
    fn a_sequence_result_with_no_row_below_it_plans_no_write() {
        // The other way ADR 0009 refuses a whole destination: out of Grid
        // rather than non-fitting. A root in the last row resolves no Portal
        // at all, and a Sequence answers for that the same way a single Atom
        // does — one diagnostic naming the encoding, and nothing planned.
        let grid = Grid::new(10, 2);
        let root = grid.position(0, 1).expect("inside the Grid");
        let span = Span::new(grid, cell(grid, 10), cell(grid, 15));

        let mut effects = Vec::new();
        effects.extend(result_effect(grid, root, span, Ok(sequence(&[0x0A, 0x0B]))));

        assert_eq!(
            effects,
            vec![diagnostic(
                grid,
                10,
                15,
                "result \"0A0B\" falls below the Source"
            )]
        );
    }

    #[test]
    fn the_cells_of_two_overlapping_sequence_results_are_contested_one_by_one() {
        // ADR 0009: every admitted write participates Cell-wise in ADR 0020's
        // producer order. A Sequence is one validated write while it is being
        // planned and as many independently contested Cells as it has
        // characters once it is resolved, so the later root takes only the four
        // Cells the two encodings share and the earlier root's first two Cells
        // still stand.
        //
        // Two roots two columns apart is a shape no Source can express today —
        // a two-Cell Atom result never overlaps a neighbour's — and exactly the
        // shape Sequence results make ordinary.
        let grid = Grid::new(20, 3);
        let earlier = grid.position(0, 0).expect("inside the Grid");
        let later = grid.position(2, 0).expect("inside the Grid");

        let mut effects = Vec::new();
        effects.extend(result_effect(
            grid,
            earlier,
            Span::new(grid, cell(grid, 0), cell(grid, 5)),
            Ok(sequence(&[0x0A, 0x0B, 0x0C])),
        ));
        effects.extend(result_effect(
            grid,
            later,
            Span::new(grid, cell(grid, 2), cell(grid, 7)),
            Ok(sequence(&[0x0D, 0x0E, 0x0F])),
        ));

        assert_eq!(
            planned(&resolve(effects)),
            vec![
                (20, '0'),
                (21, 'A'),
                (22, '0'),
                (23, 'D'),
                (24, '0'),
                (25, 'E'),
                (26, '0'),
                (27, 'F'),
            ]
        );
    }
}

///
/// The Cell-wise half of ADR 0020, over overlap shapes no example states.
///
/// The `cfg` matches the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
/// table that declares proptest, so a WASM build never sees the dependency.
///
#[cfg(all(test, not(target_arch = "wasm32")))]
mod property {
    use super::{Effect, Portal, resolve};
    use crate::grid::Grid;
    use proptest::prelude::*;

    /// How wide the Grid every generated write lands in is. Stated once, so
    /// the column strategy, the Grid, and the row the oracle scans cannot
    /// drift apart into three literals that must be changed together.
    const WIDTH: usize = 16;

    proptest! {
        ///
        /// "Later Cell effects win conflicts at each Cell independently", and
        /// its unstated other half: a Tick Plan contains a Cell exactly when
        /// some admitted write covers it. Together those are ADR 0009's
        /// promise that an ordinary result "writes only its current encoding
        /// and never clears a stale tail outside that Span" — no shorter later
        /// write can reach a Cell it does not cover, in either direction.
        ///
        /// A property rather than an example because the interesting input is
        /// the shape of the overlaps: partial at either end, one write wholly
        /// inside another, two writes on the very same Cells, and runs that
        /// stop short of a row's end. Examples state one shape each, and the
        /// third one written by hand is already an enumeration.
        ///
        /// The expectation is computed by scanning the writes in reverse for
        /// the last one covering each Cell, which is a different computation
        /// from the forward fold under test rather than a copy of it. Writes
        /// the Portal refuses are dropped rather than made to fit, so what is
        /// resolved is only ever a set of complete writes.
        ///
        #[test]
        fn a_tick_plan_gives_each_cell_to_the_last_admitted_write_covering_it(
            requested in prop::collection::vec((0usize..WIDTH, "[A-Z]{1,8}"), 0..5),
        ) {
            let grid = Grid::new(WIDTH, 2);
            // The first Cell of the destination row, asked of the same Grid the
            // writes are admitted through rather than recomputed from its width.
            let destination_row = grid
                .index(grid.position(0, 1).expect("inside the Grid"))
                .get();
            let mut admitted: Vec<(usize, String)> = Vec::new();
            let mut effects: Vec<Effect> = Vec::new();
            for (column, encoding) in requested {
                let destination = grid.position(column, 1).expect("inside the Grid");
                if let Ok(write) = Portal::at(grid, destination).admit(&encoding) {
                    effects.push(Effect::Write(write));
                    admitted.push((column, encoding));
                }
            }

            let plan = resolve(effects);

            let owner = |idx: usize| {
                admitted.iter().rev().find_map(|(column, encoding)| {
                    encoding
                        .chars()
                        .nth(idx.checked_sub(destination_row + column)?)
                })
            };
            prop_assert_eq!(
                plan.writes
                    .iter()
                    .map(|write| (write.cell.get(), write.content.as_char()))
                    .collect::<Vec<_>>(),
                (destination_row..grid.count())
                    .filter_map(|idx| owner(idx).map(|content| (idx, content)))
                    .collect::<Vec<_>>()
            );
        }
    }
}
