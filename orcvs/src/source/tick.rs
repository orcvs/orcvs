//! Tick-local execution of Parser-owned expressions (ADR 0034).
//!
//! Fixed Portal destinations and nested ownership determine the complete order
//! before execution. Spatial writes remain character encodings until consumed;
//! nested results are typed values. Only the final effects are published.

use lang::{Anchor, Atom, Function, Interpretation, Interpreter, Tick, TickInputs, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use super::language_map::{LanguageMap, Span};
use super::portal::{Portal, PortalError, SpanWrite};
use super::{CellContent, CellWrite, Diagnostic, Performance, TickPlan};
use crate::grid::{CellIndex, Grid, Position};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Effect {
    Write(SpanWrite),
    Play(Performance),
    Diagnose(Diagnostic),
}

struct Operand {
    cells: Range<usize>,
    child: Option<usize>,
}

struct Computation {
    anchor: Position,
    span: Span,
    function: Function,
    parent: Option<usize>,
    owner: usize,
    operands: Vec<Operand>,
    syntax_valid: bool,
    outputs: Vec<Result<Position, PortalError>>,
}

#[derive(Default)]
pub(super) struct Configuration {
    destinations: BTreeMap<CellIndex, Vec<Position>>,
    #[cfg(test)]
    supplied: BTreeMap<CellIndex, Atom>,
}

struct Schedule {
    nodes: Vec<Computation>,
    lookup: Lookup,
    order: Vec<usize>,
    diagnostics: Vec<Diagnostic>,
}

struct Claim {
    cells: Range<usize>,
    node: usize,
}

/// Physical claims are disjoint; enclosing expression spans are not indexed.
/// Empty operands at a Source boundary claim no Cells and must not participate
/// in the binary search. Global Cell indices also distinguish adjacent rows.
struct Claims(Vec<Claim>);

impl Claims {
    fn new(mut claims: Vec<Claim>) -> Self {
        claims.retain(|claim| !claim.cells.is_empty());
        claims.sort_unstable_by_key(|claim| claim.cells.start);
        debug_assert!(
            claims
                .windows(2)
                .all(|pair| pair[0].cells.end <= pair[1].cells.start)
        );
        Self(claims)
    }

    fn touching(&self, cells: Range<usize>) -> impl Iterator<Item = usize> + '_ {
        let first = self
            .0
            .partition_point(|claim| claim.cells.end <= cells.start);
        self.0[first..]
            .iter()
            .take_while(move |claim| claim.cells.start < cells.end)
            .map(|claim| claim.node)
    }
}

struct Lookup {
    functions: Claims,
    literals: Claims,
    operands: Claims,
    subtree_ends: Vec<usize>,
}

impl Lookup {
    fn new(grid: Grid, nodes: &[Computation]) -> Self {
        let mut functions = Vec::new();
        let mut literals = Vec::new();
        let mut operands = Vec::new();
        let mut subtree_ends: Vec<_> = (1..=nodes.len()).collect();
        for (index, node) in nodes.iter().enumerate() {
            let start = grid.index(node.anchor).get();
            functions.push(Claim {
                cells: start..start + 2,
                node: index,
            });
            for operand in &node.operands {
                operands.push(Claim {
                    cells: operand.cells.clone(),
                    node: index,
                });
                if operand.child.is_none() {
                    literals.push(Claim {
                        cells: operand.cells.clone(),
                        node: index,
                    });
                }
            }
        }
        // Nodes retain the Parser's depth-first preorder. Every subtree is a
        // contiguous range, including its root, even with missing operands.
        for index in (0..nodes.len()).rev() {
            if let Some(parent) = nodes[index].parent {
                subtree_ends[parent] = subtree_ends[parent].max(subtree_ends[index]);
            }
        }
        Self {
            functions: Claims::new(functions),
            literals: Claims::new(literals),
            operands: Claims::new(operands),
            subtree_ends,
        }
    }

    fn descendants(&self, ancestor: usize) -> Range<usize> {
        ancestor..self.subtree_ends[ancestor]
    }

    fn root_at(&self, grid: Grid, nodes: &[Computation], anchor: Position) -> Option<usize> {
        let cell = grid.index(anchor).get();
        self.functions
            .touching(cell..cell + 1)
            .find(|&index| nodes[index].parent.is_none() && nodes[index].anchor == anchor)
    }

    fn is_operand_destination(&self, grid: Grid, output: Position) -> bool {
        let start = grid.index(output).get();
        self.operands.touching(start..start + 2).next().is_some()
    }
}

pub(super) fn plan(grid: Grid, bytes: &[u8], map: &LanguageMap, tick: Tick) -> TickPlan {
    plan_configured(grid, bytes, map, tick, &Configuration::default())
}

#[cfg(test)]
fn plan_with_destinations(
    grid: Grid,
    bytes: &[u8],
    map: &LanguageMap,
    tick: Tick,
    destinations: &BTreeMap<CellIndex, Position>,
) -> TickPlan {
    plan_configured(
        grid,
        bytes,
        map,
        tick,
        &Configuration {
            destinations: destinations
                .iter()
                .map(|(anchor, output)| (*anchor, vec![*output]))
                .collect(),
            ..Configuration::default()
        },
    )
}

fn diagnose(node: &Computation, message: impl Into<String>) -> Diagnostic {
    Diagnostic::for_expression(node.anchor, node.span, message.into())
}

pub(super) fn plan_configured(
    grid: Grid,
    bytes: &[u8],
    map: &LanguageMap,
    tick: Tick,
    configuration: &Configuration,
) -> TickPlan {
    let schedule = match schedule(grid, map, configuration) {
        Ok(schedule) => schedule,
        Err(diagnostics) => {
            return TickPlan {
                writes: vec![],
                play_commands: vec![],
                diagnostics,
            };
        }
    };
    let mut working = bytes.to_vec();
    let mut effects: Vec<_> = schedule
        .diagnostics
        .into_iter()
        .map(Effect::Diagnose)
        .collect();
    for (anchor, _) in map.bangs() {
        let clear = Portal::at(grid, anchor)
            .admit("  ")
            .expect("parsed Bang fits its Grid");
        apply_write(&mut working, &clear);
        effects.push(Effect::Write(clear));
    }
    let nodes = &schedule.nodes;
    let lookup = &schedule.lookup;
    let mut results: Vec<Option<Value>> = vec![None; nodes.len()];
    let mut syntax_blocked = vec![false; nodes.len()];
    let mut activated = vec![false; nodes.len()];
    let mut suppressed = vec![false; nodes.len()];
    let mut executed = vec![false; nodes.len()];
    let mut functions: Vec<_> = nodes.iter().map(|node| node.function).collect();
    for index in schedule.order {
        let node = &nodes[index];
        // A root that answers an effect performs only once it is activated, so
        // the gate asks the owner's declared kind. It asks whether that kind
        // answers a value and not which effect it performs: the two questions
        // coincide only while Terminal Output is the one effect declared, and
        // ADR 0029 records that a Function declared with any other effect must
        // reach the same gate by its definition alone.
        if suppressed[index]
            || (!nodes[node.owner].function.answers_value() && !activated[node.owner])
        {
            continue;
        }
        executed[index] = true;
        if node.parent.is_some() && !node.function.answers_value() {
            effects.push(Effect::Diagnose(diagnose(
                node,
                lang::InterpretationError::NestedEffectFunction.to_string(),
            )));
            continue;
        }
        let function = functions[index];
        if !node.syntax_valid
            && function == node.function
            && node
                .operands
                .iter()
                .all(|operand| working[operand.cells.clone()] == bytes[operand.cells.clone()])
        {
            // Unchanged initial syntax errors belong to the Source revision.
            // Earlier writes can repair these inputs before their reserved turn.
            syntax_blocked[index] = true;
            continue;
        }
        if node.operands.iter().any(|operand| {
            operand
                .child
                .is_some_and(|child| !suppressed[child] && syntax_blocked[child])
        }) {
            // A syntax-blocked child did not fail evaluation. Preserve its
            // Source diagnostic without turning it into a repeated Tick error.
            syntax_blocked[index] = true;
            continue;
        }
        let signature = lang::Tokens::from(&function);
        if signature.len() != node.operands.len() {
            effects.push(Effect::Diagnose(diagnose(
                node,
                lang::ArgumentError::Arity {
                    expected: signature.len(),
                    found: node.operands.len(),
                }
                .to_string(),
            )));
            continue;
        }
        let operands: Result<Vec<Value>, String> = node
            .operands
            .iter()
            .zip(signature)
            .map(|(operand, token)| {
                if let Some(child) = operand.child.filter(|child| !suppressed[*child]) {
                    return results[child].clone().ok_or_else(|| {
                        format!(
                            "nested computation at column {}, row {} supplied no typed result",
                            nodes[child].anchor.x(),
                            nodes[child].anchor.y()
                        )
                    });
                }
                let spelling =
                    std::str::from_utf8(&working[operand.cells.clone()]).expect("ASCII Source");
                token
                    .decode(spelling)
                    .map(Value::from)
                    .map_err(|error| error.to_string())
            })
            .collect();
        let result = match operands {
            Ok(operands) => interpret(function, &operands, tick_inputs(tick, node.anchor))
                .map_err(|error| error.to_string()),
            Err(error) => Err(error),
        };
        #[cfg(test)]
        let result = configuration
            .supplied
            .get(&grid.index(node.anchor))
            .map_or(result, |atom| Ok(Interpretation::Cell(*atom)));
        match result {
            Err(message) => effects.push(Effect::Diagnose(diagnose(node, message))),
            Ok(Interpretation::Play(performance)) => effects.push(Effect::Play(performance)),
            Ok(answer) => {
                let value = match answer {
                    Interpretation::Cell(atom) => Value::Atom(atom),
                    Interpretation::Sequence(sequence) => Value::Sequence(sequence),
                    Interpretation::Play(_) => unreachable!(),
                };
                results[index] = Some(value.clone());
                let encoding = match &value {
                    Value::Atom(Atom::Empty) => continue,
                    Value::Atom(atom) => atom.to_string(),
                    Value::Sequence(sequence) if sequence.is_empty() => continue,
                    Value::Sequence(sequence) => {
                        if !node.outputs.is_empty() {
                            effects.push(Effect::Diagnose(diagnose(
                                node,
                                format!(
                                    "Sequence result {:?} has no fixed scalar scheduling footprint",
                                    sequence.to_string()
                                ),
                            )));
                        }
                        continue;
                    }
                };
                // Scheduling reserves one scalar Cell pair per destination.
                // A different width cannot safely use those dependency edges.
                if encoding.len() != 2 {
                    if !node.outputs.is_empty() {
                        effects.push(Effect::Diagnose(diagnose(
                            node,
                            "result is not a scalar Cell pair",
                        )));
                    }
                    continue;
                }
                for output in &node.outputs {
                    let write =
                        match output.and_then(|output| Portal::at(grid, output).admit(&encoding)) {
                            Ok(write) => write,
                            Err(reason) => {
                                effects.push(Effect::Diagnose(diagnose(
                                    node,
                                    portal_message(reason, &encoding),
                                )));
                                continue;
                            }
                        };
                    let output = output.expect("an admitted write has a destination");
                    if value == Value::Atom(Atom::Bang)
                        && !lookup.is_operand_destination(grid, output)
                    {
                        for anchor in activated_anchors(grid, output).into_iter().flatten() {
                            if let Some(owner) = lookup.root_at(grid, nodes, anchor) {
                                activated[owner] = true;
                            }
                        }
                    }
                    let start = grid.index(output).get();
                    if let Value::Atom(Atom::Function(replacement)) = value
                        && lookup.functions.touching(start..start + 1).any(|target| {
                            let target = &nodes[target];
                            grid.index(target.anchor).get() == start
                                && (replacement.answers_value() != target.function.answers_value()
                                    || replacement.can_emit_bang()
                                        != target.function.can_emit_bang())
                        })
                    {
                        effects.push(Effect::Diagnose(diagnose(
                            node,
                            "Function replacement changes activation requirements or output kind",
                        )));
                        continue;
                    }
                    // A schedule defect must not panic under the Source lock
                    // or publish any of this Tick's already accumulated effects.
                    if lookup
                        .functions
                        .touching(start..start + encoding.len())
                        .any(|target| {
                            lookup
                                .descendants(target)
                                .any(|descendant| executed[descendant])
                        })
                    {
                        let mut diagnostics: Vec<_> = effects
                            .into_iter()
                            .filter_map(|effect| {
                                if let Effect::Diagnose(diagnostic) = effect {
                                    Some(diagnostic)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        diagnostics.push(diagnose(
                            node,
                            "spatial output reached an executed computation; Tick effects rejected",
                        ));
                        return TickPlan {
                            writes: vec![],
                            play_commands: vec![],
                            diagnostics,
                        };
                    }
                    for target in lookup.functions.touching(start..start + encoding.len()) {
                        let anchor = grid.index(nodes[target].anchor).get();
                        if start == anchor
                            && !suppressed[target]
                            && let Value::Atom(Atom::Function(replacement)) = value
                        {
                            functions[target] = replacement;
                            continue;
                        }
                        for descendant in lookup.descendants(target) {
                            suppressed[descendant] = true;
                        }
                    }
                    apply_write(&mut working, &write);
                    effects.push(Effect::Write(write));
                }
            }
        }
    }
    resolve(effects)
}

/// An inactive root can contribute no child Portal. Start from value roots,
/// then close over potential Bang deliveries; actual activation is still
/// checked during execution, after those producers have settled.
fn potentially_active(grid: Grid, nodes: &[Computation], lookup: &Lookup) -> Vec<bool> {
    let mut active: Vec<_> = nodes
        .iter()
        .map(|node| node.parent.is_none() && node.function.answers_value())
        .collect();
    let mut pending: Vec<_> = active
        .iter()
        .enumerate()
        .filter_map(|(index, active)| active.then_some(index))
        .collect();
    while let Some(owner) = pending.pop() {
        for index in lookup.descendants(owner) {
            let node = &nodes[index];
            if !node.function.can_emit_bang() {
                continue;
            }
            for output in node
                .outputs
                .iter()
                .filter_map(|output| output.as_ref().ok())
            {
                if grid.offset_in_row(*output, 1).is_none()
                    || lookup.is_operand_destination(grid, *output)
                {
                    continue;
                }
                for anchor in activated_anchors(grid, *output).into_iter().flatten() {
                    if let Some(index) = lookup.root_at(grid, nodes, anchor)
                        && !nodes[index].function.answers_value()
                        && !active[index]
                    {
                        active[index] = true;
                        pending.push(index);
                    }
                }
            }
        }
    }
    active
}

fn schedule(
    grid: Grid,
    map: &LanguageMap,
    configuration: &Configuration,
) -> Result<Schedule, Vec<Diagnostic>> {
    let mut nodes: Vec<Computation> = Vec::new();
    let mut diagnostics = Vec::new();
    for expression in map.expressions() {
        if expression.function_candidate().is_none() {
            continue;
        }
        let mut functions = BTreeMap::new();
        for (entry_index, entry) in expression.positioned().enumerate() {
            let parent = entry
                .parent
                .and_then(|parent| functions.get(&parent).copied());
            let child = if let Some(Atom::Function(function)) = entry.atom {
                let index = nodes.len();
                let anchor = grid.position_at(
                    grid.cell_index(entry.cells.start)
                        .expect("parsed Function inside Grid"),
                );
                let owner = parent.map_or(index, |parent: usize| nodes[parent].owner);
                let configured = configuration.destinations.get(&grid.index(anchor));
                let outputs = if !function.answers_value() {
                    if configured.is_some() {
                        diagnostics.push(Diagnostic::for_expression(
                            anchor,
                            expression.span(),
                            "a Function that answers an effect cannot have a Portal".to_owned(),
                        ));
                    }
                    vec![]
                } else if let Some(outputs) = configured {
                    outputs
                        .iter()
                        .map(|output| {
                            grid.assert_owns(*output);
                            Ok(*output)
                        })
                        .collect()
                } else if parent.is_none() {
                    vec![Portal::ordinary_result(grid, anchor).map(|portal| portal.destination())]
                } else {
                    vec![]
                };
                nodes.push(Computation {
                    anchor,
                    span: expression.span(),
                    function,
                    parent,
                    owner,
                    operands: vec![],
                    syntax_valid: true,
                    outputs,
                });
                functions.insert(entry_index, index);
                Some(index)
            } else {
                None
            };
            if let Some(parent) = parent {
                nodes[parent].syntax_valid &= entry.atom.is_some();
                nodes[parent].operands.push(Operand {
                    cells: entry.cells.clone(),
                    child,
                });
            }
        }
    }
    for node in &nodes {
        if node
            .operands
            .iter()
            .zip(lang::Tokens::from(&node.function))
            .any(|(operand, token)| operand.child.is_none() && operand.cells.len() < token.len())
        {
            let boundary = if node.span.end().get() % grid.cols() == grid.cols() - 1 {
                "Expression layout crosses the row edge"
            } else {
                "Expression operand crosses the Source boundary"
            };
            diagnostics.push(diagnose(node, boundary));
        }
    }
    let lookup = Lookup::new(grid, &nodes);
    let active = potentially_active(grid, &nodes, &lookup);
    let mut edges = BTreeSet::new();
    for (index, node) in nodes.iter().enumerate() {
        if let Some(parent) = node.parent {
            edges.insert((index, parent));
        }
        if !active[node.owner] {
            continue;
        }
        for output in node
            .outputs
            .iter()
            .filter_map(|output| output.as_ref().ok())
        {
            if grid.offset_in_row(*output, 1).is_none() {
                continue;
            }
            let start = grid.index(*output).get();
            let cells = start..start + 2;
            for consumer in lookup.functions.touching(cells.clone()) {
                for descendant in lookup.descendants(consumer) {
                    edges.insert((index, descendant));
                }
            }
            for consumer in lookup.literals.touching(cells) {
                edges.insert((index, consumer));
            }
            if node.function.can_emit_bang() && !lookup.is_operand_destination(grid, *output) {
                for anchor in activated_anchors(grid, *output).into_iter().flatten() {
                    if let Some(owner) = lookup.root_at(grid, &nodes, anchor)
                        && !nodes[owner].function.answers_value()
                    {
                        for consumer in lookup.descendants(owner) {
                            edges.insert((index, consumer));
                        }
                    }
                }
            }
        }
    }
    let mut indegree = vec![0; nodes.len()];
    let mut outgoing = vec![vec![]; nodes.len()];
    for (producer, consumer) in edges {
        indegree[consumer] += 1;
        outgoing[producer].push(consumer);
    }
    let mut ready: BTreeSet<_> = indegree
        .iter()
        .enumerate()
        .filter(|(_, incoming)| **incoming == 0)
        .map(|(index, _)| (grid.index(nodes[index].anchor), index))
        .collect();
    let mut order = Vec::new();
    while let Some((_, index)) = ready.pop_first() {
        order.push(index);
        for &consumer in &outgoing[index] {
            indegree[consumer] -= 1;
            if indegree[consumer] == 0 {
                ready.insert((grid.index(nodes[consumer].anchor), consumer));
            }
        }
    }
    if order.len() != nodes.len() {
        let index = indegree
            .iter()
            .position(|incoming| *incoming != 0)
            .expect("cycle has a node");
        diagnostics.push(diagnose(&nodes[index], "same-Tick dependency cycle"));
        return Err(diagnostics);
    }
    Ok(Schedule {
        nodes,
        lookup,
        order,
        diagnostics,
    })
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

/// Calls the existing Evaluator once with resolved typed operands. The
/// test-only observation records exactly-once execution and Tick/anchor inputs.
fn interpret(
    function: Function,
    operands: &[Value],
    inputs: TickInputs,
) -> Result<Interpretation, lang::Error> {
    #[cfg(test)]
    observed::record(inputs);
    Interpreter::execute_function(function, operands, inputs)
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

    fn configured_source(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
        supplied: &[(usize, lang::Function)],
    ) -> (TickPlan, crate::source::Source) {
        configured_source_atoms(
            grid,
            rows,
            outputs,
            &supplied
                .iter()
                .map(|(anchor, function)| (*anchor, lang::Atom::Function(*function)))
                .collect::<Vec<_>>(),
        )
    }

    fn configured_source_atoms(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
        supplied: &[(usize, lang::Atom)],
    ) -> (TickPlan, crate::source::Source) {
        let mut source = crate::source::Source::new(grid);
        for (index, byte) in snapshot(grid, rows).bytes().enumerate() {
            source
                .set(cell(grid, index), &char::from(byte).to_string())
                .unwrap();
        }
        let mut configuration = super::Configuration::default();
        for (anchor, output) in outputs {
            configuration
                .destinations
                .entry(cell(grid, *anchor))
                .or_default()
                .push(grid.position_at(cell(grid, *output)));
        }
        configuration.supplied.extend(
            supplied
                .iter()
                .map(|(anchor, function)| (cell(grid, *anchor), *function)),
        );
        let plan = source.execute_configured(Tick::ZERO, &configuration);
        (plan, source)
    }

    #[test]
    fn live_deep_sibling_computations_preserve_operand_order() {
        // Each sibling requires more pending operands than the Parser keeps
        // inline. The second refills that stack after the first has drained it.
        let numerator = ".+".repeat(32) + &"02".repeat(33);
        let denominator = ".+".repeat(32) + &"01".repeat(33);
        let text = format!("./{numerator}{denominator}");
        let width = text.len();
        let (plan, source) = configured_source(Grid::new(width, 2), &[&text, ""], &[], &[]);
        // 66 / 33 = 2. Reversing the siblings instead produces zero.
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert!(plan.play_commands.is_empty());
        assert_eq!(source.snapshot(), snapshot(source.grid(), &[&text, "02"]));
    }

    #[test]
    fn live_unchanged_nested_syntax_errors_do_not_repeat_as_tick_failures() {
        let grid = Grid::new(20, 2);
        let rows = [".+01.x02.+03??", ""];
        let (plan, mut source) = configured_source(grid, &rows, &[], &[]);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert!(
            !source
                .language_map()
                .expression_diagnostics()
                .collect::<Vec<_>>()
                .is_empty()
        );
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
        let repeated = source.execute(Tick::new(1));
        assert!(
            repeated.diagnostics.is_empty(),
            "{:?}",
            repeated.diagnostics
        );
        assert!(repeated.play_commands.is_empty());
        source.set(cell(grid, 12), "0").unwrap();
        source.set(cell(grid, 13), "4").unwrap();
        let repaired = source.execute(Tick::new(2));
        assert!(
            repaired.diagnostics.is_empty(),
            "{:?}",
            repaired.diagnostics
        );
        assert_eq!(&source.snapshot()[20..22], "0F");

        // A writer can also repair the bad leaf before its reserved turn.
        let (repaired, source) = configured_source(
            Grid::new(20, 3),
            &[".+01.x02.+03??", ".+0004", ""],
            &[(0, 40), (20, 12)],
            &[],
        );
        assert!(
            repaired.diagnostics.is_empty(),
            "{:?}",
            repaired.diagnostics
        );
        assert_eq!(&source.snapshot()[40..42], "0F");
    }

    #[test]
    fn live_non_pair_scalar_projection_is_rejected_at_the_row_edge() {
        let grid = Grid::new(16, 2);
        let rows = [".+0203", ""];
        let (plan, source) =
            configured_source_atoms(grid, &rows, &[(0, 31)], &[(0, lang::Atom::Char('7'))]);
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
        assert!(plan.writes.is_empty());
        assert!(plan.play_commands.is_empty());
        assert!(
            plan.diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("result is not a scalar Cell pair")),
            "{:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn live_inactive_nested_portal_cannot_create_a_cycle() {
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &["!>007F.^3C", ".+0203", ""],
            &[(6, 8), (16, 32)],
            &[],
        );
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(&source.snapshot()[32..34], "05");
        assert!(plan.play_commands.is_empty());
    }

    #[test]
    fn live_claims_and_glyphs_survive_source_edits_and_publication() {
        let (plan, source) = configured_source(Grid::new(16, 2), &[".+01 02", ""], &[], &[]);
        assert!(plan.writes.is_empty());
        assert_eq!(
            source
                .language_map()
                .expressions()
                .next()
                .unwrap()
                .span()
                .end()
                .get(),
            5
        );
        assert!(source.language_map().diagnostics().any(|d| d.start() == 6));
        assert_eq!(
            source
                .language_map()
                .glyph_at(source.grid().position(4, 0).unwrap()),
            Some(crate::glyph::Glyph::Number)
        );
        let (plan, source) = configured_source(Grid::new(16, 2), &[".+0102.+0304", ""], &[], &[]);
        assert_eq!(&source.snapshot()[16..24], "03    07");
        assert!(plan.diagnostics.is_empty());
        assert_eq!(
            source
                .language_map()
                .expressions()
                .filter_map(|entry| entry.root())
                .filter(|root| root.y() == 0)
                .count(),
            2
        );
        let (plan, source) = configured_source(Grid::new(16, 2), &[".+0102Z", ""], &[], &[]);
        assert_eq!(&source.snapshot()[16..18], "03");
        assert!(plan.diagnostics.is_empty());
        assert!(source.language_map().diagnostics().any(|d| d.start() == 6));
        let (plan, source) = configured_source(Grid::new(16, 2), &["***", ""], &[], &[]);
        assert_eq!(&source.snapshot()[..3], "  *");
        assert_eq!(plan.writes.len(), 2);
        assert!(source.language_map().diagnostics().any(|d| d.start() == 2));
        let (plan, source) =
            configured_source(Grid::new(16, 2), &[".=0101 !>007FC4", ""], &[], &[]);
        assert_eq!(&source.snapshot()[16..18], "**");
        assert!(plan.play_commands.is_empty());
        assert!(plan.diagnostics.is_empty());
        assert_eq!(
            source
                .language_map()
                .expressions()
                .filter_map(|entry| entry.root())
                .filter(|root| root.y() == 0)
                .count(),
            2
        );
        // Pin the current display of plausible standalone data. These rejected
        // Function candidates have Function glyphs, no units and no execution.
        let (plan, source) = configured_source(Grid::new(16, 2), &["C4 EA 01", ""], &[], &[]);
        assert!(plan.writes.is_empty());
        assert_eq!(source.language_map().units().count(), 0);
        for column in [0, 1, 3, 4, 6, 7] {
            assert_eq!(
                source
                    .language_map()
                    .glyph_at(source.grid().position(column, 0).unwrap()),
                Some(crate::glyph::Glyph::Function)
            );
        }
    }

    #[test]
    fn live_bang_in_half_typed_terminal_claim_diagnoses_without_activation() {
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &["    .=0101", "!>00", "    !>007FC4"],
            &[],
            &[],
        );
        assert!(plan.play_commands.is_empty());
        assert_eq!(&source.snapshot()[16..24], "!>00**  ");
        assert!(
            source
                .language_map()
                .diagnostics()
                .any(|d| d.start() == 16 && d.message.contains("expected a number"))
        );
        assert_eq!(
            source
                .language_map()
                .glyph_at(source.grid().position(4, 1).unwrap()),
            Some(crate::glyph::Glyph::Number)
        );
        assert_eq!(source.language_map().bangs().count(), 0);
    }

    #[test]
    fn live_function_replacement_cannot_change_activation_or_output_kind() {
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0204", ".+0000", ""],
            &[(0, 32), (16, 0)],
            &[(16, lang::Function::RawPlay)],
        );
        assert_eq!(&source.snapshot()[..6], ".+0204");
        assert_eq!(&source.snapshot()[32..34], "06");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("activation requirements or output kind"))
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    proptest::proptest! {
        #[test]
        fn live_writes_agree_with_an_independent_cell_overlay(
            first in proptest::prelude::any::<u8>(), second in proptest::prelude::any::<u8>(),
            first_column in 2usize..6, second_column in 2usize..6,
        ) {
            let grid = Grid::new(16, 4);
            let first_source = format!(".+00{first:02X}");
            let second_source = format!(".+00{second:02X}");
            let rows = [".+0101", first_source.as_str(), second_source.as_str(), ""];
            let outputs = [(0, 48), (16, first_column), (32, second_column)];
            let mut expected = snapshot(grid, &rows).into_bytes();
            // Independent oracle: two literal Cell overlays in known Position
            // order, followed by ordinary hexadecimal arithmetic.
            expected[first_column..first_column + 2].copy_from_slice(format!("{first:02X}").as_bytes());
            expected[second_column..second_column + 2].copy_from_slice(format!("{second:02X}").as_bytes());
            let left = u16::from_str_radix(std::str::from_utf8(&expected[2..4]).unwrap(), 16).unwrap();
            let right = u16::from_str_radix(std::str::from_utf8(&expected[4..6]).unwrap(), 16).unwrap();
            expected[48..50].copy_from_slice(format!("{:02X}", (left + right) % 256).as_bytes());
            let (plan, source) = configured_source(grid, &rows, &outputs, &[]);
            proptest::prop_assert_eq!(source.snapshot().into_bytes(), expected);
            proptest::prop_assert!(plan.diagnostics.is_empty());
            let (repeated, _) = configured_source(grid, &rows, &outputs, &[]);
            proptest::prop_assert_eq!(plan, repeated);
            let full = LanguageMap::derive(grid, &source.snapshot()).unwrap();
            proptest::prop_assert_eq!(source.language_map().units().collect::<Vec<_>>(), full.units().collect::<Vec<_>>());
            for position in grid.rows().flatten() {
                proptest::prop_assert_eq!(source.language_map().glyph_at(position), full.glyph_at(position));
            }
        }

        #[test]
        fn live_rejected_complete_write_changes_no_cell(value in proptest::prelude::any::<u8>()) {
            let grid = Grid::new(16, 2);
            let producer = format!(".+00{value:02X}");
            let rows = [producer.as_str(), ""];
            let (plan, source) = configured_source(grid, &rows, &[(0, 31)], &[]);
            proptest::prop_assert!(plan.writes.is_empty());
            proptest::prop_assert_eq!(source.snapshot(), snapshot(grid, &rows));
            proptest::prop_assert!(plan.diagnostics.iter().any(|d| d.message.contains("crosses the row edge")));
        }
    }
    #[test]
    fn live_cross_boundary_chain_uses_lower_producers() {
        let (plan, source) = configured_source(
            Grid::new(16, 5),
            &[".+0101", ".+0001", ".+0001", "", ""],
            &[(0, 64), (16, 34), (32, 3)],
            &[],
        );
        assert_eq!(&source.snapshot()[..6], ".+0021");
        assert_eq!(&source.snapshot()[64..66], "21");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    #[test]
    fn live_competing_writers_follow_position_and_emissions_follow_configuration() {
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &[".+0101", ".+0101", ".+0102", ""],
            &[(0, 48), (16, 3), (32, 3)],
            &[],
        );
        assert_eq!(&source.snapshot()[..6], ".+0031");
        assert_eq!(&source.snapshot()[48..50], "31");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        observed::take();
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0101", ".+0203", ""],
            &[(0, 32), (16, 2), (16, 3)],
            &[],
        );
        assert_eq!(&source.snapshot()[..6], ".+0051");
        assert_eq!(&source.snapshot()[32..34], "51");
        assert_eq!(observed::take().len(), 2);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    #[test]
    fn live_pending_note_decodes_only_after_all_writers_settle() {
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &[".vE4", ".+E901", ".+E401", ""],
            &[(0, 48), (16, 2), (32, 2)],
            &[],
        );
        assert_eq!(&source.snapshot()[..4], ".vE5");
        assert_eq!(&source.snapshot()[48..50], "4C");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".vE4", ".+E901", ""],
            &[(0, 32), (16, 2)],
            &[],
        );
        assert_eq!(&source.snapshot()[..4], ".vEA");
        assert_eq!(&source.snapshot()[32..34], "  ");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("expected a note")),
            "{:?}",
            plan.diagnostics
        );
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0000", ".+E901", ""],
            &[(0, 32), (16, 2)],
            &[],
        );
        assert_eq!(&source.snapshot()[32..34], "EA");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    #[test]
    fn live_spatial_note_is_an_encoding_and_nested_note_stays_typed() {
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0001", ".^48", ""],
            &[(0, 32), (16, 2)],
            &[],
        );
        assert_eq!(&source.snapshot()[..6], ".+C501");
        assert_eq!(&source.snapshot()[32..34], "C6");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        let (plan, source) =
            configured_source(Grid::new(16, 2), &[".+.^4801", ""], &[(0, 16)], &[]);
        assert_eq!(&source.snapshot()[16..18], "  ");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("expected a number")),
            "{:?}",
            plan.diagnostics
        );
        assert_eq!(&source.snapshot()[..8], ".+.^4801");
    }

    #[test]
    fn live_child_write_survives_parent_failure_and_rejected_portal_keeps_typed_answer() {
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &["./.x030400", ".+0001", "", ""],
            &[(0, 48), (2, 18), (16, 52)],
            &[],
        );
        assert_eq!(&source.snapshot()[18..20], "0C");
        assert_eq!(&source.snapshot()[48..50], "  ");
        assert_eq!(&source.snapshot()[52..54], "0D");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message == "cannot divide by zero")
        );
        let (plan, source) = configured_source(
            Grid::new(16, 2),
            &[".+02.x0304", ""],
            &[(0, 16), (4, 31)],
            &[],
        );
        assert_eq!(&source.snapshot()[16..18], "0E");
        assert_eq!(&source.snapshot()[31..], " ");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("crosses the row edge"))
        );
    }

    #[test]
    fn live_failed_suppliers_preserve_spatial_data_but_not_nested_answers() {
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &[".+0001", ".+0203", "./0100", ""],
            &[(0, 48), (16, 2), (32, 2)],
            &[],
        );
        assert_eq!(&source.snapshot()[..6], ".+0501");
        assert_eq!(&source.snapshot()[48..50], "06");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message == "cannot divide by zero")
        );
        let (plan, source) =
            configured_source(Grid::new(16, 2), &[".+02./0100", ""], &[(0, 16)], &[]);
        assert_eq!(&source.snapshot()[16..18], "  ");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("supplied no typed result"))
        );
        // A failed structural writer leaves the original computation connected.
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+02.x0304", "./0100", ""],
            &[(0, 32), (16, 4)],
            &[],
        );
        assert_eq!(&source.snapshot()[..10], ".+02.x0304");
        assert_eq!(&source.snapshot()[32..34], "0E");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message == "cannot divide by zero")
        );
    }

    #[test]
    fn live_inactive_ownership_and_terminal_portal_configuration_are_independent() {
        observed::take();
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &["!>007F.^80", "", ""],
            &[(0, 16), (6, 20)],
            &[],
        );
        assert!(plan.writes.is_empty());
        assert!(plan.play_commands.is_empty());
        assert_eq!(observed::take().len(), 0);
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("cannot have a Portal"));
        assert_eq!(&source.snapshot()[16..32], "                ");
        let (plan, _) = configured_source(
            Grid::new(16, 4),
            &["!>007FC4", "", ".=0101", ""],
            &[(0, 34), (32, 16)],
            &[],
        );
        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("cannot have a Portal"));
    }

    #[test]
    fn live_deep_and_top_level_replacement_suppress_descendant_portals() {
        observed::take();
        let (plan, source) = configured_source(
            Grid::new(20, 3),
            &[".+02.x03.+0101", ".+0203", ""],
            &[(0, 40), (4, 44), (8, 48), (20, 4)],
            &[],
        );
        assert_eq!(&source.snapshot()[..14], ".+020503.+0101");
        assert_eq!(&source.snapshot()[40..42], "07");
        assert_eq!(&source.snapshot()[44..50], "      ");
        assert_eq!(observed::take().len(), 2);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert!(source.language_map().expressions().any(|entry| {
            entry
                .root()
                .is_some_and(|root| root.x() == 8 && root.y() == 0)
        }));
        observed::take();
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+02.x0304", ".+0203", ""],
            &[(0, 32), (4, 36), (16, 0)],
            &[],
        );
        assert_eq!(&source.snapshot()[..10], "0502.x0304");
        assert_eq!(&source.snapshot()[32..38], "      ");
        assert_eq!(observed::take().len(), 1);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            source
                .language_map()
                .expressions()
                .find_map(|entry| entry.root())
                .unwrap()
                .x(),
            4
        );
    }

    #[test]
    fn live_function_replacement_keeps_nesting_and_reinterprets_only_literals() {
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &[".+02.x0304", ".+0000", ".+0001", ""],
            &[(0, 48), (4, 34), (16, 0), (32, 52)],
            &[(16, lang::Function::Multiply)],
        );
        assert_eq!(&source.snapshot()[..10], ".x02.x0304");
        assert_eq!(&source.snapshot()[48..50], "18");
        assert_eq!(&source.snapshot()[52..54], "0D");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".vC4", ".+0000", ""],
            &[(0, 32), (16, 0)],
            &[(16, lang::Function::ConvertToNote)],
        );
        assert_eq!(&source.snapshot()[..4], ".^C4");
        assert_eq!(&source.snapshot()[32..34], "  ");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message == "Number C4 cannot be converted to a Note")
        );
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".v.^3C", ".+0000", ""],
            &[(0, 32), (16, 0)],
            &[(16, lang::Function::ConvertToNote)],
        );
        // The retained nested Note C4 remains typed; Numeric Conversion is idempotent.
        assert_eq!(&source.snapshot()[32..34], "C4");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    #[test]
    fn live_replacement_checks_retained_arity_and_never_runs_new_anchors() {
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0204", ".+0000", ""],
            &[(0, 32), (16, 0)],
            &[(16, lang::Function::ConvertToNote)],
        );
        assert_eq!(&source.snapshot()[..6], ".^0204");
        assert_eq!(&source.snapshot()[32..34], "  ");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("expected 1, found 2"))
        );
        assert_eq!(
            source
                .language_map()
                .expressions()
                .next()
                .unwrap()
                .span()
                .end()
                .get(),
            3
        );
        assert!(source.language_map().diagnostics().any(|d| d.start() == 4));
        observed::take();
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0101", ".+0000", ""],
            &[(0, 32), (16, 1)],
            &[(16, lang::Function::Multiply)],
        );
        assert_eq!(&source.snapshot()[..6], "..x101");
        assert_eq!(&source.snapshot()[32..34], "  ");
        assert_eq!(observed::take().len(), 1);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert!(
            source
                .language_map()
                .diagnostics()
                .any(|d| d.start() == 1 && d.message.contains("1 "))
        );
    }

    #[test]
    fn live_cycles_reject_independent_effects_and_self_dependency() {
        for outputs in [
            vec![(0, 18), (16, 2), (32, 48)],
            vec![(0, 2), (16, 64), (32, 48)],
        ] {
            observed::take();
            let (plan, source) = configured_source(
                Grid::new(16, 5),
                &[".+0001", ".+0001", ".=0101", "", "!>007FC4"],
                &outputs,
                &[],
            );
            assert!(plan.writes.is_empty());
            assert!(plan.play_commands.is_empty());
            assert_eq!(observed::take().len(), 0);
            assert_eq!(&source.snapshot()[48..50], "  ");
            assert!(
                plan.diagnostics
                    .iter()
                    .any(|d| d.message == "same-Tick dependency cycle")
            );
        }
        let (plan, _) = configured_source(
            Grid::new(16, 3),
            &[".+02.x0304", ".+0001", ""],
            &[(0, 18), (4, 18), (16, 6)],
            &[],
        );
        assert!(plan.writes.is_empty());
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message == "same-Tick dependency cycle")
        );
    }
    #[test]
    fn original_anchor_function_replacement_retains_inputs() {
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0204", ".+0000", ""],
            &[(0, 32), (16, 0)],
            &[(16, lang::Function::Multiply)],
        );
        assert_eq!(&source.snapshot()[..6], ".x0204");
        assert_eq!(&source.snapshot()[32..34], "08");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    #[test]
    fn a_value_replaces_nested_computation_and_preserves_next_tick_source() {
        observed::take();
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &[".+02.x0304", ".+0203", "", ""],
            &[(0, 48), (4, 52), (16, 4)],
            &[],
        );
        assert_eq!(&source.snapshot()[..10], ".+02050304");
        assert_eq!(&source.snapshot()[48..50], "07");
        assert_eq!(&source.snapshot()[52..54], "  ");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(observed::take().len(), 2);
        let next = source.language_map();
        assert_eq!(next.expressions().next().unwrap().span().end().get(), 5);
        assert!(next.diagnostics().any(|diagnostic| diagnostic.start() == 6));
        assert!(next.diagnostics().any(|diagnostic| diagnostic.start() == 8));
    }

    #[test]
    fn nested_computation_returns_and_projects_once() {
        let grid = Grid::new(16, 4);
        observed::take();
        let (plan, source) = configured_source(
            grid,
            &[".+02.x0304", ".+0101", "", ""],
            &[(0, 48), (4, 18), (16, 52)],
            &[],
        );
        assert_eq!(&source.snapshot()[18..20], "0C");
        assert_eq!(&source.snapshot()[48..50], "0E");
        assert_eq!(&source.snapshot()[52..54], "0D");
        assert_eq!(&source.snapshot()[..10], ".+02.x0304");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        let calls = observed::take();
        assert_eq!(
            calls
                .iter()
                .filter(|inputs| **inputs
                    == super::tick_inputs(Tick::ZERO, grid.position(4, 0).unwrap()))
                .count(),
            1
        );
    }

    #[test]
    fn partial_writers_settle_before_consumption() {
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &[".+0101", ".+0203", ".+0101", ""],
            &[(0, 56), (16, 2), (32, 3)],
            &[],
        );
        assert_eq!(&source.snapshot()[..6], ".+0021");
        assert_eq!(&source.snapshot()[56..58], "21");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            source
                .language_map()
                .expressions()
                .next()
                .unwrap()
                .span()
                .end()
                .get(),
            5
        );
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
        // Cleared display may be rewritten successfully in the same Tick.
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
    fn outputs_beside_and_over_standalone_source_are_admitted() {
        // These exact Sources used to trip the obsolete join guard.
        let (plan, source) =
            configured_source(Grid::new(8, 3), &[".=0101", "  0102", ""], &[], &[]);
        assert_eq!(&source.snapshot()[8..14], "**0102");
        assert_eq!(planned(&plan), vec![(8, '*'), (9, '*')]);
        assert!(plan.diagnostics.is_empty());
        assert_eq!(source.language_map().bangs().count(), 1);
        let (plan, source) =
            configured_source(Grid::new(10, 3), &["    .=0101", "  0102", ""], &[], &[]);
        assert_eq!(&source.snapshot()[10..16], "  01**");
        assert_eq!(planned(&plan), vec![(14, '*'), (15, '*')]);
        assert!(plan.diagnostics.is_empty());
    }

    #[test]
    fn a_destination_at_the_row_edge_costs_one_expression_its_turn_not_the_tick() {
        // A complete destination must fit even when the configured Portal
        // is at the final Cell, independently of other computations.
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
                .any(|diagnostic| diagnostic.message.contains("crosses the row edge")),
            "the Expression at the row edge keeps its own diagnostic: {:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn overlapping_outputs_beside_standalone_source_both_contribute_cells() {
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0102 .+0304", "0102", ""],
            &[(0, 20), (7, 21)],
            &[],
        );
        assert_eq!(planned(&plan), vec![(20, '0'), (21, '0'), (22, '7')]);
        assert_eq!(&source.snapshot()[16..23], "0102007");
        assert!(plan.diagnostics.is_empty());
    }

    #[test]
    fn source_after_an_expression_is_the_next_expressions_and_costs_the_tick_nothing() {
        // What used to be trailing Source. ADR 0033 ends an Expression where
        // its arity does, so the `Z` after `.+0102` is the Source the next
        // parse reads rather than evidence against the Addition: the Addition
        // takes its turn, and the Tick has nothing to report about either.
        //
        // A fifth row so the Addition has somewhere to put its result: what
        // this test is about is that the Addition takes a turn at all.
        let grid = Grid::new(16, 5);
        let bytes = snapshot(grid, &[".=0101", "", "!>007FC4", ".+0102Z", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());

        let plan = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        // The Bang the Equality writes, and the Addition's own `03` below it.
        assert_eq!(
            planned(&plan),
            vec![(16, '*'), (17, '*'), (64, '0'), (65, '3')]
        );
        assert!(
            plan.diagnostics.is_empty(),
            "the Tick reported something about Source it does not own: {:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn a_row_edge_fragment_diagnoses_its_own_expression_and_leaves_the_tick_playing() {
        // A row-edge fragment is local syntax failure, not a graph error.
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
    fn an_activated_consumer_uses_surviving_cells_after_supplier_failure() {
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &["  .+", "!>007FC4", "", ".=0101"],
            &[(48, 32)],
            &[],
        );
        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert_eq!(&source.snapshot()[16..24], "!>007FC4");
        assert!(source.language_map().diagnostics().any(|d| d.start() == 2));
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
    fn competing_writers_preserve_an_independent_rejected_destination_diagnostic() {
        let (plan, source) = configured_source(
            Grid::new(16, 3),
            &[".+0102", ".+0304", ".+0506"],
            &[(0, 26), (16, 26), (32, 31)],
            &[],
        );
        assert_eq!(planned(&plan), vec![(26, '0'), (27, '7')]);
        assert_eq!(&source.snapshot()[26..28], "07");
        assert_eq!(&source.snapshot()[31..32], " ");
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("crosses the row edge"));
    }

    #[test]
    fn competing_writers_publish_but_dependency_cycles_abort_before_output() {
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
        assert_eq!(planned(&conflict), vec![(26, '0'), (27, '7')]);
        assert!(conflict.play_commands.is_empty());
        assert!(conflict.diagnostics.is_empty());

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
