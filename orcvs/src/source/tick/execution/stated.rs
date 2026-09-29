//! One Tick whose named producers answer a value a test states rather than one
//! the Interpreter computes.
//!
//! A Jump can copy a Function value from its input Portal. These tests instead
//! choose a producer's answer independently of its operands and Portal input,
//! including answers its declared Function cannot produce, to isolate delivery
//! and replacement rules. A bare Cell also needs a stated answer: no Function
//! returns one, and it does not encode as the Cell pair a scalar result reserves.
//! These values are constructed here, one call below [`super::execute`], so that the
//! shipped Turn stays one thing in every build: the Interpreter's answer,
//! delivered.
//!
//! A reservation is stated the same way and for the same reason: scheduling
//! derives one from what a Function declares its answer to be, and a stated
//! answer need not be what its producer's Function declares, so the width a
//! stated Sequence answer reserves is stated beside the answer rather than
//! derived from it. Two consequences of stating it are worth knowing, and both
//! are refused loudly rather than discovered:
//!
//! - A stated width is not a declared one, so `Lookup::would_reserve` — which
//!   re-derives a width for a hypothetical replacement Function — cannot agree
//!   with it at the computation whose width was stated. Stating a reservation
//!   and stating a Function replacement in one Tick is therefore refused here.
//! - A width the fixture states is the input to every width production derives
//!   around it, so `derive_reservations` widens each ancestor over the stated
//!   ones by the rule the Language Map's derivation applies. Nothing about
//!   that derivation is proven by these tests: the Language Map's own tests
//!   and the declared Range rows are.
//!
//! Only the Turn loop is reimplemented, because substituting one Turn is the
//! one thing this does differently. It records each Turn's ordinal exactly as
//! the production loop does, and a rejected Tick through this loop is held to
//! that by a test of its own, so the two cannot drift apart unnoticed. The
//! starting state, the Bang cleanup it performs, the schedule, the rejection
//! path, the resolution, and the Turn every other computation takes are all
//! the production ones, reached through the same [`super::Execution::new`] that
//! [`super::execute`] reaches them through.
//!

use lang::{Tick, Value};

use super::super::{Reserved, carry, computations, derive_reservations, order_turns, unscheduled};
use super::{
    Atom, Break, ComputationState, Continue, ControlFlow, Diagnostic, Execution, Grid, LanguageMap,
    Lookup, Position, Schedule, TickPlan, resolve,
};
use crate::grid::CellIndex;
use crate::source::Cells;
use std::collections::BTreeMap;

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
    reservations: &[(CellIndex, Reserved)],
    answers: &[(CellIndex, Value)],
) -> (TickPlan, Vec<ComputationState>) {
    let (mut nodes, diagnostics) = computations(grid, map);
    carry(grid, &mut nodes, destinations);
    let mut lookup = Lookup::new(grid, nodes, map);
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
    for (index, (anchor, reserved)) in reservations.iter().enumerate() {
        assert!(
            !reservations[..index]
                .iter()
                .any(|(stated, _)| stated == anchor),
            "one computation is stated one reservation: two are stated here for the same anchor"
        );
        assert!(
            *reserved == Reserved::Row,
            "a stated reservation is a width production would not derive: \
             `derive_reservations` re-derives every computation still holding \
             `Reserved::Pair`, so stating one says nothing and is overwritten \
             by the pass that reads it"
        );
    }
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
    // the width the fixture is replacing. It also follows `Lookup::new`,
    // whose debug check refuses a width no declaration derives, which a
    // stated `Reserved::Row` is.
    for (anchor, reserved) in reservations {
        let index = anchored(&lookup, grid, *anchor)
            .expect("a stated reservation names a computation the schedule contains");
        lookup.nodes[index].reserved = *reserved;
    }
    // What a stated reservation leaves for production to derive: an
    // ancestor that widens over a row-reserving operand widens over a
    // stated one exactly as it does over a declared Range, because this
    // applies the Language Map's rule over the widths now settled.
    // Without it a stated child would leave its pervasive ancestor holding
    // the `Reserved::Pair` derived before the fixture spoke, and the
    // ancestor's own wide answer would be refused for a width the schedule
    // never reserved.
    derive_reservations(&mut lookup.nodes);
    let Schedule {
        lookup,
        order,
        diagnostics,
    } = match order_turns(lookup, diagnostics) {
        Ok(schedule) => schedule,
        Err(diagnostics) => return unscheduled(diagnostics),
    };
    let mut execution = Execution::new(grid, Cells::of(bytes), map, tick, &lookup, diagnostics);
    let mut stated = vec![false; answers.len()];
    for (turn, index) in order.into_iter().enumerate() {
        let anchor = grid.index(lookup.nodes()[index].anchor);
        // The ordinal production records, recorded here for the reason the
        // loop around it is reproduced: a stated answer replaces what one
        // computation answers and nothing else, and the Turn it took is
        // the Turn it would have taken.
        execution.states[index].turn = Some(turn);
        let outcome = match answers.iter().position(|(stated, _)| *stated == anchor) {
            Some(position) => {
                stated[position] = true;
                execution.state_answer(index, answers[position].1.clone())
            }
            None => execution.take_turn(index),
        };
        if let Break(diagnostic) = outcome {
            // The order stops where a rejection found it, so the answers
            // after that Turn are unstated for a reason the fixture chose.
            return execution.reject(diagnostic);
        }
    }
    // Reached only where an order existed and ran to its end. The two
    // returns above skip it for reasons the fixture can see in the plan it
    // gets back: a rejection stops the order where it found the defect, and
    // a cycle admits no order at all and publishes diagnostics and nothing
    // else. Neither can be mistaken for a stated answer whose computation
    // the order never reached, which is the one thing this guard is for.
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
/// `None` for, and the callers turn into a panic naming which half — the
/// answer or the reservation — named it.
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
