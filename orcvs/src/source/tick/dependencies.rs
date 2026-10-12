//! Every Dependency a Tick's Turns are ordered by, derived from a [`Lookup`].
//!
//! A Dependency comes from a Reservation, a lock, a Bang, an Input Portal or
//! nesting. This module states each rule once and answers two questions:
//! [`derive()`], every Dependency and dynamic writer known before the Tick, and
//! [`join()`], those a root activated during the Tick brings with it. Which
//! roots activation reaches is derived here too, because a Bang's
//! Reservation is what delivers it. Ordering sorts what this answers.

use std::collections::BTreeSet;
use std::ops::Range;

use lang::SourceBundle;

use super::ordering::{Join, Joined};
use super::{LockTarget, Lookup};

///
/// What a schedule is ordered from: which roots activation reaches before the
/// Tick, every Dependency as a `(producer, consumer)` pair, and the dynamic
/// writers those roots own.
///
pub(super) struct Derived {
    pub(super) active: Vec<bool>,
    pub(super) edges: BTreeSet<(usize, usize)>,
    pub(super) writers: Vec<usize>,
}

///
/// Every Dependency `lookup`'s computations hold before the Tick: a nested
/// Function before the Function it answers, and each active root's
/// Reservations, locks, Bangs and Input Portals.
///
pub(super) fn derive(lookup: &Lookup) -> Derived {
    let nodes = lookup.nodes();
    let active = active_roots(lookup);
    let mut edges = BTreeSet::new();
    for (index, node) in nodes.iter().enumerate() {
        if let Some(parent) = node.parent {
            edges.insert((index, parent));
        }
        if active[node.owner] {
            producer_edges(lookup, index, |producer, consumer| {
                edges.insert((producer, consumer));
            });
        }
    }
    for consumer in 0..nodes.len() {
        input_edges(lookup, &active, consumer, |producer, consumer| {
            edges.insert((producer, consumer));
        });
    }
    let writers = dynamic_writers(lookup, &active).collect();
    Derived {
        active,
        edges,
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
pub(super) fn join(lookup: &Lookup, active: &mut [bool], roots: Vec<usize>) -> Joined {
    let mut joined = Joined::default();
    for root in activate(lookup, active, roots) {
        for index in lookup.descendants(root) {
            let mut edge = |producer, consumer| joined.edges.push((producer, consumer));
            producer_edges(lookup, index, &mut edge);
            reader_edges(lookup, index, &mut edge);
            if lookup.nodes()[index]
                .function
                .dynamic_output_portal()
                .is_some()
            {
                joined.writers.push(index);
            }
        }
    }
    joined
}

/// [`join()`] over one Lookup, as ordering asks it.
pub(super) struct Joining<'a>(pub(super) &'a Lookup);

impl Join for Joining<'_> {
    fn join(&self, active: &mut [bool], roots: Vec<usize>) -> Joined {
        join(self.0, active, roots)
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
fn active_roots(lookup: &Lookup) -> Vec<bool> {
    let nodes = lookup.nodes();
    let mut active: Vec<_> = nodes
        .iter()
        .map(|node| node.parent.is_none() && node.function.is_intrinsically_active())
        .collect();
    let pending: Vec<_> = active
        .iter()
        .enumerate()
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
fn activate(lookup: &Lookup, active: &mut [bool], mut pending: Vec<usize>) -> Vec<usize> {
    let nodes = lookup.nodes();
    let mut reached = Vec::new();
    while let Some(owner) = pending.pop() {
        reached.push(owner);
        for index in lookup.descendants(owner) {
            if !nodes[index].function.can_emit_bang() {
                continue;
            }
            for relationships in lookup.reservations(index) {
                for index in relationships.bang_roots().collect::<Vec<_>>() {
                    if !nodes[index].function.is_intrinsically_active() && !active[index] {
                        active[index] = true;
                        pending.push(index);
                    }
                }
            }
        }
    }
    reached
}

/// The complete subtree a locking Function's Portal names.
fn locked_subtree(lookup: &Lookup, locker: usize) -> Option<&Range<usize>> {
    match &lookup.locks[locker] {
        Some(LockTarget::Root(subtree)) => Some(subtree),
        _ => None,
    }
}

/// Whether `locker` is a locking root whose Portal names `producer`.
///
/// A planned write or Bang from that producer onto the locker is not a
/// second ordering edge: the lock orders the locker first. A lock that holds
/// withholds the producer's Turn; one that does not, because nothing activated
/// the locker in time, leaves the producer to take its Turn after it, and a
/// Bang that producer delivers to the locker is missed.
fn lock_covers(lookup: &Lookup, locker: usize, producer: usize) -> bool {
    let node = &lookup.nodes()[locker];
    node.parent.is_none()
        && locked_subtree(lookup, locker).is_some_and(|target| target.contains(&producer))
}

///
/// The dynamic writers among the computations `active` roots own.
///
/// A dynamic writer's destination is known only at its Turn, so it reserves
/// nothing and is ordered writers first instead.
///
fn dynamic_writers<'a>(lookup: &'a Lookup, active: &'a [bool]) -> impl Iterator<Item = usize> + 'a {
    lookup
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| active[node.owner] && node.function.dynamic_output_portal().is_some())
        .map(|(index, _)| index)
}

///
/// Each `(producer, consumer)` edge `producer`'s own Portals give it, once
/// its root is active: a lock before the subtree it locks, and each
/// reservation before every computation whose Cells it covers and every
/// root its Bang activates.
///
fn producer_edges(lookup: &Lookup, index: usize, mut edge: impl FnMut(usize, usize)) {
    let nodes = lookup.nodes();
    let node = &nodes[index];
    let bundle = node.function.source_effect().map(|effect| effect.bundle);
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
    // A bundle that leaves its producer standing, an emission, is admitted
    // only into Cells empty at its Turn, so it never writes over a Function
    // its reservation contacts, and it delivers no activation there: occupancy
    // alone orders nothing (ADR 0060 rule 3b). An occupant that leaves first
    // makes room; one still standing refuses the emission.
    let orders_contacts = bundle.is_none_or(SourceBundle::clears_own_span);
    // A bundle that clears its own Span and writes displaced moves, and a
    // move meets an intrinsically active root it contacts in Source order.
    // Both facts are named although only bundles that clear their own Span
    // order contacts at all, so the rule reads the same wherever a move is.
    let moves = bundle.is_some_and(|bundle| bundle.clears_own_span() && bundle.writes_displaced());
    if let Some(target) = locked_subtree(lookup, index) {
        for consumer in target.clone() {
            edge(index, consumer);
        }
    }
    for relationships in lookup.reservations(index) {
        let mut order_after = |consumer: usize| {
            if exempt_from_itself && consumer == index {
                return;
            }
            if lock_covers(lookup, consumer, index) {
                return;
            }
            edge(index, consumer);
        };
        let contacts = orders_contacts
            .then(|| relationships.functions())
            .into_iter()
            .flatten();
        for contact in contacts {
            // A move is admitted only where the Cells it enters are empty,
            // so a moving bundle's Portal never writes over the Language Unit
            // it contacts: the contact blocks the move and the producer
            // leaves `**` in its own Span instead. Its reservation still
            // orders it before a contacted root that waits for a Bang, as any
            // reservation does. An intrinsically active root is exempt, as in
            // the Bang emission arm below, and meets the mover in Source
            // order.
            //
            // Without it two Functions whose Portals cover each other name
            // each other in reservations neither can write through, and
            // ordering each after the other makes that pair a cycle that
            // stops both, though two blocked moves are not a contested
            // Cell.
            if moves
                && nodes[nodes[contact.index].owner]
                    .function
                    .is_intrinsically_active()
            {
                continue;
            }
            for descendant in contact.subtree {
                order_after(descendant);
            }
        }
        for consumer in relationships.literal_consumers() {
            order_after(consumer);
        }
        if node.function.can_emit_bang() {
            for owner in relationships.bang_roots() {
                if !nodes[owner].function.is_intrinsically_active() {
                    for consumer in lookup.descendants(owner) {
                        order_after(consumer);
                    }
                }
            }
        }
    }
}

///
/// Each `(producer, consumer)` edge a static Input Portal gives `producer`,
/// once its root is active: one to every computation that reads Cells its
/// reservation covers. These are the edges [`input_edges`] gives, found from
/// the writer's side.
///
fn reader_edges(lookup: &Lookup, producer: usize, mut edge: impl FnMut(usize, usize)) {
    for relationships in lookup.reservations(producer) {
        for consumer in lookup.reads.touching(relationships.cells.clone()) {
            if consumer != producer {
                edge(producer, consumer);
            }
        }
    }
}

///
/// Each `(producer, consumer)` edge `consumer`'s static Input Portal gives
/// it: one from every writer `active` roots own whose reservation covers the
/// Cells it reads.
///
fn input_edges(
    lookup: &Lookup,
    active: &[bool],
    consumer: usize,
    mut edge: impl FnMut(usize, usize),
) {
    // An Input Portal reads working Source. Those Cells are often an operand
    // Span, so they cannot sit in `literals`; the edge is the same fact
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
    active: &'a [bool],
    reader: usize,
    read: Range<usize>,
) -> impl Iterator<Item = usize> + 'a {
    lookup
        .writes
        .touching(read)
        .filter(move |&writer| writer != reader && active[lookup.nodes()[writer].owner])
}

#[cfg(test)]
mod test {
    use super::{derive, join};
    use crate::grid::Grid;
    use crate::source::Cells;
    use crate::source::language_map::LanguageMap;
    use crate::source::tick::{Lookup, computations};

    /// The Lookup for `rows`, each padded to the Grid's width.
    fn lookup(grid: Grid, rows: &[&str]) -> Lookup {
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
        let (nodes, _) = computations(grid, &map);
        Lookup::new(grid, nodes, &map)
    }

    /// The computation anchored at `(x, y)`.
    fn at(lookup: &Lookup, x: usize, y: usize) -> usize {
        lookup
            .nodes()
            .iter()
            .position(|node| (node.anchor.x(), node.anchor.y()) == (x, y))
            .unwrap_or_else(|| panic!("no computation anchored at ({x}, {y})"))
    }

    // Equality Bangs every Tick two columns west of `*!`, which activates
    // Halt, and Halt locks the Add one row south of it.
    const HALT: [&str; 3] = [".=0101  ", "  *!    ", "  .+0304"];

    #[test]
    fn a_lock_goes_before_the_subtree_it_locks() {
        let lookup = lookup(Grid::with_shape(8, 3), &HALT);
        let (halt, add) = (at(&lookup, 2, 1), at(&lookup, 2, 2));
        assert!(derive(&lookup).edges.contains(&(halt, add)));
    }

    #[test]
    fn a_bang_goes_before_the_root_it_activates() {
        let lookup = lookup(Grid::with_shape(8, 3), &HALT);
        let (equality, halt) = (at(&lookup, 0, 0), at(&lookup, 2, 1));
        let derived = derive(&lookup);
        assert!(
            derived.active[halt],
            "the Bang reaches Halt before the Tick"
        );
        assert!(derived.edges.contains(&(equality, halt)));
    }

    #[test]
    fn a_locked_producer_is_not_ordered_before_the_lock_it_reaches() {
        // `^^` reserves the Cells Halt stands in, but Halt locks it, so the
        // lock alone orders the pair.
        let lookup = lookup(
            Grid::with_shape(8, 3),
            &[".=0101  ", "  *!    ", "  ^^    "],
        );
        let (halt, mover) = (at(&lookup, 2, 1), at(&lookup, 2, 2));
        let edges = derive(&lookup).edges;
        assert!(edges.contains(&(halt, mover)));
        assert!(!edges.contains(&(mover, halt)));
    }

    #[test]
    fn a_bundle_that_clears_its_own_span_is_not_ordered_after_itself() {
        // The Bang Function reserves the Cells it stands in and does not
        // move, so only the exemption keeps it from being its own cycle.
        let lookup = lookup(Grid::with_shape(6, 1), &["**    "]);
        let bang = at(&lookup, 0, 0);
        let derived = derive(&lookup);
        assert!(derived.active[bang]);
        assert!(!derived.edges.contains(&(bang, bang)));
    }

    #[test]
    fn a_reservation_goes_before_the_function_it_contacts() {
        let lookup = lookup(Grid::with_shape(6, 2), &[".+0101", ".-0202"]);
        let (writer, contacted) = (at(&lookup, 0, 0), at(&lookup, 0, 1));
        assert!(derive(&lookup).edges.contains(&(writer, contacted)));
    }

    #[test]
    fn a_mover_meets_an_intrinsically_active_root_in_source_order() {
        // `>>` reserves the Cells `^^` stands in, and `^^` takes its Turn
        // with nothing delivered to it, so Source order decides between them.
        let lookup = lookup(Grid::with_shape(6, 1), &[">>^^  "]);
        let (mover, active) = (at(&lookup, 0, 0), at(&lookup, 2, 0));
        assert!(!derive(&lookup).edges.contains(&(mover, active)));
        // A root that waits for a Bang is still ordered after the mover.
        let lookup = self::lookup(Grid::with_shape(6, 1), &[">>*!  "]);
        let (mover, halt) = (at(&lookup, 0, 0), at(&lookup, 2, 0));
        assert!(derive(&lookup).edges.contains(&(mover, halt)));
    }

    #[test]
    fn a_reservation_goes_before_the_literal_it_covers() {
        let lookup = lookup(Grid::with_shape(10, 2), &["    .+0101", "  .-0101  "]);
        let (writer, consumer) = (at(&lookup, 4, 0), at(&lookup, 2, 1));
        assert!(derive(&lookup).edges.contains(&(writer, consumer)));
    }

    #[test]
    fn a_joining_root_brings_what_its_bang_activates() {
        let lookup = lookup(Grid::with_shape(8, 3), &HALT);
        let (equality, halt, add) = (at(&lookup, 0, 0), at(&lookup, 2, 1), at(&lookup, 2, 2));
        let mut active = vec![false; lookup.nodes().len()];
        active[equality] = true;
        let joined = join(&lookup, &mut active, vec![equality]);
        assert!(active[halt], "the Bang reaches Halt");
        assert!(!active[add], "a lock activates nothing");
        assert!(joined.edges.contains(&(equality, halt)));
        assert!(joined.edges.contains(&(halt, add)));
        assert!(joined.writers.is_empty());
    }
}
