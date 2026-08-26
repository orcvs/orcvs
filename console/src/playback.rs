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
}

impl<A: OutputAdapter> PlaybackEngine<A> {
    pub fn new(source: SourceCommander, adapter: A) -> Self {
        Self {
            source,
            adapter,
            playing: false,
            connected: true,
            diagnostics: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.playing = true;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn diagnostics(&self) -> &[PlaybackDiagnostic] {
        &self.diagnostics
    }

    pub fn stop(&mut self) {
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
            self.diagnostics
                .push(PlaybackDiagnostic::OutputFailure(error));
        }
    }

    pub fn clock_tick(
        &mut self,
        scheduled_at: Duration,
        observed_at: Duration,
        tick_period: Duration,
    ) -> Option<TickResult> {
        if !self.playing {
            return None;
        }
        if observed_at >= scheduled_at + tick_period {
            self.diagnostics.push(PlaybackDiagnostic::Overrun {
                scheduled_at,
                observed_at,
            });
            return None;
        }

        let tick = self.source.execute();
        if self.connected {
            if let Err(error) = self.adapter.submit(&tick.plan.play_commands) {
                self.diagnostics
                    .push(PlaybackDiagnostic::OutputFailure(error));
            }
        }
        Some(tick)
    }
}

impl<A: OutputAdapter + Send + 'static> PlaybackEngine<A> {
    pub fn start_clock(
        engine: Arc<Mutex<Self>>,
        tick_period: Duration,
        cancellation: CancellationToken,
    ) -> JoinHandle<()> {
        engine.lock().unwrap().start();

        tokio::spawn(async move {
            let epoch = time::Instant::now();
            let mut interval = time::interval_at(epoch, tick_period);
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Burst);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => break,
                    scheduled = interval.tick() => {
                        let scheduled_at = scheduled.duration_since(epoch);
                        let observed_at = time::Instant::now().duration_since(epoch);
                        engine.lock().unwrap().clock_tick(
                            scheduled_at,
                            observed_at,
                            tick_period,
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

    #[test]
    fn clock_tick_commits_source_before_submitting_play_commands() {
        let source = SourceCommander::new(Grid::new(10, 4));
        write(&source, 0, "++0102");
        write(&source, 20, ">>07FC4");
        let mut engine =
            PlaybackEngine::new(source.clone(), RecordingAdapter::observing(source.clone()));
        engine.start();

        let tick = engine
            .clock_tick(Duration::ZERO, Duration::ZERO, Duration::from_secs(1))
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

        engine.clock_tick(Duration::ZERO, Duration::ZERO, Duration::from_secs(1));
        source.set(5, "D").unwrap();
        engine.clock_tick(
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_secs(1),
        );

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

        engine.clock_tick(Duration::ZERO, Duration::ZERO, Duration::from_secs(1));
        engine.clock_tick(
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_secs(1),
        );

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

        let missed = engine.clock_tick(
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(1),
        );
        let resumed = engine.clock_tick(
            Duration::from_secs(3),
            Duration::from_secs(3),
            Duration::from_secs(1),
        );

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
        let cancellation = CancellationToken::new();
        let clock = PlaybackEngine::start_clock(
            engine.clone(),
            Duration::from_secs(1),
            cancellation.clone(),
        );

        tokio::task::yield_now().await;
        time::advance(Duration::from_secs(3)).await;
        for _ in 0..4 {
            tokio::task::yield_now().await;
        }

        assert_eq!(adapter.command_lists().len(), 2);
        assert_eq!(engine.lock().unwrap().diagnostics().len(), 2);

        cancellation.cancel();
        clock.await.unwrap();
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
            .clock_tick(Duration::ZERO, Duration::ZERO, Duration::from_secs(1))
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

        engine.clock_tick(
            Duration::from_secs(1),
            Duration::from_secs(1),
            Duration::from_secs(1),
        );
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
