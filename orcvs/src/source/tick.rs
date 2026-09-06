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

///
/// One parsed operand slot, as the Cells covering it answer for it.
///
/// `start` is the slot's first Cell index rather than its Position, because
/// every question asked of it compares against an output's Cell index.
///
#[derive(Clone, Copy, Debug, PartialEq)]
struct Slot {
    consumer: usize,
    start: usize,
    token: lang::Token,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Dependency {
    producer: usize,
    consumer: usize,
    kind: DependencyKind,
}

struct Schedule<'a> {
    roots: Vec<ScheduledRoot<'a>>,
    order: Vec<usize>,
    ///
    /// The Data suppliers of each root, in dependency order.
    ///
    /// Execution asks this once per root, so it is indexed by consumer rather
    /// than rescanned out of `dependencies`: a Tick runs under the playback
    /// deadline, and scanning every edge for every root spends the product of
    /// the two to answer a question each root asks about itself.
    ///
    data_suppliers: Vec<Vec<usize>>,
    ///
    /// Whether each root's output lands on a parsed operand slot.
    ///
    /// The scheduler reads this to decide whether a Bang result is an
    /// activation event; execution reads the same answer, so activation can
    /// never depend on the order two independent roots happen to take.
    ///
    output_is_slot: Vec<bool>,
    /// One diagnostic for each candidate kept out of the schedule entirely.
    excluded: Vec<Diagnostic>,
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
    // A candidate the schedule could not take is diagnosed once, before any
    // root's turn, and changes nothing else about this Tick.
    let mut effects: Vec<_> = schedule
        .excluded
        .iter()
        .cloned()
        .map(Effect::Diagnose)
        .collect();

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

    // A root is settled once its turn can contribute nothing further, whether
    // it wrote a result, answered with absence, or had no destination to write
    // to. Only a root that failed outright leaves this false, because only
    // that leaves a consumer reading an operand nobody can account for.
    let mut settled = vec![false; schedule.roots.len()];
    // Which roots a Bang has reached so far this Tick. Recording it as the
    // Bangs are produced, rather than re-deriving it from a list of them at
    // each terminal root's turn, keeps the answer the same — only a Bang
    // produced earlier in the order can have reached this root — and asks it
    // once per Bang instead of once per pair.
    let mut activated = vec![false; schedule.roots.len()];
    for node_index in schedule.order.iter().copied() {
        let root = schedule.roots[node_index];
        let failed_input = schedule.data_suppliers[node_index]
            .iter()
            .copied()
            .find(|producer| !settled[*producer]);
        if let Some(producer) = failed_input {
            // Naming the supplier matters because the consumer is the one root
            // in this Tick that did nothing wrong: without its anchor, a Tick
            // Plan points only at the Expression that was waiting.
            let supplier = schedule.roots[producer].anchor;
            effects.push(Effect::Diagnose(Diagnostic::for_expression(
                root.anchor,
                root.expression.span(),
                format!(
                    "a current-Tick data dependency at column {}, row {} failed",
                    supplier.x(),
                    supplier.y()
                ),
            )));
            continue;
        }

        if root.function.is_terminal() && !activated[node_index] {
            settled[node_index] = true;
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
                    //
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
            Ok(Interpretation::Cell(Atom::Empty)) => settled[node_index] = true,
            Ok(Interpretation::Cell(atom)) => {
                let encoding = atom.to_string();
                let output = match root.output {
                    Ok(Some(output)) => output,
                    Ok(None) => {
                        settled[node_index] = true;
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
                // ADR 0032: a `**` rejected in a typed operand is still
                // invalid syntax, and neither activates nor receives display
                // cleanup. The schedule drew its activation edges under
                // exactly this predicate, so delivery reads the same answer
                // and activation never turns on which root went first.
                if matches!(atom, Atom::Bang) && !schedule.output_is_slot[node_index] {
                    for anchor in activated_anchors(grid, output).into_iter().flatten() {
                        if let Some(consumer) = root_at(grid, &schedule.roots, anchor) {
                            activated[consumer] = true;
                        }
                    }
                }
                apply_write(&mut working, &write);
                effects.push(Effect::Write(write));
                settled[node_index] = true;
            }
            Ok(Interpretation::Sequence(sequence)) if sequence.is_empty() => {
                settled[node_index] = true;
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
                settled[node_index] = true;
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

///
/// The anchors a Bang at `bang` activates: north, south, west, east.
///
/// ADR 0006 names four aligned cardinal anchors and no others, so activation
/// is a question about four Positions rather than about every root on the
/// Grid. Asking it the other way round — testing each root against each Bang —
/// costs the product of the two on a path a Tick runs under the playback
/// deadline, and answers the same four times over. A Position off the Grid is
/// absent rather than an error: an edge has fewer neighbours, not invalid
/// ones.
///
fn activated_anchors(grid: Grid, bang: Position) -> [Option<Position>; 4] {
    grid.assert_owns(bang);
    let (column, row) = (bang.x(), bang.y());
    [
        row.checked_sub(1)
            .and_then(|north| grid.position(column, north)),
        grid.position(column, row + 1),
        column
            .checked_sub(2)
            .and_then(|west| grid.position(west, row)),
        grid.position(column + 2, row),
    ]
}

///
/// The root anchored at `anchor`, from roots in anchor order.
///
/// Two roots cannot share an anchor, so this answers exactly one or none.
///
fn root_at(grid: Grid, roots: &[ScheduledRoot<'_>], anchor: Position) -> Option<usize> {
    roots
        .binary_search_by_key(&grid.index(anchor).get(), |root| {
            grid.index(root.anchor).get()
        })
        .ok()
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

    // A candidate whose own layout does not stand up takes no turn, and takes
    // nothing else down with it: it contributes no slots, no edges, and no
    // output, so the roots around it schedule as though it were not there.
    let mut excluded = Vec::new();
    let mut layouts = Vec::with_capacity(roots.len());
    roots.retain(|root| match root_layout(grid, root) {
        Ok(layout) => {
            layouts.push(layout);
            true
        }
        Err(diagnostic) => {
            excluded.push(diagnostic);
            false
        }
    });
    // The only question asked of the slots is which of them a two-Cell output
    // lands on. Scanning all of them for every producer answers it in the
    // product of the two, and a Tick pays that under the playback deadline, so
    // they are put in Cell order once and searched instead.
    //
    // Ordering them is not free of meaning: an incomplete Function declares
    // the slots it is missing, and those run past its own Language Map extent
    // into the Cells of the Expression after it. Slots therefore overlap each
    // other, and a producer landing on one can land on more than one.
    let mut slots: Vec<Slot> = layouts
        .into_iter()
        .enumerate()
        .flat_map(|(consumer, layout)| {
            layout.into_iter().map(move |(position, token)| Slot {
                consumer,
                start: grid.index(position).get(),
                token,
            })
        })
        .collect();
    slots.sort_by_key(|slot| slot.start);
    // How far back a slot can start and still reach an output: the bound that
    // turns "every slot before this one" into a short run of candidates.
    let widest_slot = slots.iter().map(|slot| slot.token.len()).max().unwrap_or(0);

    let mut diagnostics = Vec::new();
    let mut output_is_slot = vec![false; roots.len()];
    let mut dependencies = Vec::new();
    let mut output_cells: BTreeMap<CellIndex, usize> = BTreeMap::new();
    // Producers whose destination would join a neighbouring run, and the
    // diagnostic each one owes. Applied after the walk, because taking a
    // root's destination away while the walk still reads destinations would
    // change the answer the walk is in the middle of giving.
    let mut joins: Vec<(usize, Diagnostic)> = Vec::new();
    for (producer_index, producer) in roots.iter().enumerate() {
        let output = match producer.output {
            Ok(Some(output)) => output,
            Ok(None) => continue,
            // A result that is Empty needs no destination. Resolve this
            // failure only after evaluation establishes that a value exists.
            Err(_) => continue,
        };
        if grid.offset_in_row(output, 1).is_none() {
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

        for slot in covering_slots(&slots, widest_slot, output_start.get()) {
            let Slot {
                consumer: consumer_index,
                start: slot_start,
                token,
            } = slot;
            output_is_slot[producer_index] = true;
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

        // A row is partitioned into runs at its spaces, so a two-Cell result
        // written flush against another Expression's run joins the two. The
        // next parse then walks one longer run whose first Language Unit is no
        // longer the Function that was there, and the root this graph is
        // executing stops existing — permanently, because the display that
        // replaced it is no longer a Bang any cleanup can find.
        //
        // Only an output that landed on no parsed slot is asked. Abutting a
        // run is exactly how a write completes an incomplete Function, whose
        // missing slots run past its own extent; that write is the supported
        // case and the slots above already accounted for it. What is left is a
        // structural projection the stable graph cannot take, and ADR 0032
        // diagnoses it rather than executing it.
        if !output_is_slot[producer_index]
            && let Some(joined) = adjacent_root(grid, &roots, producer_index, output_start)
        {
            let neighbour = roots[joined].anchor;
            joins.push((
                producer_index,
                Diagnostic::for_expression(
                    producer.anchor,
                    producer.expression.span(),
                    format!(
                        "current-Tick output would join the Expression at column {}, row {}",
                        neighbour.x(),
                        neighbour.y()
                    ),
                ),
            ));
            continue;
        }

        if producer.function.can_emit_bang() && !output_is_slot[producer_index] {
            for anchor in activated_anchors(grid, output).into_iter().flatten() {
                if let Some(consumer_index) = root_at(grid, &roots, anchor)
                    && roots[consumer_index].function.is_terminal()
                {
                    dependencies.push(Dependency {
                        producer: producer_index,
                        consumer: consumer_index,
                        kind: DependencyKind::Activation,
                    });
                }
            }
        }
    }

    // A destination that would join a neighbouring run is taken away rather
    // than executed, and the root keeps its turn without one. It reaches the
    // same settled state as a root that answered with absence, which is what
    // it now is: a value nobody can receive. Only a destination that landed on
    // no parsed slot arrives here, so no consumer was waiting on it and no
    // Data edge is being cut — the rest of the program plays, as it does for
    // any other failure local to one Expression.
    for (producer_index, diagnostic) in joins {
        roots[producer_index].output = Ok(None);
        excluded.push(diagnostic);
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

    let mut data_suppliers = vec![Vec::new(); roots.len()];
    for dependency in &dependencies {
        if dependency.kind == DependencyKind::Data {
            data_suppliers[dependency.consumer].push(dependency.producer);
        }
    }

    let mut indegree = vec![0usize; roots.len()];
    let mut outgoing = vec![Vec::new(); roots.len()];
    for dependency in &dependencies {
        indegree[dependency.consumer] += 1;
        outgoing[dependency.producer].push(dependency.consumer);
    }

    let mut order = Vec::with_capacity(roots.len());
    let mut ready = BTreeSet::new();
    for (node, &incoming) in indegree.iter().enumerate() {
        if incoming == 0 {
            ready.insert(node);
        }
    }
    while let Some(node) = ready.pop_first() {
        order.push(node);
        for &consumer in &outgoing[node] {
            indegree[consumer] -= 1;
            if indegree[consumer] == 0 {
                ready.insert(consumer);
            }
        }
    }
    if order.len() != roots.len() {
        let root = roots
            .iter()
            .enumerate()
            .find(|(index, _)| indegree[*index] != 0)
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
        order,
        data_suppliers,
        output_is_slot,
        excluded,
    })
}

///
/// Every slot a two-Cell output starting at `output` lands on any part of,
/// in the root order a producer's diagnostics follow.
///
/// `slots` is in ascending `start` order and `widest` is the widest slot in
/// it, which together bound the candidates to one short run: a slot starting
/// more than `widest` Cells before the output cannot reach it, and one
/// starting at or after the output's second Cell has not begun. Both are
/// preconditions rather than checks — a `widest` that understates one slot
/// would drop it silently — so both are asserted in a test build.
///
///
/// The root whose run a two-Cell output written at `output` would join.
///
/// A row is partitioned at its spaces, so an output is joined to a run only by
/// being flush against it. Exactly two roots can be flush against a given
/// output — the one whose run begins one Cell after the output ends, and the
/// one whose run ends one Cell before it begins — so this asks about the
/// handful of roots around that point rather than scanning all of them, on a
/// path a Tick runs under the playback deadline.
///
/// `roots` is in anchor order, which is span order: an Expression's anchor
/// sits inside its own span, and spans partition each row left to right.
///
/// An output that overlaps a run rather than abutting it is not this
/// question. That is either a write to a parsed operand slot or a structural
/// write over one, and both are already answered by the slots.
///
fn adjacent_root(
    grid: Grid,
    roots: &[ScheduledRoot<'_>],
    producer: usize,
    output: CellIndex,
) -> Option<usize> {
    let output_start = output.get();
    let output_end = output_start + 1;
    let row = grid.position_at(output).y();

    let boundary =
        roots.partition_point(|root| root.expression.span().start().get() < output_start);

    (boundary.saturating_sub(1)..=boundary + 1)
        .take_while(|index| *index < roots.len())
        .filter(|index| *index != producer)
        .find(|index| {
            let span = roots[*index].expression.span();
            grid.position_at(span.start()).y() == row
                && (span.start().get() == output_end + 1 || span.end().get() + 1 == output_start)
        })
}

fn covering_slots(slots: &[Slot], widest: usize, output: usize) -> Vec<Slot> {
    debug_assert!(
        slots.is_sorted_by_key(|slot| slot.start),
        "covering_slots searches slots in Cell order",
    );
    debug_assert!(
        slots.iter().all(|slot| slot.token.len() <= widest),
        "a slot wider than `widest` would be searched past and silently dropped",
    );

    let output_end = output + 2;
    let first = slots.partition_point(|slot| slot.start + widest <= output);
    let past = slots.partition_point(|slot| slot.start < output_end);
    let mut covering: Vec<_> = slots[first..past]
        .iter()
        .copied()
        .filter(|slot| output < slot.start + slot.token.len())
        .collect();
    // Cell order found them; root order is the order their producer's
    // diagnostics are emitted in, which is the order ADR 0020 describes.
    covering.sort_by_key(|slot| (slot.consumer, slot.start));
    covering
}

///
/// This candidate's parsed operand slots as Grid Positions, or the one
/// diagnostic that keeps it out of the schedule.
///
/// Trailing Source and a layout running past the row edge are properties of
/// one Expression rather than of the graph. ADR 0032 rejects a whole Tick for
/// competing writers, cycles, partial operand writes, and structural
/// projections the stable graph cannot express; neither of these is one of
/// those, and both are ordinary states to pass through while typing. So the
/// Expression that has one is diagnosed and left inert, and every other root
/// still plays.
///
fn root_layout(
    grid: Grid,
    root: &ScheduledRoot<'_>,
) -> Result<Vec<(Position, lang::Token)>, Diagnostic> {
    let layout_width = root
        .expression
        .layout()
        .map(|(offset, token, _)| offset + token.len())
        .max()
        .unwrap_or(0);
    let span_width = root.expression.span().end().get() - root.expression.span().start().get() + 1;
    if span_width > layout_width {
        return Err(Diagnostic::for_expression(
            root.anchor,
            root.expression.span(),
            "trailing Source makes this Expression structurally unstable".to_owned(),
        ));
    }

    root.expression
        .layout()
        .map(|(offset, token, _)| {
            grid.offset_in_row(root.anchor, offset + token.len() - 1)
                .ok_or_else(|| {
                    Diagnostic::for_expression(
                        root.anchor,
                        root.expression.span(),
                        "Expression layout crosses the row edge".to_owned(),
                    )
                })?;
            let position = grid
                .position(root.anchor.x() + offset, root.anchor.y())
                .expect("a checked Expression slot is inside its row");
            Ok((position, token))
        })
        .collect()
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
/// test state the inputs of exactly the Tick it drove rather than of
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

#[cfg(test)]
mod test {
    use super::{Effect, Portal, Tick, observed, resolve};

    ///
    /// Builds one Source Snapshot from `rows`, padded to the Grid's width.
    ///
    fn snapshot(grid: Grid, rows: &[&str]) -> String {
        let width = grid.count() / rows.len();
        rows.iter().map(|row| format!("{row:width$}")).collect()
    }

    #[test]
    fn a_bang_activates_its_aligned_neighbours_and_no_further_root() {
        // ADR 0006 names four aligned cardinal anchors — north, south, west,
        // east — and scheduling now asks about those four Positions rather
        // than testing every root against every Bang. What that has to keep is
        // the boundary: the root one row away performs, and the root two rows
        // away is untouched by the same Bang.
        //
        // The Bang is delivered to (0, 2) so the roots around it are ordinary
        // Terminal Output Expressions rather than an arrangement bent to reach
        // a downward Portal.
        let grid = Grid::new(16, 6);
        let bytes = snapshot(
            grid,
            &["", "!>007FC4", "", "!>007FC5", "!>007FC6", ".=0101"],
        );
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let destinations = [(
            grid.index(grid.position(0, 5).unwrap()),
            grid.position(0, 2).unwrap(),
        )]
        .into_iter()
        .collect();

        let plan =
            super::plan_with_destinations(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        // North of the Bang, then south. The root at row 4 is two rows from
        // the Bang and never performs.
        assert_eq!(
            plan.play_commands,
            vec![raw(0, 0x7F, 60), raw(0, 0x7F, 72)],
            "diagnostics: {:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn a_bang_rejected_in_a_typed_operand_never_activates() {
        // ADR 0032: "A `**` rejected in a typed operand is still invalid
        // syntax and neither activates nor receives display cleanup." The
        // Bang lands in `.+`'s left Number slot, so it is a rejected operand
        // spelling and not an activation event, even though it is Cell-aligned
        // with the terminal root below it.
        let grid = Grid::new(16, 5);
        let bytes = snapshot(grid, &["    .=0101", "  .+0102", "    !>007FC4", "", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());

        let plan = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

        assert_eq!(plan.play_commands, vec![]);
        assert_eq!(planned(&plan), vec![(20, '*'), (21, '*')]);
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("found \"**\"")),
            "the rejected operand keeps its syntax diagnostic: {:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn trailing_source_diagnoses_its_own_expression_and_leaves_the_tick_playing() {
        // The spec rejects a whole Tick for competing writers, cycles, partial
        // operand writes, and unsupported structural projections. Trailing
        // Source is none of those: it makes one Expression unstable, so that
        // Expression takes no scheduled turn while the rest of the program
        // plays.
        let grid = Grid::new(16, 4);
        let bytes = snapshot(grid, &[".=0101", "", "!>007FC4", ".+0102Z"]);
        let map = LanguageMap::build(grid, bytes.as_bytes());

        let plan = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert_eq!(planned(&plan), vec![(16, '*'), (17, '*')]);
        assert!(
            plan.diagnostics.iter().any(|diagnostic| {
                diagnostic.message == "trailing Source makes this Expression structurally unstable"
            }),
            "the unstable Expression keeps its own diagnostic: {:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn a_row_edge_fragment_diagnoses_its_own_expression_and_leaves_the_tick_playing() {
        // A Function typed into the last Cells of a row is ordinary live-edit
        // state, not a graph error. `Expression::layout` says missing tail
        // slots may run past the supplied fragment, so this root simply has no
        // Tick outcome of its own.
        let grid = Grid::new(16, 4);
        let bytes = snapshot(grid, &[".=0101", "", "!>007FC4", "            .+01"]);
        let map = LanguageMap::build(grid, bytes.as_bytes());

        let plan = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "Expression layout crosses the row edge"),
            "the off-row Expression keeps its own diagnostic: {:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn a_suppressed_consumer_names_the_supplier_that_did_not_settle() {
        // A consumer whose supplier failed is the one root in the Tick that
        // did nothing wrong. A half-typed `.+` at (2, 0) writes over the MIDI
        // channel operand and never settles, so the Note is suppressed — but
        // the Tick Plan has to say which Expression it is waiting on, or the
        // only thing it points at is the Expression that was waiting.
        let grid = Grid::new(16, 4);
        let bytes = snapshot(grid, &["  .+", "!>007FC4", "", ".=0101"]);
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let destinations = [(
            grid.index(grid.position(0, 3).unwrap()),
            grid.position(0, 2).unwrap(),
        )]
        .into_iter()
        .collect();

        let plan =
            super::plan_with_destinations(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        assert_eq!(plan.play_commands, vec![]);
        assert!(
            plan.diagnostics.iter().any(|diagnostic| {
                diagnostic.message == "a current-Tick data dependency at column 2, row 0 failed"
            }),
            "the suppressed consumer names its supplier: {:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor() {
        // ADR 0012's inputs are only as good as something watching the thread
        // from the Playback Engine to `Interpreter::execute`. Severing it —
        // passing a fixed Tick or a fixed anchor at the call site instead of
        // this root's own — fails here rather than passing unnoticed until
        // Clock reads a Tick and Random reads an anchor.
        //
        // The expected anchors are literals, so the assertion cannot be
        // satisfied by the code under test, and they are asymmetric so a
        // transposed column and row is visible. The Tick is not `Tick::ZERO`,
        // so a hardcoded first Tick is visible too.
        let grid = Grid::new(20, 3);
        let bytes = snapshot(grid, &[".+0102 .+0304", "          .-0504", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let tick = Tick::new(11);

        // whatever an earlier Tick on this thread interpreted is not part of
        // this one
        let _ = observed::take();
        let _ = super::plan(grid, bytes.as_bytes(), &map, tick);
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
            vec![(0, 0), (7, 0), (10, 1)],
            "each root is told its own anchor, in scheduled order"
        );
    }

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
    fn one_failed_bang_candidate_does_not_suppress_another_fresh_bang() {
        let grid = Grid::new(16, 7);
        let rows = [".=0101", ".=./010001", "", "", "", "!>007FC4", ""];
        let bytes = rows
            .iter()
            .map(|row| format!("{row:16}"))
            .collect::<String>();
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let destinations = [
            (
                grid.index(grid.position(0, 0).unwrap()),
                grid.position(0, 4).unwrap(),
            ),
            (
                grid.index(grid.position(0, 1).unwrap()),
                grid.position(0, 6).unwrap(),
            ),
        ]
        .into_iter()
        .collect();

        let plan =
            super::plan_with_destinations(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "cannot divide by zero")
        );
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

    use crate::{
        grid::{CellIndex, Grid},
        source::{
            CellWrite, Diagnostic, MidiChannel, Note, Performance, PlayCommand, TickPlan, Velocity,
            language_map::LanguageMap,
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
        // any Cell of it enters the Tick Plan. This root sits in the last row,
        // so its result has nowhere to go and the Tick contributes a
        // diagnostic and nothing else — not the first Cell of a result that
        // could not be placed.
        let grid = Grid::new(10, 2);
        let bytes = snapshot(grid, &["", ".+0102"]);
        let map = LanguageMap::build(grid, bytes.as_bytes());

        let plan = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

        assert!(plan.writes.is_empty());
        assert_eq!(
            plan.diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec!["result \"03\" falls below the Source"]
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
        // shape Sequence results make ordinary. Each root's encoding is
        // admitted through the Portal below it, which is where a Sequence
        // answer would arrive.
        let grid = Grid::new(20, 3);
        let effects = vec![write(grid, 20, "0A0B0C"), write(grid, 22, "0D0E0F")];

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
