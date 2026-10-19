//! Tick-local execution of Parser-owned expressions.
//!
//! Fixed Portal destinations and nested ownership determine the order before
//! execution. A Portal found from operands at a Turn adds its dependencies to
//! that order during the Tick. Spatial writes remain character encodings until
//! consumed, and a nested result returns to its parent as the same encoding,
//! so the receiving operand decodes both. Only the final effects are
//! published.

#[cfg(test)]
mod absolute_copy;
#[cfg(test)]
mod bang_activation;
#[cfg(test)]
mod bang_function;
#[cfg(test)]
mod clear;
#[cfg(test)]
mod conflicts;
#[cfg(test)]
mod copy;
#[cfg(test)]
mod cycles;
mod dependencies;
pub(super) mod execution;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod halt;
mod inputs;
#[cfg(test)]
mod live_editing;
#[cfg(test)]
mod movers;
#[cfg(test)]
mod nested_copy;
#[cfg(test)]
mod nesting;
#[cfg(test)]
pub(super) mod observed;
mod ordering;
#[cfg(test)]
mod outputs;
#[cfg(test)]
mod play_order;
pub(super) mod portals;
#[cfg(test)]
mod push;
#[cfg(test)]
mod read;
#[cfg(test)]
mod replacement;
#[cfg(test)]
mod rest;
#[cfg(test)]
mod schedule;
#[cfg(test)]
mod schedule_reuse;
#[cfg(test)]
mod tick_functions;
#[cfg(test)]
mod track;
#[cfg(test)]
mod write;

use lang::{Anchor, Function, PlayCommand, Tick, TickInputs};
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::{Arc, OnceLock};

use super::encoding::Encoding;
use super::language_map::{LanguageMap, Span};
pub(super) use super::portal::{Coverage, Occupancy, Occupant, PortalError, PortalUnit, coverage};
use super::portal::{Portal, PortalAccess, SCALAR_WIDTH, SpanWrite};
use super::{CellContent, CellWrite, Cells, Diagnostic, TickPlan};
use crate::grid::{CellIndex, Grid, Position};
use inputs::Parsed;
pub(super) use inputs::{RowInputs, ScheduleInputs, ScheduledExpression};
use ordering::{Order, Stall};
use portals::TickPortals;

///
/// One revision's computations and the order their Turns are taken in. No
/// Tick changes it: every Tick planned against the same scheduling inputs
/// executes the one schedule they share.
///
pub(super) struct Schedule {
    lookup: Lookup,
    locks: dependencies::Locks,
    ordered: Order,
    /// The layout diagnostics the computations owe, then the cycles the order
    /// stops.
    diagnostics: Vec<Diagnostic>,
}

impl Schedule {
    fn new(
        lookup: Lookup,
        locks: dependencies::Locks,
        ordered: Order,
        mut diagnostics: Vec<Diagnostic>,
    ) -> Self {
        diagnostics.extend(word_stalls(&lookup, &ordered.stalls));
        Self {
            lookup,
            locks,
            ordered,
            diagnostics,
        }
    }

    /// Whether activation known before the Tick reaches `root`.
    #[cfg(test)]
    fn holds_active(&self, root: ComputationId) -> bool {
        self.ordered.holds_active(root)
    }
}

///
/// Takes one Tick's Turns through `schedule`, as [`ordering::take_turns`]
/// does, and answers the diagnostics for the Turns a cycle found during the
/// Tick leaves untaken.
///
fn take_turns(
    schedule: &Schedule,
    take: impl FnMut(ComputationId, &ordering::Progress<'_>) -> ordering::Turn,
) -> Vec<Diagnostic> {
    let stalls = ordering::take_turns(
        &schedule.ordered,
        &dependencies::Joining {
            lookup: &schedule.lookup,
            locks: &schedule.locks,
        },
        take,
    );
    word_stalls(&schedule.lookup, &stalls)
}

/// The diagnostic each [`Stall`] owes, at the computation it names.
fn word_stalls(lookup: &Lookup, stalls: &[Stall]) -> Vec<Diagnostic> {
    let nodes = lookup.nodes();
    stalls
        .iter()
        .map(|stall| match *stall {
            Stall::Cycle(index) => diagnose(&nodes[index], "same-Tick dependency cycle"),
            Stall::Waiting(root) => {
                diagnose(&nodes[root], "waiting on a same-Tick dependency cycle")
            }
        })
        .collect()
}

///
/// What ordering knows of `lookup`'s computations: each one's Grid position
/// and the Expression it belongs to.
///
fn ordering_nodes(lookup: &Lookup) -> ordering::Nodes {
    let computations = lookup.nodes();
    ordering::Nodes {
        keys: computations
            .iter()
            .map(|node| lookup.grid.index(node.anchor).get())
            .collect(),
        owners: computations.iter().map(Computation::owner).collect(),
        expressions: computations
            .iter()
            .map(|node| lookup.descendants(node.owner()))
            .collect(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Effect {
    Write(SpanWrite),
    Play(PlayCommand),
    Diagnose(Diagnostic),
    /// Withholds the Expression root at this anchor. Not a Cell write.
    Lock(Position),
}

/// One computation's place in its schedule: its index in [`Lookup::nodes`].
///
/// Ids order as Parser preorder does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ComputationId(usize);

impl ComputationId {
    /// The bare index this computation holds, for a type stated outside the
    /// schedule. Every table the schedule keeps is a [`PerComputation`].
    fn index(self) -> usize {
        self.0
    }
}

///
/// One value for each computation of a schedule, read and written by
/// [`ComputationId`], so that a Cell index or an ordering key cannot stand
/// where a computation belongs.
///
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PerComputation<T>(Vec<T>);

impl<T> PerComputation<T> {
    /// `value` for each of `len` computations.
    fn filled(len: usize, value: T) -> Self
    where
        T: Clone,
    {
        Self(vec![value; len])
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    /// Every computation's id, in Parser preorder.
    fn ids(&self) -> impl Iterator<Item = ComputationId> + use<T> {
        (0..self.0.len()).map(ComputationId)
    }

    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    /// Each computation's id beside its value, in Parser preorder.
    fn iter_enumerated(&self) -> impl Iterator<Item = (ComputationId, &T)> {
        self.0
            .iter()
            .enumerate()
            .map(|(index, value)| (ComputationId(index), value))
    }

    /// The values in Parser preorder, for a caller outside the schedule that
    /// reads them in that order.
    fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> FromIterator<T> for PerComputation<T> {
    /// Values in Parser preorder, the first for computation zero.
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self(values.into_iter().collect())
    }
}

impl<'a, T> IntoIterator for &'a PerComputation<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> std::ops::Index<ComputationId> for PerComputation<T> {
    type Output = T;

    fn index(&self, id: ComputationId) -> &T {
        &self.0[id.0]
    }
}

impl<T> std::ops::IndexMut<ComputationId> for PerComputation<T> {
    fn index_mut(&mut self, id: ComputationId) -> &mut T {
        &mut self.0[id.0]
    }
}

impl<T> std::ops::Index<ComputationRange> for PerComputation<T> {
    type Output = [T];

    fn index(&self, range: ComputationRange) -> &[T] {
        &self.0[range.0]
    }
}

impl<T> std::ops::IndexMut<ComputationRange> for PerComputation<T> {
    fn index_mut(&mut self, range: ComputationRange) -> &mut [T] {
        &mut self.0[range.0]
    }
}

///
/// Computations contiguous in Parser preorder, as a subtree or a whole
/// Expression is, iterated as their ids.
///
#[derive(Clone, Debug, PartialEq, Eq)]
struct ComputationRange(Range<usize>);

impl ComputationRange {
    /// From `start` up to, not including, the computation at index `end`.
    fn new(start: ComputationId, end: usize) -> Self {
        Self(start.0..end)
    }

    /// The first computation of the range, which leads a subtree.
    fn start(&self) -> ComputationId {
        ComputationId(self.0.start)
    }

    fn contains(&self, id: ComputationId) -> bool {
        self.0.contains(&id.0)
    }
}

impl Iterator for ComputationRange {
    type Item = ComputationId;

    fn next(&mut self) -> Option<ComputationId> {
        self.0.next().map(ComputationId)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// Where a computation stands in its Expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Place {
    /// It leads its Expression, which it owns.
    Root,
    /// It answers an operand of `parent`, in the Expression `owner` leads.
    Nested {
        parent: ComputationId,
        owner: ComputationId,
    },
}

struct Operand {
    cells: Range<usize>,
    /// The computation that answers this operand, when a Function fills it.
    child: Option<ComputationId>,
}

struct Computation {
    id: ComputationId,
    anchor: Position,
    span: Span,
    function: Function,
    place: Place,
    operands: Vec<Operand>,
    syntax_valid: bool,
    portal_access: PortalAccess,
}

impl Computation {
    /// The computation this one answers, or `None` for a root.
    fn parent(&self) -> Option<ComputationId> {
        match self.place {
            Place::Root => None,
            Place::Nested { parent, .. } => Some(parent),
        }
    }

    /// The root of the Expression this computation belongs to: itself for a
    /// root.
    fn owner(&self) -> ComputationId {
        match self.place {
            Place::Root => self.id,
            Place::Nested { owner, .. } => owner,
        }
    }

    fn is_root(&self) -> bool {
        self.place == Place::Root
    }
}

struct Claim {
    cells: Range<usize>,
    node: ComputationId,
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

    fn touching(&self, cells: Range<usize>) -> impl Iterator<Item = ComputationId> + '_ {
        let first = self
            .0
            .partition_point(|claim| claim.cells.end <= cells.start);
        self.0[first..]
            .iter()
            .take_while(move |claim| claim.cells.start < cells.end)
            .map(|claim| claim.node)
    }
}

/// Write reservations, or static Input Portal reads, bucketed by row.
///
/// [`Claims`] requires disjoint ranges. Producer destinations do not: two
/// Portals may name the same Cells. A Span never leaves its row, so each Input Portal
/// read searches only the writes that share it — naming an Input Portal on
/// every Increment must not scan every other root. Reads overlap in the same
/// way, and a writer searches only the reads that share its row.
struct WriteClaims {
    columns: usize,
    by_row: Vec<Vec<Claim>>,
}

impl WriteClaims {
    fn new(grid: Grid, claims: Vec<Claim>) -> Self {
        let columns = grid.columns();
        let mut by_row: Vec<Vec<Claim>> = (0..grid.rows()).map(|_| Vec::new()).collect();
        for claim in claims {
            if claim.cells.is_empty() {
                continue;
            }
            let row = claim.cells.start / columns;
            debug_assert!(
                claim.cells.end <= (row + 1) * columns,
                "a Span stays in one row"
            );
            by_row[row].push(claim);
        }
        Self { columns, by_row }
    }

    fn touching(&self, cells: Range<usize>) -> impl Iterator<Item = ComputationId> + '_ {
        let row = cells.start / self.columns;
        self.by_row
            .get(row)
            .into_iter()
            .flatten()
            .filter(move |claim| claim.cells.start < cells.end && cells.start < claim.cells.end)
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
    nodes: PerComputation<Computation>,
    functions: Claims,
    literals: Claims,
    operands: Claims,
    /// Destination Cells each producer reserved, including overlapping writes.
    writes: WriteClaims,
    /// Cells each static Input Portal reads, by reader, so that a writer
    /// whose root joins a Tick finds its readers without asking every one.
    reads: WriteClaims,
    /// The index one past each computation's last descendant, which ends
    /// the contiguous range its subtree holds in Parser preorder.
    subtree_ends: PerComputation<usize>,
}

impl Lookup {
    fn new(grid: Grid, nodes: PerComputation<Computation>) -> Self {
        let mut functions = Vec::new();
        let mut literals = Vec::new();
        let mut operands = Vec::new();
        debug_assert!(
            nodes.iter_enumerated().all(|(id, node)| node.id == id),
            "each computation's id is its index",
        );
        let mut subtree_ends: PerComputation<_> = (1..=nodes.len()).collect();
        for (index, node) in nodes.iter_enumerated() {
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
            // Copy input Cells are recorded on the computation and become
            // Dependencies in `dependencies::derive`. They are not literal claims:
            // an aligned input often overlaps an operand Span, and `Claims`
            // are disjoint.
        }
        // Nodes retain the Parser's depth-first preorder. Every subtree is a
        // contiguous range, including its root, even with missing operands.
        for node in nodes.iter().rev() {
            if let Some(parent) = node.parent() {
                subtree_ends[parent] = subtree_ends[parent].max(subtree_ends[node.id]);
            }
        }
        let mut writes = Vec::new();
        for (index, node) in nodes.iter_enumerated() {
            for output in node
                .portal_access
                .write_sites()
                .iter()
                .filter_map(|output| output.as_ref().ok())
            {
                if let Some(span) = Portal::at(grid, *output).reservation() {
                    writes.push(Claim {
                        cells: span.range(),
                        node: index,
                    });
                }
            }
        }
        let reads = nodes
            .iter_enumerated()
            .flat_map(|(index, node)| {
                node.portal_access
                    .read_spans()
                    .iter()
                    .map(move |read| Claim {
                        cells: read.clone(),
                        node: index,
                    })
            })
            .collect();
        Self {
            grid,
            nodes,
            functions: Claims::new(functions),
            literals: Claims::new(literals),
            operands: Claims::new(operands),
            writes: WriteClaims::new(grid, writes),
            reads: WriteClaims::new(grid, reads),
            subtree_ends,
        }
    }

    fn nodes(&self) -> &PerComputation<Computation> {
        &self.nodes
    }

    fn descendants(&self, ancestor: ComputationId) -> ComputationRange {
        ComputationRange::new(ancestor, self.subtree_ends[ancestor])
    }

    fn root_at(&self, anchor: Position) -> Option<ComputationId> {
        let cell = self.grid.index(anchor).get();
        self.functions
            .touching(cell..cell + 1)
            .find(|&index| self.nodes[index].is_root() && self.nodes[index].anchor == anchor)
    }
}

///
/// Plans one Tick against `map`, through the schedule every revision holding
/// the same scheduling inputs shares.
///
/// The Grid, its Cells, their Language Map and the Tick are everything a Tick
/// depends on: two Sources holding the same Cells plan the same Tick. The Tick
/// Portals are planned with the Tick Plan, so the two are committed or
/// dropped together.
///
pub(super) fn plan(
    grid: Grid,
    cells: Cells<'_>,
    map: &LanguageMap,
    tick: Tick,
) -> (TickPlan, TickPortals, Vec<execution::ComputationState>) {
    execution::execute(grid, cells, map, tick, shared_schedule(grid, map))
}

///
/// [`plan`] against a schedule ordered afresh from `map`, which is what the
/// shared schedule has to agree with.
///
#[cfg(test)]
fn plan_unshared(
    grid: Grid,
    cells: Cells<'_>,
    map: &LanguageMap,
    tick: Tick,
) -> (TickPlan, TickPortals, Vec<execution::ComputationState>) {
    execution::execute(
        grid,
        cells,
        map,
        tick,
        &schedule(grid, map.schedule_inputs()),
    )
}

///
/// The schedule one set of scheduling inputs admits, ordered the first time a
/// Tick is planned against them and shared by every Language Map revision
/// that holds the same inputs.
///
/// A schedule reads the Grid and the [`ScheduleInputs`] of its Map, and
/// nothing else: each Expression with a leading Function as a
/// [`ScheduledExpression`], and every Language Unit, whose anchors and Spans
/// decide the occupancy a Halt target is classified by.
///
/// Portal destinations are read from the Function's declaration, never from
/// an operand. A dynamic Portal's position is found at the Turn and is no
/// scheduling input; which computations are dynamic writers, and what feeds
/// each one, are. An Operand Literal's value, working
/// Source and the Tick are execution's alone, so a Tick that writes new values
/// into Cells whose units keep their Spans changes no scheduling input.
/// [`LanguageMap::rebuild`] compares each re-derived row's [`RowInputs`] with
/// the row it replaces and carries this cache to the new revision when every
/// one is equal, which is what lets the cache survive a commit that rewrites
/// identical bytes as well as one that writes nothing. Construction reads the
/// Map through the same projection, so it reads nothing that comparison
/// misses.
///
/// Ordering waits for the first Tick planned against the inputs, so a
/// revision no Tick is planned against costs no ordering. The cache is shared
/// by pointer and filled once under [`OnceLock`], so any holder of the Map
/// can plan against it, from any thread.
///
#[derive(Clone, Default)]
pub(super) struct ScheduleCache(Arc<OnceLock<Schedule>>);

impl ScheduleCache {
    /// Whether `other` is this cache, rather than one ordered separately.
    #[cfg(test)]
    fn is_shared_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// Whether a Tick has already ordered this schedule.
    #[cfg(test)]
    fn is_filled(&self) -> bool {
        self.0.get().is_some()
    }
}

///
/// The schedule `map` shares with every revision holding equal scheduling
/// inputs, ordered now from `map`'s inputs if no Tick has asked for it.
///
/// The cache and the inputs that fill it are both read from `map`, and no
/// other path fills a cache, so a cache only ever holds a schedule ordered
/// from inputs equal to those of every revision that shares it.
///
fn shared_schedule(grid: Grid, map: &LanguageMap) -> &Schedule {
    map.schedule_cache()
        .0
        .get_or_init(|| schedule(grid, map.schedule_inputs()))
}

///
/// Gives each computation the Portal destinations `destinations` names for it,
/// in place of the default Portal one row south that it resolved for itself.
///
/// Done to the computations once [`computations`] has stated them rather than
/// while it is stating them, so that a schedule can carry chosen destinations
/// without the planning path taking a parameter or a map lookup of its own.
///
/// Writes that resolved as none stay none: [`PortalAccess::carry`] cannot
/// attach a Portal to Terminal Output or to a nested effect Function. Tests
/// that need a write site name one on a Function that already demanded some.
///
#[cfg(test)]
fn carry(
    grid: Grid,
    nodes: &mut PerComputation<Computation>,
    destinations: &BTreeMap<CellIndex, Vec<Position>>,
) {
    for id in nodes.ids() {
        let node = &mut nodes[id];
        let Some(writes) = destinations.get(&grid.index(node.anchor)) else {
            continue;
        };
        node.portal_access.carry(grid, writes);
    }
}

///
/// The schedule for `map` with `destinations` carried on its computations,
/// rather than resolved for them while they were stated.
///
#[cfg(test)]
fn schedule_carrying(
    grid: Grid,
    map: &LanguageMap,
    destinations: &BTreeMap<CellIndex, Vec<Position>>,
) -> Schedule {
    let inputs = map.schedule_inputs();
    let (mut nodes, diagnostics) = computations(grid, inputs);
    carry(grid, &mut nodes, destinations);
    order_turns(Lookup::new(grid, nodes), inputs, diagnostics)
}

///
/// Plans one Tick against a schedule carrying `destinations`.
///
/// [`plan`] for a Tick whose destinations a test states, rather than the
/// default Portal one row south that its roots resolve for themselves.
///
#[cfg(test)]
pub(super) fn plan_carrying(
    grid: Grid,
    cells: Cells<'_>,
    map: &LanguageMap,
    tick: Tick,
    destinations: &BTreeMap<CellIndex, Vec<Position>>,
) -> (TickPlan, TickPortals, Vec<execution::ComputationState>) {
    execution::execute(
        grid,
        cells,
        map,
        tick,
        &schedule_carrying(grid, map, destinations),
    )
}

fn diagnose(node: &Computation, message: impl Into<String>) -> Diagnostic {
    Diagnostic::for_expression(node.anchor, node.span, message.into())
}

///
/// The order one revision's Turns are taken in, and the diagnostics its
/// layout and cycles owe.
///
/// Reads nothing but `inputs`, because every revision holding equal inputs
/// shares what this answers.
///
fn schedule(grid: Grid, inputs: ScheduleInputs<'_>) -> Schedule {
    let (nodes, diagnostics) = computations(grid, inputs);
    order_turns(Lookup::new(grid, nodes), inputs, diagnostics)
}

///
/// The Output Portal Reservation of every computation a schedule of `inputs`
/// holds, in Parser preorder: each value answer's Cell pair, root or nested,
/// through [`PortalAccess::output_portal_reservation`], the Cell pair the
/// schedule reserves for that answer's write.
///
/// Allocates nothing.
///
pub(super) fn output_portal_reservations(
    grid: Grid,
    inputs: ScheduleInputs<'_>,
) -> impl Iterator<Item = Span> + '_ {
    inputs.functions(grid).filter_map(move |site| {
        PortalAccess::output_portal_reservation(grid, site.anchor, site.function)
    })
}

///
/// The computations one Source revision holds, in Parser preorder, and the
/// diagnostics its layout owes before any of them is ordered.
///
/// This is everything a schedule knows before a [`Lookup`] indexes it: which
/// Cells each computation claims, how each interacts with Portals, and which
/// Expressions the row edge cut short. Every value answer reserves one Atom's
/// Cell pair at each write Portal, the same [`Portal::reservation`] the
/// Output Portal Reservations read.
///
fn computations(
    grid: Grid,
    inputs: ScheduleInputs<'_>,
) -> (PerComputation<Computation>, Vec<Diagnostic>) {
    let mut nodes: PerComputation<Computation> = PerComputation(Vec::new());
    let mut diagnostics = Vec::new();
    // The computation each entry of the current Expression parsed as, by
    // entry index. Reused across Expressions.
    let mut entry_nodes: Vec<Option<ComputationId>> = Vec::new();
    for expression in inputs.expressions() {
        entry_nodes.clear();
        for entry in expression.entries() {
            let parent = entry.parent.map(|parent| {
                entry_nodes[parent].expect("the Parser binds a Function at every parent entry")
            });
            let child = entry.function(grid).map(|site| {
                let id = ComputationId(nodes.len());
                let place = match parent {
                    None => Place::Root,
                    Some(parent) => Place::Nested {
                        parent,
                        owner: nodes[parent].owner(),
                    },
                };
                // A nested value Function writes its own Output Portal as a
                // root does. A root Terminal Output Function has no Cell
                // destination at all: Play is an Effect, not a Portal. A
                // locking root reserves its Output Portal and writes no Cell.
                let portal_access =
                    PortalAccess::resolve(grid, site.anchor, site.function, parent.is_some());
                nodes.0.push(Computation {
                    id,
                    anchor: site.anchor,
                    span: expression.span(),
                    function: site.function,
                    place,
                    operands: vec![],
                    syntax_valid: true,
                    portal_access,
                });
                id
            });
            entry_nodes.push(child);
            if let Some(parent) = parent {
                let parent = &mut nodes[parent];
                parent.syntax_valid &= entry.parsed != Parsed::Unparsed;
                parent.operands.push(Operand {
                    cells: entry.cells,
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
            // its row. No boundary exists mid-row to name instead: a Comment is
            // a Language Unit an Expression's claim reaches over, not a place
            // the Source stops.
            diagnostics.push(diagnose(node, "Expression layout crosses the row edge"));
        }
    }
    (nodes, diagnostics)
}

///
/// The schedule for the reservations a [`Lookup`] has already derived: every
/// Dependency [`dependencies::derive`] finds, ordered by [`ordering::schedule`], and the
/// diagnostics its cycles owe after `diagnostics`.
///
fn order_turns(
    lookup: Lookup,
    inputs: ScheduleInputs<'_>,
    diagnostics: Vec<Diagnostic>,
) -> Schedule {
    let locks = dependencies::Locks::new(&lookup, inputs);
    let dependencies::Derived {
        active,
        dependencies,
        writers,
    } = dependencies::derive(&lookup, &locks);
    let ordered = ordering::schedule(ordering_nodes(&lookup), active, dependencies, writers);
    Schedule::new(lookup, locks, ordered, diagnostics)
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
    let mut locks = Vec::new();

    for effect in effects {
        match effect {
            Effect::Write(write) => {
                for (cell, content) in write.cells() {
                    writes.insert(cell, content);
                }
            }
            Effect::Play(command) => play_commands.push(command),
            Effect::Diagnose(diagnostic) => diagnostics.push(diagnostic),
            Effect::Lock(root) => locks.push(root),
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
        locks,
    }
}

///
/// The explicit inputs for one root's evaluation.
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
