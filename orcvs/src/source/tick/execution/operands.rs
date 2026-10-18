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
    /// A Clear reached the Function's rest operand and every other operand
    /// decoded: the Turn is taken with nothing to perform.
    Rest,
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
    /// [`Decoded::Rest`] where the Turn is a rest.
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
    /// The one exception is `function`'s rest operand: a Clear there makes
    /// the Turn a rest once every other operand decodes, so a Play with no
    /// note to play sends nothing and reports nothing.
    pub(super) fn decode(
        &self,
        node: &Computation,
        function: Function,
        signature: lang::Tokens,
    ) -> Result<Decoded, String> {
        let mut rests = false;
        let mut operands = Vec::with_capacity(node.operands.len());
        for (slot, (operand, token)) in node.operands.iter().zip(signature).enumerate() {
            let Some(child) = self.returning_child(operand) else {
                let spelling = self.working.text(operand.cells.clone());
                operands.push(token.decode(spelling).map_err(|error| error.to_string())?);
                continue;
            };
            let anchor = self.schedule.lookup.nodes()[child].anchor;
            let returned = match self.turns.result(child) {
                Some(Answer::Clear) if function.rest_operand() == Some(slot) => {
                    rests = true;
                    continue;
                }
                // A Clear wrote two spaces at the child's Output Portal and
                // returns no value. The parent names the child, since
                // nothing at the operand says which Function cleared.
                Some(Answer::Clear) => return Err(nested(anchor, "cleared its operand")),
                Some(Answer::Unit(atom)) => Encoding::render(atom),
                None => return Err(nested(anchor, "returned nothing")),
            };
            operands.push(
                token
                    .decode(&returned.to_string())
                    .map_err(|error| error.to_string())?,
            );
        }
        Ok(if rests {
            Decoded::Rest
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
