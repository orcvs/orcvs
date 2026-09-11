//! Tick-local execution of Parser-owned expressions (ADR 0034).
//!
//! Fixed Portal destinations and nested ownership determine the complete order
//! before execution. Spatial writes remain character encodings until consumed;
//! nested results are typed values. Only the final effects are published.

pub(super) mod execution;

use lang::{
    Anchor, Atom, Function, ReplacementChange, SourceBundle, SourceEffect, Tick, TickInputs,
};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use super::encoding::{Encoding, RenderError, Rendered};
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
    /// How wide this computation's result may be, per ADR 0036, and the one
    /// home that fact has. A computation is built reserving the Cell pair
    /// every result reserves unless a declaration widens it, and
    /// [`derive_reservations`] — which [`Lookup::new`] runs once over every
    /// computation, because a Function's reservation reads its operand
    /// children's — settles which of them it is.
    reserved: Reserved,
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

impl Reserved {
    /// The Cells this reservation covers from `output` along its row, or
    /// `None` where a scalar pair cannot fit before the row edge — a Cell pair
    /// whose second Cell is in the next row is not a Span at all.
    ///
    /// This is ADR 0036's width rule itself rather than a reading of it, which
    /// is why the callers ask for the Cells instead of matching on the variant
    /// and measuring them again.
    fn cells_from(self, grid: Grid, output: Position) -> Option<Range<usize>> {
        let start = grid.index(output).get();
        match self {
            Self::Pair => {
                grid.offset_in_row(output, SCALAR_WIDTH - 1)?;
                Some(start..start + SCALAR_WIDTH)
            }
            // The row's remaining Cells, measured from the destination's own
            // column so the count stops at the row edge rather than running on
            // into the next row's Cells.
            Self::Row => Some(start..start + (grid.cols() - output.x())),
        }
    }

    /// Whether a result `width` Cells wide is one this reservation covers.
    ///
    /// A narrower answer is refused alongside a wider one where the
    /// reservation is a Cell pair: the reservation is what the row fit was
    /// decided against, and a single Cell at the last Cell of a row is a write
    /// the Portal admits and the schedule never reserved.
    fn admits_width(self, width: usize) -> bool {
        match self {
            Self::Pair => width == SCALAR_WIDTH,
            Self::Row => true,
        }
    }

    /// Whether an admitted write can leave Cells of this reservation
    /// untouched, rather than covering it exactly.
    ///
    /// A Cell pair is exactly the write it orders, so the reservation covering
    /// a computation and the write reaching it are one fact. A row reservation
    /// runs to the row's end and the answer may stop columns short of it, so
    /// the two are separate facts and only the admitted write settles the
    /// second.
    fn admits_a_narrower_write(self) -> bool {
        match self {
            Self::Pair => false,
            Self::Row => true,
        }
    }

    /// Whether a computation reserving this can answer a Sequence, which is
    /// the question an owning Function asks of an operand child before it
    /// decides whether it widens over one.
    ///
    /// It coincides with [`Reserved::admits_a_narrower_write`] because a row
    /// is the only reservation wider than one Atom. They are separate
    /// questions: this one is about what an answer can be, and that one about
    /// what a write must cover.
    fn may_be_a_sequence(self) -> bool {
        match self {
            Self::Pair => false,
            Self::Row => true,
        }
    }
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
    fn new(grid: Grid, mut nodes: Vec<Computation>) -> Self {
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
        derive_reservations(&mut nodes);
        let lookup = Self {
            grid,
            nodes,
            functions: Claims::new(functions),
            literals: Claims::new(literals),
            operands: Claims::new(operands),
            subtree_ends,
        };
        // The agreement [`Lookup::would_reserve`] is a hypothesis against:
        // what a computation reserves is what its own declared Function
        // re-derives, so asking about a replacement is a different question
        // rather than a settled fact answered a second way.
        //
        // It is not a cross-check between two derivations, and reading it as
        // one would overstate it. Both sides call the same [`reserved_for`]
        // over the same children, and the pass above settled every node: each
        // computation is built holding `Reserved::Pair`, so
        // [`derive_reservations`] skips none of them, and its reverse loop
        // settles every child before the parent that reads it and never
        // revisits one. Re-deriving here therefore reads the inputs that pass
        // read and answers what it answered.
        //
        // What it does prove is that those conditions still hold, which is why
        // it is kept: that the nodes are still ordered parent-before-child — a
        // child stored ahead of its parent would leave the parent holding a
        // width derived from a `Reserved::Pair` the child had not settled yet
        // — and that no skip added to the pass leaves a computation underived.
        // Both are cheap to hold in debug builds and silent everywhere else.
        //
        // It also pins an ordering `stated::plan_with_answers` depends on: the
        // `Reserved::Row` that fixture states is a width no declaration
        // derives, so it can only be written after this has run.
        debug_assert!(
            (0..lookup.nodes.len()).all(|index| {
                lookup.would_reserve(index, lookup.nodes[index].function)
                    == lookup.nodes[index].reserved
            }),
            "a computation reserves a width its own declaration does not derive"
        );
        lookup
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
        self.nodes[index].reserved
    }

    /// What scheduling would have reserved for `index` had its Function been
    /// `function`, which is a question about a Function that is not there
    /// rather than a second way to read [`Lookup::reserved`].
    ///
    /// A spatial write can replace a Function at its original anchor after the
    /// schedule is fixed, and a replacement that answers a wider result than
    /// the one reserved for would write Cells no dependency edge names. This
    /// is the question `deliver_output` asks before it admits such a
    /// replacement; it reads its children's settled reservations, which the
    /// same guard keeps stable.
    ///
    /// Asked with the computation's own Function it answers what that
    /// computation already reserves — for every computation
    /// [`derive_reservations`] settled, which is every one production builds,
    /// and what `Lookup::new` asserts. A width a test states rather than
    /// derives is the exception, and the only one: `stated::plan_with_answers`
    /// writes a `Reserved::Row` no declaration produces, and this answers
    /// `Reserved::Pair` for that computation ever after. That fixture refuses
    /// to combine a stated width with a stated Function replacement for
    /// exactly that reason, so no replacement is checked against a width this
    /// disagrees with.
    fn would_reserve(&self, index: usize, function: Function) -> Reserved {
        reserved_for(&self.nodes, index, function)
    }

    /// Which of the five facts a Function replacement at `index` changes about
    /// `running`, the Function that computation is running, or `None` where it
    /// changes none of them and the replacement is admitted.
    ///
    /// `lang` answers the four a declaration states and this crate appends the
    /// fifth, because a reservation is derived from the schedule ADR 0032
    /// settled and from the widths this computation's children hold, and `lang`
    /// has neither. Appending it rather than interleaving it is what keeps the
    /// order the guard has always applied, so every replacement reports the
    /// fact it reported before the facts had names.
    ///
    /// The width is compared against the settled reservation rather than
    /// against a second derivation: the Turns were ordered from that one, and
    /// this same guard is what keeps every admitted replacement inside it.
    fn replacement_change(
        &self,
        index: usize,
        replacement: Function,
        running: Function,
    ) -> Option<ReplacementChange> {
        replacement.replacing(running).or_else(|| {
            (self.would_reserve(index, replacement) != self.reserved(index))
                .then_some(ReplacementChange::Width)
        })
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
        let cells = self.reserved(index).cells_from(self.grid, output)?;
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

///
/// What every computation reserves, per ADR 0036, derived in one reverse pass.
///
/// Preorder puts every operand child at a higher index than the Function that
/// owns it, so one pass backwards is enough: a node's children are answered
/// before its own turn comes, which is what lets a pervasive Function widen
/// over an operand that reserves a row.
///
/// A reservation already settled as [`Reserved::Row`] is left where it stands,
/// and the guard that leaves it there is load-bearing rather than a shortcut.
/// [`reserved_for`] is a pure function of the Function table and the children's
/// settled reservations: it has no memory of what the computation already held,
/// so for a `Row` no declaration produced — one a test states, because no built
/// Function declares a Sequence answer — it answers `Pair` and narrows the
/// statement away. Skipping such a computation is what lets this pass run over
/// reservations decided before it rather than only over freshly built
/// computations, which is the whole reason it is a function and not a loop
/// inside [`Lookup::new`]. Production never reaches that case: every
/// computation is built reserving a `Pair`, so every one of them is derived
/// exactly once.
///
/// The guard is what makes this a derivation forward from all-`Pair` rather
/// than a re-derivation. A computation already holding [`Reserved::Row`] is
/// never revisited, so running this again after its Function changed would
/// answer the pre-change width without complaint. Giving the reserved width one
/// home did not make that reachable: this pass reads the parsed Function on the
/// computation, and a replacement changes only the running Function on its
/// execution state, so no width derived here is ever derived from a Function a
/// replacement has replaced. The one caller that runs the pass a second time is
/// `stated::plan_with_answers`, and it does so to widen ancestors over a width
/// the fixture stated rather than to re-derive a changed declaration.
///
fn derive_reservations(nodes: &mut [Computation]) {
    for index in (0..nodes.len()).rev() {
        if nodes[index].reserved == Reserved::Pair {
            nodes[index].reserved = reserved_for(nodes, index, nodes[index].function);
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
fn reserved_for(nodes: &[Computation], index: usize, function: Function) -> Reserved {
    let node = &nodes[index];
    let widened = function.widens_over_a_sequence_operand()
        && node.operands.iter().any(|operand| {
            operand
                .child
                .is_some_and(|child| nodes[child].reserved.may_be_a_sequence())
        });
    if function.answers_sequence() || widened {
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

    ///
    /// The Expression roots a Source-writing Function's own Span could move
    /// into at these Cells.
    ///
    /// ADR 0006 activates on contact: "Complete aligned root contact also
    /// directly delivers Bang activation". This is that question asked of the
    /// schedule, which knows the geometry and not the Source — the Cells a
    /// move newly enters, and so which of these roots is actually contacted,
    /// are settled at the Turn against working Source.
    ///
    /// It is the contacted Function rather than a cardinal neighbour, which is
    /// what separates it from [`PortalRelationships::bang_roots`]: a
    /// Self-Banging Function moves onto a root's Cells, while a Bang activates
    /// roots it never touches.
    fn contacted_roots(&self) -> impl Iterator<Item = usize> + '_ {
        self.lookup
            .functions
            .touching(self.cells.clone())
            .filter(|index| self.lookup.nodes()[*index].parent.is_none())
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
    ///
    /// The four anchors are one geometric fact and are stated as one, and the
    /// west arm was for a long time the one no Source could reach. Until
    /// `spatial-tick-planning/03`, every Function in `define_functions!` took
    /// at least one operand, so a root anchored two columns west always claimed
    /// this destination for that operand and a Bang landing here was operand
    /// contact rather than an anchor — the `(6, 2)` row of
    /// `fixed_bang_destinations_respect_alignment_and_operand_contact` asserts
    /// exactly that silence.
    ///
    /// `^^ vv << >>` made the arm answer with a root and changed nothing by it,
    /// because both callers act on a root only where it is not intrinsically
    /// active and those four take their Turn without a Bang. The Directional
    /// Bang Functions are what made it decide a Tick: `*^` declares no operand
    /// and waits for activation, so a Bang two columns east of one is the only
    /// thing that starts it. `an_active_directional_bang_function_emits_its_self_banging_function`
    /// drives that arm, and deleting it leaves the north and west halves of
    /// that test emitting nothing.
    ///
    /// The asymmetry the arm states is the language's and not this function's:
    /// a horizontally aligned root anchors two columns away because every
    /// Language Unit spells as a Cell pair.
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

pub(super) fn plan(
    grid: Grid,
    bytes: &[u8],
    map: &LanguageMap,
    tick: Tick,
) -> (TickPlan, Vec<execution::ComputationState>) {
    match schedule(grid, map) {
        Ok(schedule) => execution::execute(grid, bytes, map, tick, schedule),
        Err(diagnostics) => unscheduled(diagnostics),
    }
}

/// ADR 0009's refusal. Only [`carry`] raises it, because only a carried
/// schedule can name a destination for a Terminal Output Function. The
/// destinations [`computations`] resolves are the ordinary result position a
/// root resolves for itself and, since the Source-writing Functions arrived,
/// the displaced destination a row declares — and the
/// `performs_terminal_output()` gate is read before either, so neither reaches
/// a Function this refusal is about. Statement order is what holds the rule in
/// shipped code; this constant holds it in the carried schedule.
#[cfg(test)]
const REFUSED_PORTAL: &str = "a Terminal Output Function cannot have a Portal";

///
/// Gives each computation the Portal destinations `destinations` names for it,
/// in place of the ordinary result position it resolved for itself.
///
/// Done to the computations once [`computations`] has stated them rather than
/// while it is stating them, so that a schedule can carry chosen destinations
/// without the planning path taking a parameter or a map lookup of its own.
///
/// A test needs this because no production Tick can state such a destination
/// for a Terminal Output Function. ADR 0004's Source Function family is built
/// now and states its destinations through [`computations`], but it states them
/// from a declaration rather than from input, and no row declares both a
/// displacement and Terminal Output. Carrying one is still the only way to put
/// a destination on a Function that resolves none, which is the case ADR 0009
/// refuses.
///
#[cfg(test)]
fn carry(
    grid: Grid,
    nodes: &mut [Computation],
    diagnostics: &mut Vec<Diagnostic>,
    destinations: &BTreeMap<CellIndex, Vec<Position>>,
) {
    let mut refusals = Vec::new();
    for node in nodes.iter_mut() {
        let Some(outputs) = destinations.get(&grid.index(node.anchor)) else {
            continue;
        };
        // The gate [`computations`] applies, applied to the same question: a
        // Terminal Output Function has no Cell destination at all, so naming
        // one for it is the error rather than the destination. It resolved no
        // destination while it was stated, and keeps none here.
        if node.function.performs_terminal_output() {
            refusals.push(diagnose(node, REFUSED_PORTAL));
            continue;
        }
        node.outputs = outputs
            .iter()
            .map(|output| {
                grid.assert_owns(*output);
                Ok(*output)
            })
            .collect();
    }
    // [`computations`] raises no refusal of its own — it resolves a Terminal
    // Output Function no destination to refuse — so everything it handed here
    // is a layout diagnostic from the walk that follows its resolution, and
    // these refusals belong in front of it.
    diagnostics.splice(0..0, refusals);
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
) -> Result<Schedule, Vec<Diagnostic>> {
    let (mut nodes, mut diagnostics) = computations(grid, map);
    carry(grid, &mut nodes, &mut diagnostics, destinations);
    order_turns(Lookup::new(grid, nodes), diagnostics)
}

///
/// Plans one Tick against a schedule carrying `destinations`.
///
/// [`plan`] for a Tick whose destinations a test states, rather than the
/// ordinary result positions its roots resolve for themselves.
///
#[cfg(test)]
pub(super) fn plan_carrying(
    grid: Grid,
    bytes: &[u8],
    map: &LanguageMap,
    tick: Tick,
    destinations: &BTreeMap<CellIndex, Vec<Position>>,
) -> (TickPlan, Vec<execution::ComputationState>) {
    match schedule_carrying(grid, map, destinations) {
        Ok(schedule) => execution::execute(grid, bytes, map, tick, schedule),
        Err(diagnostics) => unscheduled(diagnostics),
    }
}

fn diagnose(node: &Computation, message: impl Into<String>) -> Diagnostic {
    Diagnostic::for_expression(node.anchor, node.span, message.into())
}

///
/// The Tick Plan of a Tick no order could be established for: its diagnostics
/// are published and nothing else is, because nothing ran.
///
/// No computation took a Turn, so there is no execution state to report either:
/// a Tick that never started is the one case where the empty plan and the empty
/// states say the same thing.
///
fn unscheduled(diagnostics: Vec<Diagnostic>) -> (TickPlan, Vec<execution::ComputationState>) {
    (
        TickPlan {
            writes: vec![],
            play_commands: vec![],
            diagnostics,
        },
        vec![],
    )
}

///
/// Which roots this Tick must order effects for: the ones that take a Turn
/// with nothing delivered to them, closed over every root those can deliver
/// activation to.
///
/// An inactive root contributes no Portal, so a schedule that left one out
/// would order nothing against the writes it turns out to make. The seed is
/// exact — a root declares its activation source and nothing else decides it,
/// per ADR 0006 — and the closure is deliberately wider than the Tick: which
/// root a delivery actually reaches depends on values and on working Source
/// that no schedule has yet, so execution checks activation again once those
/// producers have settled.
///
/// Two declarations deliver, because ADR 0006 gives activation two paths. A
/// Function that can return Bang delivers through the Bang it writes, to the
/// cardinal anchors that ADR names for a Source-resident Bang. A Function whose
/// bundle advances delivers on contact, to a complete root its own Span moves
/// into. No row declares both, and the Directional Bang Functions declare
/// neither: one emits into Cells it has already required to be empty, so there
/// is nothing there to activate.
///
fn active_roots(lookup: &Lookup) -> Vec<bool> {
    let nodes = lookup.nodes();
    let mut active: Vec<_> = nodes
        .iter()
        .map(|node| node.parent.is_none() && node.function.is_intrinsically_active())
        .collect();
    let mut pending: Vec<_> = active
        .iter()
        .enumerate()
        .filter_map(|(index, active)| active.then_some(index))
        .collect();
    while let Some(owner) = pending.pop() {
        for index in lookup.descendants(owner) {
            let function = nodes[index].function;
            if !function.can_emit_bang() && !advances(function) {
                continue;
            }
            for output in nodes[index]
                .outputs
                .iter()
                .filter_map(|output| output.as_ref().ok())
            {
                let Some(relationships) = lookup.reserved_at(index, *output) else {
                    continue;
                };
                let banged = function
                    .can_emit_bang()
                    .then(|| relationships.bang_roots())
                    .into_iter()
                    .flatten();
                let contacted = advances(function)
                    .then(|| relationships.contacted_roots())
                    .into_iter()
                    .flatten();
                for index in banged.chain(contacted).collect::<Vec<_>>() {
                    if !nodes[index].function.is_intrinsically_active() && !active[index] {
                        active[index] = true;
                        pending.push(index);
                    }
                }
            }
        }
    }
    active
}

///
/// Whether this Function's declared bundle vacates the Cells it stands in.
///
/// Three rules of the schedule ask it, and all three are about the producer's
/// own Span rather than about its destination: whether the bundle reserves that
/// Span, whether ordering the producer after itself is a defect or the design,
/// and whether a refused destination has somewhere to deliver activation from.
/// Asking `source_effect().is_some()` instead would give a Directional Bang
/// Function all three, and it earns none of them.
///
fn advances(function: Function) -> bool {
    matches!(
        function.source_effect(),
        Some(SourceEffect {
            bundle: SourceBundle::Advance,
            ..
        })
    )
}

fn schedule(grid: Grid, map: &LanguageMap) -> Result<Schedule, Vec<Diagnostic>> {
    let (nodes, diagnostics) = computations(grid, map);
    order_turns(Lookup::new(grid, nodes), diagnostics)
}

///
/// The computations one Source revision holds, in Parser preorder, and the
/// diagnostics its layout owes before any of them is ordered.
///
/// This is everything a schedule knows before a [`Lookup`] indexes it: which
/// Cells each computation claims, which Portal destinations it resolved, and
/// which Expressions the row edge cut short. Every computation here reserves
/// the Cell pair ADR 0036 gives a result nothing widens; which of them a
/// declaration does widen, and therefore what is ordered after what, is the
/// [`Lookup`]'s to settle.
///
fn computations(grid: Grid, map: &LanguageMap) -> (Vec<Computation>, Vec<Diagnostic>) {
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
                // The one gate that means Terminal Output rather than
                // "answers an effect": a Terminal Output Function has no Cell
                // destination at all, while ADR 0004 gives a Source-writing
                // Function a validated write bundle and ADR 0009 lets it
                // resolve multiple Portals. Asking the wide question here
                // would deny the Halt, Directional Bang, and Jump Functions a
                // Portal they are entitled to.
                let outputs = if function.performs_terminal_output() || parent.is_some() {
                    vec![]
                } else if let Some(effect) = function.source_effect() {
                    // ADR 0004's effect bundle, in the emission order ADR 0020
                    // resolves it by. The declared bundle says how many Portals
                    // that is: an `Advance` clears the Cells the producer
                    // stands in before it writes, and both are reserved because
                    // a clear is a write — scheduling makes its dependency
                    // edges from every Cell a producer reaches, and a clear
                    // reaching a computation that already ran is the ordering
                    // defect execution rejects a Tick for. An `Emit` plans
                    // nothing at its own Cells, so it reserves only the one it
                    // writes.
                    //
                    // The displacement is read from the declaration here and
                    // answered again by the Interpreter at the Turn. That is
                    // the relationship ADR 0036 already has between a
                    // reservation and the write it orders: a schedule is fixed
                    // before any Function evaluates, so it reads what the
                    // Function declares, and the admitted write is the answer
                    // arriving inside what was reserved for it.
                    let destination = Portal::displaced(grid, anchor, effect.columns, effect.rows)
                        .map(|portal| portal.destination());
                    match effect.bundle {
                        SourceBundle::Advance => vec![Ok(anchor), destination],
                        SourceBundle::Emit => vec![destination],
                    }
                } else {
                    vec![Portal::ordinary_result(grid, anchor).map(|portal| portal.destination())]
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
                    // ADR 0036 reserves a Cell pair for every result no
                    // declaration widens, so this is the reservation itself
                    // and not a placeholder. Which computations a declaration
                    // does widen is the pass in `Lookup::new` to settle,
                    // because it reads reservations this loop has not built
                    // yet.
                    reserved: Reserved::Pair,
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
    (nodes, diagnostics)
}

///
/// One Tick's execution order, from the reservations a [`Lookup`] has already
/// derived: every dependency edge ADR 0036's reservations name, resolved into
/// the order the Turns are taken in, or the cycle that admits no order at all.
///
fn order_turns(
    lookup: Lookup,
    mut diagnostics: Vec<Diagnostic>,
) -> Result<Schedule, Vec<Diagnostic>> {
    let grid = lookup.grid;
    let nodes = lookup.nodes();
    let active = active_roots(&lookup);
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
            let may_stop_short = lookup.reserved(index).admits_a_narrower_write();
            // The third way a producer's own Cells are not a defect, and the
            // only one a declaration states outright: an advancing bundle
            // clears the Span it stands in, so its first Portal covers its own
            // spelling by design. Ordering it after itself would reject every
            // Tick one of these takes a Turn in. An emitting bundle plans
            // nothing at its own Cells and needs no exception.
            let clears_its_own_span = advances(node.function);
            let mut order_after = |consumer: usize| {
                if (may_stop_short || clears_its_own_span) && consumer == index {
                    return;
                }
                edges.insert((index, consumer));
            };
            for contact in relationships.functions() {
                // ADR 0004 admits a move only where the Cells it enters are
                // empty, so a Source-writing Function's Portal never writes
                // over the Language Unit it contacts: the contact blocks the
                // move and the Function bangs its own Span instead. The one
                // thing contact still delivers is Bang activation, and an
                // intrinsically active root does not need it — which is the
                // exemption the Bang emission arm below already makes, for the
                // same reason.
                //
                // Without it two Functions whose Portals cover each other name
                // each other in reservations neither can write through, and
                // ordering each after the other makes that pair a cycle that
                // costs the whole Grid its Tick. ADR 0036 names that rejected
                // alternative: a reservation orders Turns and decides nothing
                // else, and two blocked moves are not a contested Cell.
                if clears_its_own_span
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

#[cfg(test)]
mod test {
    use lang::Value;

    use super::{Effect, Encoding, Portal, Tick, execution::ComputationState, resolve};

    ///
    /// Builds one Source Snapshot from `rows`, padded to the Grid's width.
    ///
    fn snapshot(grid: Grid, rows: &[&str]) -> String {
        let width = grid.count() / rows.len();
        rows.iter().map(|row| format!("{row:width$}")).collect()
    }

    ///
    /// One Source built from `rows`, its Cells set one at a time as an editor
    /// sets them.
    ///
    /// Shared by every helper that runs a Tick against stated rows, so that
    /// comparing two of them compares what they do differently and never two
    /// ways of building a Source.
    ///
    fn seeded_source(grid: Grid, rows: &[&str]) -> crate::source::Source {
        let mut source = crate::source::Source::new(grid);
        for (index, byte) in snapshot(grid, rows).bytes().enumerate() {
            source
                .set(cell(grid, index), &char::from(byte).to_string())
                .unwrap();
        }
        source
    }

    ///
    /// Runs one Tick against `rows` with the Portal destinations `outputs`
    /// states carried on its schedule, and commits its plan.
    ///
    /// The plan-only half of the carried route, for the tests that state
    /// their destinations through a fixture rather than calling
    /// [`super::plan_carrying`] themselves.
    ///
    fn carried_source(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
    ) -> (TickPlan, crate::source::Source) {
        let (plan, _, source) = carried_tick(grid, rows, outputs);
        (plan, source)
    }

    ///
    /// [`carried_source`], with the inputs the Interpreter received for each
    /// computation it ran for.
    ///
    /// Two helpers rather than one because the two questions are asked by
    /// different tests: most of this module asks only what a Tick planned, and
    /// binding a fact they never read would cost every one of them a line.
    ///
    fn carried_tick(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
    ) -> (TickPlan, Vec<lang::TickInputs>, crate::source::Source) {
        let mut source = seeded_source(grid, rows);
        let (plan, states) =
            source.execute_carrying(Tick::ZERO, &carried_destinations(grid, outputs));
        (plan, interpreted(&states), source)
    }

    ///
    /// The explicit inputs the Interpreter received during one Tick, one entry
    /// per call rather than per computation.
    ///
    /// The three questions this module asks of an execution — how many
    /// computations were interpreted, which inputs each of them received, and
    /// how many times each of them ran — read off the Tick's own output rather
    /// than off a record kept beside the thread it ran on.
    ///
    /// Per call and not per computation because the third question is the one
    /// with no other witness: a computation that runs twice is handed the same
    /// inputs both times and writes the same value both times, so neither the
    /// state's own slot nor the Source can tell it from one that ran once. A
    /// state repeated [`ComputationState::interpretations`] times restores the
    /// multiplicity that the retired thread-local record carried for free.
    ///
    /// The entries are grouped in the order the schedule holds its
    /// computations, which is the order they were parsed rather than the order
    /// their Turns were taken. One test compares that order — the one naming
    /// three roots no dependency orders against each other — and reads it as
    /// which computations ran rather than as evidence of Turn order.
    ///
    fn interpreted(states: &[ComputationState]) -> Vec<lang::TickInputs> {
        let mut calls = Vec::new();
        for state in states {
            if let Some(inputs) = state.interpreted() {
                calls.extend(std::iter::repeat_n(inputs, state.interpretations()));
            }
        }
        calls
    }

    ///
    /// The fixed Portal destinations a fixture states, in the terms a carried
    /// schedule takes them: one destination list per anchor, keyed by the
    /// anchor's own Cell.
    ///
    fn carried_destinations(
        grid: Grid,
        outputs: &[(usize, usize)],
    ) -> std::collections::BTreeMap<CellIndex, Vec<crate::grid::Position>> {
        let mut carried: std::collections::BTreeMap<_, Vec<_>> = std::collections::BTreeMap::new();
        for (anchor, output) in outputs {
            carried
                .entry(cell(grid, *anchor))
                .or_default()
                .push(grid.position_at(cell(grid, *output)));
        }
        carried
    }

    ///
    /// The committed Source as one string per Grid row.
    ///
    /// A Self-Banging Function is specified by where its Cells are, so a test
    /// of one states a Grid and compares a Grid. Reading the same fact off a
    /// list of Cell indices states the arithmetic instead of the geometry, and
    /// an expectation nobody can picture is one nobody can check.
    ///
    fn rows_of(grid: Grid, source: &crate::source::Source) -> Vec<String> {
        source
            .snapshot()
            .into_bytes()
            .chunks(grid.cols())
            .map(|row| String::from_utf8(row.to_vec()).expect("ASCII Source"))
            .collect()
    }

    ///
    /// Runs `ticks` consecutive Ticks against `rows` through the production
    /// path, answering the Grid after each one.
    ///
    /// [`crate::source::Source::execute`] and not [`carried_source`]: a
    /// Self-Banging Function resolves its own Portals from the offset it
    /// declares, so a fixture that carried destinations for it would be
    /// testing the fixture.
    ///
    fn tick_by_tick(
        grid: Grid,
        rows: &[&str],
        ticks: u64,
    ) -> (Vec<TickPlan>, Vec<Vec<String>>, crate::source::Source) {
        let mut source = seeded_source(grid, rows);
        let mut plans = Vec::new();
        let mut grids = Vec::new();
        for tick in 0..ticks {
            plans.push(source.execute(Tick::new(tick)));
            grids.push(rows_of(grid, &source));
        }
        (plans, grids, source)
    }

    ///
    /// The messages one Tick Plan diagnosed, so a test states what was said
    /// rather than how many things were.
    ///
    fn messages(plan: &TickPlan) -> Vec<&str> {
        plan.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect()
    }

    #[test]
    fn a_self_banging_function_advances_its_whole_span_by_one_cell() {
        // ADR 0006's move, in all four directions, each from the middle of a
        // Grid that admits it. The vertical pair shares no Cell with the Span
        // it left; the horizontal pair overlaps it by one, and per ADR 0004
        // the clear still covers the complete old Span while ADR 0020's
        // later-write-wins commits the Cell the two writes share. Carving the
        // clear around that Cell and testing both Cells a horizontal move
        // lands on are the two symmetrical bugs this states the answer to.
        let column = Grid::new(6, 3);
        let (_, north, _) = tick_by_tick(column, &["", "^^", ""], 1);
        assert_eq!(north[0], ["^^    ", "      ", "      "]);

        let (_, south, _) = tick_by_tick(column, &["", "vv", ""], 1);
        assert_eq!(south[0], ["      ", "      ", "vv    "]);

        let row = Grid::new(6, 1);
        let (_, east, _) = tick_by_tick(row, &["  >>  "], 1);
        assert_eq!(east[0], ["   >> "]);

        let (_, west, _) = tick_by_tick(row, &["  <<  "], 1);
        assert_eq!(west[0], [" <<   "]);
    }

    #[test]
    fn a_self_banging_function_moves_once_per_tick_and_bangs_where_it_stops() {
        // Intrinsic activation is a turn every Tick and not a one-off: the
        // Function is in the Snapshot each time, so it moves each time. The
        // last Tick is the edge, which is the other half — an out-of-Grid move
        // costs it the Span it stood in and leaves `**` there.
        let (plans, grids, _) = tick_by_tick(Grid::new(6, 1), &["  >>  "], 3);

        assert_eq!(grids[0], ["   >> "]);
        assert_eq!(grids[1], ["    >>"]);
        assert_eq!(grids[2], ["    **"]);
        for plan in &plans {
            assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        }
    }

    #[test]
    fn two_moves_that_want_the_same_cells_each_bang_in_their_own_span() {
        // Two movers approaching each other close the gap by two Cells a Tick,
        // so its parity never changes and there are two cases. An odd gap ends
        // at one Cell: the first Turn takes it and the second is blocked, which
        // is `a_self_banging_function_moves_once_per_tick_and_bangs_where_it_
        // stops` with a second mover supplying the obstacle. An even gap ends
        // flush, and that is this test — each wants the two Cells the other
        // stands in, and ADR 0006 gives each the refusal it gives a mover
        // blocked by anything else: "replaces its current Span with `**`".
        //
        // No rule here is new. `>>` holds the earlier Turn by Source order,
        // finds `<` in the Cell it would enter, and reports in its own Span.
        // `<<` then finds that fresh `*` and does the same. Both Bangs are
        // display, so the Tick after clears them and the pair is gone.
        //
        // What this pins is the schedule. Each mover reserves Cells the other
        // stands in, which is an edge each way and an order no sort satisfies.
        // Before `order_turns` dropped one of them the Tick was rejected, and
        // the Grid never moved again.
        let (plans, grids, _) = tick_by_tick(Grid::new(8, 1), &[">>  <<  "], 3);

        assert_eq!(grids[0], [" >><<   "]);
        assert_eq!(grids[1], [" ****   "]);
        assert_eq!(grids[2], ["        "]);
        for plan in &plans {
            assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        }

        // The vertical pair, which reaches the same refusal through a different
        // geometry: a vertical destination shares no Cell with the Span it
        // leaves, so each mover tests the whole of the other's Span rather than
        // the one Cell a horizontal move enters.
        let (plans, grids, _) = tick_by_tick(Grid::new(4, 2), &["vv", "^^"], 2);

        assert_eq!(grids[0], ["**  ", "**  "]);
        assert_eq!(grids[1], ["    ", "    "]);
        for plan in &plans {
            assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        }
    }

    #[test]
    fn a_blocked_pair_of_moves_leaves_the_rest_of_the_grid_running() {
        // The cost of rejecting that Tick was never local. A rejected schedule
        // discards every write on the Grid, so an Addition sharing the Source
        // with a facing pair fell silent with nothing wrong with it and no
        // diagnostic of its own. The pair reports in its own four Cells and the
        // Addition answers `03` in the same Tick.
        let (plans, grids, _) = tick_by_tick(Grid::new(13, 2), &[" >><<  .+0102", ""], 1);

        assert_eq!(grids[0], [" ****  .+0102", "       03    "]);
        assert!(
            plans[0].diagnostics.is_empty(),
            "{:?}",
            plans[0].diagnostics
        );
    }

    #[test]
    fn a_move_that_leaves_the_grid_replaces_its_own_span_with_bang() {
        // Every edge, each reached by the one Function that points at it. Two
        // refusals answer for the four: a displacement past the first column
        // or past the last row resolves no Portal at all, and a displacement
        // that stays inside the Grid but runs past the row edge resolves one
        // and is refused the whole write. ADR 0006 gives both the same cost,
        // so the Grid says the same thing four times.
        let column = Grid::new(4, 1);
        let (_, north, _) = tick_by_tick(column, &["^^  "], 1);
        assert_eq!(north[0], ["**  "]);

        let (_, south, _) = tick_by_tick(column, &["vv  "], 1);
        assert_eq!(south[0], ["**  "]);

        let (_, west, _) = tick_by_tick(column, &["<<  "], 1);
        assert_eq!(west[0], ["**  "]);

        let (_, east, _) = tick_by_tick(column, &["  >>"], 1);
        assert_eq!(east[0], ["  **"]);
    }

    #[test]
    fn an_active_directional_bang_function_emits_its_self_banging_function() {
        // ADR 0006: each of these "emit[s] the matching root-only Self-Banging
        // Function ... For a producer at `(x, y)`, north emits at `(x, y-1)`,
        // south at `(x,y+1)`, west at `(x-2,y)`, and east at `(x+2,y)`." The
        // horizontal offsets are two columns and the Self-Banging Functions'
        // are one, which is the difference a direction name would have hidden:
        // one emits outside its own Span and the other moves through its own.
        //
        // Each fixture reaches its producer through a different arm of
        // ADR 0006's cardinal rule, because the emission has to land somewhere
        // the Bang is not. Equality Bangs on every Tick, so the Bang display
        // and the emitted spelling are both in the Grid these compare.
        let tall = Grid::new(8, 3);
        let (_, south, _) = tick_by_tick(tall, &[".=0101", "  *v", ""], 1);
        assert_eq!(south[0], [".=0101  ", "***v    ", "  vv    "]);

        let wide = Grid::new(10, 2);
        let (_, east, _) = tick_by_tick(wide, &[".=0101", "  *>"], 1);
        assert_eq!(east[0], [".=0101    ", "***>>>    "]);

        let (_, north, _) = tick_by_tick(wide, &["    .=0101", "  *^"], 1);
        assert_eq!(north[0], ["  ^^.=0101", "  *^**    "]);

        let (_, west, _) = tick_by_tick(wide, &["    .=0101", "  *<"], 1);
        assert_eq!(west[0], ["    .=0101", "<<*<**    "]);
    }

    #[test]
    fn an_inert_directional_bang_function_emits_nothing() {
        // The asymmetry ADR 0029 refuses to collapse, stated as the Grid that
        // does not change. `*>` and `>>` declare the same spelling at the same
        // kind of Portal and differ in where the Turn comes from; with no Bang
        // to deliver one, this Grid stands still where the `>>` test's Grid
        // moves every Tick.
        let (plans, grids, _) = tick_by_tick(Grid::new(8, 2), &["  *>", ""], 2);

        assert_eq!(grids[0], ["  *>    ", "        "]);
        assert_eq!(grids[1], ["  *>    ", "        "]);
        for plan in &plans {
            assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        }
    }

    #[test]
    fn a_refused_emission_diagnoses_and_writes_no_cell() {
        // ADR 0006: "The complete initial destination must be empty and inside
        // the Grid or the producer diagnoses and emits nothing." This is where
        // the two groups differ in what a refusal costs — a Self-Banging
        // Function replaces its own Span with `**` — and the reason is that
        // this producer is not leaving its own Cells, so it has none to say it
        // in. Both refusals are here: Cells that are not empty, and a
        // displacement that leaves the Grid.
        //
        // The diagnostic names the producer and not the Function it would have
        // emitted. The two come apart only for this group — a Self-Banging
        // Function writes its own spelling — and naming the emission alone
        // would point the author at Cells holding `>>` when the Grid contains
        // no `>>` at all. Both spellings are here because both are the author's
        // question: which Cell to fix, and what it was trying to put where.
        let occupied = Grid::new(10, 2);
        let (plans, grids, _) = tick_by_tick(occupied, &[".=0101", "  *>xx"], 1);
        assert_eq!(grids[0], [".=0101    ", "***>xx    "]);
        assert_eq!(
            messages(&plans[0]),
            vec!["*> has no empty destination inside the Grid for >>"]
        );

        let edge = Grid::new(10, 2);
        let (plans, grids, _) = tick_by_tick(edge, &["  .=0101", "*<"], 1);
        assert_eq!(grids[0], ["  .=0101  ", "*<**      "]);
        assert_eq!(
            messages(&plans[0]),
            vec!["*< has no empty destination inside the Grid for <<"]
        );

        // The vertical half of the same refusal, which leaves the Grid by the
        // last row rather than by the first column. It is here because the two
        // reach `Portal::displaced` through different arms of `Grid::displaced`
        // — a row that does not exist against a column that does not — and one
        // fixture proves only the arm it takes.
        //
        // The other two edges are not written out because no Source reaches
        // them. A Bang comes only from the Delay, the Equality, or the
        // Euclidean, each six Cells wide, so a producer at column `C` needs
        // `C + 6 <= W` and the roots its Bang can anchor are `C - 2` and
        // `C + 2`. The rightmost is `W - 4`, whose emission ends at `W - 1` and
        // stays in the row: `*>` cannot be made to cross the row edge. A `*^`
        // in the first row would need a Bang in that row, and nothing writes
        // one there — a producer Bangs into the row below itself, and a
        // Source-resident `**` is cleared before any Turn.
        let floor = Grid::new(8, 2);
        let (plans, grids, _) = tick_by_tick(floor, &[".=0101", "  *v"], 1);
        assert_eq!(grids[0], [".=0101  ", "***v    "]);
        assert_eq!(
            messages(&plans[0]),
            vec!["*v has no empty destination inside the Grid for vv"]
        );
    }

    #[test]
    fn an_emitted_self_banging_function_first_moves_on_the_following_tick() {
        // The whole cycle in one fixture: a Delay Bangs on Tick 0 and on no
        // Tick after it, `*>` writes `>>`, and `>>` moves once per Tick from
        // the Tick after the one that wrote it. ADR 0006's "Generated Functions
        // first receive a turn from the next Source Snapshot" is the assertion
        // on the first Grid — `>>` is at Cells 2 and 3 there, not 3 and 4 —
        // and it needs no rule of its own: a schedule is built from the Source
        // Snapshot, and nothing this Tick wrote is in it.
        //
        // The Delay is what keeps the second half legible. An Equality would
        // Bang again on Tick 1, and `*>` would then be pointing at the Cells
        // its own emission had not yet vacated.
        let (plans, grids, _) = tick_by_tick(Grid::new(12, 3), &["~*0401", "", "*>"], 3);

        assert_eq!(grids[0], ["~*0401      ", "**          ", "*>>>        "]);
        assert_eq!(grids[1], ["~*0401      ", "            ", "*> >>       "]);
        assert_eq!(grids[2], ["~*0401      ", "            ", "*>  >>      "]);
        for plan in &plans {
            assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        }
    }

    #[test]
    fn an_eastward_move_stops_at_the_row_edge_rather_than_wrapping_into_the_next_row() {
        // A row is the whole horizontal extent there is. The Cell after the
        // last column of row 0 exists in the Grid and is the first Cell of row
        // 1, so a displacement alone cannot tell the two apart; the row-edge
        // refusal is what does, and it costs the whole write. The second row
        // stays empty, which is the assertion that says so.
        let (_, grids, _) = tick_by_tick(Grid::new(4, 2), &["  >>", ""], 1);

        assert_eq!(grids[0], ["  **", "    "]);
    }

    #[test]
    fn a_westward_move_stops_at_the_row_edge_rather_than_wrapping_into_the_previous_row() {
        // The mirror, and a different refusal reaching the same cost. Eastward
        // off the last column and westward off the first both stay inside the
        // Grid when the neighbouring row exists, so `Portal::displaced`
        // resolves a destination in each case and the row-edge check is the
        // only thing that refuses it. The single-row edge tests above take the
        // other path, where no Portal resolves at all.
        let (_, grids, _) = tick_by_tick(Grid::new(4, 2), &["", "<<  "], 1);

        assert_eq!(grids[0], ["    ", "**  "]);
    }

    #[test]
    fn two_movers_reserving_each_other_each_bang_rather_than_costing_the_tick() {
        // Each of these reserves a destination that covers the other's Span,
        // so the two reservations name each other. A reservation orders Turns
        // and decides nothing else, per ADR 0036, and neither of these Turns
        // can write where the other stands: a move is admitted only into empty
        // Cells, so mutual reservation describes two blocked moves rather than
        // two writes competing for one Cell. Ordering either after the other
        // would make that pair a cycle and cost the whole Grid its Tick, which
        // is the rejected alternative ADR 0036 names. Both are blocked by
        // complete root contact and both bang, which is what ADR 0006 gives a
        // blocked move whatever blocked it.
        let (plans, grids, _) = tick_by_tick(Grid::new(6, 1), &[">><<  "], 1);

        assert_eq!(grids[0], ["****  "]);
        assert!(
            plans[0].diagnostics.is_empty(),
            "{:?}",
            plans[0].diagnostics
        );

        // The vertical pair shares no Cell at all, so it says the same thing
        // about reservations rather than about the overlap a horizontal move
        // has with its own old Span.
        let (vertical, rows, _) = tick_by_tick(Grid::new(2, 2), &["vv", "^^"], 1);

        assert_eq!(rows[0], ["**", "**"]);
        assert!(
            vertical[0].diagnostics.is_empty(),
            "{:?}",
            vertical[0].diagnostics
        );
    }

    #[test]
    fn a_move_blocked_by_one_complete_language_unit_bangs_without_diagnosing() {
        // ADR 0006: "Complete non-root contact adds no collision diagnostic."
        // The two Cells north of `^^` are the whole of the `01` operand
        // literal, so the contact is complete and the unit is no root. The
        // Addition beside it still answers, which is what says the `**` is
        // this Function reporting its own blocked move rather than the Tick
        // going wrong around it.
        let (plans, grids, _) = tick_by_tick(Grid::new(8, 3), &[".+0102", "  ^^", ""], 1);

        assert_eq!(grids[0], [".+0102  ", "03**    ", "        "]);
        assert!(
            plans[0].diagnostics.is_empty(),
            "{:?}",
            plans[0].diagnostics
        );
    }

    #[test]
    fn a_move_that_lands_across_two_language_units_diagnoses_its_misalignment() {
        // The other half of the same rule: these two Cells hold the last Cell
        // of `01` and the first Cell of `02`, so neither unit is met whole.
        // ADR 0006 diagnoses and delivers nothing, and the move is blocked
        // exactly as a complete contact blocks it — the diagnostic is the only
        // difference, because a misalignment is the one outcome the `**` alone
        // does not tell a Source author about.
        let (plans, grids, _) = tick_by_tick(Grid::new(8, 3), &[".+0102", "   ^^", ""], 1);

        assert_eq!(grids[0], [".+0102  ", "03 **   ", "        "]);
        assert_eq!(
            messages(&plans[0]),
            vec!["^^ contacts part of a Language Unit"]
        );
    }

    #[test]
    fn complete_aligned_root_contact_activates_the_root_it_blocked_against() {
        // ADR 0006: "Complete aligned root contact also directly delivers Bang
        // activation." The Raw Play north of `^^` is inert on its own, so the
        // Play Command is the whole evidence that activation was delivered —
        // and the `**` is the evidence the move was still blocked, because
        // ADR 0006 gives contact both outcomes and not a choice between them.
        let (plans, grids, _) = tick_by_tick(Grid::new(8, 2), &["!>007FC4", "^^"], 1);

        assert_eq!(grids[0], ["!>007FC4", "**      "]);
        assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 60)]);
        assert!(
            plans[0].diagnostics.is_empty(),
            "{:?}",
            plans[0].diagnostics
        );
    }

    #[test]
    fn a_horizontally_aligned_root_is_contacted_two_columns_away() {
        // The same rule where the geometry differs: a horizontal move enters
        // one Cell, and the root whose Span holds that Cell is anchored two
        // columns from the producer, because every Language Unit spells as a
        // Cell pair. That is ADR 0006's east anchor `(x+2, y)` reached by
        // contact rather than by a Source-resident Bang, and it is why the
        // contact rule asks which unit covers the Cells entered rather than
        // which unit begins at them.
        let (plans, grids, _) = tick_by_tick(Grid::new(10, 2), &[">>!>007FC4", ""], 1);

        assert_eq!(grids[0], ["**!>007FC4", "          "]);
        assert_eq!(plans[0].play_commands, vec![raw(0, 0x7F, 60)]);
        assert!(
            plans[0].diagnostics.is_empty(),
            "{:?}",
            plans[0].diagnostics
        );
    }

    #[test]
    fn the_schedule_orders_a_move_ahead_of_the_computation_it_would_land_on() {
        // Reserving the destination Portal is what puts these two Turns in an
        // order, and the order is what keeps the move honest: `>>` points at
        // the Cell `^^` stands in, so `>>` takes its Turn first and reads the
        // Source Snapshot's own answer — blocked, by a complete root, so `**`
        // and an activation `^^` already had. `^^` then moves north out of the
        // way it never had to give.
        //
        // Without that edge `^^` could move first and `>>` would find the Cell
        // empty, planning a write over Cells a computation that had already
        // run was scheduled at. That is the defect ADR 0034 rejects a Tick for,
        // and this is the ordering that stops it arising.
        let (plans, grids, _) = tick_by_tick(Grid::new(6, 2), &["", ">>^^  "], 1);

        assert_eq!(grids[0], ["  ^^  ", "**    "]);
        assert!(
            plans[0].diagnostics.is_empty(),
            "{:?}",
            plans[0].diagnostics
        );
    }

    #[test]
    fn a_self_banging_function_is_not_scheduled_inside_another_expression() {
        // Root-only, enforced by the declared kind rather than by a check that
        // names these four spellings: `.+` declares two Number operands, `^^`
        // answers an effect, and ADR 0028's nesting rule refuses it where a
        // value is required. The refusal is the Expression's, so the Cells are
        // left standing and nothing moves.
        let (plans, grids, _) = tick_by_tick(Grid::new(8, 2), &[".+^^01", ""], 1);

        assert_eq!(grids[0], [".+^^01  ", "        "]);
        assert_eq!(
            messages(&plans[0]),
            vec![
                "a Function that answers an effect is valid only at the root of an Expression",
                "nested computation at column 2, row 0 supplied no typed result",
            ]
        );
    }

    /// `carry` splices its refusals in front of the diagnostics it was handed,
    /// and this states the order that produces. The fixture earns both a
    /// refused Portal, which only `carry` raises, and a row-edge layout
    /// diagnostic, which `computations` raised before `carry` ran — so the
    /// refusal arriving first is the splice and nothing else.
    #[test]
    fn a_refused_portal_is_diagnosed_before_the_row_edge_layout_it_shares_a_tick_with() {
        let grid = Grid::new(16, 4);
        let rows = ["!>007FC4", "", "", "            .+01"];

        let (plan, _) = carried_source(grid, &rows, &[(0, 16)]);

        assert_eq!(
            plan.diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "a Terminal Output Function cannot have a Portal",
                "Expression layout crosses the row edge",
            ]
        );
    }

    ///
    /// Runs one Tick in which the computations at `answers` state the value
    /// they answer rather than computing one, and commits its plan.
    ///
    /// ADR 0034 defers the Source operation that produces Function values, and
    /// no Function answers a bare Cell either, so a test that needs one of
    /// those constructs it. Where it is constructed is the whole point: the
    /// value is delivered by `stated::plan_with_answers`, one call below the
    /// planning entry point, so the Source, its destinations, the schedule and
    /// every other computation's Turn are the production ones and nothing in
    /// the shipped module compiles differently to admit the answer. The plan is
    /// committed through `Source::commit_tick`, which is the commit a Tick gets
    /// however it was planned, rather than through edits that imitate it.
    ///
    fn stated_source(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
        reservations: &[(usize, super::Reserved)],
        answers: &[(usize, Value)],
    ) -> (TickPlan, crate::source::Source) {
        let (plan, _, source) = stated_tick(grid, rows, outputs, reservations, answers);
        (plan, source)
    }

    ///
    /// [`stated_source`], with the inputs the Interpreter received for each
    /// computation it ran for. Split from it for the reason [`carried_tick`]
    /// gives.
    ///
    fn stated_tick(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
        reservations: &[(usize, super::Reserved)],
        answers: &[(usize, Value)],
    ) -> (TickPlan, Vec<lang::TickInputs>, crate::source::Source) {
        let mut source = seeded_source(grid, rows);
        let reservations: Vec<_> = reservations
            .iter()
            .map(|(anchor, reserved)| (cell(grid, *anchor), *reserved))
            .collect();
        let answers: Vec<_> = answers
            .iter()
            .map(|(anchor, value)| (cell(grid, *anchor), value.clone()))
            .collect();
        let bytes = source.snapshot();
        let (plan, states) = super::execution::stated::plan_with_answers(
            grid,
            bytes.as_bytes(),
            &source.shared_language_map(),
            Tick::ZERO,
            &carried_destinations(grid, outputs),
            &reservations,
            &answers,
        );
        source.commit_tick(&plan);
        (plan, interpreted(&states), source)
    }

    ///
    /// One Tick in which the computations at `replacements` answer a Function
    /// value, which nothing spells in Source until ADR 0034's deferred
    /// Function-producing operation exists.
    ///
    fn replaced_source(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
        replacements: &[(usize, lang::Function)],
    ) -> (TickPlan, crate::source::Source) {
        let (plan, _, source) = replaced_tick(grid, rows, outputs, replacements);
        (plan, source)
    }

    ///
    /// [`replaced_source`], with the inputs the Interpreter received for each
    /// computation it ran for. Split from it for the reason [`carried_tick`]
    /// gives.
    ///
    fn replaced_tick(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
        replacements: &[(usize, lang::Function)],
    ) -> (TickPlan, Vec<lang::TickInputs>, crate::source::Source) {
        stated_tick(
            grid,
            rows,
            outputs,
            &[],
            &replacements
                .iter()
                .map(|(anchor, function)| (*anchor, Value::Atom(lang::Atom::Function(*function))))
                .collect::<Vec<_>>(),
        )
    }

    ///
    /// One Tick in which the computations at `answers` are the Sequence-
    /// answering rows ADR 0007's Range and Concatenate will spell: each
    /// reserves the Cells ADR 0036 gives such a row — its destination through
    /// the end of that row — and each answers the Numbers stated for it.
    ///
    /// The two facts are stated together here, in one place, because a
    /// declared Range row states both together too: what it answers is a fact
    /// of the Tick and what it reserves is a fact of the schedule, and no
    /// fixture below derives either from the other. That derivation is the one
    /// thing these tests cannot prove while no Function declares a Sequence
    /// answer, which is why the test that asserted it is owed by
    /// `sequence-values/05` rather than stated here.
    ///
    fn sequence_source(
        grid: Grid,
        rows: &[&str],
        outputs: &[(usize, usize)],
        answers: &[(usize, &[u8])],
    ) -> (TickPlan, crate::source::Source) {
        stated_source(
            grid,
            rows,
            outputs,
            &answers
                .iter()
                .map(|(anchor, _)| (*anchor, super::Reserved::Row))
                .collect::<Vec<_>>(),
            &answers
                .iter()
                .map(|(anchor, values)| (*anchor, Value::Sequence(sequence(values))))
                .collect::<Vec<_>>(),
        )
    }

    ///
    /// One Sequence of Numbers, or the empty Sequence for no Numbers at all.
    ///
    fn sequence(values: &[u8]) -> lang::Sequence {
        lang::Sequence::new(values.iter().copied().map(lang::Atom::Number))
            .expect("a Number is a Sequence member")
    }

    #[test]
    fn a_stated_answer_faces_every_refusal_the_turn_it_replaces_faces() {
        // What a computation answers is the only thing a stated answer states.
        // Whether it answers at all is the Turn's to decide, and a Turn its own
        // prologue refuses has no answer to deliver, stated or interpreted. A
        // seam that delivered past those refusals would let these tests assert
        // outcomes production cannot produce, which is the one way stating an
        // answer here could be worse than the `cfg` fork it replaces.

        // An Addition whose second operand is Source it cannot read: the Turn
        // is syntax-blocked, which settles it without a Tick diagnostic.
        let grid = Grid::new(16, 2);
        let rows = [".+02", ""];
        let (plan, source) = stated_source(
            grid,
            &rows,
            &[(0, 16)],
            &[],
            &[(0, Value::Atom(lang::Atom::Number(1)))],
        );
        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert!(plan.play_commands.is_empty());
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);

        // A Terminal Output Function standing inside an Expression: its Turn
        // records the refusal ADR 0028 states, and the Addition above it is
        // left with no typed result to consume.
        let rows = [".+01!>007FC4", ""];
        let (plan, source) = stated_source(
            grid,
            &rows,
            &[(0, 16)],
            &[],
            &[(4, Value::Atom(lang::Atom::Number(1)))],
        );
        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert!(plan.play_commands.is_empty());
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
        assert!(
            plan.diagnostics.iter().any(|d| d
                .message
                .contains("valid only at the root of an Expression")),
            "{:?}",
            plan.diagnostics
        );
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("supplied no typed result")),
            "{:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn a_stated_reservation_leaves_every_derived_reservation_consistent_with_it() {
        // The seam's own regression, not ADR 0036's widening rule: what a
        // fixture states is one computation's reservation, and every other
        // reservation in the Grid still has to be the one production derives
        // beside it. The nested `.-` at column 2 reserves a row here, so the
        // pervasive `.+` that owns it reserves one too — and its own answer of
        // three Atoms across six Cells is admitted rather than refused as a
        // result that is not the Cell pair the schedule reserved.
        //
        // ADR 0036's rule that a Sequence-answering child widens its ancestor
        // is not what this proves, because the child's width is stated rather
        // than declared. That is owed by `sequence-values/05` against a Range
        // row, and this test can go when it lands.
        //
        // The premise the width above rests on, pinned so it cannot go quiet:
        // the root reserves a Row only by widening over its child. Were Add to
        // declare a Sequence answer of its own, the six Cells below would still
        // be written and this test would pass while exercising nothing.
        assert!(
            !lang::Function::Add.answers_sequence()
                && lang::Function::Add.widens_over_a_sequence_operand(),
            "the root's reservation can only have been derived from its child",
        );

        let grid = Grid::new(16, 2);
        let (plan, source) = stated_source(
            grid,
            &["                ", ".+.-000003"],
            &[(16, 0)],
            &[(18, super::Reserved::Row)],
            &[(16, Value::Sequence(sequence(&[0x0A, 0x0B, 0x0C])))],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(plan.writes.len(), 6);
        assert_eq!(source.snapshot(), snapshot(grid, &["0A0B0C", ".+.-000003"]));
    }

    #[test]
    fn a_reservation_agrees_with_the_width_its_own_declaration_derives() {
        // The agreement `Lookup::would_reserve` is a hypothesis against: asked
        // with the Function a computation actually declares, it answers the
        // width that computation reserves. `Lookup::new` asserts it over every
        // Source any test in this crate builds; this states it as the fact it
        // is, and states the one width it is false of.
        //
        // A stated width is that one. It is not a declared one, so re-deriving
        // it narrows it away — which is why `plan_with_answers` refuses to
        // combine a stated reservation with a stated Function replacement. The
        // ancestor widened over it is what gives this test its teeth: without
        // it every reservation in the crate is a Cell pair and an agreement
        // between two answers of `Pair` proves nothing. The pervasive `.+`
        // reserves a Row derived from its stated child, and `would_reserve`
        // has to answer `Row` for it.
        let grid = Grid::new(16, 2);
        let mut source = crate::source::Source::new(grid);
        for (index, byte) in snapshot(grid, &["                ", ".+.-000003"])
            .bytes()
            .enumerate()
        {
            source
                .set(cell(grid, index), &char::from(byte).to_string())
                .unwrap();
        }
        let (nodes, _) = super::computations(grid, &source.shared_language_map());
        let mut lookup = super::Lookup::new(grid, nodes);

        // Parser preorder: the owning `.+` at column 0, then the `.-` nested
        // in its first operand.
        assert_eq!(lookup.nodes().len(), 2);
        let (root, child) = (0, 1);
        assert_eq!(lookup.nodes()[child].parent, Some(root));
        for index in [root, child] {
            assert_eq!(
                lookup.would_reserve(index, lookup.nodes()[index].function),
                lookup.reserved(index),
                "computation {index} reserves a width its own declaration does not derive"
            );
            assert_eq!(lookup.reserved(index), super::Reserved::Pair);
        }

        lookup.nodes[child].reserved = super::Reserved::Row;
        super::derive_reservations(&mut lookup.nodes);

        assert_eq!(
            lookup.reserved(root),
            super::Reserved::Row,
            "a pervasive Function widens over an operand that reserves a row"
        );
        assert_eq!(
            lookup.would_reserve(root, lookup.nodes()[root].function),
            lookup.reserved(root),
            "the widened reservation is the one the root's own declaration derives"
        );
        assert_eq!(
            lookup.would_reserve(child, lookup.nodes()[child].function),
            super::Reserved::Pair,
            "a stated width is not a declared one, and re-deriving it narrows it away"
        );
    }

    #[test]
    fn a_computation_over_any_function_reserves_a_cell_pair() {
        // ADR 0036 reserves a result's Cells from the Function found at each
        // anchor, and `ReplacementChange::Width` refuses a replacement that
        // would change that reservation. The guard compares
        // `would_reserve(index, replacement)` against `reserved(index)`, so the
        // term can fire only where those two can be different widths — and this
        // is where that can be shown, because a reservation is derived from a
        // Grid and from the widths a computation's children settled, neither of
        // which `lang` has. `no_function_declares_a_sequence_answer` there
        // holds the other half: no Function answers a Sequence, so nothing
        // widens.
        //
        // A failure here is not something breaking. It means a Function now
        // reaches a schedule declaring a width wider than a Cell pair, so the
        // width term has become reachable and the guard's fifth fact is live
        // for the first time.
        //
        // Both a root and the child nested in its first operand are asked,
        // because `reserved_for` treats the two positions differently: the
        // child is a leaf over literals with nothing to widen from, and the
        // root owns an operand child whose settled width it reads.
        //
        // The premise that makes those two positions different, pinned so it
        // cannot go quiet: some Function widens over a Sequence operand, so
        // `reserved_for` reaches the `.any()` that reads a child's settled
        // width. Were no Function to widen, that term would short-circuit for
        // the root exactly as it does for the leaf, and this test would pass
        // while asking one question twice.
        assert!(
            lang::Function::ALL
                .iter()
                .any(|function| function.widens_over_a_sequence_operand()),
            "no Function widens, so the root and the child are the same question",
        );

        let grid = Grid::new(16, 2);
        let source = seeded_source(grid, &["                ", ".+.-000003"]);
        let (nodes, _) = super::computations(grid, &source.shared_language_map());
        let lookup = super::Lookup::new(grid, nodes);

        // Parser preorder: the owning `.+` at column 0, then the `.-` nested in
        // its first operand.
        assert_eq!(lookup.nodes().len(), 2);
        let (root, child) = (0, 1);
        assert_eq!(lookup.nodes()[child].parent, Some(root));

        for index in [root, child] {
            assert_eq!(
                lookup.reserved(index),
                super::Reserved::Pair,
                "computation {index} settled a width wider than a Cell pair",
            );
            for function in lang::Function::ALL.iter().copied() {
                assert_eq!(
                    lookup.would_reserve(index, function),
                    super::Reserved::Pair,
                    "{function:?} replacing computation {index} would reserve more than a Cell pair",
                );
            }
        }
    }

    #[test]
    #[should_panic(expected = "a stated answer names a computation the schedule contains")]
    fn a_cyclic_source_does_not_excuse_a_fixture_error() {
        // A Source that admits no order publishes diagnostics and nothing
        // else, which is indistinguishable from what a mistyped fixture makes
        // happen — so a fixture error checked after ordering would be the one
        // check a cycle switches off. The Addition writes over its own operand
        // and the answer names an operand Cell: both are wrong, and the one
        // the fixture author can fix is the one reported.
        let grid = Grid::new(16, 2);
        stated_source(
            grid,
            &[".+0102", ""],
            &[(0, 2)],
            &[],
            &[(4, Value::Atom(lang::Atom::Number(1)))],
        );
    }

    #[test]
    #[should_panic(expected = "two are stated here for the same anchor")]
    fn two_answers_at_one_anchor_are_a_fixture_error() {
        // The second answer could only ever be dropped: one computation takes
        // one Turn. Reported as the duplicate it is rather than as the Turn
        // that appeared not to reach it.
        let grid = Grid::new(16, 2);
        stated_source(
            grid,
            &[".+0203", ""],
            &[(0, 16)],
            &[],
            &[
                (0, Value::Atom(lang::Atom::Number(1))),
                (0, Value::Atom(lang::Atom::Number(2))),
            ],
        );
    }

    #[test]
    #[should_panic(expected = "a stated answer belongs to a computation that answers a value")]
    fn a_stated_answer_at_a_terminal_output_root_is_a_fixture_error() {
        // A root Terminal Output Function is the one Function a Turn lets
        // answer no value at all, and what it answers is a Play Command. A
        // fixture stating a value there would have this seam plan a write no
        // Turn of that computation can produce.
        //
        // The Equality above it answers the Bang that activates it, because an
        // unactivated Terminal Output root takes no Turn at all and would be
        // refused a Cell before reaching the arm under test.
        let grid = Grid::new(16, 3);
        stated_source(
            grid,
            &[".=0101", "", "!>007FC4"],
            &[(0, 16)],
            &[],
            &[(32, Value::Atom(lang::Atom::Number(1)))],
        );
    }

    #[test]
    #[should_panic(expected = "a stated reservation names a computation the schedule contains")]
    fn a_stated_reservation_at_no_computations_anchor_is_a_fixture_error() {
        // The same fixture error as below, for the other half of what a
        // Sequence-answering row states. A reservation stated at a Cell no
        // computation is anchored at would leave the schedule deriving every
        // width itself, and a Sequence answer would then be refused for a
        // reason that has nothing to do with what the test is asking.
        let grid = Grid::new(16, 2);
        stated_source(
            grid,
            &[".+0203", ""],
            &[(0, 16)],
            &[(2, super::Reserved::Row)],
            &[],
        );
    }

    #[test]
    #[should_panic(expected = "a stated reservation is a width production would not derive")]
    fn a_stated_pair_reservation_is_a_fixture_error() {
        // The one fixture mistake the seam could answer silently. Stating a
        // reservation is how a fixture says what production cannot derive yet,
        // and `Reserved::Pair` is the width production derives for everything:
        // `derive_reservations` re-derives every node still holding one, so a
        // stated Pair is overwritten by the very pass that reads it. Here the
        // pervasive `.+` at Cell 16 would widen over the row-reserving `.-` at
        // Cell 18 and answer Row again, leaving a test that states the parent
        // does not widen asserting the opposite of what it says and passing.
        let grid = Grid::new(16, 2);
        stated_source(
            grid,
            &["                ", ".+.-000003"],
            &[(16, 0)],
            &[(18, super::Reserved::Row), (16, super::Reserved::Pair)],
            &[(16, Value::Sequence(sequence(&[0x0A, 0x0B, 0x0C])))],
        );
    }

    #[test]
    #[should_panic(expected = "a stated answer names a computation the schedule contains")]
    fn a_stated_answer_at_no_computations_anchor_is_a_fixture_error() {
        // Cell 2 holds the Addition's first operand rather than a Function
        // anchor, so nothing in the schedule would ever state this answer.
        // Falling through to an ordinary Tick would leave a test asking
        // whether little happened and being told that it did, which is what
        // this refuses to do quietly.
        let grid = Grid::new(16, 2);
        stated_source(
            grid,
            &[".+0203", ""],
            &[(0, 16)],
            &[],
            &[(2, Value::Atom(lang::Atom::Number(1)))],
        );
    }

    #[test]
    fn live_deep_sibling_computations_preserve_operand_order() {
        // Each sibling requires more pending operands than the Parser keeps
        // inline. The second refills that stack after the first has drained it.
        let numerator = ".+".repeat(32) + &"02".repeat(33);
        let denominator = ".+".repeat(32) + &"01".repeat(33);
        let text = format!("./{numerator}{denominator}");
        let width = text.len();
        let (plan, source) = carried_source(Grid::new(width, 2), &[&text, ""], &[]);
        // 66 / 33 = 2. Reversing the siblings instead produces zero.
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert!(plan.play_commands.is_empty());
        assert_eq!(source.snapshot(), snapshot(source.grid(), &[&text, "02"]));
    }

    #[test]
    fn live_unchanged_nested_syntax_errors_do_not_repeat_as_tick_failures() {
        let grid = Grid::new(20, 2);
        let rows = [".+01.x02.+03??", ""];
        let (plan, mut source) = carried_source(grid, &rows, &[]);
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
        let (repaired, source) = carried_source(
            Grid::new(20, 3),
            &[".+01.x02.+03??", ".+0004", ""],
            &[(0, 40), (20, 12)],
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
        // A single Cell at the last Cell of a row: the Portal admits it and the
        // schedule never reserved it, which is the pair of facts this rejection
        // is about. No Function answers a bare Char, so the answer is stated.
        let (plan, source) = stated_source(
            grid,
            &rows,
            &[(0, 31)],
            &[],
            &[(0, Value::Atom(lang::Atom::Char('7')))],
        );
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
        let (plan, source) = carried_source(
            Grid::new(16, 3),
            &["!>007F.^3C", ".+0203", ""],
            &[(6, 8), (16, 32)],
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
        let (plan, source) = carried_source(Grid::new(8, 2), &[".||102", ""], &[]);

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
        let (plan, source) = carried_source(Grid::new(8, 2), &[".|0102||", ""], &[]);

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
        let (plan, source) = carried_source(Grid::new(8, 2), &["** :##", ""], &[]);

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
        let (plan, source) = carried_source(Grid::new(16, 2), &[".+01 02", ""], &[]);
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
        let (plan, source) = carried_source(Grid::new(16, 2), &[".+0102.+0304", ""], &[]);
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
        let (plan, source) = carried_source(Grid::new(16, 2), &[".+0102Z", ""], &[]);
        assert_eq!(&source.snapshot()[16..18], "03");
        assert!(plan.diagnostics.is_empty());
        assert!(source.language_map().diagnostics().any(|d| d.start() == 6));
        let (plan, source) = carried_source(Grid::new(16, 2), &["***", ""], &[]);
        assert_eq!(&source.snapshot()[..3], "  *");
        assert_eq!(plan.writes.len(), 2);
        assert!(source.language_map().diagnostics().any(|d| d.start() == 2));
        let (plan, source) = carried_source(Grid::new(16, 2), &[".=0101 !>007FC4", ""], &[]);
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
        let (plan, source) = carried_source(Grid::new(16, 2), &["C4 EA 01", ""], &[]);
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
        let (plan, source) = carried_source(
            Grid::new(16, 3),
            &["    .=0101", "!>00", "    !>007FC4"],
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
        let (plan, source) = replaced_source(
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
                .any(|d| { d.message.contains("whether it answers a value") })
        );
    }

    #[test]
    fn a_replacement_that_changes_only_the_activation_source_is_refused() {
        // The term the Self-Banging Functions added to that guard. Raw Play and
        // `^^` differ on where their activation comes from, and that is the
        // first difference the guard finds: neither answers a value, so the
        // term ahead of it agrees. They differ on the Source write as well —
        // `^^` declares one and Raw Play declares none — so this fixture
        // changes two facts and is named for the one that is reported.
        //
        // No pair changes activation alone. Every Function whose activation is
        // intrinsic either answers a value or declares a Source write, and no
        // Bang-activated Function does either, so a fixture that changed this
        // fact and nothing else cannot be written;
        // `every_declared_change_is_the_first_difference_for_some_pair` in
        // `lang` holds that. A guard still asking `answers_value` for this
        // question would admit the replacement and leave the schedule holding
        // edges derived from a root that now needs no Bang.
        let (plan, source) = replaced_source(
            Grid::new(16, 3),
            &[".+0000", "!>007FC4", ""],
            &[(0, 16)],
            &[(0, lang::Function::SelfBangingNorth)],
        );

        assert_eq!(&source.snapshot()[16..24], "!>007FC4");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| { d.message.contains("where its activation comes from") })
        );
    }

    #[test]
    fn a_replacement_that_changes_only_bang_emission_is_refused() {
        // The one fact of the five that a pair can differ on alone and that
        // nothing asserted until now. Equality and Addition agree on every
        // other column the guard reads — both answer a value, both are
        // intrinsically active, neither declares a Source write, both reserve a
        // Cell pair — and ADR 0011's Equality can answer Bang where Addition
        // never can.
        //
        // Scheduling reads that declaration to decide which roots can supply
        // activation, so admitting this replacement would leave the Turn run by
        // a Function that can answer Bang with no activation edge built from
        // it, and a neighbouring root waiting on a Bang the schedule never
        // ordered.
        let (plan, source) = replaced_source(
            Grid::new(16, 3),
            &[".+0000", ".+0102", ""],
            &[(0, 16)],
            &[(0, lang::Function::Equality)],
        );

        assert_eq!(
            &source.snapshot()[16..22],
            ".+0102",
            "the refused replacement wrote no Cell, so the parsed Function stands"
        );
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| { d.message.contains("whether it can emit Bang") }),
            "{:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn a_replacement_that_changes_only_the_declared_portal_offset_is_refused() {
        // Two Self-Banging Functions agree on every other column the guard
        // reads — neither answers a value, both are intrinsically active,
        // neither can return Bang, and both reserve a Cell pair — and they
        // differ only in the Portal offset they declare. The schedule reserved
        // the Cells `^^` declares, so admitting `>>` here would leave the Turn
        // writing at Cells no dependency edge names, which is the ADR 0036
        // defect this guard exists to refuse.
        let (plan, source) = replaced_source(
            Grid::new(16, 3),
            &[".+0000", "^^", ""],
            &[(0, 16)],
            &[(0, lang::Function::SelfBangingEast)],
        );

        // `^^` stays the running Function and takes its own Turn, which the
        // Addition to its north blocks, so its Span is the Bang a blocked move
        // leaves. That is the assertion: an admitted replacement moves east
        // instead, leaving a space and a `>` here and writing the second `>`
        // one Cell outside the reservation.
        assert_eq!(&source.snapshot()[16..18], "**");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| { d.message.contains("the Source write it declares") }),
            "{:?}",
            plan.diagnostics
        );
    }

    #[test]
    fn live_the_second_replacement_at_one_anchor_replaces_the_first() {
        // The guard reads the Function the computation is running, and the
        // only Tick that can tell that from the Function the Parser found is
        // one in which two replacements reach a single anchor. Both are
        // admitted here, so the Turn taken at column 0 is the second one's:
        // Multiplication, not the Subtraction that replaced the parsed
        // Addition before it.
        //
        // Reading the parsed Function answered this the same way, and no
        // fixture can make the two disagree: this guard admits no replacement
        // that changes any of the three facts it compares, so the running
        // Function agrees with the parsed one on all three for as long as the
        // guard is the only thing that changes it. That agreement is an
        // invariant about the guard itself, held nowhere and by nothing else,
        // and reading the running Function is what stops it being load-bearing.
        let grid = Grid::new(16, 3);
        let (plan, source) = replaced_source(
            grid,
            &[".+0503", ".x0405", ".-0607"],
            &[(0, 40), (16, 0), (32, 0)],
            &[
                (16, lang::Function::Subtract),
                (32, lang::Function::Multiply),
            ],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            &source.snapshot()[..2],
            ".x",
            "the later producer wins the Cells both replacements wrote"
        );
        assert_eq!(
            &source.snapshot()[40..42],
            "0F",
            "the Turn ran the second replacement, not the first and not the parsed Function"
        );
    }

    #[test]
    fn live_a_second_replacement_at_one_anchor_faces_the_same_refusal() {
        // The same anchor, reached twice, where the second replacement changes
        // the output kind: it is refused, its write is refused whole, and the
        // first replacement stands as the Function the Turn runs. The refusal
        // is stated against the Subtraction already in place rather than
        // against the parsed Addition; the two answer alike, which is the
        // invariant the test above records.
        let grid = Grid::new(16, 3);
        let (plan, source) = replaced_source(
            grid,
            &[".+0503", ".x0405", ".-0607"],
            &[(0, 40), (16, 0), (32, 0)],
            &[
                (16, lang::Function::Subtract),
                (32, lang::Function::RawPlay),
            ],
        );

        assert!(
            plan.diagnostics
                .iter()
                .any(|d| { d.message.contains("whether it answers a value") }),
            "{:?}",
            plan.diagnostics
        );
        assert_eq!(
            &source.snapshot()[..2],
            ".-",
            "the refused replacement wrote no Cell, so the admitted one stands"
        );
        assert_eq!(
            &source.snapshot()[40..42],
            "02",
            "the Turn ran the replacement that was admitted"
        );
    }

    #[test]
    fn live_a_sequence_result_reaches_its_destination_cells() {
        // ADR 0007's ordinary Sequence result, reached through a Tick rather
        // than through the Portal on its own: the schedule holds the row this
        // fixture reserves, execution encodes the answer, and the Source Grid
        // the next Tick reads carries all six Cells. Three Atoms rather than
        // one is what separates this from the scalar case it now shares a path
        // with — including ADR 0036's width guard, which refuses any answer
        // that is not a Cell pair from a computation reserving one.
        let grid = Grid::new(16, 2);
        let (plan, source) =
            sequence_source(grid, &[".+0102", ""], &[], &[(0, &[0x0A, 0x0B, 0x0C])]);

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
        let (plan, source) = sequence_source(grid, &rows, &[(0, 28)], &[(0, &[0x0A, 0x0B, 0x0C])]);

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
        let (plan, source) = sequence_source(grid, &below, &[], &[(16, &[0x0A, 0x0B])]);
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
        let (plan, source) = sequence_source(grid, &rows, &[(0, 30)], &[(0, &[0x0A, 0x0B])]);
        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert_eq!(source.snapshot(), snapshot(grid, &rows));
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("crosses the row edge")),
            "{:?}",
            plan.diagnostics
        );

        let (plan, source) = sequence_source(grid, &rows, &[(0, 30)], &[(0, &[0x0A])]);
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
        let (plan, source) = sequence_source(
            grid,
            &[".+0000.+0000", ""],
            &[(0, 16), (6, 20)],
            &[(0, &[0x0A, 0x0B, 0x0C]), (6, &[0x0D, 0x0E])],
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
        let (plan, source) = sequence_source(grid, &rows, &[], &[(0, &[])]);

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
        let (plan, source) = sequence_source(
            grid,
            &["        .+0102", ".+0000"],
            &[(16, 0)],
            &[(16, &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F])],
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
    fn live_a_reservation_the_sequence_stopped_short_of_suppresses_nothing() {
        // The other half of ADR 0036: a reservation is deliberately wider than
        // most of the writes it covers, and only the write decides what
        // happened to a Cell. The Expression at column 8 is inside the reserved
        // row and outside the four Cells the Sequence actually reached, so it
        // is ordered after the producer and then executes normally, answering
        // `03` into row 1. Suppressing everything the reservation names would
        // leave that Cell pair empty.
        let grid = Grid::new(16, 2);
        let (plan, source) = sequence_source(
            grid,
            &["        .+0102", ".+0000"],
            &[(16, 0)],
            &[(16, &[0x0A, 0x0B])],
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
        let (plan, source) = sequence_source(
            grid,
            &["        .+0102", ""],
            &[(8, 0)],
            &[(8, &[0x0A, 0x0B])],
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
        let (plan, source) = sequence_source(
            grid,
            &rows,
            &[(8, 0)],
            &[(8, &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F])],
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

    #[test]
    fn live_a_row_reservation_names_no_computation_of_the_next_row() {
        // ADR 0036 reserves the rest of the destination's row, and "the rest"
        // is counted from the destination's own column: a destination at
        // column 8 of a sixteen-column Grid reserves eight Cells, not sixteen.
        // Counting the row's full width instead reserves eight Cells of the
        // row below as well, and a reservation names dependency edges over
        // every Cell it covers — so the surplus would order Turns against
        // computations no write from this destination can ever reach.
        //
        // Both producers point at column 8 of row 0 and neither one's spelling
        // lies inside the other's reservation, so nothing orders them against
        // each other and they take their Turns in anchor order. The Sequence
        // is anchored last, takes the later Turn, and wins the two Cells the
        // two writes contest. A reservation running on into row 1 would cover
        // the Addition's spelling and its literals, order that Addition after
        // the Sequence, and hand those two Cells to it instead.
        let grid = Grid::new(16, 3);
        let (plan, source) = sequence_source(
            grid,
            &["", ".+0102", ".+0000"],
            &[(16, 8), (32, 8)],
            &[(32, &[0x0A, 0x0B])],
        );

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            source.snapshot(),
            snapshot(grid, &["        0A0B", ".+0102", ".+0000"]),
            "the Sequence took the later Turn and won the Cells it contests",
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
            let (plan, source) = carried_source(grid, &rows, &outputs);
            proptest::prop_assert_eq!(source.snapshot().into_bytes(), expected);
            proptest::prop_assert!(plan.diagnostics.is_empty());
            let (repeated, _) = carried_source(grid, &rows, &outputs);
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
            let (plan, source) = carried_source(grid, &rows, &[(0, 31)]);
            proptest::prop_assert!(plan.writes.is_empty());
            proptest::prop_assert_eq!(source.snapshot(), snapshot(grid, &rows));
            proptest::prop_assert!(plan.diagnostics.iter().any(|d| d.message.contains("crosses the row edge")));
        }
    }
    #[test]
    fn live_cross_boundary_chain_uses_lower_producers() {
        let (plan, source) = carried_source(
            Grid::new(16, 5),
            &[".+0101", ".+0001", ".+0001", "", ""],
            &[(0, 64), (16, 34), (32, 3)],
        );
        assert_eq!(&source.snapshot()[..6], ".+0021");
        assert_eq!(&source.snapshot()[64..66], "21");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    #[test]
    fn live_competing_writers_follow_position_and_emissions_follow_their_destinations() {
        let (plan, source) = carried_source(
            Grid::new(16, 4),
            &[".+0101", ".+0101", ".+0102", ""],
            &[(0, 48), (16, 3), (32, 3)],
        );
        assert_eq!(&source.snapshot()[..6], ".+0031");
        assert_eq!(&source.snapshot()[48..50], "31");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        let (plan, interpreted, source) = carried_tick(
            Grid::new(16, 3),
            &[".+0101", ".+0203", ""],
            &[(0, 32), (16, 2), (16, 3)],
        );
        assert_eq!(&source.snapshot()[..6], ".+0051");
        assert_eq!(&source.snapshot()[32..34], "51");
        assert_eq!(interpreted.len(), 2);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    #[test]
    fn live_pending_note_decodes_only_after_all_writers_settle() {
        let (plan, source) = carried_source(
            Grid::new(16, 4),
            &[".vE4", ".+E901", ".+E401", ""],
            &[(0, 48), (16, 2), (32, 2)],
        );
        assert_eq!(&source.snapshot()[..4], ".vE5");
        assert_eq!(&source.snapshot()[48..50], "4C");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        let (plan, source) = carried_source(
            Grid::new(16, 3),
            &[".vE4", ".+E901", ""],
            &[(0, 32), (16, 2)],
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
        let (plan, source) = carried_source(
            Grid::new(16, 3),
            &[".+0000", ".+E901", ""],
            &[(0, 32), (16, 2)],
        );
        assert_eq!(&source.snapshot()[32..34], "EA");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    }

    #[test]
    fn live_spatial_note_is_an_encoding_and_nested_note_stays_typed() {
        let (plan, source) = carried_source(
            Grid::new(16, 3),
            &[".+0001", ".^48", ""],
            &[(0, 32), (16, 2)],
        );
        assert_eq!(&source.snapshot()[..6], ".+C501");
        assert_eq!(&source.snapshot()[32..34], "C6");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        let (plan, source) = carried_source(Grid::new(16, 2), &[".+.^4801", ""], &[(0, 16)]);
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
        let (plan, source) = carried_source(
            Grid::new(16, 4),
            &["./.x030400", ".+0001", "", ""],
            &[(0, 48), (2, 18), (16, 52)],
        );
        assert_eq!(&source.snapshot()[18..20], "0C");
        assert_eq!(&source.snapshot()[48..50], "  ");
        assert_eq!(&source.snapshot()[52..54], "0D");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message == "cannot divide by zero")
        );
        let (plan, source) =
            carried_source(Grid::new(16, 2), &[".+02.x0304", ""], &[(0, 16), (4, 31)]);
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
        let (plan, source) = carried_source(
            Grid::new(16, 4),
            &[".+0001", ".+0203", "./0100", ""],
            &[(0, 48), (16, 2), (32, 2)],
        );
        assert_eq!(&source.snapshot()[..6], ".+0501");
        assert_eq!(&source.snapshot()[48..50], "06");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message == "cannot divide by zero")
        );
        let (plan, source) = carried_source(Grid::new(16, 2), &[".+02./0100", ""], &[(0, 16)]);
        assert_eq!(&source.snapshot()[16..18], "  ");
        assert!(
            plan.diagnostics
                .iter()
                .any(|d| d.message.contains("supplied no typed result"))
        );
        // A failed structural writer leaves the original computation connected.
        let (plan, source) = carried_source(
            Grid::new(16, 3),
            &[".+02.x0304", "./0100", ""],
            &[(0, 32), (16, 4)],
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
    fn live_inactive_ownership_and_a_refused_terminal_portal_are_independent() {
        let (plan, interpreted, source) = carried_tick(
            Grid::new(16, 3),
            &["!>007F.^80", "", ""],
            &[(0, 16), (6, 20)],
        );
        assert!(plan.writes.is_empty());
        assert!(plan.play_commands.is_empty());
        assert_eq!(interpreted.len(), 0);
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("cannot have a Portal"));
        assert_eq!(&source.snapshot()[16..32], "                ");
        let (plan, _) = carried_source(
            Grid::new(16, 4),
            &["!>007FC4", "", ".=0101", ""],
            &[(0, 34), (32, 16)],
        );
        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("cannot have a Portal"));
    }

    #[test]
    fn live_deep_and_top_level_replacement_suppress_descendant_portals() {
        let (plan, interpreted, source) = carried_tick(
            Grid::new(20, 3),
            &[".+02.x03.+0101", ".+0203", ""],
            &[(0, 40), (4, 44), (8, 48), (20, 4)],
        );
        assert_eq!(&source.snapshot()[..14], ".+020503.+0101");
        assert_eq!(&source.snapshot()[40..42], "07");
        assert_eq!(&source.snapshot()[44..50], "      ");
        assert_eq!(interpreted.len(), 2);
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert!(source.language_map().expressions().any(|entry| {
            entry
                .root()
                .is_some_and(|root| root.x() == 8 && root.y() == 0)
        }));
        let (plan, interpreted, source) = carried_tick(
            Grid::new(16, 3),
            &[".+02.x0304", ".+0203", ""],
            &[(0, 32), (4, 36), (16, 0)],
        );
        assert_eq!(&source.snapshot()[..10], "0502.x0304");
        assert_eq!(&source.snapshot()[32..38], "      ");
        assert_eq!(interpreted.len(), 1);
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
        let (plan, source) = replaced_source(
            Grid::new(16, 4),
            &[".+02.x0304", ".+0000", ".+0001", ""],
            &[(0, 48), (4, 34), (16, 0), (32, 52)],
            &[(16, lang::Function::Multiply)],
        );
        assert_eq!(&source.snapshot()[..10], ".x02.x0304");
        assert_eq!(&source.snapshot()[48..50], "18");
        assert_eq!(&source.snapshot()[52..54], "0D");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        let (plan, source) = replaced_source(
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
        let (plan, source) = replaced_source(
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
        let (plan, source) = replaced_source(
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
        let (plan, interpreted, source) = replaced_tick(
            Grid::new(16, 3),
            &[".+0101", ".+0000", ""],
            &[(0, 32), (16, 1)],
            &[(16, lang::Function::Multiply)],
        );
        assert_eq!(&source.snapshot()[..6], "..x101");
        assert_eq!(&source.snapshot()[32..34], "  ");
        // No Turn is taken at the `.x` the replacement spelled at column 1, and
        // none at the `.+` it covered: the original anchor's computation is
        // suppressed and the new one was never scheduled. The producer that
        // states the replacement answers without interpreting anything, so the
        // Interpreter is not called at all this Tick.
        assert_eq!(interpreted.len(), 0);
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
            let (plan, interpreted, source) = carried_tick(
                Grid::new(16, 5),
                &[".+0001", ".+0001", ".=0101", "", "!>007FC4"],
                &outputs,
            );
            assert!(plan.writes.is_empty());
            assert!(plan.play_commands.is_empty());
            assert_eq!(interpreted.len(), 0);
            assert_eq!(&source.snapshot()[48..50], "  ");
            assert!(
                plan.diagnostics
                    .iter()
                    .any(|d| d.message == "same-Tick dependency cycle")
            );
        }
        let (plan, _) = carried_source(
            Grid::new(16, 3),
            &[".+02.x0304", ".+0001", ""],
            &[(0, 18), (4, 18), (16, 6)],
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
            let carried =
                carried_destinations(grid, &[(32, 0), (48, 160), (64, 96), (80, 64), (112, 160)]);
            // A valid order delivers the writer before its target. Execute it
            // once to prove that this fixture has writes and a Play Command
            // which the defensive rejection below must discard.
            let (plan, _) =
                super::plan_carrying(grid, bytes.as_bytes(), &map, Tick::ZERO, &carried);
            assert!(!plan.writes.is_empty());
            assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);

            let mut schedule = super::schedule_carrying(grid, &map, &carried).unwrap();
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
            let (rejected, states) =
                super::execution::execute(grid, bytes.as_bytes(), &map, Tick::ZERO, schedule);
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
                interpreted(&states).len(),
                if target == ".+01??" { 4 } else { 5 }
            );
        }
    }

    #[test]
    fn original_anchor_function_replacement_retains_inputs() {
        let (plan, source) = replaced_source(
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
        let (plan, interpreted, source) = carried_tick(
            Grid::new(16, 4),
            &[".+02.x0304", ".+0203", "", ""],
            &[(0, 48), (4, 52), (16, 4)],
        );
        assert_eq!(&source.snapshot()[..10], ".+02050304");
        assert_eq!(&source.snapshot()[48..50], "07");
        assert_eq!(&source.snapshot()[52..54], "  ");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(interpreted.len(), 2);
        let next = source.language_map();
        assert_eq!(next.expressions().next().unwrap().span().end().get(), 5);
        assert!(next.diagnostics().any(|diagnostic| diagnostic.start() == 6));
        assert!(next.diagnostics().any(|diagnostic| diagnostic.start() == 8));
    }

    #[test]
    fn nested_computation_returns_and_projects_once() {
        let grid = Grid::new(16, 4);
        let (plan, interpreted, source) = carried_tick(
            grid,
            &[".+02.x0304", ".+0101", "", ""],
            &[(0, 48), (4, 18), (16, 52)],
        );
        assert_eq!(&source.snapshot()[18..20], "0C");
        assert_eq!(&source.snapshot()[48..50], "0E");
        assert_eq!(&source.snapshot()[52..54], "0D");
        assert_eq!(&source.snapshot()[..10], ".+02.x0304");
        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            interpreted
                .iter()
                .filter(|inputs| **inputs
                    == super::tick_inputs(Tick::ZERO, grid.position(4, 0).unwrap()))
                .count(),
            1
        );
    }

    #[test]
    fn partial_writers_settle_before_consumption() {
        let (plan, source) = carried_source(
            Grid::new(16, 4),
            &[".+0101", ".+0203", ".+0101", ""],
            &[(0, 56), (16, 2), (32, 3)],
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
            vec![grid.position(0, 2).unwrap()],
        )]
        .into_iter()
        .collect();

        let (plan, _) =
            super::plan_carrying(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

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
            let (plan, _) = carried_source(
                grid,
                &["", "", "    !>007FC4", "", "", ".=0101"],
                &[(80, row * 16 + column)],
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
        let (plan, _) = carried_source(
            Grid::new(16, 6),
            &["", "", "   .=0101", "    !>007FC4", "", ".=0101"],
            &[(80, 2 * 16 + 4)],
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
        let (plan, source) = carried_source(
            Grid::new(16, 4),
            &["    .=0101", "!>00.+0101C4", "    !>007FC5", ""],
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
        let (plan, source) = carried_source(
            Grid::new(16, 4),
            &["", "!>00.+0101C4", "", ".=0101"],
            &[(48, 4)],
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

        let (plan, _) = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

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

        let (plan, _) = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

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
        let (plan, source) = carried_source(Grid::new(8, 3), &[".=0101", "  0102", ""], &[]);
        assert_eq!(&source.snapshot()[8..14], "**0102");
        assert_eq!(planned(&plan), vec![(8, '*'), (9, '*')]);
        assert!(plan.diagnostics.is_empty());
        assert_eq!(source.language_map().bangs().count(), 1);
        let (plan, source) = carried_source(Grid::new(10, 3), &["    .=0101", "  0102", ""], &[]);
        assert_eq!(&source.snapshot()[10..16], "  01**");
        assert_eq!(planned(&plan), vec![(14, '*'), (15, '*')]);
        assert!(plan.diagnostics.is_empty());
    }

    #[test]
    fn a_destination_at_the_row_edge_costs_one_expression_its_turn_not_the_tick() {
        // A complete destination must fit even when the stated Portal is at
        // the final Cell, independently of other computations.
        let grid = Grid::new(16, 4);
        let bytes = snapshot(grid, &[".+0102", "", ".+0304", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let destinations = [(
            grid.cell_index(0).unwrap(),
            vec![grid.position(15, 1).expect("inside the Grid")],
        )]
        .into_iter()
        .collect();

        let (plan, _) =
            super::plan_carrying(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

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
        let (plan, source) = carried_source(
            Grid::new(16, 3),
            &[".+0102 .+0304", "0102", ""],
            &[(0, 20), (7, 21)],
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

        let (plan, _) = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

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

        let (plan, _) = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

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
        let (plan, source) = carried_source(
            Grid::new(16, 4),
            &["  .+", "!>007FC4", "", ".=0101"],
            &[(48, 32)],
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
        //
        // This test retires when a Function reads an anchor. The Tick half is
        // already asserted through results by the test below, which watches
        // what three Tick Functions write rather than what they were handed;
        // no built Function reads its anchor yet, so the anchor half has no
        // result to be visible in and is read off the execution states here
        // instead. When one does, this asserts through that Function's answer
        // and stops reading states at all.
        let grid = Grid::new(20, 3);
        let bytes = snapshot(grid, &[".+0102 .+0304", "          .-0504", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());
        let tick = Tick::new(11);

        let (_, states) = super::plan(grid, bytes.as_bytes(), &map, tick);
        let interpreted = interpreted(&states);

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
            "each root is told its own anchor"
        );
    }

    #[test]
    fn the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at() {
        // The read `tick-functions/01` left unpinned, now that it has a
        // consumer to pin it through: the Tick threaded from the Playback
        // Engine now changes what a Source Snapshot writes, so severing it
        // fails here rather than only in the seam test above, which watches
        // the inputs rather than the answers.
        //
        // One Grid, three roots, two Ticks. Each root's result lands in the
        // Cell pair directly south of its anchor, so the expected writes are
        // literal Cell indices and characters rather than anything the code
        // under test could satisfy by agreeing with itself.
        let grid = Grid::new(24, 2);
        let bytes = snapshot(grid, &["~.0304 ~*0202 ~%0304", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());

        // Tick 1. The Clock is still in its first step of three, so it writes
        // `00`; the Delay's cycle is 2 * 2 = 4 Ticks and 1 is not a multiple of
        // it; the Euclidean's `X.XX` over four steps has no onset at step 1. A
        // Function that answered the Absence Marker plans no Cell write, so
        // only the Clock's pair is planned.
        let (plan, _) = super::plan(grid, bytes.as_bytes(), &map, Tick::new(1));

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(planned(&plan), vec![(24, '0'), (25, '0')]);

        // Tick 4. The Clock has counted one whole step of three; the Delay is
        // on a multiple of its cycle; and step 0 of `X.XX` is an onset. A
        // hardcoded first Tick would write `00` here and a hardcoded Tick of
        // its own would move all three at once, so the pair of assertions is
        // what makes this about the Tick rather than about the formulas.
        let (plan, _) = super::plan(grid, bytes.as_bytes(), &map, Tick::new(4));

        assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
        assert_eq!(
            planned(&plan),
            vec![
                (24, '0'),
                (25, '1'),
                (31, '*'),
                (32, '*'),
                (38, '*'),
                (39, '*')
            ]
        );
    }

    #[test]
    fn a_tick_function_with_no_cycle_diagnoses_and_writes_nothing() {
        // ADR 0012 refuses a cycle with a zero factor rather than inventing
        // one, and the diagnostic has to reach the Source through the ordinary
        // Tick Plan: it names the Function's spelling and the operand role, so
        // the console can say which of the two Cell pairs to edit.
        let grid = Grid::new(16, 2);
        let bytes = snapshot(grid, &["~*0300 ~%0400", ""]);
        let map = LanguageMap::build(grid, bytes.as_bytes());

        let (plan, _) = super::plan(grid, bytes.as_bytes(), &map, Tick::new(3));

        assert!(plan.writes.is_empty(), "{:?}", plan.writes);
        assert_eq!(
            plan.diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>(),
            vec![
                "~* cannot count a cycle with a zero modulus".to_string(),
                "~% cannot count a cycle with a zero step count".to_string(),
            ]
        );
    }

    #[test]
    fn a_pulse_activates_an_aligned_root_only_on_the_ticks_it_bangs() {
        // Delay and Euclidean declare `can_emit_bang`, and that declaration is
        // what `schedule` reads at `tick.rs:319` and `:462` to decide which
        // roots can supply activation. Nothing else asserts the edge is built:
        // the tests above watch the two Cells a pulse writes, which a Function
        // that Banged into no activation edge would still satisfy while the
        // neighbouring terminal fell silent with no diagnostic anywhere.
        //
        // So the assertion is the play, not the write, and it is made at two
        // Ticks per Function. A hardcoded edge would fire the terminal at both
        // and a missing one at neither, so the pair is what makes this about
        // the pulse rather than about scheduling in general.
        for (spelling, banging, silent) in [("~*0202", 4, 1), ("~%0304", 4, 1)] {
            let grid = Grid::new(16, 6);
            let bytes = snapshot(grid, &["", "!>007FC4", "", "", "", spelling]);
            let map = LanguageMap::build(grid, bytes.as_bytes());
            let destinations = [(
                grid.index(grid.position(0, 5).unwrap()),
                vec![grid.position(0, 2).unwrap()],
            )]
            .into_iter()
            .collect();

            let (plan, _) = super::plan_carrying(
                grid,
                bytes.as_bytes(),
                &map,
                Tick::new(banging),
                &destinations,
            );

            assert_eq!(
                plan.play_commands,
                vec![raw(0, 0x7F, 60)],
                "{spelling} at Tick {banging}, diagnostics: {:?}",
                plan.diagnostics
            );

            let (plan, _) = super::plan_carrying(
                grid,
                bytes.as_bytes(),
                &map,
                Tick::new(silent),
                &destinations,
            );

            assert!(
                plan.play_commands.is_empty(),
                "{spelling} at Tick {silent} played {:?}",
                plan.play_commands
            );
        }
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
                vec![grid.position(6, 2).unwrap()],
            ),
            (
                grid.index(grid.position(0, 4).unwrap()),
                vec![grid.position(0, 3).unwrap()],
            ),
        ]
        .into_iter()
        .collect();

        let (plan, _) =
            super::plan_carrying(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

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
                vec![grid.position(0, 2).unwrap()],
            ),
            (
                grid.index(grid.position(0, 5).unwrap()),
                vec![grid.position(0, 4).unwrap()],
            ),
        ]
        .into_iter()
        .collect();

        let (plan, _) =
            super::plan_carrying(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

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
                vec![grid.position(0, 4).unwrap()],
            ),
            (
                grid.index(grid.position(0, 1).unwrap()),
                vec![grid.position(0, 6).unwrap()],
            ),
        ]
        .into_iter()
        .collect();

        let (plan, _) =
            super::plan_carrying(grid, bytes.as_bytes(), &map, Tick::ZERO, &destinations);

        assert_eq!(plan.play_commands, vec![raw(0, 0x7F, 60)]);
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "cannot divide by zero")
        );
    }

    #[test]
    fn competing_writers_preserve_an_independent_rejected_destination_diagnostic() {
        let (plan, source) = carried_source(
            Grid::new(16, 3),
            &[".+0102", ".+0304", ".+0506"],
            &[(0, 26), (16, 26), (32, 31)],
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
            (conflict_grid.cell_index(0).unwrap(), vec![shared]),
            (conflict_grid.cell_index(16).unwrap(), vec![shared]),
        ]
        .into_iter()
        .collect();
        let (conflict, _) = super::plan_carrying(
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
                vec![cycle_grid.position(2, 1).unwrap()],
            ),
            (
                cycle_grid.cell_index(16).unwrap(),
                vec![cycle_grid.position(2, 0).unwrap()],
            ),
        ]
        .into_iter()
        .collect();
        let (cycle, _) = super::plan_carrying(
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
                .admit(&Encoding::literal(content).expect("printable test content"))
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

        let (plan, _) = super::plan(grid, bytes.as_bytes(), &map, Tick::ZERO);

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
    use super::{Effect, Encoding, Portal, resolve};
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
                let cells = Encoding::literal(&encoding).expect("printable test content");
                if let Ok(write) = Portal::at(grid, destination).admit(&cells) {
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
