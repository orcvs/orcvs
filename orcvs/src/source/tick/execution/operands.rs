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

use lang::{Admission, Atom, Function};

use super::{
    Answer, Computation, ComputationId, Encoding, Execution, Operand, Position, SCALAR_WIDTH,
};

/// What a Turn's operands decode to.
pub(super) enum Decoded {
    /// The operands, in signature order.
    Operands(Vec<Atom>),
    /// The Function sends nothing on what its slots hold: the Turn is taken
    /// with nothing to send.
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
    /// [`Decoded::Empty`] where the Function has nothing to send.
    ///
    /// Each operand is read by its declared Token, whichever way its
    /// characters arrived. Spatial delivery leaves them pending in working
    /// Source until consumption, and a returning child supplies its answer's
    /// two-Cell encoding. The child's Atom type does not cross: a Note
    /// returned into a Number operand is read as the Number it spells,
    /// exactly as the same characters written there by a Portal would be.
    ///
    /// Two blank Cells are unwritten whichever way they arrived: written in
    /// Source, left by a Portal write, or written by a child that answered a
    /// Clear, as a pulse that did not Bang or a Copy, Read or Track of empty
    /// Cells does. The Function decides what its slots leave it to do, as
    /// [`lang::Function::admit`] answers, which is the question the Language
    /// Map asked of the Source revision. A refusal of a Clear names the child
    /// that cleared, since nothing at the operand says which Function did.
    pub(super) fn decode(
        &self,
        node: &Computation,
        function: Function,
        signature: lang::Tokens,
    ) -> Result<Decoded, String> {
        let mut read = Vec::with_capacity(node.operands.len());
        for (operand, token) in node.operands.iter().zip(signature) {
            let rendered;
            let (spelling, cleared_by) = match self.returning_child(operand) {
                None => (self.working.text(operand.cells.clone()), None),
                Some(child) => {
                    let anchor = self.schedule.lookup.nodes()[child].anchor;
                    match self.turns.result(child) {
                        // A Clear wrote two blank Cells at the child's Output
                        // Portal, so the operand holds what a blank one holds.
                        Some(Answer::Clear) => {
                            rendered = Encoding::clear().to_string();
                            (rendered.as_str(), Some(anchor))
                        }
                        Some(Answer::Unit(atom)) => {
                            rendered = Encoding::render(atom).to_string();
                            (rendered.as_str(), None)
                        }
                        None => return Err(nested(anchor, "returned nothing")),
                    }
                }
            };
            read.push((token, token.read(spelling), cleared_by));
        }
        let slots = read.iter().map(|(_, value, _)| match value {
            Ok(Some(_)) => lang::Slot::Written,
            Ok(None) => lang::Slot::Unwritten,
            Err(_) => lang::Slot::Malformed,
        });
        match function.admit(slots) {
            Admission::Runs => Ok(Decoded::Operands(
                read.into_iter()
                    .filter_map(|(_, value, _)| value.ok().flatten())
                    .collect(),
            )),
            Admission::SendsNothing => Ok(Decoded::Empty),
            Admission::Malformed { slot } => Err(match &read[slot] {
                (_, Err(error), _) => error.to_string(),
                _ => unreachable!("a malformed slot is one whose Cells failed to read"),
            }),
            Admission::Unwritten { slot } => Err(match read[slot] {
                (_, _, Some(anchor)) => nested(anchor, "cleared its operand"),
                (token, _, None) => token.unwritten_refusal().to_string(),
            }),
        }
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
