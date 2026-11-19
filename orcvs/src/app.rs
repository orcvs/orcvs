use crate::midi::{MidiOutputAdapter, MidiSelectionHandle};
use crate::opts::{Bpm, Opts};

use crate::editor::{Edit, Editor};
use crate::grid::Grid;
use crate::playback::{OutputOnlyAdapter, PlaybackDiagnostic, PlaybackEngine, PlaybackStartError};
use crate::render_frame::{RenderFrame, RenderFrameConfig};
use crate::source::{Source, SourceCommander};

pub use crate::editor::{Arrow, InputEvent, InputKey};

///
/// What one input batch asks of the console that delivered it.
///
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Handled {
    /// Whether the batch wrote to the Source.
    pub repaint: bool,
    ///
    /// The text a Copy or a Cut in the batch put on the clipboard, for the
    /// console to hand the platform. The last one in the batch wins.
    ///
    pub copied: Option<String>,
    ///
    /// How many Space presses the batch carried. Space is a Playback request
    /// the console answers through its start policy, so the batch counts it
    /// rather than toggling Playback; it still disarms a fill where it falls.
    ///
    pub playback_requests: usize,
}

///
/// One running Orcvs: its options, Source and Grid, Editor, and Playback
/// lifecycle. Output-device discovery and selection belong to the console.
///
/// `S` is the selection capability, not the output backend. MIDI construction
/// supplies a [`MidiSelectionHandle`]; output-only construction uses `()`.
///
/// Its Render Frame shows the Region its [`Editor`] holds:
///
/// ```
/// use orcvs::app::Orcvs;
///
/// // A running Orcvs runs: its Playback Engine is a task, and a task needs a
/// // runtime to be spawned on, so building one is fallible and eager.
/// let runtime = tokio::runtime::Runtime::new().unwrap();
/// let _runtime = runtime.enter();
///
/// let mut orcvs = Orcvs::new().expect("a Tokio runtime");
/// let grid = orcvs.grid();
///
/// let position = grid.position(15, 15).expect("inside the grid");
/// orcvs.editor_mut().select(position);
/// assert_eq!(orcvs.render_frame().cursor(), position);
/// ```
///
pub struct Orcvs<S = MidiSelectionHandle> {
    opts: Opts,
    editor: Editor,
    grid: Grid,

    source: SourceCommander,
    playback: PlaybackEngine,
    selection: S,
    ///
    /// Whether this Orcvs has asked its Playback Engine to be running.
    ///
    /// Not a second copy of the engine's lifecycle state: the engine publishes
    /// that, and it answers what the engine *is*. This answers what it has
    /// been *asked* to be, which is the only thing Space can toggle against.
    /// An input batch is a whole frame's events handled with nothing awaited
    /// between them, so the engine has had no turn in which to apply the
    /// previous event's request, and the published state still says what it
    /// said before the batch began.
    ///
    playback_requested: bool,
}

impl Orcvs {
    ///
    /// A running Orcvs over an empty Source on the one Grid, on the output the
    /// platform supplies.
    ///
    pub fn new() -> Result<Self, PlaybackStartError> {
        Self::with_midi_output_adapter(MidiOutputAdapter::new())
    }

    ///
    /// A running Orcvs over an empty Source on a Grid smaller than the one
    /// shape, for tests that state their Source as a few short rows.
    ///
    /// Test-only, beside [`Orcvs::new`], for the reason [`Grid::with_shape`] is.
    ///
    #[cfg(any(test, feature = "test-grid-shapes"))]
    pub fn with_shape(cols: usize, rows: usize) -> Result<Self, PlaybackStartError> {
        Self::with_source(Source::new(Grid::with_shape(cols, rows)))
    }

    ///
    /// A running Orcvs over `source`, such as a Source read back from
    /// persistence, on the output the platform supplies.
    ///
    /// ```
    /// use orcvs::app::Orcvs;
    /// use orcvs::grid::Grid;
    /// use orcvs::source::Source;
    ///
    /// # let runtime = tokio::runtime::Runtime::new().unwrap();
    /// # let _runtime = runtime.enter();
    /// let mut restored = Source::new(Grid::new());
    /// let cell = restored.grid().cell_index(0).expect("inside the Grid");
    /// restored.set(cell, "1").expect("a Cell the Source accepts");
    ///
    /// let orcvs = Orcvs::with_source(restored).expect("a Tokio runtime");
    ///
    /// // the Source arrives whole: its Cells, and the Grid it was built from
    /// let frame = orcvs.render_frame();
    /// let grid = frame.grid();
    /// assert_eq!(grid.rows(), 256);
    /// assert_eq!(grid.columns(), 256);
    /// assert_eq!(frame.at(grid.origin()).content(), Some('1'));
    /// ```
    ///
    pub fn with_source(source: Source) -> Result<Self, PlaybackStartError> {
        Self::with_source_and_midi_output_adapter(source, MidiOutputAdapter::new())
    }
}

impl Orcvs<()> {
    /// Builds output-only Playback, with no MIDI selection capability.
    ///
    /// Pass an [`OutputOnlyAdapter`]: [`MidiOutputAdapter`]
    /// is rejected here because its destination publication has no publisher on
    /// this path. Use [`Orcvs::with_midi_output_adapter`] for selectable MIDI.
    ///
    /// ```compile_fail,E0599
    /// use orcvs::app::Orcvs;
    /// use orcvs::playback::{OutputAdapter, OutputAdapterError, OutputCommand, OutputOnlyAdapter};
    ///
    /// struct Silent;
    ///
    /// impl OutputAdapter for Silent {
    ///     fn submit(
    ///         &mut self,
    ///         _: &[OutputCommand],
    ///         _: std::time::Duration,
    ///     ) -> Result<(), OutputAdapterError> {
    ///         Ok(())
    ///     }
    ///
    ///     fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// impl OutputOnlyAdapter for Silent {}
    ///
    /// let app = Orcvs::with_output_adapter(Silent).unwrap();
    /// app.midi_selection_handle();
    /// ```
    ///
    /// ```compile_fail
    /// use orcvs::app::Orcvs;
    /// use orcvs::midi::MidiOutputAdapter;
    ///
    /// let _orcvs = Orcvs::with_output_adapter(MidiOutputAdapter::new()).unwrap();
    /// ```
    pub fn with_output_adapter<A: OutputOnlyAdapter + Send + 'static>(
        adapter: A,
    ) -> Result<Self, PlaybackStartError> {
        Self::with_source_and_output_adapter(Source::new(Grid::new()), adapter)
    }

    ///
    /// A running Orcvs over `source`, delivering to `adapter` and taking the
    /// Grid the Source was built from: a Source read back from persistence
    /// carries the shape it was stored with, and the Cursor starts at that
    /// Grid's origin.
    ///
    /// Fallible because the Playback Engine is: a task owns the engine's state,
    /// and a task needs a runtime to be spawned on, so a running Orcvs is one
    /// whose engine is already running.
    ///
    /// ```
    /// use orcvs::app::Orcvs;
    /// use orcvs::grid::Grid;
    /// use orcvs::playback::InMemoryOutputAdapter;
    /// use orcvs::source::Source;
    ///
    /// # let runtime = tokio::runtime::Runtime::new().unwrap();
    /// # let _runtime = runtime.enter();
    /// let restored = Source::new(Grid::new());
    /// let orcvs = Orcvs::with_source_and_output_adapter(restored, InMemoryOutputAdapter::default())
    ///     .expect("a Tokio runtime");
    ///
    /// // the shape is the Source's, not a pair passed alongside it, and the
    /// // Cursor opens on that Grid's origin
    /// let frame = orcvs.render_frame();
    /// assert_eq!(frame.grid().rows(), 256);
    /// assert_eq!(frame.cursor(), frame.grid().origin());
    /// ```
    ///
    pub fn with_source_and_output_adapter<A: OutputOnlyAdapter + Send + 'static>(
        source: Source,
        adapter: A,
    ) -> Result<Self, PlaybackStartError> {
        let source = SourceCommander::with_source(source);
        let playback = PlaybackEngine::new(source.clone(), adapter)?;
        Ok(Self::from_playback(source, playback, ()))
    }
}

impl<S> Orcvs<S> {
    fn from_playback(source: SourceCommander, playback: PlaybackEngine, selection: S) -> Self {
        let grid = source.grid();
        let opts = Opts::new();
        Self {
            editor: Editor::new(grid, opts.sector_seam_spacing),
            grid,
            opts,
            source,
            playback,
            selection,
            playback_requested: false,
        }
    }

    ///
    /// The Source root: what a console saves the current revision into
    /// storage from, writes a Source File from, and asks whether the Source
    /// has changed.
    ///
    pub fn source(&self) -> &SourceCommander {
        &self.source
    }

    ///
    /// Takes the Playback diagnostics recorded since the last drain, for the
    /// console to report, and withdraws a request the engine has just said it
    /// is no longer carrying out.
    ///
    /// It answers with diagnostics alone. Do not hand lifecycle state back
    /// beside them or cache it here: the engine publishes it (ADR 0041), and
    /// whoever needs it reads the published value at the moment it is asked
    /// rather than the value the last frame happened to carry away.
    ///
    /// A `ClockFailure` withdraws the play request: the engine ended the run
    /// by itself — an adapter panicking out of a delivery, a grid it can no
    /// longer schedule — and a request left standing would make the next Space
    /// cancel a run that is already over. `ClockFailure` arrives on the one
    /// ordered stream the engine reports failures on, so this is where the
    /// request is withdrawn. Every other diagnostic leaves it alone: an
    /// Overrun or a refused device is a run continuing, not a run ending.
    ///
    pub fn drain_playback_diagnostics(&mut self) -> Vec<PlaybackDiagnostic> {
        let diagnostics = self.playback.drain_diagnostics();
        if diagnostics
            .iter()
            .any(|diagnostic| matches!(diagnostic, PlaybackDiagnostic::ClockFailure { .. }))
        {
            self.playback_requested = false;
        }
        diagnostics
    }

    ///
    /// The observation the Playback Engine last published, read without
    /// awaiting and without reaching the engine.
    ///
    /// The console reads this while drawing a frame rather than asking the
    /// engine a question: the browser main thread has no blocking receive, so
    /// a frame cannot wait for an answer at all.
    ///
    pub fn playback_observation(&self) -> crate::playback::PlaybackObservation {
        self.playback.observation()
    }

    ///
    /// A subscriber to the observation a Render Frame reads.
    ///
    /// The console cannot wait for the next Tick on the browser main thread,
    /// so it paints [`Self::playback_observation`] and wakes when this
    /// receiver moves rather than starting a second clock from the frame.
    ///
    pub fn playback_observation_watch(&self) -> crate::playback::PlaybackObservationWatch {
        self.playback.observation_watch()
    }

    ///
    /// Whether this Orcvs has asked its Playback Engine to be running: what
    /// [`Self::toggle_playback`] decides from, so it answers a request made
    /// in this frame before the engine publishes it.
    ///
    pub fn playback_requested(&self) -> bool {
        self.playback_requested
    }

    pub fn bpm(&self) -> Bpm {
        self.opts.bpm
    }

    ///
    /// Sends UDP and OSC to `destination` from the next send on. It is the
    /// console's setting, never the Source's, so a Source File plays the same
    /// wherever it is opened.
    ///
    pub fn set_network_destination(&self, destination: crate::network::NetworkDestination) {
        self.playback.set_network_destination(destination);
    }

    pub fn set_bpm(&mut self, bpm: Bpm) {
        if self.playback_requested
            && let Err(error) = self.playback.retune(bpm.tick_period())
        {
            self.playback.report_retune_error(error);
            return;
        }
        self.opts.bpm = bpm;
    }

    ///
    /// The Region, with its Cursor, and the edits made through it.
    ///
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    ///
    /// The Editor, for a pointer that selects or spans the Region and for
    /// input that went elsewhere and so disarms a fill.
    ///
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

    ///
    /// The Grid this running Orcvs's Source occupies.
    ///
    /// The only thing that mints a Position [`Editor::select`] will
    /// accept. `Grid` is `Copy`, and the same Grid is already reachable as
    /// `render_frame().grid()` — this answers it without deriving a Frame.
    ///
    pub fn grid(&self) -> Grid {
        self.grid
    }

    pub fn render_frame(&self) -> RenderFrame {
        RenderFrame::derive(
            self.source.read_revision(),
            self.editor.region(),
            RenderFrameConfig {
                sector_seam_spacing: self.opts.sector_seam_spacing,
                cursor_bloom_radius: self.opts.cursor_bloom_radius,
            },
        )
    }

    ///
    /// Handles one batch of input, and answers whether it wrote to the Source
    /// and what it put on the clipboard.
    ///
    pub fn event_handler(&mut self, events: Vec<InputEvent>) -> Handled {
        let mut handled = Handled::default();
        for event in &events {
            match self.editor.handle(event, &self.source) {
                Some(Edit::Wrote) => handled.repaint = true,
                Some(Edit::Copied(text)) => handled.copied = Some(text),
                Some(Edit::Cut(text)) => {
                    handled.copied = Some(text);
                    handled.repaint = true;
                }
                None => {}
            }
            match event {
                InputEvent::KeyPressed(InputKey::Space) => handled.playback_requests += 1,
                InputEvent::Rewind => self.rewind(),
                _ => {}
            }
        }
        handled
    }

    ///
    /// Plays a stopped Playback and stops a playing one. A stopped engine
    /// holds its Tick, and the next start resumes at the Tick after it.
    ///
    pub fn toggle_playback(&mut self) {
        if self.playback_requested {
            self.stop();
        } else {
            self.play();
        }
    }

    ///
    /// Makes Tick `0` the next Tick Playback executes (ADR 0076).
    ///
    pub fn rewind(&mut self) {
        self.playback.rewind();
    }

    fn stop(&mut self) {
        self.playback_requested = false;
        self.playback.stop();
    }

    fn play(&mut self) {
        // A start failure is already recorded as a Playback diagnostic, which
        // `drain_playback_diagnostics` hands to the console; reporting it again
        // here would put one failure on two channels. A refused start raises
        // no request, so it leaves nothing standing for the next Space to
        // cancel.
        if self.playback.start(self.opts.bpm.tick_period()).is_ok() {
            self.playback_requested = true;
        }
    }
}

impl Orcvs {
    /// A running Orcvs with MIDI discovery and selection connected to Playback.
    pub fn with_midi_output_adapter(
        adapter: MidiOutputAdapter,
    ) -> Result<Self, PlaybackStartError> {
        Self::with_source_and_midi_output_adapter(Source::new(Grid::new()), adapter)
    }

    /// Restores Source with MIDI publication established before Playback owns the adapter.
    pub fn with_source_and_midi_output_adapter(
        source: Source,
        adapter: MidiOutputAdapter,
    ) -> Result<Self, PlaybackStartError> {
        let source = SourceCommander::with_source(source);
        let (playback, selection) =
            PlaybackEngine::with_midi_output_adapter(source.clone(), adapter)?;
        Ok(Self::from_playback(source, playback, selection))
    }

    /// Returns the MIDI configuration capability without exposing Playback
    /// lifecycle control.
    ///
    /// A running Orcvs does not hand its complete Playback Engine to callers:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new().unwrap();
    /// let _playback = orcvs.playback_engine();
    /// ```
    ///
    /// The MIDI selection handle cannot start Playback:
    ///
    /// ```compile_fail
    /// use std::time::Duration;
    /// let orcvs = orcvs::app::Orcvs::new().unwrap();
    /// orcvs
    ///     .midi_selection_handle()
    ///     .start(Duration::from_millis(100));
    /// ```
    ///
    /// It cannot stop or disconnect Playback:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new().unwrap();
    /// orcvs.midi_selection_handle().stop();
    /// ```
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new().unwrap();
    /// orcvs.midi_selection_handle().disconnect();
    /// ```
    ///
    /// It cannot read the Playback lifecycle state or drain its diagnostics:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new().unwrap();
    /// let _state = orcvs.midi_selection_handle().state();
    /// ```
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new().unwrap();
    /// let _diagnostics = orcvs.midi_selection_handle().drain_diagnostics();
    /// ```
    pub fn midi_selection_handle(&self) -> MidiSelectionHandle {
        self.selection.clone()
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::{InputEvent, Orcvs};
    use crate::opts::Bpm;
    use crate::playback::PlaybackState;
    use crate::source::Tick;
    use crate::test::trace;
    use crate::{
        opts::DEFAULT_SECTOR_SEAM_SPACING,
        source::{SourcePaint, Token},
    };

    #[tokio::test]
    async fn playback_observation_is_tick_zero_stopped_and_run_clock_zero_before_the_first_run() {
        let orcvs = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
            crate::playback::InMemoryOutputAdapter::default(),
        )
        .expect("the test runtime");

        let observation = orcvs.playback_observation();

        assert_eq!(observation.state, PlaybackState::Stopped);
        assert_eq!(observation.tick, Tick::ZERO);
        assert!(observation.on_beat);
        assert_eq!(observation.run_clock(), Duration::ZERO);
    }

    ///
    /// A toggle pauses and resumes: the toggle that stops Playback holds its
    /// Tick, and the one after it resumes at the next. A Rewind leaves
    /// Playback stopped and makes the next toggle begin at Tick `0`.
    ///
    #[tokio::test(start_paused = true)]
    async fn a_toggle_resumes_at_the_next_tick_and_a_rewind_returns_to_tick_zero() {
        let mut orcvs = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
            crate::playback::InMemoryOutputAdapter::default(),
        )
        .expect("the test runtime");
        let period = orcvs.bpm().tick_period();

        // Ticks 0, 1 and 2 are due within two and a half periods.
        orcvs.toggle_playback();
        tokio::time::sleep(period * 5 / 2).await;
        assert_eq!(orcvs.playback_observation().tick, Tick::new(2));

        orcvs.toggle_playback();
        tokio::time::sleep(period * 3).await;
        let paused = orcvs.playback_observation();
        assert_eq!(paused.state, PlaybackState::Stopped);
        assert_eq!(paused.tick, Tick::new(2), "the pause lost its Tick");

        orcvs.toggle_playback();
        tokio::time::sleep(period / 2).await;
        let resumed = orcvs.playback_observation();
        assert_eq!(resumed.state, PlaybackState::Playing);
        assert_eq!(resumed.tick, Tick::new(3), "Space did not resume");

        orcvs.toggle_playback();
        orcvs.rewind();
        tokio::time::sleep(period / 2).await;
        let rewound = orcvs.playback_observation();
        assert_eq!(rewound.state, PlaybackState::Stopped);
        assert_eq!(rewound.tick, Tick::ZERO);

        orcvs.toggle_playback();
        tokio::time::sleep(period * 3 / 2).await;
        let begun = orcvs.playback_observation();
        assert_eq!(begun.state, PlaybackState::Playing);
        assert_eq!(
            begun.tick,
            Tick::new(1),
            "the run after a rewind began elsewhere"
        );
    }

    ///
    /// A Rewind ahead of the toggle that pauses, before the engine's turn,
    /// ends where the other order does: stopped, with Tick `0` next, and the
    /// toggle after it beginning there rather than ending a run the console
    /// never asked for.
    ///
    #[tokio::test(start_paused = true)]
    async fn a_rewind_before_the_pausing_toggle_leaves_playback_stopped_at_tick_zero() {
        let mut orcvs = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
            crate::playback::InMemoryOutputAdapter::default(),
        )
        .expect("the test runtime");
        let period = orcvs.bpm().tick_period();

        orcvs.toggle_playback();
        tokio::time::sleep(period * 5 / 2).await;
        assert_eq!(orcvs.playback_observation().tick, Tick::new(2));

        orcvs.rewind();
        orcvs.toggle_playback();
        tokio::time::sleep(period * 2).await;
        let rewound = orcvs.playback_observation();
        assert_eq!(rewound.state, PlaybackState::Stopped);
        assert_eq!(rewound.tick, Tick::ZERO);

        orcvs.toggle_playback();
        tokio::time::sleep(period * 3 / 2).await;
        let begun = orcvs.playback_observation();
        assert_eq!(
            begun.state,
            PlaybackState::Playing,
            "the console and the engine disagreed on whether Playback was stopped"
        );
        assert_eq!(begun.tick, Tick::new(1));
    }

    ///
    /// An output adapter that dies on its first delivery, which is how a
    /// Playback Engine's task dies without being asked to.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[derive(Default)]
    struct PanickingOutputAdapter;

    #[cfg(not(target_arch = "wasm32"))]
    impl crate::playback::OutputOnlyAdapter for PanickingOutputAdapter {}

    #[cfg(not(target_arch = "wasm32"))]
    impl crate::playback::OutputAdapter for PanickingOutputAdapter {
        fn submit(
            &mut self,
            _commands: &[crate::playback::OutputCommand],
            _since_due: std::time::Duration,
        ) -> Result<(), crate::playback::OutputAdapterError> {
            panic!("test output panic");
        }

        fn safety_reset(&mut self) -> Result<(), crate::playback::OutputAdapterError> {
            Ok(())
        }
    }

    ///
    /// A toggle asks to play again once the engine has reported that its run
    /// ended without being asked to.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_toggle_after_the_engine_ended_its_own_run_asks_to_play_again() {
        use crate::playback::PlaybackDiagnostic;

        let mut orcvs = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(10, 6)),
            PanickingOutputAdapter,
        )
        .expect("the test runtime");
        {
            let source = &orcvs.source;
            let grid = source.grid();
            for (offset, content) in ".=0101".chars().enumerate() {
                source
                    .set(
                        grid.cell_index(offset).expect("inside the Grid"),
                        &content.to_string(),
                    )
                    .expect("a writable Cell");
            }
            for (offset, content) in "!>007FC4".chars().enumerate() {
                source
                    .set(
                        grid.cell_index(20 + offset).expect("inside the Grid"),
                        &content.to_string(),
                    )
                    .expect("a writable Cell");
            }
        }

        orcvs.toggle_playback();

        // The task dies inside the delivery, and its state reports the failure
        // on the way out.
        let mut ended = false;
        for _ in 0..1_000 {
            if orcvs
                .drain_playback_diagnostics()
                .iter()
                .any(|diagnostic| matches!(diagnostic, PlaybackDiagnostic::ClockFailure { .. }))
            {
                ended = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        assert!(ended, "the engine never reported that its run ended");

        orcvs.toggle_playback();

        // A press that asked to play reports why it could not; a press that
        // asked to stop reports nothing at all.
        let diagnostics = orcvs.drain_playback_diagnostics();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| matches!(diagnostic, PlaybackDiagnostic::StartFailure { .. })),
            "Space asked to stop a run that had already ended: {diagnostics:?}"
        );
    }

    #[tokio::test]
    async fn a_fresh_orcvs_opens_at_one_hundred_and_twenty_bpm() {
        let orcvs = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
            crate::playback::InMemoryOutputAdapter::default(),
        )
        .expect("the test runtime");

        assert_eq!(orcvs.bpm().beats_per_minute(), 120);
    }

    #[tokio::test]
    async fn user_can_change_the_tempo() {
        let mut orcvs = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
            crate::playback::InMemoryOutputAdapter::default(),
        )
        .expect("the test runtime");

        orcvs.set_bpm(Bpm::new(120).unwrap());

        assert_eq!(orcvs.bpm().beats_per_minute(), 120);
    }

    #[tokio::test(start_paused = true)]
    async fn repeated_tempo_changes_preserve_the_current_beat_phase() {
        let adapter = crate::playback::InMemoryOutputAdapter::default();
        let mut orcvs = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
            adapter.clone(),
        )
        .expect("the test runtime");
        orcvs.toggle_playback();
        tokio::task::yield_now().await;
        assert_eq!(adapter.command_lists().len(), 1);

        tokio::time::advance(Duration::from_millis(500)).await;
        for _ in 0..5 {
            orcvs.set_bpm(Bpm::new(20).unwrap());
            tokio::time::advance(Duration::from_millis(40)).await;
            tokio::task::yield_now().await;
        }

        assert_eq!(adapter.safety_reset_count(), 0);
        assert_eq!(adapter.command_lists().len(), 1);
        assert_eq!(
            orcvs.playback.state(),
            crate::playback::PlaybackState::Playing
        );

        tokio::time::advance(Duration::from_millis(49)).await;
        tokio::task::yield_now().await;
        assert_eq!(adapter.command_lists().len(), 1);

        tokio::time::advance(Duration::from_millis(1)).await;
        tokio::task::yield_now().await;
        assert_eq!(adapter.command_lists().len(), 2);
    }

    ///
    /// Two toggles before the engine's turn are a toggle and its
    /// cancellation, and leave Playback where they found it.
    ///
    /// The engine coalesces two starts into one run deliberately — that is
    /// where idempotence belongs — so a second toggle that asked to start
    /// again is not corrected downstream. It has to not ask.
    ///
    #[tokio::test]
    async fn a_second_toggle_before_the_engines_turn_cancels_the_first() {
        let adapter = crate::playback::InMemoryOutputAdapter::default();
        let mut orcvs = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
            adapter.clone(),
        )
        .expect("the test runtime");

        orcvs.toggle_playback();
        orcvs.toggle_playback();
        tokio::task::yield_now().await;

        assert_eq!(
            orcvs.playback.state(),
            crate::playback::PlaybackState::Stopped,
            "an even number of toggles leaves Playback stopped"
        );
        assert!(
            adapter.command_lists().is_empty(),
            "a run cancelled before the engine's turn delivers no Tick"
        );
    }

    ///
    /// A batch counts each Space as a Playback request for the console to
    /// answer and leaves Playback alone, while each Space still disarms a
    /// fill armed before it.
    ///
    #[tokio::test]
    async fn a_batch_counts_each_space_as_a_playback_request() {
        use super::InputKey;

        let mut orcvs = Orcvs::new().expect("the test runtime");
        let space = InputEvent::KeyPressed(InputKey::Space);

        let handled = orcvs.event_handler(vec![space.clone(), InputEvent::Fill, space]);

        assert_eq!(handled.playback_requests, 2);
        assert!(!orcvs.playback_requested(), "a Space toggled Playback");
        // The Space after the Fill disarmed it, so a character is typed at the
        // Cursor, which steps right, rather than filling its one Cell.
        orcvs.event_handler(vec![InputEvent::Text("x".to_owned())]);
        let stepped = orcvs.grid().position(1, 0).expect("inside the Grid");
        assert_eq!(orcvs.render_frame().cursor(), stepped);
    }

    ///
    /// Native only: what this states is a running Orcvs that finds no Tokio
    /// runtime, and staging that means owning the decision about whether one
    /// is entered. `tokio::runtime::Runtime::new` is the multi-threaded
    /// builder, which the `[target.'cfg(not(target_arch = "wasm32"))'
    /// .dependencies]` table pulls in and a browser target never has.
    ///
    /// The Playback Engine is a task, so a runtime is what building a running
    /// Orcvs requires and construction is where the absence of one is
    /// answered. There is nothing half-built left over: an Orcvs that could
    /// not spawn its engine is not returned at all.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_running_orcvs_is_refused_when_there_is_no_runtime_to_run_on() {
        let adapter = crate::playback::InMemoryOutputAdapter::default();

        let outside = Orcvs::with_source_and_output_adapter(
            crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
            adapter.clone(),
        );

        assert_eq!(
            outside.err(),
            Some(crate::playback::PlaybackStartError::RuntimeUnavailable)
        );

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _runtime = runtime.enter();

        assert!(
            Orcvs::with_source_and_output_adapter(
                crate::source::Source::new(crate::grid::Grid::with_shape(2, 1)),
                adapter
            )
            .is_ok()
        );
    }

    #[tokio::test]
    async fn render_frame_reflects_a_write_and_the_selection() {
        let mut app = orcvs();
        app.event_handler(vec![InputEvent::Text("x".to_owned())]);

        let frame = app.render_frame();

        assert_eq!(frame.grid().rows(), 1);
        assert_eq!(frame.grid().columns(), 2);
        assert_eq!(frame.at(app.grid.origin()).content(), Some('x'));
        assert_eq!(
            frame.at(app.grid.position(1, 0).unwrap()).position(),
            app.grid.position(1, 0).unwrap()
        );
        assert_eq!(frame.cursor(), app.grid.position(1, 0).unwrap());
    }

    ///
    /// The Cursor is the one the Region `derive` was given, carried rather
    /// than found.
    ///
    #[tokio::test]
    async fn render_frame_answers_the_cursor_it_was_derived_for() {
        let mut app = Orcvs::with_shape(4, 3).expect("the test runtime");
        let origin = app.grid.origin();
        let initial = app.render_frame();
        assert_eq!(initial.cursor(), origin);

        let moved = app.grid.position(2, 1).unwrap();
        app.editor_mut().select(moved);
        let moved_frame = app.render_frame();
        assert_eq!(moved_frame.cursor(), moved);
    }

    #[tokio::test]
    async fn a_fresh_orcvs_has_a_region_of_the_cursors_one_cell() {
        let app = Orcvs::with_shape(4, 3).expect("the test runtime");

        let region = app.render_frame().region();

        assert!(region.is_one_cell());
        assert_eq!(region.cursor(), app.grid.origin());
    }

    ///
    /// A two-by-one running Orcvs on the test's own runtime.
    ///
    /// Every test that builds one is running on a runtime, which is what a
    /// running Orcvs needs to spawn its Playback Engine onto.
    ///
    fn orcvs() -> Orcvs {
        Orcvs::with_shape(2, 1).expect("the test runtime")
    }

    fn app() -> Orcvs {
        let rows = 1; // * (DEFAULT_SECTOR_SEAM_SPACING as usize);
        let cols = DEFAULT_SECTOR_SEAM_SPACING;

        Orcvs::with_shape(cols, rows).expect("the test runtime")
    }

    fn rendered(app: &Orcvs, position: crate::grid::Position) -> (Option<char>, Option<Token>) {
        let frame = app.render_frame();
        let cell = frame.at(position);
        let token = match cell.source_paint() {
            SourcePaint::Unclaimed => None,
            SourcePaint::Function => Some(Token::Function),
            SourcePaint::Bang => Some(Token::Function),
            SourcePaint::Comment => Some(Token::Comment),
            SourcePaint::Operand { token, .. } => Some(token),
        };
        (cell.content(), token)
    }

    impl Orcvs {
        pub fn src(&mut self, src: &str) {
            for (i, c) in src.chars().enumerate() {
                self.set_at(i, 0, &c.to_string())
            }
        }

        /// The one place a test turns coordinates into a Position. It panics
        /// rather than ignoring a pair the Grid refuses: a test that silently
        /// wrote to the previously selected Cell would assert nothing about the
        /// Cell it named.
        fn select_or_panic(&mut self, x: usize, y: usize) {
            let position = self
                .grid
                .position(x, y)
                .unwrap_or_else(|| panic!("test position ({x}, {y}) is outside the Grid"));
            self.editor.select(position);
        }

        pub fn set_at(&mut self, x: usize, y: usize, s: &str) {
            self.select_or_panic(x, y);
            self.event_handler(vec![InputEvent::Text(s.to_owned())]);
        }
    }

    #[tokio::test]
    async fn test_get_reads_the_cell_at_the_position() {
        trace();

        // 4 columns, 2 rows: transposing the axes addresses a different Cell.
        let mut app = Orcvs::with_shape(4, 2).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        // written into the second row
        app.set_at(0, 1, ".");
        app.set_at(1, 1, "+");

        assert_eq!(rendered(&app, at(0, 1)), (Some('.'), Some(Token::Function)));
        let written = (Some('+'), Some(Token::Function));
        assert_eq!(rendered(&app, at(1, 1)), written);

        // and it is those Cells' content, not another's
        for row in grid.positions_by_row() {
            for position in row {
                if position == at(0, 1) || position == at(1, 1) {
                    continue;
                }
                assert_ne!(
                    rendered(&app, position),
                    written,
                    "({}, {}) holds no written Cell",
                    position.x(),
                    position.y()
                );
            }
        }
    }

    #[tokio::test]
    async fn test_write_renders_cell_and_glyph_immediately() {
        trace();

        let mut app = app();
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        app.set_at(0, 0, ".");
        app.set_at(1, 0, "+");

        // the accepted edits are observable as soon as write returns
        assert_eq!(rendered(&app, at(0, 0)), (Some('.'), Some(Token::Function)));
        assert_eq!(rendered(&app, at(1, 0)), (Some('+'), Some(Token::Function)));
    }

    #[tokio::test]
    async fn test_editing_an_operand_hint_never_renders_an_occupied_cell_as_empty() {
        trace();

        let mut app = Orcvs::with_shape(10, 1).expect("the test runtime");
        let position = app.grid.position(5, 0).expect("inside the grid");
        app.set_at(0, 0, ".");
        app.set_at(1, 0, "+");

        app.set_at(5, 0, "x");

        // Cell 5 is the second Cell of the Addition's second operand — `.+`
        // claims six Cells whatever they hold — so it is presented as the
        // operand it is. What this test is about is that the character survives
        // the classification: an operand-slot Token never renders an occupied
        // Cell as empty.
        assert_eq!(rendered(&app, position), (Some('x'), Some(Token::Number)));
    }

    #[tokio::test]
    async fn test_empty_source_cells_remain_empty_without_marker_glyphs() {
        trace();

        let mut app = app();
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        app.select_or_panic(7, 0);

        assert!((0..=2).all(|x| rendered(&app, at(x, 0)) == (None, None)));
    }

    ///
    /// An unclosed Comment claims every remaining Cell of its row, empty Cells
    /// included, so an empty Cell it claims still carries a Token. The Token
    /// decides the colour and nothing else: a Comment hints at no spelling, so
    /// a Cell it claims renders the character the Source holds, and an empty
    /// one renders empty.
    ///
    /// An empty operand Cell renders `h` or `n` because a signature says what
    /// belongs there, which
    /// `test_editing_an_operand_hint_never_renders_an_occupied_cell_as_empty`
    /// pins from the other side. A Comment declares nothing, so a placeholder
    /// there would be a character the user never typed.
    ///
    #[tokio::test]
    async fn test_a_comment_renders_its_own_text_and_leaves_its_empty_cells_empty() {
        trace();

        let mut app = Orcvs::with_shape(10, 1).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.src("||hi there");

        // Every Cell of the row belongs to the one Comment, so every Cell
        // carries its Token, the space between the two words included.
        let frame = app.render_frame();
        assert!((0..10).all(|x| { frame.at(at(x, 0)).source_paint() == SourcePaint::Comment }));
        // And each renders what the Source holds there, no more.
        assert_eq!(
            (0..10)
                .map(|x| rendered(&app, at(x, 0)).0.unwrap_or(' '))
                .collect::<String>(),
            "||hi there",
        );

        // The same row with a tail the text does not reach. A second Grid
        // places its own Positions, so the Cells are named through it.
        let mut app = Orcvs::with_shape(10, 1).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.src("||hi");

        assert_eq!(
            (0..10)
                .map(|x| rendered(&app, at(x, 0)).0.unwrap_or(' '))
                .collect::<String>(),
            "||hi      ",
        );
    }

    #[tokio::test]
    async fn test_empty_cells_between_markers_remain_spaces() {
        let mut app = Orcvs::with_shape(24, 16).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.editor_mut().select(at(8, 8));

        assert_eq!(rendered(&app, at(14, 10)), (None, None));
        assert_eq!(rendered(&app, at(16, 10)), (None, None));
        assert_eq!(rendered(&app, at(17, 10)), (None, None));
    }
}
