use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

/// No stop is standing and no Tick is executing.
const OPEN: u8 = 0;
/// A Tick has been admitted and has not finished yet.
const EXECUTING: u8 = 1;
/// Someone has asked the engine to stop, and no Tick may be admitted until the
/// message behind that request has been applied.
const STOPPING: u8 = 2;

///
/// The one piece of state a synchronous `stop` and the Tick loop share.
///
/// ADR 0002 requires that further Ticks are prevented before `stop` returns,
/// and a message cannot do that: `send` returns once the message is queued and
/// the task may be about to execute a Tick. ADR 0041 admits this state
/// deliberately. It carries one fact — someone has asked me to stop — and
/// nothing reads it to decide which state the engine is in.
///
/// What it adds over the flag it replaces is a moment. A flag can be read and
/// then acted on, and those are two steps: the read says "no stop standing",
/// the request arrives, and the Tick the read admitted executes anyway with
/// nothing recording that it had begun. Admission here is one atomic step, so
/// there is an instant before which a Tick is this run's business and after
/// which it is refused, and `stop` lands on one side of it or the other.
///
/// A Tick already admitted still runs to completion. `stop` does not wait for
/// it — the browser main thread has nothing to wait with — so the guarantee is
/// that no *further* Tick is admitted, not that the engine is silent by the
/// time `stop` returns.
///
/// Outstanding requests are counted, not collapsed into a single bit: two
/// cloned handles may each raise a stop before either message is applied, and
/// answering the first must not reopen admission for a Tick that overtakes the
/// second.
///
#[derive(Debug)]
pub(super) struct TickGate {
    state: AtomicU8,
    outstanding_stops: AtomicUsize,
}

impl TickGate {
    pub(super) fn new() -> Self {
        Self {
            state: AtomicU8::new(OPEN),
            outstanding_stops: AtomicUsize::new(0),
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
        self.outstanding_stops.fetch_add(1, Ordering::AcqRel);
        self.state.store(STOPPING, Ordering::Release);
    }

    ///
    /// Admits a Tick, or refuses it because a stop is standing.
    ///
    /// The caller must pair a `true` with [`finish_tick`](Self::finish_tick).
    ///
    pub(super) fn begin_tick(&self) -> bool {
        self.state
            .compare_exchange(OPEN, EXECUTING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    ///
    /// Records that the admitted Tick has finished.
    ///
    /// Reopening is conditional on the gate still being the one this Tick shut,
    /// so a stop that arrived while the Tick was executing survives it. An
    /// unconditional reopen here would discard exactly the request the caller
    /// raised to prevent the *next* Tick.
    ///
    pub(super) fn finish_tick(&self) {
        let _ = self
            .state
            .compare_exchange(EXECUTING, OPEN, Ordering::AcqRel, Ordering::Acquire);
    }

    ///
    /// Answers one standing stop request, reopening the gate only when none
    /// remain.
    ///
    /// The task calls this as it applies the `Stop` message, which is the only
    /// thing that can answer a request. Conditional on a request actually
    /// standing: a `Stop` applied against an open gate has nothing to clear,
    /// and storing `OPEN` regardless would let a message answer a request
    /// raised after it was sent.
    ///
    pub(super) fn clear_stop(&self) {
        loop {
            let outstanding = self.outstanding_stops.load(Ordering::Acquire);
            if outstanding == 0 {
                return;
            }
            if self
                .outstanding_stops
                .compare_exchange(
                    outstanding,
                    outstanding - 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_err()
            {
                continue;
            }
            if outstanding == 1 {
                let _ = self.state.compare_exchange(
                    STOPPING,
                    OPEN,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TickGate;

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
}
