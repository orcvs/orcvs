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

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::ops::Range;

use super::{Diagnostic, Lookup, diagnose};

///
/// One revision's computations and the order [`schedule`] gives their Turns.
/// No Tick changes it: a Tick's progress through it, and any dependency a
/// Turn finds, belong to [`take_turns`].
///
pub(super) struct Schedule {
    pub(super) lookup: Lookup,
    pub(super) order: Vec<usize>,
    pub(super) diagnostics: Vec<Diagnostic>,
    /// Every dependency edge known before the Tick, by producer, so that a
    /// Tick can continue ordering from a Turn that finds one more. The
    /// writers-first edges are not among them: they follow from these and
    /// from what a Tick finds, so each ordering derives its own.
    outgoing: Vec<Vec<usize>>,
    /// The dynamic writers activation can reach this Tick.
    writers: Vec<usize>,
    /// Which roots activation can reach this Tick, by computation.
    active: Vec<bool>,
    /// Which computations a same-Tick dependency cycle stops: exactly those
    /// `order` holds no Turn for.
    pub(super) stopped: Vec<bool>,
}

#[cfg(test)]
impl Schedule {
    /// Whether activation known before the Tick reaches `root`.
    pub(super) fn holds_active(&self, root: usize) -> bool {
        self.active[root]
    }
}

///
/// The schedule for `lookup`'s computations under `edges`, as `(producer,
/// consumer)` pairs, with each of `writers`, the dynamic writers, ordered
/// writers first: the Turns in the order the edges and Grid position give
/// them, less every Expression a same-Tick dependency cycle reaches, which
/// takes no Turn and is diagnosed after `diagnostics`.
///
pub(super) fn schedule(
    lookup: Lookup,
    active: Vec<bool>,
    edges: BTreeSet<(usize, usize)>,
    writers: Vec<usize>,
    mut diagnostics: Vec<Diagnostic>,
) -> Schedule {
    let nodes = lookup.nodes();
    let everything = vec![true; nodes.len()];
    let mut known = Dependencies::new(nodes.len(), edges.iter().copied());
    let writers_first = known.writers_first(&lookup, &writers, &everything);
    // Ordering changes only `indegree`, so with no writers-first edges `known`
    // is ordered itself and still holds the edges the schedule keeps.
    let mut with_writers = (!writers_first.is_empty())
        .then(|| Dependencies::new(nodes.len(), edges.iter().copied().chain(writers_first)));
    let dependencies = with_writers.as_mut().unwrap_or(&mut known);
    let ready = dependencies.free(|_| true);
    let stopped = dependencies.cycle_closure(&lookup, &everything);
    let mut order = Vec::new();
    dependencies.take_ready(&lookup, ready, &stopped, |index| {
        order.push(index);
        true
    });
    if order.len() != nodes.len() {
        let mut placed = vec![false; nodes.len()];
        for &index in &order {
            placed[index] = true;
        }
        diagnostics.extend(diagnose_cycles(
            &lookup,
            &dependencies.outgoing,
            &placed,
            &stopped,
            &active,
        ));
    }
    let Dependencies { outgoing, .. } = known;
    Schedule {
        lookup,
        order,
        diagnostics,
        outgoing,
        writers,
        active,
        stopped,
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
    active: &'a [bool],
    reader: usize,
    read: Range<usize>,
) -> impl Iterator<Item = usize> + 'a {
    lookup
        .writes
        .touching(read)
        .filter(move |&writer| writer != reader && active[lookup.nodes()[writer].owner])
}

///
/// What one attempt at a Turn did.
///
pub(super) enum Turn {
    /// The Turn was taken, or settled without effect. `activated` names the
    /// roots a dynamic write in it delivered activation to, which the
    /// schedule could not know of.
    Taken { activated: Vec<usize> },
    /// The Turn must wait for these writers, which
    /// [`Progress::unresolved_writers`] names, and is taken again once they
    /// have been.
    Waits(Vec<usize>),
}

///
/// Takes one Tick's Turns through `schedule`, and answers the diagnostics for
/// the Turns a cycle found during the Tick leaves untaken.
///
/// `fired` names the roots a Bang fired at the start of the Tick activates.
/// One the schedule did not hold active joins the order before any Turn, as
/// a root a dynamic write activates joins it at that write.
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
pub(super) fn take_turns(
    schedule: &Schedule,
    fired: Vec<usize>,
    mut take: impl FnMut(usize, &Progress<'_>) -> Turn,
) -> Vec<Diagnostic> {
    let mut progress = Progress {
        schedule,
        waiting: schedule.stopped.iter().map(|stopped| !stopped).collect(),
        active: Cow::Borrowed(&schedule.active),
        joined_edges: Vec::new(),
        joined_writers: Vec::new(),
    };
    if progress.activate(fired) {
        return progress.continue_ordering(Vec::new(), take);
    }
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
    schedule: &'a Schedule,
    /// Which computations the schedule holds a Turn for that they have not
    /// yet taken.
    waiting: Vec<bool>,
    /// Which roots activation can reach this Tick: the schedule's, and every
    /// root a Bang fired at the start of the Tick or a dynamic write has
    /// activated since, with what those reach.
    active: Cow<'a, [bool]>,
    /// The edges the roots activated during the Tick give: what their own
    /// Portals reach and the static Input Portals their reservations cover.
    /// Found once, when each root joins, because what a root's Portals give
    /// does not change during the Tick.
    joined_edges: Vec<(usize, usize)>,
    /// The dynamic writers the roots activated during the Tick own.
    joined_writers: Vec<usize>,
}

impl Progress<'_> {
    ///
    /// The writers of the Cells `read` that have not yet taken their Turn,
    /// which `reader`'s Turn must wait for.
    ///
    /// The writers are those [`input_writers`] orders a static Input Portal
    /// after, among the roots active this Tick. A writer the schedule stopped
    /// never takes its Turn, so the reader waits on it and is stopped with
    /// it, as a static reader of those Cells is.
    ///
    pub(super) fn unresolved_writers(&self, reader: usize, read: Range<usize>) -> Vec<usize> {
        let schedule = self.schedule;
        let mut writers: Vec<usize> = input_writers(&schedule.lookup, &self.active, reader, read)
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
    pub(super) fn joined(&self, index: usize) -> bool {
        let owner = self.schedule.lookup.nodes()[index].owner;
        self.active[owner] && !self.schedule.active[owner]
    }

    ///
    /// Makes active this Tick each of `roots` the Tick does not hold active
    /// yet, and every root those can deliver activation to. Answers whether
    /// any was new.
    ///
    fn activate(&mut self, roots: Vec<usize>) -> bool {
        let pending: Vec<usize> = roots
            .into_iter()
            .filter(|&root| !self.active[root])
            .collect();
        if pending.is_empty() {
            return false;
        }
        let lookup = &self.schedule.lookup;
        let active = self.active.to_mut();
        for &root in &pending {
            active[root] = true;
        }
        // A root activated during the Tick orders what its own Portals
        // reach, as an active root's do before it, and its dynamic writers
        // go writers first.
        for root in super::activate(lookup, active, pending) {
            for index in lookup.descendants(root) {
                let mut edge = |producer, consumer| self.joined_edges.push((producer, consumer));
                super::producer_edges(lookup, index, &mut edge);
                super::reader_edges(lookup, index, &mut edge);
                if lookup.nodes()[index]
                    .function
                    .dynamic_output_portal()
                    .is_some()
                {
                    self.joined_writers.push(index);
                }
            }
        }
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
        mut found: Vec<(usize, usize)>,
        mut take: impl FnMut(usize, &Progress<'_>) -> Turn,
    ) -> Vec<Diagnostic> {
        let schedule = self.schedule;
        let lookup = &schedule.lookup;
        let dependencies = loop {
            let mut dependencies = self.dependencies(&found);
            let stopped = dependencies.cycle_closure(lookup, &self.waiting);
            let ready = dependencies.free(|index| self.waiting[index]);
            let reordered = dependencies.take_ready(lookup, ready, &stopped, |index| {
                match take(index, &self) {
                    Turn::Taken { activated } => {
                        self.waiting[index] = false;
                        !self.activate(activated)
                    }
                    Turn::Waits(writers) => {
                        found.extend(writers.into_iter().map(|writer| (writer, index)));
                        false
                    }
                }
            });
            if !reordered {
                break dependencies;
            }
        };
        if !self.waiting.contains(&true) {
            return Vec::new();
        }
        let placed: Vec<bool> = self.waiting.iter().map(|waiting| !waiting).collect();
        diagnose_cycles(
            lookup,
            &dependencies.outgoing,
            &placed,
            &self.waiting,
            &self.active,
        )
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
    fn dependencies(&self, found: &[(usize, usize)]) -> Dependencies {
        let schedule = self.schedule;
        let lookup = &schedule.lookup;
        let waiting = &self.waiting;
        let edges: Vec<(usize, usize)> = schedule
            .outgoing
            .iter()
            .enumerate()
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
        let writers: Vec<usize> = schedule
            .writers
            .iter()
            .chain(&self.joined_writers)
            .copied()
            .collect();
        let known = Dependencies::new(waiting.len(), edges.iter().copied());
        let writers_first = known.writers_first(lookup, &writers, waiting);
        Dependencies::new(waiting.len(), edges.into_iter().chain(writers_first))
    }
}

///
/// Dependency edges by producer, and how many producers each consumer still
/// waits for.
///
struct Dependencies {
    outgoing: Vec<Vec<usize>>,
    indegree: Vec<usize>,
}

impl Dependencies {
    /// `edges` as `(producer, consumer)` pairs among `nodes` computations.
    fn new(nodes: usize, edges: impl IntoIterator<Item = (usize, usize)>) -> Self {
        let mut dependencies = Self {
            outgoing: vec![vec![]; nodes],
            indegree: vec![0; nodes],
        };
        for (producer, consumer) in edges {
            dependencies.indegree[consumer] += 1;
            dependencies.outgoing[producer].push(consumer);
        }
        dependencies
    }

    /// The computations `candidate` admits that wait for no producer.
    fn free(&self, candidate: impl Fn(usize) -> bool) -> Vec<usize> {
        (0..self.indegree.len())
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
        lookup: &Lookup,
        ready: impl IntoIterator<Item = usize>,
        stopped: &[bool],
        mut take: impl FnMut(usize) -> bool,
    ) -> bool {
        let key = |index: usize| (lookup.grid.index(lookup.nodes()[index].anchor), index);
        let mut ready: BTreeSet<_> = ready.into_iter().map(key).collect();
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
                    ready.insert(key(consumer));
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
        lookup: &Lookup,
        writers: &[usize],
        remaining: &[bool],
    ) -> Vec<(usize, usize)> {
        if writers.is_empty() {
            return Vec::new();
        }
        let stopped = self.cycle_closure(lookup, remaining);
        let mut incoming = vec![vec![]; self.outgoing.len()];
        for (producer, consumers) in self.outgoing.iter().enumerate() {
            for &consumer in consumers {
                incoming[consumer].push(producer);
            }
        }
        let key = |index: usize| (lookup.grid.index(lookup.nodes()[index].anchor), index);
        let mut seen = vec![false; self.outgoing.len()];
        let mut pending: Vec<(usize, Vec<usize>)> = writers
            .iter()
            .copied()
            .filter(|&writer| remaining[writer] && !stopped[writer])
            .map(|writer| (writer, feeds(&incoming, writer, &mut seen)))
            .collect();
        pending.sort_unstable_by_key(|&(writer, _)| key(writer));
        let mut is_pending = vec![false; self.outgoing.len()];
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
        let mut covered = vec![false; remaining.len()];
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
                (0..remaining.len())
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
    fn cycle_closure(&self, lookup: &Lookup, remaining: &[bool]) -> Vec<bool> {
        let mut indegree = self.indegree.clone();
        let mut unresolved = remaining.to_vec();
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
        let mut stopped = vec![false; remaining.len()];
        let mut pending: Vec<_> = unresolved
            .iter()
            .enumerate()
            .filter_map(|(index, &pending)| pending.then_some(index))
            .collect();
        while let Some(index) = pending.pop() {
            if !remaining[index] || std::mem::replace(&mut stopped[index], true) {
                continue;
            }
            pending.extend(lookup.descendants(lookup.nodes()[index].owner));
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
fn feeds(incoming: &[Vec<usize>], writer: usize, seen: &mut [bool]) -> Vec<usize> {
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
/// The diagnostics for the computations no order placed.
///
/// Each cycle is diagnosed once, at the first computation on it in Parser
/// order: a computation still waiting may only be downstream of a cycle, and
/// two computations that reach each other share one. An Expression stopped
/// only because it depends on a cycle says so at its root, so a performer can
/// tell the cycle from what it starves. A root no activation can reach this
/// Tick takes no Turn with or without the cycle, so it has nothing to wait
/// for and stays quiet.
///
fn diagnose_cycles(
    lookup: &Lookup,
    outgoing: &[Vec<usize>],
    placed: &[bool],
    stopped: &[bool],
    active: &[bool],
) -> Vec<Diagnostic> {
    let nodes = lookup.nodes();
    let mut diagnostics = Vec::new();
    let on_cycle: Vec<_> = (0..nodes.len())
        .map(|index| !placed[index] && reaches(outgoing, index, index))
        .collect();
    let mut diagnosed = vec![false; nodes.len()];
    let mut holds_cycle = vec![false; nodes.len()];
    for index in (0..nodes.len()).filter(|&index| on_cycle[index]) {
        holds_cycle[nodes[index].owner] = true;
        if diagnosed[index] {
            continue;
        }
        diagnostics.push(diagnose(&nodes[index], "same-Tick dependency cycle"));
        for other in (index..nodes.len()).filter(|&other| on_cycle[other]) {
            if reaches(outgoing, index, other) && reaches(outgoing, other, index) {
                diagnosed[other] = true;
            }
        }
    }
    for (index, node) in nodes.iter().enumerate() {
        if node.parent.is_none() && stopped[index] && !holds_cycle[index] && active[index] {
            diagnostics.push(diagnose(node, "waiting on a same-Tick dependency cycle"));
        }
    }
    diagnostics
}

///
/// Whether a path of at least one edge leads from `from` to `to`.
///
fn reaches(outgoing: &[Vec<usize>], from: usize, to: usize) -> bool {
    let mut seen = vec![false; outgoing.len()];
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
