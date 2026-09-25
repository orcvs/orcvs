//! The requests a Playback Engine's handles leave for its task, held in slots
//! of fixed size.
//!
//! Every request a handle can make is one where only the newest matters, or
//! where several collapse into one, so each has a slot and a later request
//! overwrites or merges with the one waiting there. What a handle can leave
//! pending is therefore bounded by the shape of [`Backlog`] and not by how
//! fast anyone calls:
//!
//! - one `stop` flag, standing for every `stop` made since the task last took
//!   the backlog, and at most two standing requests on the [`TickGate`];
//! - one period to begin a run at and one to retune a live run to;
//! - one destination change, which holds at most one connection.
//!
//! Nothing else is held: the slots are the whole backlog. The task wakes on a
//! [`Notify`], which stores at most one permit. A connection the task has
//! taken but not yet installed is outside the slots, so Playback holds at most
//! two connections that are not installed: that one, and one pending. A
//! connection a request replaces is held only by the caller that replaced it,
//! until that call returns.
//!
//! A caller never waits on the task or on a Tick. The lock over the slots is
//! held for a constant-time update on either side and never across an `await`
//! or a Tick. On native targets a caller can still meet it briefly held by the
//! task or another caller; on the browser's single thread it is never
//! contended. A caller whose destination change replaces a pending one drops
//! the replaced connection itself, which may wait on that device to close its
//! port.
//!

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::Notify;

use super::gate::TickGate;

///
/// A change to where Playback delivers, one slot for both kinds.
///
/// Disconnecting and installing a connection answer the same question — what
/// the next Tick is delivered to — so the newest of them is the answer and the
/// one it replaces is dropped unapplied.
///
pub(crate) enum Destination<D> {
    Disconnect,
    Output(D),
}

///
/// Everything handles asked of the task since it last looked, reduced to what
/// applying it in order would do.
///
/// The task applies it as one step, in a fixed order: the stop, then the
/// tempo, then the destination. That order leaves the engine in the state the
/// requests would have left it in had they been applied in the order they
/// were made, because each slot absorbs whatever an earlier request in
/// another slot would have changed, with the two exceptions below:
///
/// - A `stop` empties both tempo slots. Nothing asked before it survives it,
///   and whatever is asked after it applies to the stopped engine it leaves.
/// - `start` and `retune` are split by the state they act on, which only the
///   task knows. `start` does nothing to a live run and `retune` nothing to a
///   stopped engine, so each writes the slot for the state it changes. A
///   `retune` made while a `start` is pending also writes the start slot,
///   because the run that start begins is the run it retunes.
/// - The destination commutes with the other two: beginning, retuning and
///   stopping a run do not choose where it is delivered.
///
/// The exceptions are where a slot keeps the newest request rather than what
/// order would have applied. A `start` that replaces a pending one sets the
/// period the run begins at, where in order it would have found the run live
/// and done nothing: stopped, `start(1s)`, `retune(2s)`, `start(3s)` begins
/// at 3s rather than 2s. A `disconnect` that replaces a pending installation
/// leaves the published destination on the device installed before it,
/// where in order the replaced one would have been published first; nothing
/// is delivered either way.
///
/// The safety actions sent on the way there can differ from that order. A
/// `disconnect` then a `stop` made while playing silences the device twice,
/// once for each, where applying them in order silences it once; and an
/// installation then a `stop` sends the stop's silence to the outgoing device
/// rather than to the new one. Both are harmless: no Tick runs between the
/// parts of one backlog, and each device that was delivered to is silenced.
///
pub(super) struct Backlog<D> {
    /// Whether a stop was asked for. It answers one request on the gate.
    pub(super) stop: bool,
    /// The period to begin a run at, if the engine is stopped once `stop` is
    /// applied.
    pub(super) start: Option<Duration>,
    /// The period to retune to, if a run is live once `stop` is applied.
    pub(super) retune: Option<Duration>,
    pub(super) destination: Option<Destination<D>>,
}

impl<D> Backlog<D> {
    fn is_empty(&self) -> bool {
        !self.stop && self.start.is_none() && self.retune.is_none() && self.destination.is_none()
    }
}

impl<D> Default for Backlog<D> {
    fn default() -> Self {
        Self {
            stop: false,
            start: None,
            retune: None,
            destination: None,
        }
    }
}

struct Slots<D> {
    backlog: Backlog<D>,
    /// Whether the handle side is still there to ask for anything.
    sender_open: bool,
    /// Whether the task is still there to take what is asked.
    receiver_open: bool,
}

struct Mailbox<D> {
    slots: Mutex<Slots<D>>,
    arrived: Notify,
}

impl<D> Mailbox<D> {
    ///
    /// The slots, recovered from a poisoned lock.
    ///
    /// Every update under this lock is a handful of assignments that cannot
    /// panic part-way, so the slots a poisoned lock guards are whole.
    ///
    fn slots(&self) -> MutexGuard<'_, Slots<D>> {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The task is gone, so nothing asked now would be applied.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Unavailable;

///
/// The handle side of the mailbox.
///
/// There is one per engine, shared behind an `Arc` by every clone of the
/// engine's handle, so it is dropped exactly when the last one is. Dropping it
/// is what tells the task nothing else will be asked.
///
pub(crate) struct RequestSender<D> {
    mailbox: Arc<Mailbox<D>>,
}

///
/// The task's side of the mailbox.
///
pub(crate) struct RequestReceiver<D> {
    mailbox: Arc<Mailbox<D>>,
}

///
/// An empty mailbox and its two ends.
///
pub(crate) fn open<D>() -> (RequestSender<D>, RequestReceiver<D>) {
    let mailbox = Arc::new(Mailbox {
        slots: Mutex::new(Slots {
            backlog: Backlog::default(),
            sender_open: true,
            receiver_open: true,
        }),
        arrived: Notify::new(),
    });
    (
        RequestSender {
            mailbox: Arc::clone(&mailbox),
        },
        RequestReceiver { mailbox },
    )
}

impl<D> RequestSender<D> {
    ///
    /// Applies `update` to the backlog and wakes the task, or answers that the
    /// task is gone.
    ///
    fn update(&self, update: impl FnOnce(&mut Backlog<D>)) -> Result<(), Unavailable> {
        {
            let mut slots = self.mailbox.slots();
            if !slots.receiver_open {
                return Err(Unavailable);
            }
            update(&mut slots.backlog);
        }
        self.mailbox.arrived.notify_one();
        Ok(())
    }

    ///
    /// Asks for a run at `period`. A `start` still pending is replaced, so
    /// the newest period is the one a run begins at.
    ///
    pub(super) fn start(&self, period: Duration) -> Result<(), Unavailable> {
        self.update(|backlog| backlog.start = Some(period))
    }

    ///
    /// Asks for the live run to be retuned to `period`, and for a run a
    /// pending `start` begins to run at it.
    ///
    pub(super) fn retune(&self, period: Duration) -> Result<(), Unavailable> {
        self.update(|backlog| {
            backlog.retune = Some(period);
            if backlog.start.is_some() {
                backlog.start = Some(period);
            }
        })
    }

    ///
    /// Shuts Tick admission and asks the task to end the run.
    ///
    /// The gate is raised under the lock and only when no stop is pending, so
    /// the gate carries one standing request for the pending flag and at most
    /// one more for a flag the task has taken and not yet answered. A stop
    /// already pending has shut the gate, and the task answers it only after
    /// taking the flag, which this lock keeps from happening in between: a
    /// `stop` that finds one pending is covered by it and is answered with
    /// it.
    ///
    /// A task that is gone runs no Ticks, so there is nothing left to shut.
    ///
    pub(super) fn stop(&self, gate: &TickGate) {
        let _ = self.update(|backlog| {
            if !backlog.stop {
                gate.request_stop();
                backlog.stop = true;
            }
            backlog.start = None;
            backlog.retune = None;
        });
    }

    ///
    /// Replaces any destination change still pending with `destination`.
    ///
    /// The one replaced, and `destination` itself when the task is gone, is
    /// dropped here after the lock is released. Dropping a connection closes
    /// its port, which may wait on the device, and the task must not wait on
    /// that to take its backlog.
    ///
    pub(crate) fn change_destination(
        &self,
        destination: Destination<D>,
    ) -> Result<(), Unavailable> {
        let (answer, released) = {
            let mut slots = self.mailbox.slots();
            if slots.receiver_open {
                (Ok(()), slots.backlog.destination.replace(destination))
            } else {
                (Err(Unavailable), Some(destination))
            }
        };
        if answer.is_ok() {
            self.mailbox.arrived.notify_one();
        }
        drop(released);
        answer
    }

    ///
    /// Replaces the pending destination with what `change` makes of it, for
    /// a test fixture that has to see what is pending before it leaves a
    /// request of its own.
    ///
    #[cfg(test)]
    pub(super) fn fold_destination(
        &self,
        change: impl FnOnce(Option<Destination<D>>) -> Destination<D>,
    ) -> Result<(), Unavailable> {
        self.update(|backlog| {
            let pending = backlog.destination.take();
            backlog.destination = Some(change(pending));
        })
    }
}

impl<D> Drop for RequestSender<D> {
    fn drop(&mut self) {
        self.mailbox.slots().sender_open = false;
        self.mailbox.arrived.notify_one();
    }
}

impl<D> RequestReceiver<D> {
    ///
    /// Takes the whole backlog once there is one, or answers `None` once it is
    /// empty and nothing is left to ask.
    ///
    /// A backlog left when the handles are dropped is still delivered, so a
    /// request made just before the last handle went is applied before the
    /// engine shuts down.
    ///
    /// Cancel safe: the backlog is taken under the lock with no `await`
    /// between taking and returning it, so a caller that stops polling this
    /// loses nothing, and a wake it did not consume is only a later spurious
    /// check of the slots.
    ///
    pub(super) async fn recv(&self) -> Option<Backlog<D>> {
        loop {
            {
                let mut slots = self.mailbox.slots();
                if !slots.backlog.is_empty() {
                    return Some(std::mem::take(&mut slots.backlog));
                }
                if !slots.sender_open {
                    return None;
                }
            }
            self.mailbox.arrived.notified().await;
        }
    }

    ///
    /// Whether every handle is gone, so nothing further will be asked.
    ///
    pub(super) fn is_closed(&self) -> bool {
        !self.mailbox.slots().sender_open
    }
}

///
/// The task is gone, so a destination still pending will never be installed.
/// It is dropped here, releasing its device while handles may still be held.
///
impl<D> Drop for RequestReceiver<D> {
    fn drop(&mut self) {
        let released = {
            let mut slots = self.mailbox.slots();
            slots.receiver_open = false;
            slots.backlog.destination.take()
        };
        drop(released);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const ONE: Duration = Duration::from_secs(1);
    const TWO: Duration = Duration::from_secs(2);

    fn taken<D>(receiver: &RequestReceiver<D>) -> Backlog<D> {
        std::mem::take(&mut receiver.mailbox.slots().backlog)
    }

    /// A connection stand-in that counts its own release.
    struct Held(Arc<AtomicUsize>);

    impl Drop for Held {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn repeated_starts_keep_the_newest_period() {
        let (sender, receiver) = open::<()>();
        for _ in 0..1_000 {
            sender.start(ONE).unwrap();
        }
        sender.start(TWO).unwrap();

        let backlog = taken(&receiver);
        assert_eq!(backlog.start, Some(TWO));
        assert_eq!(backlog.retune, None);
    }

    #[test]
    fn a_retune_after_a_pending_start_sets_the_period_that_run_begins_at() {
        let (sender, receiver) = open::<()>();
        sender.start(ONE).unwrap();
        sender.retune(TWO).unwrap();

        let backlog = taken(&receiver);
        assert_eq!(backlog.start, Some(TWO));
        assert_eq!(backlog.retune, Some(TWO));
    }

    ///
    /// A retune asked before a start acts only on a run already live, and the
    /// start only on a stopped engine, so each keeps its own period.
    ///
    #[test]
    fn a_retune_before_a_start_keeps_its_own_period() {
        let (sender, receiver) = open::<()>();
        sender.retune(ONE).unwrap();
        sender.start(TWO).unwrap();

        let backlog = taken(&receiver);
        assert_eq!(backlog.retune, Some(ONE));
        assert_eq!(backlog.start, Some(TWO));
    }

    #[test]
    fn a_stop_empties_the_tempo_asked_before_it_and_keeps_what_follows() {
        let (sender, receiver) = open::<()>();
        let gate = TickGate::new();
        sender.start(ONE).unwrap();
        sender.retune(ONE).unwrap();
        sender.stop(&gate);
        sender.start(TWO).unwrap();

        let backlog = taken(&receiver);
        assert!(backlog.stop);
        assert_eq!(backlog.start, Some(TWO));
        assert_eq!(backlog.retune, None);
    }

    ///
    /// However many stops are made before the task takes the backlog, the
    /// gate carries one request for them, and answering it once reopens it.
    ///
    #[test]
    fn coalesced_stops_are_answered_by_one_answer() {
        let (sender, receiver) = open::<()>();
        let gate = TickGate::new();
        for _ in 0..1_000 {
            sender.stop(&gate);
        }
        assert!(!gate.begin_tick(), "a stop left Tick admission open");

        assert!(taken(&receiver).stop);
        gate.clear_stop();

        assert!(gate.begin_tick(), "a coalesced stop was left unanswered");
    }

    ///
    /// A stop made after the task took the flag and before it answered the
    /// request raises a request of its own, so the answer to the first does
    /// not reopen the gate over the second.
    ///
    #[test]
    fn a_stop_made_while_one_is_being_answered_keeps_the_gate_shut() {
        let (sender, receiver) = open::<()>();
        let gate = TickGate::new();
        sender.stop(&gate);
        assert!(taken(&receiver).stop);

        sender.stop(&gate);
        gate.clear_stop();
        assert!(!gate.begin_tick(), "the first answer reopened the gate");

        assert!(taken(&receiver).stop);
        gate.clear_stop();
        assert!(gate.begin_tick());
    }

    #[test]
    fn a_newer_destination_releases_the_one_it_replaces_at_once() {
        let released = Arc::new(AtomicUsize::new(0));
        let (sender, receiver) = open::<Held>();

        sender
            .change_destination(Destination::Output(Held(released.clone())))
            .unwrap();
        sender
            .change_destination(Destination::Output(Held(released.clone())))
            .unwrap();
        assert_eq!(released.load(Ordering::SeqCst), 1);

        sender.change_destination(Destination::Disconnect).unwrap();
        assert_eq!(released.load(Ordering::SeqCst), 2);
        assert!(matches!(
            taken(&receiver).destination,
            Some(Destination::Disconnect)
        ));
    }

    #[test]
    fn a_gone_task_refuses_requests_and_releases_the_connection() {
        let released = Arc::new(AtomicUsize::new(0));
        let (sender, receiver) = open::<Held>();
        let gate = TickGate::new();
        sender
            .change_destination(Destination::Output(Held(released.clone())))
            .unwrap();

        drop(receiver);
        assert_eq!(
            released.load(Ordering::SeqCst),
            1,
            "the pending connection outlived the task"
        );

        assert_eq!(
            sender.change_destination(Destination::Output(Held(released.clone()))),
            Err(Unavailable)
        );
        assert_eq!(released.load(Ordering::SeqCst), 2);
        assert_eq!(sender.start(ONE), Err(Unavailable));
        assert_eq!(sender.retune(ONE), Err(Unavailable));
        sender.stop(&gate);
        assert!(gate.begin_tick(), "a stop no task can answer shut the gate");
    }

    #[tokio::test]
    async fn the_backlog_left_by_the_last_handle_is_delivered_before_the_close() {
        let (sender, receiver) = open::<()>();
        sender.start(ONE).unwrap();
        drop(sender);

        let backlog = receiver.recv().await.expect("the backlog is delivered");
        assert_eq!(backlog.start, Some(ONE));
        assert!(receiver.recv().await.is_none());
    }

    #[tokio::test]
    async fn a_waiting_task_wakes_for_a_request() {
        let (sender, receiver) = open::<()>();
        let waiting = tokio::spawn(async move { receiver.recv().await.map(|b| b.retune) });
        tokio::task::yield_now().await;

        sender.retune(TWO).unwrap();

        assert_eq!(waiting.await.unwrap(), Some(Some(TWO)));
    }
}
