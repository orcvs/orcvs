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

use lang::Atom;

use super::{
    Computation, EMPTY_PAIR, Encoding, Execution, Operand, Rendered, SCALAR_WIDTH, render_message,
};

impl Execution<'_> {
    /// The nested Function whose Return `operand` consumes, or `None` where
    /// it consumes the Cells at `operand.cells`.
    fn returning_child(&self, operand: &Operand) -> Option<usize> {
        operand
            .child
            .filter(|&child| !self.states[child].suppressed)
    }

    ///
    /// Whether a nested Function returning to `node` was blocked by a syntax
    /// error the Source revision already reports, which blocks `node` in
    /// turn without repeating the report.
    ///
    pub(super) fn blocked_by_child(&self, node: &Computation) -> bool {
        node.operands.iter().any(|operand| {
            self.returning_child(operand)
                .is_some_and(|child| self.states[child].syntax_blocked)
        })
    }

    /// The operands of `node`'s Turn, in signature order.
    ///
    /// Each operand is decoded by its declared Token, whichever way its
    /// characters arrived. Spatial delivery leaves them pending in working
    /// Source until consumption, and a returning child supplies its answer's
    /// two-Cell encoding. The child's Atom type does not cross: a Note
    /// returned into a Number operand is read as the Number it spells,
    /// exactly as the same characters written there by a Portal would be.
    ///
    /// The first operand that does not decode refuses the Turn. A slot with
    /// no written Cell refuses it as a partly written slot does, and so does
    /// a child that copied empty Cells into it.
    pub(super) fn decode(
        &self,
        node: &Computation,
        signature: lang::Tokens,
    ) -> Result<Vec<Atom>, String> {
        node.operands
            .iter()
            .zip(signature)
            .map(|(operand, token)| {
                let Some(child) = self.returning_child(operand) else {
                    let spelling = self.working.text(operand.cells.clone());
                    return token.decode(spelling).map_err(|error| error.to_string());
                };
                let state = &self.states[child];
                let anchor = self.schedule.lookup.nodes()[child].anchor;
                let returned = match state.result.map(Encoding::render) {
                    Some(Ok(Rendered::Cells(encoding))) => encoding,
                    // A Function that copies Cells returns the empty Cells it
                    // copied, so the operand it stands in is unwritten.
                    Some(Ok(Rendered::Nothing)) if state.function.copies_language_unit() => {
                        return token.decode(EMPTY_PAIR).map_err(|error| error.to_string());
                    }
                    Some(Ok(Rendered::Nothing)) | None => {
                        return Err(format!(
                            "nested computation at column {}, row {} returned nothing",
                            anchor.x(),
                            anchor.y()
                        ));
                    }
                    // A rendering a Cell cannot hold is its own fault, not an
                    // absent answer.
                    Some(Err(reason)) => return Err(render_message(reason)),
                };
                token
                    .decode(&returned.to_string())
                    .map_err(|error| error.to_string())
            })
            .collect()
    }

    /// The Cell after the last one `index`'s operands occupy. A returning last
    /// operand ends where its child's own operands end; any other ends with
    /// the Cells it consumes.
    pub(super) fn operands_end(&self, index: usize) -> usize {
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
