//! The changing state and Effects of one Tick, behind one execution seam.
//!
//! Lookup retains the original Parser structure. Execution owns whether its
//! computations can take a Turn, how their operands are consumed, and how an
//! admitted spatial result changes later Turns. Which Turn is taken next is
//! [`super::ordering`]'s, and what working Source holds is
//! [`working::WorkingSource`]'s. Nothing here survives the Tick.

use lang::{
    Atom, Function, FunctionInputs, Interpretation, Interpreter, Output, PortalAddress,
    PortalSource, SourceBundle, SourceEffect, Tick,
};

use super::Schedule;
use super::dependencies::{self, Consequence, Delivery, Locked, Phase, Refusal};
use super::ordering::{Progress, Turn};
use super::{
    Computation, ComputationId, Coverage, Effect, Encoding, Grid, LanguageMap, Occupancy, Occupant,
    Operand, Portal, PortalError, PortalUnit, Position, RenderError, Rendered, SCALAR_WIDTH,
    SpanWrite, TickPlan, coverage, diagnose, resolve, tick_inputs,
};
use crate::source::buffer::Cells;
use turns::{Activation, Answer, Turns};
use working::{WorkingSource, WriteKind};

mod operands;
mod turns;
mod working;

pub(in crate::source) use turns::ComputationState;

/// The two empty Cells a Function that copies Cells writes and returns for an
/// empty pair.
const EMPTY_PAIR: &str = "  ";

/// Where a Function's Input Portal stands for one Turn.
#[derive(Clone, Copy)]
enum InputSite {
    /// The Function declares no Input Portal.
    Undeclared,
    /// The Portal its operands select stands outside the Grid.
    Outside,
    /// The Portal its operands select.
    At(Portal),
}

///
/// Executes an established order against the original Source Snapshot.
///
/// The caller supplies no mutable state and receives two things: the
/// publishable Tick Plan, and what each computation's Turn actually did. They
/// are different facts, which is why the second is not folded into the first —
/// a Tick Plan says what to apply, and a Turn that ran can apply nothing.
///
/// The schedule is borrowed and left as it was: every Tick planned against the
/// same scheduling inputs executes the one schedule they share.
/// [`super::take_turns`] decides which Turn is taken next, and each Turn
/// answers whether it was taken or must wait for writers.
///
pub(super) fn execute(
    grid: Grid,
    cells: Cells<'_>,
    map: &LanguageMap,
    tick: Tick,
    schedule: &Schedule,
) -> (TickPlan, Vec<ComputationState>) {
    let mut execution = Execution::new(grid, cells, map, tick, schedule);
    let diagnostics = super::take_turns(schedule, |index, progress| {
        execution.take_turn(index, progress)
    });
    execution
        .effects
        .extend(diagnostics.into_iter().map(Effect::Diagnose));
    (resolve(execution.effects), execution.turns.into_states())
}

struct Execution<'a> {
    grid: Grid,
    original: Cells<'a>,
    working: WorkingSource<'a>,
    tick: Tick,
    schedule: &'a Schedule,
    turns: Turns,
    effects: Vec<Effect>,
    /// The roots the Turn being taken has activated through a dynamic
    /// write, which only the Turn can know of.
    activated: Vec<ComputationId>,
}

impl<'a> Execution<'a> {
    ///
    /// The state one Tick starts from, before any computation takes a Turn:
    /// every computation untouched and the schedule's own diagnostics
    /// recorded.
    ///
    fn new(
        grid: Grid,
        cells: Cells<'a>,
        map: &'a LanguageMap,
        tick: Tick,
        schedule: &'a Schedule,
    ) -> Self {
        let Schedule {
            lookup,
            diagnostics,
            ..
        } = schedule;
        Self {
            grid,
            original: cells,
            working: WorkingSource::new(cells, map),
            tick,
            schedule,
            turns: Turns::new(lookup),
            effects: diagnostics.iter().cloned().map(Effect::Diagnose).collect(),
            activated: Vec::new(),
        }
    }

    ///
    /// The refusals a Turn faces before it can have an answer at all, and the
    /// signature the Turn proceeds with when it faces none.
    ///
    /// A Turn nothing here settles is one whose answer decides the rest: which
    /// is why this stops at the signature, one step before the operands are
    /// resolved. Every arm settles the Turn, and an ordering defect is
    /// discovered by delivering an answer, never by a computation's own
    /// prologue.
    ///
    fn opens_turn(&mut self, index: ComputationId) -> Option<lang::Tokens> {
        let node = &self.schedule.lookup.nodes()[index];
        if !self.turns.may_open(index, node.owner()) {
            return None;
        }
        self.turns.open(index);
        let function = self.turns.function(index);
        // A nested Function that answers no value has no Return for its
        // parent. The Parser reports that against the Expression from Source
        // alone, so the Turn is blocked as an unparsed operand's is, without
        // repeating the report, and its parent is blocked in turn.
        if self.syntax_blocks(node, function)
            || (!node.is_root() && !function.answers_value())
            || self.blocked_by_child(node)
        {
            self.turns.block_syntax(index);
            return None;
        }
        let signature = lang::Tokens::from(&function);
        if signature.len() != node.operands.len() {
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                lang::ArgumentError::Arity {
                    expected: signature.len(),
                    found: node.operands.len(),
                }
                .to_string(),
            )));
            return None;
        }
        Some(signature)
    }

    ///
    /// Takes `index`'s Turn, or answers the writers it waits for.
    ///
    /// A Turn waits only where its Input Portal is found from its operands and
    /// a writer of the Cells it selects has not yet taken its Turn. It then
    /// leaves no effect and takes its Turn again once they have. Every other
    /// Turn is taken here, and answers the roots its dynamic writes
    /// activated.
    ///
    fn take_turn(&mut self, index: ComputationId, progress: &Progress<'_>) -> Turn {
        match self.turn(index, progress) {
            Some(writers) => Turn::Waits(writers),
            None => {
                self.settle(index);
                Turn::Taken {
                    activated: std::mem::take(&mut self.activated),
                }
            }
        }
    }

    /// Records that `index` has taken its Turn this Tick, settled with or
    /// without effect.
    fn settle(&mut self, index: ComputationId) {
        self.turns.settle(index);
    }

    /// Every failure settles this Turn and records its diagnostic, a violated
    /// execution order included: it refuses the one write or lock it reaches,
    /// and every other Turn of the Tick still takes place.
    fn turn(
        &mut self,
        index: ComputationId,
        progress: &Progress<'_>,
    ) -> Option<Vec<ComputationId>> {
        let signature = self.opens_turn(index)?;
        let lookup = &self.schedule.lookup;
        let node = &lookup.nodes()[index];
        let function = self.turns.function(index);
        let tick = tick_inputs(self.tick, node.anchor);
        let result = self.decode(node, signature).and_then(|operands| {
            let portal = self.turn_portal(index, function, &operands)?;
            let destination = self.turn_destination(index, function, &operands)?;
            Ok((operands, portal, destination))
        });
        let (operands, portal, destination) = match result {
            Ok(resolved) => resolved,
            Err(message) => {
                self.effects.push(Effect::Diagnose(diagnose(node, message)));
                return None;
            }
        };
        // The schedule orders a static Input Portal after its writers, and a
        // Function Replacement keeps the declared Input Portal, so only a
        // dynamic one finds writers still to take their Turn here.
        if let InputSite::At(portal) = portal
            && let Ok(span) = portal.span(SCALAR_WIDTH)
        {
            let writers = progress.unresolved(dependencies::input_writers(
                lookup,
                progress.active(),
                index,
                span.range(),
            ));
            if !writers.is_empty() {
                return Some(writers);
            }
        }
        // Recorded beside the call rather than before it: a Turn whose
        // operands would not resolve is one the Interpreter never ran for, and
        // the record says which of the two happened.
        #[cfg(test)]
        self.turns.interpret(index, tick);
        let delivery = if destination.is_some() {
            Delivery::Selected
        } else if progress.joined(index) {
            Delivery::Joined
        } else {
            Delivery::Reserved
        };
        let inputs = FunctionInputs::with_portal_source(tick, self.portal_source(function, portal));
        match Interpreter::execute_function(function, operands, inputs) {
            Err(error) => self
                .effects
                .push(Effect::Diagnose(diagnose(node, error.to_string()))),
            Ok(Interpretation::Play(command)) => self.effects.push(Effect::Play(command)),
            Ok(Interpretation::Cell(atom)) => {
                self.deliver_value(index, Answer::Unit(atom), destination, delivery);
            }
            Ok(Interpretation::Clear) => {
                self.deliver_value(index, Answer::Clear, destination, delivery);
            }
            Ok(Interpretation::Source(effect)) => {
                self.deliver_source_effect(index, effect, delivery);
            }
            Ok(Interpretation::Lock) => self.lock_portal(index, delivery),
        }
        None
    }

    ///
    /// `function`'s Input Portal for `index`'s Turn once its operands are
    /// resolved. A resolution that refuses diagnoses the Turn.
    ///
    fn turn_portal(
        &self,
        index: ComputationId,
        function: Function,
        operands: &[Atom],
    ) -> Result<InputSite, String> {
        let Some(declared) = function.input_portal() else {
            return Ok(InputSite::Undeclared);
        };
        Ok(
            match self.select(index, |columns| declared.resolve(operands, columns))? {
                Ok(portal) => InputSite::At(portal),
                Err(_) => InputSite::Outside,
            },
        )
    }

    ///
    /// The destination `function`'s dynamic Output Portal selects for
    /// `index`'s Turn once its operands are resolved: `None` where its Output
    /// Portal is static, and otherwise the Position the pair starts at or
    /// why no pair in the Grid answers.
    ///
    /// A pair that starts inside the Grid and is cut short by the row edge is
    /// refused when the write is admitted. A resolution that refuses
    /// diagnoses the Turn.
    ///
    fn turn_destination(
        &self,
        index: ComputationId,
        function: Function,
        operands: &[Atom],
    ) -> Result<Option<Result<Position, PortalError>>, String> {
        let Output::Dynamic(selection) = function.output() else {
            return Ok(None);
        };
        Ok(Some(
            self.select(index, |columns| selection.resolve(operands, columns))?
                .map(Portal::destination),
        ))
    }

    ///
    /// The Portal a declaration `resolve`s for `index`'s Turn, or why the
    /// Grid holds none.
    ///
    /// `lang` resolves the declaration; the columns it is given are those
    /// `index`'s operands occupy east of its anchor, nested operands
    /// included. An offset is taken from the anchor, and a Position is the
    /// Grid's own. A resolution that refuses is the outer error, which
    /// diagnoses the Turn.
    ///
    fn select(
        &self,
        index: ComputationId,
        resolve: impl FnOnce(usize) -> Result<PortalAddress, lang::Error>,
    ) -> Result<Result<Portal, PortalError>, String> {
        let anchor = self.schedule.lookup.nodes()[index].anchor;
        let address = resolve(self.operands_end(index) - self.grid.index(anchor).get())
            .map_err(|error| error.to_string())?;
        Ok(match address {
            PortalAddress::Offset(coords) => {
                Portal::displaced(self.grid, anchor, coords.columns, coords.rows)
            }
            PortalAddress::Position { column, row } => self
                .grid
                .position(usize::from(column), usize::from(row))
                .map(|position| Portal::at(self.grid, position))
                .ok_or(PortalError::OutsideGrid),
        })
    }

    fn syntax_blocks(&self, node: &Computation, function: Function) -> bool {
        // Unchanged initial syntax errors belong to the Source revision.
        // Earlier writes or a Function replacement can repair those inputs.
        !node.syntax_valid
            && function == node.function
            && node.operands.iter().all(|operand| {
                self.working.text(operand.cells.clone()).as_bytes()
                    == self.original.slice(operand.cells.clone()).bytes()
            })
    }

    /// Borrow working Source at the Input Portal `portal` resolved for this
    /// Turn. A missing or truncated site stays absent so binding diagnoses it
    /// after all cell operands have been validated; a Function that copies a
    /// Language Unit reads the pair only when it is one complete aligned unit.
    fn portal_source(&self, function: Function, portal: InputSite) -> PortalSource<'_> {
        let portal = match portal {
            InputSite::Undeclared => return PortalSource::none(),
            InputSite::Outside => return PortalSource::from_cells(None),
            InputSite::At(portal) => portal,
        };
        PortalSource::from_cells(match function.portal_input() {
            Some(input) => self.working.portal_cells(portal, input.token().len()),
            None => self.working.portal_unit(portal),
        })
    }

    /// Delivers `index`'s answer through its Output Portal: through
    /// `destination` where its Turn selected one, and otherwise through the
    /// static sites the schedule reserved.
    fn deliver_value(
        &mut self,
        index: ComputationId,
        answer: Answer,
        destination: Option<Result<Position, PortalError>>,
        delivery: Delivery,
    ) {
        self.project_value(index, answer, destination, delivery);
        // A successful nested answer survives every refusal to project it.
        self.turns.answer(index, answer);
    }

    /// Plans the Cell writes, activation, or clear one answer makes, whether
    /// its computation is a root or nested.
    fn project_value(
        &mut self,
        index: ComputationId,
        answer: Answer,
        destination: Option<Result<Position, PortalError>>,
        delivery: Delivery,
    ) {
        let node = &self.schedule.lookup.nodes()[index];
        // Every arm below plans or diagnoses a write at an Output Portal, so an
        // answer with none to write, which only its consumer reads, is not
        // rendered at all. Every arm below relies on this return and does not
        // ask again.
        let selected;
        let sites: &[Result<Position, PortalError>] = match destination {
            Some(destination) => {
                selected = [destination];
                &selected
            }
            None if node.portal_access.writes_cells() => node.portal_access.write_sites(),
            None => return,
        };
        // Whether this answer can be Cells at all is a question about the
        // value, settled before any destination is asked: a rendering a Cell
        // cannot hold refuses whole. A clear writes two spaces, and every
        // other answer renders as the Cell pair the schedule reserved. A
        // pulse clears the pair it reserves for its Bang, root or nested, so
        // clearing it orders nothing new.
        let encoding = match answer {
            Answer::Clear => Encoding::literal(EMPTY_PAIR).expect("a space is a printable Cell"),
            Answer::Unit(atom) => match Encoding::render(atom) {
                Ok(Rendered::Nothing) => return,
                Ok(Rendered::Cells(encoding)) => encoding,
                Err(reason) => {
                    self.effects
                        .push(Effect::Diagnose(diagnose(node, render_message(reason))));
                    return;
                }
            },
        };
        for output in sites {
            self.deliver_output(index, answer.unit(), &encoding, *output, delivery);
        }
    }

    ///
    /// Delivers one encoding at one Output Portal site.
    ///
    /// What the write may change is the reach rules' to answer:
    /// [`dependencies::bang_root`] which roots its Bang activates,
    /// [`dependencies::refusal`] whether it is refused whole, and
    /// [`dependencies::contact`] what each Contact it covers means. Execution
    /// applies the answers and words the diagnostics. A Bang's activation is
    /// delivered before the write is judged, because it lasts only for the Tick
    /// that produces it: a root that has taken its Turn without it is diagnosed
    /// as missed, and nothing else is stopped.
    ///
    fn deliver_output(
        &mut self,
        index: ComputationId,
        written: Option<Atom>,
        encoding: &Encoding,
        output: Result<Position, PortalError>,
        delivery: Delivery,
    ) {
        let node = &self.schedule.lookup.nodes()[index];
        let destination = match output {
            Ok(destination) => destination,
            Err(reason) => {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    portal_message(reason, encoding),
                )));
                return;
            }
        };
        let write = match Portal::at(self.grid, destination).admit(encoding) {
            Ok(write) => write,
            Err(reason) => {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    portal_message(reason, encoding),
                )));
                return;
            }
        };
        let lookup = &self.schedule.lookup;
        let reached = dependencies::reached(lookup, &write);
        for root in reached.bang_roots() {
            let phase = self.phase(delivery, written);
            if let Consequence::Activate(root) = dependencies::bang_root(lookup, index, root, phase)
                && self.activate(root) == Activation::Delivered
                && delivery == Delivery::Selected
            {
                self.activated.push(root);
            }
        }
        match dependencies::refusal(&reached, written, delivery, &self.turns) {
            // The fact that differed, not the list of facts that could have.
            // Each is argued on `ReplacementChange`, and the wording is
            // CONTEXT.md's, so the diagnostic continues the glossary's own
            // sentence about what a replacement may not change.
            Some(Refusal::Replacement(change)) => {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    format!("Function replacement changes {change}"),
                )));
                return;
            }
            Some(Refusal::OrderingDefect) => {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    "spatial output reached an executed computation",
                )));
                return;
            }
            None => {}
        }
        // Each Contact is asked in turn, so one a covering Contact before it
        // suppressed is left as it is.
        let mut reaches_own_expression = false;
        for contacted in reached.functions() {
            let target = contacted.index();
            let contacted =
                dependencies::contact(lookup, index, &contacted, self.phase(delivery, written));
            reaches_own_expression |= contacted.own_expression;
            match contacted.consequence {
                Consequence::Replace(replacement) => self.turns.replace(target, replacement),
                Consequence::Suppress(subtree) => self.turns.suppress(subtree),
                _ => {}
            }
        }
        if reaches_own_expression {
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                "dynamic output reached its own Expression",
            )));
        }
        self.write(WriteKind::Output, write);
    }

    ///
    /// The one validated Source Bundle for a Source-writing Function.
    ///
    /// The Bang Function's `Fire` takes its Turn in [`Execution::fire`]. Every
    /// other bundle writes displaced, and its two facts decide the rest. One
    /// that also clears its own Span moves: an empty destination plans one validated Portal bundle, spaces
    /// over the current Span followed by its own spelling at the shifted
    /// destination, and a blocked or out-of-Grid move instead replaces its
    /// current Span with `**`. One that writes displaced and leaves its own
    /// Span standing emits: the complete initial destination must be empty and
    /// inside the Grid, or the producer diagnoses and emits nothing.
    ///
    /// It is not [`Execution::deliver_value`] with a different destination. That
    /// path delivers one encoding through every Portal a computation resolved;
    /// a move writes different Cells at each of its two, and what it writes at
    /// the second decides what it writes at the first: spaces at its origin
    /// when the destination admits its spelling, `**` when it does not.
    ///
    /// The precondition is one rule and the refusal is two, which is why the
    /// groups share this path rather than each having one. What a refusal costs
    /// follows from whether the bundle clears the producer's own Span: a
    /// producer that was leaving those Cells reports in them, and a producer
    /// that is staying has nothing of its own to report in.
    ///
    /// Admission is atomic, as for ordinary value output, but empty-only
    /// placement cannot overwrite a standing Function. Snapshot ownership of
    /// vacated Cells therefore imposes no executed-computation guard here.
    /// A moving bundle also clears its own Span; the schedule excludes that
    /// producer from its own overwrite dependency.
    ///
    /// The displacement is the Interpreter's answer and the destination
    /// `computations` reserved is the same declaration read before the Turn.
    /// Resolving it again here is what makes this the answer being delivered
    /// rather than the schedule replaying itself, which is the relationship
    /// every reservation has with the write it orders.
    ///
    fn deliver_source_effect(
        &mut self,
        index: ComputationId,
        effect: SourceEffect,
        delivery: Delivery,
    ) {
        if effect.bundle == SourceBundle::Fire {
            self.fire(index, delivery);
            return;
        }
        let node = &self.schedule.lookup.nodes()[index];
        let anchor = node.anchor;
        let spelling = Encoding::literal(
            effect
                .spelling
                .expect("a bundle that writes displaced declares the Function it writes"),
        )
        .expect("a Function spelling is printable ASCII Cells");
        // What stands in the Source, which is not always what gets written. A
        // Self-Banging Function writes its own spelling and the two agree; a
        // Directional Bang Function writes the Function it emits, and only this
        // one names the Cells the author would go and fix. `Function` displays
        // as its spelling, which is how every other diagnostic in this file
        // names one.
        let producer = self.turns.function(index);
        // The Cells this Function stands in. Every Function spelling is two
        // ASCII Cells, which `define_functions!` asserts at compile time, and
        // every spelling a Source-writing Function writes is another
        // Function's, so the Span it occupies and the Span it writes are the
        // same width and one length serves both.
        //
        // A bundle that writes displaced and clears its own Span moves: it
        // leaves the Cells it stood in, so they are not newly entered and a
        // refusal has them to leave `**` in. One that only writes displaced
        // stays standing.
        let moves = effect.bundle.clears_own_span() && effect.bundle.writes_displaced();
        let start = self.grid.index(anchor).get();
        let own = start..start + spelling.len();

        // Which Cells the precondition is asked about, and the one place a
        // move and an emission read differently. A move tests only the Cells
        // newly entered by a one-Cell step — the whole destination for a
        // vertical move and one Cell of it for a horizontal one — and an
        // emission tests the complete initial destination. The two
        // coincide for every offset declared, because no emission overlaps its
        // producer, so the distinction is stated rather than relied on.
        //
        // The asymmetry is not carved into the write either way: a move's
        // clear covers the complete old Span, and later-write-wins settles the
        // Cell the two share.
        let admitted = Portal::displaced(self.grid, anchor, effect.columns, effect.rows)
            .and_then(|portal| portal.admit(&spelling));
        let entered: Vec<usize> = match &admitted {
            Ok(write) => write
                .cells()
                .map(|(cell, _)| cell.get())
                .filter(|cell| !moves || !own.contains(cell))
                .collect(),
            Err(_) => vec![],
        };
        let empty = self.working.vacant(&entered);

        match admitted {
            Ok(write) if empty => {
                // Placement observes current vacancy, not Snapshot ownership.
                // No standing Function is overwritten or suppressed. A claimed
                // operand still waits for this supplier and decodes these Cells
                // when its consumer executes.
                if moves {
                    let clear = self.own_span_clear(anchor);
                    self.write(WriteKind::Vacate, clear);
                }
                self.write(WriteKind::Place, write);
            }
            // Refused: out of the Grid, past the row edge, or blocked by Cells
            // that are not empty. No partial write is admitted, so the whole
            // destination is gone in every case, and what the producer does
            // instead is the bundle's to say.
            _ if moves => {
                // Source content rather than an answer: the Bang Function a
                // refused move leaves, which activates nothing in this Turn
                // and takes its own Turn from the next Snapshot. It is stated
                // rather than rendered, and reaches no Portal of a value.
                let bang =
                    Encoding::literal("**").expect("the Bang spelling is printable ASCII Cells");
                let left = Portal::at(self.grid, anchor)
                    .admit(&bang)
                    .expect("a Function standing in the Source fits its own Span");
                self.write(WriteKind::Place, left);
                let lookup = &self.schedule.lookup;
                // `Occupancy` is stated outside the schedule, so it names a
                // root by its bare index.
                let root_at = |anchor| lookup.root_at(anchor).map(ComputationId::index);
                if let Occupancy::Partial = self.working.contact(&entered, root_at) {
                    self.effects.push(Effect::Diagnose(diagnose(
                        node,
                        format!("{producer} contacts part of a Language Unit"),
                    )));
                }
            }
            // An emitting Function stays where it is, so it has no Cells of its
            // own to report in and diagnoses instead. It classifies no contact
            // either: what it would have emitted into is not a Cell it was
            // moving to, so there is nothing there it could be aligned with.
            //
            // Both spellings are named because this is the one group where they
            // differ: the producer is the Cell pair to go and fix, and the
            // emission is what it was trying to put outside its own Span. The
            // precondition is one conjunction — empty and inside the Grid — so
            // the refusals share one wording.
            _ => self.effects.push(Effect::Diagnose(diagnose(
                node,
                format!(
                    "{producer} has no empty destination inside the Grid for {}",
                    effect
                        .spelling
                        .expect("a bundle that writes displaced declares the Function it writes"),
                ),
            ))),
        }
    }

    ///
    /// The Bang Function's Turn: it delivers Bang to the roots aligned with
    /// its Span and clears its two Cells.
    ///
    /// The schedule ordered every producer that writes over those Cells, and
    /// every root the Bang activates, by the same edges any producer's
    /// reservation gives, so an overwrite has already suppressed this Turn
    /// and a reader of the Cells waits for the clear.
    ///
    fn fire(&mut self, index: ComputationId, delivery: Delivery) {
        let lookup = &self.schedule.lookup;
        let clear = self.own_span_clear(lookup.nodes()[index].anchor);
        for root in dependencies::reached(lookup, &clear).bang_roots() {
            let phase = self.phase(delivery, Some(Atom::Bang));
            if let Consequence::Activate(root) = dependencies::bang_root(lookup, index, root, phase)
            {
                let _ = self.activate(root);
            }
        }
        self.write(WriteKind::Vacate, clear);
    }

    /// The Turn phase a Turn whose place `delivery` decided, writing
    /// `written`, asks the reach rules in.
    fn phase(&self, delivery: Delivery, written: Option<Atom>) -> Phase<'_> {
        Phase::Turn {
            delivery,
            written,
            turns: &self.turns,
        }
    }

    ///
    /// The two spaces that clear the Span of the Function anchored at
    /// `anchor`.
    ///
    /// Source content rather than an answer, so it is stated rather than
    /// rendered. Every Function spelling is two Cells, so two spaces cover it.
    ///
    fn own_span_clear(&self, anchor: Position) -> SpanWrite {
        let cleared = Encoding::literal("  ").expect("a space is a printable Cell");
        Portal::at(self.grid, anchor)
            .admit(&cleared)
            .expect("a Function standing in the Source fits its own Span")
    }

    ///
    /// Delivers a Bang's activation to `root`, and diagnoses at `root` a Bang
    /// that misses it. Nothing else is stopped.
    ///
    fn activate(&mut self, root: ComputationId) -> Activation {
        let activation = self.turns.activate(root);
        if activation == Activation::Missed {
            self.effects.push(Effect::Diagnose(diagnose(
                &self.schedule.lookup.nodes()[root],
                "Bang reached a root that has taken its Turn",
            )));
        }
        activation
    }

    ///
    /// Applies a lock to the Expression root at the Function's Output Portal,
    /// as [`dependencies::lock_verdict`] answers it, and words its
    /// diagnostics. An ordering defect is diagnosed at the Halt, as a late
    /// spatial write is, and a missed lock at the root it missed. Halt itself
    /// is not suppressed here — `opens_turn` already refused a suppressed
    /// Halt, so reaching this arm means this Halt locks.
    ///
    fn lock_portal(&mut self, index: ComputationId, delivery: Delivery) {
        let lookup = &self.schedule.lookup;
        let node = &lookup.nodes()[index];
        match dependencies::lock_verdict(&self.schedule.locks, index, self.phase(delivery, None)) {
            Locked::Lock(target) => {
                let root = lookup.nodes()[target.start()].anchor;
                self.turns.suppress(target);
                self.effects.push(Effect::Lock(root));
            }
            Locked::Missed(root) => self.effects.push(Effect::Diagnose(diagnose(
                &lookup.nodes()[root],
                "lock reached a root that has taken its Turn",
            ))),
            Locked::OrderingDefect => self.effects.push(Effect::Diagnose(diagnose(
                node,
                "spatial output reached an executed computation",
            ))),
            Locked::NotARoot => self.effects.push(Effect::Diagnose(diagnose(
                node,
                format!("{} target is not an Expression root", node.function),
            ))),
            Locked::Unchanged | Locked::OrderAfter(_) => {}
        }
    }

    /// Applying a write to working Source and recording its Effect are one
    /// operation, so the Tick Plan writes exactly what later Turns read.
    fn write(&mut self, kind: WriteKind, write: SpanWrite) {
        self.working.apply(kind, &write);
        self.effects.push(Effect::Write(write));
    }
}

/// Why a destination refused the value sent to it.
///
/// `OutsideGrid` is live for a Copy whose reserved output Portal left the
/// Grid. Advance and Emit settle an out-of-Grid displacement in
/// [`Execution::deliver_source_effect`], so they never reach this function.
/// The other refusals come from `Portal::at(..).admit(..)`, which resolves
/// inside the Grid by construction and so can only answer `BelowSource` or
/// `CrossesRowEdge`.
/// The match is exhaustive over `PortalError` because ADR 0028 rules out a
/// panic inside Tick planning.
fn portal_message(reason: PortalError, encoding: &Encoding) -> String {
    let encoding = encoding.to_string();
    match reason {
        PortalError::BelowSource => format!("result {encoding:?} falls below the Source"),
        PortalError::OutsideGrid => format!("result {encoding:?} falls outside the Grid"),
        PortalError::CrossesRowEdge => format!("result {encoding:?} crosses the row edge"),
    }
}

/// A value that could not become Cells names what it rendered to, which is the
/// same thing the destination refusals above name. The two are separate
/// messages because they are separate questions: this one is true of the value
/// wherever it was sent, and no destination was asked before it was refused.
fn render_message(reason: RenderError) -> String {
    match reason {
        RenderError::Unrepresentable(rendering) => {
            format!("result {rendering:?} contains Cells outside printable ASCII")
        }
    }
}

#[cfg(test)]
pub(super) mod stated;
