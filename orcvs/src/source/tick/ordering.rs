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
//! The schedule is shared by every Tick planned against the same scheduling
//! inputs and is never changed by one. A Tick's progress through it is kept
//! here, and the dependency graph a Turn that waits needs is rebuilt only
//! when one does.

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
    /// Every dependency edge `order` was built from, by producer, so that a
    /// Tick can continue ordering from a Turn that finds one more.
    outgoing: Vec<Vec<usize>>,
    /// Which roots activation can reach this Tick, by computation.
    active: Vec<bool>,
    /// Which computations a same-Tick dependency cycle stops: exactly those
    /// `order` holds no Turn for.
    pub(super) stopped: Vec<bool>,
}

///
/// The schedule for `lookup`'s computations under `edges`, as `(producer,
/// consumer)` pairs: the Turns in the order the edges and Grid position give
/// them, less every Expression a same-Tick dependency cycle reaches, which
/// takes no Turn and is diagnosed after `diagnostics`.
///
pub(super) fn schedule(
    lookup: Lookup,
    active: Vec<bool>,
    edges: BTreeSet<(usize, usize)>,
    mut diagnostics: Vec<Diagnostic>,
) -> Schedule {
    let nodes = lookup.nodes();
    let mut dependencies = Dependencies::new(nodes.len(), edges);
    let ready = dependencies.free(|_| true);
    let mut order = Vec::new();
    dependencies.take_ready(&lookup, ready, vec![true; nodes.len()], |index| {
        order.push(index);
        None
    });
    let Dependencies { outgoing, .. } = dependencies;
    let mut stopped = vec![false; nodes.len()];
    if order.len() != nodes.len() {
        let mut placed = vec![false; nodes.len()];
        for &index in &order {
            placed[index] = true;
        }
        let mut pending: Vec<_> = (0..nodes.len()).filter(|&index| !placed[index]).collect();
        while let Some(index) = pending.pop() {
            if std::mem::replace(&mut stopped[index], true) {
                continue;
            }
            pending.extend(lookup.descendants(nodes[index].owner));
            pending.extend_from_slice(&outgoing[index]);
        }
        order.retain(|&index| !stopped[index]);
        diagnostics.extend(diagnose_cycles(
            &lookup, &outgoing, &placed, &stopped, &active,
        ));
    }
    Schedule {
        lookup,
        order,
        diagnostics,
        outgoing,
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
/// Takes one Tick's Turns through `schedule`, and answers the diagnostics for
/// the Turns a cycle found during the Tick leaves untaken.
///
/// `take` takes a computation's Turn and answers `None`, or answers the
/// writers it must wait for, which [`Progress::unresolved_writers`] names. A
/// Turn that waits leaves no effect and is taken again once its writers have
/// been. Turns are taken in the schedule's order until one waits; from then
/// on the computations still to take their Turn keep every edge the schedule
/// ordered them by, the waiting Turn gains one from each writer, and they are
/// taken by the loop the schedule was ordered by. A Tick that waits for
/// nothing therefore takes its Turns in exactly the schedule's order.
///
/// Turns still waiting when nothing is ready are on a same-Tick dependency
/// cycle or wait on one. They are diagnosed and left untaken, and every other
/// Turn of the Tick is still taken.
///
pub(super) fn take_turns(
    schedule: &Schedule,
    mut take: impl FnMut(usize, &Progress<'_>) -> Option<Vec<usize>>,
) -> Vec<Diagnostic> {
    let mut progress = Progress {
        schedule,
        waiting: schedule.stopped.iter().map(|stopped| !stopped).collect(),
    };
    for &index in &schedule.order {
        match take(index, &progress) {
            None => progress.waiting[index] = false,
            Some(writers) => return progress.continue_ordering(index, writers, take),
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
}

impl Progress<'_> {
    ///
    /// The writers of the Cells `read` that have not yet taken their Turn,
    /// which `reader`'s Turn must wait for.
    ///
    /// The writers are those [`input_writers`] orders an anchored Input
    /// Portal after. A writer the schedule stopped never takes its Turn, so
    /// the reader waits on it and is stopped with it, as an anchored reader
    /// of those Cells is.
    ///
    pub(super) fn unresolved_writers(&self, reader: usize, read: Range<usize>) -> Vec<usize> {
        let schedule = self.schedule;
        let mut writers: Vec<usize> =
            input_writers(&schedule.lookup, &schedule.active, reader, read)
                .filter(|&writer| self.waiting[writer] || schedule.stopped[writer])
                .collect();
        writers.sort_unstable();
        writers.dedup();
        writers
    }

    /// Orders the rest of the Tick from `waiter`'s Turn, which found
    /// `writers` still to take theirs, as [`take_turns`] describes.
    fn continue_ordering(
        mut self,
        waiter: usize,
        writers: Vec<usize>,
        mut take: impl FnMut(usize, &Progress<'_>) -> Option<Vec<usize>>,
    ) -> Vec<Diagnostic> {
        let schedule = self.schedule;
        let waiting = &self.waiting;
        let mut dependencies = Dependencies::new(
            waiting.len(),
            schedule
                .outgoing
                .iter()
                .enumerate()
                .filter(|&(producer, _)| waiting[producer])
                .flat_map(|(producer, consumers)| {
                    consumers
                        .iter()
                        .filter(|&&consumer| waiting[consumer])
                        .map(move |&consumer| (producer, consumer))
                }),
        );
        dependencies.wait_on(waiter, writers);
        let ready = dependencies.free(|index| waiting[index]);
        dependencies.take_ready(&schedule.lookup, ready, self.waiting.clone(), |index| {
            let writers = take(index, &self);
            if writers.is_none() {
                self.waiting[index] = false;
            }
            writers
        });
        if !self.waiting.contains(&true) {
            return Vec::new();
        }
        let placed: Vec<bool> = self.waiting.iter().map(|waiting| !waiting).collect();
        diagnose_cycles(
            &schedule.lookup,
            &dependencies.outgoing,
            &placed,
            &self.waiting,
            &schedule.active,
        )
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
    /// Tick continues in.
    ///
    /// `take` takes a computation's Turn, or answers the computations it must
    /// wait for. One that waits is ordered after them and is taken again once
    /// they have been, so a dependency found at a Turn joins the order there.
    /// A computation whose dependencies are never all taken is left untaken.
    ///
    fn take_ready(
        &mut self,
        lookup: &Lookup,
        ready: impl IntoIterator<Item = usize>,
        mut remaining: Vec<bool>,
        mut take: impl FnMut(usize) -> Option<Vec<usize>>,
    ) {
        let key = |index: usize| (lookup.grid.index(lookup.nodes()[index].anchor), index);
        let mut ready: BTreeSet<_> = ready.into_iter().map(key).collect();
        let mut stopped = self.cycle_closure(lookup, &remaining);
        while let Some((_, index)) = ready.pop_first() {
            if stopped[index] {
                continue;
            }
            if let Some(writers) = take(index) {
                self.wait_on(index, writers);
                stopped = self.cycle_closure(lookup, &remaining);
                continue;
            }
            remaining[index] = false;
            for &consumer in &self.outgoing[index] {
                self.indegree[consumer] -= 1;
                if self.indegree[consumer] == 0 {
                    ready.insert(key(consumer));
                }
            }
        }
    }

    /// The unresolved computations a cycle stops, closed over whole
    /// Expressions and their consumers. Completed computations keep their
    /// effects; this closure prevents any remaining part from taking a Turn.
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
            if std::mem::replace(&mut stopped[index], true) {
                continue;
            }
            pending.extend(lookup.descendants(lookup.nodes()[index].owner));
            pending.extend_from_slice(&self.outgoing[index]);
        }
        stopped
    }

    /// Orders `waiter` after each of `writers`.
    fn wait_on(&mut self, waiter: usize, writers: Vec<usize>) {
        self.indegree[waiter] += writers.len();
        for writer in writers {
            self.outgoing[writer].push(waiter);
        }
    }
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
