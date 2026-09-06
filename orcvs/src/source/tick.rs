//! Dependency-scheduled Tick evaluation (ADR 0032).
//!
//! Starting Source fixes the candidate roots and their operand layout. Data and
//! Bang-activation edges determine execution order; each root binds its operands
//! from working Source when its dependencies have settled. Source and terminal
//! effects are published atomically after the schedule completes.

use lang::{
    Anchor, Atom, Atoms, Error as LangError, Function, Interpretation, Interpreter, Tick,
    TickInputs,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::grid::{CellIndex, Grid, Position};

#[cfg(test)]
use super::language_map::Span;
use super::language_map::{ExpressionEntry, LanguageMap};
use super::portal::{Portal, PortalError, SpanWrite};
use super::{CellContent, CellWrite, Diagnostic, Performance, TickPlan};

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

    /// The ordered group of Play Commands from one active Terminal Output
    /// Function root.
    ///
    /// One Effect per Expression rather than one per command, because ADR 0020
    /// orders effects by their producer's Position and every command a widened
    /// Expression performs shares one. ADR 0030 settles what that leaves
    /// undecided: within one Expression, commands order by element index, so
    /// element order is an order inside a single Effect and producer-then-
    /// emission order between Expressions is untouched.
    Play(Performance),

    /// One diagnostic about this producer's scheduled evaluation.
    Diagnose(Diagnostic),
}

#[derive(Clone, Copy)]
struct ScheduledRoot<'a> {
    anchor: Position,
    function: Function,
    expression: &'a ExpressionEntry,
    output: Result<Option<Position>, PortalError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DependencyKind {
    Data,
    Activation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Dependency {
    producer: usize,
    consumer: usize,
    kind: DependencyKind,
}

struct Schedule<'a> {
    roots: Vec<ScheduledRoot<'a>>,
    dependencies: Vec<Dependency>,
    order: Vec<usize>,
}

/// Executes the graph derived from one starting Source revision.
pub(super) fn plan(grid: Grid, bytes: &[u8], map: &LanguageMap, tick: Tick) -> TickPlan {
    plan_with_destinations(grid, bytes, map, tick, &BTreeMap::new())
}

fn plan_with_destinations(
    grid: Grid,
    bytes: &[u8],
    map: &LanguageMap,
    tick: Tick,
    destinations: &BTreeMap<CellIndex, Position>,
) -> TickPlan {
    let schedule = match schedule(grid, map, destinations) {
        Ok(schedule) => schedule,
        Err(diagnostics) => {
            return TickPlan {
                writes: Vec::new(),
                play_commands: Vec::new(),
                diagnostics,
            };
        }
    };

    let mut working = bytes.to_vec();
    let mut effects = Vec::new();

    // A Source-resident `**` is display left by an earlier result (or text a
    // person entered). It is never an event for this Tick. Clear only Bangs
    // whose validity the starting parse established; an invalid operand
    // spelling remains untouched and diagnosed by the Language Map.
    for (anchor, _) in map.bangs() {
        let clear = Portal::at(grid, anchor)
            .admit("  ")
            .expect("a parsed Bang's complete encoding fits its Grid");
        apply_write(&mut working, &clear);
        effects.push(Effect::Write(clear));
    }

    let mut succeeded = vec![false; schedule.roots.len()];
    let mut bang_events = Vec::new();
    for node_index in schedule.order {
        let root = schedule.roots[node_index];
        let failed_input = schedule
            .dependencies
            .iter()
            .any(|dependency| dependency.consumer == node_index && !succeeded[dependency.producer]);
        if failed_input {
            effects.push(Effect::Diagnose(Diagnostic::for_expression(
                root.anchor,
                root.expression.span(),
                "a current-Tick dependency failed".to_owned(),
            )));
            continue;
        }

        if root.function.is_terminal()
            && !bang_events
                .iter()
                .copied()
                .any(|bang| activates(grid, bang, root.anchor))
        {
            succeeded[node_index] = true;
            continue;
        }

        let start = grid.index(root.anchor).get();
        let row_end = grid
            .position(0, root.anchor.y() + 1)
            .map(|next_row| grid.index(next_row).get())
            .unwrap_or_else(|| grid.count());
        let atoms = match root.expression.bind_source(
            std::str::from_utf8(&working[start..row_end])
                .expect("Source Cells are printable ASCII"),
        ) {
            Ok(atoms) => atoms,
            Err(error) => {
                if root.expression.atoms().is_none() {
                    // Live-edit fragments and invalid typed operands already
                    // have Source diagnostics. They receive a scheduled
                    // opportunity so a dependency can repair them, but an
                    // unrepaired fragment has no Tick outcome of its own.
                    succeeded[node_index] = true;
                    continue;
                }
                effects.push(Effect::Diagnose(Diagnostic::for_expression(
                    root.anchor,
                    root.expression.span(),
                    error.to_string(),
                )));
                continue;
            }
        };

        match interpret(&atoms, tick_inputs(tick, root.anchor)) {
            Ok(Interpretation::Cell(Atom::Empty)) => succeeded[node_index] = true,
            Ok(Interpretation::Cell(atom)) => {
                let encoding = atom.to_string();
                let output = match root.output {
                    Ok(Some(output)) => output,
                    Ok(None) => {
                        succeeded[node_index] = true;
                        continue;
                    }
                    Err(reason) => {
                        effects.push(Effect::Diagnose(Diagnostic::for_expression(
                            root.anchor,
                            root.expression.span(),
                            portal_message(reason, &encoding),
                        )));
                        continue;
                    }
                };
                if encoding.len() != 2 {
                    effects.push(Effect::Diagnose(Diagnostic::for_expression(
                        root.anchor,
                        root.expression.span(),
                        format!("result {encoding:?} is not a scalar Cell pair"),
                    )));
                    continue;
                }
                let write = match Portal::at(grid, output).admit(&encoding) {
                    Ok(write) => write,
                    Err(reason) => {
                        effects.push(Effect::Diagnose(Diagnostic::for_expression(
                            root.anchor,
                            root.expression.span(),
                            portal_message(reason, &encoding),
                        )));
                        continue;
                    }
                };
                if matches!(atom, Atom::Bang) {
                    bang_events.push(output);
                }
                apply_write(&mut working, &write);
                effects.push(Effect::Write(write));
                succeeded[node_index] = true;
            }
            Ok(Interpretation::Sequence(sequence)) if sequence.is_empty() => {
                succeeded[node_index] = true;
            }
            Ok(Interpretation::Sequence(sequence)) => {
                effects.push(Effect::Diagnose(Diagnostic::for_expression(
                    root.anchor,
                    root.expression.span(),
                    format!(
                        "Sequence result {:?} has no fixed scalar scheduling footprint",
                        sequence.to_string()
                    ),
                )));
            }
            Ok(Interpretation::Play(performance)) => {
                effects.push(Effect::Play(performance));
                succeeded[node_index] = true;
            }
            Err(error) => effects.push(Effect::Diagnose(Diagnostic::for_expression(
                root.anchor,
                root.expression.span(),
                error.to_string(),
            ))),
        }
    }
    resolve(effects)
}

fn apply_write(bytes: &mut [u8], write: &SpanWrite) {
    for (cell, content) in write.cells() {
        bytes[cell.get()] = content.as_char() as u8;
    }
}

fn portal_message(reason: PortalError, encoding: &str) -> String {
    match reason {
        PortalError::BelowSource => format!("result {encoding:?} falls below the Source"),
        PortalError::CrossesRowEdge => format!("result {encoding:?} crosses the row edge"),
        PortalError::InvalidContent => {
            format!("result {encoding:?} contains Cells outside printable ASCII")
        }
    }
}

fn activates(grid: Grid, bang: Position, root: Position) -> bool {
    grid.assert_owns(bang);
    grid.assert_owns(root);
    (bang.x() == root.x() && bang.y().abs_diff(root.y()) == 1)
        || (bang.y() == root.y() && bang.x().abs_diff(root.x()) == 2)
}

fn schedule<'a>(
    grid: Grid,
    map: &'a LanguageMap,
    destinations: &BTreeMap<CellIndex, Position>,
) -> Result<Schedule<'a>, Vec<Diagnostic>> {
    let mut roots: Vec<_> = map
        .expressions()
        .filter_map(|expression| {
            expression
                .function_candidate()
                .map(|(anchor, function)| ScheduledRoot {
                    anchor,
                    function,
                    expression,
                    output: if function.is_terminal() {
                        Ok(None)
                    } else if let Some(destination) = destinations.get(&grid.index(anchor)) {
                        Ok(Some(*destination))
                    } else {
                        Portal::ordinary_result(grid, anchor)
                            .map(|portal| Some(portal.destination()))
                    },
                })
        })
        .collect();
    roots.sort_by_key(|root| grid.index(root.anchor));

    let mut diagnostics = Vec::new();
    let mut slots = Vec::new();
    for (root_index, root) in roots.iter().enumerate() {
        let layout_width = root
            .expression
            .layout()
            .map(|(offset, token, _)| offset + token.len())
            .max()
            .unwrap_or(0);
        let span_width =
            root.expression.span().end().get() - root.expression.span().start().get() + 1;
        if span_width > layout_width {
            diagnostics.push(Diagnostic::for_expression(
                root.anchor,
                root.expression.span(),
                "trailing Source makes this Expression structurally unstable".to_owned(),
            ));
            continue;
        }
        for (offset, token, _) in root.expression.layout() {
            if !grid.fits(root.anchor, offset + token.len()) {
                diagnostics.push(Diagnostic::for_expression(
                    root.anchor,
                    root.expression.span(),
                    "Expression layout crosses the row edge".to_owned(),
                ));
                break;
            }
            let position = grid
                .position(root.anchor.x() + offset, root.anchor.y())
                .expect("a checked Expression slot is inside its row");
            slots.push((root_index, position, token));
        }
    }

    let mut dependencies = Vec::new();
    let mut output_cells: BTreeMap<CellIndex, usize> = BTreeMap::new();
    for (producer_index, producer) in roots.iter().enumerate() {
        let output = match producer.output {
            Ok(Some(output)) => output,
            Ok(None) => continue,
            // A result that is Empty needs no destination. Resolve this
            // failure only after evaluation establishes that a value exists.
            Err(_) => continue,
        };
        if !grid.fits(output, 2) {
            diagnostics.push(Diagnostic::for_expression(
                producer.anchor,
                producer.expression.span(),
                "a scalar output crosses the row edge".to_owned(),
            ));
            continue;
        }
        let output_start = grid.index(output);
        let output_indices = [
            output_start,
            grid.cell_index(output_start.get() + 1)
                .expect("a checked two-Cell output is inside the Grid"),
        ];
        for cell in output_indices {
            if let Some(other) = output_cells.insert(cell, producer_index)
                && other != producer_index
            {
                diagnostics.push(Diagnostic::for_expression(
                    producer.anchor,
                    producer.expression.span(),
                    "multiple current-Tick producers write the same Cell".to_owned(),
                ));
            }
        }

        let mut output_is_slot = false;
        for &(consumer_index, slot, token) in &slots {
            let slot_start = grid.index(slot).get();
            let slot_end = slot_start + token.len();
            let output_end = output_start.get() + 2;
            let overlaps = output_start.get() < slot_end && slot_start < output_end;
            if !overlaps {
                continue;
            }
            output_is_slot = true;
            if matches!(token, lang::Token::Function) {
                diagnostics.push(Diagnostic::for_expression(
                    producer.anchor,
                    producer.expression.span(),
                    "current-Tick output cannot replace Function structure".to_owned(),
                ));
            } else if output_start.get() == slot_start && token.len() == 2 {
                dependencies.push(Dependency {
                    producer: producer_index,
                    consumer: consumer_index,
                    kind: DependencyKind::Data,
                });
            } else {
                diagnostics.push(Diagnostic::for_expression(
                    producer.anchor,
                    producer.expression.span(),
                    "current-Tick output only partly covers an operand".to_owned(),
                ));
            }
        }

        if producer.function.can_emit_bang() && !output_is_slot {
            for (consumer_index, consumer) in roots.iter().enumerate() {
                if consumer.function.is_terminal() && activates(grid, output, consumer.anchor) {
                    dependencies.push(Dependency {
                        producer: producer_index,
                        consumer: consumer_index,
                        kind: DependencyKind::Activation,
                    });
                }
            }
        }
    }

    dependencies.sort_by_key(|dependency| {
        (
            dependency.producer,
            dependency.consumer,
            dependency.kind as u8,
        )
    });
    dependencies.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut order = Vec::with_capacity(roots.len());
    let mut ready = BTreeSet::new();
    for node in 0..roots.len() {
        if !dependencies
            .iter()
            .any(|dependency| dependency.consumer == node)
        {
            ready.insert(node);
        }
    }
    while let Some(node) = ready.pop_first() {
        order.push(node);
        for candidate in 0..roots.len() {
            if order.contains(&candidate) || ready.contains(&candidate) {
                continue;
            }
            if dependencies
                .iter()
                .filter(|dependency| dependency.consumer == candidate)
                .all(|dependency| order.contains(&dependency.producer))
            {
                ready.insert(candidate);
            }
        }
    }
    if order.len() != roots.len() {
        let root = roots
            .iter()
            .enumerate()
            .find(|(index, _)| !order.contains(index))
            .map(|(_, root)| root)
            .expect("an incomplete topological order leaves one root");
        return Err(vec![Diagnostic::for_expression(
            root.anchor,
            root.expression.span(),
            "same-Tick dependency cycle".to_owned(),
        )]);
    }

    Ok(Schedule {
        roots,
        dependencies,
        order,
    })
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
            // Element order within one producer's group, producer order between
            // groups: extending preserves both, where pushing the group as one
            // item would have made the Tick Plan carry a shape the Playback
            // Engine does not deliver.
            Effect::Play(performance) => play_commands.extend(&performance),
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
    Interpreter::execute(atoms, inputs)
}

///
/// The effects of one Expression root's turn.
///
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
#[cfg(test)]
pub(super) fn result_effect(
    grid: Grid,
    root: Position,
    span: Span,
    result: Result<Interpretation, LangError>,
) -> Option<Effect> {
    grid.assert_owns_index(span.start());
    grid.assert_owns_index(span.end());
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
        Ok(Interpretation::Play(performance)) => return Some(Effect::Play(performance)),
        Err(error) => {
            return Some(Effect::Diagnose(Diagnostic::for_expression(
                root,
                span,
                error.to_string(),
            )));
        }
    };
    // Resolution, validity, and fit are one expression because a producer
    // answers all refusals the same way: no write, and a diagnostic saying which it
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
                    PortalError::InvalidContent => {
                        format!("result {encoded:?} contains Cells outside printable ASCII")
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
#[cfg(test)]
mod test {
    use super::{Effect, Portal, Tick, resolve, result_effect};

    #[test]
    fn fixed_upward_portals_schedule_note_and_bang_before_midi() {
        let grid = Grid::new(16, 5);
        let rows = ["", "", "!>007FD4", "      .^3C", ".=0101"];
        let bytes = rows
            .iter()
            .map(|row| format!("{row:16}"))
            .collect::<String>();
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let destinations = [
            (
                grid.index(grid.position(6, 3).unwrap()),
                grid.position(6, 2).unwrap(),
            ),
            (
                grid.index(grid.position(0, 4).unwrap()),
                grid.position(0, 3).unwrap(),
            ),
        ]
        .into_iter()
        .collect();

        let plan =
            super::plan_with_destinations(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert!(plan.diagnostics.is_empty());
        assert_eq!(
            planned(&plan),
            vec![(38, 'C'), (39, '4'), (48, '*'), (49, '*')]
        );
    }

    #[test]
    fn two_current_bang_results_still_execute_midi_once() {
        let grid = Grid::new(16, 6);
        let rows = ["", ".=0101", "", "!>007FC4", "", ".=0202"];
        let bytes = rows
            .iter()
            .map(|row| format!("{row:16}"))
            .collect::<String>();
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let destinations = [
            (
                grid.index(grid.position(0, 1).unwrap()),
                grid.position(0, 2).unwrap(),
            ),
            (
                grid.index(grid.position(0, 5).unwrap()),
                grid.position(0, 4).unwrap(),
            ),
        ]
        .into_iter()
        .collect();

        let plan =
            super::plan_with_destinations(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert!(plan.diagnostics.is_empty());
    }

    #[test]
    fn competing_writers_and_dependency_cycles_abort_before_output() {
        let conflict_grid = Grid::new(16, 2);
        let conflict_bytes = format!("{:<16}{:<16}", ".+0102", ".+0304");
        let conflict_map = LanguageMap::build(conflict_grid, conflict_bytes.as_bytes());
        let shared = conflict_grid.position(10, 1).unwrap();
        let conflict_destinations = [
            (conflict_grid.cell_index(0).unwrap(), shared),
            (conflict_grid.cell_index(16).unwrap(), shared),
        ]
        .into_iter()
        .collect();
        let conflict = super::plan_with_destinations(
            conflict_grid,
            conflict_bytes.as_bytes(),
            &conflict_map,
            Tick::ZERO,
            &conflict_destinations,
        );
        assert!(conflict.writes.is_empty());
        assert!(conflict.play_commands.is_empty());
        assert!(conflict.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("multiple current-Tick producers")
        }));

        let cycle_grid = Grid::new(16, 2);
        let cycle_bytes = format!("{:<16}{:<16}", ".+0001", ".+0001");
        let cycle_map = LanguageMap::build(cycle_grid, cycle_bytes.as_bytes());
        let cycle_destinations = [
            (
                cycle_grid.cell_index(0).unwrap(),
                cycle_grid.position(2, 1).unwrap(),
            ),
            (
                cycle_grid.cell_index(16).unwrap(),
                cycle_grid.position(2, 0).unwrap(),
            ),
        ]
        .into_iter()
        .collect();
        let cycle = super::plan_with_destinations(
            cycle_grid,
            cycle_bytes.as_bytes(),
            &cycle_map,
            Tick::ZERO,
            &cycle_destinations,
        );
        assert!(cycle.writes.is_empty());
        assert_eq!(cycle.diagnostics.len(), 1);
        assert_eq!(cycle.diagnostics[0].message, "same-Tick dependency cycle");
    }
    use lang::{Atom, Interpretation, Sequence};

    use crate::{
        grid::{CellIndex, Grid},
        source::{
            CellWrite, Diagnostic, MidiChannel, Note, Performance, PlayCommand, TickPlan, Velocity,
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

    ///
    /// One Raw Play Command, stated as the three Numbers a Source writes.
    ///
    fn raw(channel: u8, velocity: u8, note: u8) -> PlayCommand {
        PlayCommand::Raw {
            channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
            velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
            note: Note::try_from(note).expect("a MIDI note"),
        }
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
    fn test_a_write_whose_destination_leaves_the_grid_emits_no_partial_write() {
        // ADR 0004: a complete write validates its whole destination before
        // any Cell of it enters the Tick Plan. This root's result has nowhere
        // below it to go, so the turn contributes a diagnostic and nothing
        // else — not the first Cell of a result that could not be placed.
        let grid = Grid::new(10, 2);
        let root = grid.position(0, 1).unwrap();
        let span = super::Span::new(grid, cell(grid, 10), cell(grid, 15));

        assert_eq!(
            result_effect(grid, root, span, Ok(Interpretation::Cell(Atom::Number(3))))
                .into_iter()
                .collect::<Vec<_>>(),
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
        //
        // The earlier producer performs a group of two, which is ADR 0030's
        // widened Expression. Element index orders the commands inside one
        // producer's Effect and ADR 0020's producer order holds around it, so
        // the Tick Plan reads as though the chord had been written left to
        // right as separate Expressions. The three commands differ in every
        // field that can be read back, so a group flattened in reverse, or a
        // producer order that let the later Expression in first, is a different
        // Tick Plan rather than the same one.
        let grid = Grid::new(10, 3);
        let first = raw(0, 1, 60);
        let second = raw(0, 1, 64);
        let third = raw(1, 2, 61);
        let earlier = diagnostic(grid, 0, 5, "earlier producer");
        let later = diagnostic(grid, 20, 25, "later producer");

        let plan = resolve(vec![
            Effect::Play(Performance::Many(vec![first, second])),
            earlier.clone(),
            write(grid, 10, "0"),
            Effect::Play(Performance::One(third)),
            later.clone(),
        ]);

        assert_eq!(plan.play_commands, vec![first, second, third]);
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
    fn a_widened_terminal_root_answers_one_effect_carrying_its_whole_group() {
        // ADR 0020 orders effects by their producer's Position, and every
        // command of one widened Expression shares that Position, so the group
        // crosses the seam as a single Effect with element index ordering it
        // inside. Splitting it into one Effect per command at this seam would
        // put an order between them that no anchor decides.
        //
        // No Source text reaches this yet: the Range Functions that would spell
        // a Sequence operand are `sequence-values/05`, so the answer is stated
        // at the seam it arrives through, as the Sequence result test does.
        let grid = Grid::new(10, 3);
        let root = grid.position(0, 0).expect("inside the Grid");
        let span = Span::new(grid, cell(grid, 0), cell(grid, 5));
        let chord = vec![raw(0, 0x7F, 60), raw(0, 0x7F, 64), raw(0, 0x7F, 67)];

        let effect = result_effect(
            grid,
            root,
            span,
            Ok(Interpretation::Play(Performance::Many(chord.clone()))),
        );

        assert_eq!(
            effect,
            Some(Effect::Play(Performance::Many(chord.clone()))),
            "a widened Play crossed the seam as something other than one group"
        );
        assert_eq!(
            resolve(effect.into_iter().collect()).play_commands,
            chord,
            "the group reached the Tick Plan out of element index order"
        );
    }

    #[test]
    fn a_terminal_root_that_performs_no_output_plans_no_play_commands() {
        // The empty Sequence operand: a real width of no elements, so the
        // Expression is well formed and answers an effect that happens to carry
        // nothing. It must still be an effect — a Play answers no value at any
        // width — so this plans no write and no diagnostic either, which is the
        // difference between performing nothing and failing to perform.
        let grid = Grid::new(10, 3);
        let root = grid.position(0, 0).expect("inside the Grid");
        let span = Span::new(grid, cell(grid, 0), cell(grid, 5));

        let effect = result_effect(
            grid,
            root,
            span,
            Ok(Interpretation::Play(Performance::Many(Vec::new()))),
        );

        assert_eq!(effect, Some(Effect::Play(Performance::Many(Vec::new()))));

        let plan = resolve(effect.into_iter().collect());

        assert!(plan.play_commands.is_empty());
        assert!(plan.writes.is_empty());
        assert!(plan.diagnostics.is_empty());
    }

    #[test]
    #[should_panic(expected = "CellIndex belongs to another Grid")]
    fn result_effect_rejects_a_span_from_another_grid() {
        let grid = Grid::new(10, 1);
        let other_grid = Grid::new(10, 1);
        let root = grid.position(0, 0).expect("inside the Grid");
        let span = Span::new(other_grid, cell(other_grid, 0), cell(other_grid, 5));

        // With no row below the root, delivery would otherwise produce a
        // diagnostic whose Cells belong to a different Source.
        let _ = result_effect(grid, root, span, Ok(sequence(&[0x0A])));
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
    fn a_sequence_with_nonprintable_characters_diagnoses_without_partial_writes() {
        let grid = Grid::new(10, 3);
        let root = grid.position(2, 0).expect("inside the Grid");
        let span = Span::new(grid, cell(grid, 2), cell(grid, 7));

        for character in ['\0', '\x1f', '\x7f', 'é'] {
            let encoded = format!("0A{character}");
            let sequence = Sequence::new([Atom::Number(0x0A), Atom::Char(character)])
                .expect("Numbers and Chars are Sequence members");
            let effects: Vec<_> =
                result_effect(grid, root, span, Ok(Interpretation::Sequence(sequence)))
                    .into_iter()
                    .collect();

            assert_eq!(
                effects,
                vec![diagnostic(
                    grid,
                    2,
                    7,
                    &format!("result {encoded:?} contains Cells outside printable ASCII")
                )]
            );
            assert!(resolve(effects).writes.is_empty());
        }
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
