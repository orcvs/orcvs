//! What a Function's operands consume at one attempted Turn.
//!
//! Each operand consumes one of two things, and [`Execution::returning_child`]
//! alone decides which: a nested Function that survives supplies its Return,
//! and any other operand supplies the Cells working Source holds at it. For a
//! child a write suppressed those are the two Cells at its anchor. Whether a
//! child blocks the Function, what each operand decodes to, and where the
//! operands end all follow from that decision.
//!
//! Nothing here is kept between attempts. A Turn that waits for a writer
//! resolves its operands again from current execution state when it retries,
//! and reads a completed child's Return from that child's state rather than
//! interpreting the child again.

use lang::{Atom, Function};

use super::{
    Answer, Computation, ComputationId, Encoding, Execution, Operand, Position, SCALAR_WIDTH,
};

/// What a Turn's operands decode to.
pub(super) enum Decoded {
    /// The operands, in signature order.
    Operands(Vec<Atom>),
    /// The Function is a MIDI output and an operand's two Cells are empty,
    /// with every other operand decoded or empty: the Turn is taken with
    /// nothing to send.
    Empty,
}

impl Execution<'_> {
    /// The nested Function whose Return `operand` consumes, or `None` where
    /// it consumes the Cells at `operand.cells`.
    fn returning_child(&self, operand: &Operand) -> Option<ComputationId> {
        operand.child.filter(|&child| !self.turns.suppressed(child))
    }

    ///
    /// Whether a nested Function returning to `node` was blocked by a syntax
    /// error the Source revision already reports, which blocks `node` in
    /// turn without repeating the report.
    ///
    pub(super) fn blocked_by_child(&self, node: &Computation) -> bool {
        node.operands.iter().any(|operand| {
            self.returning_child(operand)
                .is_some_and(|child| self.turns.syntax_blocked(child))
        })
    }

    /// The operands of `node`'s Turn, in signature order, or
    /// [`Decoded::Empty`] where a MIDI output has nothing to send.
    ///
    /// Each operand is decoded by its declared Token, whichever way its
    /// characters arrived. Spatial delivery leaves them pending in working
    /// Source until consumption, and a returning child supplies its answer's
    /// two-Cell encoding. The child's Atom type does not cross: a Note
    /// returned into a Number operand is read as the Number it spells,
    /// exactly as the same characters written there by a Portal would be.
    ///
    /// The first operand that does not decode refuses the Turn. A slot with
    /// no written Cell refuses it as a partly written slot does. A child that
    /// answered a Clear, a pulse that did not Bang or a Copy, Read or Track
    /// of empty Cells, returns no value, and the refusal names that child.
    ///
    /// The one exception is a MIDI output: an operand whose two Cells are
    /// empty, or whose child answered a Clear and so wrote two empty Cells,
    /// leaves it nothing to send. Any other operand that does not decode still
    /// refuses the Turn, wherever it stands, so the order of the operands
    /// does not matter and nothing is held back.
    pub(super) fn decode(
        &self,
        node: &Computation,
        function: Function,
        signature: lang::Tokens,
    ) -> Result<Decoded, String> {
        let may_be_empty = function.sends_nothing_on_an_empty_operand();
        let mut empty = false;
        let mut operands = Vec::with_capacity(node.operands.len());
        for (operand, token) in node.operands.iter().zip(signature) {
            let rendered;
            let spelling = match self.returning_child(operand) {
                None => self.working.text(operand.cells.clone()),
                Some(child) => match self.turns.result(child) {
                    // A Clear wrote two empty Cells at the child's Output
                    // Portal, so the operand holds what a blank one holds.
                    Some(Answer::Clear) if may_be_empty => lang::BLANK_PAIR,
                    // A Clear returns no value. The parent names the child,
                    // since nothing at the operand says which Function
                    // cleared.
                    Some(Answer::Clear) => {
                        return Err(nested(
                            self.schedule.lookup.nodes()[child].anchor,
                            "cleared its operand",
                        ));
                    }
                    Some(Answer::Unit(atom)) => {
                        rendered = Encoding::render(atom).to_string();
                        &rendered
                    }
                    None => {
                        return Err(nested(
                            self.schedule.lookup.nodes()[child].anchor,
                            "returned nothing",
                        ));
                    }
                },
            };
            if may_be_empty && spelling == lang::BLANK_PAIR {
                empty = true;
                continue;
            }
            let atom = token.decode(spelling).map_err(|error| error.to_string())?;
            if !empty {
                operands.push(atom);
            }
        }
        Ok(if empty {
            Decoded::Empty
        } else {
            Decoded::Operands(operands)
        })
    }

    /// The Cell after the last one `index`'s operands occupy. A returning last
    /// operand ends where its child's own operands end; any other ends with
    /// the Cells it consumes.
    pub(super) fn operands_end(&self, index: ComputationId) -> usize {
        let node = &self.schedule.lookup.nodes()[index];
        match node.operands.last() {
            Some(operand) => match self.returning_child(operand) {
                Some(child) => self.operands_end(child),
                None => operand.cells.end,
            },
            None => self.grid.index(node.anchor).get() + SCALAR_WIDTH,
        }
    }
}

/// The refusal a parent gives when the nested computation at `anchor` supplies
/// no value for its operand, naming that child.
fn nested(anchor: Position, outcome: &str) -> String {
    format!(
        "nested computation at column {}, row {} {outcome}",
        anchor.x(),
        anchor.y()
    )
}
