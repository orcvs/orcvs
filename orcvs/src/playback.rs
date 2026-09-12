use std::fmt;
use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::{mpsc, watch};
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::{self, Instant as ClockInstant};
#[cfg(target_arch = "wasm32")]
use web_time::Instant as ClockInstant;

mod gate;
mod schedule;

use crate::midi::MidiDestinations;
use crate::source::{
    BendLsb, BendMsb, ControlValue, Controller, MidiChannel, Note, SourceCommander, Tick, TickPlan,
    Velocity,
};
use gate::TickGate;
use schedule::OwnedNotes;

///
/// One MIDI message the Playback Engine hands an output adapter.
///
/// A Play Command says what the Source asked for; an Output Command says what
/// is delivered. The two differ wherever this module owns the difference: ADR
/// 0016 gives Timed and Monophonic Play a Tick lifetime, and resolving that
/// lifetime into a start now and a stop at Tick `T + length` belongs to the
/// engine that counts Ticks. Monophonic Play differs twice over, because
/// replacing the voice a channel was sounding also becomes a stop here — of
/// the note the engine found there, which the Source never named. Every
/// variant here is one message an adapter assembles immediately, so an adapter
/// holding a lifetime it would have to schedule is unrepresentable rather than
/// merely avoided.
///
/// A tagged variant set for the same reason a
/// [`PlayCommand`](crate::source::PlayCommand) is one. Control Change and
/// Pitch Bend are here as variants of their own and reach the
/// adapter exactly as the Source wrote them: neither carries a lifetime, so
/// there is nothing about either for this module to resolve, and a variant
/// apiece is what keeps that pass-through from being a Note On with the wrong
/// fields in it.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputCommand {
    /// MIDI's Note On, whose velocity `00` is the explicit stop the protocol
    /// gives both Raw Play and a scheduled expiry.
    NoteOn {
        channel: MidiChannel,
        velocity: Velocity,
        note: Note,
    },
    /// MIDI's Control Change: a controller and the value sent to it, each
    /// carrying the role it plays rather than the data-byte domain they share.
    ControlChange {
        channel: MidiChannel,
        controller: Controller,
        value: ControlValue,
    },
    /// MIDI's Pitch Bend, as the two seven-bit halves the wire carries. The
    /// adapter puts the LSB out first; nothing between the Source and the wire
    /// combines them, so nothing has to take them apart again.
    PitchBend {
        channel: MidiChannel,
        lsb: BendLsb,
        msb: BendMsb,
    },
}

pub trait OutputAdapter {
    fn submit(&mut self, commands: &[OutputCommand]) -> Result<(), OutputAdapterError>;
    fn safety_reset(&mut self) -> Result<(), OutputAdapterError>;

    ///
    /// A reader of the MIDI destinations this adapter publishes.
    ///
    /// ADR 0040 moves the adapter into the task that owns the engine's state,
    /// so nothing outside that task can reach the adapter to ask it anything.
    /// The subscription is therefore taken here, while the adapter is still in
    /// the constructor's hand, and travels to the handle that reads it — which
    /// is how the console draws its menu without awaiting an answer the
    /// browser main thread has no way to wait for.
    ///
    /// An adapter with no destination to choose has nothing to publish and
    /// answers with a reader of a channel whose sender is already gone, which
    /// is the same answer a reader gets once the running Orcvs publishing into
    /// it has ended.
    ///
    fn published_destinations(&self) -> watch::Receiver<MidiDestinations> {
        watch::Sender::new(MidiDestinations::default()).subscribe()
    }
}

fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutputAdapterError {
    pub message: String,
}

impl OutputAdapterError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlaybackDiagnostic {
    Overrun {
        scheduled_at: Duration,
        observed_at: Duration,
    },
    OutputFailure(OutputAdapterError),
    ClockFailure {
        message: String,
    },
    StartFailure {
        message: String,
    },
    RetuneFailure {
        message: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackStartError {
    ZeroTickPeriod,
    RuntimeUnavailable,
    ///
    /// The clock cannot express the deadlines a grid of this period would run
    /// on, so there is no instant for the run to wait until.
    ///
    UnschedulableTickPeriod,
    ///
    /// The task that owns the engine's state has ended, so there is nothing
    /// left to carry out the transition.
    ///
    EngineUnavailable,
}

///
/// Whether the clock can express a deadline one `tick_period` from now.
///
/// The task owns the clock, so an addition that cannot be made happens there,
/// where nothing can be returned to anyone. Asking here puts the answer where
/// a caller still exists to receive it. Both `start` and `retune` ask, because
/// both hand the clock a period to build a grid from.
///
fn is_schedulable(tick_period: Duration) -> bool {
    ClockInstant::now().checked_add(tick_period).is_some()
}

#[derive(Clone, Copy)]
struct TickTiming {
    epoch: ClockInstant,
    scheduled_at: Duration,
    observed_at: Duration,
    period: Duration,
}

impl TickTiming {
    fn is_overrun(self) -> bool {
        self.observed_at >= self.scheduled_at + self.period
    }

    ///
    /// The instant this Tick was due at.
    ///
    /// The clock holds the deadline outright — its epoch plus the offset the
    /// Tick was scheduled at — and carries it here rather than letting the
    /// engine rebuild it from the present. A reconstruction taken after the
    /// engine lock is acquired absorbs however long the Tick waited for that
    /// lock, and hands the next retune a grid offset by it, which is the
    /// permanent shift ADR 0037 rejects.
    ///
    /// An epoch and offset that cannot be added together describe a run some
    /// centuries long; answering with the present keeps the grid anchored on
    /// an instant rather than on nothing.
    ///
    fn deadline(self) -> ClockInstant {
        self.epoch
            .checked_add(self.scheduled_at)
            .unwrap_or_else(ClockInstant::now)
    }
}

///
/// The Tick the clock waits for after executing or declining the one scheduled
/// at `scheduled_at` and observed at `observed_at`.
///
/// ADR 0037 makes a late Tick lose its turn and holds the grid: every Tick of a
/// run is due at a whole multiple of `tick_period` from the deadline that run
/// began on, and a deadline the clock could not reach in time is skipped rather
/// than replayed or rescheduled from where the clock woke up. The ordinary case
/// is one period on from the deadline just handled; the missed case subtracts
/// how far into the current period the clock woke, so the answer lands back on
/// the grid instead of carrying the delay forward into every Tick after it.
///
/// Both native and browser clocks use this rule explicitly. Tokio's `Interval`
/// consults its `MissedTickBehavior` only once lateness exceeds five
/// milliseconds and replays the backlog below that, so `Skip` cannot express
/// this rule at the short end of the supported range: `Bpm` admits 1 to 15000,
/// which `Bpm::delay_ms` turns into periods from 15 seconds down to 1
/// millisecond.
///
/// A zero period cannot be divided into, and reaches here only if a caller
/// admitted one: `start` and `retune` both refuse `ZeroTickPeriod` before any
/// clock is spawned, so the guard answers the ordinary case rather than
/// choosing a policy for a run that cannot exist.
///
fn next_scheduled_at(
    scheduled_at: Duration,
    observed_at: Duration,
    tick_period: Duration,
) -> Duration {
    let next = scheduled_at.saturating_add(tick_period);
    if next > observed_at || tick_period.is_zero() {
        return next;
    }
    // A zero period returned above, so the divisor here is proven non-zero and
    // the phase is smaller than one period. A period too wide to express in
    // `u64` nanoseconds is some six centuries long; losing its phase costs grid
    // alignment, where saturating to `u64::MAX` would answer with a deadline
    // already behind the clock and spin it.
    debug_assert!(!tick_period.is_zero(), "the zero period returns above");
    let into_period = observed_at.saturating_sub(scheduled_at).as_nanos() % tick_period.as_nanos();
    let phase = Duration::from_nanos(u64::try_from(into_period).unwrap_or(0));
    observed_at
        .saturating_add(tick_period)
        .saturating_sub(phase)
}

///
/// The instant the first Tick of a retuned grid is due at, for a run whose
/// last executed Tick was due at `last_tick_at` and which is being retuned to
/// `tick_period` at `now`.
///
/// ADR 0037 runs a retuned grid from the deadline the last executed Tick was
/// due at, not from the moment the retune arrived, so this reads that deadline
/// and takes the first point of the new grid still ahead of `now`. A run with
/// no executed Tick behind it has no grid to keep, and begins one at `now`.
///
/// The answer is an instant rather than a wait, because a wait is only correct
/// against the instant it was measured from. Both clocks spawn a task before
/// they can sleep, and the browser's `spawn_local` defers behind whatever the
/// main thread is doing; a delay measured under the engine lock and applied
/// against the epoch that task captures later is late by exactly that gap, for
/// the rest of the run. Each clock re-derives its own wait from this instant
/// at its own epoch instead, which is what keeps the two targets holding one
/// rule — the reason ADR 0037 gives for `next_scheduled_at` being one function.
///
fn first_retuned_tick_at(
    last_tick_at: Option<ClockInstant>,
    now: ClockInstant,
    tick_period: Duration,
) -> ClockInstant {
    last_tick_at
        .and_then(|last_tick_at| {
            last_tick_at.checked_add(next_scheduled_at(
                Duration::ZERO,
                now.saturating_duration_since(last_tick_at),
                tick_period,
            ))
        })
        .unwrap_or(now)
}

#[cfg(any(test, target_arch = "wasm32"))]
fn wasm_timeout_millis(delay: Duration) -> u32 {
    delay
        .as_nanos()
        .div_ceil(1_000_000)
        .min(u128::from(u32::MAX)) as u32
}

///
/// A start error is an error: the console's own startup hands it to `eframe`,
/// which asks for one.
///
impl std::error::Error for PlaybackStartError {}

impl fmt::Display for PlaybackStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroTickPeriod => formatter.write_str("Tick period must be greater than zero"),
            Self::RuntimeUnavailable => formatter.write_str("Playback requires a Tokio runtime"),
            Self::UnschedulableTickPeriod => {
                formatter.write_str("Tick period is too long to schedule a deadline for")
            }
            Self::EngineUnavailable => formatter.write_str("Playback is no longer running"),
        }
    }
}

#[derive(Default)]
struct InMemoryOutputState {
    command_lists: Vec<Vec<OutputCommand>>,
    safety_reset_count: usize,
    next_failure: Option<OutputAdapterError>,
}

#[derive(Clone, Default)]
pub struct InMemoryOutputAdapter {
    state: Arc<Mutex<InMemoryOutputState>>,
}

impl InMemoryOutputAdapter {
    pub fn command_lists(&self) -> Vec<Vec<OutputCommand>> {
        self.state.lock().unwrap().command_lists.clone()
    }

    pub fn safety_reset_count(&self) -> usize {
        self.state.lock().unwrap().safety_reset_count
    }

    pub fn fail_next_submission(&self, message: impl Into<String>) {
        self.state.lock().unwrap().next_failure = Some(OutputAdapterError::new(message));
    }
}

impl OutputAdapter for InMemoryOutputAdapter {
    fn submit(&mut self, commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
        let mut state = self.state.lock().unwrap();
        if let Some(error) = state.next_failure.take() {
            return Err(error);
        }
        state.command_lists.push(commands.to_vec());
        Ok(())
    }

    fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
        self.state.lock().unwrap().safety_reset_count += 1;
        Ok(())
    }
}

struct PlaybackInner<A: OutputAdapter> {
    source: SourceCommander,
    adapter: A,
    ///
    /// The lifecycle state, published rather than held.
    ///
    /// ADR 0040 has the console read this without awaiting and without
    /// reaching the engine, because the frame that gates Space on it cannot
    /// wait for an answer. The sender is the one copy of the fact: this
    /// engine reads its own state back through `borrow`, so what it acts on
    /// and what the console sees cannot drift apart.
    ///
    state: watch::Sender<PlaybackState>,
    connected: bool,
    ///
    /// The writing end of the diagnostics stream.
    ///
    /// ADR 0002 asks that diagnostics be drained in order and exactly once
    /// while lifecycle state is observed; ADR 0040 moves that guarantee from
    /// the lock to this channel, which is ordered, and from which a receive
    /// takes each diagnostic away. It is unbounded because the queue it
    /// replaces — a `Vec` drained by the console each frame — was, and because
    /// a dropped diagnostic is a device failure the user is never told about.
    ///
    diagnostics: mpsc::UnboundedSender<PlaybackDiagnostic>,
    last_output_failure: Option<OutputAdapterError>,
    last_tick_at: Option<ClockInstant>,
    ///
    /// The absolute Tick the next executed Tick interprets its Source Snapshot
    /// at.
    ///
    /// It lives here because the Playback Engine owns musical time and ADR 0003
    /// keeps every piece of language state in the Source Snapshot: a counter
    /// beside the Source would be neither. It counts executed Ticks, so a Tick
    /// this engine declines to run — stopped, superseded, or overrun — leaves it
    /// where it was.
    ///
    tick: Tick,
    ///
    /// The notes Timed and Monophonic Play own, and the Tick each is stopped
    /// at.
    ///
    /// Here rather than beside the Source for the reason the absolute Tick is:
    /// a scheduled stop belongs to one Playback run, and ADR 0003 keeps every
    /// piece of language state in the Source Snapshot, which a schedule of
    /// future effects is not.
    ///
    owned: OwnedNotes,
}

///
/// One message the engine's task applies to the state it owns.
///
/// A handle validates and then sends; the task is the only thing that touches
/// the state, so every transition arrives here in the order it was asked for.
///
enum PlaybackCommand<A: OutputAdapter> {
    Start {
        tick_period: Duration,
    },
    Retune {
        tick_period: Duration,
    },
    Stop,
    Disconnect,
    ///
    /// A transition whose shape belongs to the output adapter rather than to
    /// the lifecycle.
    ///
    /// `PlaybackEngine` is generic over its adapter and choosing a MIDI
    /// destination exists for one of them, so the variant carries the
    /// transition itself rather than naming a message only one adapter could
    /// ever answer. It is still one message in one queue, applied by the one
    /// task that owns the state: the alternative shapes — a second channel, or
    /// a lifecycle variant that most adapters must refuse — buy nothing and
    /// cost an ordering guarantee each.
    ///
    Adapter(AdapterTransition<A>),
}

type AdapterTransition<A> = Box<dyn FnOnce(&mut PlaybackInner<A>) + Send>;

///
/// A cloneable handle to one Playback Engine.
///
/// ADR 0040 puts the state in a task and leaves this holding the ends of the
/// channels that reach it: a sender for the transitions, readers for what the
/// engine publishes, and the one bit of shared state a synchronous `stop`
/// needs. There is no lock here, no task handle, and nothing to be stale
/// relative to.
///
pub struct PlaybackEngine<A: OutputAdapter> {
    ///
    /// The writing end of the transition queue.
    ///
    /// Every handle holds a clone, so the queue closes exactly when the last
    /// one is dropped. That close is what shuts the engine down: the task sees
    /// it, sends the safety action, and exits. The count that used to say the
    /// same thing arithmetically is gone with it.
    ///
    commands: mpsc::UnboundedSender<PlaybackCommand<A>>,
    ///
    /// Whether someone has asked this engine to stop.
    ///
    /// ADR 0002 requires that further Ticks are prevented before `stop`
    /// returns, and sending a message does not do that: `send` returns once the
    /// message is queued and the task may be mid-Tick. This is set by the
    /// handle before the message goes and read by the task immediately before
    /// it executes each Tick, so a Tick whose read begins after `stop` returned
    /// finds the request and declines.
    ///
    /// ADR 0040 admits this one piece of shared state deliberately. It carries
    /// one fact in one direction — someone has asked me to stop — and nothing
    /// reads it to decide which state the engine is in. "A stop has been
    /// requested" and "this engine is playing" are different facts: the second
    /// one is published through `state` and belongs to the task alone, and
    /// collapsing it into this gate would re-admit the shared lifecycle state
    /// this decision removes.
    ///
    tick_gate: Arc<TickGate>,
    /// The reading end of the published lifecycle state. Read without awaiting
    /// and without reaching the engine, which is what lets a console frame
    /// gate on it.
    state: watch::Receiver<PlaybackState>,
    ///
    /// The reading end of the diagnostics stream.
    ///
    /// One receiver for however many handles there are, which is what an
    /// ordered stream drained exactly once means: two handles draining split
    /// the diagnostics between them rather than each seeing every one, exactly
    /// as two callers of the drained `Vec` this replaces did. The lock is over
    /// the receiver alone and is never taken by the engine, so a drain waits
    /// on no Tick and a Tick waits on no drain.
    ///
    diagnostics: Arc<Mutex<mpsc::UnboundedReceiver<PlaybackDiagnostic>>>,
    /// The writing end this handle reports its own failures on, so that a
    /// caller draining on the next line finds them.
    reports: mpsc::UnboundedSender<PlaybackDiagnostic>,
    ///
    /// The reading end of the adapter's published MIDI destinations,
    /// subscribed while the adapter was still in hand at construction.
    ///
    destinations: watch::Receiver<MidiDestinations>,
}

///
/// The MIDI configuration capability, without Playback lifecycle control.
///
/// It holds a weak sender rather than a clone of one, so that it cannot keep
/// the engine's task alive: every method answers "running Orcvs is no longer
/// available" once the last `PlaybackEngine` has been dropped, which is the
/// guarantee the strong senders and this weak one draw between them.
///
pub struct MidiSelectionHandle<B: crate::midi::MidiBackend> {
    commands: mpsc::WeakUnboundedSender<PlaybackCommand<crate::midi::MidiOutputAdapter<B>>>,
    destinations: watch::Receiver<MidiDestinations>,
}

impl<B: crate::midi::MidiBackend + 'static> MidiSelectionHandle<B> {
    pub(crate) fn new(playback: &PlaybackEngine<crate::midi::MidiOutputAdapter<B>>) -> Self {
        Self {
            commands: playback.commands.downgrade(),
            destinations: playback.destinations.clone(),
        }
    }

    ///
    /// Queues `transition` for the engine's task, or answers that there is no
    /// longer a running Orcvs to queue it for.
    ///
    fn request(
        &self,
        transition: impl FnOnce(&mut PlaybackInner<crate::midi::MidiOutputAdapter<B>>) + Send + 'static,
    ) -> Result<(), crate::midi::MidiError> {
        let commands = self
            .commands
            .upgrade()
            .ok_or_else(|| crate::midi::MidiError::new("running Orcvs is no longer available"))?;
        commands
            .send(PlaybackCommand::Adapter(Box::new(transition)))
            .map_err(|_| crate::midi::MidiError::new("running Orcvs is no longer available"))
    }

    ///
    /// Asks the engine to discover the destinations its backend offers.
    ///
    /// Discovery reaches a platform MIDI service through the adapter, which
    /// the engine's task owns, so this asks rather than answers: what it
    /// found — or the failure it reported — arrives through
    /// [`destinations`](Self::destinations) once the task has run.
    ///
    pub fn refresh_destinations(&self) -> Result<(), crate::midi::MidiError> {
        self.request(PlaybackInner::refresh_destinations)
    }

    ///
    /// The destinations the engine last published, or the failure the last
    /// discovery reported.
    ///
    /// Read from the published value rather than asked of the engine, for the
    /// reason every other observation is: the console compares this list
    /// against its menu while drawing a frame, and the browser main thread has
    /// no blocking receive with which to wait for an answer.
    ///
    pub fn destinations(
        &self,
    ) -> Result<Vec<crate::midi::MidiDestination>, crate::midi::MidiError> {
        if self.destinations.has_changed().is_err() {
            return Err(crate::midi::MidiError::new(
                "running Orcvs is no longer available",
            ));
        }
        self.destinations.borrow().discovered.clone()
    }

    ///
    /// Asks the engine to connect its output to `destination_id`.
    ///
    /// The answer this returns is whether there is still a running Orcvs to
    /// ask. Whether the device accepted the connection is the engine's to
    /// report: a refusal becomes a Playback diagnostic, on the one ordered
    /// stream every other output failure is reported on, and the selection
    /// that succeeded appears in the published destinations.
    ///
    pub fn select(
        &self,
        destination_id: &crate::midi::MidiDestinationId,
    ) -> Result<(), crate::midi::MidiError> {
        let destination_id = destination_id.clone();
        self.request(move |inner| inner.select_destination(&destination_id))
    }

    ///
    /// The destination the engine last published, read without awaiting and
    /// without reaching the engine.
    ///
    /// The console compares this against every row of its menu while drawing a
    /// frame, which is why it is read from the published value rather than
    /// asked of the engine. The running Orcvs that publishes into the channel
    /// is what keeps it open, so a handle outliving that Orcvs learns it the
    /// same way the methods above do.
    ///
    pub fn selected_destination_id(
        &self,
    ) -> Result<Option<crate::midi::MidiDestinationId>, crate::midi::MidiError> {
        if self.destinations.has_changed().is_err() {
            return Err(crate::midi::MidiError::new(
                "running Orcvs is no longer available",
            ));
        }
        Ok(self.destinations.borrow().selected.clone())
    }
}

impl<A: OutputAdapter> Clone for PlaybackEngine<A> {
    fn clone(&self) -> Self {
        Self {
            commands: self.commands.clone(),
            tick_gate: Arc::clone(&self.tick_gate),
            state: self.state.clone(),
            diagnostics: self.diagnostics.clone(),
            reports: self.reports.clone(),
            destinations: self.destinations.clone(),
        }
    }
}
///
/// The reading ends of what a Playback Engine publishes, handed back by
/// [`PlaybackInner::new`] to whoever is building a handle over that state.
///
struct PlaybackChannels {
    state: watch::Receiver<PlaybackState>,
    diagnostics: mpsc::UnboundedReceiver<PlaybackDiagnostic>,
    ///
    /// A second writing end of the diagnostics stream, for the handle's own
    /// reports.
    ///
    /// A handle that could not report would have to queue its failures for the
    /// task, which is a message behind the caller draining the stream on the
    /// next line. One queue with two writers keeps the order the reports were
    /// made in, which is the order ADR 0002 asks for.
    ///
    reports: mpsc::UnboundedSender<PlaybackDiagnostic>,
}

impl<A: OutputAdapter> PlaybackInner<A> {
    ///
    /// The state one Playback Engine begins with, and the readers of what it
    /// publishes.
    ///
    /// The writing ends stay with the state, which is what keeps the published
    /// facts and the facts the engine acts on the same ones.
    ///
    fn new(source: SourceCommander, adapter: A) -> (Self, PlaybackChannels) {
        let state = watch::Sender::new(PlaybackState::Stopped);
        let observed_state = state.subscribe();
        let (diagnostics, drained) = mpsc::unbounded_channel();
        let reports = diagnostics.clone();
        (
            Self {
                source,
                adapter,
                state,
                connected: true,
                diagnostics,
                last_output_failure: None,
                last_tick_at: None,
                tick: Tick::ZERO,
                owned: OwnedNotes::default(),
            },
            PlaybackChannels {
                state: observed_state,
                diagnostics: drained,
                reports,
            },
        )
    }

    ///
    /// Whether this engine is in a run, read back from the value it publishes.
    ///
    /// The published state is the only copy, so this is the same fact the
    /// console gates Space on rather than a second one kept beside it.
    ///
    fn is_playing(&self) -> bool {
        *self.state.borrow() == PlaybackState::Playing
    }

    ///
    /// Publishes `state` as this engine's lifecycle state.
    ///
    /// `send_replace` rather than `send`, because the value must be stored
    /// whether or not anyone is reading: an engine whose console has gone
    /// still has to know its own state, and the next reader to subscribe
    /// reads the latest one.
    ///
    fn publish_state(&self, state: PlaybackState) {
        self.state.send_replace(state);
    }

    ///
    /// Queues one diagnostic for whoever drains the stream.
    ///
    /// The send fails only once the receiving end is gone, which happens when
    /// the last handle is dropping and there is no console left to tell.
    ///
    fn report(&self, diagnostic: PlaybackDiagnostic) {
        let _ = self.diagnostics.send(diagnostic);
    }

    fn stop(&mut self) {
        if self.is_playing() {
            self.publish_state(PlaybackState::Stopped);
            if self.connected {
                self.send_safety_reset();
            }
        }
        // The safety action has stopped whatever was sounding, so every claim is
        // over and every scheduled stop is redundant. Clearing unconditionally
        // rather than alongside the safety action above is deliberate: a run
        // that is already stopped owns nothing, and an engine that reached
        // here holding a claim would otherwise carry it into the next run.
        self.owned.clear();
    }

    ///
    /// Gives up this engine's output, sending the safety action to the device
    /// it is leaving.
    ///
    fn disconnect(&mut self) {
        if self.connected {
            self.send_safety_reset();
            self.connected = false;
        }
        // Nothing this engine owns is sounding on a disconnected output, and
        // nothing it delivers while disconnected can start a note, so the
        // schedule goes with the connection.
        self.owned.clear();
    }

    fn send_safety_reset(&mut self) {
        if let Err(error) = self.adapter.safety_reset() {
            self.record_output_failure(error);
        }
    }

    fn record_output_failure(&mut self, error: OutputAdapterError) {
        if self.last_output_failure.as_ref() != Some(&error) {
            self.report(PlaybackDiagnostic::OutputFailure(error.clone()));
            self.last_output_failure = Some(error);
        }
    }

    ///
    /// Begins a Playback run. ADR 0002 keeps lifecycle concurrency inside this
    /// module, so every run begins through this one place: the native target,
    /// the browser and the tests all begin a run identically, and a run-scoped
    /// input added here cannot be reset on one target and forgotten on
    /// another.
    ///
    /// ADR 0012 makes the absolute Tick an interpretation input, so a run that
    /// began while still carrying the previous run's counter would fire a Delay
    /// `~*0104` on the wrong beat. Resetting the counter is therefore part of
    /// beginning a run, alongside the cleared last Tick that lets this run's
    /// first Tick execute immediately.
    ///
    /// `retune` deliberately does not begin a run: retuning changes the Tick
    /// period of the run it is already in, so it keeps that run's absolute Tick
    /// and reads the last Tick to schedule the first retuned Tick against it.
    ///
    fn begin_run(&mut self) {
        self.publish_state(PlaybackState::Playing);
        self.last_tick_at = None;
        self.tick = Tick::ZERO;
        // A scheduled stop is due at an absolute Tick, and this run's absolute
        // Ticks begin again at zero, so an inherited expiry would stop a note
        // of the new run that has not started. Discarding the schedule is part
        // of beginning a run for the same reason resetting the counter is.
        self.owned.clear();
        // The latch suppresses a report of the failure it already reported, so
        // a device refusing every Tick is reported once rather than once per
        // Tick. That is one report per run, not one per adapter lifetime: a
        // drain takes the diagnostics away, so a run inheriting the latch would
        // say nothing at all about a device still failing in front of the user.
        //
        // Here alone, and deliberately not in `stop`. The safety action `stop`
        // sends is the last submission of the run that recorded the latch, so a
        // refusal repeating that run's failure is the same fault, which is what
        // the latch is for; clearing before sending it would report one device
        // fault twice. A stopped engine submits nothing, so a latch left
        // standing after `stop` suppresses nothing until this line clears it.
        self.last_output_failure = None;
    }

    ///
    /// Executes one Tick of the run in progress: interprets a Source Snapshot
    /// into a Tick Plan and advances the absolute Tick. Named apart from the
    /// `tick` field it advances, so a call site says whether it reads the
    /// counter or spends one.
    ///
    /// There is no run to check for and no clock to be stale relative to. The
    /// deadlines belong to the task that owns this state, and that task keeps
    /// them only while a run is live, so reaching here at all is what says a
    /// run is in progress.
    ///
    fn execute_tick(&mut self, timing: TickTiming) -> Option<TickPlan> {
        if timing.is_overrun() {
            self.report(PlaybackDiagnostic::Overrun {
                scheduled_at: timing.scheduled_at,
                observed_at: timing.observed_at,
            });
            return None;
        }
        // A retune runs its new grid from this instant, and a grid is made of
        // deadlines, so what is recorded is the deadline this Tick was due at
        // rather than the moment it was seen. Recording the observation would
        // let an ordinary slow Tick shift the grid at the next retune, which is
        // the permanent offset ADR 0037 rejects; so would rebuilding the
        // deadline from the present here, which would carry the wait for this
        // engine's lock into the grid instead.
        self.last_tick_at = Some(timing.deadline());
        // One Tick executed, one increment. The advance sits with the
        // execution rather than with the clock so that a Tick the engine
        // declines above never consumes an absolute Tick.
        let tick = self.tick;
        let plan = self.source.execute(tick);
        self.tick = tick.next();
        if self.connected {
            // Nothing is delivered while disconnected, so nothing is owned
            // while disconnected either: resolving the Tick Plan here rather
            // than above keeps a Note Off from being scheduled for a note that
            // never sounded.
            //
            // The schedule describes notes that are sounding, so it may record
            // only what the adapter accepted. The Tick is resolved against a
            // copy and adopted once the submission is: a refused submission
            // leaves every claim and every expiry standing, and the stop this
            // Tick drained is drained again by the next executed Tick rather
            // than discarded with no retry and no diagnostic. `OutputAdapter`
            // is a trait, so the alternative would rest on every
            // implementation giving up its connection the way
            // `MidiOutputAdapter` does. The copy is two maps of the notes
            // currently sounding, taken once per executed Tick.
            let mut owned = self.owned.clone();
            let delivery = owned.deliver(tick, &plan.play_commands);
            match self.adapter.submit(&delivery) {
                Ok(()) => {
                    self.owned = owned;
                    self.last_output_failure = None;
                }
                Err(error) => self.record_output_failure(error),
            }
        }
        Some(plan)
    }
}

///
/// The last thing the state does is silence whatever it was sounding.
///
/// The task's ordinary exit stops the run itself, so this is the extraordinary
/// one: a panic unwinding out of an adapter, or a runtime dropping the task
/// while a run is live. The device that a run was delivering to is the same
/// device either way, and leaving it holding notes is the failure worth
/// preventing. It is not a second party watching a first — it is this state
/// finishing what it started, at the one moment nothing else can.
///
impl<A: OutputAdapter> Drop for PlaybackInner<A> {
    fn drop(&mut self) {
        if !self.is_playing() {
            return;
        }
        self.report(PlaybackDiagnostic::ClockFailure {
            message: "Playback clock terminated unexpectedly".to_string(),
        });
        // The unwind that reached this destructor is the one an adapter
        // started, and the safety action reaches straight back into that same
        // adapter. A backend that panicked delivering is a fair bet to panic
        // being silenced, and a panic escaping a `Drop` mid-unwind is not a
        // failed safety action — it is `abort`, taking every other window and
        // whatever the user had not saved with it. So the attempt is made and
        // its failure is contained: a device left holding notes is the cost of
        // a backend that will not answer either call, and it is the smaller
        // one.
        //
        // That containment is native. `wasm32-unknown-unknown` defaults to
        // `panic = "abort"` and no profile in the workspace changes it, so on
        // the browser target a panicking adapter traps where it is raised:
        // nothing unwinds into this destructor, and a `stop` that panics here
        // ends the page rather than being contained. The call is left
        // unconditional because it costs nothing where it cannot fire and is
        // the whole safety action where it can.
        //
        // `AssertUnwindSafe` is sound here because this state is being
        // dropped. A `stop` that panics part-way leaves fields nothing will
        // read again, and the borrow ends with this function.
        let silenced = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.stop()));
        if silenced.is_err() {
            // Reported for the same reason the clock failure above is: the
            // console is the only place a user learns that a device may still
            // be sounding. Recorded after the attempt, so the ordered stream
            // reads in the order things happened.
            self.report(PlaybackDiagnostic::ClockFailure {
                message: "Playback output could not be silenced".to_string(),
            });
        }
    }
}

impl<A: OutputAdapter> PlaybackEngine<A> {
    ///
    /// The lifecycle state this engine last published.
    ///
    /// It reads the latest published value: no await, and no wait on whatever
    /// this engine is doing. That is what lets a console frame gate Space on
    /// it — and on the browser it is not merely the faster option, because the
    /// main thread there has no blocking receive with which to ask.
    ///
    pub fn state(&self) -> PlaybackState {
        *self.state.borrow()
    }

    ///
    /// Takes every diagnostic recorded since the last drain, in the order the
    /// engine recorded them.
    ///
    /// A non-blocking receive repeated to exhaustion: a caller drawing a frame
    /// gets what is queued and never waits for what is not. Each diagnostic is
    /// delivered to exactly one drain, so the engine reports a device failure
    /// once and a console shows it once.
    ///
    pub fn drain_diagnostics(&self) -> Vec<PlaybackDiagnostic> {
        let mut drained = lock_recover(&self.diagnostics);
        let mut diagnostics = Vec::new();
        while let Ok(diagnostic) = drained.try_recv() {
            diagnostics.push(diagnostic);
        }
        diagnostics
    }

    ///
    /// Queues `command` for the task that owns the state, or answers that
    /// there is no longer a task to queue it for.
    ///
    /// A send fails once the receiving end is gone, which happens either
    /// because the last handle was dropped — not this one, which is being
    /// called on — or because the task ended without being asked to. A panic
    /// unwinding out of an adapter is the way that happens, and `ClockSpawner`
    /// keeps no `JoinHandle` to notice it by, so this failure is the only
    /// evidence a handle ever gets. ADR 0040 puts the state in the task, so a
    /// task that ended took the state with it and there is nothing left to
    /// respawn a clock over: what is owed the caller is the truth, not a
    /// recovery.
    ///
    fn send(&self, command: PlaybackCommand<A>) -> Result<(), PlaybackStartError> {
        self.commands
            .send(command)
            .map_err(|_| PlaybackStartError::EngineUnavailable)
    }

    ///
    /// Ends the Playback run, if there is one, and silences the output.
    ///
    /// ADR 0002 requires that further Ticks are prevented before this returns,
    /// which the request outruns the message to do: the gate is shut here, and
    /// the task must be admitted through it to execute a Tick. Admission is one
    /// atomic step, so every Tick is on one side of this call or the other —
    /// either it was admitted before the gate shut, and runs to completion, or
    /// it is refused. There is no third case of a Tick that read the gate as
    /// open and executes afterwards, which is the whole of what the guarantee
    /// asks for. The message behind the request carries the transition — the
    /// published state, the safety action and the cleared schedule — which is
    /// the task's alone to make.
    ///
    /// A Tick already admitted is not waited for. The browser main thread has
    /// no blocking receive to wait with, so this returns while that Tick is
    /// still delivering, and the safety action the message carries silences
    /// whatever it started.
    ///
    pub fn stop(&self) {
        self.tick_gate.request_stop();
        // Nothing to tell a caller: a stop asked of an engine whose task has
        // ended is a transition that engine has already made, and the state it
        // would have silenced went with the task.
        let _ = self.send(PlaybackCommand::Stop);
    }

    pub fn disconnect(&self) {
        // An engine whose task has ended is disconnected from everything it
        // was delivering to, so there is nothing to report and nothing left to
        // ask.
        let _ = self.send(PlaybackCommand::Disconnect);
    }
}

impl<B: crate::midi::MidiBackend> PlaybackInner<crate::midi::MidiOutputAdapter<B>> {
    ///
    /// Discovers the destinations this engine's backend offers and publishes
    /// what it found, or the failure it reported.
    ///
    fn refresh_destinations(&mut self) {
        self.adapter.refresh_destinations();
    }

    ///
    /// Connects this engine's output to `destination_id`.
    ///
    /// Both ways a destination is chosen — this engine's own method and the
    /// selection handle the console holds — arrive here, so what a change of
    /// destination owes is stated once rather than twice.
    ///
    /// A refusal is reported rather than returned. The task that owns this
    /// state cannot answer a caller synchronously, so the one ordered stream
    /// every other output failure travels on is where this one goes too.
    ///
    fn select_destination(&mut self, destination_id: &crate::midi::MidiDestinationId) {
        // The notes this engine owned are sounding on the destination it is
        // leaving, which is sent the safety action before the new connection is
        // reached. Their scheduled stops would arrive at a device that never
        // started them, so the schedule goes with the attempt rather than with
        // its success: a change that cannot connect has silenced the old
        // device just the same, and a claim kept across it would stop a note
        // the Source starts on that voice afterwards. Nothing is owned while
        // disconnected, so clearing before a failure that leaves this engine
        // connected to the destination it already had discards nothing else.
        self.owned.clear();
        // The latch stops a run reporting the same broken device once per
        // Tick, and a selection is not a Tick: it is a thing the user just
        // asked for, and it is owed its own answer even when the answer is the
        // one the last attempt got. Clearing before the attempt rather than
        // after it is what makes a second refusal of the same device visible;
        // clearing only on success leaves the console showing nothing while
        // the device is still unplugged.
        self.last_output_failure = None;
        let selection = match self.adapter.select(destination_id) {
            Ok(selection) => selection,
            Err(error) => {
                self.record_output_failure(OutputAdapterError::new(error.message));
                return;
            }
        };
        if let Some(error) = selection.safety_failure() {
            self.record_output_failure(OutputAdapterError::new(error.message));
        }
        self.connected = true;
    }
}

impl<A: OutputAdapter + Send + 'static> PlaybackEngine<A> {
    ///
    /// One Playback Engine over `source`, delivering to `adapter`, with the
    /// task that owns its state already running.
    ///
    /// Fallible and eager, because an engine without its task is not one: the
    /// state has no owner, `stop` has nothing to silence the output, and every
    /// message queued against it is queued against nothing. A runtime is what
    /// the task needs and this is where it is needed, so
    /// [`PlaybackStartError::RuntimeUnavailable`] is answered here rather than
    /// at the first `start`.
    ///
    pub fn new(source: SourceCommander, adapter: A) -> Result<Self, PlaybackStartError> {
        let spawner = ClockSpawner::acquire()?;
        // Subscribed while the adapter is still in hand. Once it is the task's
        // there is no way back to it, which is the whole of what ADR 0040 buys.
        let destinations = adapter.published_destinations();
        let (inner, channels) = PlaybackInner::new(source, adapter);
        let (commands, queued) = mpsc::unbounded_channel();
        let tick_gate = Arc::new(TickGate::new());
        spawner.spawn(run_engine(inner, queued, Arc::clone(&tick_gate)));
        Ok(Self {
            commands,
            tick_gate,
            state: channels.state,
            diagnostics: Arc::new(Mutex::new(channels.diagnostics)),
            reports: channels.reports,
            destinations,
        })
    }

    ///
    /// Records a start failure as a diagnostic so the console can surface it, and
    /// hands the error back for the caller to return. Every `start` failure path
    /// goes through here; a caller that only returns the error leaves the user
    /// with silence.
    ///
    /// Reported by the handle rather than queued for the task, because the
    /// caller that is being handed the error is the one whose next line drains
    /// the stream: a report made a message behind would not be there yet.
    ///
    fn report_start_error(&self, error: PlaybackStartError) -> PlaybackStartError {
        self.report(PlaybackDiagnostic::StartFailure {
            message: error.to_string(),
        });
        error
    }

    pub(crate) fn report_retune_error(&self, error: PlaybackStartError) {
        self.report(PlaybackDiagnostic::RetuneFailure {
            message: error.to_string(),
        });
    }

    fn report(&self, diagnostic: PlaybackDiagnostic) {
        let _ = self.reports.send(diagnostic);
    }

    ///
    /// Changes the Tick period of the run in progress, and does nothing to an
    /// engine that is not in one.
    ///
    /// The period is validated here, so a caller that must know whether its
    /// tempo was accepted — `Orcvs::set_bpm` decides whether to apply the BPM
    /// at all — is answered without waiting. The transition is the task's: it
    /// anchors the new grid on the deadline the last executed Tick was due at,
    /// which is ADR 0037's rule and is read from the state that task owns.
    ///
    pub(crate) fn retune(&self, tick_period: Duration) -> Result<(), PlaybackStartError> {
        if tick_period.is_zero() {
            return Err(PlaybackStartError::ZeroTickPeriod);
        }
        if !is_schedulable(tick_period) {
            return Err(PlaybackStartError::UnschedulableTickPeriod);
        }
        self.send(PlaybackCommand::Retune { tick_period })
    }

    ///
    /// Begins a Playback run at `tick_period`, and does nothing to an engine
    /// already in one.
    ///
    /// Idempotence belongs to the task, which is the only thing that knows
    /// whether a run is live at the moment the message is applied: two starts
    /// queued before either is applied are one run, and a check made here
    /// against a published value either message could outrun is not what makes
    /// that true.
    ///
    pub fn start(&self, tick_period: Duration) -> Result<(), PlaybackStartError> {
        if tick_period.is_zero() {
            return Err(self.report_start_error(PlaybackStartError::ZeroTickPeriod));
        }
        if !is_schedulable(tick_period) {
            return Err(self.report_start_error(PlaybackStartError::UnschedulableTickPeriod));
        }
        self.send(PlaybackCommand::Start { tick_period })
            .map_err(|error| self.report_start_error(error))
    }
}

/// Runtime availability is settled before a run's state changes. The browser
/// spawner needs no runtime handle; its future need not be Send either.
struct ClockSpawner {
    #[cfg(not(target_arch = "wasm32"))]
    runtime: tokio::runtime::Handle,
}

impl ClockSpawner {
    fn acquire() -> Result<Self, PlaybackStartError> {
        Ok(Self {
            #[cfg(not(target_arch = "wasm32"))]
            runtime: tokio::runtime::Handle::try_current()
                .map_err(|_| PlaybackStartError::RuntimeUnavailable)?,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn(self, clock: impl Future<Output = ()> + Send + 'static) {
        self.runtime.spawn(clock);
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn(self, clock: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(clock);
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_until(deadline: ClockInstant) {
    time::sleep_until(deadline).await;
}

#[cfg(target_arch = "wasm32")]
async fn sleep_until(deadline: ClockInstant) {
    let delay = deadline.saturating_duration_since(ClockInstant::now());
    // A deadline already behind the clock still waits, on a timer of zero.
    // This is the browser clock's only yield back to the event loop, so
    // skipping it for an elapsed deadline would let a Tick costing more than
    // its period run the loop again and again without the page ever getting a
    // turn: nothing rendered, no input dispatched, and no moment in which the
    // Space that calls `stop` could be delivered. The immediate first Tick,
    // which is what a browser timer would cost a whole period at the short end
    // of `Bpm`, is spared this wait by `run_clock` and not by this function.
    gloo_timers::future::TimeoutFuture::new(wasm_timeout_millis(delay)).await;
}

///
/// The deadlines of the run in progress.
///
/// One grid at a time: an epoch to measure it from, the offset of the deadline
/// next due, and the period between deadlines. It exists exactly while a run
/// has a grid, so the engine's task holding one is what says a Tick is due at
/// all — which is why nothing else needs to be asked whether a run is live.
///
struct TickClock {
    epoch: ClockInstant,
    scheduled_at: Duration,
    period: Duration,
    ///
    /// Whether the deadline next due is executed on arrival rather than waited
    /// for.
    ///
    /// `start` is due at its own epoch, and a `retune` may be anchored on a
    /// Tick already behind it. On the browser even a zero timer costs a
    /// `setTimeout` hop of one to four milliseconds, which is enough for
    /// `is_overrun` to decline that Tick at the one-millisecond end of `Bpm`.
    ///
    /// Only the first. Every deadline after it waits whether or not it has
    /// elapsed, because that wait is the browser clock's only yield back to the
    /// event loop.
    ///
    due_on_arrival: bool,
}

impl TickClock {
    /// The grid a run begins on: its first Tick is due at the moment the run
    /// began, which is now.
    fn beginning(period: Duration) -> Self {
        Self {
            epoch: ClockInstant::now(),
            scheduled_at: Duration::ZERO,
            period,
            due_on_arrival: true,
        }
    }

    /// The grid a retune runs from, whose first deadline `first_tick_at`
    /// already names.
    fn retuned(first_tick_at: ClockInstant, period: Duration) -> Self {
        let epoch = ClockInstant::now();
        let scheduled_at = first_tick_at.saturating_duration_since(epoch);
        Self {
            epoch,
            scheduled_at,
            period,
            due_on_arrival: scheduled_at.is_zero(),
        }
    }

    ///
    /// The instant this clock's next Tick is due at, or nothing when that
    /// instant cannot be expressed.
    ///
    /// `TickTiming::deadline` answers the same question for a Tick being
    /// executed and falls back to the present, which is right there and wrong
    /// here: a deadline of now is one this loop reaches immediately, executes,
    /// and computes the same unrepresentable offset from again. The two agree
    /// that the addition is checked; they differ on what an overflow means,
    /// because one is reporting a Tick and the other is waiting for one.
    ///
    fn deadline(&self) -> Option<ClockInstant> {
        self.epoch.checked_add(self.scheduled_at)
    }

    fn timing(&self, observed_at: Duration) -> TickTiming {
        TickTiming {
            epoch: self.epoch,
            scheduled_at: self.scheduled_at,
            observed_at,
            period: self.period,
        }
    }

    /// Moves to the deadline after the one handled at `observed_at`, by ADR
    /// 0037's rule.
    fn advance(&mut self, observed_at: Duration) {
        self.due_on_arrival = false;
        self.scheduled_at = next_scheduled_at(self.scheduled_at, observed_at, self.period);
    }
}

///
/// What the engine's task wakes for.
///
enum PlaybackEvent<A: OutputAdapter> {
    Command(PlaybackCommand<A>),
    Deadline,
    /// The next deadline is not an instant this clock can express, so there is
    /// nothing to wait until and the run cannot go on.
    Unschedulable,
    /// Every handle has been dropped, so nothing can ask this engine for
    /// anything ever again.
    Closed,
}

///
/// How many messages a live run answers before its deadline is looked at
/// first.
///
/// The bias exists for a tie, and a queue that is never empty is not a tie: it
/// is a caller holding the deadline arm off for as long as it keeps sending.
/// Nothing in the API makes that hard to do by accident — an idempotent
/// `start` the task discards costs a caller almost nothing to send — and the
/// run does not fail when it happens, it just stops delivering while still
/// publishing `Playing`. Sixty-four is well above any burst the console
/// produces in a frame and far below the number it takes to lose a Tick.
///
const MESSAGES_BEFORE_A_DEADLINE: usize = 64;

///
/// Waits for whichever comes first: a message, or the deadline of the run in
/// progress.
///
/// A message wins a tie, which is the arm that matters for `stop`: a request
/// arriving in the same moment as a deadline must not leave the Tick to be
/// executed by a task that already has the stop in hand.
///
/// Past `MESSAGES_BEFORE_A_DEADLINE` the bias inverts and the deadline is
/// taken first, because a queue that never empties is not a tie. Inverting it
/// costs `stop` nothing: the request is raised before its message is sent, so
/// a Tick that overtakes a queued `Stop` still meets a shut gate and is
/// refused admission.
///
async fn next_playback_event<A: OutputAdapter>(
    commands: &mut mpsc::UnboundedReceiver<PlaybackCommand<A>>,
    clock: Option<&TickClock>,
    messages_since_tick: usize,
) -> PlaybackEvent<A> {
    let Some(clock) = clock else {
        return match commands.recv().await {
            Some(command) => PlaybackEvent::Command(command),
            None => PlaybackEvent::Closed,
        };
    };
    if clock.due_on_arrival {
        // Answered without awaiting, so that the first Tick of a run is
        // executed in the turn the run began in rather than one browser timer
        // later. A message already queued is still taken first, until enough
        // of them have been that the Tick is owed its turn.
        if messages_since_tick >= MESSAGES_BEFORE_A_DEADLINE {
            return PlaybackEvent::Deadline;
        }
        return match commands.try_recv() {
            Ok(command) => PlaybackEvent::Command(command),
            Err(mpsc::error::TryRecvError::Empty) => PlaybackEvent::Deadline,
            Err(mpsc::error::TryRecvError::Disconnected) => PlaybackEvent::Closed,
        };
    }
    let Some(deadline) = clock.deadline() else {
        return PlaybackEvent::Unschedulable;
    };
    if messages_since_tick >= MESSAGES_BEFORE_A_DEADLINE {
        // The bias is given up for one turn, so a deadline already reached is
        // taken ahead of a queue that has had its share. A `stop` waiting
        // behind this Tick is not lost by it: the request is raised before its
        // message is sent, and the gate this Tick has to be admitted through
        // is already shut.
        tokio::select! {
            biased;
            () = sleep_until(deadline) => PlaybackEvent::Deadline,
            command = commands.recv() => match command {
                Some(command) => PlaybackEvent::Command(command),
                None => PlaybackEvent::Closed,
            },
        }
    } else {
        tokio::select! {
            biased;
            command = commands.recv() => match command {
                Some(command) => PlaybackEvent::Command(command),
                None => PlaybackEvent::Closed,
            },
            () = sleep_until(deadline) => PlaybackEvent::Deadline,
        }
    }
}

///
/// The Playback Engine: one task, owning the state and the clock that drives
/// it.
///
/// A Tick is one arm of this loop and the messages are the other, so there is
/// no second party to be stale relative to, nothing asleep that has to be
/// woken, and no other task whose death has to be noticed. A retune recomputes
/// the deadline the loop waits on, and that is the whole of it.
///
async fn run_engine<A: OutputAdapter>(
    mut inner: PlaybackInner<A>,
    mut commands: mpsc::UnboundedReceiver<PlaybackCommand<A>>,
    tick_gate: Arc<TickGate>,
) {
    let mut clock: Option<TickClock> = None;
    let mut messages_since_tick = 0usize;
    loop {
        let event = next_playback_event(&mut commands, clock.as_ref(), messages_since_tick).await;
        if matches!(event, PlaybackEvent::Deadline) {
            messages_since_tick = 0;
        } else {
            messages_since_tick = messages_since_tick.saturating_add(1);
        }
        match event {
            PlaybackEvent::Closed => break,
            PlaybackEvent::Command(PlaybackCommand::Start { tick_period }) => {
                // A start that finds a run already live is that run, not a
                // second one. This is where idempotence is decided, because
                // this is the only place that knows whether a run is live at
                // the moment the start is applied.
                if !inner.is_playing() {
                    inner.begin_run();
                    clock = Some(TickClock::beginning(tick_period));
                }
            }
            PlaybackEvent::Command(PlaybackCommand::Retune { tick_period }) => {
                // Retuning changes the Tick period of the run in progress and
                // does not begin one, so the absolute Tick and the last
                // executed deadline both stay: ADR 0037 runs the new grid from
                // that deadline rather than from the moment the retune arrived.
                if inner.is_playing() {
                    let first_tick_at =
                        first_retuned_tick_at(inner.last_tick_at, ClockInstant::now(), tick_period);
                    clock = Some(TickClock::retuned(first_tick_at, tick_period));
                }
            }
            PlaybackEvent::Command(PlaybackCommand::Stop) => {
                // The request is answered here and nowhere else: it was raised
                // to hold the line until this message arrived, and a run begun
                // after it must not find it standing.
                tick_gate.clear_stop();
                inner.stop();
                clock = None;
            }
            PlaybackEvent::Command(PlaybackCommand::Disconnect) => inner.disconnect(),
            PlaybackEvent::Command(PlaybackCommand::Adapter(transition)) => transition(&mut inner),
            PlaybackEvent::Unschedulable => {
                // `start` and `retune` refuse a period whose deadlines cannot
                // be expressed, so reaching this means a run outlasted its own
                // grid rather than that a caller asked for one. Ending the run
                // with a diagnostic is what the engine does with every other
                // failure it cannot continue through; the alternative here is
                // a deadline of now, which this loop would reach, execute, and
                // arrive back at immediately.
                inner.report(PlaybackDiagnostic::ClockFailure {
                    message: "Playback clock ran past the last instant it can schedule".to_string(),
                });
                inner.stop();
                clock = None;
            }
            PlaybackEvent::Deadline => {
                let running = clock
                    .as_mut()
                    .expect("a deadline is answered only while there is a clock");
                if !tick_gate.begin_tick() {
                    // ADR 0002's synchronous guarantee, asked at the last
                    // moment before the Tick would be executed: a handle that
                    // raised the request before returning from `stop` has
                    // prevented this Tick. The grid goes with it, so nothing
                    // here spins declining deadlines while the message behind
                    // the request makes its way to the arm above.
                    clock = None;
                    continue;
                }
                let observed_at = running.epoch.elapsed();
                inner.execute_tick(running.timing(observed_at));
                // Admission and completion are paired, so a stop raised while
                // the Tick above was executing is still standing for the next
                // deadline to be refused by.
                tick_gate.finish_tick();
                running.advance(observed_at);
            }
        }
    }
    // Every handle is gone, so nothing is left to ask this engine to stop and
    // nothing is left to tell. Stopping here is what makes dropping the last
    // handle safe, structurally rather than arithmetically.
    inner.stop();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{CellIndex, Grid};
    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::atomic::{AtomicBool, Ordering};

    ///
    /// The index `grid` mints for `idx`. A Cell is named by an index its Grid
    /// minted, so a test states the number and the Grid answers with the Cell.
    ///
    fn cell(grid: Grid, idx: usize) -> CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    ///
    /// The Note On an adapter is handed for `channel`, `velocity` and `note`,
    /// stated as the three Numbers a Source writes.
    ///
    fn note_on(channel: u8, velocity: u8, note: u8) -> OutputCommand {
        OutputCommand::NoteOn {
            channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
            velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
            note: Note::try_from(note).expect("a MIDI note"),
        }
    }

    ///
    /// The stop an adapter is handed for `channel` and `note`: MIDI's
    /// zero-velocity Note On, named for what it does rather than what it is.
    ///
    fn stop(channel: u8, note: u8) -> OutputCommand {
        note_on(channel, 0, note)
    }

    ///
    /// One Playback Engine, whose task is spawned on the test's own runtime.
    ///
    /// `new` is fallible because a task needs a runtime to be spawned on, and
    /// every test that builds an engine is running on one; the test that is
    /// about not having one builds its engine deliberately outside it.
    ///
    fn engine<A: OutputAdapter + Send + 'static>(
        source: SourceCommander,
        adapter: A,
    ) -> PlaybackEngine<A> {
        PlaybackEngine::new(source, adapter).expect("the test runtime")
    }

    ///
    /// Runs the engine's task to the end of what the present instant owes it.
    ///
    /// A probe is queued behind whatever the engine has already been sent and
    /// answered from inside its task, so awaiting the answer says the task has
    /// been polled. The turn that answers does not end there: the loop
    /// re-enters its wait, and a deadline already reached completes that wait
    /// without parking, so a Tick due at the present instant is executed
    /// before the task hands the runtime back and this future is polled at
    /// all. What the answer carries is therefore "every Tick this instant owed
    /// has run", which is what these tests assert on and is as true of a
    /// deadline that owed nothing.
    ///
    /// `yield_now` cannot say that. Tokio documents that the runtime may poll
    /// the yielding task again without polling any other, and that a yield
    /// under `select!` — the shape of the engine's wait — may not reach the
    /// executor at all; the order it polls tasks in is not part of its
    /// compatibility promise. A count of yields is a number tuned until the
    /// suite passed, and the negative assertions it stands under — nothing ran
    /// yet — are the ones it cannot support at any count.
    ///
    /// The probe travels as an ordinary `Adapter` transition, which is the
    /// variant `MidiSelectionHandle` sends through in production. Nothing is
    /// staged here that the shipped queue does not already carry.
    ///
    async fn settle<A: OutputAdapter>(engine: &PlaybackEngine<A>) {
        settle_queue(&engine.commands).await;
    }

    /// [`settle`], against the queue rather than a handle holding one, for the
    /// test that owns the two ends separately.
    async fn settle_queue<A: OutputAdapter>(commands: &mpsc::UnboundedSender<PlaybackCommand<A>>) {
        let (probe, answered) = tokio::sync::oneshot::channel();
        commands
            .send(PlaybackCommand::Adapter(Box::new(move |_| {
                let _ = probe.send(());
            })))
            .unwrap_or_else(|_| panic!("the engine's task holds the queue open"));
        answered.await.expect("the engine's task answers its probe");
    }

    ///
    /// One Playback Engine's state, driven by hand.
    ///
    /// The engine's state belongs to a task that also owns the deadlines that
    /// drive it. These tests are about what an executed Tick does — ownership,
    /// counting, delivery, what each lifecycle action clears — and not about
    /// when one is due, so they hold that state directly and spend Ticks on it.
    /// Nothing is staged here that a run cannot reach: the run begins through
    /// the same `begin_run` a `start` message begins one with, and each Tick is
    /// the one the loop would have executed at that deadline.
    ///
    struct HandDrivenRun<A: OutputAdapter> {
        inner: PlaybackInner<A>,
        state: watch::Receiver<PlaybackState>,
        diagnostics: mpsc::UnboundedReceiver<PlaybackDiagnostic>,
    }

    impl<A: OutputAdapter> HandDrivenRun<A> {
        fn new(source: SourceCommander, adapter: A) -> Self {
            let (inner, channels) = PlaybackInner::new(source, adapter);
            Self {
                inner,
                state: channels.state,
                diagnostics: channels.diagnostics,
            }
        }

        ///
        /// Begins a Playback run, exactly as a `start` message does. A test
        /// that began one differently would pin a state no run ever reaches.
        ///
        fn begin_run(&mut self) {
            self.inner.begin_run();
        }

        fn tick(&mut self, timing: TickTiming) -> Option<TickPlan> {
            self.inner.execute_tick(timing)
        }

        ///
        /// Runs the Tick numbered `tick` of the run, on time.
        ///
        /// The engine counts executed Ticks, so a test that runs them in order
        /// names each by its absolute Tick and states the schedule under test
        /// in the same numbers ADR 0016 does.
        ///
        fn run_tick(&mut self, tick: u64) {
            self.tick(scheduled(
                Duration::from_secs(tick),
                Duration::from_secs(tick),
            ))
            .expect("a scheduled Tick runs");
        }

        fn state(&self) -> PlaybackState {
            *self.state.borrow()
        }

        ///
        /// The absolute Tick the next executed Tick will interpret at.
        ///
        fn current_tick(&self) -> Tick {
            self.inner.tick
        }

        ///
        /// Whether this run's note schedule holds anything.
        ///
        fn holds_note_ownership(&self) -> bool {
            self.inner.owned.holds_note_ownership()
        }

        fn drain_diagnostics(&mut self) -> Vec<PlaybackDiagnostic> {
            let mut diagnostics = Vec::new();
            while let Ok(diagnostic) = self.diagnostics.try_recv() {
                diagnostics.push(diagnostic);
            }
            diagnostics
        }
    }

    #[derive(Default)]
    struct RecordingAdapter {
        command_lists: Vec<Vec<OutputCommand>>,
        source: Option<SourceCommander>,
        source_at_submission: Vec<String>,
    }

    impl RecordingAdapter {
        fn observing(source: SourceCommander) -> Self {
            Self {
                source: Some(source),
                ..Self::default()
            }
        }
    }

    impl OutputAdapter for RecordingAdapter {
        fn submit(&mut self, commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
            if let Some(source) = &self.source {
                self.source_at_submission.push(source.snapshot());
            }
            self.command_lists.push(commands.to_vec());
            Ok(())
        }

        fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
            Ok(())
        }
    }

    ///
    /// An adapter that refuses every submission and every safety action with
    /// the same error, as a device that is gone refuses everything sent to it.
    ///
    /// `InMemoryOutputAdapter::fail_next_submission` arms a single refusal,
    /// and the failure a latch de-duplicates is one that keeps happening: a
    /// test of that latch needs an adapter that keeps refusing.
    ///
    struct RefusingOutputAdapter;

    impl RefusingOutputAdapter {
        const ERROR: &'static str = "device lost";
    }

    impl OutputAdapter for RefusingOutputAdapter {
        fn submit(&mut self, _commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
            Err(OutputAdapterError::new(Self::ERROR))
        }

        fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
            Err(OutputAdapterError::new(Self::ERROR))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::{Condvar, mpsc as std_mpsc};
    #[cfg(target_arch = "wasm32")]
    use tokio::time;

    ///
    /// How long a harness wait gives the thing it is waiting for.
    ///
    /// These waits are on another thread — a Tokio worker carrying the
    /// engine's task, or that task's own death — so there is nothing to await
    /// and a poll is what is left. Five seconds is orders of magnitude past
    /// the microseconds each of them takes, and is the budget the drop-thread
    /// `recv_timeout` in this module already uses.
    ///
    /// It is a bound on failure, not a schedule. A wait that reaches it has
    /// found a defect, and reaching it is what turns that defect into a red
    /// test rather than a watchdog kill with no assertion attached to it. The
    /// one-second budget these replace was the tightest timing margin in this
    /// file, and its signature — fails once, passes on the re-run — is the
    /// flake that gets re-run rather than read.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    const HARNESS_TIMEOUT: Duration = Duration::from_secs(5);

    ///
    /// Polls until `answered`, or fails saying what never happened.
    ///
    /// `std::time::Instant` rather than the clock these tests otherwise use,
    /// because the budget is wall-clock patience with another thread and must
    /// not move if anything pauses the runtime's clock.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    async fn wait_until(never_happened: &str, mut answered: impl FnMut() -> bool) {
        let deadline = std::time::Instant::now() + HARNESS_TIMEOUT;
        loop {
            if answered() {
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "{never_happened} within {HARNESS_TIMEOUT:?}"
            );
            time::sleep(Duration::from_millis(1)).await;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[derive(Default)]
    struct BlockingOutputState {
        deliveries: usize,
        release_delivery: bool,
        safety_reset_count: usize,
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[derive(Clone, Default)]
    struct BlockingOutputControl {
        state: Arc<(Mutex<BlockingOutputState>, Condvar)>,
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl BlockingOutputControl {
        ///
        /// Waits on the test's own thread for the engine to reach a delivery.
        ///
        /// Bounded, because an unbounded `wait_while` here is a harness that
        /// fails as a watchdog kill with nothing to read: the release it is
        /// waiting for comes from a Tokio worker, and a worker that never gets
        /// there is exactly the defect worth seeing.
        ///
        fn wait_for_delivery(&self) {
            let (lock, changed) = &*self.state;
            let state = lock.lock().unwrap();
            let (_state, timed_out) = changed
                .wait_timeout_while(state, HARNESS_TIMEOUT, |state| state.deliveries == 0)
                .unwrap();
            assert!(
                !timed_out.timed_out(),
                "the engine never reached a delivery within {HARNESS_TIMEOUT:?}"
            );
        }

        fn release_delivery(&self) {
            let (lock, changed) = &*self.state;
            lock.lock().unwrap().release_delivery = true;
            changed.notify_all();
        }

        fn deliveries(&self) -> usize {
            self.state.0.lock().unwrap().deliveries
        }

        fn safety_reset_count(&self) -> usize {
            self.state.0.lock().unwrap().safety_reset_count
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    struct BlockingOutputAdapter {
        control: BlockingOutputControl,
    }

    #[cfg(not(target_arch = "wasm32"))]
    struct PanickingOutputAdapter {
        delivery_started: Arc<AtomicBool>,
    }

    ///
    /// An adapter that dies on delivery and dies again being silenced, which
    /// is the device a panicking backend actually presents.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    struct DoublyPanickingOutputAdapter {
        delivery_started: Arc<AtomicBool>,
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl OutputAdapter for DoublyPanickingOutputAdapter {
        fn submit(&mut self, _commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
            self.delivery_started.store(true, Ordering::SeqCst);
            panic!("test output panic");
        }

        fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
            panic!("test safety panic");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl OutputAdapter for PanickingOutputAdapter {
        fn submit(&mut self, _commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
            self.delivery_started.store(true, Ordering::SeqCst);
            panic!("test output panic");
        }

        fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
            Ok(())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl OutputAdapter for BlockingOutputAdapter {
        ///
        /// Holds the engine inside a delivery until the test releases it.
        ///
        /// This occupies a Tokio worker for the duration, and it has to: the
        /// seam is a synchronous trait method the engine's task calls inline,
        /// so holding it is the only way to stage "the engine is mid-Tick" —
        /// `spawn_blocking` would move a different call, not this one. Every
        /// test that uses it runs on `worker_threads = 2`, and the other
        /// worker is what carries the test's own future while this one is
        /// held. Two things would break that: a third party wanting a worker
        /// at the same moment, and a second engine held here concurrently.
        /// Neither exists in this module, and a test that adds one owes this
        /// harness another worker.
        ///
        /// The wait is bounded for the reason the control's is.
        ///
        fn submit(&mut self, _commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
            let (lock, changed) = &*self.control.state;
            let mut state = lock.lock().unwrap();
            state.deliveries += 1;
            changed.notify_all();
            let (_state, timed_out) = changed
                .wait_timeout_while(state, HARNESS_TIMEOUT, |state| !state.release_delivery)
                .unwrap();
            assert!(
                !timed_out.timed_out(),
                "the delivery was never released within {HARNESS_TIMEOUT:?}"
            );
            Ok(())
        }

        fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
            self.control.state.0.lock().unwrap().safety_reset_count += 1;
            Ok(())
        }
    }

    fn write(source: &SourceCommander, start: usize, content: &str) {
        let grid = source.grid();
        for (offset, content) in content.chars().enumerate() {
            source
                .set(cell(grid, start + offset), &content.to_string())
                .unwrap();
        }
    }

    ///
    /// A hand-driven run that has executed one Tick of `expression`, so it
    /// owns a note whose stop is due at a Tick it has not reached.
    ///
    /// Taking the Expression rather than naming one spelling is what lets the
    /// lifecycle claims be made of a Timed note and a Mono note alike: the two
    /// share one schedule, and the rule under test is that nothing in it
    /// survives.
    ///
    fn run_owning_a_note(
        expression: &str,
    ) -> (HandDrivenRun<InMemoryOutputAdapter>, InMemoryOutputAdapter) {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, ".=0101");
        write(&source, 20, expression);
        let adapter = InMemoryOutputAdapter::default();
        let mut run = HandDrivenRun::new(source, adapter.clone());
        run.begin_run();
        run.run_tick(0);

        assert!(run.holds_note_ownership());
        (run, adapter)
    }

    ///
    /// A one-second Tick of a hand-driven run, due at `scheduled_at` and seen
    /// at `observed_at`, measured from the instant this timing is built.
    ///
    /// A clock states its deadlines against the epoch it began on, so a
    /// hand-driven Tick states them against the moment the test hands it over.
    /// A test that cares which instant that is takes its own epoch and builds
    /// the timing against it.
    ///
    fn scheduled(scheduled_at: Duration, observed_at: Duration) -> TickTiming {
        scheduled_from(ClockInstant::now(), scheduled_at, observed_at)
    }

    fn scheduled_from(
        epoch: ClockInstant,
        scheduled_at: Duration,
        observed_at: Duration,
    ) -> TickTiming {
        TickTiming {
            epoch,
            scheduled_at,
            observed_at,
            period: Duration::from_secs(1),
        }
    }
    ///
    /// The stall is a whole number of periods plus half of one, so each of the
    /// three policies a clock could hold answers differently: replaying the
    /// backlog would name `2s`, restarting the period from where the clock woke
    /// would name `3.5s`, and ADR 0037's rule names the next Tick still on the
    /// grid. A whole-second stall cannot tell the last two apart, which is why
    /// the case this replaces held for a fortnight while the two targets
    /// disagreed.
    ///
    #[test]
    fn a_missed_deadline_does_not_move_the_grid() {
        assert_eq!(
            super::next_scheduled_at(
                Duration::from_secs(1),
                Duration::from_millis(3_500),
                Duration::from_secs(1),
            ),
            Duration::from_secs(4)
        );
    }

    ///
    /// A Tick observed within its own period is not late, so the deadline after
    /// it is one period on from the deadline it was due at rather than one
    /// period on from when it was seen.
    ///
    #[test]
    fn an_unmissed_deadline_schedules_one_period_past_the_deadline() {
        assert_eq!(
            super::next_scheduled_at(
                Duration::from_secs(1),
                Duration::from_millis(1_900),
                Duration::from_secs(1),
            ),
            Duration::from_secs(2)
        );
    }

    ///
    /// The Tick declined here is the one due at `1s`, which `is_overrun`
    /// refuses because `2s` is a whole period past it. The deadline standing
    /// exactly at `2s` is skipped too, and by this function rather than by the
    /// engine: it has none of its period left ahead of it, so it never reaches
    /// `execute_tick` and leaves no diagnostic of its own. That is the boundary
    /// ADR 0037 draws, and it is the same one `is_overrun` draws with `>=`.
    ///
    #[test]
    fn a_deadline_reached_one_whole_period_late_is_missed() {
        assert_eq!(
            super::next_scheduled_at(
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(1),
            ),
            Duration::from_secs(3)
        );
    }

    ///
    /// A run whose period is zero is refused by `start` and `retune` before a
    /// clock exists, so this states what the arithmetic does rather than a
    /// policy for a run: it answers without dividing by the period.
    ///
    #[test]
    fn a_zero_period_answers_without_dividing_by_it() {
        assert_eq!(
            super::next_scheduled_at(
                Duration::from_secs(1),
                Duration::from_secs(9),
                Duration::ZERO
            ),
            Duration::from_secs(1)
        );
    }

    ///
    /// Both retunes anchor their new grid with this one function, and it
    /// answers with an instant rather than a wait so that neither target can
    /// apply a wait against an epoch it was not measured from. The browser
    /// retune is exercised by `console/tests/wasm.rs`; this native test pins the
    /// arithmetic independently of browser timer jitter.
    ///
    /// The stall is a whole number of periods plus half of one, so the three
    /// candidate rules answer differently: the backlog rule would name `2s`,
    /// rebasing onto the retune instant would name `4.5s`, and ADR 0037's rule
    /// names the grid point still ahead.
    ///
    #[test]
    fn a_retuned_grid_is_anchored_on_a_deadline_not_on_a_wait() {
        let last_tick_at = ClockInstant::now();
        let now = last_tick_at + Duration::from_millis(3_500);

        assert_eq!(
            super::first_retuned_tick_at(Some(last_tick_at), now, Duration::from_secs(1)),
            last_tick_at + Duration::from_secs(4),
            "the first point of the new grid still ahead of the retune"
        );
        assert_eq!(
            super::first_retuned_tick_at(Some(now), now, Duration::from_secs(1)),
            now + Duration::from_secs(1),
            "a Tick due exactly now has none of its period left ahead of it"
        );
        assert_eq!(
            super::first_retuned_tick_at(None, now, Duration::from_secs(1)),
            now,
            "a run with no executed Tick behind it has no grid to keep"
        );
    }

    ///
    /// Drives the real engine loop through a missed deadline and holds it
    /// to a deadline written out here, not to one recomputed by calling the
    /// function under test.
    ///
    /// Each case stalls a whole number of periods plus a fraction of one, so
    /// the phase is never zero and the three candidate rules answer
    /// differently: replaying the backlog delivers more than one Tick, rebasing
    /// the grid onto the wake instant lands at `observed + period`, and ADR
    /// 0037's rule lands on the grid point written in the third column. The
    /// periods at and below five milliseconds are the ones Tokio's missed-tick
    /// machinery cannot express, which is why the loop no longer uses it.
    ///
    /// Both targets run this loop. Browser waiting and the public tempo
    /// change path are also exercised by `console/tests/wasm.rs`.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    async fn assert_native_clock_holds_the_shared_rule(retune: bool) {
        // (tick period, when the stalled clock wakes, the deadline it resumes
        // on) in microseconds, each measured from the run's first deadline.
        const CASES: [(u64, u64, u64); 6] = [
            (1_000, 3_500, 4_000),
            (2_000, 7_000, 8_000),
            (3_000, 10_500, 12_000),
            (4_000, 14_000, 16_000),
            (5_000, 17_500, 20_000),
            (1_000_000, 3_500_000, 4_000_000),
        ];

        for (period_us, observed_us, resumed_us) in CASES {
            let period = Duration::from_micros(period_us);
            let observed = Duration::from_micros(observed_us);
            let resumed = Duration::from_micros(resumed_us);
            let case = format!("period={period:?}, retune={retune}");

            let adapter = InMemoryOutputAdapter::default();
            let engine = engine(SourceCommander::new(Grid::new(1, 1)), adapter.clone());
            engine.start(period).unwrap();
            settle(&engine).await;
            assert_eq!(adapter.command_lists().len(), 1, "first Tick: {case}");
            if retune {
                engine.retune(period).unwrap();
                settle(&engine).await;
                assert_eq!(adapter.command_lists().len(), 1, "retune waits: {case}");
            }

            time::advance(observed).await;
            settle(&engine).await;
            assert_eq!(
                engine.drain_diagnostics(),
                vec![PlaybackDiagnostic::Overrun {
                    scheduled_at: period,
                    observed_at: observed,
                }],
                "one stall is one Overrun: {case}"
            );
            assert_eq!(
                adapter.command_lists().len(),
                1,
                "no backlog executes: {case}"
            );

            time::advance(resumed - observed - Duration::from_micros(1)).await;
            settle(&engine).await;
            assert_eq!(
                adapter.command_lists().len(),
                1,
                "nothing is due before the grid point: {case}"
            );

            time::advance(Duration::from_micros(1)).await;
            settle(&engine).await;
            assert_eq!(
                adapter.command_lists().len(),
                2,
                "the run resumes on the grid it began on: {case}"
            );
            assert!(
                engine.drain_diagnostics().is_empty(),
                "the resumed Tick is on time, so the drain above took the only                  diagnostic there was: {case}"
            );

            engine.stop();
            settle(&engine).await;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(start_paused = true)]
    async fn the_native_start_clock_holds_the_shared_deadline_rule() {
        assert_native_clock_holds_the_shared_rule(false).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(start_paused = true)]
    async fn the_native_retuned_clock_holds_the_shared_deadline_rule() {
        assert_native_clock_holds_the_shared_rule(true).await;
    }

    #[test]
    fn browser_wait_rounds_up_a_fractional_millisecond() {
        for (nanos, millis) in [(1, 1), (999_999, 1), (1_000_000, 1), (1_000_001, 2)] {
            assert_eq!(
                super::wasm_timeout_millis(Duration::from_nanos(nanos)),
                millis
            );
        }
    }

    #[test]
    fn a_tick_commits_source_before_submitting_play_commands() {
        let source = SourceCommander::new(Grid::new(10, 12));
        write(&source, 20, ".+0102");
        write(&source, 80, "!>007FC4");
        write(&source, 60, ".=0101");
        let mut run =
            HandDrivenRun::new(source.clone(), RecordingAdapter::observing(source.clone()));
        run.begin_run();

        let tick = run
            .tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("scheduled Tick runs");

        assert_eq!(&run.inner.adapter.source_at_submission[0][30..32], "03");
        assert_eq!(&source.snapshot()[30..32], "03");
        assert_eq!(tick.play_commands.len(), 1);
        assert_eq!(
            run.inner.adapter.command_lists,
            vec![vec![note_on(0, 0x7F, 60)]]
        );
    }

    #[test]
    fn playback_begins_at_the_first_tick_and_advances_one_per_executed_tick() {
        // ADR 0012's counter in full: the first Tick of a run is absolute Tick
        // `0`, and each executed Tick increments it by exactly one. Reading the
        // counter is the whole of what is observable today — no Function reads
        // the Tick yet — so the count is what is pinned.
        let mut run = HandDrivenRun::new(
            SourceCommander::new(Grid::new(10, 9)),
            InMemoryOutputAdapter::default(),
        );
        run.begin_run();

        assert_eq!(run.current_tick(), Tick::ZERO);

        for executed in 1..=4u64 {
            run.run_tick(executed - 1);

            assert_eq!(run.current_tick(), Tick::new(executed));
        }
    }

    #[test]
    fn an_overrun_consumes_no_absolute_tick() {
        // The counter counts executed Ticks, not deadlines: a Tick declined as
        // an Overrun planned nothing, so there is no Tick for it to have been.
        // It is the one way an executed run declines a deadline it reached —
        // the clock belongs to the task that owns this state and exists only
        // while a run does, so there is no stopped engine to hand a Tick to and
        // no retired clock for one to arrive from.
        let mut run = HandDrivenRun::new(
            SourceCommander::new(Grid::new(10, 9)),
            InMemoryOutputAdapter::default(),
        );
        run.begin_run();
        run.run_tick(0);
        assert_eq!(run.current_tick(), Tick::new(1));

        assert!(
            run.tick(scheduled(Duration::from_secs(1), Duration::from_secs(5)))
                .is_none()
        );

        assert_eq!(run.current_tick(), Tick::new(1));
        assert_eq!(
            run.drain_diagnostics(),
            vec![PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_secs(1),
                observed_at: Duration::from_secs(5),
            }]
        );
    }

    #[test]
    fn each_playback_run_begins_again_at_the_first_tick() {
        // ADR 0012's first-Tick rule is about a Playback run, not about the
        // lifetime of the engine: a run that is stopped and started again is a
        // new run and counts from `0` again.
        let mut run = HandDrivenRun::new(
            SourceCommander::new(Grid::new(10, 9)),
            InMemoryOutputAdapter::default(),
        );

        run.begin_run();
        run.run_tick(0);
        assert_eq!(run.current_tick(), Tick::new(1));

        run.inner.stop();
        assert_eq!(run.state(), PlaybackState::Stopped);
        run.begin_run();

        assert_eq!(run.current_tick(), Tick::ZERO);

        // And the new run counts from there, rather than resuming the old one.
        run.run_tick(0);
        assert_eq!(run.current_tick(), Tick::new(1));
    }

    #[test]
    fn beginning_a_run_discards_the_previous_runs_absolute_tick() {
        // ADR 0012's first-Tick rule belongs to beginning a Playback run, not
        // to the clock that happens to drive it, so every path that begins one
        // opens the same way. The engine is carried far enough into a first run
        // that a counter left standing would be plainly visible, and the run
        // begun after it must still open at absolute Tick `0` with no last Tick
        // behind it for the clock to schedule against.
        let mut run = HandDrivenRun::new(
            SourceCommander::new(Grid::new(10, 9)),
            InMemoryOutputAdapter::default(),
        );
        run.begin_run();

        for executed in 0..3u64 {
            run.run_tick(executed);
        }
        assert_eq!(run.current_tick(), Tick::new(3));

        run.begin_run();

        assert_eq!(run.current_tick(), Tick::ZERO);
        assert!(run.inner.last_tick_at.is_none());
    }

    #[test]
    fn live_editing_changes_the_next_unsampled_tick() {
        let source = SourceCommander::new(Grid::new(10, 9));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
        run.begin_run();

        run.run_tick(0);
        source.set(cell(source.grid(), 26), "D").unwrap();
        run.run_tick(1);

        // ADR 0012's other half of Live Editing: the edit lands in the next
        // Source Snapshot because a Snapshot is taken per Tick, and the run
        // keeps counting, because editing the Source is not starting a
        // Playback run.
        assert_eq!(run.current_tick(), Tick::new(2));
        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.command_lists()[0][0], note_on(0, 0x7F, 60));
        assert_eq!(adapter.command_lists()[1][0], note_on(0, 0x7F, 62));
    }

    #[test]
    fn repeated_commands_are_dispatched_as_exact_tick_lists() {
        let source = SourceCommander::new(Grid::new(10, 9));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let mut run = HandDrivenRun::new(source, adapter.clone());
        run.begin_run();

        run.run_tick(0);
        run.run_tick(1);

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.command_lists()[0].len(), 1);
        assert_eq!(adapter.command_lists()[0], adapter.command_lists()[1]);
    }

    #[test]
    fn an_inactive_terminal_root_reaches_the_output_adapter_as_an_empty_command_list() {
        let source = SourceCommander::new(Grid::new(10, 9));
        // The Raw Play has no Bang anywhere in the Source, so nothing
        // activates its root.
        write(&source, 20, "!>007FC4");
        let adapter = InMemoryOutputAdapter::default();
        let mut run = HandDrivenRun::new(source, adapter.clone());
        run.begin_run();

        run.run_tick(0);

        // The engine submits once for every Tick it runs, so the proof is not
        // a missing submission but an empty one: the Tick reached the adapter
        // and carried no command.
        assert_eq!(adapter.command_lists(), vec![Vec::<OutputCommand>::new()]);
    }

    #[test]
    fn two_active_terminal_roots_dispatch_in_tick_plan_order_within_one_submission() {
        let source = SourceCommander::new(Grid::new(10, 12));
        // Each comparison emits a fresh Bang one row above its terminal root.
        write(&source, 20, "!>0001C4");
        write(&source, 0, ".=0101");
        write(&source, 60, ".=0101");
        write(&source, 80, "!>017FA4");
        let adapter = InMemoryOutputAdapter::default();
        let mut run = HandDrivenRun::new(source, adapter.clone());
        run.begin_run();

        run.run_tick(0);

        assert_eq!(
            adapter.command_lists(),
            vec![vec![note_on(0, 1, 60), note_on(1, 0x7F, 69)]]
        );
    }

    #[test]
    fn a_refused_submission_leaves_the_schedule_standing_for_the_next_tick() {
        // Both owning spellings, for the reason the lifecycle test loops them:
        // the retry is a property of the one schedule they share, and a Tick
        // resolved against a copy adopts or discards every claim in it at
        // once. Asserting it of `!~` alone would leave the claim CONTEXT.md
        // makes the same promise to untested.
        for expression in ["!~007FC402", "!%007FC402"] {
            let source = SourceCommander::new(Grid::new(10, 9));
            write(&source, 20, expression);
            write(&source, 0, ".=0101");
            let adapter = InMemoryOutputAdapter::default();
            let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
            run.begin_run();

            run.run_tick(0);
            write(&source, 0, ".=0102");
            run.run_tick(1);
            // The adapter refuses the Tick the stop is due at. The schedule
            // describes what is sounding, so a stop no device received leaves
            // the note it stops owned: an adapter that survives a refusal is
            // one this engine still owes a Note Off.
            adapter.fail_next_submission("output unavailable");
            run.run_tick(2);
            assert!(run.holds_note_ownership(), "{expression}");
            run.run_tick(3);

            // Three submissions were accepted: the start, the Tick between,
            // and the stop the next executed Tick drains again.
            assert_eq!(
                adapter.command_lists(),
                vec![vec![note_on(0, 0x7F, 60)], vec![], vec![stop(0, 60)]],
                "{expression}"
            );
            assert!(!run.holds_note_ownership(), "{expression}");
            assert_eq!(
                run.drain_diagnostics(),
                vec![PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(
                    "output unavailable"
                ))],
                "{expression}"
            );
        }
    }

    #[test]
    fn a_scheduled_stop_is_due_at_an_absolute_tick_rather_than_at_a_clock_tick() {
        let source = SourceCommander::new(Grid::new(10, 9));
        write(&source, 20, "!~007FC402");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
        run.begin_run();

        run.run_tick(0);
        write(&source, 0, ".=0102");
        // A Tick the engine declines consumes no absolute Tick, so it moves
        // nothing towards the stop either: the note lasts the two Ticks it
        // was given however many deadlines pass.
        assert!(
            run.tick(scheduled(Duration::from_secs(1), Duration::from_secs(5)))
                .is_none()
        );
        run.run_tick(1);
        run.run_tick(2);

        assert_eq!(
            adapter.command_lists(),
            vec![vec![note_on(0, 0x7F, 60)], vec![], vec![stop(0, 60)]]
        );
        assert_eq!(
            run.drain_diagnostics(),
            vec![PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_secs(1),
                observed_at: Duration::from_secs(5),
            }]
        );
    }

    #[test]
    fn every_lifecycle_action_that_silences_output_clears_the_note_schedule() {
        // Each of these silences the output the schedule describes, so a stop
        // left standing would be delivered to a device that has already been
        // told to stop everything, or into a run that never started the note.
        // A Timed claim and a Mono claim are both asserted of every action,
        // because the two are one schedule and a clear that reached only one
        // of them would leave the other hanging.
        for expression in ["!~007FC40A", "!%007FC40A"] {
            let (mut run, _) = run_owning_a_note(expression);
            run.inner.stop();
            assert!(!run.holds_note_ownership(), "{expression}");

            let (mut run, _) = run_owning_a_note(expression);
            run.inner.disconnect();
            assert!(!run.holds_note_ownership(), "{expression}");

            // Beginning a run restarts the absolute Tick at zero, so an
            // inherited stop would come due before the note it stops had been
            // played.
            let (mut run, _) = run_owning_a_note(expression);
            run.begin_run();
            assert!(!run.holds_note_ownership(), "{expression}");

            // The state ending is what silences the device, and stopping is
            // what clears the schedule. What is left to observe once the state
            // is gone is the safety the owned note is silenced by.
            let (run, adapter) = run_owning_a_note(expression);
            drop(run);
            assert_eq!(adapter.safety_reset_count(), 1, "{expression}");
            assert_eq!(
                adapter.command_lists(),
                vec![vec![note_on(0, 0x7F, 60)]],
                "{expression}"
            );
        }
    }

    #[test]
    fn missed_deadline_is_dropped_and_the_next_scheduled_tick_runs() {
        let source = SourceCommander::new(Grid::new(10, 9));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let mut run = HandDrivenRun::new(source, adapter.clone());
        run.begin_run();

        let missed = run.tick(scheduled(Duration::from_secs(1), Duration::from_secs(2)));
        let resumed = run.tick(scheduled(Duration::from_secs(3), Duration::from_secs(3)));

        assert!(missed.is_none());
        assert!(resumed.is_some());
        assert_eq!(adapter.command_lists().len(), 1);
        assert_eq!(
            run.drain_diagnostics(),
            vec![PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_secs(1),
                observed_at: Duration::from_secs(2),
            }]
        );
    }

    ///
    /// A stall costs the Ticks it covered and nothing after them.
    ///
    /// Three deadlines pass while the engine is away, and ADR 0037 gives all
    /// three the same answer: one Tick is declined with one diagnostic naming
    /// the deadline it was due at, the two the loop never reached are not
    /// manufactured to be declined in turn, and the run resumes on the grid it
    /// began on rather than on a grid rebased onto the moment it woke. The
    /// stall runs half a period past a whole one so that all three candidate
    /// rules answer differently: replaying the backlog sounds the `3s` deadline
    /// at `3.5s` behind two declines, rebasing the grid puts the next Tick at
    /// `4.5s`, and this rule puts it at `4s` with one decline. The clock is
    /// paused rather than slept against, so the stall costs the suite
    /// nothing.
    ///
    #[tokio::test(start_paused = true)]
    async fn a_late_tick_loses_its_turn_and_the_run_resumes_on_the_grid() {
        let source = SourceCommander::new(Grid::new(10, 9));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(source, adapter.clone());
        engine.start(Duration::from_secs(1)).unwrap();

        settle(&engine).await;
        assert_eq!(
            adapter.command_lists().len(),
            1,
            "the first Tick is immediate"
        );

        time::advance(Duration::from_millis(3_500)).await;
        settle(&engine).await;

        assert_eq!(
            adapter.command_lists().len(),
            1,
            "the deadlines at 1s, 2s and 3s deliver nothing"
        );
        assert_eq!(
            engine.drain_diagnostics(),
            vec![PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_secs(1),
                observed_at: Duration::from_millis(3_500),
            }],
            "one stall is one Overrun, named for the deadline it missed"
        );

        time::advance(Duration::from_millis(500)).await;
        settle(&engine).await;

        assert_eq!(
            adapter.command_lists().len(),
            2,
            "4s is on the grid the run began on, so the Tick due there runs"
        );
        assert!(
            engine.drain_diagnostics().is_empty(),
            "the Tick on the grid is on time, so it adds nothing to the stall \
             the drain above took"
        );

        engine.stop();
        settle(&engine).await;

        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(adapter.safety_reset_count(), 1);
    }

    ///
    /// A stopped run does not keep Ticking, and the run started after it does.
    ///
    /// The stop and the start are queued together, so the engine applies them
    /// in the order they were asked for: what would once have been a retired
    /// clock reaching into a restarted run is now a message the one task has
    /// already handled.
    ///
    #[tokio::test(start_paused = true)]
    async fn a_stopped_run_does_not_tick_and_a_restarted_one_does() {
        let source = SourceCommander::new(Grid::new(10, 9));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(source, adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;
        assert_eq!(adapter.command_lists().len(), 1);

        engine.stop();
        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;

        assert_eq!(engine.state(), PlaybackState::Playing);
        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.safety_reset_count(), 1);

        engine.stop();
        settle(&engine).await;
        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(adapter.safety_reset_count(), 2);
    }

    ///
    /// A message already queued when a run begins is applied before that run's
    /// first Tick, not after it.
    ///
    /// The first Tick of a run is due at the instant the run began, and
    /// `next_playback_event` answers it without awaiting so that the browser
    /// gets it in the turn the run started in rather than a timer hop later.
    /// What that shortcut must not do is jump the queue: a `Disconnect` or a
    /// destination change that arrived before the run and is applied after its
    /// first Tick delivers that Tick to an output the user has already left,
    /// and nothing reports it — no diagnostic, no state change, a note on the
    /// wrong device.
    ///
    /// This pins the deterministic half. Both messages are queued before the
    /// task is polled at all, so the shortcut's `try_recv` is what has to take
    /// the disconnect, and no tie is involved. The `biased;` in the select
    /// below it governs the other half — a message and a deadline becoming
    /// ready together — and this test says nothing about that one: without
    /// `biased;` the poll order is randomised, so a green run there would be
    /// evidence and not proof.
    ///
    #[tokio::test(start_paused = true)]
    async fn a_message_queued_before_a_run_begins_is_applied_to_its_first_tick() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(source, adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        engine.disconnect();
        settle(&engine).await;

        assert!(
            adapter.command_lists().is_empty(),
            "the first Tick was delivered to an output the queue had already \
             closed behind it"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn retuning_a_restart_does_not_inherit_the_previous_runs_phase() {
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(SourceCommander::new(Grid::new(1, 1)), adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;
        engine.stop();

        engine.start(Duration::from_secs(1)).unwrap();
        engine.retune(Duration::from_secs(1)).unwrap();
        settle(&engine).await;

        assert_eq!(adapter.command_lists().len(), 2);
    }

    ///
    /// A retune begins a new grid, and it anchors that grid on the deadline the
    /// last executed Tick was due at rather than on the instant the engine
    /// happened to wake. ADR 0037 rejects a grid rebased onto a wake instant
    /// because the shift is permanent; a retune that clamped its first deadline
    /// to the present would reintroduce exactly that shift, on the same run and
    /// the same absolute Tick, every time the tempo moved after a stall.
    ///
    /// It is also what holds a retune to changing the period of the run it is
    /// in rather than beginning one: a retune that began a run would clear the
    /// last executed deadline, and a grid with nothing to anchor on starts at
    /// the retune instant, which is the answer this refuses.
    ///
    #[tokio::test(start_paused = true)]
    async fn retuning_after_a_stall_anchors_on_the_grid_not_the_wake_instant() {
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(SourceCommander::new(Grid::new(1, 1)), adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;
        assert_eq!(adapter.command_lists().len(), 1);

        // The deadline at 1s is missed and the engine wakes half a period past
        // 3s, so the grid the run began on next comes due at 4s.
        time::advance(Duration::from_millis(3_500)).await;
        settle(&engine).await;
        assert_eq!(adapter.command_lists().len(), 1);

        engine.retune(Duration::from_secs(1)).unwrap();
        settle(&engine).await;
        assert_eq!(
            adapter.command_lists().len(),
            1,
            "a retune does not deliver a Tick at the instant it was asked for"
        );

        time::advance(Duration::from_millis(499)).await;
        settle(&engine).await;
        assert_eq!(
            adapter.command_lists().len(),
            1,
            "3.999s is not on the grid"
        );

        time::advance(Duration::from_millis(1)).await;
        settle(&engine).await;
        assert_eq!(
            adapter.command_lists().len(),
            2,
            "4s is on the grid the run began on"
        );

        engine.stop();
        settle(&engine).await;
    }

    ///
    /// A Tick observed late but inside its own period is executed, and the
    /// grid a later retune runs from is the deadline that Tick was due at, not
    /// the instant it was seen. Anchoring on the observation would carry every
    /// ordinary slow Tick into the next grid as a permanent offset, which is
    /// the failure ADR 0037 rejects arriving by a quieter route than a stall.
    ///
    #[tokio::test(start_paused = true)]
    async fn retuning_anchors_on_the_deadline_a_late_tick_was_due_at() {
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(SourceCommander::new(Grid::new(1, 1)), adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;
        assert_eq!(adapter.command_lists().len(), 1);

        // The Tick due at 1s is seen at 1.4s: late, but inside its own period,
        // so it executes rather than being declined.
        time::advance(Duration::from_millis(1_400)).await;
        settle(&engine).await;
        assert_eq!(adapter.command_lists().len(), 2);
        assert!(
            engine.drain_diagnostics().is_empty(),
            "a slow Tick is not an Overrun"
        );

        time::advance(Duration::from_millis(100)).await;
        engine.retune(Duration::from_secs(1)).unwrap();
        settle(&engine).await;

        time::advance(Duration::from_millis(499)).await;
        settle(&engine).await;
        assert_eq!(
            adapter.command_lists().len(),
            2,
            "1.999s is not on the grid"
        );

        time::advance(Duration::from_millis(1)).await;
        settle(&engine).await;
        assert_eq!(
            adapter.command_lists().len(),
            3,
            "2s is one period past the deadline the late Tick was due at"
        );

        engine.stop();
        settle(&engine).await;
    }

    ///
    /// A retune keeps the absolute Tick of the run it retunes.
    ///
    /// `begin_run` resets the counter because ADR 0012 makes the absolute Tick
    /// an interpretation input and a run must open at Tick `0`; retuning
    /// changes the Tick period of the run already in progress and does not
    /// begin one, so resetting there would silently restart every Tick-reading
    /// Function's cycle each time the tempo moved.
    ///
    /// The counter is asserted through what it is an input to rather than read
    /// out of the state that owns it. Delay `~*0102` pulses on the Ticks that
    /// divide its cycle of two, so the note it Bangs on Tick `0` and withholds
    /// on Tick `1` is the counter, stated in the Source the rule is about.
    ///
    #[tokio::test(start_paused = true)]
    async fn retuning_keeps_the_absolute_tick_of_the_run_it_retunes() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "~*0102");
        write(&source, 20, "!>007FC4");
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(source, adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;
        assert_eq!(
            adapter.command_lists(),
            vec![vec![note_on(0, 0x7F, 60)]],
            "Tick 0 divides the Delay's cycle"
        );

        // Anchored on the deadline Tick 0 was due at, so the Tick after the
        // retune is due one retuned period later.
        engine.retune(Duration::from_secs(2)).unwrap();
        settle(&engine).await;
        time::advance(Duration::from_secs(2)).await;
        settle(&engine).await;

        assert_eq!(
            adapter.command_lists(),
            vec![vec![note_on(0, 0x7F, 60)], vec![]],
            "the Tick after a retune is Tick 1, which the Delay does not pulse on"
        );

        engine.stop();
        settle(&engine).await;
    }

    ///
    /// The deadline a Tick is recorded against is the one it was due at, not
    /// the moment the engine got to it.
    ///
    /// A Tick may be observed late and still execute — anything inside its own
    /// period does — and the next retune anchors its grid on what was recorded
    /// here. Recording the observation would carry every ordinary slow Tick
    /// into the next grid as a permanent offset, which is the failure ADR 0037
    /// rejects arriving by a quieter route than a stall.
    ///
    #[test]
    fn the_deadline_recorded_for_a_tick_is_the_one_it_was_due_at() {
        let mut run = HandDrivenRun::new(
            SourceCommander::new(Grid::new(1, 1)),
            InMemoryOutputAdapter::default(),
        );
        run.begin_run();
        let epoch = ClockInstant::now();

        // The Tick due at 1s is seen at 1.4s: late, but inside its own period.
        run.tick(scheduled_from(
            epoch,
            Duration::from_secs(1),
            Duration::from_millis(1_400),
        ))
        .expect("a late Tick inside its own period runs");

        assert_eq!(
            run.inner.last_tick_at,
            epoch.checked_add(Duration::from_secs(1)),
            "the grid point the Tick was due at, not the point it was seen at"
        );
    }

    #[test]
    fn adapter_failure_does_not_roll_back_source_or_stop_playback() {
        let source = SourceCommander::new(Grid::new(10, 12));
        write(&source, 20, ".+0102");
        write(&source, 80, "!>007FC4");
        write(&source, 60, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        adapter.fail_next_submission("output unavailable");
        let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
        run.begin_run();

        let failed_dispatch = run
            .tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("Source Tick still succeeds");

        assert_eq!(&source.snapshot()[30..32], "03");
        assert_eq!(failed_dispatch.play_commands.len(), 1);
        assert_eq!(run.state(), PlaybackState::Playing);
        assert_eq!(
            run.drain_diagnostics(),
            vec![PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(
                "output unavailable"
            ))]
        );

        run.run_tick(1);
        assert_eq!(adapter.command_lists(), vec![vec![note_on(0, 0x7F, 60)]]);
    }

    #[test]
    fn a_run_that_begins_after_a_failed_run_reports_the_failure_again() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let mut run = HandDrivenRun::new(source, RefusingOutputAdapter);

        // Two runs, each of two Ticks the adapter refuses identically, with
        // the safety action `stop` sends refused the same way between them.
        run.begin_run();
        run.run_tick(0);
        run.run_tick(1);
        run.inner.stop();
        run.begin_run();
        run.run_tick(2);
        run.run_tick(3);

        // One report per run: not one per Tick, which is the de-duplication
        // the latch exists for, and not one per adapter lifetime, which would
        // leave the second run silent about a device that is still failing.
        // `stop`'s own refusal is not a third report — it is the tail of the
        // run whose failure has already been reported.
        assert_eq!(
            run.drain_diagnostics(),
            vec![
                PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(
                    RefusingOutputAdapter::ERROR
                )),
                PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(
                    RefusingOutputAdapter::ERROR
                )),
            ]
        );
        run.inner.stop();
    }

    #[tokio::test]
    async fn stopping_and_disconnecting_each_send_the_safety_action() {
        let stopped_adapter = InMemoryOutputAdapter::default();
        let stopped = engine(
            SourceCommander::new(Grid::new(10, 6)),
            stopped_adapter.clone(),
        );
        stopped.start(Duration::from_secs(1)).unwrap();
        stopped.stop();

        let disconnected_adapter = InMemoryOutputAdapter::default();
        let disconnected = engine(
            SourceCommander::new(Grid::new(10, 6)),
            disconnected_adapter.clone(),
        );
        disconnected.start(Duration::from_secs(1)).unwrap();
        disconnected.disconnect();

        settle(&stopped).await;
        settle(&disconnected).await;

        assert_eq!(stopped_adapter.safety_reset_count(), 1);
        assert_eq!(disconnected_adapter.safety_reset_count(), 1);
        assert_eq!(stopped.state(), PlaybackState::Stopped);
    }

    ///
    /// An engine is its task, so a runtime to spawn that task on is what
    /// constructing one requires — and is refused at construction rather than
    /// at the first `start`, because there is nothing to start without it.
    ///
    #[test]
    fn an_engine_cannot_be_constructed_without_a_runtime() {
        let constructed = PlaybackEngine::new(
            SourceCommander::new(Grid::new(1, 1)),
            InMemoryOutputAdapter::default(),
        );

        assert!(matches!(
            constructed,
            Err(PlaybackStartError::RuntimeUnavailable)
        ));
    }

    #[tokio::test]
    async fn a_zero_tick_period_is_refused_and_reported_without_changing_state() {
        let engine = engine(
            SourceCommander::new(Grid::new(1, 1)),
            InMemoryOutputAdapter::default(),
        );

        assert_eq!(
            engine.start(Duration::ZERO),
            Err(PlaybackStartError::ZeroTickPeriod)
        );
        settle(&engine).await;

        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(
            engine.drain_diagnostics(),
            vec![PlaybackDiagnostic::StartFailure {
                message: "Tick period must be greater than zero".to_string(),
            }]
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn unexpected_engine_termination_stops_playback_and_reports_failure() {
        let adapter = InMemoryOutputAdapter::default();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let engine = runtime.block_on(async {
            let engine = engine(SourceCommander::new(Grid::new(1, 1)), adapter.clone());
            engine.start(Duration::from_secs(1)).unwrap();
            settle(&engine).await;
            engine
        });

        drop(runtime);

        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(
            engine.drain_diagnostics(),
            vec![PlaybackDiagnostic::ClockFailure {
                message: "Playback clock terminated unexpectedly".to_string(),
            }]
        );
        assert_eq!(adapter.safety_reset_count(), 1);
    }

    ///
    /// Diagnostics are one ordered stream, and each is delivered exactly once.
    ///
    /// The console reads them inside a frame, so what a user is told rests on
    /// two properties: a diagnostic arrives after the one recorded before it,
    /// whichever part of the engine recorded either, and a drain takes it away
    /// rather than leaving it to be shown again on the next frame. Stated over
    /// three recording sites — a refused start, which the handle reports, and a
    /// refused submission and a Tick declined as an Overrun, which the task
    /// does — because ordering within one site is the weaker claim: the stream
    /// is one queue that every site writes into, and here two different writers
    /// write into it.
    ///
    #[tokio::test(start_paused = true)]
    async fn diagnostics_drain_in_order_and_exactly_once() {
        let adapter = InMemoryOutputAdapter::default();
        adapter.fail_next_submission("device lost");
        let engine = engine(SourceCommander::new(Grid::new(1, 1)), adapter.clone());

        assert!(engine.start(Duration::ZERO).is_err());
        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;
        time::advance(Duration::from_millis(3_500)).await;
        settle(&engine).await;

        assert_eq!(
            engine.drain_diagnostics(),
            vec![
                PlaybackDiagnostic::StartFailure {
                    message: "Tick period must be greater than zero".to_string(),
                },
                PlaybackDiagnostic::OutputFailure(OutputAdapterError::new("device lost")),
                PlaybackDiagnostic::Overrun {
                    scheduled_at: Duration::from_secs(1),
                    observed_at: Duration::from_millis(3_500),
                },
            ],
            "the drain answers in the order the engine recorded them"
        );
        assert!(
            engine.drain_diagnostics().is_empty(),
            "a diagnostic already drained is not delivered a second time"
        );

        time::advance(Duration::from_millis(3_500)).await;
        settle(&engine).await;

        assert_eq!(
            engine.drain_diagnostics(),
            vec![PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_secs(4),
                observed_at: Duration::from_secs(7),
            }],
            "a later drain carries what was recorded after the last one, and only that"
        );
        engine.stop();
    }

    ///
    /// A diagnostic the task recorded is drained before one the handle
    /// recorded after it.
    ///
    /// The sibling above states the same guarantee in the other direction, and
    /// the two are not the same claim. Both of its cases open with the
    /// handle's `StartFailure`, so a build that gave the handle its own buffer
    /// and drained that buffer first would reproduce the order they assert by
    /// accident and stay green. Only a case where the task writes first can
    /// tell a shared ordered queue apart from two queues read handle-first,
    /// and that is the case with the user-visible answer: a device that failed
    /// mid-run, followed by a start the handle refused, has to reach the
    /// status line in the order the two things happened.
    ///
    #[tokio::test(start_paused = true)]
    async fn a_task_recorded_diagnostic_drains_before_a_handle_recorded_one() {
        let adapter = InMemoryOutputAdapter::default();
        adapter.fail_next_submission("device lost");
        let engine = engine(SourceCommander::new(Grid::new(1, 1)), adapter.clone());

        // The task writes first: the run's immediate first Tick is refused by
        // the device.
        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;

        // Then the handle writes, on the same queue, without reaching the task
        // at all.
        assert!(engine.start(Duration::ZERO).is_err());

        assert_eq!(
            engine.drain_diagnostics(),
            vec![
                PlaybackDiagnostic::OutputFailure(OutputAdapterError::new("device lost")),
                PlaybackDiagnostic::StartFailure {
                    message: "Tick period must be greater than zero".to_string(),
                },
            ],
            "the drain answers in the order the two writers recorded them"
        );

        engine.stop();
    }

    #[tokio::test(start_paused = true)]
    async fn start_is_idempotent_and_draining_takes_the_diagnostics() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        adapter.fail_next_submission("device lost");
        let engine = engine(source, adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        engine.start(Duration::from_secs(9)).unwrap();
        assert_eq!(
            engine.start(Duration::ZERO),
            Err(PlaybackStartError::ZeroTickPeriod)
        );
        settle(&engine).await;

        assert_eq!(adapter.command_lists().len(), 0);
        assert_eq!(engine.state(), PlaybackState::Playing);
        assert_eq!(
            engine.drain_diagnostics(),
            vec![
                PlaybackDiagnostic::StartFailure {
                    message: "Tick period must be greater than zero".to_string(),
                },
                PlaybackDiagnostic::OutputFailure(OutputAdapterError::new("device lost")),
            ]
        );
        assert!(engine.drain_diagnostics().is_empty());

        // A start that finds the run already live is that run. The engine has
        // executed the first Tick of it by now, so a start that began a second
        // run would execute that run's own first Tick immediately and deliver
        // where nothing is due.
        let delivered = adapter.command_lists().len();
        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;

        assert_eq!(
            adapter.command_lists().len(),
            delivered,
            "a start into a live run began a second one"
        );
        assert!(
            engine.drain_diagnostics().is_empty(),
            "a start into a live run reported the failure of the run it did not begin"
        );
        engine.stop();
    }

    ///
    /// ADR 0002 requires that further Ticks are prevented before `stop`
    /// returns, and a handle that only queued a message would not do that: the
    /// task may be anywhere, including at a deadline it is about to execute.
    /// The request is what closes that window, so this raises the request
    /// alone — with no message behind it — and holds the engine to it across
    /// several deadlines it would otherwise have executed.
    ///
    /// The request is raised by the shipped `stop`, on a handle built from the
    /// engine's own parts so that this test holds the queue between the handle
    /// and the loop. That is what makes the window reachable: over the queue
    /// `PlaybackEngine::new` wires, the two halves of `stop` are inseparable
    /// and the message is already waiting by the time any deadline comes due,
    /// so every deadline is superseded by the message and the request declines
    /// none of them — which proves the queue rather than the guarantee, and is
    /// why deleting the request from `stop` left this module's tests passing.
    /// Nothing is staged here that the shipped path does not do: the handle is
    /// the handle, `stop` is `stop`, and only the moment the message lands is
    /// the test's.
    ///
    /// Native only, because spawning the loop as a task of its own asks for a
    /// `Send` future and the browser's own spawn does not provide one. The rule
    /// is the loop's and is the same on both targets; `console/tests/wasm.rs`
    /// is where the browser's clock is exercised.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(start_paused = true)]
    async fn a_requested_stop_prevents_a_tick_before_the_message_arrives() {
        let adapter = InMemoryOutputAdapter::default();
        let destinations = adapter.published_destinations();
        let (inner, channels) =
            PlaybackInner::new(SourceCommander::new(Grid::new(1, 1)), adapter.clone());
        let (commands, queued) = mpsc::unbounded_channel();
        let tick_gate = Arc::new(TickGate::new());
        let task = tokio::spawn(run_engine(inner, queued, Arc::clone(&tick_gate)));

        // The handle the stop is asked of: the loop's own request flag, the
        // loop's own published state and diagnostics, and a queue that ends
        // here. `in_flight` is where the message waits, so the request reaches
        // the loop and the message it travels ahead of does not.
        let (undelivered, _in_flight) =
            mpsc::unbounded_channel::<PlaybackCommand<InMemoryOutputAdapter>>();
        let engine = PlaybackEngine {
            commands: undelivered,
            tick_gate: Arc::clone(&tick_gate),
            state: channels.state,
            diagnostics: Arc::new(Mutex::new(channels.diagnostics)),
            reports: channels.reports,
            destinations,
        };

        commands
            .send(PlaybackCommand::Start {
                tick_period: Duration::from_secs(1),
            })
            .unwrap();
        tokio::task::yield_now().await;
        assert_eq!(
            adapter.command_lists().len(),
            1,
            "the first Tick is immediate"
        );

        engine.stop();
        // One period at a time, so every one of these deadlines is reached on
        // time and would be executed rather than declined as an Overrun. What
        // declines them is the request.
        for _ in 0..5 {
            time::advance(Duration::from_secs(1)).await;
            for _ in 0..4 {
                tokio::task::yield_now().await;
            }
        }

        assert_eq!(
            adapter.command_lists().len(),
            1,
            "five deadlines came due on time after the stop was requested"
        );
        assert!(
            engine.drain_diagnostics().is_empty(),
            "a Tick the request declined is not a Tick the grid missed"
        );

        drop(commands);
        task.await.unwrap();
        assert_eq!(adapter.safety_reset_count(), 1);
    }

    ///
    /// Closing the queue is an ordinary shutdown, and an ordinary shutdown is
    /// not a failure.
    ///
    /// The run is ended on the way out, so the state that is then dropped is
    /// already stopped and has nothing to report. A shutdown that left the run
    /// standing would reach the same safety action through the drop — and would
    /// tell the user their engine had terminated unexpectedly, which is what
    /// that report is for and is not what happened.
    ///
    /// Native only, for the reason the stop-request test above is: spawning the
    /// loop as a task of its own asks for a `Send` future.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(start_paused = true)]
    async fn an_orderly_shutdown_silences_the_output_without_reporting_a_failure() {
        let adapter = InMemoryOutputAdapter::default();
        let (inner, mut channels) =
            PlaybackInner::new(SourceCommander::new(Grid::new(1, 1)), adapter.clone());
        let (commands, queued) = mpsc::unbounded_channel();
        let task = tokio::spawn(run_engine(inner, queued, Arc::new(TickGate::new())));

        commands
            .send(PlaybackCommand::Start {
                tick_period: Duration::from_secs(1),
            })
            .unwrap();
        settle_queue(&commands).await;

        drop(commands);
        task.await.unwrap();

        assert_eq!(adapter.safety_reset_count(), 1);
        assert!(
            channels.diagnostics.try_recv().is_err(),
            "an orderly shutdown reported a failure"
        );
        assert_eq!(*channels.state.borrow(), PlaybackState::Stopped);
    }

    #[tokio::test(start_paused = true)]
    async fn dropping_the_final_handle_stops_playback_safely() {
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(SourceCommander::new(Grid::new(1, 1)), adapter.clone());
        // Kept past the handle, because the handle is what this drops and the
        // task publishes its way out through here. Dropping the last sender
        // leaves nothing to probe with, so the wait is on what the shutdown
        // itself says rather than on a message sent after it.
        let mut state = engine.state.clone();
        engine.start(Duration::from_secs(1)).unwrap();
        settle(&engine).await;
        assert_eq!(*state.borrow_and_update(), PlaybackState::Playing);

        drop(engine);
        state
            .changed()
            .await
            .expect("the task publishes the stop before it drops the sender");

        assert_eq!(*state.borrow(), PlaybackState::Stopped);
        assert_eq!(adapter.safety_reset_count(), 1);
    }

    ///
    /// A handle dropped while the engine is mid-Tick still gets the safety
    /// action, and gets it exactly once.
    ///
    /// The drop no longer waits for the Tick — there is no lock left for it to
    /// wait on, and a `stop` that blocked a console frame behind a device
    /// submission is the cost ADR 0040 removes. What survives is the guarantee
    /// itself: the queue closes, the task sees the close when it next looks,
    /// and the last thing it does is silence the device.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_the_final_handle_during_a_tick_completes_playback_safety() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let control = BlockingOutputControl::default();
        let engine = engine(
            source,
            BlockingOutputAdapter {
                control: control.clone(),
            },
        );
        engine.start(Duration::from_secs(1)).unwrap();
        control.wait_for_delivery();

        let (dropped_tx, dropped_rx) = std_mpsc::channel();
        let drop_thread = std::thread::spawn(move || {
            drop(engine);
            dropped_tx.send(()).unwrap();
        });
        dropped_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("dropping a handle does not wait for a Tick to finish");
        control.release_delivery();
        drop_thread.join().unwrap();

        wait_until("the engine never sent the safety action", || {
            control.safety_reset_count() == 1
        })
        .await;
    }

    ///
    /// `stop` returns without waiting for the Tick in flight, and no Tick runs
    /// after it.
    ///
    /// The handle's half of ADR 0002's guarantee, stated through the public
    /// surface: the engine is held inside a submission, `stop` is called from
    /// the test's thread while it is there, and the deadlines that pass while
    /// the submission is held deliver nothing once it is released.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stop_returns_without_waiting_for_a_tick_and_no_tick_follows_it() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let control = BlockingOutputControl::default();
        let engine = engine(
            source,
            BlockingOutputAdapter {
                control: control.clone(),
            },
        );
        engine.start(Duration::from_millis(1)).unwrap();
        control.wait_for_delivery();

        let stopping = engine.clone();
        let (stopped_tx, stopped_rx) = std_mpsc::channel();
        let stop_thread = std::thread::spawn(move || {
            stopping.stop();
            stopped_tx.send(()).unwrap();
        });
        stopped_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("stop does not wait for the Tick in flight");
        control.release_delivery();
        stop_thread.join().unwrap();

        // Hundreds of periods, and nothing is delivered in any of them.
        time::sleep(Duration::from_millis(200)).await;

        assert_eq!(control.deliveries(), 1);
        assert_eq!(engine.state(), PlaybackState::Stopped);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn clock_failure_remains_observable_after_output_panics() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let delivery_started = Arc::new(AtomicBool::new(false));
        let engine = engine(
            source,
            PanickingOutputAdapter {
                delivery_started: delivery_started.clone(),
            },
        );
        engine.start(Duration::from_secs(1)).unwrap();
        wait_until("the adapter was never asked to deliver", || {
            delivery_started.load(Ordering::SeqCst)
        })
        .await;

        let diagnostics = diagnostics_once_stopped(&engine).await;

        assert_eq!(
            diagnostics,
            vec![PlaybackDiagnostic::ClockFailure {
                message: "Playback clock terminated unexpectedly".to_string(),
            }]
        );
    }

    ///
    /// A run keeps ticking while messages keep arriving.
    ///
    /// The loop takes messages ahead of the deadline so that a `stop` wins a
    /// tie against the Tick it means to prevent. Unconditionally, that same
    /// bias lets a caller that keeps the queue non-empty hold the deadline arm
    /// off forever: the run stops delivering, keeps publishing `Playing`, and
    /// reports nothing, because from the engine's side nothing has gone wrong.
    /// An idempotent `start` is the cheapest such caller — the task looks at
    /// it, sees a run already live, and does nothing at all.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn a_run_keeps_ticking_while_messages_keep_arriving() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(source, adapter.clone());
        engine.start(Duration::from_millis(1)).unwrap();

        let stop_flooding = Arc::new(AtomicBool::new(false));
        let floods: Vec<_> = (0..6)
            .map(|_| {
                let flooding = engine.clone();
                let flood_until = stop_flooding.clone();
                std::thread::spawn(move || {
                    while !flood_until.load(Ordering::Relaxed) {
                        // Applied by the task as a no-op, so what this measures
                        // is the queue never being empty rather than the work
                        // of draining it.
                        let _ = flooding.start(Duration::from_millis(1));
                    }
                })
            })
            .collect();

        // Waited for rather than sampled after a fixed window. The bound says
        // a deadline gets its turn once the queue has had its share; it does
        // not say how long that takes, and it cannot, because the engine's
        // task is competing with six OS threads for a worker. A window would
        // be asserting a rate nothing promises — which is what made this fail
        // about one run in twelve — where the property actually claimed is
        // that the deadline arm is reached at all.
        let delivered_before = adapter.command_lists().len();
        wait_until(
            "the clock was starved: no Tick was delivered while messages kept arriving",
            || adapter.command_lists().len() > delivered_before,
        )
        .await;

        stop_flooding.store(true, Ordering::Relaxed);
        for flood in floods {
            flood.join().unwrap();
        }
    }

    ///
    /// A backend that panics being silenced does not take the process with it.
    ///
    /// The destructor exists for the unwind an adapter started, and the first
    /// thing it does is reach back into that same adapter for the safety
    /// action. A panic escaping a `Drop` that is already unwinding aborts the
    /// process — every other Orcvs window, the user's unsaved Source, all of
    /// it — which is a steep price for a device that was already refusing to
    /// listen. The safety action is worth attempting and is not worth that.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_backend_that_panics_being_silenced_does_not_abort_the_process() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let delivery_started = Arc::new(AtomicBool::new(false));
        let engine = engine(
            source,
            DoublyPanickingOutputAdapter {
                delivery_started: delivery_started.clone(),
            },
        );
        engine.start(Duration::from_secs(1)).unwrap();
        wait_until("the adapter was never asked to deliver", || {
            delivery_started.load(Ordering::SeqCst)
        })
        .await;

        // The stop is published before the safety action is attempted, so a
        // drain gated on `Stopped` alone can outrun the second report. Both
        // are collected instead, which is also what states the behaviour: the
        // engine says the run ended, and then says the device may still be
        // sounding.
        let mut diagnostics = Vec::new();
        wait_until(
            "the engine never reported both the stop and the silence",
            || {
                diagnostics.extend(engine.drain_diagnostics());
                diagnostics.len() == 2
            },
        )
        .await;

        assert_eq!(
            diagnostics,
            vec![
                PlaybackDiagnostic::ClockFailure {
                    message: "Playback clock terminated unexpectedly".to_string(),
                },
                PlaybackDiagnostic::ClockFailure {
                    message: "Playback output could not be silenced".to_string(),
                },
            ]
        );
    }

    ///
    /// An engine whose task has died refuses the run it can no longer begin.
    ///
    /// The task owns the state, so its death is the engine's: there is no
    /// second party left to spawn a fresh clock over surviving state. What is
    /// left to do is say so. Answering `Ok` here is what lets a console show a
    /// transport that has been asked to play, emits nothing, and reports
    /// nothing for the rest of the process.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn an_engine_whose_task_died_refuses_a_later_start() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let delivery_started = Arc::new(AtomicBool::new(false));
        let engine = engine(
            source,
            PanickingOutputAdapter {
                delivery_started: delivery_started.clone(),
            },
        );
        engine.start(Duration::from_secs(1)).unwrap();
        wait_until("the adapter was never asked to deliver", || {
            delivery_started.load(Ordering::SeqCst)
        })
        .await;
        // The task drops its receiver on the way out, before the state it owns
        // publishes the stop, so a published `Stopped` means the queue is shut.
        let _ = diagnostics_once_stopped(&engine).await;

        assert_eq!(
            engine.start(Duration::from_millis(1)),
            Err(PlaybackStartError::EngineUnavailable)
        );
        assert_eq!(
            engine.retune(Duration::from_millis(1)),
            Err(PlaybackStartError::EngineUnavailable)
        );
    }

    ///
    /// A Tick period too wide to schedule is refused where it is asked for,
    /// and leaves an engine that still works.
    ///
    /// The clock adds the period to its epoch to reach the next deadline, and
    /// an instant that cannot be expressed is not a deadline the run can wait
    /// on. Answering the caller is the whole point: the alternative is the
    /// task dying on the addition, which no handle can see and no diagnostic
    /// reports, leaving a published `Playing` that will never tick again.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_tick_period_too_wide_to_schedule_is_refused_and_leaves_the_engine_usable() {
        let source = SourceCommander::new(Grid::new(10, 6));
        write(&source, 20, "!>007FC4");
        write(&source, 0, ".=0101");
        let adapter = InMemoryOutputAdapter::default();
        let engine = engine(source, adapter.clone());

        assert_eq!(
            engine.start(Duration::from_secs(u64::MAX)),
            Err(PlaybackStartError::UnschedulableTickPeriod)
        );
        assert_eq!(engine.state(), PlaybackState::Stopped);

        // The engine is still there to be asked for a run it can schedule.
        engine
            .start(Duration::from_millis(1))
            .expect("a schedulable Tick period");
        wait_until(
            "the engine never delivered a Tick after refusing the wide period",
            || !adapter.command_lists().is_empty(),
        )
        .await;
        assert_eq!(engine.state(), PlaybackState::Playing);
    }

    ///
    /// Waits for the engine to publish `Stopped`, and answers with the
    /// diagnostics it recorded on its way there.
    ///
    /// Neither read waits on the engine — that is what publishing the state
    /// and draining the diagnostics buys — so a test whose subject is a
    /// failure reported by a task dying on another worker thread has to wait
    /// for that task rather than read once and race it.
    ///
    /// **The `Drop` path only.** There the report is recorded before the stop
    /// it explains is published, so a drain after this answers cannot have
    /// missed it. The ordinary `PlaybackInner::stop` is the other way round:
    /// it publishes `Stopped` first and then attempts the safety action, which
    /// can record an output failure after this has already returned. A test
    /// about a failing safety action on that path cannot gate on `Stopped` —
    /// it has to collect until it has the diagnostics it expects, the way
    /// `a_backend_that_panics_being_silenced_does_not_abort_the_process` does.
    /// The order is deliberate rather than incidental: the published state is
    /// what the console gates Space on, and moving the publish behind a device
    /// call would delay it by the length of that call.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    async fn diagnostics_once_stopped<A: OutputAdapter>(
        engine: &PlaybackEngine<A>,
    ) -> Vec<PlaybackDiagnostic> {
        wait_until("the engine never published a stop", || {
            engine.state() == PlaybackState::Stopped
        })
        .await;
        engine.drain_diagnostics()
    }
}
