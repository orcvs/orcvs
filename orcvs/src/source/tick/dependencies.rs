//! Every Dependency a Tick's Turns are ordered by, derived from a [`Lookup`].
//!
//! A Dependency comes from a Reservation, a lock, a Bang, an Input Portal or
//! nesting. This module states each rule once. Ordering asks two questions:
//! [`derive()`], every Dependency and dynamic writer known before the Tick, and
//! [`join()`], those a root activated during the Tick brings with it. Which
//! roots activation reaches is derived here too, because a Bang's
//! Reservation is what delivers it. Execution asks what an admitted write
//! covers ([`reached()`]), whether it is refused ([`refusal`]), and what each
//! Contact and aligned root it covers means ([`contact`], [`bang_root`]); what
//! a lock reaches ([`Locks::lock_target`]); and which writers a dynamic Input
//! Portal waits on ([`input_writers`]).
//!
//! A Contact means one thing in each [`Phase`]. Scheduling asks before the
//! Tick, when a Contact can only order its producer first; execution asks
//! during it, against the Turns taken so far. Each rule answers both phases in
//! one place, so a rule cannot order one thing and decide another.

use std::collections::BTreeSet;
use std::ops::Range;

use lang::{Atom, Function, Output, ReplacementChange, SourceBundle};

use super::ordering::{Join, Joined};
use super::{ComputationId, ComputationRange, Lookup, PerComputation, ScheduleInputs};
use crate::grid::Position;
use crate::source::portal::{Occupancy, Portal, SpanWrite};

///
/// What a schedule is ordered from: which roots activation reaches before the
/// Tick, every Dependency as a `(producer, consumer)` pair, and the dynamic
/// writers those roots own.
///
pub(super) struct Derived {
    pub(super) active: PerComputation<bool>,
    pub(super) dependencies: BTreeSet<(ComputationId, ComputationId)>,
    pub(super) writers: Vec<ComputationId>,
}

///
/// How a write's place in the Tick was decided, which decides what it may
/// reach.
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Delivery {
    /// A static Output Portal of a root the schedule held active: the
    /// schedule ordered it before every computation whose Cells it reaches.
    Reserved,
    /// A static Output Portal of a root activated during the Tick: its place
    /// in the order was decided during the Tick.
    Joined,
    /// A dynamic Output Portal, whose destination its Turn selected.
    Selected,
}

impl Delivery {
    /// Whether a computation this write reaches may already have taken its
    /// Turn, which then meets the write next Tick as feedback.
    pub(super) fn feeds_back(self) -> bool {
        self != Self::Reserved
    }
}

/// What the Turns taken so far this Tick have done, as the reach rules read
/// it.
pub(super) trait TurnFacts {
    /// Whether `index`'s Turn has opened.
    fn attempted(&self, index: ComputationId) -> bool;
    /// Whether a write covered `index` before its Turn.
    fn suppressed(&self, index: ComputationId) -> bool;
    /// The Function `index` is running: the one the Parser found, until a
    /// Function Replacement in this Tick changes it.
    fn running(&self, index: ComputationId) -> Function;
}

/// When a reach rule is asked what a Contact or an aligned root means.
#[derive(Clone, Copy)]
pub(super) enum Phase<'a> {
    /// Before the Tick. Nothing has a Turn yet, so a Contact can only order
    /// its producer first: a Reservation orders Turns and decides nothing
    /// else.
    Schedule,
    /// During the Tick, for a Turn whose place `delivery` decided, against
    /// the Turns taken so far. `written` is the Atom its admitted write
    /// carries, and `None` when it carries no Atom: `Answer::unit` answers
    /// `None` for a Clear, which writes two spaces, and a lock writes no
    /// Cells. `None` therefore does not mean no Cells were written.
    Turn {
        delivery: Delivery,
        written: Option<Atom>,
        turns: &'a dyn TurnFacts,
    },
}

/// What one Contact, or one root a Bang aligns with, means in a [`Phase`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) enum Consequence {
    /// Nothing follows from it.
    Unchanged,
    /// The producer takes its Turn before every computation in the range.
    OrderAfter(ComputationRange),
    /// The contacted computation runs this Function for the rest of the
    /// Tick.
    Replace(Function),
    /// Every computation in the range that has not attempted its Turn is
    /// suppressed.
    Suppress(ComputationRange),
    /// The root is activated, unless it has taken its Turn without it.
    Activate(ComputationId),
}

/// One Contact's [`Consequence`] during a Turn, and whether it lands on the
/// producer's own Expression as feedback the producer is diagnosed for.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct Contacted {
    pub(super) consequence: Consequence,
    pub(super) own_expression: bool,
}

/// Why an admitted write is refused whole, before any Contact it covers
/// changes.
#[derive(Clone, Copy, Debug)]
pub(super) enum Refusal {
    /// A Function written at a contacted anchor differs from the running
    /// Function in a fact the schedule was ordered by.
    Replacement(ReplacementChange),
    /// A write the schedule ordered first reaches a computation that has
    /// taken its Turn, so the schedule ordered it wrongly.
    OrderingDefect,
}

/// What one locking Function's lock means in a [`Phase`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) enum Locked {
    /// The Function locks nothing: it declares no lock, or its site is
    /// outside the Grid or empty.
    Unchanged,
    /// The locker takes its Turn before every computation in the locked
    /// subtree.
    OrderAfter(ComputationRange),
    /// The locked subtree is suppressed and its root locked for the Tick.
    Lock(ComputationRange),
    /// The locked root has taken its Turn, so a lock whose place the Tick
    /// decided misses it.
    Missed(ComputationId),
    /// A lock the schedule ordered first finds its subtree has taken a Turn,
    /// so the schedule ordered it wrongly.
    OrderingDefect,
    /// The site holds something other than an Expression root.
    NotARoot,
}

///
/// Every Dependency `lookup`'s computations hold before the Tick: a nested
/// Function before the Function it answers, and each active root's
/// Reservations, locks, Bangs and Input Portals.
///
pub(super) fn derive(lookup: &Lookup, locks: &Locks) -> Derived {
    let nodes = lookup.nodes();
    let active = active_roots(lookup);
    let mut dependencies = BTreeSet::new();
    for (index, node) in nodes.iter_enumerated() {
        if let Some(parent) = node.parent() {
            dependencies.insert((index, parent));
        }
        if active[node.owner()] {
            producer_edges(lookup, locks, index, |producer, consumer| {
                dependencies.insert((producer, consumer));
            });
        }
    }
    for consumer in nodes.ids() {
        input_edges(lookup, &active, consumer, |producer, consumer| {
            dependencies.insert((producer, consumer));
        });
    }
    let writers = dynamic_writers(lookup, &active).collect();
    Derived {
        active,
        dependencies,
        writers,
    }
}

///
/// Marks active every root `roots`, already marked in `active`, can deliver
/// activation to, and answers the Dependencies and dynamic writers of every
/// root reached, `roots` among them: a root activated during the Tick orders
/// what its own Portals reach, as an active root's do before it, and its
/// dynamic writers go writers first.
///
pub(super) fn join(
    lookup: &Lookup,
    locks: &Locks,
    active: &mut PerComputation<bool>,
    roots: Vec<ComputationId>,
) -> Joined {
    let mut joined = Joined::default();
    for root in activate(lookup, active, roots) {
        for index in lookup.descendants(root) {
            let mut edge = |producer, consumer| joined.edges.push((producer, consumer));
            producer_edges(lookup, locks, index, &mut edge);
            reader_edges(lookup, index, &mut edge);
            if matches!(lookup.nodes()[index].function.output(), Output::Dynamic(_)) {
                joined.writers.push(index);
            }
        }
    }
    joined
}

/// [`join()`] over one Lookup and its locks, as ordering asks it.
pub(super) struct Joining<'a> {
    pub(super) lookup: &'a Lookup,
    pub(super) locks: &'a Locks,
}

impl Join for Joining<'_> {
    fn join(&self, active: &mut PerComputation<bool>, roots: Vec<ComputationId>) -> Joined {
        join(self.lookup, self.locks, active, roots)
    }
}

///
/// Which roots this Tick must order effects for: the ones that take a Turn
/// with nothing delivered to them, closed over every root those can deliver
/// activation to.
///
/// An inactive root contributes no Portal, so a schedule that left one out
/// would order nothing against the writes it turns out to make. The seed is
/// exact — a root declares its activation source and nothing else decides
/// it — and the closure is deliberately wider than the Tick: which
/// root a delivery actually reaches depends on values and on working Source
/// that no schedule has yet, so execution checks activation again once those
/// producers have settled.
///
/// One declaration delivers: a Function that can return Bang delivers through
/// the Bang it writes, to the cardinal anchors a Source-resident Bang
/// activates. The Bang Function declares it, and delivers to the anchors
/// aligned with its own Span. A blocked Self-Banging Function delivers
/// nothing in its Turn; the `**` it leaves is a Bang Function on the next
/// Tick.
///
fn active_roots(lookup: &Lookup) -> PerComputation<bool> {
    let nodes = lookup.nodes();
    let mut active: PerComputation<_> = nodes
        .iter()
        .map(|node| node.is_root() && node.function.is_intrinsically_active())
        .collect();
    let pending: Vec<_> = active
        .iter_enumerated()
        .filter_map(|(index, active)| active.then_some(index))
        .collect();
    activate(lookup, &mut active, pending);
    active
}

///
/// Marks active every root the roots in `pending`, already marked in
/// `active`, can deliver activation to, and every root those can, as
/// [`active_roots`] describes.
///
/// Answers every root it reached, those in `pending` among them.
///
fn activate(
    lookup: &Lookup,
    active: &mut PerComputation<bool>,
    mut pending: Vec<ComputationId>,
) -> Vec<ComputationId> {
    let nodes = lookup.nodes();
    let mut reached = Vec::new();
    while let Some(owner) = pending.pop() {
        reached.push(owner);
        for index in lookup.descendants(owner) {
            // Only a Function that can return Bang delivers, which `bang_root`
            // also answers; asking first spares every other Function's
            // Reservations.
            if !nodes[index].function.can_emit_bang() {
                continue;
            }
            for relationships in reservations(lookup, index) {
                for root in relationships.bang_roots().collect::<Vec<_>>() {
                    let delivers = matches!(
                        bang_root(lookup, index, root, Phase::Schedule),
                        Consequence::OrderAfter(_)
                    );
                    if delivers && !active[root] {
                        active[root] = true;
                        pending.push(root);
                    }
                }
            }
        }
    }
    reached
}

///
/// What `locker`'s lock means in `phase`.
///
/// Before the Tick a lock orders the locker before the whole subtree it
/// locks. During its Turn a lock that finds that subtree untouched
/// suppresses it and locks its root for the Tick. One the schedule held
/// active was placed ahead of that root, so finding it taken is an ordering
/// defect; one the Tick joined took its place during the Tick, and a lock
/// lasts only for its Tick, so it misses a root that has taken its Turn.
///
pub(super) fn lock_verdict(locks: &Locks, locker: ComputationId, phase: Phase<'_>) -> Locked {
    match (locks.lock_target(locker), phase) {
        (None, _) | (Some(Lock::NotARoot), Phase::Schedule) => Locked::Unchanged,
        (Some(Lock::NotARoot), Phase::Turn { .. }) => Locked::NotARoot,
        (Some(Lock::Root(target)), Phase::Schedule) => Locked::OrderAfter(target.clone()),
        (
            Some(Lock::Root(target)),
            Phase::Turn {
                delivery, turns, ..
            },
        ) => {
            if !target.clone().any(|descendant| turns.attempted(descendant)) {
                Locked::Lock(target.clone())
            } else if delivery.feeds_back() {
                Locked::Missed(target.start())
            } else {
                Locked::OrderingDefect
            }
        }
    }
}

/// Whether `locker` is a locking root whose lock covers `producer`.
///
/// A planned write or Bang from that producer onto the locker is not a
/// second Dependency: the lock orders the locker first. A lock that holds
/// withholds the producer's Turn; one that does not, because nothing activated
/// the locker in time, leaves the producer to take its Turn after it, and a
/// Bang that producer delivers to the locker is missed.
fn lock_covers(
    lookup: &Lookup,
    locks: &Locks,
    locker: ComputationId,
    producer: ComputationId,
) -> bool {
    lookup.nodes()[locker].is_root()
        && matches!(
            lock_verdict(locks, locker, Phase::Schedule),
            Locked::OrderAfter(target) if target.contains(producer)
        )
}

///
/// The dynamic writers among the computations `active` roots own.
///
/// A dynamic writer's destination is known only at its Turn, so it reserves
/// nothing and is ordered writers first instead.
///
fn dynamic_writers<'a>(
    lookup: &'a Lookup,
    active: &'a PerComputation<bool>,
) -> impl Iterator<Item = ComputationId> + 'a {
    lookup
        .nodes()
        .iter_enumerated()
        .filter(|(_, node)| {
            active[node.owner()] && matches!(node.function.output(), Output::Dynamic(_))
        })
        .map(|(index, _)| index)
}

///
/// Each `(producer, consumer)` Dependency `producer`'s own Portals give it, once
/// its root is active: a lock before the subtree it locks, and each
/// reservation before every computation whose Cells it covers and every
/// root its Bang activates.
///
fn producer_edges(
    lookup: &Lookup,
    locks: &Locks,
    index: ComputationId,
    mut edge: impl FnMut(ComputationId, ComputationId),
) {
    let bundle = match lookup.nodes()[index].function.output() {
        Output::Source(effect) => Some(effect.bundle),
        _ => None,
    };
    // A self-edge is an unsatisfiable indegree, so it is how a computation
    // that writes over its own Cells reports itself as a same-Tick cycle: the
    // reservation covering the producer and the Cell pair its write reaches
    // are the same fact, and `live_cycles_stop_only_the_expressions_they_reach`
    // holds that rule.
    //
    // The one way a producer's own Cells are not a defect is one a declaration
    // states outright: a bundle that clears its own Span, which a Self-Banging
    // Function's and the Bang Function's do. Ordering such a producer after
    // itself would stop it every Tick it takes a Turn in.
    let exempt_from_itself = bundle.is_some_and(SourceBundle::clears_own_span);
    if let Locked::OrderAfter(target) = lock_verdict(locks, index, Phase::Schedule) {
        for consumer in target {
            edge(index, consumer);
        }
    }
    for relationships in reservations(lookup, index) {
        let mut order_after = |consumer: ComputationId| {
            if exempt_from_itself && consumer == index {
                return;
            }
            if lock_covers(lookup, locks, consumer, index) {
                return;
            }
            edge(index, consumer);
        };
        for contacted in relationships.functions() {
            let contacted = contact(lookup, index, &contacted, Phase::Schedule);
            if let Consequence::OrderAfter(consumers) = contacted.consequence {
                consumers.for_each(&mut order_after);
            }
        }
        for consumer in relationships.literal_consumers() {
            order_after(consumer);
        }
        for root in relationships.bang_roots() {
            if let Consequence::OrderAfter(consumers) =
                bang_root(lookup, index, root, Phase::Schedule)
            {
                consumers.for_each(&mut order_after);
            }
        }
    }
}

///
/// What `contacted`, a Contact of `producer`'s Reservation or admitted
/// write, means in `phase`.
///
/// Before the Tick a Contact orders its producer before the whole contacted
/// subtree, unless the producer's Source Bundle writes displaced. An emission
/// or a move is admitted only into Cells empty at its Turn, so it never
/// writes over a Function its reservation contacts, and it delivers no
/// activation there: occupancy alone orders nothing (ADR 0060 rule 3b). An
/// occupant that leaves first makes room; one still standing refuses the
/// placement, and a refused move leaves `**` in its own Span instead. Its
/// reservation orders none of the Functions it contacts, whether they wait
/// for a Bang or not, so Source order decides between them.
///
/// During a Turn, what has attempted its Turn keeps it and what is already
/// suppressed stays so. Otherwise a Function written at the contacted anchor
/// replaces the one running there, and anything else written over a
/// Function suppresses its whole subtree: its spelling is no longer the one
/// that was scheduled. A static writer whose Reservation covers its own
/// Expression is a same-Tick cycle and never takes its Turn. A dynamic
/// writer's destination is known only at its Turn, so a write that reaches
/// a Function of its own Expression still to come is applied as feedback and
/// diagnosed.
///
pub(super) fn contact(
    lookup: &Lookup,
    producer: ComputationId,
    contacted: &FunctionContact,
    phase: Phase<'_>,
) -> Contacted {
    let nodes = lookup.nodes();
    let target = contacted.index;
    let consequence = match phase {
        Phase::Schedule => {
            let displaced = matches!(
                nodes[producer].function.output(),
                Output::Source(effect) if effect.bundle.writes_displaced()
            );
            if displaced {
                Consequence::Unchanged
            } else {
                Consequence::OrderAfter(contacted.subtree())
            }
        }
        Phase::Turn { turns, .. } if turns.attempted(target) || turns.suppressed(target) => {
            Consequence::Unchanged
        }
        Phase::Turn { written, .. } => match written {
            Some(Atom::Function(replacement)) if contacted.at_anchor => {
                Consequence::Replace(replacement)
            }
            _ => Consequence::Suppress(contacted.subtree()),
        },
    };
    let own_expression = matches!(
        phase,
        Phase::Turn {
            delivery: Delivery::Selected,
            ..
        }
    ) && consequence != Consequence::Unchanged
        && nodes[target].owner() == nodes[producer].owner();
    Contacted {
        consequence,
        own_expression,
    }
}

///
/// What `root`, a root aligned with a Bang `producer` may write, means in
/// `phase`.
///
/// Only a Function that can return Bang delivers activation through the
/// Cells it writes. Before the Tick its Reservation orders it before every
/// root waiting for that Bang, whatever its operands turn out to answer. A
/// root that is intrinsically active waits for no Bang, so nothing orders it.
/// During a Turn a written Bang activates every root it aligns with, and one
/// that has taken its Turn without it misses it.
///
pub(super) fn bang_root(
    lookup: &Lookup,
    producer: ComputationId,
    root: ComputationId,
    phase: Phase<'_>,
) -> Consequence {
    let nodes = lookup.nodes();
    match phase {
        Phase::Schedule
            if nodes[producer].function.can_emit_bang()
                && !nodes[root].function.is_intrinsically_active() =>
        {
            Consequence::OrderAfter(lookup.descendants(root))
        }
        Phase::Turn {
            written: Some(Atom::Bang),
            ..
        } => Consequence::Activate(root),
        _ => Consequence::Unchanged,
    }
}

///
/// Why an admitted write from `producer` carrying `written` is refused whole
/// during a Turn, if it is.
///
/// A Function written at a contacted anchor still to take its Turn must
/// agree with the Function running there on every fact the schedule was
/// ordered by. A [`Delivery::Reserved`] write was ordered before every
/// computation whose Cells it reaches, so reaching one that has attempted
/// its Turn is an ordering defect: only this write is refused, and every
/// other Turn of the Tick still takes place. A write whose place was decided
/// during the Tick lands on such a computation as feedback instead.
///
pub(super) fn refusal(
    relationships: &PortalRelationships<'_>,
    written: Option<Atom>,
    delivery: Delivery,
    turns: &dyn TurnFacts,
) -> Option<Refusal> {
    if let Some(Atom::Function(replacement)) = written
        && let Some(change) = relationships.functions().find_map(|contacted| {
            if !contacted.at_anchor || (delivery.feeds_back() && turns.attempted(contacted.index)) {
                return None;
            }
            // The Function the contacted computation is running, which is the
            // one a replacement replaces: a second replacement reaching one
            // anchor in a Tick compares against the first.
            replacement.replacing(turns.running(contacted.index))
        })
    {
        return Some(Refusal::Replacement(change));
    }
    let reaches_attempted = relationships.functions().any(|contacted| {
        contacted
            .subtree
            .clone()
            .any(|descendant| turns.attempted(descendant))
    });
    (!delivery.feeds_back() && reaches_attempted).then_some(Refusal::OrderingDefect)
}

///
/// Each `(producer, consumer)` Dependency a static Input Portal gives `producer`,
/// once its root is active: one to every computation that reads Cells its
/// reservation covers. These are the Dependencies [`input_edges`] gives, found from
/// the writer's side.
///
fn reader_edges(
    lookup: &Lookup,
    producer: ComputationId,
    mut edge: impl FnMut(ComputationId, ComputationId),
) {
    for relationships in reservations(lookup, producer) {
        for consumer in lookup.reads.touching(relationships.cells.clone()) {
            if consumer != producer {
                edge(producer, consumer);
            }
        }
    }
}

///
/// Each `(producer, consumer)` Dependency `consumer`'s static Input Portal gives
/// it: one from every writer `active` roots own whose reservation covers the
/// Cells it reads.
///
fn input_edges(
    lookup: &Lookup,
    active: &PerComputation<bool>,
    consumer: ComputationId,
    mut edge: impl FnMut(ComputationId, ComputationId),
) {
    // An Input Portal reads working Source. Those Cells are often an operand
    // Span, so they cannot sit in `literals`; the Dependency is the same fact
    // `literal_consumers` records for a declared operand.
    for read in lookup.nodes()[consumer].portal_access.read_spans() {
        for producer in input_writers(lookup, active, consumer, read.clone()) {
            edge(producer, consumer);
        }
    }
}

///
/// The computations an Input Portal reading `read` is ordered after: every
/// writer whose reservation covers those Cells, other than `reader` itself
/// and those activation cannot reach this Tick. A Portal declared before the
/// Tick and one found at a Turn are ordered by this one rule.
///
pub(super) fn input_writers<'a>(
    lookup: &'a Lookup,
    active: &'a PerComputation<bool>,
    reader: ComputationId,
    read: Range<usize>,
) -> impl Iterator<Item = ComputationId> + 'a {
    lookup
        .writes
        .touching(read)
        .filter(move |&writer| writer != reader && active[lookup.nodes()[writer].owner()])
}

///
/// What one Halt's lock reaches in the Source Snapshot, before any Turn can
/// change it.
///
pub(super) enum Lock {
    /// The complete subtree of the Expression root at the lock site, which
    /// Parser preorder makes contiguous.
    Root(ComputationRange),
    /// A site something other than an Expression root occupies.
    NotARoot,
}

///
/// The [`Lock`] each computation's Portal reaches, shared by scheduling and
/// execution. Only execution decides whether an active, unsuppressed Halt
/// actually applies it.
///
pub(super) struct Locks(PerComputation<Option<Lock>>);

impl Locks {
    pub(super) fn new(lookup: &Lookup, inputs: ScheduleInputs<'_>) -> Self {
        Self(
            lookup
                .nodes()
                .iter()
                .map(|node| {
                    // A site outside the Grid locks nothing.
                    let destination = node.portal_access.lock_site()?.ok()?;
                    if let Some(root) = lookup.root_at(destination) {
                        return Some(Lock::Root(lookup.descendants(root)));
                    }
                    // A site no root covers is judged by its Snapshot
                    // occupancy, and an empty one locks nothing.
                    // `Occupancy` is stated outside the schedule, so it names
                    // a root by its bare index.
                    let root_at = |anchor| lookup.root_at(anchor).map(ComputationId::index);
                    let portal = Portal::at(lookup.grid, destination);
                    (!matches!(portal.occupancy(inputs.units(), root_at), Occupancy::Empty))
                        .then_some(Lock::NotARoot)
                })
                .collect(),
        )
    }

    ///
    /// What `index`'s lock reaches: nothing for a computation that declares
    /// no lock, or one whose site is outside the Grid or empty.
    ///
    pub(super) fn lock_target(&self, index: ComputationId) -> Option<&Lock> {
        self.0[index].as_ref()
    }
}

///
/// The Functions and Bang roots the Cells an admitted write actually covers
/// reach.
///
/// A value answer renders as one Atom's Cell pair, which is exactly the
/// Reservation its schedule ordered, so every contact execution acts on is
/// one a Dependency already names.
///
pub(super) fn reached<'a>(lookup: &'a Lookup, write: &SpanWrite) -> PortalRelationships<'a> {
    let span = write.span();
    PortalRelationships {
        lookup,
        output: lookup.grid.position_at(span.start()),
        cells: span.range(),
    }
}

/// A fixed destination has relationships only if the Cell pair reserved
/// from it fits the row ([`Portal::reservation`]). Actual writes still go
/// through `Portal::admit`, which also validates their encoding and
/// supplies the producer's diagnostic.
fn reserved_at(lookup: &Lookup, output: Position) -> Option<PortalRelationships<'_>> {
    let cells = Portal::at(lookup.grid, output).reservation()?.range();
    Some(PortalRelationships {
        lookup,
        output,
        cells,
    })
}

/// The relationships of every Reservation `index`'s write Portals make,
/// skipping a Portal whose destination is refused or does not fit the row.
fn reservations(
    lookup: &Lookup,
    index: ComputationId,
) -> impl Iterator<Item = PortalRelationships<'_>> {
    lookup.nodes()[index]
        .portal_access
        .write_sites()
        .iter()
        .filter_map(|output| output.as_ref().ok())
        .filter_map(move |output| reserved_at(lookup, *output))
}

/// Relationships of one fixed Portal destination, and of the Cells following it
/// along its row, to the original computations. A Portal names where a result
/// begins rather than how wide it is, so how many Cells to ask about is the
/// caller's to state: scheduling asks over the Cells it reserved and execution
/// asks over the Cells an admitted write actually covers. What each fact means
/// in either phase is [`contact`]'s and [`bang_root`]'s to answer.
pub(super) struct PortalRelationships<'a> {
    lookup: &'a Lookup,
    output: Position,
    cells: Range<usize>,
}

/// A Function whose Cells a write or Reservation covers.
pub(super) struct FunctionContact {
    index: ComputationId,
    at_anchor: bool,
    /// Includes the contacted Function itself, in Parser preorder.
    subtree: ComputationRange,
}

impl PortalRelationships<'_> {
    pub(super) fn functions(&self) -> impl Iterator<Item = FunctionContact> + '_ {
        self.lookup
            .functions
            .touching(self.cells.clone())
            .map(|index| FunctionContact {
                index,
                at_anchor: self.lookup.nodes()[index].anchor == self.output,
                subtree: self.lookup.descendants(index),
            })
    }

    fn literal_consumers(&self) -> impl Iterator<Item = ComputationId> + '_ {
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
    /// "The whole destination" is the Cell pair a Bang occupies: alignment is
    /// a fact about a Bang's own two Cells.
    ///
    /// The four anchors are one geometric fact and are stated as one. The west
    /// arm answers a root only for a Function that declares no operand: a root
    /// anchored two columns west with an operand claims this destination for
    /// that operand, so a Bang landing here is operand contact rather than an
    /// anchor — the `(6, 2)` row of
    /// `fixed_bang_destinations_respect_alignment_and_operand_contact` asserts
    /// exactly that silence.
    ///
    /// A Self-Banging Function `^^ vv << >>` answers the arm with a root and
    /// changes nothing by it: those four take their Turn without a Bang, so
    /// scheduling orders nothing after an intrinsically active root, and a
    /// Bang that reaches one, before its Turn or after it, neither starts nor
    /// misses anything. The Directional Bang Functions are where it decides a
    /// Tick: `*^`
    /// declares no operand and waits for activation, so a Bang two columns east
    /// of one is the only thing that starts it.
    /// `an_active_directional_bang_function_emits_its_self_banging_function`
    /// drives that arm, and deleting it leaves the north and west halves of
    /// that test emitting nothing.
    ///
    /// The asymmetry the arm states is the language's and not this function's:
    /// a horizontally aligned root anchors two columns away because every
    /// Language Unit spells as a Cell pair.
    pub(super) fn bang_roots(&self) -> impl Iterator<Item = ComputationId> + '_ {
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

impl FunctionContact {
    pub(super) fn index(&self) -> ComputationId {
        self.index
    }

    /// The contacted Function and every computation nested in it, in Parser
    /// preorder.
    pub(super) fn subtree(&self) -> ComputationRange {
        self.subtree.clone()
    }
}

#[cfg(test)]
mod test {
    use lang::{Atom, Function, ReplacementChange};

    use super::{
        Consequence, Delivery, FunctionContact, Locked, Locks, Phase, Refusal, TurnFacts,
        bang_root, contact, derive, join, lock_verdict, refusal, reservations,
    };
    use crate::grid::Grid;
    use crate::source::Cells;
    use crate::source::language_map::LanguageMap;
    use crate::source::tick::{ComputationId, Lookup, PerComputation, computations};

    /// The Lookup for `rows`, each padded to the Grid's width, and its locks.
    fn fixture(grid: Grid, rows: &[&str]) -> (Lookup, Locks) {
        let bytes: String = (0..grid.rows())
            .map(|row| {
                format!(
                    "{:width$}",
                    rows.get(row).unwrap_or(&""),
                    width = grid.columns()
                )
            })
            .collect();
        let map = LanguageMap::build(grid, Cells::of(bytes.as_bytes()));
        let inputs = map.schedule_inputs();
        let (nodes, _) = computations(grid, inputs);
        let lookup = Lookup::new(grid, nodes);
        let locks = Locks::new(&lookup, inputs);
        (lookup, locks)
    }

    /// The computation anchored at `(x, y)`.
    fn at(lookup: &Lookup, x: usize, y: usize) -> ComputationId {
        lookup
            .nodes()
            .iter()
            .find(|node| (node.anchor.x(), node.anchor.y()) == (x, y))
            .map(|node| node.id)
            .unwrap_or_else(|| panic!("no computation anchored at ({x}, {y})"))
    }

    // Equality Bangs every Tick two columns west of `*!`, which activates
    // Halt, and Halt locks the Add one row south of it.
    const HALT: [&str; 3] = [".=0101  ", "  *!    ", "  .+0304"];

    #[test]
    fn a_lock_goes_before_the_subtree_it_locks() {
        let (lookup, locks) = fixture(Grid::with_shape(8, 3), &HALT);
        let (halt, add) = (at(&lookup, 2, 1), at(&lookup, 2, 2));
        assert!(derive(&lookup, &locks).dependencies.contains(&(halt, add)));
    }

    #[test]
    fn a_bang_goes_before_the_root_it_activates() {
        let (lookup, locks) = fixture(Grid::with_shape(8, 3), &HALT);
        let (equality, halt) = (at(&lookup, 0, 0), at(&lookup, 2, 1));
        let derived = derive(&lookup, &locks);
        assert!(
            derived.active[halt],
            "the Bang reaches Halt before the Tick"
        );
        assert!(derived.dependencies.contains(&(equality, halt)));
    }

    #[test]
    fn a_locked_producer_is_not_ordered_before_the_lock_it_reaches() {
        // `^^` reserves the Cells Halt stands in, but Halt locks it, so the
        // lock alone orders the pair.
        let (lookup, locks) = fixture(
            Grid::with_shape(8, 3),
            &[".=0101  ", "  *!    ", "  ^^    "],
        );
        let (halt, mover) = (at(&lookup, 2, 1), at(&lookup, 2, 2));
        let dependencies = derive(&lookup, &locks).dependencies;
        assert!(dependencies.contains(&(halt, mover)));
        assert!(!dependencies.contains(&(mover, halt)));
    }

    #[test]
    fn a_bundle_that_clears_its_own_span_is_not_ordered_after_itself() {
        // The Bang Function reserves the Cells it stands in and does not
        // move, so only the exemption keeps it from being its own cycle.
        let (lookup, locks) = fixture(Grid::with_shape(6, 1), &["**    "]);
        let bang = at(&lookup, 0, 0);
        let derived = derive(&lookup, &locks);
        assert!(derived.active[bang]);
        assert!(!derived.dependencies.contains(&(bang, bang)));
    }

    #[test]
    fn a_reservation_goes_before_the_function_it_contacts() {
        let (lookup, locks) = fixture(Grid::with_shape(6, 2), &[".+0101", ".-0202"]);
        let (writer, contacted) = (at(&lookup, 0, 0), at(&lookup, 0, 1));
        assert!(
            derive(&lookup, &locks)
                .dependencies
                .contains(&(writer, contacted))
        );
    }

    #[test]
    fn a_mover_meets_every_root_it_contacts_in_source_order() {
        // `>>` reserves the Cells the root east of it stands in, whether that
        // root takes its Turn with nothing delivered to it, as `^^` does, or
        // waits for a Bang, as `*!` does. Neither is ordered after the mover.
        for rows in [[">>^^  "], [">>*!  "]] {
            let (lookup, locks) = fixture(Grid::with_shape(6, 1), &rows);
            let (mover, contacted) = (at(&lookup, 0, 0), at(&lookup, 2, 0));
            assert!(
                !derive(&lookup, &locks)
                    .dependencies
                    .contains(&(mover, contacted)),
                "{rows:?}"
            );
        }
    }

    #[test]
    fn a_reservation_goes_before_the_literal_it_covers() {
        let (lookup, locks) = fixture(Grid::with_shape(10, 2), &["    .+0101", "  .-0101  "]);
        let (writer, consumer) = (at(&lookup, 4, 0), at(&lookup, 2, 1));
        assert!(
            derive(&lookup, &locks)
                .dependencies
                .contains(&(writer, consumer))
        );
    }

    #[test]
    fn a_joining_root_brings_what_its_bang_activates() {
        let (lookup, locks) = fixture(Grid::with_shape(8, 3), &HALT);
        let (equality, halt, add) = (at(&lookup, 0, 0), at(&lookup, 2, 1), at(&lookup, 2, 2));
        let mut active = PerComputation::filled(lookup.nodes().len(), false);
        active[equality] = true;
        let joined = join(&lookup, &locks, &mut active, vec![equality]);
        assert!(active[halt], "the Bang reaches Halt");
        assert!(joined.edges.contains(&(equality, halt)));
        assert!(joined.edges.contains(&(halt, add)));
        assert!(joined.writers.is_empty());
    }

    #[test]
    fn a_lock_activates_nothing() {
        // The upper Halt locks the lower one, which waits for a Bang that
        // nothing delivers, so only activation could make it active.
        let (lookup, locks) = fixture(
            Grid::with_shape(8, 3),
            &[".=0101  ", "  *!    ", "  *!    "],
        );
        let (upper, lower) = (at(&lookup, 2, 1), at(&lookup, 2, 2));
        let derived = derive(&lookup, &locks);
        assert!(derived.active[upper], "the Bang reaches the upper Halt");
        assert!(!derived.active[lower], "a lock activates nothing");
        assert!(derived.dependencies.contains(&(upper, lower)));
    }

    /// What the Turns taken so far have done, stated by a test rather than
    /// taken: every computation runs its parsed Function until `running`
    /// names another.
    struct Stated {
        attempted: Vec<ComputationId>,
        suppressed: Vec<ComputationId>,
        running: Vec<(ComputationId, Function)>,
        parsed: PerComputation<Function>,
    }

    impl Stated {
        fn new(lookup: &Lookup) -> Self {
            Self {
                attempted: vec![],
                suppressed: vec![],
                running: vec![],
                parsed: lookup.nodes().iter().map(|node| node.function).collect(),
            }
        }
    }

    impl TurnFacts for Stated {
        fn attempted(&self, index: ComputationId) -> bool {
            self.attempted.contains(&index)
        }

        fn suppressed(&self, index: ComputationId) -> bool {
            self.suppressed.contains(&index)
        }

        fn running(&self, index: ComputationId) -> Function {
            self.running
                .iter()
                .find_map(|&(running, function)| (running == index).then_some(function))
                .unwrap_or(self.parsed[index])
        }
    }

    /// The Contact `writer`'s first Reservation makes with `contacted`.
    fn contact_of(
        lookup: &Lookup,
        writer: ComputationId,
        contacted: ComputationId,
    ) -> FunctionContact {
        reservations(lookup, writer)
            .flat_map(|relationships| relationships.functions().collect::<Vec<_>>())
            .find(|candidate| candidate.index() == contacted)
            .unwrap_or_else(|| panic!("{writer:?} reserves no Cells of {contacted:?}"))
    }

    fn number(value: u8) -> Atom {
        Atom::Number(value)
    }

    // `.+` reserves one row south of itself, where `.-` is anchored.
    const STACKED: [&str; 2] = [".+0101", ".-0202"];

    #[test]
    fn each_reach_rule_answers_its_consequence_in_each_phase() {
        let grid = Grid::with_shape(10, 2);
        let (lookup, _) = fixture(grid, &STACKED);
        let (writer, below) = (at(&lookup, 0, 0), at(&lookup, 0, 1));
        let touched = contact_of(&lookup, writer, below);
        let subtree = lookup.descendants(below);
        let fresh = Stated::new(&lookup);
        let turn = |delivery, written, turns: &'_ Stated| -> Consequence {
            contact(
                &lookup,
                writer,
                &touched,
                Phase::Turn {
                    delivery,
                    written: Some(written),
                    turns,
                },
            )
            .consequence
        };

        // Before the Tick a Contact orders its producer before the subtree.
        assert_eq!(
            contact(&lookup, writer, &touched, Phase::Schedule).consequence,
            Consequence::OrderAfter(subtree.clone())
        );
        // A value written over a Function suppresses its subtree.
        assert_eq!(
            turn(Delivery::Reserved, number(3), &fresh),
            Consequence::Suppress(subtree.clone())
        );
        // A Function written at its anchor replaces the running one.
        assert_eq!(
            turn(
                Delivery::Reserved,
                Atom::Function(Function::Multiply),
                &fresh
            ),
            Consequence::Replace(Function::Multiply)
        );
        // What has attempted its Turn keeps it, and what is suppressed stays so.
        let mut taken = Stated::new(&lookup);
        taken.attempted.push(below);
        assert_eq!(
            turn(Delivery::Joined, number(3), &taken),
            Consequence::Unchanged
        );
        let mut covered = Stated::new(&lookup);
        covered.suppressed.push(below);
        assert_eq!(
            turn(Delivery::Selected, number(3), &covered),
            Consequence::Unchanged
        );
    }

    #[test]
    fn a_dynamic_write_onto_its_own_expression_is_diagnosed_as_feedback() {
        // `.-` is nested in `.+`, so a write from the root that reaches it
        // reaches the root's own Expression. Only a dynamic writer's
        // destination is unknown until its Turn.
        let (lookup, _) = fixture(Grid::with_shape(10, 2), &[".+01.-0101"]);
        let (root, child) = (at(&lookup, 0, 0), at(&lookup, 4, 0));
        assert_eq!(lookup.nodes()[child].owner(), lookup.nodes()[root].owner());
        let touched = FunctionContact {
            index: child,
            at_anchor: false,
            subtree: lookup.descendants(child),
        };
        let fresh = Stated::new(&lookup);
        let own = |delivery| {
            contact(
                &lookup,
                root,
                &touched,
                Phase::Turn {
                    delivery,
                    written: Some(number(3)),
                    turns: &fresh,
                },
            )
            .own_expression
        };
        assert!(own(Delivery::Selected));
        assert!(!own(Delivery::Reserved));
        assert!(!own(Delivery::Joined));
    }

    #[test]
    fn an_emission_orders_nothing_by_occupancy() {
        // `*>` emits into the two Cells east of it, where `.+` stands.
        let (lookup, _) = fixture(Grid::with_shape(10, 1), &["*>.+0101"]);
        let (emitter, occupant) = (at(&lookup, 0, 0), at(&lookup, 2, 0));
        let touched = contact_of(&lookup, emitter, occupant);
        assert_eq!(
            contact(&lookup, emitter, &touched, Phase::Schedule).consequence,
            Consequence::Unchanged
        );
    }

    #[test]
    fn a_mover_orders_no_root_it_contacts() {
        for row in [">>^^  ", ">>*!  "] {
            let (lookup, _) = fixture(Grid::with_shape(6, 1), &[row]);
            let (mover, contacted) = (at(&lookup, 0, 0), at(&lookup, 2, 0));
            let touched = contact_of(&lookup, mover, contacted);
            assert_eq!(
                contact(&lookup, mover, &touched, Phase::Schedule).consequence,
                Consequence::Unchanged,
                "{row:?}"
            );
        }
    }

    #[test]
    fn a_bang_orders_and_then_activates_the_root_it_aligns_with() {
        let (lookup, _) = fixture(Grid::with_shape(8, 3), &HALT);
        let (equality, halt) = (at(&lookup, 0, 0), at(&lookup, 2, 1));
        let fresh = Stated::new(&lookup);
        let written = |written| {
            bang_root(
                &lookup,
                equality,
                halt,
                Phase::Turn {
                    delivery: Delivery::Reserved,
                    written: Some(written),
                    turns: &fresh,
                },
            )
        };
        assert_eq!(
            bang_root(&lookup, equality, halt, Phase::Schedule),
            Consequence::OrderAfter(lookup.descendants(halt))
        );
        assert_eq!(written(Atom::Bang), Consequence::Activate(halt));
        assert_eq!(written(number(1)), Consequence::Unchanged);
        // A producer that cannot answer Bang delivers no activation.
        let add = at(&lookup, 2, 2);
        assert_eq!(
            bang_root(&lookup, add, halt, Phase::Schedule),
            Consequence::Unchanged
        );
    }

    #[test]
    fn a_reserved_write_onto_an_attempted_computation_is_an_ordering_defect() {
        let (lookup, _) = fixture(Grid::with_shape(10, 2), &STACKED);
        let (writer, below) = (at(&lookup, 0, 0), at(&lookup, 0, 1));
        let relationships = reservations(&lookup, writer)
            .next()
            .expect("`.+` reserves one row south");
        let mut taken = Stated::new(&lookup);
        taken.attempted.push(below);
        assert!(matches!(
            refusal(&relationships, Some(number(3)), Delivery::Reserved, &taken),
            Some(Refusal::OrderingDefect)
        ));
        // A write whose place the Tick decided lands there as feedback.
        for delivery in [Delivery::Joined, Delivery::Selected] {
            assert!(refusal(&relationships, Some(number(3)), delivery, &taken).is_none());
        }
        assert!(
            refusal(
                &relationships,
                Some(number(3)),
                Delivery::Reserved,
                &Stated::new(&lookup)
            )
            .is_none()
        );
    }

    #[test]
    fn a_replacement_that_changes_a_scheduled_fact_is_refused() {
        let (lookup, _) = fixture(Grid::with_shape(10, 2), &STACKED);
        let writer = at(&lookup, 0, 0);
        let relationships = reservations(&lookup, writer)
            .next()
            .expect("`.+` reserves one row south");
        let fresh = Stated::new(&lookup);
        // Halt answers no value, which the schedule ordered `.-` by.
        assert!(matches!(
            refusal(
                &relationships,
                Some(Atom::Function(Function::Halt)),
                Delivery::Reserved,
                &fresh
            ),
            Some(Refusal::Replacement(ReplacementChange::AnswerKind))
        ));
        assert!(
            refusal(
                &relationships,
                Some(Atom::Function(Function::Multiply)),
                Delivery::Reserved,
                &fresh
            )
            .is_none()
        );
    }

    #[test]
    fn a_lock_orders_then_locks_and_a_late_one_misses_or_is_refused() {
        let (lookup, locks) = fixture(Grid::with_shape(8, 3), &HALT);
        let (halt, add) = (at(&lookup, 2, 1), at(&lookup, 2, 2));
        let subtree = lookup.descendants(add);
        let turn = |delivery, turns: &'_ Stated| {
            lock_verdict(
                &locks,
                halt,
                Phase::Turn {
                    delivery,
                    written: None,
                    turns,
                },
            )
        };
        assert_eq!(
            lock_verdict(&locks, halt, Phase::Schedule),
            Locked::OrderAfter(subtree.clone())
        );
        assert_eq!(
            turn(Delivery::Reserved, &Stated::new(&lookup)),
            Locked::Lock(subtree)
        );
        let mut taken = Stated::new(&lookup);
        taken.attempted.push(add);
        assert_eq!(turn(Delivery::Reserved, &taken), Locked::OrderingDefect);
        assert_eq!(turn(Delivery::Joined, &taken), Locked::Missed(add));
        // A Function that declares no lock locks nothing.
        let equality = at(&lookup, 0, 0);
        assert_eq!(
            lock_verdict(&locks, equality, Phase::Schedule),
            Locked::Unchanged
        );
    }

    #[test]
    fn a_lock_on_something_other_than_a_root_is_diagnosed_only_at_its_turn() {
        // Halt's site holds an operand of `.+`, not an Expression root.
        let (lookup, locks) = fixture(
            Grid::with_shape(8, 3),
            &[".=0101  ", "  *!    ", ".+0101  "],
        );
        let halt = at(&lookup, 2, 1);
        assert_eq!(
            lock_verdict(&locks, halt, Phase::Schedule),
            Locked::Unchanged
        );
        assert_eq!(
            lock_verdict(
                &locks,
                halt,
                Phase::Turn {
                    delivery: Delivery::Reserved,
                    written: None,
                    turns: &Stated::new(&lookup),
                },
            ),
            Locked::NotARoot
        );
    }
}
