//! The changing state and Effects of one Tick, behind one execution seam.
//!
//! Lookup retains the original Parser structure. Execution owns whether its
//! computations can take a Turn, how their operands are consumed, and how an
//! admitted spatial result changes later Turns. Nothing here survives the Tick.

use std::ops::ControlFlow::{self, Break, Continue};

use lang::{Atom, Function, Interpretation, Tick, Value};

use super::{
    Computation, Configuration, Diagnostic, Effect, Grid, LanguageMap, Lookup, Portal, PortalError,
    Position, SCALAR_WIDTH, Schedule, SpanWrite, TickPlan, diagnose, interpret, resolve,
    tick_inputs,
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

    for (anchor, _) in map.bangs() {
        let clear = Portal::at(grid, anchor)
            .admit("  ")
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
            .map_or(result, |atom| Ok(Interpretation::Cell(*atom)));
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
        let encoding = match &value {
            Value::Atom(Atom::Empty) => return Continue(()),
            Value::Atom(atom) => atom.to_string(),
            Value::Sequence(sequence) if sequence.is_empty() => return Continue(()),
            Value::Sequence(sequence) => {
                if !node.outputs.is_empty() {
                    self.effects.push(Effect::Diagnose(diagnose(
                        node,
                        format!(
                            "Sequence result {:?} has no fixed scalar scheduling footprint",
                            sequence.to_string()
                        ),
                    )));
                }
                return Continue(());
            }
        };
        // The dependency schedule reserves one scalar Cell pair per destination.
        // A different width cannot safely use those dependency edges.
        if encoding.len() != SCALAR_WIDTH {
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
        encoding: &str,
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
        let relationships = self
            .lookup
            .at(output)
            .expect("an admitted Cell pair fits its row");
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
                        || replacement.can_emit_bang() != target.function.can_emit_bang())
            })
        {
            self.effects.push(Effect::Diagnose(diagnose(
                node,
                "Function replacement changes activation requirements or output kind",
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

fn portal_message(reason: PortalError, encoding: &str) -> String {
    match reason {
        PortalError::BelowSource => format!("result {encoding:?} falls below the Source"),
        PortalError::CrossesRowEdge => format!("result {encoding:?} crosses the row edge"),
        PortalError::InvalidContent => {
            format!("result {encoding:?} contains Cells outside printable ASCII")
        }
    }
}
