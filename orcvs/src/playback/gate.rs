use std::sync::atomic::{AtomicUsize, Ordering};

/// Set while an admitted Tick has not finished.
const EXECUTING: usize = 1;
/// One outstanding stop request. The count occupies every bit above
/// [`EXECUTING`], so a word of zero is the one state that admits a Tick.
const STOP: usize = 2;

///
/// The one piece of state a synchronous `stop` and the Tick loop share.
///
/// ADR 0002 requires that further Ticks are prevented before `stop` returns,
/// and a message cannot do that: `send` returns once the message is queued and
/// the task may be about to execute a Tick. ADR 0041 admits this state
/// deliberately. It carries one fact — someone has asked me to stop — and
/// nothing reads it to decide which state the engine is in.
///
/// Admission is one atomic step, so there is an instant before which a Tick
/// is this run's business and after which it is refused, and `stop` lands on
/// one side of it or the other. Do not split admission into a read and an act
/// on it: a request landing between them is overtaken by the Tick the read
/// admitted.
///
/// A Tick already admitted still runs to completion. `stop` does not wait for
/// it — the browser main thread has nothing to wait with — so the guarantee is
/// that no *further* Tick is admitted, not that the engine is silent by the
/// time `stop` returns.
///
/// Outstanding requests are counted, not collapsed into a single bit: two
/// cloned handles may each raise a stop before either message is applied, and
/// answering the first must not reopen admission for a Tick that overtakes the
/// second. The count and whether a Tick is executing share one word, and the
/// gate is open exactly when that word is zero. Keep them in one word: held
/// apart, answering a request becomes a decrement and a separate reopen, and a
/// request landing between the two is left standing behind an open gate.
///
/// The count does not overflow while anything reads it. A request the task
/// can still answer has a queued `Stop` message behind it, and no target has
/// the memory for `usize::MAX / STOP` of them.
///
#[derive(Debug)]
pub(super) struct TickGate {
    word: AtomicUsize,
}

impl TickGate {
    pub(super) fn new() -> Self {
        Self {
            word: AtomicUsize::new(0),
        }
    }

    ///
    /// Records that a stop has been asked for, closing the gate behind
    /// whatever is executing now.
    ///
    /// Unconditional, because a request is not a negotiation: a Tick in flight
    /// stays in flight and the gate is shut behind it, so the next admission
    /// is refused whichever of the two got here first.
    ///
    pub(super) fn request_stop(&self) {
        self.word.fetch_add(STOP, Ordering::AcqRel);
    }

    ///
    /// Admits a Tick, or refuses it because a stop is standing.
    ///
    /// The caller must pair a `true` with [`finish_tick`](Self::finish_tick).
    ///
    pub(super) fn begin_tick(&self) -> bool {
        self.word
            .compare_exchange(0, EXECUTING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    ///
    /// Records that the admitted Tick has finished.
    ///
    /// Clears the executing bit and nothing else, so a stop that arrived while
    /// the Tick was executing survives it and refuses the *next* Tick.
    ///
    pub(super) fn finish_tick(&self) {
        self.word.fetch_and(!EXECUTING, Ordering::AcqRel);
    }

    ///
    /// Answers one standing stop request, reopening the gate only when none
    /// remain.
    ///
    /// The task calls this as it applies the `Stop` message, which is the only
    /// thing that can answer a request. Conditional on a request actually
    /// standing: a `Stop` applied when none stands has nothing to clear, and
    /// answering regardless would let a message answer a request raised after
    /// it was sent.
    ///
    /// Taking one request off the count is the reopening when it was the last
    /// one; there is no second step for another request to land before.
    ///
    pub(super) fn clear_stop(&self) {
        let _ = self
            .word
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, answered);
    }
}

///
/// The word after one standing request is answered, or `None` when none
/// stands.
///
fn answered(word: usize) -> Option<usize> {
    word.checked_sub(STOP)
}

#[cfg(test)]
mod tests {
    use super::{TickGate, answered};
    use std::sync::atomic::Ordering;

    #[test]
    fn an_untouched_gate_admits_a_tick() {
        let gate = TickGate::new();

        assert!(gate.begin_tick());
    }

    #[test]
    fn a_stop_raised_before_a_tick_refuses_it() {
        let gate = TickGate::new();

        gate.request_stop();

        assert!(!gate.begin_tick());
    }

    ///
    /// The window ADR 0002's guarantee is about: the request lands while a
    /// Tick is executing, and the Tick that follows it is refused.
    ///
    #[test]
    fn a_stop_raised_while_a_tick_executes_refuses_the_next_tick() {
        let gate = TickGate::new();

        assert!(gate.begin_tick());
        gate.request_stop();
        gate.finish_tick();

        assert!(!gate.begin_tick());
    }

    #[test]
    fn answering_the_request_admits_ticks_again() {
        let gate = TickGate::new();

        gate.request_stop();
        gate.clear_stop();

        assert!(gate.begin_tick());
    }

    ///
    /// Finishing a Tick nobody stopped leaves the gate open, so an ordinary
    /// run keeps ticking.
    ///
    #[test]
    fn finishing_an_unstopped_tick_leaves_the_gate_open() {
        let gate = TickGate::new();

        assert!(gate.begin_tick());
        gate.finish_tick();

        assert!(gate.begin_tick());
    }

    ///
    /// A stop raised after a Tick was admitted is not answered by that Tick
    /// ending, so the run does not resume behind the request's back.
    ///
    #[test]
    fn a_tick_ending_does_not_answer_a_stop_request() {
        let gate = TickGate::new();

        assert!(gate.begin_tick());
        gate.request_stop();
        gate.finish_tick();
        // Only the `Stop` message answers it.
        gate.clear_stop();

        assert!(gate.begin_tick());
    }

    ///
    /// Two handles can each raise a stop before either message is applied.
    /// Answering the first must not reopen admission for a Tick that overtakes
    /// the second — that Tick would run after the second `stop` had already
    /// returned, which ADR 0002 / 0040 forbid.
    ///
    #[test]
    fn two_outstanding_stops_keep_the_gate_shut_until_both_are_answered() {
        let gate = TickGate::new();

        gate.request_stop();
        gate.request_stop();
        gate.clear_stop();

        assert!(
            !gate.begin_tick(),
            "the first Stop cleared a second handle's still-standing request"
        );

        gate.clear_stop();
        assert!(gate.begin_tick());
    }

    ///
    /// The answer to the last standing request reads the word, and a second
    /// handle's request lands before the answer commits. An answer computed
    /// from a word the request has since moved cannot commit an open gate over
    /// the new request, and `clear_stop` commits only through that
    /// compare-and-swap, so it retries against the word the request left and
    /// one request stays standing.
    ///
    /// The read and the commit are driven by hand, one at a time, so the
    /// request lands between them; `clear_stop` itself then answers against
    /// the moved word.
    ///
    #[test]
    fn a_stop_requested_while_the_last_one_is_answered_keeps_the_gate_shut() {
        let gate = TickGate::new();
        gate.request_stop();

        let seen = gate.word.load(Ordering::Acquire);
        let reopened = answered(seen).expect("one request stands");
        assert_eq!(reopened, 0, "answering the only request reopens the gate");

        gate.request_stop();

        assert!(
            gate.word
                .compare_exchange(seen, reopened, Ordering::AcqRel, Ordering::Acquire)
                .is_err(),
            "the answer committed a word the second request had moved"
        );
        gate.clear_stop();

        assert!(
            !gate.begin_tick(),
            "the gate opened with the second handle's request still standing"
        );

        gate.clear_stop();
        assert!(gate.begin_tick());
    }

    ///
    /// Two stops requested while a Tick executes both outlast it, and the gate
    /// reopens only once both are answered and the Tick has finished.
    ///
    #[test]
    fn stops_raised_during_a_tick_outlast_it_until_each_is_answered() {
        let gate = TickGate::new();

        assert!(gate.begin_tick());
        gate.request_stop();
        gate.request_stop();
        gate.finish_tick();

        assert!(!gate.begin_tick(), "both requests still stand");
        gate.clear_stop();
        assert!(!gate.begin_tick(), "the second request still stands");

        gate.clear_stop();
        assert!(gate.begin_tick());
    }

    #[test]
    fn answering_with_no_request_standing_changes_nothing() {
        let gate = TickGate::new();

        gate.clear_stop();
        gate.request_stop();

        assert!(
            !gate.begin_tick(),
            "a stray answer was banked against a later request"
        );
    }
}
