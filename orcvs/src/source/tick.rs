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

use super::language_map::{ExpressionEntry, LanguageMap, Span};
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
    ///
    /// The anchor of a supplier that was kept out of the schedule, for each
    /// consumer it was going to write to.
    ///
    /// Such a supplier has no node to leave unsettled, so the answer it owes
    /// its consumer is recorded against the consumer instead.
    ///
    failed_suppliers: Vec<Option<Position>>,
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
        // Asked before anything about this root's inputs, because a terminal
        // root with no Bang never reads them: it takes no turn, so it has
        // nothing to say about an operand it was never going to look at.
        if root.function.is_terminal() && !activated[node_index] {
            settled[node_index] = true;
            continue;
        }

        // A supplier that took a turn and failed leaves its node unsettled; one
        // that never got a node was recorded against this consumer when the
        // schedule was built. Both are the same answer to the same question,
        // so they are asked as one.
        let failed_input = schedule.data_suppliers[node_index]
            .iter()
            .copied()
            .find(|producer| !settled[*producer])
            .map(|producer| schedule.roots[producer].anchor)
            .or(schedule.failed_suppliers[node_index]);
        if let Some(supplier) = failed_input {
            // Naming the supplier matters because the consumer is the one root
            // in this Tick that did nothing wrong: without its anchor, a Tick
            // Plan points only at the Expression that was waiting.
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

    // A candidate whose own layout does not stand up takes no turn: it
    // contributes no slots and no output of its own, and the roots around it
    // schedule as though it were not there.
    //
    // Its destination is the exception. A root that fails at evaluation leaves
    // its consumer unsettled and named; erasing this one from the graph
    // entirely would leave the same consumer reading whatever its operand
    // Cells still hold, which is the previous Tick's value presented as this
    // one's. The destination is a property of the anchor rather than of the
    // layout that failed, so it is still known, and it is kept precisely so a
    // consumer waiting on it can be told.
    let mut excluded = Vec::new();
    let mut unstable: Vec<(Position, CellIndex)> = Vec::new();
    // Paired and then unzipped, so a layout belongs to its root by
    // construction. A slot names its consumer by index into `roots`, and
    // building the two lists side by side would leave that correspondence
    // resting on the order a predicate happened to be called in — true today,
    // and silent about it if it ever stopped being.
    let (kept, layouts): (Vec<_>, Vec<_>) = std::mem::take(&mut roots)
        .into_iter()
        .filter_map(|root| match root_layout(grid, &root) {
            Ok(layout) => Some((root, layout)),
            Err(diagnostic) => {
                if let Ok(Some(output)) = root.output {
                    unstable.push((root.anchor, grid.index(output)));
                }
                excluded.push(diagnostic);
                None
            }
        })
        .unzip();
    roots = kept;
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

    // Every run this Tick's writes have to live beside, in the row-major order
    // the Map builds its Expressions in.
    //
    // Runs, not roots: a run holding no Function candidate is still Source the
    // next parse reads, and a result written into one changes what that parse
    // sees exactly as much as a result written into a root's does.
    //
    // And the runs as this Tick leaves them, not as the starting Snapshot
    // spells them: stale Bang display is cleared before the first root takes
    // its turn, so a result written where a `**` stood is written into empty
    // Source and joins nothing.
    let cleared: Vec<Span> = map.bangs().map(|(_, span)| span).collect();
    let mut spans: Vec<Span> = Vec::new();
    for expression in map.expressions() {
        extend_surviving_runs(grid, expression.span(), &cleared, &mut spans);
    }
    debug_assert!(
        spans.is_sorted_by_key(|span| span.start()),
        "disturbed_expression searches Expression Spans in Cell order",
    );

    let mut diagnostics = Vec::new();
    let mut output_is_slot = vec![false; roots.len()];
    let mut dependencies = Vec::new();
    let mut output_cells: BTreeMap<CellIndex, usize> = BTreeMap::new();
    // Producers whose destination this Tick cannot deliver to — one that
    // crosses the row edge, one that would disturb a neighbouring run — and
    // the diagnostic each one owes. Applied after the walk, because taking a
    // root's destination away while the walk still reads destinations would
    // change the answer the walk is in the middle of giving.
    let mut withdrawn: Vec<(usize, Diagnostic)> = Vec::new();
    for (producer_index, producer) in roots.iter().enumerate() {
        let output = match producer.output {
            Ok(Some(output)) => output,
            Ok(None) => continue,
            // A result that is Empty needs no destination. Resolve this
            // failure only after evaluation establishes that a value exists.
            Err(_) => continue,
        };
        // A destination with only one Cell left in its row can receive no
        // scalar result. That is a property of one Expression's anchor, like
        // the layout that crosses the same edge in `root_layout`, so it costs
        // that Expression its turn's destination and leaves every other root
        // playing rather than rejecting the whole Tick.
        if grid.offset_in_row(output, 1).is_none() {
            withdrawn.push((
                producer_index,
                Diagnostic::for_expression(
                    producer.anchor,
                    producer.expression.span(),
                    "a scalar output crosses the row edge".to_owned(),
                ),
            ));
            continue;
        }
        let output_start = grid.index(output);
        let output_indices = [
            output_start,
            grid.cell_index(output_start.get() + 1)
                .expect("a checked two-Cell output is inside the Grid"),
        ];

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
        // written into or flush against another Expression's run makes the
        // next parse walk a different run: one whose first Language Unit is no
        // longer what was there, and whose Cells the graph this Tick executed
        // no longer describes. Whatever was displayed there stops existing —
        // permanently, because the display that replaced it is no longer a
        // Bang any cleanup can find.
        //
        // Only an output that landed on no parsed slot is asked. Abutting a
        // run is exactly how a write completes an incomplete Function, whose
        // missing slots run past its own extent; landing inside one is how a
        // write repairs a typed operand. Both are the supported case and the
        // slots above already accounted for them. What is left is a structural
        // projection the stable graph cannot take, and ADR 0032 diagnoses it
        // rather than executing it.
        if !output_is_slot[producer_index]
            && let Some(disturbed) =
                disturbed_expression(grid, &spans, producer.expression.span(), output_start)
        {
            let neighbour = grid.position_at(disturbed.start());
            withdrawn.push((
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

        // Both Cells are registered, and `count` is what registers them: the
        // collision they may reveal is one collision, this producer meeting
        // whichever producer reached the destination first, and a two-Cell
        // output is not two conflicts. No producer can meet itself here —
        // the two Cells are distinct keys and no two roots share an anchor.
        //
        // Registered only once every reason to withdraw this destination has
        // been asked, because a producer that is about to lose its destination
        // writes nothing: holding its Cells here would reject the Tick over a
        // conflict with a writer that no longer exists.
        let contended = output_indices
            .into_iter()
            .filter_map(|cell| output_cells.insert(cell, producer_index))
            .count()
            > 0;
        if contended {
            diagnostics.push(Diagnostic::for_expression(
                producer.anchor,
                producer.expression.span(),
                "multiple current-Tick producers write the same Cell".to_owned(),
            ));
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

    // A supplier that never took a turn cannot settle, so every consumer whose
    // operand it was going to write is owed the same answer a supplier that
    // failed at evaluation gives. Asked through the same slots, so a
    // destination that covers no operand concerns nobody.
    let mut failed_suppliers: Vec<Option<Position>> = vec![None; roots.len()];
    for (anchor, output) in unstable {
        for slot in covering_slots(&slots, widest_slot, output.get()) {
            failed_suppliers[slot.consumer].get_or_insert(anchor);
        }
    }

    // A destination this Tick cannot deliver to is taken away rather than
    // executed, and the root keeps its turn without one. It reaches the same
    // settled state as a root that answered with absence, which is what it now
    // is: a value nobody can receive. Only a destination that landed on no
    // parsed slot arrives here, so no consumer was waiting on it and no Data
    // edge is being cut — the rest of the program plays, as it does for any
    // other failure local to one Expression.
    for (producer_index, diagnostic) in withdrawn {
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
    // A rejected Tick still answers for the candidates kept out of the
    // schedule before the graph existed. They are reported first, as they are
    // on the path that publishes: their Expressions were unstable whatever the
    // graph then turned out to be, and dropping them here meant fixing the
    // graph error uncovered a second problem that had been there all along.
    if !diagnostics.is_empty() {
        excluded.extend(diagnostics);
        return Err(excluded);
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
        excluded.push(Diagnostic::for_expression(
            root.anchor,
            root.expression.span(),
            "same-Tick dependency cycle".to_owned(),
        ));
        return Err(excluded);
    }

    Ok(Schedule {
        roots,
        order,
        data_suppliers,
        output_is_slot,
        failed_suppliers,
        excluded,
    })
}

///
/// What is left of `span` once this Tick has cleared the stale Bang display
/// inside it, as the runs a space-partitioned row would then hold.
///
/// A Source-resident `**` is display from an earlier Tick and is blanked
/// before any root takes its turn, so the Cells it occupied are empty by the
/// time a result is written. An Expression that was nothing but Bangs leaves
/// no run at all; one that mixes them with Activations leaves the stretches
/// between them.
///
/// `cleared` is every Bang the starting parse established, in Cell order, and
/// each lies wholly inside exactly one Expression Span. Those Spans ascend
/// too, so the Bangs inside one are a contiguous stretch of `cleared` and are
/// found by search rather than by testing every Bang against every Span.
///
/// Appended to `runs` rather than returned, so one Source revision's runs cost
/// one allocation rather than one per Expression on a path a Tick runs under
/// the playback deadline. Spans ascend, so appending keeps `runs` in Cell
/// order.
///
fn extend_surviving_runs(grid: Grid, span: Span, cleared: &[Span], runs: &mut Vec<Span>) {
    let mut run = |start: usize, end: usize| {
        runs.push(Span::new(
            grid,
            grid.cell_index(start)
                .expect("a Cell of an Expression Span"),
            grid.cell_index(end).expect("a Cell of an Expression Span"),
        ));
    };

    let end = span.end().get();
    let mut start = span.start().get();
    let first = cleared.partition_point(|bang| bang.end().get() < start);
    for bang in cleared[first..]
        .iter()
        .take_while(|bang| bang.start().get() <= end)
    {
        if start < bang.start().get() {
            run(start, bang.start().get() - 1);
        }
        start = bang.end().get() + 1;
    }
    if start <= end {
        run(start, end);
    }
}

///
/// The Expression run a two-Cell output written at `output` would disturb:
/// one it lands inside, or one it is written flush against.
///
/// A row is partitioned at its spaces, so an output changes what the next
/// parse reads by overlapping a run or by abutting one, and it is the run
/// rather than the root that decides this: an Expression holding no Function
/// candidate — a bare Operand Literal, or a candidate the schedule already
/// excluded — declares no operand slot, so nothing else in this walk answers
/// for a result written into it.
///
/// Only an output that landed on no parsed operand slot asks, so an overlap
/// reaching here is an overlap with a run that declared no slot for the Cells
/// being written. A root that declared them answered through `covering_slots`
/// instead, as a Data edge, a structural rejection, or a partial cover.
///
/// The producer's own run is not an answer: a root writing inside its own
/// Expression is the fixed-destination question `covering_slots` owns, and a
/// root cannot join itself. It is recognised by its Span, which is exact: a
/// root's Expression leads with a Function candidate, so no Bang inside it is
/// ever cleared and its surviving run is the whole of it.
///
/// `spans` is every Expression Span of the revision in Cell order and Spans
/// within a row do not overlap, so the runs that can reach a given output are
/// one short stretch of it, found by search rather than by scanning them all
/// on a path a Tick runs under the playback deadline.
///
fn disturbed_expression(
    grid: Grid,
    spans: &[Span],
    producer: Span,
    output: CellIndex,
) -> Option<Span> {
    let output_start = output.get();
    let output_end = output_start + 1;
    let row = grid.position_at(output).y();

    // The first run that ends late enough to touch the output or to sit
    // immediately before it; from there, runs that begin past one Cell after
    // the output cannot reach back to it.
    let first = spans.partition_point(|span| span.end().get() + 1 < output_start);
    spans[first..]
        .iter()
        .copied()
        .take_while(|span| span.start().get() <= output_end + 1)
        .filter(|span| *span != producer)
        // A Span is confined to one row, but the Cell either side of an output
        // at a row edge belongs to the next row or the previous one, so the
        // row is asked rather than assumed.
        .find(|span| grid.position_at(span.start()).y() == row)
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
        // A row's runs end at its spaces, so a write completes an incomplete
        // Function only by abutting the run it is completing. The first
        // missing slot does; the ones past it are separated from the run by
        // Cells this write is not touching, and a result landing there joins
        // nothing and completes nothing. Declaring them anyway made such a
        // result read as a slot write rather than as the activation it is.
        .filter(|(offset, _, _)| *offset <= span_width)
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
    fn a_result_written_where_stale_bang_display_stood_joins_nothing() {
        // The Bang a comparison leaves on the Grid is an Expression of the
        // next revision, and the same comparison writes over it every Tick.
        // The join guard reads the runs this Tick leaves rather than the ones
        // the starting Snapshot spells, so the run that was there is already
        // cleared and the repeated write is the ordinary case it looks like —
        // not a producer joining its own display.
        let grid = Grid::new(16, 3);
        let bytes = snapshot(grid, &[".=0101", "**", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());

        let plan = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

        assert_eq!(planned(&plan), vec![(16, '*'), (17, '*')]);
        assert_eq!(
            plan.diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn a_result_landing_on_a_run_with_no_root_is_refused_like_any_other_join() {
        // The guard exists because a two-Cell result written into a
        // neighbouring run makes the next parse walk a different run, and the
        // display that replaced it is no longer a Bang any cleanup can find.
        // Nothing in that turns on the neighbouring run holding a Function:
        // `0102` is an Expression too, it declares no operand slot for the
        // Cells beside it, and `**0102` is a run whose Bang `bangs()` never
        // yields — so the Bang stays in the Source for every later Tick and
        // the literal beside it is destroyed.
        //
        // Both geometries are the same failure: the abutting write joins the
        // run, and the overlapping write lands inside it.
        let abutting_grid = Grid::new(8, 3);
        let abutting_bytes = snapshot(abutting_grid, &[".=0101", "  0102", ""]);
        let abutting_map = LanguageMap::build(abutting_grid, abutting_bytes.as_bytes());

        let abutting = super::plan(
            abutting_grid,
            abutting_bytes.as_bytes(),
            &abutting_map,
            Tick::ZERO,
        );

        assert_eq!(planned(&abutting), vec![]);
        assert!(
            abutting.diagnostics.iter().any(|diagnostic| {
                diagnostic.message
                    == "current-Tick output would join the Expression at column 2, row 1"
            }),
            "the abutting write went unreported: {:?}",
            abutting.diagnostics
        );

        let overlapping_grid = Grid::new(10, 3);
        let overlapping_bytes = snapshot(overlapping_grid, &["    .=0101", "  0102", ""]);
        let overlapping_map = LanguageMap::build(overlapping_grid, overlapping_bytes.as_bytes());

        let overlapping = super::plan(
            overlapping_grid,
            overlapping_bytes.as_bytes(),
            &overlapping_map,
            Tick::ZERO,
        );

        assert_eq!(planned(&overlapping), vec![]);
        assert!(
            overlapping.diagnostics.iter().any(|diagnostic| {
                diagnostic.message
                    == "current-Tick output would join the Expression at column 2, row 1"
            }),
            "the overlapping write went unreported: {:?}",
            overlapping.diagnostics
        );
    }

    #[test]
    fn a_destination_at_the_row_edge_costs_one_expression_its_turn_not_the_tick() {
        // `root_layout` routes "Expression layout crosses the row edge" into
        // the excluded candidates so every other root still plays. A
        // destination with one Cell left in its row is the same shape of
        // failure — one Expression's anchor, not the graph — and used to
        // reject the whole Tick instead.
        //
        // `Portal::ordinary_result` cannot reach the last column today: a
        // Function spelling is two Cells, so a root anchor is at most `cols-2`
        // and so is the destination below it. The destination override is the
        // entry point that can, and ADR 0009 expects destination resolution to
        // change.
        let grid = Grid::new(16, 4);
        let bytes = snapshot(grid, &[".+0102", "", ".+0304", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let destinations = [(
            grid.cell_index(0).unwrap(),
            grid.position(15, 1).expect("inside the Grid"),
        )]
        .into_iter()
        .collect();

        let plan =
            super::plan_with_destinations(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        assert_eq!(
            planned(&plan),
            vec![(48, '0'), (49, '7')],
            "the other root still plays: {:?}",
            plan.diagnostics
        );
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "a scalar output crosses the row edge"),
            "the Expression at the row edge keeps its own diagnostic: {:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn a_withdrawn_destination_holds_no_cell_against_the_producer_that_keeps_one() {
        // A producer the join guard removes writes nothing, so the Cells it
        // asked for are free. Registering them before that verdict made a
        // later producer collide with a writer that no longer exists, and one
        // Expression's local failure rejected the whole Tick.
        //
        // The two destinations share exactly one Cell. The first abuts the
        // literal run beside it and loses its destination; the second is one
        // Cell further on, abuts nothing, and is the only writer left.
        let grid = Grid::new(16, 3);
        let bytes = snapshot(grid, &[".+0102 .+0304", "0102", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let destinations = [
            (
                grid.cell_index(0).unwrap(),
                grid.position(4, 1).expect("inside the Grid"),
            ),
            (
                grid.cell_index(7).unwrap(),
                grid.position(5, 1).expect("inside the Grid"),
            ),
        ]
        .into_iter()
        .collect();

        let plan =
            super::plan_with_destinations(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        assert_eq!(
            planned(&plan),
            vec![(21, '0'), (22, '7')],
            "the surviving producer still writes: {:?}",
            plan.diagnostics
        );
        assert!(
            plan.diagnostics.iter().any(|diagnostic| {
                diagnostic.message
                    == "current-Tick output would join the Expression at column 0, row 1"
            }),
            "the withdrawn producer keeps its own diagnostic: {:?}",
            plan.diagnostics
        );
        assert!(
            !plan.diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("multiple current-Tick producers")),
            "nothing collided with a writer that no longer exists: {:?}",
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
    fn a_rejected_tick_still_reports_the_candidates_it_excluded() {
        // A graph error rejects the Tick, but it does not answer for the
        // Expressions that were kept out of the schedule before the graph was
        // even built. Dropping their diagnostics meant fixing the reported
        // problem uncovered a second one that had been there all along.
        let grid = Grid::new(16, 3);
        let bytes = snapshot(grid, &[".+0102", ".+0304", ".+0102Z"]);
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let shared = grid.position(10, 1).unwrap();
        let destinations = [
            (grid.cell_index(0).unwrap(), shared),
            (grid.cell_index(16).unwrap(), shared),
        ]
        .into_iter()
        .collect();

        let plan =
            super::plan_with_destinations(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        assert!(plan.writes.is_empty());
        assert!(
            plan.diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("multiple current-Tick producers")),
            "{:?}",
            plan.diagnostics
        );
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("structurally unstable")),
            "the excluded candidate went unreported: {:?}",
            plan.diagnostics
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
        // One diagnostic per producer that collided, not one per Cell of the
        // destination they collided over: a two-Cell output means the same
        // pair of producers meets twice, and saying so twice describes two
        // conflicts where the Source has one.
        assert_eq!(
            conflict
                .diagnostics
                .iter()
                .filter(|diagnostic| {
                    diagnostic
                        .message
                        .contains("multiple current-Tick producers")
                })
                .count(),
            1
        );

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
