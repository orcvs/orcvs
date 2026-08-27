use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time;
use tokio_util::sync::CancellationToken;

use crate::source::{PlayCommand, SourceCommander, TickResult};

pub trait OutputAdapter {
    fn submit(&mut self, commands: &[PlayCommand]) -> Result<(), OutputAdapterError>;
    fn all_notes_off(&mut self) -> Result<(), OutputAdapterError>;
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScheduledTick {
    pub scheduled_at: Duration,
    pub observed_at: Duration,
    pub period: Duration,
}

impl ScheduledTick {
    pub fn new(scheduled_at: Duration, observed_at: Duration, period: Duration) -> Self {
        Self {
            scheduled_at,
            observed_at,
            period,
        }
    }

    fn is_overrun(self) -> bool {
        self.observed_at >= self.scheduled_at + self.period
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

pub struct PlaybackEngine<A> {
    source: SourceCommander,
    adapter: A,
    playing: bool,
    connected: bool,
    diagnostics: Vec<PlaybackDiagnostic>,
    output_failure: Option<OutputAdapterError>,
    generation: u64,
    cancellation: Option<CancellationToken>,
}

impl<A: OutputAdapter> PlaybackEngine<A> {
    pub fn new(source: SourceCommander, adapter: A) -> Self {
        Self {
            source,
            adapter,
            playing: false,
            connected: true,
            diagnostics: Vec::new(),
            output_failure: None,
            generation: 0,
            cancellation: None,
        }
    }

    pub fn start(&mut self) {
        if let Some(previous) = self.cancellation.take() {
            previous.cancel();
        }
        self.generation = self.generation.wrapping_add(1);
        self.playing = true;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn diagnostics(&self) -> &[PlaybackDiagnostic] {
        &self.diagnostics
    }

    pub fn output_failure(&self) -> Option<&OutputAdapterError> {
        self.output_failure.as_ref()
    }

    pub fn stop(&mut self) {
        if let Some(cancellation) = self.cancellation.take() {
            cancellation.cancel();
        }
        if self.playing {
            if self.connected {
                self.send_all_notes_off();
            }
            self.playing = false;
        }
    }

    pub fn disconnect(&mut self) {
        if self.connected {
            self.send_all_notes_off();
            self.connected = false;
        }
    }

    fn send_all_notes_off(&mut self) {
        if let Err(error) = self.adapter.all_notes_off() {
            self.record_output_failure(error);
        }
    }

    fn record_output_failure(&mut self, error: OutputAdapterError) {
        self.output_failure = Some(error.clone());
        self.diagnostics
            .push(PlaybackDiagnostic::OutputFailure(error));
    }

    pub fn clock_tick(&mut self, scheduled_tick: ScheduledTick) -> Option<TickResult> {
        if !self.playing {
            return None;
        }
        if scheduled_tick.is_overrun() {
            self.diagnostics.push(PlaybackDiagnostic::Overrun {
                scheduled_at: scheduled_tick.scheduled_at,
                observed_at: scheduled_tick.observed_at,
            });
            return None;
        }

        let tick = self.source.execute();
        if self.connected {
            if let Err(error) = self.adapter.submit(&tick.plan.play_commands) {
                self.record_output_failure(error);
            }
        }
        Some(tick)
    }

    fn clock_tick_for_generation(
        &mut self,
        generation: u64,
        scheduled_tick: ScheduledTick,
    ) -> Option<TickResult> {
        if self.generation != generation {
            return None;
        }

        self.clock_tick(scheduled_tick)
    }

    fn stop_generation(&mut self, generation: u64) {
        if self.generation == generation {
            self.stop();
        }
    }
}

impl<B: crate::midi::MidiBackend> PlaybackEngine<crate::midi::MidiOutputAdapter<B>> {
    pub fn midi_destinations(
        &mut self,
    ) -> Result<Vec<crate::midi::MidiDestination>, crate::midi::MidiError> {
        self.adapter.destinations()
    }

    pub fn select_midi_destination(
        &mut self,
        destination_id: &crate::midi::MidiDestinationId,
    ) -> Result<(), crate::midi::MidiError> {
        let selection = self.adapter.select(destination_id)?;
        self.output_failure = None;
        if let Some(error) = selection.safety_failure() {
            self.record_output_failure(OutputAdapterError::new(error.message));
        }
        self.connected = true;
        Ok(())
    }

    pub fn selected_midi_destination_id(&self) -> Option<&crate::midi::MidiDestinationId> {
        self.adapter.selected_destination_id()
    }
}

impl<A: OutputAdapter + Send + 'static> PlaybackEngine<A> {
    pub fn start_clock(engine: Arc<Mutex<Self>>, tick_period: Duration) -> JoinHandle<()> {
        let (generation, cancellation) = {
            let mut engine = engine.lock().unwrap();
            engine.start();
            let cancellation = CancellationToken::new();
            engine.cancellation = Some(cancellation.clone());
            (engine.generation, cancellation)
        };

        tokio::spawn(async move {
            let epoch = time::Instant::now();
            let mut interval = time::interval_at(epoch, tick_period);
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Burst);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => {
                        engine.lock().unwrap().stop_generation(generation);
                        break;
                    },
                    scheduled = interval.tick() => {
                        let scheduled_at = scheduled.duration_since(epoch);
                        let observed_at = time::Instant::now().duration_since(epoch);
                        let scheduled_tick = ScheduledTick::new(
                            scheduled_at,
                            observed_at,
                            tick_period,
                        );
                        engine.lock().unwrap().clock_tick_for_generation(
                            generation,
                            scheduled_tick,
                        );
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

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

    fn write(source: &SourceCommander, start: usize, content: &str) {
        for (offset, cell) in content.chars().enumerate() {
            source.set(start + offset, &cell.to_string()).unwrap();
        }
    }

    fn scheduled(scheduled_at: Duration, observed_at: Duration) -> ScheduledTick {
        ScheduledTick::new(scheduled_at, observed_at, Duration::from_secs(1))
    }

    #[test]
    fn clock_tick_commits_source_before_submitting_play_commands() {
        let source = SourceCommander::new(Grid::new(10, 4));
        write(&source, 0, "++0102");
        write(&source, 20, ">>07FC4");
        let mut engine =
            PlaybackEngine::new(source.clone(), RecordingAdapter::observing(source.clone()));
        engine.start();

        let tick = engine
            .clock_tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("scheduled Tick runs");

        assert_eq!(&engine.adapter.source_at_submission[0][10..12], "03");
        assert_eq!(&source.snapshot()[10..12], "03");
        assert_eq!(engine.adapter.command_lists, vec![tick.plan.play_commands]);
    }

    #[test]
    fn live_editing_changes_the_next_unsampled_tick() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, ">>07FC4");
        let adapter = InMemoryOutputAdapter::default();
        let mut engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.start();

        engine.clock_tick(scheduled(Duration::ZERO, Duration::ZERO));
        source.set(5, "D").unwrap();
        engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(1)));

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.command_lists()[0][0].note, 60);
        assert_eq!(adapter.command_lists()[1][0].note, 62);
    }

    #[test]
    fn repeated_commands_are_dispatched_as_exact_tick_lists() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, ">>07FC4");
        let adapter = InMemoryOutputAdapter::default();
        let mut engine = PlaybackEngine::new(source, adapter.clone());
        engine.start();

        engine.clock_tick(scheduled(Duration::ZERO, Duration::ZERO));
        engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(1)));

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.command_lists()[0], adapter.command_lists()[1]);
    }

    #[test]
    fn missed_deadline_is_dropped_and_the_next_scheduled_tick_runs() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, ">>07FC4");
        let adapter = InMemoryOutputAdapter::default();
        let mut engine = PlaybackEngine::new(source, adapter.clone());
        engine.start();

        let missed = engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(2)));
        let resumed = engine.clock_tick(scheduled(Duration::from_secs(3), Duration::from_secs(3)));

        assert!(missed.is_none());
        assert!(resumed.is_some());
        assert_eq!(adapter.command_lists().len(), 1);
        assert_eq!(
            engine.diagnostics(),
            &[PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_secs(1),
                observed_at: Duration::from_secs(2),
            }]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn playback_clock_reports_each_overrun_and_resumes_without_wall_clock_sleep() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, ">>07FC4");
        let adapter = InMemoryOutputAdapter::default();
        let engine = Arc::new(Mutex::new(PlaybackEngine::new(source, adapter.clone())));
        let clock = PlaybackEngine::start_clock(engine.clone(), Duration::from_secs(1));

        tokio::task::yield_now().await;
        time::advance(Duration::from_secs(3)).await;
        for _ in 0..4 {
            tokio::task::yield_now().await;
        }

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(engine.lock().unwrap().diagnostics().len(), 2);

        engine.lock().unwrap().stop();
        clock.await.unwrap();

        assert!(!engine.lock().unwrap().is_playing());
        assert_eq!(adapter.all_notes_off_count(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn cancelled_clock_cannot_stop_or_tick_restarted_playback() {
        let source = SourceCommander::new(Grid::new(10, 3));
        write(&source, 0, ">>07FC4");
        let adapter = InMemoryOutputAdapter::default();
        let engine = Arc::new(Mutex::new(PlaybackEngine::new(source, adapter.clone())));

        let old_clock = PlaybackEngine::start_clock(engine.clone(), Duration::from_secs(1));
        tokio::task::yield_now().await;
        assert_eq!(adapter.command_lists().len(), 1);

        engine.lock().unwrap().stop();
        let new_clock = PlaybackEngine::start_clock(engine.clone(), Duration::from_secs(1));
        old_clock.await.unwrap();
        tokio::task::yield_now().await;

        assert!(engine.lock().unwrap().is_playing());
        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(adapter.all_notes_off_count(), 1);

        engine.lock().unwrap().stop();
        new_clock.await.unwrap();
        assert!(!engine.lock().unwrap().is_playing());
        assert_eq!(adapter.all_notes_off_count(), 2);
    }

    #[test]
    fn adapter_failure_does_not_roll_back_source_or_stop_playback() {
        let source = SourceCommander::new(Grid::new(10, 4));
        write(&source, 0, "++0102");
        write(&source, 20, ">>07FC4");
        let adapter = InMemoryOutputAdapter::default();
        adapter.fail_next_submission("output unavailable");
        let mut engine = PlaybackEngine::new(source.clone(), adapter.clone());
        engine.start();

        let failed_dispatch = engine
            .clock_tick(scheduled(Duration::ZERO, Duration::ZERO))
            .expect("Source Tick still succeeds");

        assert_eq!(&source.snapshot()[10..12], "03");
        assert_eq!(failed_dispatch.plan.play_commands.len(), 1);
        assert!(engine.is_playing());
        assert_eq!(
            engine.diagnostics(),
            &[PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(
                "output unavailable"
            ))]
        );

        engine.clock_tick(scheduled(Duration::from_secs(1), Duration::from_secs(1)));
        assert_eq!(adapter.command_lists().len(), 1);
    }

    #[test]
    fn stopping_and_disconnecting_each_send_all_notes_off() {
        let stopped_adapter = InMemoryOutputAdapter::default();
        let mut stopped = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 2)),
            stopped_adapter.clone(),
        );
        stopped.start();
        stopped.stop();

        let disconnected_adapter = InMemoryOutputAdapter::default();
        let mut disconnected = PlaybackEngine::new(
            SourceCommander::new(Grid::new(10, 2)),
            disconnected_adapter.clone(),
        );
        disconnected.start();
        disconnected.disconnect();

        assert_eq!(stopped_adapter.all_notes_off_count(), 1);
        assert_eq!(disconnected_adapter.all_notes_off_count(), 1);
        assert!(!stopped.is_playing());
    }
}
