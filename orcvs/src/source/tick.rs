//! Tick-local execution of Parser-owned expressions (ADR 0034).
//!
//! Fixed Portal destinations and nested ownership determine the complete order
//! before execution. Spatial writes remain character encodings until consumed;
//! nested results are typed values. Only the final effects are published.

mod execution;

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
    /// A test-supplied answer replaces this computation's declared one, so it
    /// replaces the width scheduling reserves for it too. Production reads the
    /// Function's own declaration and nothing else, which is why the field is
    /// absent rather than false there: a reservation that could be widened from
    /// outside the table would not be the declared fact ADR 0036 rests on.
    #[cfg(test)]
    supplied_sequence: bool,
}

impl Computation {
    #[cfg(test)]
    fn supplies_sequence(&self) -> bool {
        self.supplied_sequence
    }

    #[cfg(not(test))]
    fn supplies_sequence(&self) -> bool {
        false
    }
}

#[derive(Default)]
pub(super) struct Configuration {
    destinations: BTreeMap<CellIndex, Vec<Position>>,
    /// The answer to deliver in place of the one the Function would compute.
    ///
    /// A whole [`Interpretation`] rather than an [`Atom`], because ADR 0036's
    /// reservation is a fact about the answer's width and an Atom can only ever
    /// state one of the two widths. Nothing spells a Sequence-answering Function
    /// in Source until ADR 0007's Range and Concatenate are built, so this is
    /// the seam a test states one through, and it is read twice: once while
    /// scheduling, to reserve the Cells the answer can reach, and once during
    /// execution, in place of the Interpreter's result.
    #[cfg(test)]
    supplied: BTreeMap<CellIndex, Interpretation>,
}

struct Schedule {
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
    /// The Grid whose Cell numbering every Claim and subtree range below is
    /// stated in. Owning it keeps a query from restating a Position in another
    /// Grid's coordinates, which `Grid::assert_owns` cannot refuse because the
    /// caller would be offering a Position that Grid genuinely owns.
    grid: Grid,
    /// Claims and subtree ranges index this fixed collection. Owning it keeps
    /// queries from pairing those indices with a different set of computations.
    nodes: Vec<Computation>,
    functions: Claims,
    literals: Claims,
    operands: Claims,
    subtree_ends: Vec<usize>,
    /// One entry per node, in the same order, computed once because a node's
    /// reservation reads its children's.
    reserved: Vec<Reserved>,
}

/// How many Cells scheduling reserves for one computation's result, per
/// ADR 0036.
///
/// A schedule is fixed before any Function evaluates, so this is derived from
/// declarations rather than from a width that does not exist yet. It is an
/// over-approximation on purpose: the reservation orders the Turns, and the
/// admitted write — always a subset of it — decides what is actually written,
/// suppressed, or activated.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Reserved {
    /// The `SCALAR_WIDTH` Cells one Atom occupies. This stays the case for
    /// every computation whose answer cannot be a Sequence, which today is
    /// every computation there is.
    Pair,
    /// Every Cell from the destination through the end of its row. A Sequence's
    /// width is not known when the schedule is built, and no Span reaches past
    /// the row it begins in, so the rest of the row is the smallest reservation
    /// that can name every Cell such a write might reach.
    Row,
}

/// Relationships of one fixed Portal destination, and of the Cells following it
/// along its row, to the original computations. A Portal names where a result
/// begins rather than how wide it is, so how many Cells to ask about is the
/// caller's to state: scheduling asks over the Cells it reserved and execution
/// asks over the Cells an admitted write actually covers. The phases share these
/// facts, but decide separately whether a producer can activate, must precede,
/// or suppresses a contacted computation.
struct PortalRelationships<'a> {
    lookup: &'a Lookup,
    output: Position,
    cells: Range<usize>,
}

struct FunctionContact {
    index: usize,
    at_anchor: bool,
    /// Includes the contacted Function itself, in Parser preorder.
    subtree: Range<usize>,
}

/// The Cell width scheduling reserves for one scalar result. Execution
/// refuses any other width from a [`Reserved::Pair`] computation before it
/// admits a write, which is the only reason a scalar destination admitted by a
/// Portal is always one the reservation covers.
const SCALAR_WIDTH: usize = 2;

impl Lookup {
    fn new(grid: Grid, nodes: Vec<Computation>) -> Self {
        let mut functions = Vec::new();
        let mut literals = Vec::new();
        let mut operands = Vec::new();
        let mut subtree_ends: Vec<_> = (1..=nodes.len()).collect();
        for (index, node) in nodes.iter().enumerate() {
            let start = grid.index(node.anchor).get();
            // A Function's own spelling, not a result: this 2 is the glyph
            // width and stays a literal, because `SCALAR_WIDTH` would tie it to
            // the Cells a scalar result reserves, which is a different fact.
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
        // The same preorder is what makes one reverse pass enough here: every
        // operand child sits at a higher index than the Function that owns it,
        // so a node's children are already answered when its own turn comes.
        let mut reserved = vec![Reserved::Pair; nodes.len()];
        for index in (0..nodes.len()).rev() {
            reserved[index] = reserved_for(&nodes, &reserved, index, nodes[index].function);
        }
        Self {
            grid,
            nodes,
            functions: Claims::new(functions),
            literals: Claims::new(literals),
            operands: Claims::new(operands),
            reserved,
            subtree_ends,
        }
    }

    fn nodes(&self) -> &[Computation] {
        &self.nodes
    }

    fn descendants(&self, ancestor: usize) -> Range<usize> {
        ancestor..self.subtree_ends[ancestor]
    }

    fn root_at(&self, anchor: Position) -> Option<usize> {
        let cell = self.grid.index(anchor).get();
        self.functions
            .touching(cell..cell + 1)
            .find(|&index| self.nodes[index].parent.is_none() && self.nodes[index].anchor == anchor)
    }

    /// What scheduling reserved for `index`'s result, per ADR 0036.
    fn reserved(&self, index: usize) -> Reserved {
        self.reserved[index]
    }

    /// What scheduling would have reserved for `index` had its Function been
    /// `function`.
    ///
    /// A spatial write can replace a Function at its original anchor after the
    /// schedule is fixed, and a replacement that answers a wider result than
    /// the one reserved for would write Cells no dependency edge names. This
    /// is the question `deliver_output` asks before it admits such a
    /// replacement; it reads its children's settled reservations, which the
    /// same guard keeps stable.
    fn reserved_with(&self, index: usize, function: Function) -> Reserved {
        reserved_for(&self.nodes, &self.reserved, index, function)
    }

    /// A fixed destination has relationships only if the Cells reserved for
    /// `index` fit the row. Actual writes still go through `Portal::admit`,
    /// which also validates their encoding and supplies the producer's
    /// diagnostic.
    ///
    /// A [`Reserved::Row`] reservation runs to the end of the destination's own
    /// row and so always fits, which is what makes the answer `None` a
    /// statement about a scalar result specifically: a Cell pair whose second
    /// Cell is in the next row is not a Span at all.
    fn reserved_at(&self, index: usize, output: Position) -> Option<PortalRelationships<'_>> {
        let start = self.grid.index(output).get();
        let cells = match self.reserved(index) {
            Reserved::Pair => {
                self.grid.offset_in_row(output, SCALAR_WIDTH - 1)?;
                start..start + SCALAR_WIDTH
            }
            // The row's remaining Cells, measured from the destination's own
            // column so the count stops at the row edge rather than running on
            // into the next row's Cells.
            Reserved::Row => start..start + (self.grid.cols() - output.x()),
        };
        Some(PortalRelationships {
            lookup: self,
            output,
            cells,
        })
    }

    /// The relationships of the `width` Cells one admitted write actually
    /// covers, from `output` along its row.
    ///
    /// Execution asks this rather than [`Lookup::reserved_at`] because the
    /// reservation is deliberately wider than most writes: a computation inside
    /// a `Reserved::Row` reservation that the write stopped short of was
    /// ordered after its producer and then not written over, so it must not be
    /// suppressed. A width is what the caller has — the Cells it names are this
    /// Grid's to number — and those Cells are always a subset of what
    /// scheduling reserved: a Cell pair is exactly the scalar reservation, and
    /// a Portal refuses any encoding that leaves the destination's row.
    ///
    /// That subset holds over the Cells, and so over the two relationships that
    /// grow with them: [`PortalRelationships::functions`] and
    /// [`PortalRelationships::literal_consumers`] each answer a subset of what
    /// they answered for the reservation, so every contact execution acts on is
    /// one a dependency edge already names. It does not carry to
    /// [`PortalRelationships::bang_roots`], which is not monotonic in its
    /// range: a range touching any operand Cell answers nothing at all, so the
    /// wide reservation can answer no root where these narrower Cells answer
    /// one. Only a `Reserved::Row` producer answering Bang could tell the two
    /// apart, and none exists — Equality is the sole Function that can emit
    /// Bang and it declares `Answer::Atom`, so every Bang producer is reserved
    /// a Cell pair and asks both questions over the same two Cells.
    /// [`PortalRelationships::bang_roots`] states what the first such Function
    /// has to settle.
    fn written_over(&self, output: Position, width: usize) -> PortalRelationships<'_> {
        let start = self.grid.index(output).get();
        PortalRelationships {
            lookup: self,
            output,
            cells: start..start + width,
        }
    }
}

/// Whether a computation's answer can be wider than one Atom, given the
/// reservations already settled for its operand children.
///
/// Sequence-capability is derivable before any Function evaluates because a
/// Sequence can only reach a computation from a nested child: ADR 0034 makes a
/// spatial write literal characters that the receiving operand decodes by its
/// declared literal type, and ADR 0007 gives a Sequence no literal spelling to
/// decode. So a literal operand is always one Atom however it was written over,
/// and the two declared columns decide the rest — a Function that answers a
/// Sequence outright, or one that widens over an operand that is itself one.
///
/// The nodes are read rather than the [`Lookup`] because this also runs while
/// that `Lookup` is being built.
fn reserved_for(
    nodes: &[Computation],
    reserved: &[Reserved],
    index: usize,
    function: Function,
) -> Reserved {
    let node = &nodes[index];
    let widened = function.widens_over_a_sequence_operand()
        && node.operands.iter().any(|operand| {
            operand
                .child
                .is_some_and(|child| reserved[child] == Reserved::Row)
        });
    if function.answers_sequence() || widened || node.supplies_sequence() {
        Reserved::Row
    } else {
        Reserved::Pair
    }
}

impl PortalRelationships<'_> {
    fn functions(&self) -> impl Iterator<Item = FunctionContact> + '_ {
        self.lookup
            .functions
            .touching(self.cells.clone())
            .map(|index| FunctionContact {
                index,
                at_anchor: self.lookup.nodes()[index].anchor == self.output,
                subtree: self.lookup.descendants(index),
            })
    }

    fn literal_consumers(&self) -> impl Iterator<Item = usize> + '_ {
        self.lookup.literals.touching(self.cells.clone())
    }

    /// Geometrically eligible roots, regardless of their activation policy or
    /// whether this producer actually returns Bang. Even a partial overlap
    /// with an operand excludes activation; nested operands count here too.
    ///
    /// Operand contact is a fact about the whole destination rather than about
    /// any one anchor, so it decides the empty answer up front instead of
    /// filtering the four cardinal anchors one at a time.
    ///
    /// "The whole destination" is whichever Cells the caller stated, which for
    /// a [`Reserved::Row`] producer asked through [`Lookup::reserved_at`] is
    /// the rest of the row: such a producer touches an operand almost wherever
    /// it points and so answers no Bang root at all. Nothing reaches that
    /// today, because Equality is the only Function that can emit Bang and it
    /// answers one Atom whatever its operands carry, so every Bang producer is
    /// reserved a Cell pair. The constraint becomes live the first time a
    /// Function that answers or widens into a Sequence can also emit Bang, and
    /// it is that Function's decision to make: alignment is a fact about a
    /// Bang's own two Cells rather than about the Cells a wider answer might
    /// have reached, so what the reservation should be asked here is settled
    /// against that Function's tests rather than guessed at now.
    fn bang_roots(&self) -> impl Iterator<Item = usize> + '_ {
        let in_operand = self
            .lookup
            .operands
            .touching(self.cells.clone())
            .next()
            .is_some();
        let (column, row) = (self.output.x(), self.output.y());
        let anchors = if in_operand {
            [None, None, None, None]
        } else {
            [
                row.checked_sub(1)
                    .and_then(|north| self.lookup.grid.position(column, north)),
                self.lookup.grid.position(column, row + 1),
                column
                    .checked_sub(2)
                    .and_then(|west| self.lookup.grid.position(west, row)),
                self.lookup.grid.position(column + 2, row),
            ]
        };
        anchors
            .into_iter()
            .flatten()
            .filter_map(|anchor| self.lookup.root_at(anchor))
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
    execution::execute(grid, bytes, map, tick, configuration, schedule)
}

/// An inactive root can contribute no child Portal. Start from value roots,
/// then close over potential Bang deliveries; actual activation is still
/// checked during execution, after those producers have settled.
fn potentially_active(lookup: &Lookup) -> Vec<bool> {
    let nodes = lookup.nodes();
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
                let Some(relationships) = lookup.reserved_at(index, *output) else {
                    continue;
                };
                for index in relationships.bang_roots() {
                    if !nodes[index].function.answers_value() && !active[index] {
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
                // The one gate that means Terminal Output rather than
                // "answers an effect": a Terminal Output Function has no Cell
                // destination at all, while ADR 0004 gives a Source-writing
                // Function a validated write bundle and ADR 0009 lets it
                // resolve multiple Portals. Asking the wide question here
                // would diagnose the Halt, Directional Bang, and Jump
                // Functions for a Portal they are entitled to and hand each of
                // them no destination.
                let outputs = if function.performs_terminal_output() {
                    if configured.is_some() {
                        diagnostics.push(Diagnostic::for_expression(
                            anchor,
                            expression.span(),
                            "a Terminal Output Function cannot have a Portal".to_owned(),
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
                    // Read here, beside the destinations the same Configuration
                    // supplies, so a stated answer reaches the schedule that
                    // has to reserve for it and not only the execution that
                    // delivers it.
                    #[cfg(test)]
                    supplied_sequence: matches!(
                        configuration.supplied.get(&grid.index(anchor)),
                        Some(Interpretation::Sequence(_))
                    ),
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
            // The row edge is the only boundary a truncated operand can meet.
            // An operand is short of its Token width only where `take_token`
            // ran out of Source, and that path claims what is left of the
            // Source it was handed, so the Expression ends at the last Cell of
            // its row. The second message this chose between —
            // "Expression operand crosses the Source boundary" — named the cut
            // the `##` pre-pass made mid-row, and ADR 0035 deleted the pre-pass
            // rather than the boundary it invented: a Comment is a Language
            // Unit an Expression's claim reaches over, not a place the Source
            // stops.
            diagnostics.push(diagnose(node, "Expression layout crosses the row edge"));
        }
    }
    let lookup = Lookup::new(grid, nodes);
    let nodes = lookup.nodes();
    let active = potentially_active(&lookup);
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
            let Some(relationships) = lookup.reserved_at(index, *output) else {
                continue;
            };
            // A self-edge is an unsatisfiable indegree, so it is how a
            // computation that writes over its own Cells reports itself as a
            // same-Tick cycle. That is exact for a `Reserved::Pair` producer,
            // whose write is held to the `SCALAR_WIDTH` Cells it reserved: the
            // reservation covering the producer and the write reaching it are
            // the same fact, and `live_cycles_reject_independent_effects_and_
            // self_dependency` holds that rule.
            //
            // The two come apart for a `Reserved::Row` producer. Its
            // reservation runs to the end of the destination's row, so a
            // destination in its own row at or left of its Cells covers its
            // spelling and literals whatever the answer turns out to be, and
            // per ADR 0036 a reservation orders Turns and decides nothing else.
            // Ordering such a producer after itself would reject the whole
            // Grid's Tick for a write that may stop columns short of it, which
            // is the cycle between computations that never touch that ADR
            // 0036's rejected alternative exists to avoid. No edge can express
            // "take your Turn after yourself" in any case, so whether the write
            // reached the producer is left to the admitted write: execution
            // asks `written_over` over the Cells actually covered, and ADR
            // 0034's executed-computation guard is waiting for them there.
            let reserves_row = lookup.reserved(index) == Reserved::Row;
            let mut order_after = |consumer: usize| {
                if !(reserves_row && consumer == index) {
                    edges.insert((index, consumer));
                }
            };
            for contact in relationships.functions() {
                for descendant in contact.subtree {
                    order_after(descendant);
                }
            }
            for consumer in relationships.literal_consumers() {
                order_after(consumer);
            }
            if node.function.can_emit_bang() {
                for owner in relationships.bang_roots() {
                    if !nodes[owner].function.answers_value() {
                        for consumer in lookup.descendants(owner) {
                            order_after(consumer);
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
        lookup,
        order,
        diagnostics,
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
    use super::{Effect, Interpretation, Portal, Tick, observed, resolve};

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
        configured_source_answers(
            grid,
            rows,
            outputs,
            &supplied
                .iter()
                .map(|(anchor, atom)| (*anchor, Interpretation::Cell(*atom)))
                .collect::<Vec<_>>(),
        )
    }

    ///
    /// Supplies a whole answer where the Interpreter would have computed one.
    ///
    /// The seam a Sequence result is stated through until ADR 0007's Range and
    /// Concatenate can spell one in Source. A supplied Sequence replaces the
    /// answer its Function declares, so scheduling reserves for it exactly as
    /// it will for a Range: these tests drive the production reservation rather
    /// than a test-only path around it.
    ///
    fn configured_source_answers(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
        supplied: &[(usize, Interpretation)],
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
                .map(|(anchor, answer)| (cell(grid, *anchor), answer.clone())),
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

    ///
    /// The aligned successors of the `:#` case that motivated ADR 0035.
    ///
    /// `.|` is the one spelling whose second Cell is a `|`, so it is the one
    /// that could present a `||` to a scan stepping over overlapping byte
    /// pairs rather than in two-Cell units. The Parser reads in units, so a
    /// `|` beside an Absolute Difference is that Function's first operand and
    /// fails to bind, and the row holds no Comment at all.
    ///
    /// A row of `.|` and a real introducer is the other half: the Function
    /// keeps the claim its arity declares, and the `||` after it opens a
    /// Comment that claims what is left. Both rows run a Tick, because what
    /// changed is which Cells the walk hands the Parser and the answer has to
    /// hold through execution rather than only through derivation.
    ///
    #[test]
    fn an_absolute_difference_beside_a_vertical_rule_is_read_in_two_cell_units() {
        // `.|` at Cells 0 and 1 with a `|` at Cell 2: the pair at Cells 1 and
        // 2 spells `||` and means nothing, because nothing reads it.
        let (plan, source) = configured_source(Grid::new(8, 2), &[".||102", ""], &[], &[]);

        assert!(plan.writes.is_empty());
        assert!(
            !source
                .language_map()
                .units()
                .any(|unit| unit.kind() == crate::source::LanguageUnitKind::Comment)
        );
        let refused = source.language_map().expressions().next().unwrap();
        assert_eq!(refused.span().positions().count(), 6);
        assert!(refused.root().is_none());

        // The same Function beside a real introducer. The Absolute Difference
        // of 01 and 02 answers 01 and writes it below; the Comment claims the
        // rest of the row and answers nothing.
        let (plan, source) = configured_source(Grid::new(8, 2), &[".|0102||", ""], &[], &[]);

        assert_eq!(&source.snapshot()[8..10], "01");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            source
                .language_map()
                .units()
                .filter(|unit| unit.kind() == crate::source::LanguageUnitKind::Comment)
                .map(|unit| unit.anchor().x())
                .collect::<Vec<_>>(),
            vec![6]
        );
        for column in 6..8 {
            assert_eq!(
                source
                    .language_map()
                    .glyph_at(source.grid().position(column, 0).unwrap()),
                Some(crate::glyph::Glyph::Comment)
            );
        }
    }

    ///
    /// The `##` collision that broke the pre-pass holds no Comment.
    ///
    /// This is the shape of the row that motivated ADR 0035: `:#` at Cells 3
    /// and 4 with a `#` at Cell 5 presented `##` at Cells 4 and 5, and the
    /// walk cut the row at Cell 4, in the middle of what a Function would
    /// have been. One keystroke of a Live Edit reached it.
    ///
    /// ADR 0035 moved the Comment off `#` as well as into the parse, so `#`
    /// spells nothing at all now and the collision class is gone. What this
    /// pins is that no Comment forms and the row is read one Cell at a time.
    /// It does not pin the Note Range reading: `:#` is not in the Function
    /// table until `sequence-values/05` lands, so `:` `#` `#` are three
    /// characters the table does not hold, each refused its own Cell by ADR
    /// 0018's recovery. When `:#` becomes a Function the diagnostics below
    /// change to one refused Function, and no Comment still forms — which is
    /// this test's claim either way.
    ///
    #[test]
    fn the_hash_collision_that_broke_the_pre_pass_holds_no_comment() {
        let (plan, source) = configured_source(Grid::new(8, 2), &["** :##", ""], &[], &[]);

        assert!(
            !source
                .language_map()
                .units()
                .any(|unit| unit.kind() == crate::source::LanguageUnitKind::Comment)
        );
        assert_eq!(
            source
                .language_map()
                .diagnostics()
                .filter(|diagnostic| diagnostic.message.starts_with("invalid Language Unit"))
                .map(|diagnostic| diagnostic.start())
                .collect::<Vec<_>>(),
            vec![3, 4, 5]
        );
        // The Bang before them is a whole Expression and still fires, which
        // clears its own two Cells and writes nothing else. Nothing the row
        // holds after it is Source anything reads, so nothing else can.
        assert_eq!(&source.snapshot()[..2], "  ");
        assert_eq!(
            plan.writes
                .iter()
                .map(|write| write.cell.get())
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
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
        assert!(plan.diagnostics.iter().any(|d| {
            d.message
                .contains("activation requirements, output kind, or result width")
        }));
    }

    ///
    /// One Sequence answer of Numbers, stated for the `supplied` seam.
    ///
    fn numbers(values: &[u8]) -> Interpretation {
        Interpretation::Sequence(
            lang::Sequence::new(values.iter().copied().map(lang::Atom::Number))
                .expect("a Number is a Sequence member"),
        )
    }

    #[test]
    fn live_a_sequence_result_reaches_its_destination_cells() {
        // ADR 0007's ordinary Sequence result, reached through a Tick rather
        // than through the Portal on its own: the schedule reserves Cells for
        // it, execution encodes it, and the Source Grid the next Tick reads
        // carries all six Cells. Three Atoms rather than one is what separates
        // this from the scalar case it now shares a path with.
        let grid = Grid::new(16, 2);
        let (plan, source) = configured_source_answers(
            grid,
            &[".+0102", ""],
            &[],
            &[(0, numbers(&[0x0A, 0x0B, 0x0C]))],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert!(plan.play_commands.is_empty());
        assert_eq!(plan.writes.len(), 6, "one Cell write per encoded Cell");
        assert_eq!(source.snapshot(), snapshot(grid, &[".+0102", "0A0B0C"]));
    }

    #[test]
    fn live_a_sequence_result_that_leaves_its_row_writes_no_cell_of_it() {
        // ADR 0007's complete-fit rule, which ADR 0009's Portal enforces, for a
        // Sequence exactly as for a scalar: no Span reaches past the row it
        // begins in, so an encoding running past the row's end is refused
        // entire. Four of the six Cells fit and none of them is written, which
        // is the half of the rule a diagnostic alone would not hold.
        //
        // The destination sits in the middle row of three, so the row edge it
        // overruns has another row after it: what is refused here is leaving
        // the row, not approaching the end of the Grid. The Grid's own edges
        // are the test below.
        let grid = Grid::new(16, 3);
        let rows = [".+0102", "", ""];
        let (plan, source) = configured_source_answers(
            grid,
            &rows,
            &[(0, 28)],
            &[(0, numbers(&[0x0A, 0x0B, 0x0C]))],
        );

        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("crosses the row edge")),
            "{:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn live_a_sequence_result_at_a_grid_edge_writes_no_cell_of_it() {
        // The two ways the Grid, rather than the row, refuses a Sequence. A
        // root in the last row resolves no ordinary destination at all, so
        // there is nothing to fit into; and the last row's final Cells are the
        // Grid's final Cells, so an encoding past them leaves the Grid. The
        // third case is the one that must still work: a Sequence ending exactly
        // on the Grid's last Cell is admitted, so the refusals above are about
        // leaving the Grid and not about being near its edge.
        let grid = Grid::new(16, 2);
        let below = ["", ".+0102"];
        let (plan, source) =
            configured_source_answers(grid, &below, &[], &[(16, numbers(&[0x0A, 0x0B]))]);
        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert_eq!(source.snapshot(), snapshot(grid, &below));
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("falls below the Source")),
            "{:?}",
            plan.diagnostics
        );

        let rows = [".+0102", ""];
        let (plan, source) =
            configured_source_answers(grid, &rows, &[(0, 30)], &[(0, numbers(&[0x0A, 0x0B]))]);
        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("crosses the row edge")),
            "{:?}",
            plan.diagnostics
        );

        let (plan, source) =
            configured_source_answers(grid, &rows, &[(0, 30)], &[(0, numbers(&[0x0A]))]);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(plan.writes.len(), 2);
        assert_eq!(
            source.snapshot(),
            snapshot(grid, &[".+0102", "              0A"])
        );
    }

    #[test]
    fn live_two_overlapping_sequence_results_resolve_cell_by_cell() {
        // ADR 0020's Cell-wise conflict resolution, which a Sequence inherits
        // rather than restates: one admitted write is one validated effect
        // until the Tick Plan resolves, and then as many independently
        // contested Cells as it has characters. The later producer wins the two
        // Cells it overlaps and the earlier producer's first four Cells stand,
        // which a rule resolving whole writes would have replaced together.
        let grid = Grid::new(16, 2);
        let (plan, source) = configured_source_answers(
            grid,
            &[".+0000.+0000", ""],
            &[(0, 16), (6, 20)],
            &[
                (0, numbers(&[0x0A, 0x0B, 0x0C])),
                (6, numbers(&[0x0D, 0x0E])),
            ],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            plan.writes.len(),
            8,
            "six Cells and four Cells sharing two of them"
        );
        assert_eq!(
            source.snapshot(),
            snapshot(grid, &[".+0000.+0000", "0A0B0D0E"])
        );
    }

    #[test]
    fn live_an_empty_sequence_result_plans_no_write_and_no_diagnostic() {
        // ADR 0007: the empty Sequence is a value holding no Atoms rather than
        // a refused one, so it plans no Cell write and reports nothing. It
        // never reaches a Portal, which is what lets `Portal::admit` assert
        // that a write places at least one Cell.
        let grid = Grid::new(16, 2);
        let rows = [".+0102", ""];
        let (plan, source) = configured_source_answers(
            grid,
            &rows,
            &[],
            &[(0, Interpretation::Sequence(lang::Sequence::empty()))],
        );

        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert!(plan.play_commands.is_empty());
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
    }

    #[test]
    fn live_a_sequence_reservation_orders_every_computation_its_write_can_reach() {
        // ADR 0036's reservation, observed as the ordering it buys. The
        // producer sits in row 1 and writes upward into row 0, so row-major
        // order alone would run the Expression at column 8 first. A scalar
        // reservation covers only columns 0 and 1 and names no edge to it; the
        // Sequence's write then reaches an already-executed computation and
        // ADR 0034 rejects the whole Tick. Reserving through the end of the
        // destination row instead orders the producer first, and the
        // Expression it covers is suppressed rather than executed against a
        // spelling that is no longer there.
        let grid = Grid::new(16, 2);
        let (plan, source) = configured_source_answers(
            grid,
            &["        .+0102", ".+0000"],
            &[(16, 0)],
            &[(16, numbers(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]))],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(plan.writes.len(), 12);
        assert_eq!(
            source.snapshot(),
            snapshot(grid, &["0A0B0C0D0E0F02", ".+0000"]),
            "the covered Expression neither executed nor kept its spelling",
        );
    }

    #[test]
    fn live_a_nested_sequence_answer_widens_the_reservation_of_the_root_above_it() {
        // ADR 0036's Consequences: a Function that answers a Sequence widens
        // the reservation of every pervasive ancestor between it and its root.
        // The Sequence is answered by the nested `.-` at column 2 of row 1, and
        // the root `.+` that owns it declares no Sequence answer of its own —
        // it reserves the whole of the destination row only because the
        // production table says it widens over an operand that is one. That is
        // the propagation every other Sequence test here short-circuits by
        // stating its answer at the root itself.
        //
        // The ordering is again the evidence, for the reason the test above
        // gives: the root writes upward into row 0, and the Expression at
        // column 10 there is inside the widened reservation, so it is ordered
        // after the root and suppressed rather than executed against a spelling
        // the write has replaced. A scalar reservation would name no edge to
        // it, and the root's twelve-Cell answer would then be refused as a
        // result that is not a scalar Cell pair.
        assert!(
            !lang::Function::Add.answers_sequence()
                && lang::Function::Add.widens_over_a_sequence_operand(),
            "the root's reservation can only have been derived from its child",
        );

        let grid = Grid::new(16, 2);
        let (plan, source) = configured_source_answers(
            grid,
            &["          .+0102", ".+.-000003"],
            &[(16, 0)],
            &[(18, numbers(&[0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C]))],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(plan.writes.len(), 12);
        assert_eq!(
            source.snapshot(),
            snapshot(grid, &["0A0B0C0D0E0F0102", ".+.-000003"]),
            "the root answered its child's Sequence widened by `03`, and the \
             Expression it covered neither executed nor kept its spelling",
        );
    }

    #[test]
    fn live_a_reservation_the_sequence_stopped_short_of_suppresses_nothing() {
        // The other half of ADR 0036: a reservation is deliberately wider than
        // most of the writes it covers, and only the write decides what
        // happened to a Cell. The Expression at column 8 is inside the reserved
        // row and outside the four Cells the Sequence actually reached, so it
        // is ordered after the producer and then executes normally, answering
        // `03` into row 1. Suppressing everything the reservation names would
        // leave that Cell pair empty.
        let grid = Grid::new(16, 2);
        let (plan, source) = configured_source_answers(
            grid,
            &["        .+0102", ".+0000"],
            &[(16, 0)],
            &[(16, numbers(&[0x0A, 0x0B]))],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            source.snapshot(),
            snapshot(grid, &["0A0B    .+0102", ".+0000  03"]),
        );
    }

    #[test]
    fn live_a_reservation_covering_its_own_producer_orders_nothing_against_it() {
        // ADR 0036: a Reservation orders Turns and decides nothing else, and a
        // computation the admitted write stopped short of is left standing.
        // The producer is one such computation whenever its destination lies
        // in its own row at or left of its own Cells, because a
        // `Reserved::Row` reservation runs from the destination through the
        // end of that row and so covers the producer's spelling and literals
        // along with everything else.
        //
        // Ordering a producer after itself is not a dependency, it is an
        // artefact of measuring the reservation from the row rather than from
        // the write: the four Cells this Sequence actually reaches stop at
        // column 3 and never come near the Expression at column 8. A self-edge
        // makes that Tick a cycle and discards every write and Play Command in
        // the Grid, which is the outcome ADR 0036's rejected alternative names
        // — a cycle manufactured between computations that never touch.
        let grid = Grid::new(16, 2);
        let (plan, source) = configured_source_answers(
            grid,
            &["        .+0102", ""],
            &[(8, 0)],
            &[(8, numbers(&[0x0A, 0x0B]))],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(plan.writes.len(), 4);
        assert_eq!(source.snapshot(), snapshot(grid, &["0A0B    .+0102", ""]));
    }

    #[test]
    fn live_a_sequence_that_writes_over_its_own_producer_rejects_the_tick() {
        // The other half of the rule above. Dropping the self-edge for a
        // `Reserved::Row` producer moves the question of writing over itself
        // from the schedule to the admitted write; it does not answer it away.
        // Twelve Cells from column 0 reach the `.+` at column 8, and the
        // producer is the executed computation ADR 0034 refuses an output to,
        // so the Tick is rejected entire and the Source is unchanged. The
        // Expression that is left standing when the write stops short is the
        // test above; this is what happens when it does not.
        let grid = Grid::new(16, 2);
        let rows = ["        .+0102", ""];
        let (plan, source) = configured_source_answers(
            grid,
            &rows,
            &[(8, 0)],
            &[(8, numbers(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]))],
        );

        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
        assert!(
            plan.diagnostics.iter().any(|d| {
                d.message == "spatial output reached an executed computation; Tick effects rejected"
            }),
            "{:?}",
            plan.diagnostics
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
    fn a_late_spatial_write_rejects_the_tick_even_when_the_earlier_turn_failed() {
        for target in [".+0101", "./0100", ".+01??"] {
            let grid = Grid::new(16, 11);
            let bytes = snapshot(
                grid,
                &[
                    "", "!>007FC4", ".=0101", "./0100", target, ".+0203", "", ".+0304", "**", "",
                    "",
                ],
            );
            let map = LanguageMap::derive(grid, &bytes).unwrap();
            let mut configuration = super::Configuration::default();
            for (producer, destination) in [(32, 0), (48, 160), (64, 96), (80, 64), (112, 160)] {
                configuration.destinations.insert(
                    cell(grid, producer),
                    vec![grid.position_at(cell(grid, destination))],
                );
            }
            // A valid order delivers the writer before its target. Execute it
            // once to prove that this fixture has writes and a Play Command
            // which the defensive rejection below must discard.
            let plan =
                super::plan_configured(grid, bytes.as_bytes(), &map, Tick::ZERO, &configuration);
            assert!(!plan.writes.is_empty());
            assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);

            let mut schedule = super::schedule(grid, &map, &configuration).unwrap();
            // Supply a broken order at the execution seam: the scheduler must
            // never produce this, but execution promises to reject it rather
            // than panic or publish the effects already accumulated.
            schedule.order = [32, 16, 48, 64, 80, 112]
                .into_iter()
                .map(|anchor| {
                    schedule
                        .lookup
                        .nodes()
                        .iter()
                        .position(|node| grid.index(node.anchor) == cell(grid, anchor))
                        .unwrap()
                })
                .collect();
            observed::take();
            let rejected = super::execution::execute(
                grid,
                bytes.as_bytes(),
                &map,
                Tick::ZERO,
                &configuration,
                schedule,
            );
            assert!(rejected.writes.is_empty());
            assert!(rejected.play_commands.is_empty());
            let mut expected = vec![(48, "cannot divide by zero")];
            if target == "./0100" {
                expected.push((64, "cannot divide by zero"));
            }
            expected.push((
                80,
                "spatial output reached an executed computation; Tick effects rejected",
            ));
            assert_eq!(
                rejected
                    .diagnostics
                    .iter()
                    .map(|diagnostic| {
                        (
                            grid.index(diagnostic.anchor()).get(),
                            diagnostic.message.as_str(),
                        )
                    })
                    .collect::<Vec<_>>(),
                expected,
            );
            // Attempting a syntax-blocked Turn still prevents later writes
            // reaching it, although that Turn never calls the Evaluator.
            assert_eq!(
                observed::take().len(),
                if target == ".+01??" { 4 } else { 5 }
            );
        }
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
    fn fixed_bang_destinations_respect_alignment_and_operand_contact() {
        let grid = Grid::new(16, 6);
        // One terminal at (4, 2). A Bang two Cells east of it is in its
        // channel operand, so cardinal alignment alone cannot activate it.
        for (column, row, performs) in [
            (4, 1, true),
            (4, 3, true),
            (2, 2, true),
            (6, 2, false),
            (3, 1, false),
            (5, 1, false),
            (1, 2, false),
            (3, 2, false),
            (15, 1, false),
        ] {
            let (plan, _) = configured_source(
                grid,
                &["", "", "    !>007FC4", "", "", ".=0101"],
                &[(80, row * 16 + column)],
                &[],
            );
            let expected = if performs {
                vec![raw(0, 0x7F, 60)]
            } else {
                vec![]
            };
            assert_eq!(
                plan.play_commands, expected,
                "destination ({column}, {row}): {:?}",
                plan.diagnostics
            );
            if column == 15 {
                // A destination whose pair leaves the row is refused at the
                // Portal, and a terminal wide enough to sound cannot be
                // cardinally aligned with one, so this case cannot separate
                // `Lookup::reserved_at`'s row-fit guard from `Portal::admit`'s
                // refusal.
                // `competing_writers_preserve_an_independent_rejected_destination_diagnostic`
                // is what holds that guard.
                assert!(plan.writes.is_empty());
                assert_eq!(plan.diagnostics.len(), 1);
                assert!(plan.diagnostics[0].message.contains("crosses the row edge"));
            } else {
                assert_eq!(plan.writes.len(), 2);
                assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
            }
        }
    }

    #[test]
    fn a_bang_half_inside_an_operand_does_not_activate_an_aligned_terminal() {
        // Operand contact is a fact about the whole reserved pair, so a
        // destination whose first Cell is free still belongs to an operand when
        // its second Cell lands in one. Row 2 holds `.=` at columns 3 and 4
        // with its first operand at columns 5 and 6; the destination (4, 2)
        // covers columns 4 and 5, so only its second Cell is in that operand.
        // The terminal directly below is cardinally aligned and must stay
        // silent anyway -- narrowing the contact test to the anchor Cell alone
        // would sound it.
        let (plan, _) = configured_source(
            Grid::new(16, 6),
            &["", "", "   .=0101", "    !>007FC4", "", ".=0101"],
            &[(80, 2 * 16 + 4)],
            &[],
        );
        assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(plan.writes.len(), 2);
    }

    #[test]
    fn a_bang_touching_a_nested_function_operand_does_not_activate_a_neighbour() {
        // The destination contacts a nested Function rather than a literal.
        // It still belongs to an operand, so neither its inactive owner nor
        // the aligned terminal below it can perform.
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &["    .=0101", "!>00.+0101C4", "    !>007FC5", ""],
            &[],
            &[],
        );
        assert!(plan.play_commands.is_empty());
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(&source.snapshot()[16..28], "!>00**0101C4");
        assert_eq!(plan.writes.len(), 2);
    }

    #[test]
    fn a_bang_cardinally_aligned_with_a_nested_function_anchor_activates_nothing() {
        // The other half of non-root contact. The test above puts the Bang
        // inside the operand Cells, so operand contact alone answers it. Here
        // the destination is a clear row of its own and only its southern
        // anchor lands anywhere: on the `.+` at column 4, which is a nested
        // Function and so no root. Nothing is eligible, so the terminal that
        // owns that `.+` never performs and its row is left exactly as typed.
        let (plan, source) = configured_source(
            Grid::new(16, 4),
            &["", "!>00.+0101C4", "", ".=0101"],
            &[(48, 4)],
            &[],
        );
        assert!(plan.play_commands.is_empty(), "{:?}", plan.play_commands);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(&source.snapshot()[0..8], "    **  ");
        assert_eq!(&source.snapshot()[16..28], "!>00.+0101C4");
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
