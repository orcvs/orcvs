use std::collections::BTreeMap;
use std::fmt;
#[cfg(any(test, target_arch = "wasm32"))]
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::{self, Instant as ClockInstant};
use tokio_util::sync::CancellationToken;
#[cfg(target_arch = "wasm32")]
use web_time::Instant as ClockInstant;

use crate::source::{
    Length, MidiChannel, Note, PlayCommand, SourceCommander, Tick, TickPlan, Velocity,
};

///
/// One MIDI message the Playback Engine hands an output adapter.
///
/// A Play Command says what the Source asked for; an Output Command says what
/// is delivered. The two differ wherever this module owns the difference: ADR
/// 0016 gives Timed Play a Tick lifetime, and resolving that lifetime into a
/// start now and a stop at Tick `T + length` belongs to the engine that counts
/// Ticks. Every variant here is one message an adapter assembles immediately,
/// so an adapter holding a lifetime it would have to schedule is
/// unrepresentable rather than merely avoided.
///
/// A tagged variant set for the same reason [`PlayCommand`] is one: Control
/// Change and Pitch Bend join it as variants of their own, carried through
/// unresolved because nothing about them is the engine's to resolve.
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
}

pub trait OutputAdapter {
    fn submit(&mut self, commands: &[OutputCommand]) -> Result<(), OutputAdapterError>;
    fn all_notes_off(&mut self) -> Result<(), OutputAdapterError>;
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

#[derive(Clone, Debug, PartialEq)]
pub struct PlaybackObservation {
    pub state: PlaybackState,
    pub diagnostics: Vec<PlaybackDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackStartError {
    ZeroTickPeriod,
    RuntimeUnavailable,
}

#[derive(Clone, Copy)]
struct TickTiming {
    scheduled_at: Duration,
    observed_at: Duration,
    period: Duration,
}

impl TickTiming {
    fn is_overrun(self) -> bool {
        self.observed_at >= self.scheduled_at + self.period
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn next_scheduled_at(
    scheduled_at: Duration,
    observed_at: Duration,
    tick_period: Duration,
) -> Duration {
    let next = scheduled_at.saturating_add(tick_period);
    if next <= observed_at {
        observed_at.saturating_add(tick_period)
    } else {
        next
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn wasm_timeout_millis(delay: Duration) -> u32 {
    delay
        .as_nanos()
        .div_ceil(1_000_000)
        .min(u128::from(u32::MAX)) as u32
}

#[cfg(any(test, target_arch = "wasm32"))]
async fn wait_for_tick_or_cancellation<F>(delay: F, cancellation: &CancellationToken) -> bool
where
    F: Future<Output = ()>,
{
    tokio::select! {
        () = delay => true,
        () = cancellation.cancelled() => false,
    }
}

impl fmt::Display for PlaybackStartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroTickPeriod => formatter.write_str("Tick period must be greater than zero"),
            Self::RuntimeUnavailable => formatter.write_str("Playback requires a Tokio runtime"),
        }
    }
}

///
/// The channel and note one Timed Play command sounds on.
///
/// Ownership is per channel *and* note: Timed Play is polyphonic, and ADR 0016
/// gives one voice per channel to Monophonic Play alone. The key carries the
/// two domain types the interpreter proved rather than their bytes, so the
/// stop this module delivers re-derives neither.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TimedVoice {
    channel: MidiChannel,
    note: Note,
}

///
/// Which claim on a voice a scheduled stop belongs to.
///
/// ADR 0016's generation token. An expiry is scheduled at the Tick it is due
/// at and cannot be found again when the note it would stop is replaced or
/// stopped early, so a stale one is left in the schedule and refused when it
/// comes due: it carries the claim that scheduled it, and only the claim still
/// standing then is stopped. Without it a Source that stops a note and starts
/// it again would have the first command's expiry cut the second note short.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Claim(u64);

/// One scheduled stop, and the claim it belongs to.
#[derive(Clone, Copy, Debug)]
struct Expiry {
    voice: TimedVoice,
    claim: Claim,
}

///
/// Every note a Timed Play command owns, and the Tick each is stopped at.
///
/// ADR 0001 keeps musical intent out of the output adapter and ADR 0016 puts a
/// Timed Play's whole lifetime in the Tick Plan, which leaves exactly this
/// between them: the engine reads the length, delivers the start in Tick Plan
/// order, and delivers the stop when the run reaches the Tick it is due at.
///
/// It is the only state here that outlives one Tick, and it describes notes
/// that are sounding, so everything that silences output clears it: beginning
/// a run, stopping, disconnecting, and changing destination.
///
#[derive(Clone, Default)]
struct TimedNotes {
    owned: BTreeMap<TimedVoice, Claim>,
    expiries: BTreeMap<Tick, Vec<Expiry>>,
    next_claim: u64,
}

///
/// The explicit stop for `voice`.
///
/// MIDI's zero-velocity Note On, which is the stop Raw Play already gives the
/// Source through velocity `00`: a scheduled expiry is delivered as a message
/// a Source could have written for itself rather than as a shape of its own.
///
fn note_off(voice: TimedVoice) -> OutputCommand {
    OutputCommand::NoteOn {
        channel: voice.channel,
        velocity: Velocity::ZERO,
        note: voice.note,
    }
}

impl TimedNotes {
    ///
    /// This Tick's delivery: every stop due at `tick`, then `commands`
    /// resolved against ownership, in Tick Plan order.
    ///
    /// One list rather than two submissions, because the order is the whole of
    /// what ADR 0016 requires here — a scheduled Note Off arrives at the
    /// beginning of executed Tick `T + length`, before that Tick's new Play
    /// Commands — and a Tick that submitted twice would leave that order to
    /// the adapter to keep.
    ///
    fn deliver(&mut self, tick: Tick, commands: &[PlayCommand]) -> Vec<OutputCommand> {
        let mut delivery = self.expired_at(tick);

        for command in commands {
            match *command {
                // Raw notes do not enter Timed ownership: what the Source
                // wrote is delivered, and nothing stops it that the Source did
                // not ask to stop.
                PlayCommand::Raw {
                    channel,
                    velocity,
                    note,
                } => delivery.push(OutputCommand::NoteOn {
                    channel,
                    velocity,
                    note,
                }),
                PlayCommand::Timed {
                    channel,
                    velocity,
                    note,
                    length,
                } => {
                    let voice = TimedVoice { channel, note };
                    if velocity == Velocity::ZERO {
                        // An explicit stop, whatever length accompanies it,
                        // scheduling no expiry. Releasing the claim is what
                        // keeps the expiry this note already had from stopping
                        // whatever sounds on the voice next.
                        self.release(voice);
                        delivery.push(note_off(voice));
                    } else if length == Length::ZERO {
                        // A lifetime of no Ticks never starts, and is not a
                        // stop: the note this voice owns and the expiry it is
                        // due both stand.
                    } else {
                        // A replacement stops the instance it replaces before
                        // it starts, and retires that instance's expiry with it.
                        if self.release(voice) {
                            delivery.push(note_off(voice));
                        }
                        delivery.push(OutputCommand::NoteOn {
                            channel,
                            velocity,
                            note,
                        });
                        self.claim(voice, tick.after(length.ticks()));
                    }
                }
            }
        }

        delivery
    }

    ///
    /// The stops due at `tick`, in the order they were scheduled.
    ///
    /// Everything due at or before it, though an ordinary run reaches every
    /// Tick in turn: a Tick the engine declines consumes no absolute Tick, so
    /// nothing is skipped, and draining the whole range regardless is what
    /// keeps an expiry from outliving the Tick it was due at by the Ticks a
    /// future scheduling rule might skip. One expiry is beyond it — a stop
    /// `Tick::after` saturated at the last Tick, which the counter it is
    /// compared against can no longer reach — and a run whose absolute Tick
    /// has stopped advancing has already lost more than a Note Off.
    ///
    fn expired_at(&mut self, tick: Tick) -> Vec<OutputCommand> {
        let later = self.expiries.split_off(&tick.next());
        let due = std::mem::replace(&mut self.expiries, later);

        let mut stops = Vec::new();
        for expiry in due.into_values().flatten() {
            // A stale expiry stops nothing: its claim was released when the
            // voice was replaced or stopped, so what sounds there now is not
            // what it was scheduled for.
            if self.owned.get(&expiry.voice) == Some(&expiry.claim) {
                self.owned.remove(&expiry.voice);
                stops.push(note_off(expiry.voice));
            }
        }
        stops
    }

    ///
    /// Claims `voice` until `due`, so the Tick it is due at stops it.
    ///
    fn claim(&mut self, voice: TimedVoice, due: Tick) {
        // Unreachable for the reason `Tick::next`'s saturation is unreachable:
        // a run would have to claim a voice every nanosecond for five hundred
        // years to wrap this counter.
        let claim = Claim(self.next_claim);
        self.next_claim = self.next_claim.wrapping_add(1);
        self.owned.insert(voice, claim);
        self.expiries
            .entry(due)
            .or_default()
            .push(Expiry { voice, claim });
    }

    ///
    /// Gives up any claim on `voice`, invalidating the stop it scheduled, and
    /// answers whether a note was standing.
    ///
    fn release(&mut self, voice: TimedVoice) -> bool {
        self.owned.remove(&voice).is_some()
    }

    ///
    /// Forgets every claim and every scheduled stop.
    ///
    fn clear(&mut self) {
        self.owned.clear();
        self.expiries.clear();
    }
}

#[derive(Default)]
struct InMemoryOutputState {
    command_lists: Vec<Vec<OutputCommand>>,
    all_notes_off_count: usize,
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

    pub fn all_notes_off_count(&self) -> usize {
        self.state.lock().unwrap().all_notes_off_count
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

    fn all_notes_off(&mut self) -> Result<(), OutputAdapterError> {
        self.state.lock().unwrap().all_notes_off_count += 1;
        Ok(())
    }
}

struct PlaybackInner<A> {
    source: SourceCommander,
    adapter: A,
    playing: bool,
    connected: bool,
    diagnostics: Vec<PlaybackDiagnostic>,
    last_output_failure: Option<OutputAdapterError>,
    generation: u64,
    cancellation: Option<CancellationToken>,
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
    /// The notes Timed Play owns, and the Tick each is stopped at.
    ///
    /// Here rather than beside the Source for the reason the absolute Tick is:
    /// a scheduled stop belongs to one Playback run, and ADR 0003 keeps every
    /// piece of language state in the Source Snapshot, which a schedule of
    /// future effects is not.
    ///
    timed: TimedNotes,
}

pub struct PlaybackEngine<A: OutputAdapter> {
    inner: Arc<Mutex<PlaybackInner<A>>>,
    handle_count: Arc<AtomicUsize>,
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
pub struct MidiSelectionHandle<B: crate::midi::MidiBackend> {
    inner: Weak<Mutex<PlaybackInner<crate::midi::MidiOutputAdapter<B>>>>,
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
impl<B: crate::midi::MidiBackend> MidiSelectionHandle<B> {
    pub(crate) fn new(playback: &PlaybackEngine<crate::midi::MidiOutputAdapter<B>>) -> Self {
        Self {
            inner: Arc::downgrade(&playback.inner),
        }
    }

    fn inner(
        &self,
    ) -> Result<Arc<Mutex<PlaybackInner<crate::midi::MidiOutputAdapter<B>>>>, crate::midi::MidiError>
    {
        self.inner
            .upgrade()
            .ok_or_else(|| crate::midi::MidiError::new("running Orcvs is no longer available"))
    }

    pub fn destinations(
        &self,
    ) -> Result<Vec<crate::midi::MidiDestination>, crate::midi::MidiError> {
        let inner = self.inner()?;
        lock_recover(&inner).adapter.destinations()
    }

    pub fn select(
        &self,
        destination_id: &crate::midi::MidiDestinationId,
    ) -> Result<(), crate::midi::MidiError> {
        let inner = self.inner()?;
        let mut inner = lock_recover(&inner);
        inner.select_destination(destination_id)
    }

    pub fn selected_destination_id(
        &self,
    ) -> Result<Option<crate::midi::MidiDestinationId>, crate::midi::MidiError> {
        let inner = self.inner()?;
        Ok(lock_recover(&inner)
            .adapter
            .selected_destination_id()
            .cloned())
    }
}

impl<A: OutputAdapter> Clone for PlaybackEngine<A> {
    fn clone(&self) -> Self {
        self.handle_count.fetch_add(1, Ordering::Relaxed);
        Self {
            inner: self.inner.clone(),
            handle_count: self.handle_count.clone(),
        }
    }
}

impl<A: OutputAdapter> PlaybackInner<A> {
    fn stop(&mut self) {
        if let Some(cancellation) = self.cancellation.take() {
            cancellation.cancel();
        }
        if self.playing {
            self.playing = false;
            if self.connected {
                self.send_all_notes_off();
            }
        }
        // All-notes-off has stopped whatever was sounding, so every claim is
        // over and every scheduled stop is redundant. Clearing unconditionally
        // rather than alongside the all-notes-off above is deliberate: a run
        // that is already stopped owns nothing, and an engine that reached
        // here holding a claim would otherwise carry it into the next run.
        self.timed.clear();
    }

    fn send_all_notes_off(&mut self) {
        if let Err(error) = self.adapter.all_notes_off() {
            self.record_output_failure(error);
        }
    }

    fn record_output_failure(&mut self, error: OutputAdapterError) {
        if self.last_output_failure.as_ref() != Some(&error) {
            self.diagnostics
                .push(PlaybackDiagnostic::OutputFailure(error.clone()));
            self.last_output_failure = Some(error);
        }
    }

    ///
    /// Begins a Playback run, and hands back the generation and cancellation
    /// token the clock driving that run must carry. ADR 0002 keeps lifecycle
    /// concurrency inside this module, so every clock enters a run through this
    /// one place: the native clock, the browser clock, and the tests all begin
    /// a run identically, and a run-scoped input added here cannot be reset on
    /// one target and forgotten on another.
    ///
    /// ADR 0012 makes the absolute Tick an interpretation input, so a run that
    /// began while still carrying the previous run's counter would fire a Delay
    /// `~*0104` on the wrong beat. Resetting the counter is therefore part of
    /// beginning a run, alongside the bumped generation that retires the old
    /// clock and the cleared last Tick that lets this run's first Tick execute
    /// immediately.
    ///
    /// `retune` deliberately does not begin a run: retuning changes the Tick
    /// period of the run it is already in, so it keeps that run's absolute Tick
    /// and reads the last Tick to schedule the first retuned Tick against it.
    ///
    fn begin_run(&mut self) -> (u64, CancellationToken) {
        self.generation = self.generation.wrapping_add(1);
        self.playing = true;
        self.last_tick_at = None;
        self.tick = Tick::ZERO;
        // A scheduled stop is due at an absolute Tick, and this run's absolute
        // Ticks begin again at zero, so an inherited expiry would stop a note
        // of the new run that has not started. Discarding the schedule is part
        // of beginning a run for the same reason resetting the counter is.
        self.timed.clear();
        if let Some(previous) = self.cancellation.take() {
            previous.cancel();
        }
        let cancellation = CancellationToken::new();
        self.cancellation = Some(cancellation.clone());
        (self.generation, cancellation)
    }

    ///
    /// Executes one Tick of the run named by `generation`: interprets a Source
    /// Snapshot into a Tick Plan and advances the absolute Tick. Named apart
    /// from the `tick` field it advances, so a call site says whether it reads
    /// the counter or spends one.
    ///
    fn execute_tick(&mut self, generation: u64, timing: TickTiming) -> Option<TickPlan> {
        if !self.playing || self.generation != generation {
            return None;
        }
        if timing.is_overrun() {
            self.diagnostics.push(PlaybackDiagnostic::Overrun {
                scheduled_at: timing.scheduled_at,
                observed_at: timing.observed_at,
            });
            return None;
        }
        self.last_tick_at = Some(ClockInstant::now());
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
            let mut timed = self.timed.clone();
            let delivery = timed.deliver(tick, &plan.play_commands);
            match self.adapter.submit(&delivery) {
                Ok(()) => {
                    self.timed = timed;
                    self.last_output_failure = None;
                }
                Err(error) => self.record_output_failure(error),
            }
        }
        Some(plan)
    }
}

impl<A: OutputAdapter> PlaybackEngine<A> {
    pub fn new(source: SourceCommander, adapter: A) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PlaybackInner {
                source,
                adapter,
                playing: false,
                connected: true,
                diagnostics: Vec::new(),
                last_output_failure: None,
                generation: 0,
                cancellation: None,
                last_tick_at: None,
                tick: Tick::ZERO,
                timed: TimedNotes::default(),
            })),
            handle_count: Arc::new(AtomicUsize::new(1)),
        }
    }

    pub fn observe(&self) -> PlaybackObservation {
        let mut inner = lock_recover(&self.inner);
        PlaybackObservation {
            state: if inner.playing {
                PlaybackState::Playing
            } else {
                PlaybackState::Stopped
            },
            diagnostics: std::mem::take(&mut inner.diagnostics),
        }
    }

    pub fn stop(&self) {
        lock_recover(&self.inner).stop();
    }

    pub fn disconnect(&self) {
        let mut inner = lock_recover(&self.inner);
        if inner.connected {
            inner.send_all_notes_off();
            inner.connected = false;
        }
        // Nothing this engine owns is sounding on a disconnected output, and
        // nothing it delivers while disconnected can start a note, so the
        // schedule goes with the connection.
        inner.timed.clear();
    }

    #[cfg(test)]
    fn clock_tick(&self, timing: TickTiming) -> Option<TickPlan> {
        let mut inner = self.inner.lock().unwrap();
        let generation = inner.generation;
        inner.execute_tick(generation, timing)
    }

    ///
    /// Begins a Playback run without a clock, so a test can execute Ticks by
    /// hand. It begins the run exactly as `start` does, because a test that
    /// began one differently would pin a state no run ever reaches.
    ///
    #[cfg(test)]
    fn activate_for_test(&self) {
        self.inner.lock().unwrap().begin_run();
    }

    #[cfg(test)]
    fn diagnostics(&self) -> Vec<PlaybackDiagnostic> {
        self.inner.lock().unwrap().diagnostics.clone()
    }

    ///
    /// The absolute Tick the next executed Tick will interpret at.
    ///
    #[cfg(test)]
    fn current_tick(&self) -> Tick {
        self.inner.lock().unwrap().tick
    }

    #[cfg(test)]
    fn is_playing(&self) -> bool {
        self.inner.lock().unwrap().playing
    }

    ///
    /// Whether any Timed Play note is claimed or any stop still scheduled.
    ///
    /// The lifecycle rule is that nothing survives a run, and a run that has
    /// ended delivers nothing more for a test to read: what is left to observe
    /// is the state itself.
    ///
    #[cfg(test)]
    fn holds_timed_ownership(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        !inner.timed.owned.is_empty() || !inner.timed.expiries.is_empty()
    }
}

impl<B: crate::midi::MidiBackend> PlaybackInner<crate::midi::MidiOutputAdapter<B>> {
    ///
    /// Connects this engine's output to `destination_id`.
    ///
    /// Both ways a destination is chosen — this engine's own method and the
    /// selection handle the shell holds — arrive here, so what a change of
    /// destination owes is stated once rather than twice.
    ///
    fn select_destination(
        &mut self,
        destination_id: &crate::midi::MidiDestinationId,
    ) -> Result<(), crate::midi::MidiError> {
        // The notes this engine owned are sounding on the destination it is
        // leaving, which is sent all-notes-off before the new connection is
        // reached. Their scheduled stops would arrive at a device that never
        // started them, so the schedule goes with the attempt rather than with
        // its success: a change that cannot connect has silenced the old
        // device just the same, and a claim kept across it would stop a note
        // the Source starts on that voice afterwards. Nothing is owned while
        // disconnected, so clearing before a failure that leaves this engine
        // connected to the destination it already had discards nothing else.
        self.timed.clear();
        let selection = self.adapter.select(destination_id)?;
        self.last_output_failure = None;
        if let Some(error) = selection.safety_failure() {
            self.record_output_failure(OutputAdapterError::new(error.message));
        }
        self.connected = true;
        Ok(())
    }
}

impl<B: crate::midi::MidiBackend> PlaybackEngine<crate::midi::MidiOutputAdapter<B>> {
    pub fn midi_destinations(
        &self,
    ) -> Result<Vec<crate::midi::MidiDestination>, crate::midi::MidiError> {
        lock_recover(&self.inner).adapter.destinations()
    }

    pub fn select_midi_destination(
        &self,
        destination_id: &crate::midi::MidiDestinationId,
    ) -> Result<(), crate::midi::MidiError> {
        lock_recover(&self.inner).select_destination(destination_id)
    }

    pub fn selected_midi_destination_id(&self) -> Option<crate::midi::MidiDestinationId> {
        lock_recover(&self.inner)
            .adapter
            .selected_destination_id()
            .cloned()
    }
}

impl<A: OutputAdapter + Send + 'static> PlaybackEngine<A> {
    ///
    /// Records a start failure as a diagnostic so the shell can surface it, and
    /// hands the error back for the caller to return. Every `start` failure path
    /// goes through here; a caller that only returns the error leaves the user
    /// with silence.
    ///
    fn report_start_error(&self, error: PlaybackStartError) -> PlaybackStartError {
        lock_recover(&self.inner)
            .diagnostics
            .push(PlaybackDiagnostic::StartFailure {
                message: error.to_string(),
            });
        error
    }

    pub(crate) fn report_retune_error(&self, error: PlaybackStartError) {
        lock_recover(&self.inner)
            .diagnostics
            .push(PlaybackDiagnostic::RetuneFailure {
                message: error.to_string(),
            });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn retune(&self, tick_period: Duration) -> Result<(), PlaybackStartError> {
        if tick_period.is_zero() {
            return Err(PlaybackStartError::ZeroTickPeriod);
        }
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| PlaybackStartError::RuntimeUnavailable)?;

        let (generation, cancellation, weak, first_tick_at) = {
            let mut inner = lock_recover(&self.inner);
            if !inner.playing {
                return Ok(());
            }
            let now = ClockInstant::now();
            let first_tick_at = inner
                .last_tick_at
                .and_then(|last_tick_at| last_tick_at.checked_add(tick_period))
                .map_or(now, |scheduled_at| scheduled_at.max(now));
            // Retuning retires the running clock and hands the run to a new
            // one, but it does not begin a run: `begin_run` is deliberately not
            // called here, because this run keeps its absolute Tick and its
            // last Tick is what the first retuned Tick is scheduled against.
            if let Some(previous) = inner.cancellation.take() {
                previous.cancel();
            }
            inner.generation = inner.generation.wrapping_add(1);
            let cancellation = CancellationToken::new();
            inner.cancellation = Some(cancellation.clone());
            (
                inner.generation,
                cancellation,
                Arc::downgrade(&self.inner),
                first_tick_at,
            )
        };

        runtime.spawn(async move {
            let mut guard = ClockRunGuard::new(weak.clone(), generation);
            let epoch = time::Instant::now();
            let first_tick_delay = first_tick_at.saturating_duration_since(ClockInstant::now());
            let mut interval = time::interval_at(epoch + first_tick_delay, tick_period);
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Burst);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => {
                        guard.finish();
                        break;
                    },
                    scheduled = interval.tick() => {
                        let scheduled_at = scheduled.duration_since(epoch);
                        let observed_at = time::Instant::now().duration_since(epoch);
                        let Some(inner) = weak.upgrade() else { break };
                        lock_recover(&inner).execute_tick(generation, TickTiming {
                            scheduled_at,
                            observed_at,
                            period: tick_period,
                        });
                    }
                }
            }
        });
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn retune(&self, tick_period: Duration) -> Result<(), PlaybackStartError> {
        if tick_period.is_zero() {
            return Err(PlaybackStartError::ZeroTickPeriod);
        }

        let (generation, cancellation, weak, first_tick_delay) = {
            let mut inner = lock_recover(&self.inner);
            if !inner.playing {
                return Ok(());
            }
            let now = ClockInstant::now();
            let first_tick_delay = inner
                .last_tick_at
                .and_then(|last_tick_at| last_tick_at.checked_add(tick_period))
                .map_or(Duration::ZERO, |scheduled_at| {
                    scheduled_at.saturating_duration_since(now)
                });
            // Retuning retires the running clock and hands the run to a new
            // one, but it does not begin a run: `begin_run` is deliberately not
            // called here, because this run keeps its absolute Tick and its
            // last Tick is what the first retuned Tick is scheduled against.
            if let Some(previous) = inner.cancellation.take() {
                previous.cancel();
            }
            inner.generation = inner.generation.wrapping_add(1);
            let cancellation = CancellationToken::new();
            inner.cancellation = Some(cancellation.clone());
            (
                inner.generation,
                cancellation,
                Arc::downgrade(&self.inner),
                first_tick_delay,
            )
        };

        wasm_bindgen_futures::spawn_local(async move {
            let mut guard = ClockRunGuard::new(weak.clone(), generation);
            let epoch = web_time::Instant::now();
            let mut scheduled_at = first_tick_delay;

            loop {
                let delay = scheduled_at.saturating_sub(epoch.elapsed());
                if !wait_for_tick_or_cancellation(
                    gloo_timers::future::TimeoutFuture::new(wasm_timeout_millis(delay)),
                    &cancellation,
                )
                .await
                {
                    guard.finish();
                    break;
                }

                let observed_at = epoch.elapsed();
                let Some(inner) = weak.upgrade() else { break };
                lock_recover(&inner).execute_tick(
                    generation,
                    TickTiming {
                        scheduled_at,
                        observed_at,
                        period: tick_period,
                    },
                );
                scheduled_at = next_scheduled_at(scheduled_at, observed_at, tick_period);
            }
        });
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn start(&self, tick_period: Duration) -> Result<(), PlaybackStartError> {
        if tick_period.is_zero() {
            return Err(self.report_start_error(PlaybackStartError::ZeroTickPeriod));
        }
        if lock_recover(&self.inner).playing {
            return Ok(());
        }
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| self.report_start_error(PlaybackStartError::RuntimeUnavailable))?;

        let (generation, cancellation, weak) = {
            let mut inner = lock_recover(&self.inner);
            if inner.playing {
                return Ok(());
            }
            let (generation, cancellation) = inner.begin_run();
            (generation, cancellation, Arc::downgrade(&self.inner))
        };

        runtime.spawn(async move {
            let mut guard = ClockRunGuard::new(weak.clone(), generation);
            let epoch = time::Instant::now();
            let mut interval = time::interval_at(epoch, tick_period);
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Burst);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => {
                        guard.finish();
                        break;
                    },
                    scheduled = interval.tick() => {
                        let scheduled_at = scheduled.duration_since(epoch);
                        let observed_at = time::Instant::now().duration_since(epoch);
                        let Some(inner) = weak.upgrade() else { break };
                        lock_recover(&inner).execute_tick(generation, TickTiming {
                            scheduled_at,
                            observed_at,
                            period: tick_period,
                        });
                    }
                }
            }
        });
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn start(&self, tick_period: Duration) -> Result<(), PlaybackStartError> {
        if tick_period.is_zero() {
            return Err(self.report_start_error(PlaybackStartError::ZeroTickPeriod));
        }
        if lock_recover(&self.inner).playing {
            return Ok(());
        }

        let (generation, cancellation, weak) = {
            let mut inner = lock_recover(&self.inner);
            if inner.playing {
                return Ok(());
            }
            let (generation, cancellation) = inner.begin_run();
            (generation, cancellation, Arc::downgrade(&self.inner))
        };

        wasm_bindgen_futures::spawn_local(async move {
            let mut guard = ClockRunGuard::new(weak.clone(), generation);
            let epoch = web_time::Instant::now();
            let mut scheduled_at = Duration::ZERO;

            loop {
                if cancellation.is_cancelled() {
                    guard.finish();
                    break;
                }

                let observed_at = epoch.elapsed();
                let Some(inner) = weak.upgrade() else { break };
                lock_recover(&inner).execute_tick(
                    generation,
                    TickTiming {
                        scheduled_at,
                        observed_at,
                        period: tick_period,
                    },
                );

                scheduled_at = next_scheduled_at(scheduled_at, observed_at, tick_period);
                let delay = scheduled_at.saturating_sub(epoch.elapsed());
                let delay_ms = wasm_timeout_millis(delay);
                if !wait_for_tick_or_cancellation(
                    gloo_timers::future::TimeoutFuture::new(delay_ms),
                    &cancellation,
                )
                .await
                {
                    guard.finish();
                    break;
                }
            }
        });
        Ok(())
    }
}

struct ClockRunGuard<A: OutputAdapter> {
    inner: Weak<Mutex<PlaybackInner<A>>>,
    generation: u64,
    finished: bool,
}

impl<A: OutputAdapter> ClockRunGuard<A> {
    fn new(inner: Weak<Mutex<PlaybackInner<A>>>, generation: u64) -> Self {
        Self {
            inner,
            generation,
            finished: false,
        }
    }

    fn finish(&mut self) {
        self.finished = true;
    }
}

impl<A: OutputAdapter> Drop for ClockRunGuard<A> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let Some(inner) = self.inner.upgrade() else {
            return;
        };
        let mut inner = lock_recover(&inner);
        if inner.generation == self.generation && inner.playing {
            inner.diagnostics.push(PlaybackDiagnostic::ClockFailure {
                message: "Playback clock terminated unexpectedly".to_string(),
            });
            inner.stop();
        }
    }
}

impl<A: OutputAdapter> Drop for PlaybackEngine<A> {
    fn drop(&mut self) {
        if self.handle_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            lock_recover(&self.inner).stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{CellIndex, Grid};

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

    #[cfg(not(target_arch = "wasm32"))]
    use std::sync::{Condvar, atomic::AtomicBool, mpsc};
    #[cfg(target_arch = "wasm32")]
    use tokio::time;

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn tick_wait_exits_as_soon_as_playback_is_cancelled() {
        let cancellation = CancellationToken::new();
        let waiting = wait_for_tick_or_cancellation(std::future::pending(), &cancellation);
        let cancellation_trigger = cancellation.clone();
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            cancellation_trigger.cancel();
        });

        let result = time::timeout(Duration::from_millis(100), waiting).await;

        assert_eq!(result, Ok(false));
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

        fn all_notes_off(&mut self) -> Result<(), OutputAdapterError> {
            Ok(())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[derive(Default)]
    struct BlockingOutputState {
        delivery_started: bool,
        release_delivery: bool,
        all_notes_off_count: usize,
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[derive(Clone, Default)]
    struct BlockingOutputControl {
        state: Arc<(Mutex<BlockingOutputState>, Condvar)>,
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl BlockingOutputControl {
        fn wait_for_delivery(&self) {
            let (lock, changed) = &*self.state;
            let state = lock.lock().unwrap();
            let _state = changed
                .wait_while(state, |state| !state.delivery_started)
                .unwrap();
        }

        fn release_delivery(&self) {
            let (lock, changed) = &*self.state;
            lock.lock().unwrap().release_delivery = true;
            changed.notify_all();
        }

        fn all_notes_off_count(&self) -> usize {
            self.state.0.lock().unwrap().all_notes_off_count
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

    #[cfg(not(target_arch = "wasm32"))]
    impl OutputAdapter for PanickingOutputAdapter {
        fn submit(&mut self, _commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
            self.delivery_started.store(true, Ordering::SeqCst);
            panic!("test output panic");
        }

        fn all_notes_off(&mut self) -> Result<(), OutputAdapterError> {
            Ok(())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl OutputAdapter for BlockingOutputAdapter {
        fn submit(&mut self, _commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
            let (lock, changed) = &*self.control.state;
            let mut state = lock.lock().unwrap();
            state.delivery_started = true;
            changed.notify_all();
            let _state = changed
                .wait_while(state, |state| !state.release_delivery)
                .unwrap();
            Ok(())
        }

        fn all_notes_off(&mut self) -> Result<(), OutputAdapterError> {
            self.control.state.0.lock().unwrap().all_notes_off_count += 1;
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
    /// Clears `len` Cells from `start`, so a test can retire the Bang that
    /// activates a terminal root.
    ///
    /// A Source-resident Bang persists across Ticks, so an activated terminal
    /// root repeats on every Tick until `spatial-tick-planning/02` gives the
    /// Bang its one-Tick expiry. A test about what the Playback Engine
    /// schedules retires the Bang instead, which leaves every Tick after it
    /// carrying the schedule alone.
    ///
    fn erase(source: &SourceCommander, start: usize, len: usize) {
        let grid = source.grid();
        for offset in 0..len {
            source.unset(cell(grid, start + offset));
        }
    }

    ///
    /// Runs the Tick numbered `tick` of a hand-driven run, on time.
    ///
    /// The engine counts executed Ticks, so a test that runs them in order
    /// names each by its absolute Tick and states the schedule under test in
    /// the same numbers ADR 0016 does.
    ///
    fn run_tick<A: OutputAdapter>(engine: &PlaybackEngine<A>, tick: u64) {
        engine
            .clock_tick(scheduled(
                Duration::from_secs(tick),
                Duration::from_secs(tick),
            ))
            .expect("a scheduled Tick runs");
    }

    ///
    /// A hand-driven run that has executed one Tick of a Timed Play, so it
    /// owns a note whose stop is due at a Tick it has not reached.
    ///
    fn engine_owning_a_timed_note() -> (PlaybackEngine<InMemoryOutputAdapter>, InMemoryOutputAdapter)
    {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC40A");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source, adapter.clone());
        engine.activate_for_test();
        run_tick(&engine, 0);

        assert!(engine.holds_timed_ownership());
        (engine, adapter)
    }

    fn scheduled(scheduled_at: Duration, observed_at: Duration) -> TickTiming {
        TickTiming {
            scheduled_at,
            observed_at,
            period: Duration::from_secs(1),
        }
    }

    #[test]
    fn resumed_wasm_clock_discards_tick_debt_before_scheduling_again() {
        assert_eq!(
            super::next_scheduled_at(
                Duration::from_secs(1),
                Duration::from_secs(60),
                Duration::from_secs(1),
            ),
            Duration::from_secs(61)
        );
    }

    #[test]
    fn zero_wasm_delay_still_schedules_a_browser_timer() {
        assert_eq!(super::wasm_timeout_millis(Duration::ZERO), 0);
    }

    #[tokio::test]
    async fn clock_tick_commits_source_before_submitting_play_commands() {
        let source = SourceCommander::new(Grid::new(10, 4));
        write(&source, 0, ".+0102");
        write(&source, 20, "!>007FC4");
        write(&source, 30, "**");
        let engine =
            PlaybackEngine::new(source.clone(), RecordingAdapter::observing(source.clone()));
        engine.activate_for_test();

        let tick = engine
            .clock_tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("scheduled Tick runs");

        let inner = engine.inner.lock().unwrap();
        assert_eq!(&inner.adapter.source_at_submission[0][10..12], "03");
        assert_eq!(&source.snapshot()[10..12], "03");
        assert_eq!(tick.play_commands.len(), 1);
        assert_eq!(
            inner.adapter.command_lists,
            vec![vec![note_on(0, 0x7F, 60)]]
        );
    }

    #[tokio::test]
    async fn playback_begins_at_the_first_tick_and_advances_one_per_executed_tick() {
        // ADR 0012's counter in full: the first Tick of a run is absolute Tick
        // `0`, and each executed Tick increments it by exactly one. Reading the
        // counter is the whole of what is observable today — no Function reads
        // the Tick yet — so the count is what is pinned.
        let engine = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 3)),
            InMemoryOutputAdapter::default(),
        );
        engine.activate_for_test();

        assert_eq!(engine.current_tick(), Tick::ZERO);

        for executed in 1..=4u64 {
            engine
                .clock_tick(scheduled(
                    Duration::from_secs(executed - 1),
                    Duration::from_secs(executed - 1),
                ))
                .expect("a scheduled Tick runs");

            assert_eq!(engine.current_tick(), Tick::new(executed));
        }
    }

    #[tokio::test]
    async fn a_tick_the_engine_declines_consumes_no_absolute_tick() {
        // The counter counts executed Ticks, not clock ticks: a Tick that
        // returns before interpreting a Source Snapshot planned nothing, so
        // there is no Tick for it to have been. Each of the three ways the
        // engine declines one is pinned, because each is a separate early
        // return that a later change could move the increment above.
        let engine = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 3)),
            InMemoryOutputAdapter::default(),
        );

        // Stopped: nothing is playing, so nothing is interpreted.
        assert!(
            engine
                .clock_tick(scheduled(Duration::ZERO, Duration::ZERO))
                .is_none()
        );
        assert_eq!(engine.current_tick(), Tick::ZERO);

        engine.activate_for_test();
        engine
            .clock_tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("a scheduled Tick runs");
        assert_eq!(engine.current_tick(), Tick::new(1));

        // Overrun: the Tick is dropped rather than played late.
        assert!(
            engine
                .clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(5)))
                .is_none()
        );
        assert_eq!(engine.current_tick(), Tick::new(1));

        // Superseded: a clock from an earlier run cannot drive this one.
        let mut inner = engine.inner.lock().unwrap();
        let superseded = inner.generation.wrapping_sub(1);
        assert!(
            inner
                .execute_tick(
                    superseded,
                    scheduled(Duration::from_secs(1), Duration::from_secs(1))
                )
                .is_none()
        );
        assert_eq!(inner.tick, Tick::new(1));
    }

    #[tokio::test(start_paused = true)]
    async fn each_playback_run_begins_again_at_the_first_tick() {
        // ADR 0012's first-Tick rule is about a Playback run, not about the
        // lifetime of the engine: a run that is stopped and started again is a
        // new run and counts from `0` again.
        let engine = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 3)),
            InMemoryOutputAdapter::default(),
        );

        // A paused clock fires once at the epoch, so one yield is one executed
        // Tick — the exact count the tests around this one are written against.
        engine.start(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;
        assert_eq!(engine.current_tick(), Tick::new(1));

        engine.stop();
        tokio::task::yield_now().await;
        engine.start(Duration::from_secs(1)).unwrap();

        assert_eq!(engine.current_tick(), Tick::ZERO);

        // And the new run counts from there, rather than resuming the old one.
        tokio::task::yield_now().await;
        assert_eq!(engine.current_tick(), Tick::new(1));
    }

    #[tokio::test]
    async fn beginning_a_run_discards_the_previous_runs_absolute_tick() {
        // ADR 0012's first-Tick rule belongs to beginning a Playback run, not
        // to the clock that happens to drive it, so every path that begins one
        // opens the same way. The engine is carried far enough into a first run
        // that a counter left standing would be plainly visible, and the run
        // begun after it must still open at absolute Tick `0` with no last Tick
        // behind it for the clock to schedule against.
        let engine = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 3)),
            InMemoryOutputAdapter::default(),
        );
        engine.activate_for_test();

        for executed in 0..3u64 {
            engine
                .clock_tick(scheduled(
                    Duration::from_secs(executed),
                    Duration::from_secs(executed),
                ))
                .expect("a scheduled Tick runs");
        }
        assert_eq!(engine.current_tick(), Tick::new(3));

        engine.activate_for_test();

        assert_eq!(engine.current_tick(), Tick::ZERO);
        assert!(engine.inner.lock().unwrap().last_tick_at.is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn retuning_keeps_the_absolute_tick_of_the_run_it_retunes() {
        // Retuning changes the Tick period of the run already in progress; it
        // does not end that run. Resetting the counter here would silently
        // restart every Tick-reading Function's cycle each time the tempo moved.
        let engine = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 3)),
            InMemoryOutputAdapter::default(),
        );

        engine.start(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;
        assert_eq!(engine.current_tick(), Tick::new(1));

        engine.retune(Duration::from_secs(2)).unwrap();

        assert_eq!(engine.current_tick(), Tick::new(1));
    }

    #[tokio::test]
    async fn live_editing_changes_the_next_unsampled_tick() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        engine.clock_tick(scheduled(Duration::ZERO, Duration::ZERO));
        source.set(cell(source.grid(), 6), "D").unwrap();
        engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(1)));

        // ADR 0012's other half of Live Editing: the edit lands in the next
        // Source Snapshot because a Snapshot is taken per Tick, and the run
        // keeps counting, because editing the Source is not starting a
        // Playback run.
        assert_eq!(engine.current_tick(), Tick::new(2));
        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.command_lists()[0][0], note_on(0, 0x7F, 60));
        assert_eq!(adapter.command_lists()[1][0], note_on(0, 0x7F, 62));
    }

    #[tokio::test]
    async fn repeated_commands_are_dispatched_as_exact_tick_lists() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source, adapter.clone());
        engine.activate_for_test();

        engine.clock_tick(scheduled(Duration::ZERO, Duration::ZERO));
        engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(1)));

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.command_lists()[0].len(), 1);
        assert_eq!(adapter.command_lists()[0], adapter.command_lists()[1]);
    }

    #[tokio::test]
    async fn an_inactive_terminal_root_reaches_the_output_adapter_as_an_empty_command_list() {
        let source = SourceCommander::new(Grid::new(10, 3));
        // The Raw Play has no Bang anywhere in the Source, so nothing
        // activates its root.
        write(&source, 0, "!>007FC4");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source, adapter.clone());
        engine.activate_for_test();

        engine.clock_tick(scheduled(Duration::ZERO, Duration::ZERO));

        // The engine submits once for every Tick it runs, so the proof is not
        // a missing submission but an empty one: the Tick reached the adapter
        // and carried no command.
        assert_eq!(adapter.command_lists(), vec![Vec::<OutputCommand>::new()]);
    }

    #[tokio::test]
    async fn two_active_terminal_roots_dispatch_in_tick_plan_order_within_one_submission() {
        let source = SourceCommander::new(Grid::new(10, 3));
        // One Bang between the two roots activates both: the row above it is
        // its north anchor and the row below it is its south anchor.
        write(&source, 0, "!>0001C4");
        write(&source, 10, "**");
        write(&source, 20, "!>017FA4");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source, adapter.clone());
        engine.activate_for_test();

        engine.clock_tick(scheduled(Duration::ZERO, Duration::ZERO));

        assert_eq!(
            adapter.command_lists(),
            vec![vec![note_on(0, 1, 60), note_on(1, 0x7F, 69)]]
        );
    }

    #[tokio::test]
    async fn a_timed_play_starts_in_tick_plan_order_and_stops_at_the_tick_its_length_names() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC402");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        erase(&source, 10, 2);
        for tick in 1..=3 {
            run_tick(&engine, tick);
        }

        // The start is delivered in the Tick that planned it and the stop at
        // the beginning of Tick `0 + 02`, with the Tick between them carrying
        // neither: a submission per executed Tick, so an empty one is the
        // engine saying this Tick owed no MIDI rather than not having run.
        assert_eq!(
            adapter.command_lists(),
            vec![
                vec![note_on(0, 0x7F, 60)],
                vec![],
                vec![stop(0, 60)],
                vec![],
            ]
        );
        assert!(!engine.holds_timed_ownership());
    }

    #[tokio::test]
    async fn a_repeated_timed_play_stops_the_instance_it_replaces_and_retires_its_expiry() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC403");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        // The Bang stands, so the root plays again on each of these three
        // Ticks, each command replacing the instance the last one owned.
        for tick in 0..=2 {
            run_tick(&engine, tick);
        }
        erase(&source, 10, 2);
        for tick in 3..=5 {
            run_tick(&engine, tick);
        }

        assert_eq!(
            adapter.command_lists(),
            vec![
                vec![note_on(0, 0x7F, 60)],
                vec![stop(0, 60), note_on(0, 0x7F, 60)],
                vec![stop(0, 60), note_on(0, 0x7F, 60)],
                // Ticks 3 and 4 are where the first two commands scheduled
                // their stops. Both claims were retired by the replacement
                // that followed them, so neither stop is delivered — and only
                // the surviving claim, from Tick 2, stops at Tick 5.
                vec![],
                vec![],
                vec![stop(0, 60)],
            ]
        );
    }

    #[tokio::test]
    async fn a_timed_play_with_velocity_zero_stops_the_note_and_schedules_nothing() {
        let source = SourceCommander::new(Grid::new(10, 3));
        // A stop still carries and validates its length, and the length still
        // schedules nothing: ADR 0016 keeps the arity fixed either way.
        write(&source, 0, "!~0000C405");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        erase(&source, 10, 2);
        for tick in 1..=6 {
            run_tick(&engine, tick);
        }

        assert_eq!(adapter.command_lists()[0], vec![stop(0, 60)]);
        assert!(
            adapter.command_lists()[1..]
                .iter()
                .all(|commands| commands.is_empty()),
            "{:?}",
            adapter.command_lists()
        );
        assert!(!engine.holds_timed_ownership());
    }

    #[tokio::test]
    async fn a_timed_play_with_no_length_emits_nothing_and_leaves_the_note_it_finds_standing() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC403");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        // A lifetime of no Ticks, live-edited into the length operand. It is
        // not a stop, so the note started at Tick 0 keeps both its sound and
        // the stop it is due.
        write(&source, 8, "00");
        run_tick(&engine, 1);
        erase(&source, 10, 2);
        for tick in 2..=3 {
            run_tick(&engine, tick);
        }

        assert_eq!(
            adapter.command_lists(),
            vec![
                vec![note_on(0, 0x7F, 60)],
                vec![],
                vec![],
                vec![stop(0, 60)],
            ]
        );
    }

    #[tokio::test]
    async fn a_stale_expiry_cannot_stop_the_note_claimed_after_it() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC403");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        // Tick 0 claims the voice until Tick 3. Tick 1 stops it explicitly,
        // which retires that claim while leaving its scheduled stop where it
        // was, and Tick 2 claims the same voice again until Tick 7.
        run_tick(&engine, 0);
        write(&source, 4, "00");
        run_tick(&engine, 1);
        write(&source, 4, "7F");
        write(&source, 8, "05");
        run_tick(&engine, 2);
        erase(&source, 10, 2);
        for tick in 3..=7 {
            run_tick(&engine, tick);
        }

        assert_eq!(
            adapter.command_lists(),
            vec![
                vec![note_on(0, 0x7F, 60)],
                vec![stop(0, 60)],
                vec![note_on(0, 0x7F, 60)],
                // Tick 3 is where the first claim's stop was due. Delivering
                // it here would cut the note claimed at Tick 2 short by four
                // Ticks, which is exactly what its claim exists to prevent.
                vec![],
                vec![],
                vec![],
                vec![],
                vec![stop(0, 60)],
            ]
        );
    }

    #[tokio::test]
    async fn a_stop_due_this_tick_is_delivered_before_the_play_commands_that_tick_plans() {
        let source = SourceCommander::new(Grid::new(10, 5));
        write(&source, 0, "!~007FC401");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        // Retire the Timed root and activate a Raw Play of the note it owns,
        // so Tick 1 carries both a stop due from Tick 0 and a command of its
        // own for the voice that stop names.
        erase(&source, 10, 2);
        write(&source, 30, "!>007FC4");
        write(&source, 40, "**");
        run_tick(&engine, 1);

        // The order is the whole of what ADR 0016 asks of the Tick a stop
        // comes due at. Delivered the other way round, the note this Tick
        // sounds is silenced by the stop of the note it succeeds.
        assert_eq!(
            adapter.command_lists(),
            vec![
                vec![note_on(0, 0x7F, 60)],
                vec![stop(0, 60), note_on(0, 0x7F, 60)],
            ]
        );
        // The Raw note that outlives the stop is the Source's to end.
        assert!(!engine.holds_timed_ownership());
    }

    #[tokio::test]
    async fn two_notes_on_one_channel_are_owned_and_stopped_independently() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC403");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        // A second note on the channel the first is sounding on. Timed Play is
        // polyphonic, and ADR 0016 gives one voice per channel to Monophonic
        // Play alone, so this starts a note rather than replacing one.
        write(&source, 6, "E4");
        run_tick(&engine, 1);
        erase(&source, 10, 2);
        for tick in 2..=4 {
            run_tick(&engine, tick);
        }

        assert_eq!(
            adapter.command_lists(),
            vec![
                vec![note_on(0, 0x7F, 60)],
                // Owned per channel alone, this Tick would stop C4 to sound
                // E4, cutting a note the Source gave three Ticks short by two.
                vec![note_on(0, 0x7F, 64)],
                vec![],
                vec![stop(0, 60)],
                vec![stop(0, 64)],
            ]
        );
        assert!(!engine.holds_timed_ownership());
    }

    #[tokio::test]
    async fn one_note_on_two_channels_is_owned_and_stopped_independently() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC403");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        // The same note on a second channel, which is a second instrument
        // sounding it: the channel discriminates as the note does.
        write(&source, 2, "01");
        run_tick(&engine, 1);
        erase(&source, 10, 2);
        for tick in 2..=4 {
            run_tick(&engine, tick);
        }

        assert_eq!(
            adapter.command_lists(),
            vec![
                vec![note_on(0, 0x7F, 60)],
                vec![note_on(1, 0x7F, 60)],
                vec![],
                vec![stop(0, 60)],
                vec![stop(1, 60)],
            ]
        );
        assert!(!engine.holds_timed_ownership());
    }

    #[tokio::test]
    async fn two_timed_plays_for_one_voice_within_one_tick_leave_the_second_owning_it() {
        let source = SourceCommander::new(Grid::new(10, 3));
        // One Bang between the two roots activates both, so one Tick Plan
        // carries two commands for the same voice.
        write(&source, 0, "!~007FC405");
        write(&source, 10, "**");
        write(&source, 20, "!~007FC402");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        erase(&source, 10, 2);
        for tick in 1..=5 {
            run_tick(&engine, tick);
        }

        assert_eq!(
            adapter.command_lists(),
            vec![
                // The second command replaces what the first started, inside
                // the one submission the Tick makes: ownership is resolved in
                // Tick Plan order, not once per Tick.
                vec![note_on(0, 0x7F, 60), stop(0, 60), note_on(0, 0x7F, 60)],
                vec![],
                vec![stop(0, 60)],
                // Tick 5 is where the first command's stop was due. Its claim
                // was retired before the Tick that scheduled it had ended.
                vec![],
                vec![],
                vec![],
            ]
        );
        assert!(!engine.holds_timed_ownership());
    }

    #[tokio::test]
    async fn a_refused_submission_leaves_the_schedule_standing_for_the_next_tick() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC402");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        erase(&source, 10, 2);
        run_tick(&engine, 1);
        // The adapter refuses the Tick the stop is due at. The schedule
        // describes what is sounding, so a stop no device received leaves the
        // note it stops owned: an adapter that survives a refusal is one this
        // engine still owes a Note Off.
        adapter.fail_next_submission("output unavailable");
        run_tick(&engine, 2);
        assert!(engine.holds_timed_ownership());
        run_tick(&engine, 3);

        // Three submissions were accepted: the start, the Tick between, and
        // the stop the next executed Tick drains again.
        assert_eq!(
            adapter.command_lists(),
            vec![vec![note_on(0, 0x7F, 60)], vec![], vec![stop(0, 60)]]
        );
        assert!(!engine.holds_timed_ownership());
        assert_eq!(
            engine.diagnostics(),
            vec![PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(
                "output unavailable"
            ))]
        );
    }

    #[tokio::test]
    async fn a_scheduled_stop_is_due_at_an_absolute_tick_rather_than_at_a_clock_tick() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!~007FC402");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        erase(&source, 10, 2);
        // A Tick the engine declines consumes no absolute Tick, so it moves
        // nothing towards the stop either: the note lasts the two Ticks it
        // was given however many clock ticks pass.
        assert!(
            engine
                .clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(5)))
                .is_none()
        );
        run_tick(&engine, 1);
        run_tick(&engine, 2);

        assert_eq!(
            adapter.command_lists(),
            vec![vec![note_on(0, 0x7F, 60)], vec![], vec![stop(0, 60)]]
        );
        assert_eq!(
            engine.diagnostics(),
            vec![PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_secs(1),
                observed_at: Duration::from_secs(5),
            }]
        );
    }

    #[tokio::test]
    async fn raw_play_notes_never_enter_timed_ownership() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        run_tick(&engine, 0);
        erase(&source, 10, 2);
        for tick in 1..=4 {
            run_tick(&engine, tick);
        }

        // Raw Play leaves Note Off under Source control, so nothing this
        // engine owns can stop a note the Source did not ask to stop.
        assert_eq!(adapter.command_lists()[0], vec![note_on(0, 0x7F, 60)]);
        assert!(
            adapter.command_lists()[1..]
                .iter()
                .all(|commands| commands.is_empty()),
            "{:?}",
            adapter.command_lists()
        );
        assert!(!engine.holds_timed_ownership());
    }

    #[tokio::test]
    async fn every_lifecycle_action_that_silences_output_clears_the_timed_schedule() {
        // Each of these silences the output the schedule describes, so a stop
        // left standing would be delivered to a device that has already been
        // told to stop everything, or into a run that never started the note.
        let (engine, _) = engine_owning_a_timed_note();
        engine.stop();
        assert!(!engine.holds_timed_ownership());

        let (engine, _) = engine_owning_a_timed_note();
        engine.disconnect();
        assert!(!engine.holds_timed_ownership());

        // Beginning a run restarts the absolute Tick at zero, so an inherited
        // stop would come due before the note it stops had been played.
        let (engine, _) = engine_owning_a_timed_note();
        engine.activate_for_test();
        assert!(!engine.holds_timed_ownership());

        // Dropping the final handle stops the run, and stopping is what clears
        // the schedule. What is left to observe once the engine is gone is the
        // safety the owned note is silenced by.
        let (engine, adapter) = engine_owning_a_timed_note();
        drop(engine);
        assert_eq!(adapter.all_notes_off_count(), 1);
        assert_eq!(adapter.command_lists(), vec![vec![note_on(0, 0x7F, 60)]]);
    }

    #[tokio::test]
    async fn missed_deadline_is_dropped_and_the_next_scheduled_tick_runs() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source, adapter.clone());
        engine.activate_for_test();

        let missed = engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(2)));
        let resumed = engine.clock_tick(scheduled(Duration::from_secs(3), Duration::from_secs(3)));

        assert!(missed.is_none());
        assert!(resumed.is_some());
        assert_eq!(adapter.command_lists().len(), 1);
        assert_eq!(
            engine.diagnostics(),
            vec![PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_secs(1),
                observed_at: Duration::from_secs(2),
            }]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn playback_clock_reports_each_overrun_and_resumes_without_wall_clock_sleep() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source, adapter.clone());
        engine.start(Duration::from_secs(1)).unwrap();

        tokio::task::yield_now().await;
        time::advance(Duration::from_secs(3)).await;
        for _ in 0..4 {
            tokio::task::yield_now().await;
        }

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(engine.observe().diagnostics.len(), 2);

        engine.stop();
        tokio::task::yield_now().await;

        assert_eq!(engine.observe().state, PlaybackState::Stopped);
        assert_eq!(adapter.all_notes_off_count(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn cancelled_clock_cannot_stop_or_tick_restarted_playback() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source, adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;
        assert_eq!(adapter.command_lists().len(), 1);

        engine.stop();
        engine.start(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;

        assert_eq!(engine.observe().state, PlaybackState::Playing);
        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.all_notes_off_count(), 1);

        engine.stop();
        tokio::task::yield_now().await;
        assert_eq!(engine.observe().state, PlaybackState::Stopped);
        assert_eq!(adapter.all_notes_off_count(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn retuning_a_restart_does_not_inherit_the_previous_runs_phase() {
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(SourceCommander::new(Grid::new(1, 1)), adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;
        engine.stop();

        engine.start(Duration::from_secs(1)).unwrap();
        engine.retune(Duration::from_secs(1)).unwrap();
        tokio::task::yield_now().await;

        assert_eq!(adapter.command_lists().len(), 2);
    }

    #[tokio::test]
    async fn adapter_failure_does_not_roll_back_source_or_stop_playback() {
        let source = SourceCommander::new(Grid::new(10, 4));
        write(&source, 0, ".+0102");
        write(&source, 20, "!>007FC4");
        write(&source, 30, "**");
        let adapter = InMemoryOutputAdapter::default();
        adapter.fail_next_submission("output unavailable");
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        let failed_dispatch = engine
            .clock_tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("Source Tick still succeeds");

        assert_eq!(&source.snapshot()[10..12], "03");
        assert_eq!(failed_dispatch.play_commands.len(), 1);
        assert!(engine.is_playing());
        assert_eq!(
            engine.diagnostics(),
            vec![PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(
                "output unavailable"
            ))]
        );

        engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(1)));
        assert_eq!(adapter.command_lists().len(), 1);
    }

    #[tokio::test]
    async fn stopping_and_disconnecting_each_send_all_notes_off() {
        let stopped_adapter = InMemoryOutputAdapter::default();
        let stopped = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 2)),
            stopped_adapter.clone(),
        );
        stopped.start(Duration::from_secs(1)).unwrap();
        stopped.stop();

        let disconnected_adapter = InMemoryOutputAdapter::default();
        let disconnected = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 2)),
            disconnected_adapter.clone(),
        );
        disconnected.start(Duration::from_secs(1)).unwrap();
        disconnected.disconnect();

        assert_eq!(stopped_adapter.all_notes_off_count(), 1);
        assert_eq!(disconnected_adapter.all_notes_off_count(), 1);
        assert!(!stopped.is_playing());
    }

    #[test]
    fn start_rejects_invalid_environment_without_changing_state() {
        let engine = PlaybackEngine::new(
            SourceCommander::new(Grid::new(1, 1)),
            InMemoryOutputAdapter::default(),
        );

        assert_eq!(
            engine.start(Duration::ZERO),
            Err(PlaybackStartError::ZeroTickPeriod)
        );
        assert_eq!(
            engine.start(Duration::from_secs(1)),
            Err(PlaybackStartError::RuntimeUnavailable)
        );
        assert_eq!(engine.observe().state, PlaybackState::Stopped);
    }

    #[test]
    fn every_start_failure_is_reported_as_a_start_failure_diagnostic() {
        let engine = PlaybackEngine::new(
            SourceCommander::new(Grid::new(1, 1)),
            InMemoryOutputAdapter::default(),
        );

        assert!(engine.start(Duration::ZERO).is_err());
        assert!(engine.start(Duration::from_secs(1)).is_err());

        assert_eq!(
            engine.observe().diagnostics,
            vec![
                PlaybackDiagnostic::StartFailure {
                    message: "Tick period must be greater than zero".to_string(),
                },
                PlaybackDiagnostic::StartFailure {
                    message: "Playback requires a Tokio runtime".to_string(),
                },
            ]
        );
    }

    #[test]
    fn unexpected_clock_termination_stops_playback_and_reports_failure() {
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(SourceCommander::new(Grid::new(1, 1)), adapter.clone());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(async {
            engine.start(Duration::from_secs(1)).unwrap();
            tokio::task::yield_now().await;
        });

        drop(runtime);

        let observation = engine.observe();
        assert_eq!(observation.state, PlaybackState::Stopped);
        assert_eq!(
            observation.diagnostics,
            vec![PlaybackDiagnostic::ClockFailure {
                message: "Playback clock terminated unexpectedly".to_string(),
            }]
        );
        assert_eq!(adapter.all_notes_off_count(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn start_is_idempotent_and_observation_drains_diagnostics() {
        let source = SourceCommander::new(Grid::new(10, 2));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let adapter = InMemoryOutputAdapter::default();
        adapter.fail_next_submission("device lost");
        let engine = PlaybackEngine::new(source, adapter.clone());

        engine.start(Duration::from_secs(1)).unwrap();
        engine.start(Duration::from_secs(9)).unwrap();
        assert_eq!(
            engine.start(Duration::ZERO),
            Err(PlaybackStartError::ZeroTickPeriod)
        );
        tokio::task::yield_now().await;

        assert_eq!(adapter.command_lists().len(), 0);
        let first = engine.observe();
        assert_eq!(first.state, PlaybackState::Playing);
        assert_eq!(
            first.diagnostics,
            vec![
                PlaybackDiagnostic::StartFailure {
                    message: "Tick period must be greater than zero".to_string(),
                },
                PlaybackDiagnostic::OutputFailure(OutputAdapterError::new("device lost")),
            ]
        );
        assert!(engine.observe().diagnostics.is_empty());
        engine.stop();
    }

    #[tokio::test(start_paused = true)]
    async fn dropping_the_final_handle_stops_playback_safely() {
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(SourceCommander::new(Grid::new(1, 1)), adapter.clone());
        engine.start(Duration::from_secs(1)).unwrap();

        drop(engine);
        tokio::task::yield_now().await;

        assert_eq!(adapter.all_notes_off_count(), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_the_final_handle_during_a_tick_completes_playback_safety() {
        let source = SourceCommander::new(Grid::new(10, 2));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let control = BlockingOutputControl::default();
        let engine = PlaybackEngine::new(
            source,
            BlockingOutputAdapter {
                control: control.clone(),
            },
        );
        engine.start(Duration::from_secs(1)).unwrap();
        control.wait_for_delivery();

        let (dropped_tx, dropped_rx) = mpsc::channel();
        let drop_thread = std::thread::spawn(move || {
            drop(engine);
            dropped_tx.send(()).unwrap();
        });
        let drop_returned_before_delivery =
            dropped_rx.recv_timeout(Duration::from_millis(50)).is_ok();
        control.release_delivery();
        drop_thread.join().unwrap();

        assert!(!drop_returned_before_delivery);
        assert_eq!(control.all_notes_off_count(), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn clock_failure_remains_observable_after_output_panics() {
        let source = SourceCommander::new(Grid::new(10, 2));
        write(&source, 0, "!>007FC4");
        write(&source, 10, "**");
        let delivery_started = Arc::new(AtomicBool::new(false));
        let engine = PlaybackEngine::new(
            source,
            PanickingOutputAdapter {
                delivery_started: delivery_started.clone(),
            },
        );
        engine.start(Duration::from_secs(1)).unwrap();
        while !delivery_started.load(Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }
        tokio::task::yield_now().await;

        let observation = engine.observe();

        assert_eq!(observation.state, PlaybackState::Stopped);
        assert_eq!(
            observation.diagnostics,
            vec![PlaybackDiagnostic::ClockFailure {
                message: "Playback clock terminated unexpectedly".to_string(),
            }]
        );
    }
}
