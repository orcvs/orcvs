//! What each computation's Turn has done so far this Tick.
//!
//! A computation's facts are independent: an attempted Turn can be
//! syntax-blocked, and a successful answer, returned to a parent, can coexist
//! with a rejected spatial delivery. Keeping them together does not turn them
//! into an exclusive lifecycle enum. Only this module changes them, so each
//! rule about how they change together holds in one place: a Bang that
//! reaches a root which has taken its Turn without it misses that root, and
//! suppression never reaches a computation that has attempted its Turn.

use lang::{Atom, Function};

use super::super::dependencies::TurnFacts;
use super::super::{ComputationId, ComputationRange, Lookup, PerComputation};

/// What a value Function's Turn answered, which is what it leaves at its
/// Output Portal and returns when nested.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::source) enum Answer {
    /// The Language Unit it writes.
    Unit(Atom),
    /// Two spaces, which write no Language Unit.
    Clear,
}

impl Answer {
    /// The Language Unit this answer writes; a clear writes none.
    pub(in crate::source) fn unit(self) -> Option<Atom> {
        match self {
            Self::Unit(atom) => Some(atom),
            Self::Clear => None,
        }
    }
}

/// What one computation's Turn has done so far this Tick.
pub(in crate::source) struct ComputationState {
    /// The Function this computation is running: the one the Parser found,
    /// until a Function Replacement in this same Tick changes it.
    function: Function,
    result: Option<Answer>,
    syntax_blocked: bool,
    activated: bool,
    suppressed: bool,
    /// Whether this computation's Turn opened: it was active and not
    /// suppressed, whatever its operands then made of it.
    attempted: bool,
    /// Whether this computation's Turn has been taken, or settled without
    /// effect, so that it takes none again this Tick.
    taken: bool,
    /// Which Turn this computation took, counted from zero, or `None` where
    /// its order holds no Turn for it.
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
    /// Which Turn this computation took, or `None` where its order holds no
    /// Turn for it.
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

/// What a Bang delivered to a root.
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Activation {
    /// The root is active for the rest of this Tick.
    Delivered,
    /// The root has taken its Turn without the activation it needed, which no
    /// later Tick can deliver. Nothing changed.
    Missed,
}

/// Every computation's [`ComputationState`], indexed as the Lookup indexes
/// its computations.
pub(super) struct Turns {
    states: PerComputation<ComputationState>,
    /// How many Turns have been taken, which is the next Turn's ordinal.
    #[cfg(test)]
    taken: usize,
}

impl Turns {
    /// Every computation untouched, running the Function the Parser found.
    pub(super) fn new(lookup: &Lookup) -> Self {
        Self {
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
                    taken: false,
                    #[cfg(test)]
                    turn: None,
                    #[cfg(test)]
                    interpreted: None,
                    #[cfg(test)]
                    interpretations: 0,
                })
                .collect(),
            #[cfg(test)]
            taken: 0,
        }
    }

    /// What each computation's Turn did, once the Tick is over.
    pub(super) fn into_states(self) -> Vec<ComputationState> {
        self.states.into_vec()
    }

    /// The Function `index` is running.
    pub(super) fn function(&self, index: ComputationId) -> Function {
        self.states[index].function
    }

    /// The answer `index`'s Turn returned, or `None` where it returned none.
    pub(super) fn result(&self, index: ComputationId) -> Option<Answer> {
        self.states[index].result
    }

    /// Whether `index`'s Turn has opened, which no later writer may reach.
    pub(super) fn attempted(&self, index: ComputationId) -> bool {
        self.states[index].attempted
    }

    /// Whether a write covered `index` before its Turn.
    pub(super) fn suppressed(&self, index: ComputationId) -> bool {
        self.states[index].suppressed
    }

    /// Whether `index`'s Turn was blocked by a syntax error the Source
    /// revision already reports.
    pub(super) fn syntax_blocked(&self, index: ComputationId) -> bool {
        self.states[index].syntax_blocked
    }

    ///
    /// Whether `index`, owned by the root `owner`, may open its Turn: it was
    /// not suppressed, and its owner is active.
    ///
    /// Activation is the owner's to declare, and the declaration that decides
    /// it is the one the owner is running. It is independent of whether this
    /// computation will produce a typed answer — a Self-Banging Function
    /// answers none and takes its Turn anyway, which is why this asks the
    /// activation source rather than `answers_value`.
    ///
    pub(super) fn may_open(&self, index: ComputationId, owner: ComputationId) -> bool {
        let owner = &self.states[owner];
        !self.states[index].suppressed
            && (owner.function.is_intrinsically_active() || owner.activated)
    }

    /// Records that `index`'s Turn opened. Opening precedes syntax and
    /// evaluation checks, so a later writer must not reach a computation even
    /// when its attempted Turn failed.
    pub(super) fn open(&mut self, index: ComputationId) {
        self.states[index].attempted = true;
    }

    /// Records that `index`'s Turn was blocked by a syntax error.
    pub(super) fn block_syntax(&mut self, index: ComputationId) {
        self.states[index].syntax_blocked = true;
    }

    /// Records that `index` has taken its Turn this Tick, settled with or
    /// without effect.
    pub(super) fn settle(&mut self, index: ComputationId) {
        self.states[index].taken = true;
        // The ordinal counts Turns as they are taken, which is not a position
        // in the schedule's order once a Turn waits.
        #[cfg(test)]
        {
            self.states[index].turn = Some(self.taken);
            self.taken += 1;
        }
    }

    /// Records the answer `index`'s Turn returned. A successful answer
    /// survives every refusal to project it.
    pub(super) fn answer(&mut self, index: ComputationId, answer: Answer) {
        self.states[index].result = Some(answer);
    }

    /// Records that `index` now runs `function`.
    pub(super) fn replace(&mut self, index: ComputationId, function: Function) {
        self.states[index].function = function;
    }

    ///
    /// Delivers a Bang's activation to `root`.
    ///
    /// Activation lasts only for the Tick that produces it, so a root that has
    /// taken its Turn without it misses it. A root that is intrinsically
    /// active, or already activated, needs nothing a Bang could arrive too
    /// late for.
    ///
    pub(super) fn activate(&mut self, root: ComputationId) -> Activation {
        let state = &mut self.states[root];
        if state.taken && !state.activated && !state.function.is_intrinsically_active() {
            return Activation::Missed;
        }
        state.activated = true;
        Activation::Delivered
    }

    /// Suppresses every computation in `subtree` that has not attempted its
    /// Turn: its spelling is no longer the one that was scheduled. What has
    /// taken its Turn keeps it.
    pub(super) fn suppress(&mut self, subtree: ComputationRange) {
        for state in &mut self.states[subtree] {
            if !state.attempted {
                state.suppressed = true;
            }
        }
    }

    /// Records the inputs the Interpreter is handed for `index`.
    #[cfg(test)]
    pub(super) fn interpret(&mut self, index: ComputationId, tick: lang::TickInputs) {
        self.states[index].interpreted = Some(tick);
        self.states[index].interpretations += 1;
    }
}

impl TurnFacts for Turns {
    fn attempted(&self, index: ComputationId) -> bool {
        Turns::attempted(self, index)
    }

    fn suppressed(&self, index: ComputationId) -> bool {
        Turns::suppressed(self, index)
    }

    fn running(&self, index: ComputationId) -> Function {
        self.function(index)
    }
}
