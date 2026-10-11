//! One Tick whose named producers answer a value a test states rather than one
//! the Interpreter computes.
//!
//! A Copy can copy a Function value from its input Portal. These tests instead
//! choose a producer's answer independently of its operands and Portal input,
//! including answers its declared Function cannot produce, to isolate delivery
//! and replacement rules. These values are constructed here, one call below
//! [`super::execute`], so that the shipped Turn stays one thing in every build:
//! the Interpreter's answer, delivered.
//!
//! Substituting one Turn is the one thing this does differently, and it
//! records that Turn's ordinal as a production Turn records its own. The
//! starting state, the Bang cleanup it performs, the schedule, the order its
//! Turns are taken in, the resolution, and the Turn every other computation
//! takes are all the production ones, reached through the same
//! [`super::Execution::new`] and [`ordering::take_turns`] that
//! [`super::execute`] reaches them through.
//!

use lang::Tick;

use super::super::{Lookup, carry, computations, order_turns};
use super::{
    Atom, ComputationState, Execution, Grid, LanguageMap, Position, TickPlan, Turn, ordering,
    resolve,
};
use crate::grid::CellIndex;
use crate::source::Cells;
use std::collections::{BTreeMap, BTreeSet};

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
    answers: &[(CellIndex, Atom)],
) -> (TickPlan, Vec<ComputationState>) {
    let (mut nodes, diagnostics) = computations(grid, map);
    carry(grid, &mut nodes, destinations);
    let lookup = Lookup::new(grid, nodes, map);
    // Every fixture error the schedule can be asked about is asked here,
    // before an order exists. A cycle leaves the computations it stops out
    // of the order, which is indistinguishable from the little a mistyped
    // fixture makes happen — so a guard that ran after ordering would be the
    // one guard a cyclic fixture switches off.
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
    let schedule = order_turns(lookup, diagnostics);
    let (mut execution, fired) = Execution::new(
        grid,
        Cells::of(bytes),
        map,
        &BTreeSet::new(),
        tick,
        &schedule,
    );
    let mut stated = vec![false; answers.len()];
    ordering::take_turns(&schedule, fired, |index, progress| {
        let anchor = grid.index(schedule.lookup.nodes()[index].anchor);
        match answers.iter().position(|(stated, _)| *stated == anchor) {
            Some(position) => {
                stated[position] = true;
                // Counted as production counts a Turn it takes: a stated
                // answer replaces what one computation answers and nothing
                // else, and the Turn it took is the Turn it would have taken.
                execution.settle(index);
                execution.state_answer(index, answers[position].1);
                Turn::Taken {
                    activated: Vec::new(),
                }
            }
            None => {
                let turn = execution.take_turn(index, progress);
                assert!(
                    matches!(turn, Turn::Taken { .. }),
                    "a stated fixture holds no Turn that waits on a writer"
                );
                turn
            }
        }
    });
    // Every order runs to its end, so an answer whose computation the order
    // never reached was stated for a computation a cycle stops, and the
    // fixture is told so here rather than handed a quiet Tick.
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
/// `None` for, and the caller turns into a panic naming it.
///
fn anchored(lookup: &Lookup, grid: Grid, anchor: CellIndex) -> Option<usize> {
    lookup
        .nodes()
        .iter()
        .position(|node| grid.index(node.anchor) == anchor)
}

impl Execution<'_> {
    ///
    /// Delivers `atom` as this computation's answer, in place of the
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
    fn state_answer(&mut self, index: usize, atom: Atom) {
        if self.opens_turn(index).is_none() {
            return;
        }
        // The one arm of the Turn that decides an answer's kind rather than
        // its content, and the only one left standing here: a Function
        // answering an effect answers a Play Command, and the root is the
        // one place `opens_turn` lets such a Function stand — it diagnoses
        // every other. Delivering a value there would plan a write from a
        // computation production can only ever hear a Play Command from.
        assert!(
            self.states[index].function.answers_value(),
            "a stated answer belongs to a computation that answers a value"
        );
        self.deliver_value(index, atom, None, super::Delivery::Reserved);
    }
}
