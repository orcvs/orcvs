//! The changing state and Effects of one Tick, behind one execution seam.
//!
//! Lookup retains the original Parser structure. Execution owns whether its
//! computations can take a Turn, how their operands are consumed, and how an
//! admitted spatial result changes later Turns. Nothing here survives the Tick.

use std::ops::ControlFlow::{self, Break, Continue};

use lang::{Atom, Function, Interpretation, Tick, Value};

use super::{
    Computation, Configuration, Diagnostic, Effect, Encoding, Grid, LanguageMap, Lookup, Portal,
    PortalError, Position, RenderError, Rendered, Reserved, SCALAR_WIDTH, Schedule, SpanWrite,
    TickPlan, diagnose, interpret, resolve, tick_inputs,
};

/// Executes an established order against the original Source Snapshot. The
/// caller supplies no mutable state and receives only the publishable Tick Plan.
pub(super) fn execute(
    grid: Grid,
    bytes: &[u8],
    map: &LanguageMap,
    tick: Tick,
    configuration: &Configuration,
    schedule: Schedule,
) -> TickPlan {
    let Schedule {
        lookup,
        order,
        diagnostics,
    } = schedule;
    let mut execution = Execution::new(grid, bytes, map, tick, configuration, &lookup, diagnostics);
    for index in order {
        if let Break(diagnostic) = execution.take_turn(index) {
            return execution.reject(diagnostic);
        }
    }
    resolve(execution.effects)
}

/// These facts are independent: an attempted Turn can be syntax-blocked, and
/// a successful typed result can coexist with a rejected spatial delivery.
/// Keeping them together does not turn them into an exclusive lifecycle enum.
struct ComputationState {
    function: Function,
    result: Option<Value>,
    syntax_blocked: bool,
    activated: bool,
    suppressed: bool,
    attempted: bool,
}

struct Execution<'a> {
    grid: Grid,
    original: &'a [u8],
    working: Vec<u8>,
    tick: Tick,
    lookup: &'a Lookup,
    states: Vec<ComputationState>,
    effects: Vec<Effect>,
    #[cfg(test)]
    configuration: &'a Configuration,
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
        configuration: &'a Configuration,
        lookup: &'a Lookup,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        // Configuration's supplied answers exist only for bounded replacement
        // tests. Production used its destinations when it built the schedule.
        #[cfg(not(test))]
        let _ = configuration;
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
                })
                .collect(),
            effects: diagnostics.into_iter().map(Effect::Diagnose).collect(),
            #[cfg(test)]
            configuration,
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
        let nodes = self.lookup.nodes();
        let node = &nodes[index];
        // Activation belongs to the original owner's declared kind. It is
        // independent of whether this computation will produce a typed answer.
        if self.states[index].suppressed
            || (!nodes[node.owner].function.answers_value() && !self.states[node.owner].activated)
        {
            return None;
        }
        // Taking a Turn precedes syntax and evaluation checks. A later writer
        // must not reach a computation even when its attempted Turn failed.
        self.states[index].attempted = true;
        if node.parent.is_some() && !node.function.answers_value() {
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                lang::InterpretationError::NestedEffectFunction.to_string(),
            )));
            return None;
        }
        let function = self.states[index].function;
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
        let result = self.operands(node, signature).and_then(|operands| {
            interpret(function, &operands, tick_inputs(self.tick, node.anchor))
                .map_err(|error| error.to_string())
        });
        #[cfg(test)]
        let result = self
            .configuration
            .supplied
            .get(&self.grid.index(node.anchor))
            .map_or(result, |answer| Ok(answer.clone()));
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
        // write Cells no dependency edge names. A narrower answer is refused
        // alongside a wider one: the reservation is what the row fit was
        // decided against, and a single Cell at the last Cell of a row is a
        // write the Portal admits and the schedule never reserved.
        if self.lookup.reserved(index) == Reserved::Pair && encoding.len() != SCALAR_WIDTH {
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
                let target = &self.lookup.nodes()[contact.index];
                contact.at_anchor
                    && (replacement.answers_value() != target.function.answers_value()
                        || replacement.can_emit_bang() != target.function.can_emit_bang()
                        // ADR 0036: a schedule reserves Cells from the Function
                        // it found at each anchor, so a replacement that would
                        // widen or narrow that reservation is refused with the
                        // ones that change activation or output kind. The
                        // reservations it reads are its children's, which this
                        // same guard keeps as the schedule settled them.
                        || self.lookup.reserved_with(contact.index, *replacement)
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

    fn reject(mut self, diagnostic: Diagnostic) -> TickPlan {
        // An ordering defect discards all writes, including Bang cleanup, and
        // independent Play Commands, but keeps ordered diagnostics.
        self.effects
            .retain(|effect| matches!(effect, Effect::Diagnose(_)));
        self.effects.push(Effect::Diagnose(diagnostic));
        resolve(self.effects)
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
/// - A stated width is not a declared one, so `Lookup::reserved_with` — which
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

    use super::super::{computations, derive_reservations, order_turns, unscheduled};
    use super::{
        Atom, Break, Configuration, Continue, ControlFlow, Diagnostic, Execution, Grid,
        LanguageMap, Lookup, Reserved, Schedule, TickPlan, resolve,
    };
    use crate::grid::CellIndex;

    ///
    /// Plans one Tick, delivering the value stated for a computation's anchor
    /// in place of the answer that computation would have interpreted.
    ///
    pub(in crate::source::tick) fn plan_with_answers(
        grid: Grid,
        bytes: &[u8],
        map: &LanguageMap,
        tick: Tick,
        configuration: &Configuration,
        reservations: &[(CellIndex, Reserved)],
        answers: &[(CellIndex, Value)],
    ) -> TickPlan {
        let (nodes, layout) = computations(grid, map, configuration);
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
        for (index, (anchor, _)) in reservations.iter().enumerate() {
            assert!(
                !reservations[..index]
                    .iter()
                    .any(|(stated, _)| stated == anchor),
                "one computation is stated one reservation: two are stated here for the same anchor"
            );
        }
        // A replacement's width is derived from what it declares and compared
        // against what its target reserves, and a stated reservation is a width
        // nothing declares. `Lookup::reserved_with` would answer for the
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
            lookup.reserved[index] = *reserved;
        }
        // What a stated reservation leaves for production to derive: an
        // ancestor that widens over a row-reserving operand widens over a
        // stated one exactly as it will over a declared Range, because this is
        // the pass `Lookup::new` ran, run again over the widths now settled.
        // Without it a stated child would leave its pervasive ancestor holding
        // the `Reserved::Pair` derived before the fixture spoke, and the
        // ancestor's own wide answer would be refused for a width the schedule
        // never reserved.
        derive_reservations(&lookup.nodes, &mut lookup.reserved);
        let Schedule {
            lookup,
            order,
            diagnostics,
        } = match order_turns(lookup, layout) {
            Ok(schedule) => schedule,
            Err(diagnostics) => return unscheduled(diagnostics),
        };
        let mut execution =
            Execution::new(grid, bytes, map, tick, configuration, &lookup, diagnostics);
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
        assert!(
            stated.iter().all(|stated| *stated),
            "every stated answer reached the Turn of the computation it names"
        );
        resolve(execution.effects)
    }

    ///
    /// What scheduling reserved for the computation anchored at `anchor`, as a
    /// place a fixture can state into.
    ///
    /// ADR 0036 derives a reservation from what a Function declares its answer
    /// to be, and no built Function declares a Sequence one — ADR 0007's Range
    /// and Concatenate are unbuilt — so the width a Sequence-answering row
    /// reserves is the second thing a test has to state alongside the answer
    /// itself. It is stated as the reservation, not as an answer the schedule
    /// reads back: what a computation answers is a fact of the Tick, and what
    /// it reserves is a fact of the schedule, and the seam that let one imply
    /// the other is what these tickets are removing.
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
