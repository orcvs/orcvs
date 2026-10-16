//! The order a Tick's Turns are taken in, from the schedule built before the
//! Tick to the last Turn of the Tick.
//!
//! A schedule orders every dependency known before the Tick, Grid position
//! breaking ties, and stops what a cycle reaches. A Tick takes its Turns in
//! that order until a Turn finds a writer it must wait for. From that Turn
//! on, the same loop that built the schedule orders the rest of the Tick, so
//! a dependency found at a Turn joins the order there and a cycle it closes
//! is diagnosed as one known before the Tick is.
//!
//! A dynamic writer's destination is known only at its Turn, so no reader
//! can wait on it. Instead every dynamic writer goes before each computation
//! that feeds neither it nor a dynamic writer ordered before it, and the
//! writers are ordered among themselves by what feeds them, Grid position
//! breaking ties. These writers-first edges follow from the dependencies:
//! a Turn that finds one more recomputes them for the rest of its Tick, so
//! a dependency a writer finds through what feeds it goes before the writer
//! rather than closing a cycle.
//!
//! The schedule is shared by every Tick planned against the same scheduling
//! inputs and is never changed by one. A Tick's progress through it is kept
//! here, and the dependency graph a Turn that waits needs is rebuilt only
//! when one does.
//!
//! Ordering sees a plain graph: nodes named by the `ComputationId` the
//! schedule gives each computation, in Parser preorder, the Dependencies
//! between them, each node's Grid position and the Expression it belongs to.
//! What a node computes, and why one goes before another, belong to the
//! caller, which answers through [`Join`] when roots join a Tick.

use std::borrow::Cow;
use std::collections::BTreeSet;

use super::{ComputationId, ComputationRange, PerComputation};

///
/// What ordering knows of each node: where it stands and which Expression it
/// belongs to.
///
pub(super) struct Nodes {
    /// Each node's Grid position as a Cell index, which breaks ties between
    /// independent nodes.
    pub(super) keys: PerComputation<usize>,
    /// Each node's root, which is the node itself for a root.
    pub(super) owners: PerComputation<ComputationId>,
    /// Each node's whole Expression: its root and every node nested in it,
    /// contiguous in Parser preorder. A cycle stops the whole Expression.
    pub(super) expressions: PerComputation<ComputationRange>,
}

impl Nodes {
    fn len(&self) -> usize {
        self.keys.len()
    }

    fn key(&self, index: ComputationId) -> (usize, ComputationId) {
        (self.keys[index], index)
    }
}

///
/// The Dependencies and dynamic writers roots joining a Tick bring, which the
/// caller derives and ordering only orders.
///
#[derive(Default)]
pub(super) struct Joined {
    pub(super) edges: Vec<(ComputationId, ComputationId)>,
    pub(super) writers: Vec<ComputationId>,
}

///
/// How ordering learns what roots activated during a Tick bring with them.
///
pub(super) trait Join {
    /// Marks active every root `roots`, already marked in `active`, can
    /// deliver activation to, and answers the Dependencies and dynamic
    /// writers of every root reached, `roots` among them.
    fn join(&self, active: &mut PerComputation<bool>, roots: Vec<ComputationId>) -> Joined;
}

///
/// A computation no order placed, as ordering reports it for the caller to
/// diagnose.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stall {
    /// The first node of a same-Tick Dependency cycle, in Parser order.
    Cycle(ComputationId),
    /// An active root stopped only because it depends on a cycle.
    Waiting(ComputationId),
}

///
/// One revision's nodes and the order [`schedule`] gives their Turns.
/// No Tick changes it: a Tick's progress through it, and any dependency a
/// Turn finds, belong to [`take_turns`].
///
pub(super) struct Order {
    nodes: Nodes,
    pub(super) order: Vec<ComputationId>,
    /// Why the order stops what it stops, in the order it is diagnosed: each
    /// cycle by its first node, then each root stopped only because it waits
    /// on one.
    pub(super) stalls: Vec<Stall>,
    /// Every dependency edge known before the Tick, by producer, so that a
    /// Tick can continue ordering from a Turn that finds one more. The
    /// writers-first edges are not among them: they follow from these and
    /// from what a Tick finds, so each ordering derives its own.
    outgoing: PerComputation<Vec<ComputationId>>,
    /// The dynamic writers activation can reach this Tick.
    writers: Vec<ComputationId>,
    /// Which roots activation can reach this Tick, by computation.
    active: PerComputation<bool>,
    /// Which computations a same-Tick dependency cycle stops: exactly those
    /// `order` holds no Turn for.
    pub(super) stopped: PerComputation<bool>,
}

#[cfg(test)]
impl Order {
    /// Whether activation known before the Tick reaches `root`.
    pub(super) fn holds_active(&self, root: ComputationId) -> bool {
        self.active[root]
    }
}

///
/// The order of `nodes`' Turns under `edges`, as `(producer, consumer)`
/// pairs, with each of `writers`, the dynamic writers, ordered writers first:
/// the Turns in the order the edges and Grid position give them, less every
/// Expression a same-Tick dependency cycle reaches, which takes no Turn and is
/// reported as a [`Stall`]. `active` holds the roots activation reaches before
/// the Tick.
///
pub(super) fn schedule(
    nodes: Nodes,
    active: PerComputation<bool>,
    edges: BTreeSet<(ComputationId, ComputationId)>,
    writers: Vec<ComputationId>,
) -> Order {
    let everything = PerComputation::filled(nodes.len(), true);
    let mut known = Graph::new(nodes.len(), edges.iter().copied());
    let writers_first = known.writers_first(&nodes, &writers, &everything);
    // Ordering changes only `indegree`, so with no writers-first edges `known`
    // is ordered itself and still holds the edges the schedule keeps.
    let mut with_writers = (!writers_first.is_empty())
        .then(|| Graph::new(nodes.len(), edges.iter().copied().chain(writers_first)));
    let graph = with_writers.as_mut().unwrap_or(&mut known);
    let ready = graph.free(|_| true);
    let stopped = graph.cycle_closure(&nodes, &everything);
    let mut order = Vec::new();
    graph.take_ready(&nodes, ready, &stopped, |index| {
        order.push(index);
        true
    });
    let mut stalls = Vec::new();
    if order.len() != nodes.len() {
        let mut placed = PerComputation::filled(nodes.len(), false);
        for &index in &order {
            placed[index] = true;
        }
        stalls = stall(&nodes, &graph.outgoing, &placed, &stopped, &active);
    }
    let Graph { outgoing, .. } = known;
    Order {
        nodes,
        order,
        stalls,
        outgoing,
        writers,
        active,
        stopped,
    }
}

///
/// What one attempt at a Turn did.
///
pub(super) enum Turn {
    /// The Turn was taken, or settled without effect. `activated` names the
    /// roots a dynamic write in it delivered activation to, which the
    /// schedule could not know of.
    Taken { activated: Vec<ComputationId> },
    /// The Turn must wait for these writers, which
    /// [`Progress::unresolved`] names, and is taken again once they
    /// have been.
    Waits(Vec<ComputationId>),
}

///
/// Takes one Tick's Turns through `schedule`, and answers the [`Stall`]s of
/// the Turns a cycle found during the Tick leaves untaken.
///
/// `take` takes a computation's Turn and answers what it did. A Turn that
/// waits leaves no effect and is taken again once its writers have been.
/// Turns are taken in the schedule's order until one waits, or until a
/// dynamic write activates a root the schedule did not hold active. From
/// then on the rest of the Tick is ordered by [`Progress`], as one found
/// dependency orders it: the computations still to take their Turn keep
/// every edge known before the Tick, a waiting Turn gains one from each
/// writer, a root activated during the Tick joins with the edges its own
/// Portals give, the writers-first edges are derived again from those, and
/// the Turns are taken by the loop the schedule was ordered by. A Tick that
/// finds nothing therefore takes its Turns in exactly the schedule's order.
///
/// Turns still waiting when nothing is ready are on a same-Tick dependency
/// cycle or wait on one. They are diagnosed and left untaken, and every other
/// Turn of the Tick is still taken.
///
/// A root activated during the Tick brings the Dependencies `join` answers
/// for it.
///
pub(super) fn take_turns(
    schedule: &Order,
    join: &dyn Join,
    mut take: impl FnMut(ComputationId, &Progress<'_>) -> Turn,
) -> Vec<Stall> {
    let mut progress = Progress {
        schedule,
        join,
        waiting: schedule.stopped.iter().map(|stopped| !stopped).collect(),
        active: Cow::Borrowed(&schedule.active),
        joined_edges: Vec::new(),
        joined_writers: Vec::new(),
    };
    for &index in &schedule.order {
        match take(index, &progress) {
            Turn::Taken { activated } => {
                progress.waiting[index] = false;
                if progress.activate(activated) {
                    return progress.continue_ordering(Vec::new(), take);
                }
            }
            Turn::Waits(writers) => {
                let found = writers.into_iter().map(|writer| (writer, index)).collect();
                return progress.continue_ordering(found, take);
            }
        }
    }
    Vec::new()
}

///
/// How far one Tick has got through its schedule.
///
pub(super) struct Progress<'a> {
    schedule: &'a Order,
    join: &'a dyn Join,
    /// Which computations the schedule holds a Turn for that they have not
    /// yet taken.
    waiting: PerComputation<bool>,
    /// Which roots activation can reach this Tick: the schedule's, and every
    /// root a dynamic write has activated since, with what those reach.
    active: Cow<'a, PerComputation<bool>>,
    /// The edges the roots activated during the Tick give: what their own
    /// Portals reach and the static Input Portals their reservations cover.
    /// Found once, when each root joins, because what a root's Portals give
    /// does not change during the Tick.
    joined_edges: Vec<(ComputationId, ComputationId)>,
    /// The dynamic writers the roots activated during the Tick own.
    joined_writers: Vec<ComputationId>,
}

impl Progress<'_> {
    ///
    /// Which roots activation reaches this Tick so far, by node.
    ///
    pub(super) fn active(&self) -> &PerComputation<bool> {
        &self.active
    }

    ///
    /// The ones among `writers` that have not yet taken their Turn, which a
    /// reader of the Cells they write must wait for.
    ///
    /// A writer the schedule stopped never takes its Turn, so the reader
    /// waits on it and is stopped with it, as a static reader of those Cells
    /// is.
    ///
    pub(super) fn unresolved(
        &self,
        writers: impl IntoIterator<Item = ComputationId>,
    ) -> Vec<ComputationId> {
        let schedule = self.schedule;
        let mut writers: Vec<ComputationId> = writers
            .into_iter()
            .filter(|&writer| self.waiting[writer] || schedule.stopped[writer])
            .collect();
        writers.sort_unstable();
        writers.dedup();
        writers
    }

    ///
    /// Whether `index`'s root was activated during the Tick rather than held
    /// active by the schedule.
    ///
    pub(super) fn joined(&self, index: ComputationId) -> bool {
        let owner = self.schedule.nodes.owners[index];
        self.active[owner] && !self.schedule.active[owner]
    }

    ///
    /// Makes active this Tick each of `roots` the Tick does not hold active
    /// yet, and every root those can deliver activation to. Answers whether
    /// any was new.
    ///
    fn activate(&mut self, roots: Vec<ComputationId>) -> bool {
        let pending: Vec<ComputationId> = roots
            .into_iter()
            .filter(|&root| !self.active[root])
            .collect();
        if pending.is_empty() {
            return false;
        }
        let active = self.active.to_mut();
        for &root in &pending {
            active[root] = true;
        }
        // A root activated during the Tick orders what its own Portals
        // reach, as an active root's do before it, and its dynamic writers
        // go writers first.
        let joined = self.join.join(active, pending);
        self.joined_edges.extend(joined.edges);
        self.joined_writers.extend(joined.writers);
        true
    }

    /// Orders the rest of the Tick from the dependencies `found` so far, as
    /// [`take_turns`] describes.
    ///
    /// Each dependency a Turn finds, and each root a Turn activates, can
    /// change what is ordered after what, so the order is rebuilt from the
    /// computations still waiting each time either happens.
    fn continue_ordering(
        mut self,
        mut found: Vec<(ComputationId, ComputationId)>,
        mut take: impl FnMut(ComputationId, &Progress<'_>) -> Turn,
    ) -> Vec<Stall> {
        let nodes = &self.schedule.nodes;
        let graph = loop {
            let mut graph = self.graph(&found);
            let stopped = graph.cycle_closure(nodes, &self.waiting);
            let ready = graph.free(|index| self.waiting[index]);
            let reordered =
                graph.take_ready(nodes, ready, &stopped, |index| match take(index, &self) {
                    Turn::Taken { activated } => {
                        self.waiting[index] = false;
                        !self.activate(activated)
                    }
                    Turn::Waits(writers) => {
                        found.extend(writers.into_iter().map(|writer| (writer, index)));
                        false
                    }
                });
            if !reordered {
                break graph;
            }
        };
        if !self.waiting.iter().any(|&waiting| waiting) {
            return Vec::new();
        }
        let placed: PerComputation<bool> = self.waiting.iter().map(|waiting| !waiting).collect();
        stall(nodes, &graph.outgoing, &placed, &self.waiting, &self.active)
    }

    ///
    /// The dependencies among the computations still waiting: the edges
    /// known before the Tick, the dependencies `found` during it, and the
    /// writers-first edges they give.
    ///
    /// A found writer that has taken its Turn orders nothing more. One the
    /// schedule stopped never takes its Turn, so its edge is kept, and its
    /// reader is stopped with it.
    ///
    fn graph(&self, found: &[(ComputationId, ComputationId)]) -> Graph {
        let schedule = self.schedule;
        let waiting = &self.waiting;
        let edges: Vec<(ComputationId, ComputationId)> = schedule
            .outgoing
            .iter_enumerated()
            .filter(|&(producer, _)| waiting[producer])
            .flat_map(|(producer, consumers)| {
                consumers
                    .iter()
                    .filter(|&&consumer| waiting[consumer])
                    .map(move |&consumer| (producer, consumer))
            })
            .chain(
                found
                    .iter()
                    .copied()
                    .filter(|&(writer, _)| waiting[writer] || schedule.stopped[writer]),
            )
            .chain(
                self.joined_edges
                    .iter()
                    .copied()
                    .filter(|&(producer, consumer)| waiting[producer] && waiting[consumer]),
            )
            .collect();
        let writers: Vec<ComputationId> = schedule
            .writers
            .iter()
            .chain(&self.joined_writers)
            .copied()
            .collect();
        let known = Graph::new(waiting.len(), edges.iter().copied());
        let writers_first = known.writers_first(&schedule.nodes, &writers, waiting);
        Graph::new(waiting.len(), edges.into_iter().chain(writers_first))
    }
}

///
/// Dependency edges by producer, and how many producers each consumer still
/// waits for.
///
struct Graph {
    outgoing: PerComputation<Vec<ComputationId>>,
    indegree: PerComputation<usize>,
}

impl Graph {
    /// `edges` as `(producer, consumer)` pairs among `nodes` computations.
    fn new(nodes: usize, edges: impl IntoIterator<Item = (ComputationId, ComputationId)>) -> Self {
        let mut graph = Self {
            outgoing: PerComputation::filled(nodes, vec![]),
            indegree: PerComputation::filled(nodes, 0),
        };
        for (producer, consumer) in edges {
            graph.indegree[consumer] += 1;
            graph.outgoing[producer].push(consumer);
        }
        graph
    }

    /// The computations `candidate` admits that wait for no producer.
    fn free(&self, candidate: impl Fn(ComputationId) -> bool) -> Vec<ComputationId> {
        self.indegree
            .ids()
            .filter(|&index| candidate(index) && self.indegree[index] == 0)
            .collect()
    }

    ///
    /// Takes each computation in `ready`, and each one whose last dependency
    /// is then taken, in Grid order: the order a schedule is built in and a
    /// Tick continues in, skipping each one `stopped` names.
    ///
    /// `take` takes a computation's Turn and answers whether the order still
    /// holds. Taking stops at the first Turn that answers `false`, a Turn
    /// that waits or one that changed what is ordered after what, and this
    /// answers `true` so that the caller can order the rest again. `false`
    /// once nothing more is ready: a computation whose dependencies are never
    /// all taken is left untaken.
    ///
    fn take_ready(
        &mut self,
        nodes: &Nodes,
        ready: impl IntoIterator<Item = ComputationId>,
        stopped: &PerComputation<bool>,
        mut take: impl FnMut(ComputationId) -> bool,
    ) -> bool {
        let mut ready: BTreeSet<_> = ready.into_iter().map(|index| nodes.key(index)).collect();
        while let Some((_, index)) = ready.pop_first() {
            if stopped[index] {
                continue;
            }
            if !take(index) {
                return true;
            }
            for &consumer in &self.outgoing[index] {
                self.indegree[consumer] -= 1;
                if self.indegree[consumer] == 0 {
                    ready.insert(nodes.key(consumer));
                }
            }
        }
        false
    }

    ///
    /// The writers-first edges these dependencies give `writers`, among the
    /// computations `remaining` names.
    ///
    /// What feeds a computation is every computation with a path of
    /// dependencies to it. The writers a cycle stops take no Turn and order
    /// nothing; the rest are ordered by what feeds them, so a writer that
    /// feeds another goes first, and otherwise by Grid position. Each writer
    /// then goes before every remaining computation that feeds neither it nor
    /// a writer ordered before it. A writer edge never closes a cycle: along
    /// every edge the first writer a computation feeds, or is, comes no later,
    /// and along a writer edge strictly later.
    ///
    fn writers_first(
        &self,
        nodes: &Nodes,
        writers: &[ComputationId],
        remaining: &PerComputation<bool>,
    ) -> Vec<(ComputationId, ComputationId)> {
        if writers.is_empty() {
            return Vec::new();
        }
        let stopped = self.cycle_closure(nodes, remaining);
        let mut incoming = PerComputation::filled(self.outgoing.len(), vec![]);
        for (producer, consumers) in self.outgoing.iter_enumerated() {
            for &consumer in consumers {
                incoming[consumer].push(producer);
            }
        }
        let mut seen = PerComputation::filled(self.outgoing.len(), false);
        let mut pending: Vec<(ComputationId, Vec<ComputationId>)> = writers
            .iter()
            .copied()
            .filter(|&writer| remaining[writer] && !stopped[writer])
            .map(|writer| (writer, feeds(&incoming, writer, &mut seen)))
            .collect();
        pending.sort_unstable_by_key(|&(writer, _)| nodes.key(writer));
        let mut is_pending = PerComputation::filled(self.outgoing.len(), false);
        for &(writer, _) in &pending {
            is_pending[writer] = true;
        }
        // Each writer goes before every remaining computation not yet
        // covered, and the next writer is never fed by one placed before it,
        // so it is uncovered when its predecessor is placed: the writers form
        // a chain. Covered only grows, so a computation is uncovered for a
        // prefix of the chain and the last writer of that prefix orders it
        // after every earlier one through the chain. That one edge each keeps
        // the order the whole prefix's edges give.
        let mut covered = PerComputation::filled(remaining.len(), false);
        let mut edges = Vec::new();
        let mut placed = None;
        while !pending.is_empty() {
            // The first writer in Grid order that no other pending writer
            // feeds.
            let next = (0..pending.len())
                .find(|&candidate| pending[candidate].1.iter().all(|&fed| !is_pending[fed]))
                .expect(
                    "pending writers are on no cycle, so what feeds them is acyclic \
                     and one of them is fed by none of the others",
                );
            let (writer, fed) = pending.remove(next);
            is_pending[writer] = false;
            for index in fed.into_iter().chain([writer]) {
                if !std::mem::replace(&mut covered[index], true)
                    && let Some(previous) = placed.filter(|_| remaining[index])
                {
                    edges.push((previous, index));
                }
            }
            placed = Some(writer);
        }
        if let Some(last) = placed {
            edges.extend(
                remaining
                    .ids()
                    .filter(|&consumer| remaining[consumer] && !covered[consumer])
                    .map(|consumer| (last, consumer)),
            );
        }
        edges
    }

    /// The unresolved computations a cycle stops, closed over whole
    /// Expressions and their consumers. Completed computations keep their
    /// effects, so their outgoing edges do not propagate later faults. Only
    /// unresolved computations propagate stopping to their consumers.
    fn cycle_closure(
        &self,
        nodes: &Nodes,
        remaining: &PerComputation<bool>,
    ) -> PerComputation<bool> {
        let mut indegree = self.indegree.clone();
        let mut unresolved = remaining.clone();
        let mut ready = self.free(|index| remaining[index]);
        while let Some(index) = ready.pop() {
            unresolved[index] = false;
            for &consumer in &self.outgoing[index] {
                indegree[consumer] -= 1;
                if remaining[consumer] && indegree[consumer] == 0 {
                    ready.push(consumer);
                }
            }
        }
        let mut stopped = PerComputation::filled(remaining.len(), false);
        let mut pending: Vec<_> = unresolved
            .iter_enumerated()
            .filter_map(|(index, &pending)| pending.then_some(index))
            .collect();
        while let Some(index) = pending.pop() {
            if !remaining[index] || std::mem::replace(&mut stopped[index], true) {
                continue;
            }
            pending.extend(nodes.expressions[index].clone());
            pending.extend_from_slice(&self.outgoing[index]);
        }
        stopped
    }
}

///
/// Which computations feed `writer`: those with a path of at least one edge
/// to it, read from `incoming`, the edges by consumer.
///
/// `seen` is all `false` on entry and is left so, so one buffer serves every
/// writer and each call costs what it visits rather than every computation.
///
fn feeds(
    incoming: &PerComputation<Vec<ComputationId>>,
    writer: ComputationId,
    seen: &mut PerComputation<bool>,
) -> Vec<ComputationId> {
    let mut fed = Vec::new();
    let mut pending = incoming[writer].clone();
    while let Some(index) = pending.pop() {
        if !std::mem::replace(&mut seen[index], true) {
            fed.push(index);
            pending.extend_from_slice(&incoming[index]);
        }
    }
    for &index in &fed {
        seen[index] = false;
    }
    fed
}

///
/// The [`Stall`]s of the computations no order placed.
///
/// Each cycle is reported once, at the first computation on it in Parser
/// order: a computation still waiting may only be downstream of a cycle, and
/// two computations that reach each other share one. An Expression stopped
/// only because it depends on a cycle is reported at its root, so a performer
/// can tell the cycle from what it starves. A root no activation can reach
/// this Tick takes no Turn with or without the cycle, so it has nothing to
/// wait for and stays quiet.
///
fn stall(
    nodes: &Nodes,
    outgoing: &PerComputation<Vec<ComputationId>>,
    placed: &PerComputation<bool>,
    stopped: &PerComputation<bool>,
    active: &PerComputation<bool>,
) -> Vec<Stall> {
    let mut stalls = Vec::new();
    let on_cycle: PerComputation<_> = nodes
        .owners
        .ids()
        .map(|index| !placed[index] && reaches(outgoing, index, index))
        .collect();
    let mut diagnosed = PerComputation::filled(nodes.len(), false);
    let mut holds_cycle = PerComputation::filled(nodes.len(), false);
    for index in on_cycle.ids().filter(|&index| on_cycle[index]) {
        holds_cycle[nodes.owners[index]] = true;
        if diagnosed[index] {
            continue;
        }
        stalls.push(Stall::Cycle(index));
        for other in ComputationRange::new(index, nodes.len()).filter(|&other| on_cycle[other]) {
            if reaches(outgoing, index, other) && reaches(outgoing, other, index) {
                diagnosed[other] = true;
            }
        }
    }
    for index in nodes.owners.ids() {
        let root = nodes.owners[index] == index;
        if root && stopped[index] && !holds_cycle[index] && active[index] {
            stalls.push(Stall::Waiting(index));
        }
    }
    stalls
}

///
/// Whether a path of at least one edge leads from `from` to `to`.
///
fn reaches(
    outgoing: &PerComputation<Vec<ComputationId>>,
    from: ComputationId,
    to: ComputationId,
) -> bool {
    let mut seen = PerComputation::filled(outgoing.len(), false);
    let mut pending = outgoing[from].clone();
    while let Some(index) = pending.pop() {
        if index == to {
            return true;
        }
        if !std::mem::replace(&mut seen[index], true) {
            pending.extend_from_slice(&outgoing[index]);
        }
    }
    false
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        ComputationId, ComputationRange, Join, Joined, Nodes, PerComputation, Stall, Turn,
        schedule, take_turns,
    };

    /// The computation numbered `index` in Parser preorder.
    fn id(index: usize) -> ComputationId {
        ComputationId(index)
    }

    fn ids<const N: usize>(indices: [usize; N]) -> [ComputationId; N] {
        indices.map(id)
    }

    /// Which roots activation reaches, by computation.
    fn active(reached: &[bool]) -> PerComputation<bool> {
        reached.iter().copied().collect()
    }

    /// Roots at `keys`, each its own Expression unless `nested` names a node
    /// and the root it belongs to. Nested nodes follow their root in Parser
    /// preorder, as the Parser numbers them.
    fn nodes(keys: &[usize], nested: &[(usize, usize)]) -> Nodes {
        let owners: PerComputation<ComputationId> = (0..keys.len())
            .map(|index| {
                nested
                    .iter()
                    .find(|&&(node, _)| node == index)
                    .map_or(id(index), |&(_, root)| id(root))
            })
            .collect();
        let expressions = owners
            .iter()
            .map(|&owner| {
                let end = ComputationRange::new(owner, owners.len())
                    .take_while(|&index| owners[index] == owner)
                    .last()
                    .map_or(owner.index() + 1, |last| last.index() + 1);
                ComputationRange::new(owner, end)
            })
            .collect();
        Nodes {
            keys: keys.iter().copied().collect(),
            owners,
            expressions,
        }
    }

    fn edges(pairs: &[(usize, usize)]) -> BTreeSet<(ComputationId, ComputationId)> {
        pairs
            .iter()
            .map(|&(producer, consumer)| (id(producer), id(consumer)))
            .collect()
    }

    /// A [`Join`] that answers from a table, by root.
    struct Table(BTreeMap<ComputationId, Joined>);

    impl Join for Table {
        fn join(&self, active: &mut PerComputation<bool>, roots: Vec<ComputationId>) -> Joined {
            let mut joined = Joined::default();
            for root in roots {
                active[root] = true;
                if let Some(entry) = self.0.get(&root) {
                    joined.edges.extend(&entry.edges);
                    joined.writers.extend(&entry.writers);
                }
            }
            joined
        }
    }

    #[test]
    fn independent_nodes_take_their_turns_in_grid_order() {
        let order = schedule(
            nodes(&[20, 5, 10], &[]),
            active(&[true; 3]),
            edges(&[]),
            vec![],
        );
        assert_eq!(order.order, ids([1, 2, 0]));
        assert!(order.stalls.is_empty());
    }

    #[test]
    fn a_dependency_goes_before_grid_order() {
        let order = schedule(
            nodes(&[20, 5], &[]),
            active(&[true; 2]),
            edges(&[(0, 1)]),
            vec![],
        );
        assert_eq!(order.order, ids([0, 1]));
    }

    #[test]
    fn a_cycle_stops_every_expression_it_reaches_and_what_waits_on_it() {
        // Root 0 holds node 1, which is on a cycle with root 2. Root 3 waits
        // on root 2, and root 4 depends on nothing.
        let order = schedule(
            nodes(&[0, 1, 2, 3, 4], &[(1, 0)]),
            active(&[true; 5]),
            edges(&[(1, 0), (1, 2), (2, 1), (2, 3)]),
            vec![],
        );
        assert_eq!(order.order, ids([4]));
        assert_eq!(order.stopped, active(&[true, true, true, true, false]));
        assert_eq!(order.stalls, [Stall::Cycle(id(1)), Stall::Waiting(id(3))]);
    }

    #[test]
    fn a_root_no_activation_reaches_waits_on_a_cycle_quietly() {
        let order = schedule(
            nodes(&[0, 1, 2], &[]),
            active(&[true, true, false]),
            edges(&[(0, 1), (1, 0), (1, 2)]),
            vec![],
        );
        assert_eq!(order.stalls, [Stall::Cycle(id(0))]);
    }

    #[test]
    fn a_dynamic_writer_goes_before_what_does_not_feed_it() {
        // Node 1 feeds writer 2, and node 0 feeds nothing.
        let order = schedule(
            nodes(&[0, 1, 2], &[]),
            active(&[true; 3]),
            edges(&[(1, 2)]),
            vec![id(2)],
        );
        assert_eq!(order.order, ids([1, 2, 0]));
    }

    #[test]
    fn a_root_that_joins_brings_its_dependencies_into_the_rest_of_the_tick() {
        // Root 0 activates root 1, which joins with an edge after root 2.
        let order = schedule(
            nodes(&[0, 1, 2], &[]),
            active(&[true, false, true]),
            edges(&[]),
            vec![],
        );
        assert_eq!(order.order, ids([0, 1, 2]));
        let join = Table(BTreeMap::from([(
            id(1),
            Joined {
                edges: vec![(id(2), id(1))],
                writers: vec![],
            },
        )]));
        let mut taken = Vec::new();
        let mut joined = Vec::new();
        let stalls = take_turns(&order, &join, |index, progress| {
            taken.push(index);
            joined.push(progress.joined(index));
            Turn::Taken {
                activated: if index == id(0) { vec![id(1)] } else { vec![] },
            }
        });
        assert_eq!(taken, ids([0, 2, 1]));
        assert_eq!(joined, [false, false, true]);
        assert!(stalls.is_empty());
    }

    #[test]
    fn a_turn_that_waits_is_taken_again_after_its_writers() {
        let order = schedule(nodes(&[0, 1], &[]), active(&[true; 2]), edges(&[]), vec![]);
        let mut taken = Vec::new();
        take_turns(&order, &Table(BTreeMap::new()), |index, progress| {
            taken.push(index);
            if index == id(0) && taken.len() == 1 {
                Turn::Waits(progress.unresolved([id(1)]))
            } else {
                Turn::Taken { activated: vec![] }
            }
        });
        assert_eq!(taken, ids([0, 1, 0]));
    }
}
