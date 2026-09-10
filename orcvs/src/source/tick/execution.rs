//! The changing state and Effects of one Tick, behind one execution seam.
//!
//! Lookup retains the original Parser structure. Execution owns whether its
//! computations can take a Turn, how their operands are consumed, and how an
//! admitted spatial result changes later Turns. Nothing here survives the Tick.

use std::ops::ControlFlow::{self, Break, Continue};

use lang::{Atom, Function, Interpretation, Interpreter, Tick, TickInputs, Value};

use super::{
    Computation, Diagnostic, Effect, Encoding, Grid, LanguageMap, Lookup, Portal, PortalError,
    Position, RenderError, Rendered, Schedule, SpanWrite, TickPlan, diagnose, resolve, tick_inputs,
};

///
/// Executes an established order against the original Source Snapshot.
///
/// The caller supplies no mutable state and receives two things: the
/// publishable Tick Plan, and what each computation's Turn actually did. They
/// are different facts, which is why the second is not folded into the first —
/// a Tick Plan says what to apply, and a rejected Tick applies nothing however
/// much of it ran.
///
pub(super) fn execute(
    grid: Grid,
    bytes: &[u8],
    map: &LanguageMap,
    tick: Tick,
    schedule: Schedule,
) -> (TickPlan, Vec<ComputationState>) {
    let Schedule {
        lookup,
        order,
        diagnostics,
    } = schedule;
    let mut execution = Execution::new(grid, bytes, map, tick, &lookup, diagnostics);
    for index in order {
        if let Break(diagnostic) = execution.take_turn(index) {
            return execution.reject(diagnostic);
        }
    }
    (resolve(execution.effects), execution.states)
}

/// These facts are independent: an attempted Turn can be syntax-blocked, and
/// a successful typed result can coexist with a rejected spatial delivery.
/// Keeping them together does not turn them into an exclusive lifecycle enum.
pub(in crate::source) struct ComputationState {
    function: Function,
    result: Option<Value>,
    syntax_blocked: bool,
    activated: bool,
    suppressed: bool,
    attempted: bool,
    /// The explicit inputs the Interpreter was handed for this computation, or
    /// `None` where it was never called for it. A Turn that was suppressed,
    /// refused by its own prologue, or stopped by operands it could not
    /// resolve reaches no Interpreter and keeps `None`.
    interpreted: Option<TickInputs>,
}

impl ComputationState {
    ///
    /// The inputs the Interpreter received for this computation, or `None`
    /// where it never ran for it.
    ///
    /// The one thing a caller outside this module reads off a state. Nothing
    /// publishes it yet: a Tick Plan carries what to apply, and this carries
    /// what happened, which is what a console or a diagnostic view will ask
    /// for and what the tests of this module ask for today.
    ///
    /// Allowed rather than expected: the method is dead in the library build
    /// and live in the test build, so an expectation would go unfulfilled in
    /// the second and fail the gate that compiles both.
    #[allow(dead_code, reason = "an output the shipped callers discard")]
    pub(in crate::source) fn interpreted(&self) -> Option<TickInputs> {
        self.interpreted
    }
}

struct Execution<'a> {
    grid: Grid,
    original: &'a [u8],
    working: Vec<u8>,
    tick: Tick,
    lookup: &'a Lookup,
    states: Vec<ComputationState>,
    effects: Vec<Effect>,
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
        bytes: &'a [u8],
        map: &LanguageMap,
        tick: Tick,
        lookup: &'a Lookup,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        let mut execution = Self {
            grid,
            original: bytes,
            working: bytes.to_vec(),
            tick,
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
                    interpreted: None,
                })
                .collect(),
            effects: diagnostics.into_iter().map(Effect::Diagnose).collect(),
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
        // Activation belongs to the owner's kind, and the kind that decides it
        // is the one the owner is running: the Function the Parser found until
        // an earlier replacement in this same Tick changed it. It is
        // independent of whether this computation will produce a typed answer.
        if self.states[index].suppressed
            || (!self.states[node.owner].function.answers_value()
                && !self.states[node.owner].activated)
        {
            return None;
        }
        // Taking a Turn precedes syntax and evaluation checks. A later writer
        // must not reach a computation even when its attempted Turn failed.
        self.states[index].attempted = true;
        // The Function this Turn will run, asked for here rather than below
        // because the nesting rule is about the answer this computation is
        // going to produce, which is the running Function's to declare.
        let function = self.states[index].function;
        if node.parent.is_some() && !function.answers_value() {
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                lang::InterpretationError::NestedEffectFunction.to_string(),
            )));
            return None;
        }
        if self.syntax_blocks(node, function) {
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
        let node = &self.lookup.nodes()[index];
        let function = self.states[index].function;
        let inputs = tick_inputs(self.tick, node.anchor);
        let result = self.operands(node, signature).and_then(|operands| {
            // Recorded beside the call rather than before it: a Turn whose
            // operands would not resolve is one the Interpreter never ran for,
            // and the record says which of the two happened.
            self.states[index].interpreted = Some(inputs);
            Interpreter::execute_function(function, &operands, inputs)
                .map_err(|error| error.to_string())
        });
        match result {
            Err(message) => self.effects.push(Effect::Diagnose(diagnose(node, message))),
            Ok(Interpretation::Play(performance)) => self.effects.push(Effect::Play(performance)),
            Ok(Interpretation::Cell(atom)) => return self.deliver_value(index, Value::Atom(atom)),
            Ok(Interpretation::Sequence(sequence)) => {
                return self.deliver_value(index, Value::Sequence(sequence));
            }
        }
        Continue(())
    }

    fn syntax_blocks(&self, node: &Computation, function: Function) -> bool {
        // Unchanged initial syntax errors belong to the Source revision.
        // Earlier writes or a Function replacement can repair those inputs.
        let unchanged = !node.syntax_valid
            && function == node.function
            && node.operands.iter().all(|operand| {
                self.working[operand.cells.clone()] == self.original[operand.cells.clone()]
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

    fn operands(&self, node: &Computation, signature: lang::Tokens) -> Result<Vec<Value>, String> {
        node.operands
            .iter()
            .zip(signature)
            .map(|(operand, token)| {
                if let Some(child) = operand
                    .child
                    .filter(|child| !self.states[*child].suppressed)
                {
                    let anchor = self.lookup.nodes()[child].anchor;
                    return self.states[child].result.clone().ok_or_else(|| {
                        format!(
                            "nested computation at column {}, row {} supplied no typed result",
                            anchor.x(),
                            anchor.y()
                        )
                    });
                }
                // Spatial delivery leaves characters pending until consumption;
                // a surviving nested child instead supplies an already typed value.
                let spelling = std::str::from_utf8(&self.working[operand.cells.clone()])
                    .expect("ASCII Source");
                token
                    .decode(spelling)
                    .map(Value::from)
                    .map_err(|error| error.to_string())
            })
            .collect()
    }

    fn deliver_value(&mut self, index: usize, value: Value) -> ControlFlow<Diagnostic> {
        let node = &self.lookup.nodes()[index];
        // A successful nested answer survives every refusal to project it.
        self.states[index].result = Some(value.clone());
        // Whether this answer can be Cells at all is a question about the
        // value, settled before any destination is asked: the two values that
        // plan no write answer `Nothing`, and a rendering a Cell cannot hold
        // refuses whole. A Sequence needs nothing of its own here, which is
        // the point — `Portal::admit` refuses an encoding wider than its row
        // entire and `SpanWrite::cells` fans one admitted write out Cell-wise,
        // so ADR 0007's complete-fit rule and ADR 0020's Cell-wise conflict
        // resolution are inherited rather than restated for a second width.
        let encoding = match Encoding::render(&value) {
            Ok(Rendered::Nothing) => return Continue(()),
            Ok(Rendered::Cells(encoding)) => encoding,
            Err(reason) => {
                if !node.outputs.is_empty() {
                    self.effects
                        .push(Effect::Diagnose(diagnose(node, render_message(reason))));
                }
                return Continue(());
            }
        };
        // ADR 0036: scheduling reserved one Cell pair for a computation whose
        // answer could not be a Sequence, so any other width from one would
        // write Cells no dependency edge names.
        if !self.lookup.reserved(index).admits_width(encoding.len()) {
            if !node.outputs.is_empty() {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    "result is not a scalar Cell pair",
                )));
            }
            return Continue(());
        }
        for output in &node.outputs {
            self.deliver_output(index, &value, &encoding, *output)?;
        }
        Continue(())
    }

    fn deliver_output(
        &mut self,
        index: usize,
        value: &Value,
        encoding: &Encoding,
        output: Result<Position, PortalError>,
    ) -> ControlFlow<Diagnostic> {
        let node = &self.lookup.nodes()[index];
        let write = match output.and_then(|output| Portal::at(self.grid, output).admit(encoding)) {
            Ok(write) => write,
            Err(reason) => {
                self.effects.push(Effect::Diagnose(diagnose(
                    node,
                    portal_message(reason, encoding),
                )));
                return Continue(());
            }
        };
        let output = output.expect("an admitted write has a destination");
        // The Cells this write actually covers, not the Cells scheduling
        // reserved for it. The two coincide for a scalar answer and come apart
        // for a Sequence, whose reservation runs to the end of its row: a
        // computation inside that reservation which the encoding stopped short
        // of was ordered after this producer and then never written over, so it
        // is neither suppressed nor replaced. Ordering is what a reservation
        // decides; what happened to a Cell is what the write decides.
        let relationships = self.lookup.written_over(output, encoding.len());
        // Both rules below read `value` rather than the Cells, and both are
        // therefore untouched by the width of the write: `Atom::Bang` and
        // `Atom::Function` are single Atoms by construction, so a Sequence
        // answer never satisfies either pattern. A Sequence carrying a Function
        // spelling writes those two Cells as ordinary Source content under
        // ADR 0007 — the next Tick's parse reads a Function there, this one
        // replaces nothing.
        if *value == Value::Atom(Atom::Bang) {
            for owner in relationships.bang_roots() {
                self.states[owner].activated = true;
            }
        }
        if let Value::Atom(Atom::Function(replacement)) = value
            && relationships.functions().any(|contact| {
                // The Function this computation is running, which is the one a
                // replacement replaces. It is the Function the Parser found
                // until an earlier replacement in this same Tick changed it,
                // and a second replacement reaching one anchor is what tells
                // the two apart. Every Function this file asks a computation
                // about is the running one, for the same reason;
                // `syntax_blocks` names the parsed Function, but there it is
                // the other half of a comparison rather than the Function in
                // force.
                //
                // No test pins the choice, and none can be written. Terms one
                // and two are the two facts this guard refuses to let a
                // replacement change, so the first admitted replacement leaves
                // the parsed and the running Function agreeing on both, and
                // term three reads neither one. Reverting this line to
                // `self.lookup.nodes()[contact.index].function` passes the
                // whole suite. The running Function is read because it is the
                // one being replaced, not because a fixture can say so.
                let target = self.states[contact.index].function;
                contact.at_anchor
                    && (replacement.answers_value() != target.answers_value()
                        || replacement.can_emit_bang() != target.can_emit_bang()
                        // ADR 0036: a schedule reserves Cells from the Function
                        // it found at each anchor, so a replacement that would
                        // widen or narrow that reservation is refused with the
                        // ones that change activation or output kind. What it
                        // is compared against stays the settled reservation:
                        // the Turns were ordered from that one, and this same
                        // guard is what keeps every admitted replacement inside
                        // it. The widths it reads are its children's, settled
                        // the same way.
                        || self.lookup.would_reserve(contact.index, *replacement)
                            != self.lookup.reserved(contact.index))
            })
        {
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                "Function replacement changes activation requirements, output kind, or result width",
            )));
            return Continue(());
        }
        if relationships.functions().any(|mut contact| {
            contact
                .subtree
                .any(|descendant| self.states[descendant].attempted)
        }) {
            return Break(diagnose(
                node,
                "spatial output reached an executed computation; Tick effects rejected",
            ));
        }
        // The one rule of the three that a wide write genuinely changes: a
        // Sequence can cover several Expressions along its row, and each of
        // them is suppressed for the same reason a scalar suppresses the one it
        // covers — its spelling is no longer the one that was scheduled.
        for contact in relationships.functions() {
            let target = contact.index;
            if contact.at_anchor
                && !self.states[target].suppressed
                && let Value::Atom(Atom::Function(replacement)) = value
            {
                self.states[target].function = *replacement;
                continue;
            }
            for descendant in contact.subtree {
                self.states[descendant].suppressed = true;
            }
        }
        self.write(write);
        Continue(())
    }

    /// Applying a write and recording its Effect are one operation, including
    /// the cleanup of prior Bang display before any Turn is attempted.
    fn write(&mut self, write: SpanWrite) {
        for (cell, content) in write.cells() {
            self.working[cell.get()] = content.as_char() as u8;
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

fn portal_message(reason: PortalError, encoding: &Encoding) -> String {
    let encoding = encoding.to_string();
    match reason {
        PortalError::BelowSource => format!("result {encoding:?} falls below the Source"),
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

///
/// One Tick whose named producers answer a value a test states rather than one
/// the Interpreter computes.
///
/// ADR 0034 defers the Source operation that produces Function values, so no
/// Function spelling answers one; and every Atom a Function does answer encodes
/// as the Cell pair a scalar result reserves. The rules those two absences
/// leave unreachable — replacement at an original anchor, and ADR 0036's
/// refusal of a result that is not the Cell pair the schedule reserved — are
/// therefore reached only from a value a test constructs. That value
/// is constructed here, one call below [`execute`], so that the shipped Turn
/// stays one thing in every build: the Interpreter's answer, delivered.
///
/// A reservation is stated the same way and for the same reason: ADR 0036
/// derives one from what a Function declares its answer to be, and no built
/// Function declares a Sequence answer, so the width a Sequence-answering row
/// reserves is stated beside the answer rather than derived from it. Two
/// consequences of stating it are worth knowing, and both are refused loudly
/// rather than discovered:
///
/// - A stated width is not a declared one, so `Lookup::would_reserve` — which
///   re-derives a width for a hypothetical replacement Function — cannot agree
///   with it at the computation whose width was stated. Stating a reservation
///   and stating a Function replacement in one Tick is therefore refused here.
///   `test-only-seams/09` owns the underlying modelling problem: the stored
///   width and the re-derived one are two homes for one fact.
/// - A width the fixture states is the input to every width production derives
///   around it, so the pass `Lookup::new` ran is run again over the stated
///   ones. Nothing about that pass is proven by these tests: no Function
///   declares a Sequence answer, so every `Reserved::Row` in the crate is a
///   stated one, and `sequence-values/05` owes the derivation against a
///   declared Range row.
///
/// Only the Turn loop is reimplemented, because substituting one Turn is the
/// one thing this does differently. The starting state, the Bang cleanup it
/// performs, the schedule, the rejection path, the resolution, and the Turn
/// every other computation takes are all the production ones, reached through
/// the same [`Execution::new`] that [`execute`] reaches them through.
///
#[cfg(test)]
pub(super) mod stated {
    use lang::{Tick, Value};

    use super::super::{
        Configuration, Reserved, carry, computations, derive_reservations, order_turns, unscheduled,
    };
    use super::{
        Atom, Break, ComputationState, Continue, ControlFlow, Diagnostic, Execution, Grid,
        LanguageMap, Lookup, Position, Schedule, TickPlan, resolve,
    };
    use crate::grid::CellIndex;
    use std::collections::BTreeMap;

    ///
    /// Plans one Tick, delivering the value stated for a computation's anchor
    /// in place of the answer that computation would have interpreted.
    ///
    pub(in crate::source::tick) fn plan_with_answers(
        grid: Grid,
        bytes: &[u8],
        map: &LanguageMap,
        tick: Tick,
        destinations: &BTreeMap<CellIndex, Vec<Position>>,
        reservations: &[(CellIndex, Reserved)],
        answers: &[(CellIndex, Value)],
    ) -> (TickPlan, Vec<ComputationState>) {
        let (mut nodes, mut layout) = computations(grid, map, &Configuration::default());
        carry(grid, &mut nodes, &mut layout, destinations);
        let mut lookup = Lookup::new(grid, nodes);
        // Every fixture error the schedule can be asked about is asked here,
        // before an order exists. A Source with a cycle answers `Err` from
        // `order_turns` and a plan carrying nothing but diagnostics, which is
        // indistinguishable from the little a mistyped fixture makes happen —
        // so a guard that ran after ordering would be the one guard a cyclic
        // fixture switches off.
        for (anchor, _) in answers {
            assert!(
                anchored(&lookup, grid, *anchor).is_some(),
                "a stated answer names a computation the schedule contains"
            );
        }
        for (index, (anchor, _)) in answers.iter().enumerate() {
            assert!(
                !answers[..index].iter().any(|(stated, _)| stated == anchor),
                "one computation is stated one answer: two are stated here for the same anchor"
            );
        }
        for (index, (anchor, reserved)) in reservations.iter().enumerate() {
            assert!(
                !reservations[..index]
                    .iter()
                    .any(|(stated, _)| stated == anchor),
                "one computation is stated one reservation: two are stated here for the same anchor"
            );
            assert!(
                *reserved == Reserved::Row,
                "a stated reservation is a width production would not derive: \
                 `Reserved::Pair` is what `derive_reservations` answers for every \
                 computation that states nothing, so stating one says nothing and \
                 is overwritten by the pass that reads it"
            );
        }
        // A replacement's width is derived from what it declares and compared
        // against what its target reserves, and a stated reservation is a width
        // nothing declares. `Lookup::would_reserve` would answer for the
        // replacement and disagree with the stated width for every replacement
        // there is, including the target's own Function, which production
        // admits. Refusing the combination keeps that from being discovered as
        // a wrong answer inside a test.
        assert!(
            reservations.is_empty()
                || !answers
                    .iter()
                    .any(|(_, value)| matches!(value, Value::Atom(Atom::Function(_)))),
            "a stated reservation and a stated Function replacement cannot be combined: \
             a replacement is checked against a declared width and this one is stated"
        );
        // Stated between the reservations a Lookup derives and the order those
        // reservations decide, because a reservation is read by both halves of
        // a schedule and only one of them is execution. Ordering, admission,
        // suppression and rejection are all read out of it downstream, so
        // stating it after `order_turns` would leave every edge derived from
        // the width the fixture is replacing.
        for (anchor, reserved) in reservations {
            let index = anchored(&lookup, grid, *anchor)
                .expect("a stated reservation names a computation the schedule contains");
            lookup.nodes[index].reserved = *reserved;
        }
        // What a stated reservation leaves for production to derive: an
        // ancestor that widens over a row-reserving operand widens over a
        // stated one exactly as it will over a declared Range, because this is
        // the pass `Lookup::new` ran, run again over the widths now settled.
        // Without it a stated child would leave its pervasive ancestor holding
        // the `Reserved::Pair` derived before the fixture spoke, and the
        // ancestor's own wide answer would be refused for a width the schedule
        // never reserved.
        derive_reservations(&mut lookup.nodes);
        let Schedule {
            lookup,
            order,
            diagnostics,
        } = match order_turns(lookup, layout) {
            Ok(schedule) => schedule,
            Err(diagnostics) => return unscheduled(diagnostics),
        };
        let mut execution = Execution::new(grid, bytes, map, tick, &lookup, diagnostics);
        let mut stated = vec![false; answers.len()];
        for index in order {
            let anchor = grid.index(lookup.nodes()[index].anchor);
            let turn = match answers.iter().position(|(stated, _)| *stated == anchor) {
                Some(position) => {
                    stated[position] = true;
                    execution.state_answer(index, answers[position].1.clone())
                }
                None => execution.take_turn(index),
            };
            if let Break(diagnostic) = turn {
                // The order stops where a rejection found it, so the answers
                // after that Turn are unstated for a reason the fixture chose.
                return execution.reject(diagnostic);
            }
        }
        // Reached only where an order existed and ran to its end. The two
        // returns above skip it for reasons the fixture can see in the plan it
        // gets back: a rejection stops the order where it found the defect, and
        // a cycle admits no order at all and publishes diagnostics and nothing
        // else. Neither can be mistaken for a stated answer whose computation
        // the order never reached, which is the one thing this guard is for.
        //
        // Reaching the Turn is all it claims, and all it can claim: the flag
        // is set before `state_answer` runs, and a Turn `opens_turn` refuses
        // is settled with the answer undelivered. That is the seam behaving as
        // `state_answer` documents rather than a hole in the guard — a stated
        // answer says what a computation answers, never whether it answers at
        // all — so a fixture whose computation is suppressed for a reason it
        // did not intend is a fixture that asserts a quiet Tick and is told it
        // got one. What this refuses is the narrower thing it names: an answer
        // stated for a Turn the order never took.
        assert!(
            stated.iter().all(|stated| *stated),
            "every stated answer reached the Turn of the computation it names"
        );
        (resolve(execution.effects), execution.states)
    }

    ///
    /// The computation anchored at `anchor`, as the index everything else here
    /// addresses it by, or `None` where no computation is anchored there.
    ///
    /// Both halves of a fixture name a computation by the Cell its Function is
    /// anchored at, because that is the coordinate a fixture author can read
    /// off the Source rows they wrote. Every use is an existence check first:
    /// an anchor naming no computation is the fixture error this answers
    /// `None` for, and the callers turn into a panic naming which half — the
    /// answer or the reservation — named it.
    ///
    fn anchored(lookup: &Lookup, grid: Grid, anchor: CellIndex) -> Option<usize> {
        lookup
            .nodes()
            .iter()
            .position(|node| grid.index(node.anchor) == anchor)
    }

    impl Execution<'_> {
        ///
        /// Delivers `value` as this computation's answer, in place of the
        /// operands it would have resolved and the interpretation it would have
        /// answered from them.
        ///
        /// It replaces those two and nothing else. Every refusal
        /// [`Execution::opens_turn`] applies still applies, through the same
        /// call [`Execution::take_turn`] makes: a stated answer says what this
        /// computation answers, never whether it answers at all. A Turn that
        /// prologue settles — suppressed, inactive, nested and effectful,
        /// syntax-blocked, or refused for its arity — is settled here too, and
        /// the stated answer is never delivered.
        ///
        fn state_answer(&mut self, index: usize, value: Value) -> ControlFlow<Diagnostic> {
            if self.opens_turn(index).is_none() {
                return Continue(());
            }
            // The one arm of the Turn that decides an answer's kind rather than
            // its content, and the only one left standing here: a Function
            // answering an effect answers a Play Command, and the root is the
            // one place `opens_turn` lets such a Function stand — it diagnoses
            // every other. Delivering a value there would plan a write from a
            // computation production can only ever hear a Performance from.
            assert!(
                self.states[index].function.answers_value(),
                "a stated answer belongs to a computation that answers a value"
            );
            self.deliver_value(index, value)
        }
    }
}
