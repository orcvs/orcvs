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

use crate::source::{PlayCommand, SourceCommander, TickResult};

pub trait OutputAdapter {
    fn submit(&mut self, commands: &[PlayCommand]) -> Result<(), OutputAdapterError>;
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

#[derive(Default)]
struct InMemoryOutputState {
    command_lists: Vec<Vec<PlayCommand>>,
    all_notes_off_count: usize,
    next_failure: Option<OutputAdapterError>,
}

#[derive(Clone, Default)]
pub struct InMemoryOutputAdapter {
    state: Arc<Mutex<InMemoryOutputState>>,
}

impl InMemoryOutputAdapter {
    pub fn command_lists(&self) -> Vec<Vec<PlayCommand>> {
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
    fn submit(&mut self, commands: &[PlayCommand]) -> Result<(), OutputAdapterError> {
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
        let selection = inner.adapter.select(destination_id)?;
        inner.last_output_failure = None;
        if let Some(error) = selection.safety_failure() {
            inner.record_output_failure(OutputAdapterError::new(error.message));
        }
        inner.connected = true;
        Ok(())
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

    fn tick(&mut self, generation: u64, timing: TickTiming) -> Option<TickResult> {
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
        let tick = self.source.execute();
        if self.connected {
            match self.adapter.submit(&tick.plan.play_commands) {
                Ok(()) => self.last_output_failure = None,
                Err(error) => self.record_output_failure(error),
            }
        }
        Some(tick)
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
    }

    #[cfg(test)]
    fn clock_tick(&self, timing: TickTiming) -> Option<TickResult> {
        let mut inner = self.inner.lock().unwrap();
        let generation = inner.generation;
        inner.tick(generation, timing)
    }

    #[cfg(test)]
    fn activate_for_test(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.generation = inner.generation.wrapping_add(1);
        inner.playing = true;
    }

    #[cfg(test)]
    fn diagnostics(&self) -> Vec<PlaybackDiagnostic> {
        self.inner.lock().unwrap().diagnostics.clone()
    }

    #[cfg(test)]
    fn is_playing(&self) -> bool {
        self.inner.lock().unwrap().playing
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
        let mut inner = lock_recover(&self.inner);
        let selection = inner.adapter.select(destination_id)?;
        inner.last_output_failure = None;
        if let Some(error) = selection.safety_failure() {
            inner.record_output_failure(OutputAdapterError::new(error.message));
        }
        inner.connected = true;
        Ok(())
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
                        lock_recover(&inner).tick(generation, TickTiming {
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
                lock_recover(&inner).tick(
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
            inner.generation = inner.generation.wrapping_add(1);
            inner.playing = true;
            inner.last_tick_at = None;
            let cancellation = CancellationToken::new();
            inner.cancellation = Some(cancellation.clone());
            (inner.generation, cancellation, Arc::downgrade(&self.inner))
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
                        lock_recover(&inner).tick(generation, TickTiming {
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
            inner.generation = inner.generation.wrapping_add(1);
            inner.playing = true;
            inner.last_tick_at = None;
            let cancellation = CancellationToken::new();
            inner.cancellation = Some(cancellation.clone());
            (inner.generation, cancellation, Arc::downgrade(&self.inner))
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
                lock_recover(&inner).tick(
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
    use crate::grid::Grid;
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
        command_lists: Vec<Vec<PlayCommand>>,
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
        fn submit(&mut self, commands: &[PlayCommand]) -> Result<(), OutputAdapterError> {
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
        fn submit(&mut self, _commands: &[PlayCommand]) -> Result<(), OutputAdapterError> {
            self.delivery_started.store(true, Ordering::SeqCst);
            panic!("test output panic");
        }

        fn all_notes_off(&mut self) -> Result<(), OutputAdapterError> {
            Ok(())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl OutputAdapter for BlockingOutputAdapter {
        fn submit(&mut self, _commands: &[PlayCommand]) -> Result<(), OutputAdapterError> {
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
        for (offset, cell) in content.chars().enumerate() {
            source.set(start + offset, &cell.to_string()).unwrap();
        }
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
        let engine =
            PlaybackEngine::new(source.clone(), RecordingAdapter::observing(source.clone()));
        engine.activate_for_test();

        let tick = engine
            .clock_tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("scheduled Tick runs");

        let inner = engine.inner.lock().unwrap();
        assert_eq!(&inner.adapter.source_at_submission[0][10..12], "03");
        assert_eq!(&source.snapshot()[10..12], "03");
        assert_eq!(inner.adapter.command_lists, vec![tick.plan.play_commands]);
    }

    #[tokio::test]
    async fn live_editing_changes_the_next_unsampled_tick() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        engine.clock_tick(scheduled(Duration::ZERO, Duration::ZERO));
        source.set(6, "D").unwrap();
        engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(1)));

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.command_lists()[0][0].note, 60);
        assert_eq!(adapter.command_lists()[1][0].note, 62);
    }

    #[tokio::test]
    async fn repeated_commands_are_dispatched_as_exact_tick_lists() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
        let adapter = InMemoryOutputAdapter::default();
        let engine = PlaybackEngine::new(source, adapter.clone());
        engine.activate_for_test();

        engine.clock_tick(scheduled(Duration::ZERO, Duration::ZERO));
        engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(1)));

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.command_lists()[0], adapter.command_lists()[1]);
    }

    #[tokio::test]
    async fn missed_deadline_is_dropped_and_the_next_scheduled_tick_runs() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, "!>007FC4");
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
        let adapter = InMemoryOutputAdapter::default();
        adapter.fail_next_submission("output unavailable");
        let engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.activate_for_test();

        let failed_dispatch = engine
            .clock_tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("Source Tick still succeeds");

        assert_eq!(&source.snapshot()[10..12], "03");
        assert_eq!(failed_dispatch.plan.play_commands.len(), 1);
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
        let source = SourceCommander::new(Grid::new(10, 1));
        write(&source, 0, "!>007FC4");
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
        let source = SourceCommander::new(Grid::new(10, 1));
        write(&source, 0, "!>007FC4");
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
