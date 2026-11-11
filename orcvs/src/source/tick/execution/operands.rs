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
    Answer, Computation, ComputationId, Encoding, Execution, Function, Operand, Position,
    SCALAR_WIDTH,
};

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

    /// The operand slots of `node`'s Turn as `function`, in signature order,
    /// once binding has checked them.
    ///
    /// Each operand is decoded by its declared Token, whichever way its
    /// characters arrived. Spatial delivery leaves them pending in working
    /// Source until consumption, and a returning child supplies its answer's
    /// two-Cell encoding. The child's Atom type does not cross: a Note
    /// returned into a Number operand is read as the Number it spells,
    /// exactly as the same characters written there by a Portal would be.
    ///
    /// A slot holds no value, `None`, where its Cells are blank or its child
    /// answered a Clear: a pulse that did not Bang, or a Copy, Read or Track
    /// of empty Cells. It is passed on as it is, and `function`'s operand
    /// checks decide what it means before any Portal is resolved or read, so
    /// a Turn whose operands do not bind reads nothing.
    ///
    /// The refusal is for the first slot in signature order that does not
    /// decode, that a child returned nothing to, or that the checks refuse.
    pub(super) fn decode(
        &self,
        node: &Computation,
        function: Function,
        signature: lang::Tokens,
    ) -> Result<Vec<Option<Atom>>, String> {
        let mut operands = Vec::with_capacity(node.operands.len());
        for (operand, token) in node.operands.iter().zip(signature) {
            let Some(child) = self.returning_child(operand) else {
                let spelling = self.working.text(operand.cells.clone());
                if is_blank(spelling, token) {
                    operands.push(None);
                    continue;
                }
                match token.decode(spelling) {
                    Ok(atom) => operands.push(Some(atom)),
                    Err(error) => {
                        return Err(self.undecoded(node, function, operands, error.to_string()));
                    }
                }
                continue;
            };
            let anchor = self.schedule.lookup.nodes()[child].anchor;
            let returned = match self.turns.result(child) {
                // A Clear wrote two spaces at the child's Output Portal and
                // returns no value.
                Some(Answer::Clear) => {
                    operands.push(None);
                    continue;
                }
                Some(Answer::Unit(atom)) => Encoding::render(atom),
                None => {
                    let message = nested(anchor, "returned nothing");
                    return Err(self.undecoded(node, function, operands, message));
                }
            };
            match token.decode(&returned.to_string()) {
                Ok(atom) => operands.push(Some(atom)),
                Err(error) => {
                    return Err(self.undecoded(node, function, operands, error.to_string()));
                }
            }
        }
        function
            .check_operands(&operands)
            .map_err(|error| self.refusal(node, &error))?;
        Ok(operands)
    }

    ///
    /// The refusal of `node`'s Turn as `function` when the slot after
    /// `decoded` supplies no value it can decode, which `message` states.
    ///
    /// The checks refuse slots in signature order, so a slot before it that
    /// they refuse is the one named, as it would be if every slot had
    /// decoded.
    ///
    fn undecoded(
        &self,
        node: &Computation,
        function: Function,
        mut decoded: Vec<Option<Atom>>,
        message: String,
    ) -> String {
        let slot = decoded.len();
        decoded.resize(node.operands.len(), None);
        match function.check_operands(&decoded) {
            Err(error)
                if matches!(
                    error,
                    lang::Error::Argument(lang::ArgumentError::Absent { slot: earlier, .. })
                        if earlier < slot
                ) =>
            {
                self.refusal(node, &error)
            }
            _ => message,
        }
    }

    ///
    /// The diagnostic for `error`, which refused `node`'s Turn.
    ///
    /// A slot whose child answered a Clear holds no value, and nothing at the
    /// operand says which Function cleared it, so a refusal of that slot
    /// names the child. Every other refusal is `lang`'s own.
    ///
    pub(super) fn refusal(&self, node: &Computation, error: &lang::Error) -> String {
        if let lang::Error::Argument(lang::ArgumentError::Absent { slot, .. }) = error
            && let Some(child) = node
                .operands
                .get(*slot)
                .and_then(|operand| self.returning_child(operand))
            && matches!(self.turns.result(child), Some(Answer::Clear))
        {
            return nested(
                self.schedule.lookup.nodes()[child].anchor,
                "cleared its operand",
            );
        }
        error.to_string()
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

/// Whether `spelling` is the blank Cells of a whole `token` slot: one that
/// holds no value. A slot cut short at the row edge is not blank; its token
/// refuses it.
fn is_blank(spelling: &str, token: lang::Token) -> bool {
    spelling.len() == token.len() && spelling.bytes().all(|cell| cell == b' ')
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
