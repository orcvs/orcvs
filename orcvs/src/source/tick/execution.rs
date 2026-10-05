//! The changing state and Effects of one Tick, behind one execution seam.
//!
//! Lookup retains the original Parser structure. Execution owns whether its
//! computations can take a Turn, how their operands are consumed, and how an
//! admitted spatial result changes later Turns. Nothing here survives the Tick.

use std::ops::ControlFlow::{self, Break, Continue};

use lang::{
    Atom, Function, FunctionInputs, Interpretation, Interpreter, PortalCoords, PortalInput,
    PortalSource, SourceBundle, SourceEffect, Tick,
};

use super::{
    Computation, Diagnostic, Effect, Encoding, Grid, LanguageMap, Lookup, Occupancy, Portal,
    PortalError, PortalUnit, Position, RenderError, Rendered, Schedule, SpanWrite, TickPlan,
    diagnose, resolve, tick_inputs,
};
use crate::source::buffer::{Cells, WorkingCells};

///
/// Executes an established order against the original Source Snapshot.
///
/// The caller supplies no mutable state and receives two things: the
/// publishable Tick Plan, and what each computation's Turn actually did. They
/// are different facts, which is why the second is not folded into the first —
/// a Tick Plan says what to apply, and a rejected Tick applies nothing however
/// much of it ran.
///
/// The schedule is borrowed and left as it was: every Tick planned against the
/// same scheduling inputs executes the one schedule they share.
///
pub(super) fn execute(
    grid: Grid,
    cells: Cells<'_>,
    map: &LanguageMap,
    tick: Tick,
    schedule: &Schedule,
) -> (TickPlan, Vec<ComputationState>) {
    let Schedule {
        lookup,
        order,
        diagnostics,
    } = schedule;
    let mut execution = Execution::new(grid, cells, map, tick, lookup, diagnostics.clone());
    #[cfg_attr(
        not(test),
        expect(unused_variables, reason = "only a test build records the Turn")
    )]
    for (turn, &index) in order.iter().enumerate() {
        // Recorded here rather than where the order was built: the ordinal is
        // the Turn a computation took, and a Tick that stops partway through
        // leaves every computation after it without one.
        #[cfg(test)]
        {
            execution.states[index].turn = Some(turn);
        }
        if let Break(diagnostic) = execution.take_turn(index) {
            return execution.reject(diagnostic);
        }
    }
    (resolve(execution.effects), execution.states)
}

/// These facts are independent: an attempted Turn can be syntax-blocked, and
/// a successful answer, returned to a parent, can coexist with a rejected
/// spatial delivery.
/// Keeping them together does not turn them into an exclusive lifecycle enum.
pub(in crate::source) struct ComputationState {
    function: Function,
    result: Option<Atom>,
    syntax_blocked: bool,
    activated: bool,
    suppressed: bool,
    attempted: bool,
    /// Which Turn this computation took, counted from zero, or `None` where
    /// the Tick ended before reaching it.
    #[cfg(test)]
    turn: Option<usize>,
    /// The explicit inputs the Interpreter was handed for this computation, or
    /// `None` where it was never called for it. A Turn that was suppressed,
    /// refused by its own prologue, or stopped by operands it could not
    /// resolve reaches no Interpreter and keeps `None`.
    ///
    /// One slot records the Tick and anchor of this computation. Portal Cells
    /// are borrowed separately from working Source when its Turn binds.
    #[cfg(test)]
    interpreted: Option<lang::TickInputs>,
    /// How many times the Interpreter ran for this computation.
    ///
    /// A Turn is taken once, so a Tick that behaves leaves this `0` or `1` and
    /// `interpreted` alone would say everything. It is counted anyway because
    /// the one thing `interpreted` cannot say is "twice": a second call
    /// overwrites the slot with equal inputs, and a computation that ran twice
    /// writes the same value twice, so the Source cannot tell either. Without
    /// this field a double projection has no witness anywhere.
    #[cfg(test)]
    interpretations: usize,
}

/// What a Turn did, read only by `source::tick`'s tests: a Tick Plan carries
/// what to apply, and no shipped caller asks how it was reached.
#[cfg(test)]
impl ComputationState {
    ///
    /// Which Turn this computation took, or `None` where the Tick ended before
    /// reaching it.
    ///
    /// The order a schedule establishes is consumed by the loop that walks it
    /// and survives nowhere else, so a claim about which computation took the
    /// earlier Turn is asserted here rather than inferred from the Cells the
    /// later write won.
    ///
    pub(in crate::source) fn turn(&self) -> Option<usize> {
        self.turn
    }

    ///
    /// The inputs the Interpreter received for this computation, or `None`
    /// where it never ran for it.
    ///
    pub(in crate::source) fn interpreted(&self) -> Option<lang::TickInputs> {
        self.interpreted
    }

    ///
    /// How many times the Interpreter ran for this computation.
    ///
    /// Read beside [`ComputationState::interpreted`] rather than in place of
    /// it: the pair is the record of one call per unit, which is what a caller
    /// counting Interpreter calls needs and what `interpreted` alone cannot
    /// give it.
    ///
    pub(in crate::source) fn interpretations(&self) -> usize {
        self.interpretations
    }
}

struct Execution<'a> {
    grid: Grid,
    original: Cells<'a>,
    working: WorkingCells,
    tick: Tick,
    /// The Language Units of the Source Snapshot, retained for occupancy and
    /// Jump's Language Unit at a Portal. A `Lookup` indexes Expressions, so a
    /// Comment and a standalone Bang are absent from it, and those questions
    /// classify every Language Unit rather than the computations alone.
    map: &'a LanguageMap,
    lookup: &'a Lookup,
    states: Vec<ComputationState>,
    effects: Vec<Effect>,
    /// Intact Functions and Bang displays placed this Tick. Neither has a pending Turn.
    placed_units: Vec<std::ops::Range<usize>>,
    /// Source-effect writes obscure Snapshot ownership even if a later value
    /// write replaces the generated Function. Neither creates a new computation.
    source_writes: Vec<std::ops::Range<usize>>,
}

impl<'a> Execution<'a> {
    ///
    /// The state one Tick starts from, before any computation takes a Turn.
    ///
    /// Every computation begins untouched, the schedule's own diagnostics are
    /// already recorded, and the previous revision's Bang display is cleared:
    /// the clearing is part of starting a Tick rather than part of taking a
    /// Turn, which is why it happens here and not in the loop that follows.
    ///
    fn new(
        grid: Grid,
        cells: Cells<'a>,
        map: &'a LanguageMap,
        tick: Tick,
        lookup: &'a Lookup,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        let mut execution = Self {
            grid,
            original: cells,
            working: WorkingCells::new(cells),
            tick,
            map,
            lookup,
            states: lookup
                .nodes()
                .iter()
                .map(|node| ComputationState {
                    function: node.function,
                    result: None,
                    syntax_blocked: false,
                    activated: false,
                    suppressed: false,
                    attempted: false,
                    #[cfg(test)]
                    turn: None,
                    #[cfg(test)]
                    interpreted: None,
                    #[cfg(test)]
                    interpretations: 0,
                })
                .collect(),
            effects: diagnostics.into_iter().map(Effect::Diagnose).collect(),
            placed_units: Vec::new(),
            source_writes: Vec::new(),
        };
        // Source content rather than an answer, so it is stated here rather than
        // rendered: a Bang occupies two Cells and clearing it writes two spaces.
        let blank = Encoding::literal("  ").expect("a space is a printable Cell");
        for (anchor, _) in map.bangs() {
            let clear = Portal::at(grid, anchor)
                .admit(&blank)
                .expect("parsed Bang fits its Grid");
            execution.write(clear);
        }
        execution
    }

    ///
    /// The refusals a Turn faces before it can have an answer at all, and the
    /// signature the Turn proceeds with when it faces none.
    ///
    /// A Turn nothing here settles is one whose answer decides the rest: which
    /// is why this stops at the signature, one step before the operands are
    /// resolved. Every arm settles the Turn rather than breaking the Tick, so
    /// it answers an `Option` and not a `ControlFlow` — the ordering defect
    /// that breaks a Tick is discovered by delivering an answer, never by a
    /// computation's own prologue.
    ///
    fn opens_turn(&mut self, index: usize) -> Option<lang::Tokens> {
        let node = &self.lookup.nodes()[index];
        // Activation is the owner's to declare, and the declaration that
        // decides it is the one the owner is running: the Function the Parser
        // found until an earlier replacement in this same Tick changed it. It
        // is independent of whether this computation will produce a typed
        // answer — a Self-Banging Function answers none and takes its Turn
        // anyway, which is why this asks the activation source rather than
        // `answers_value`.
        if self.states[index].suppressed
            || (!self.states[node.owner].function.is_intrinsically_active()
                && !self.states[node.owner].activated)
        {
            return None;
        }
        // Taking a Turn precedes syntax and evaluation checks. A later writer
        // must not reach a computation even when its attempted Turn failed.
        self.states[index].attempted = true;
        let function = self.states[index].function;
        // A nested Function that answers no value has no Return for its
        // parent. The Parser reports that against the Expression from Source
        // alone, so the Turn is blocked as an unparsed operand's is, without
        // repeating the report, and its parent is blocked in turn.
        if self.syntax_blocks(node, function)
            || (node.parent.is_some() && !function.answers_value())
        {
            self.states[index].syntax_blocked = true;
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

    /// A normal failure settles this Turn and records its diagnostic. A violated
    /// execution order breaks the Tick, discarding writes and Play Commands
    /// while retaining diagnostics.
    fn take_turn(&mut self, index: usize) -> ControlFlow<Diagnostic> {
        let Some(signature) = self.opens_turn(index) else {
            return Continue(());
        };
        let lookup = self.lookup;
        let node = &lookup.nodes()[index];
        let function = self.states[index].function;
        let tick = tick_inputs(self.tick, node.anchor);
        let result = self.operands(node, signature).and_then(|operands| {
            // Recorded beside the call rather than before it: a Turn whose
            // operands would not resolve is one the Interpreter never ran for,
            // and the record says which of the two happened.
            #[cfg(test)]
            {
                self.states[index].interpreted = Some(tick);
                self.states[index].interpretations += 1;
            }
            let inputs =
                FunctionInputs::with_portal_source(tick, self.portal_source(node, function));
            Interpreter::execute_function(function, operands, inputs)
                .map_err(|error| error.to_string())
        });
        match result {
            Err(message) => self.effects.push(Effect::Diagnose(diagnose(node, message))),
            Ok(Interpretation::Play(command)) => self.effects.push(Effect::Play(command)),
            Ok(Interpretation::Cell(atom)) => return self.deliver_value(index, atom),
            Ok(Interpretation::Source(effect)) => {
                return self.deliver_source_effect(index, effect);
            }
            Ok(Interpretation::Lock) => return self.lock_portal(index),
        }
        Continue(())
    }

    fn syntax_blocks(&self, node: &Computation, function: Function) -> bool {
        // Unchanged initial syntax errors belong to the Source revision.
        // Earlier writes or a Function replacement can repair those inputs.
        let unchanged = !node.syntax_valid
            && function == node.function
            && node.operands.iter().all(|operand| {
                self.working.cells().slice(operand.cells.clone()).bytes()
                    == self.original.slice(operand.cells.clone()).bytes()
            });
        // A syntax-blocked child did not fail evaluation. Propagate the block
        // without inventing another Tick diagnostic. A suppressed child is
        // instead consumed as literal characters from working Source.
        unchanged
            || node.operands.iter().any(|operand| {
                operand.child.is_some_and(|child| {
                    !self.states[child].suppressed && self.states[child].syntax_blocked
                })
            })
    }

    /// Borrow working Source at the Function's Input Portal.
    fn portal_source(&self, node: &Computation, function: Function) -> PortalSource<'_> {
        let Some(coords) = function.input_portal() else {
            return PortalSource::none();
        };
        if let Some(input) = function.portal_input() {
            return PortalSource::from_cells(self.borrow_portal_cells(node, coords, input));
        }
        PortalSource::from_cells(self.borrow_jump_input(node, coords))
    }

    /// Borrow one Portal's Cells directly from working Source. A missing or
    /// truncated site stays absent so binding diagnoses it after all cell
    /// operands have been validated.
    fn borrow_portal_cells(
        &self,
        node: &Computation,
        coords: PortalCoords,
        input: PortalInput,
    ) -> Option<&str> {
        let portal = Portal::named(self.grid, node.anchor, coords).ok()?;
        let span = portal.span(input.token().len()).ok()?;
        Some(self.working.text(span.range()))
    }

    /// The Cells a Jump reads, when they are one complete aligned unit.
    ///
    /// Invalid and partial input stay absent so the Interpreter diagnoses
    /// rather than answering an Atom that was never a Language Unit.
    fn borrow_jump_input(&self, node: &Computation, coords: PortalCoords) -> Option<&str> {
        let portal = Portal::named(self.grid, node.anchor, coords).ok()?;
        match portal.language_unit(self.working.cells(), self.map) {
            PortalUnit::Invalid => None,
            PortalUnit::Empty | PortalUnit::Bang | PortalUnit::Unit => {
                let span = portal
                    .reservation()
                    .expect("an admitted unit fitted its row");
                Some(self.working.text(span.range()))
            }
        }
    }

    /// The operands of `node`'s Turn, in signature order.
    ///
    /// Each operand is decoded by its declared Token, whichever way its
    /// characters arrived. Spatial delivery leaves them pending in working
    /// Source until consumption; a surviving nested child returns its answer's
    /// two-Cell encoding, taken out of its state because this Turn is the
    /// child's one consumer. The child's Atom type does not cross: a Note
    /// returned into a Number operand is read as the Number it spells, exactly
    /// as the same characters written there by a Portal would be.
    fn operands(
        &mut self,
        node: &Computation,
        signature: lang::Tokens,
    ) -> Result<Vec<Atom>, String> {
        node.operands
            .iter()
            .zip(signature)
            .map(|(operand, token)| {
                if let Some(child) = operand
                    .child
                    .filter(|child| !self.states[*child].suppressed)
                {
                    let anchor = self.lookup.nodes()[child].anchor;
                    let returned = match self.states[child].result.take().map(Encoding::render) {
                        Some(Ok(Rendered::Cells(encoding))) => encoding,
                        Some(Ok(Rendered::Nothing)) | None => {
                            return Err(format!(
                                "nested computation at column {}, row {} returned nothing",
                                anchor.x(),
                                anchor.y()
                            ));
                        }
                        // A rendering a Cell cannot hold is its own fault, not
                        // an absent answer.
                        Some(Err(reason)) => return Err(render_message(reason)),
                    };
                    return token
                        .decode(&returned.to_string())
                        .map_err(|error| error.to_string());
                }
                let spelling = self.working.text(operand.cells.clone());
                token.decode(spelling).map_err(|error| error.to_string())
            })
            .collect()
    }

    fn deliver_value(&mut self, index: usize, atom: Atom) -> ControlFlow<Diagnostic> {
        let flow = self.project_value(index, atom);
        // A successful nested answer survives every refusal to project it.
        self.states[index].result = Some(atom);
        flow
    }

    /// Plans the Cell writes, activation, or clear one answer makes, whether
    /// its computation is a root or nested.
    fn project_value(&mut self, index: usize, atom: Atom) -> ControlFlow<Diagnostic> {
        let node = &self.lookup.nodes()[index];
        // Every arm below plans or diagnoses a write at an Output Portal, so an
        // answer with none to write, which only its consumer reads, is not
        // rendered at all. Every arm below relies on this return and does not
        // ask `writes_cells` again.
        if !node.portal_access.writes_cells() {
            return Continue(());
        }
        // Whether this answer can be Cells at all is a question about the
        // value, settled before any destination is asked: the Absence Marker
        // plans no write and answers `Nothing`, and a rendering a Cell cannot
        // hold refuses whole. Every other Atom renders as the Cell pair the
        // schedule reserved.
        let encoding = match Encoding::render(atom) {
            Ok(Rendered::Nothing) => {
                // A Jump answers Empty when its input is two spaces. That is a
                // clear of the reserved output Portal, not an omitted write.
                if self.states[index].function.copies_language_unit() {
                    let cleared = Encoding::literal("  ").expect("a space is a printable Cell");
                    for output in node.portal_access.write_sites() {
                        self.deliver_output(index, Atom::Empty, &cleared, *output)?;
                    }
                }
                return Continue(());
            }
            Ok(Rendered::Cells(encoding)) => encoding,
            Err(reason) => {
                self.effects
                    .push(Effect::Diagnose(diagnose(node, render_message(reason))));
                return Continue(());
            }
        };
        for output in node.portal_access.write_sites() {
            self.deliver_output(index, atom, &encoding, *output)?;
        }
        Continue(())
    }

    fn deliver_output(
        &mut self,
        index: usize,
        atom: Atom,
        encoding: &Encoding,
        output: Result<Position, PortalError>,
    ) -> ControlFlow<Diagnostic> {
        let node = &self.lookup.nodes()[index];
        let destination = match output {
            Ok(destination) => destination,
            Err(reason) => {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    portal_message(reason, encoding),
                )));
                return Continue(());
            }
        };
        if atom == Atom::Bang && self.states[index].function.copies_language_unit() {
            if let Some(root) = self.lookup.root_at(destination) {
                self.states[root].activated = true;
                return Continue(());
            }
            if Portal::at(self.grid, destination).occupied_in(self.working.cells()) {
                let producer = self.states[index].function;
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    format!("{producer} cannot activate an occupied non-root"),
                )));
                return Continue(());
            }
        }
        let write = match Portal::at(self.grid, destination).admit(encoding) {
            Ok(write) => write,
            Err(reason) => {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    portal_message(reason, encoding),
                )));
                return Continue(());
            }
        };
        // The Cells this write actually covers: `Lookup::written_over`.
        let relationships = self.lookup.written_over(&write);
        if atom == Atom::Bang {
            for owner in relationships.bang_roots() {
                self.states[owner].activated = true;
            }
        }
        if let Atom::Function(replacement) = atom
            && let Some(change) = relationships.functions().find_map(|contact| {
                if !contact.at_anchor {
                    return None;
                }
                // The Function this computation is running, which is the one a
                // replacement replaces. It is the Function the Parser found
                // until an earlier replacement in this same Tick changed it,
                // and a second replacement reaching one anchor is what tells
                // the two apart. Every Function this file asks a computation
                // about is the running one, for the same reason;
                // `syntax_blocks` names the parsed Function, but there it is
                // the other half of a comparison rather than the Function in
                // force. No test can pin the choice: the guard keeps the
                // parsed and the running Function agreeing on every fact it
                // compares.
                let running = self.states[contact.index].function;
                replacement.replacing(running)
            })
        {
            // The fact that differed, not the list of facts that could have.
            // Each is argued on `ReplacementChange`, and the wording is
            // CONTEXT.md's, so the diagnostic continues the glossary's own
            // sentence about what a replacement may not change.
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                format!("Function replacement changes {change}"),
            )));
            return Continue(());
        }
        if relationships.functions().any(|mut contact| {
            contact
                .subtree
                .any(|descendant| self.states[descendant].attempted)
        }) {
            // A computation that has taken its Turn is past changing, so the
            // schedule ordered this write wrongly. Only this write is refused.
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                "spatial output reached an executed computation",
            )));
            return Continue(());
        }
        // A covered Expression is suppressed: its spelling is no longer the
        // one that was scheduled.
        for contact in relationships.functions() {
            let target = contact.index;
            if contact.at_anchor
                && !self.states[target].suppressed
                && let Atom::Function(replacement) = atom
            {
                self.states[target].function = replacement;
                continue;
            }
            for descendant in contact.subtree {
                self.states[descendant].suppressed = true;
            }
        }
        self.write(write);
        Continue(())
    }

    ///
    /// The one validated effect bundle for a Source-writing Function.
    ///
    /// The declared bundle decides which of the two it is. An `Advance` moves:
    /// an empty destination plans one validated Portal bundle, spaces over the
    /// current Span followed by its own spelling at the shifted destination,
    /// and a blocked or out-of-Grid move instead replaces its current Span with
    /// `**`. An `Emit` emits: the complete initial destination must be empty
    /// and inside the Grid, or the producer diagnoses and emits nothing.
    ///
    /// It is not [`Execution::deliver_value`] with a different destination. That
    /// path delivers one encoding through every Portal a computation resolved;
    /// an `Advance` writes different Cells at each of its two, and what it
    /// writes at the second decides what it writes at the first: spaces at its
    /// origin when the destination admits its spelling, `**` when it does not.
    ///
    /// The precondition is one rule and the refusal is two, which is why the
    /// groups share this path rather than each having one. What a refusal costs
    /// follows from whether the bundle plans the producer's own Span: a
    /// producer that was leaving those Cells reports in them, and a producer
    /// that is staying has nothing of its own to report in.
    ///
    /// Admission is atomic, as for ordinary value output, but empty-only
    /// placement cannot overwrite a standing Function. Snapshot ownership of
    /// vacated Cells therefore imposes no executed-computation guard here.
    /// An advancing bundle also clears its own Span; the schedule excludes
    /// that producer from its own overwrite dependency.
    ///
    /// The displacement is the Interpreter's answer and the destination
    /// `computations` reserved is the same declaration read before the Turn.
    /// Resolving it again here is what makes this the answer being delivered
    /// rather than the schedule replaying itself, which is the relationship
    /// every reservation has with the write it orders.
    ///
    fn deliver_source_effect(
        &mut self,
        index: usize,
        effect: SourceEffect,
    ) -> ControlFlow<Diagnostic> {
        let node = &self.lookup.nodes()[index];
        let anchor = node.anchor;
        let spelling = Encoding::literal(
            effect
                .spelling
                .expect("an Advance or Emit declares the Function it writes"),
        )
        .expect("a Function spelling is printable ASCII Cells");
        // What stands in the Source, which is not always what gets written. A
        // Self-Banging Function writes its own spelling and the two agree; a
        // Directional Bang Function writes the Function it emits, and only this
        // one names the Cells the author would go and fix. `Function` displays
        // as its spelling, which is how every other diagnostic in this file
        // names one.
        let producer = self.states[index].function;
        // The Cells this Function stands in. Every Function spelling is two
        // ASCII Cells, which `define_functions!` asserts at compile time, and
        // every spelling a Source-writing Function writes is another
        // Function's, so the Span it occupies and the Span it writes are the
        // same width and one length serves both.
        let advancing = effect.bundle == SourceBundle::Advance;
        let start = self.grid.index(anchor).get();
        let own = start..start + spelling.len();

        // Which Cells the precondition is asked about, and the one place the
        // two bundles read differently. An advancing Function tests only the
        // Cells newly entered by a one-Cell move — the whole destination for a
        // vertical move and one Cell of it for a horizontal one — and an
        // emitting one tests the complete initial destination. The two
        // coincide for every offset declared, because no emission overlaps its
        // producer, so the distinction is stated rather than relied on.
        //
        // The asymmetry is not carved into the write either way: an advancing
        // clear covers the complete old Span, and later-write-wins settles the
        // Cell the two share.
        let admitted = Portal::displaced(self.grid, anchor, effect.columns, effect.rows)
            .and_then(|portal| portal.admit(&spelling));
        let entered: Vec<usize> = match &admitted {
            Ok(write) => write
                .cells()
                .map(|(cell, _)| cell.get())
                .filter(|cell| !advancing || !own.contains(cell))
                .collect(),
            Err(_) => vec![],
        };
        let empty = entered.iter().all(|&cell| self.working.is_empty_at(cell));

        match admitted {
            Ok(write) if empty => {
                // Placement observes current vacancy, not Snapshot ownership.
                // No standing Function is overwritten or suppressed. A claimed
                // operand still waits for this supplier and decodes these Cells
                // when its consumer executes.
                if advancing {
                    // Stated rather than built, for the reason `Execution::new`
                    // states it. It clears `own`, which is always two Cells
                    // because every spelling is, so two spaces cover it.
                    let cleared = Encoding::literal("  ").expect("a space is a printable Cell");
                    let clear = Portal::at(self.grid, anchor)
                        .admit(&cleared)
                        .expect("a Function standing in the Source fits its own Span");
                    self.source_writes.push(clear.span().range());
                    self.write(clear);
                }
                let placed = write.span().range();
                self.source_writes.push(placed.clone());
                self.write(write);
                self.placed_units.push(placed);
            }
            // Refused: out of the Grid, past the row edge, or blocked by Cells
            // that are not empty. No partial write is admitted, so the whole
            // destination is gone in every case, and what the producer does
            // instead is the bundle's to say.
            _ if advancing => {
                // Source content rather than an answer: this Bang is the
                // display a refused move leaves, so it is stated here the way
                // the Bang cleanup in `Execution::new` is, and it reaches no
                // Portal of a value.
                let bang =
                    Encoding::literal("**").expect("the Bang spelling is printable ASCII Cells");
                let display = Portal::at(self.grid, anchor)
                    .admit(&bang)
                    .expect("a Function standing in the Source fits its own Span");
                let placed = display.span().range();
                self.source_writes.push(placed.clone());
                self.write(display);
                self.placed_units.push(placed);
                match self.contact_occupancy(&entered) {
                    // Only a surviving Snapshot root can receive activation.
                    // A placed unit has no pending computation this Tick.
                    Occupancy::Root(root) => self.states[root].activated = true,
                    Occupancy::Partial => self.effects.push(Effect::Diagnose(diagnose(
                        node,
                        format!("{producer} contacts part of a Language Unit"),
                    ))),
                    Occupancy::Empty | Occupancy::NonRoot => {}
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
                        .expect("an Emit declares the Function it writes"),
                ),
            ))),
        }
        Continue(())
    }

    /// Contact follows intact placements and the Snapshot Cells they have not
    /// obscured. Source writes establish unit geometry without reparsing or
    /// admitting generated computations into this Tick's schedule.
    fn contact_occupancy(&self, cells: &[usize]) -> Occupancy {
        let mut partial = false;
        for placed in &self.placed_units {
            if cells.iter().any(|cell| placed.contains(cell)) {
                if cells.iter().all(|cell| placed.contains(cell)) {
                    return Occupancy::NonRoot;
                }
                partial = true;
            }
        }
        for unit in self.map.units() {
            let span = unit.span().range();
            let covers = |cell: &usize| {
                span.contains(cell) && !self.source_writes.iter().any(|write| write.contains(cell))
            };
            if !cells.iter().any(covers) {
                continue;
            }
            if cells.iter().all(covers) {
                return self
                    .lookup
                    .root_at(unit.anchor())
                    .map_or(Occupancy::NonRoot, Occupancy::Root);
            }
            partial = true;
        }
        if partial {
            Occupancy::Partial
        } else {
            Occupancy::Empty
        }
    }

    ///
    /// Applies a lock to the Expression root at the Function's Output Portal.
    ///
    /// The schedule already placed this Turn ahead of that root, so a lock
    /// that finds it executed is a scheduler defect and rejects the Tick the
    /// same way a late spatial write does. An empty target is a no-op; an
    /// occupied non-root diagnoses and invents no lock. Halt itself is not
    /// suppressed here — `opens_turn` already refused a suppressed Halt, so
    /// reaching this arm means this Halt locks.
    ///
    fn lock_portal(&mut self, index: usize) -> ControlFlow<Diagnostic> {
        let node = &self.lookup.nodes()[index];
        match &self.lookup.locks[index] {
            Some(super::LockTarget::Root(target)) => {
                if target
                    .clone()
                    .any(|descendant| self.states[descendant].attempted)
                {
                    return Break(diagnose(
                        node,
                        "spatial output reached an executed computation; Tick effects rejected",
                    ));
                }
                for descendant in target.clone() {
                    self.states[descendant].suppressed = true;
                }
                self.effects
                    .push(Effect::Lock(self.lookup.nodes()[target.start].anchor));
            }
            Some(super::LockTarget::Occupied) => {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    format!("{} target is not an Expression root", node.function),
                )));
            }
            None | Some(super::LockTarget::Empty | super::LockTarget::Outside) => {}
        }
        Continue(())
    }

    /// Applying a write and recording its Effect are one operation, including
    /// the cleanup of prior Bang display before any Turn is attempted.
    fn write(&mut self, write: SpanWrite) {
        let written = write.span().range();
        self.placed_units
            .retain(|placed| written.end <= placed.start || placed.end <= written.start);
        for (cell, content) in write.cells() {
            self.working.write(cell, content);
        }
        self.effects.push(Effect::Write(write));
    }

    fn reject(mut self, diagnostic: Diagnostic) -> (TickPlan, Vec<ComputationState>) {
        // An ordering defect discards all writes, including Bang cleanup, and
        // independent Play Commands, but keeps ordered diagnostics. The states
        // survive it: what ran is still what ran, and a rejected Tick is the
        // one case an empty plan cannot be told apart from a quiet one.
        self.effects
            .retain(|effect| matches!(effect, Effect::Diagnose(_)));
        self.effects.push(Effect::Diagnose(diagnostic));
        (resolve(self.effects), self.states)
    }
}

/// Why a destination refused the value sent to it.
///
/// `OutsideGrid` is live for a Jump whose reserved output Portal left the
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
