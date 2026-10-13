//! Every Dependency a Tick's Turns are ordered by, derived from a [`Lookup`].
//!
//! A Dependency comes from a Reservation, a lock, a Bang, an Input Portal or
//! nesting. This module states each rule once. Ordering asks two questions:
//! [`derive()`], every Dependency and dynamic writer known before the Tick, and
//! [`join()`], those a root activated during the Tick brings with it. Which
//! roots activation reaches is derived here too, because a Bang's
//! Reservation is what delivers it. Execution asks three: [`reached()`], what
//! an admitted write covers; [`Locks::lock_target`], what a lock reaches; and
//! [`input_writers`], which writers a dynamic Input Portal waits on.

use std::collections::BTreeSet;
use std::ops::Range;

use lang::SourceBundle;

use super::Lookup;
use super::ordering::{Join, Joined};
use crate::grid::Position;
use crate::source::language_map::LanguageMap;
use crate::source::portal::{Occupancy, Portal, SpanWrite};

///
/// What a schedule is ordered from: which roots activation reaches before the
/// Tick, every Dependency as a `(producer, consumer)` pair, and the dynamic
/// writers those roots own.
///
pub(super) struct Derived {
    pub(super) active: Vec<bool>,
    pub(super) dependencies: BTreeSet<(usize, usize)>,
    pub(super) writers: Vec<usize>,
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
    for (index, node) in nodes.iter().enumerate() {
        if let Some(parent) = node.parent {
            dependencies.insert((index, parent));
        }
        if active[node.owner] {
            producer_edges(lookup, locks, index, |producer, consumer| {
                dependencies.insert((producer, consumer));
            });
        }
    }
    for consumer in 0..nodes.len() {
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
    active: &mut [bool],
    roots: Vec<usize>,
) -> Joined {
    let mut joined = Joined::default();
    for root in activate(lookup, active, roots) {
        for index in lookup.descendants(root) {
            let mut edge = |producer, consumer| joined.edges.push((producer, consumer));
            producer_edges(lookup, locks, index, &mut edge);
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

/// [`join()`] over one Lookup and its locks, as ordering asks it.
pub(super) struct Joining<'a> {
    pub(super) lookup: &'a Lookup,
    pub(super) locks: &'a Locks,
}

impl Join for Joining<'_> {
    fn join(&self, active: &mut [bool], roots: Vec<usize>) -> Joined {
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
            for relationships in reservations(lookup, index) {
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
fn locked_subtree(locks: &Locks, locker: usize) -> Option<&Range<usize>> {
    match locks.lock_target(locker) {
        Some(Lock::Root(subtree)) => Some(subtree),
        _ => None,
    }
}

/// Whether `locker` is a locking root whose Portal names `producer`.
///
/// A planned write or Bang from that producer onto the locker is not a
/// second Dependency: the lock orders the locker first. A lock that holds
/// withholds the producer's Turn; one that does not, because nothing activated
/// the locker in time, leaves the producer to take its Turn after it, and a
/// Bang that producer delivers to the locker is missed.
fn lock_covers(lookup: &Lookup, locks: &Locks, locker: usize, producer: usize) -> bool {
    let node = &lookup.nodes()[locker];
    node.parent.is_none()
        && locked_subtree(locks, locker).is_some_and(|target| target.contains(&producer))
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
/// Each `(producer, consumer)` Dependency `producer`'s own Portals give it, once
/// its root is active: a lock before the subtree it locks, and each
/// reservation before every computation whose Cells it covers and every
/// root its Bang activates.
///
fn producer_edges(
    lookup: &Lookup,
    locks: &Locks,
    index: usize,
    mut edge: impl FnMut(usize, usize),
) {
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
    // A bundle that writes displaced, an emission or a move, is admitted only
    // into Cells empty at its Turn, so it never writes over a Function its
    // reservation contacts, and it delivers no activation there: occupancy
    // alone orders nothing (ADR 0060 rule 3b). An occupant that leaves first
    // makes room; one still standing refuses the placement, and a refused
    // move leaves `**` in its own Span instead. Its reservation orders none
    // of the Functions it contacts, whether they wait for a Bang or not, so
    // Source order decides between them.
    let orders_contacts = bundle.is_none_or(|bundle| !bundle.writes_displaced());
    if let Some(target) = locked_subtree(locks, index) {
        for consumer in target.clone() {
            edge(index, consumer);
        }
    }
    for relationships in reservations(lookup, index) {
        let mut order_after = |consumer: usize| {
            if exempt_from_itself && consumer == index {
                return;
            }
            if lock_covers(lookup, locks, consumer, index) {
                return;
            }
            edge(index, consumer);
        };
        let contacts = orders_contacts
            .then(|| relationships.functions())
            .into_iter()
            .flatten();
        for contact in contacts {
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
/// Each `(producer, consumer)` Dependency a static Input Portal gives `producer`,
/// once its root is active: one to every computation that reads Cells its
/// reservation covers. These are the Dependencies [`input_edges`] gives, found from
/// the writer's side.
///
fn reader_edges(lookup: &Lookup, producer: usize, mut edge: impl FnMut(usize, usize)) {
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
    active: &[bool],
    consumer: usize,
    mut edge: impl FnMut(usize, usize),
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
/// What one Halt's lock reaches in the Source Snapshot, before any Turn can
/// change it.
///
pub(super) enum Lock {
    /// The complete subtree of the Expression root at the lock site, which
    /// Parser preorder makes contiguous.
    Root(Range<usize>),
    /// A site something other than an Expression root occupies.
    NotARoot,
}

///
/// The [`Lock`] each computation's Portal reaches, shared by scheduling and
/// execution. Only execution decides whether an active, unsuppressed Halt
/// actually applies it.
///
pub(super) struct Locks(Vec<Option<Lock>>);

impl Locks {
    pub(super) fn new(lookup: &Lookup, map: &LanguageMap) -> Self {
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
                    let portal = Portal::at(lookup.grid, destination);
                    (!matches!(
                        portal.occupancy(map, |anchor| lookup.root_at(anchor)),
                        Occupancy::Empty
                    ))
                    .then_some(Lock::NotARoot)
                })
                .collect(),
        )
    }

    ///
    /// What `index`'s lock reaches: nothing for a computation that declares
    /// no lock, or one whose site is outside the Grid or empty.
    ///
    pub(super) fn lock_target(&self, index: usize) -> Option<&Lock> {
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
fn reservations(lookup: &Lookup, index: usize) -> impl Iterator<Item = PortalRelationships<'_>> {
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
/// asks over the Cells an admitted write actually covers. The phases share these
/// facts, but decide separately whether a producer can activate, must precede,
/// or suppresses a contacted computation.
pub(super) struct PortalRelationships<'a> {
    lookup: &'a Lookup,
    output: Position,
    cells: Range<usize>,
}

/// A Function whose Cells a write or Reservation covers.
pub(super) struct FunctionContact {
    index: usize,
    at_anchor: bool,
    /// Includes the contacted Function itself, in Parser preorder.
    subtree: Range<usize>,
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
    pub(super) fn bang_roots(&self) -> impl Iterator<Item = usize> + '_ {
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
    pub(super) fn index(&self) -> usize {
        self.index
    }

    /// Whether the write begins at the contacted Function's anchor.
    pub(super) fn at_anchor(&self) -> bool {
        self.at_anchor
    }

    /// The contacted Function and every computation nested in it, in Parser
    /// preorder.
    pub(super) fn subtree(&self) -> Range<usize> {
        self.subtree.clone()
    }
}

#[cfg(test)]
mod test {
    use super::{Locks, derive, join};
    use crate::grid::Grid;
    use crate::source::Cells;
    use crate::source::language_map::LanguageMap;
    use crate::source::tick::{Lookup, computations};

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
        let (nodes, _) = computations(grid, &map);
        let lookup = Lookup::new(grid, nodes);
        let locks = Locks::new(&lookup, &map);
        (lookup, locks)
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
        let mut active = vec![false; lookup.nodes().len()];
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
}
