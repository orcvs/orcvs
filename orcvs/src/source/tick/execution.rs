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
    let mut execution = Execution {
        grid,
        original: bytes,
        working: bytes.to_vec(),
        tick,
        lookup: &lookup,
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
    // Configuration's supplied answers exist only for bounded replacement
    // tests. Production used its destinations when it built the schedule.
    #[cfg(not(test))]
    let _ = configuration;

    // Source content rather than an answer, so it is stated here rather than
    // rendered: a Bang occupies two Cells and clearing it writes two spaces.
    let blank = Encoding::literal("  ").expect("a space is a printable Cell");
    for (anchor, _) in map.bangs() {
        let clear = Portal::at(grid, anchor)
            .admit(&blank)
            .expect("parsed Bang fits its Grid");
        execution.write(clear);
    }
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

impl Execution<'_> {
    /// A normal failure settles this Turn and records its diagnostic. A violated
    /// execution order breaks the Tick, discarding writes and Play Commands
    /// while retaining diagnostics.
    fn take_turn(&mut self, index: usize) -> ControlFlow<Diagnostic> {
        let nodes = self.lookup.nodes();
        let node = &nodes[index];
        // Activation belongs to the original owner's declared kind. It is
        // independent of whether this computation will produce a typed answer.
        if self.states[index].suppressed
            || (!nodes[node.owner].function.answers_value() && !self.states[node.owner].activated)
        {
            return Continue(());
        }
        // Taking a Turn precedes syntax and evaluation checks. A later writer
        // must not reach a computation even when its attempted Turn failed.
        self.states[index].attempted = true;
        if node.parent.is_some() && !node.function.answers_value() {
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                lang::InterpretationError::NestedEffectFunction.to_string(),
            )));
            return Continue(());
        }
        let function = self.states[index].function;
        if self.syntax_blocks(node, function) {
            self.states[index].syntax_blocked = true;
            return Continue(());
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
            return Continue(());
        }
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
