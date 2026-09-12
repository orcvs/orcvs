use std::time::Duration;
use tracing::error;

use crate::opts::{Bpm, Opts};

use crate::cursor::Cursor;
use crate::grid::{Grid, Position};
use crate::native_midi::{self, NativeMidiOutputAdapter};
use crate::playback::{OutputAdapter, PlaybackDiagnostic, PlaybackEngine, PlaybackStartError};
use crate::render_frame::{RenderFrame, RenderFrameConfig};
use crate::source::{Source, SourceCommander};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKey {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Backspace,
    Delete,
    Space,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    KeyPressed(InputKey),
    Text(String),
}

///
/// What a running Orcvs sends its Output Commands to unless it is handed
/// another adapter. `orcvs::native_midi` decides what backend that adapter has
/// on this target, so this alias names a valid type everywhere, the browser
/// included.
///
pub type OrcvsOutputAdapter = NativeMidiOutputAdapter;

///
/// One running Orcvs: its options, Source and Grid, Cursor, and Playback
/// lifecycle. Output-device discovery and selection belong to the console.
///
/// Selection names a Position, and only a Grid mints one. A pair outside the
/// Grid never becomes a Position at all, so `select` has no rejection to make
/// and cannot silently leave the Cursor on the Cell it was already on and send
/// the next write there:
///
/// ```
/// use orcvs::app::Orcvs;
///
/// // A running Orcvs runs: its Playback Engine is a task, and a task needs a
/// // runtime to be spawned on, so building one is fallible and eager.
/// let runtime = tokio::runtime::Runtime::new().unwrap();
/// let _runtime = runtime.enter();
///
/// let mut orcvs = Orcvs::new(16, 16).expect("a Tokio runtime");
/// let grid = orcvs.grid();
///
/// // the Grid refuses a pair outside itself, so there is no Position to select
/// assert_eq!(grid.position(99, 99), None);
///
/// // every Position `select` can be handed is one this Grid minted
/// let position = grid.position(15, 15).expect("inside the grid");
/// orcvs.select(position);
/// assert_eq!(orcvs.render_frame().cursor(), position);
/// ```
///
pub struct Orcvs<A: OutputAdapter = OrcvsOutputAdapter> {
    opts: Opts,
    cursor: Cursor,
    grid: Grid,

    source: SourceCommander,
    playback: PlaybackEngine<A>,
    ///
    /// Whether this Orcvs has asked its Playback Engine to be running.
    ///
    /// Not a second copy of the engine's lifecycle state: ADR 0041 publishes
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
    pub fn new(cols: usize, rows: usize) -> Result<Self, PlaybackStartError> {
        Self::with_output_adapter(cols, rows, native_midi::output_adapter())
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
    /// let mut restored = Source::new(Grid::new(6, 3));
    /// let cell = restored.grid().cell_index(0).expect("inside the Grid");
    /// restored.set(cell, "1").expect("a Cell the Source accepts");
    ///
    /// let orcvs = Orcvs::with_source(restored).expect("a Tokio runtime");
    ///
    /// // the Source arrives whole: its Cells, and the Grid it was built from
    /// let frame = orcvs.render_frame();
    /// let grid = frame.grid();
    /// assert_eq!(grid.rows(), 3);
    /// assert_eq!(grid.columns(), 6);
    /// assert_eq!(frame.at(grid.origin()).content(), Some('1'));
    /// ```
    ///
    pub fn with_source(source: Source) -> Result<Self, PlaybackStartError> {
        Self::with_source_and_output_adapter(source, native_midi::output_adapter())
    }
}

impl<A: OutputAdapter + Send + 'static> Orcvs<A> {
    pub fn with_output_adapter(
        cols: usize,
        rows: usize,
        adapter: A,
    ) -> Result<Self, PlaybackStartError> {
        Self::with_source_and_output_adapter(Source::new(Grid::new(cols, rows)), adapter)
    }

    ///
    /// A running Orcvs over `source`, delivering to `adapter` and taking the
    /// Grid the Source was built from: a Source read back from persistence
    /// carries the shape it was stored with, and the Cursor starts at that
    /// Grid's origin.
    ///
    /// Fallible because the Playback Engine is: ADR 0041 gives the engine's
    /// state a task that owns it, and a task needs a runtime to be spawned on,
    /// so a running Orcvs is one whose engine is already running.
    ///
    /// ```
    /// use orcvs::app::Orcvs;
    /// use orcvs::grid::Grid;
    /// use orcvs::playback::InMemoryOutputAdapter;
    /// use orcvs::source::Source;
    ///
    /// # let runtime = tokio::runtime::Runtime::new().unwrap();
    /// # let _runtime = runtime.enter();
    /// let restored = Source::new(Grid::new(6, 3));
    /// let orcvs = Orcvs::with_source_and_output_adapter(restored, InMemoryOutputAdapter::default())
    ///     .expect("a Tokio runtime");
    ///
    /// // the shape is the Source's, not a pair passed alongside it, and the
    /// // Cursor opens on that Grid's origin
    /// let frame = orcvs.render_frame();
    /// assert_eq!(frame.grid().rows(), 3);
    /// assert!(frame.at(frame.grid().origin()).selected());
    /// ```
    ///
    pub fn with_source_and_output_adapter(
        source: Source,
        adapter: A,
    ) -> Result<Self, PlaybackStartError> {
        let grid = source.grid();
        let opts = Opts::new();
        let source = SourceCommander::with_source(source);
        let playback = PlaybackEngine::new(source.clone(), adapter)?;

        Ok(Self {
            cursor: Cursor::new(grid.origin(), opts.cursor_delay),
            grid,
            opts,
            source,
            playback,
            playback_requested: false,
        })
    }

    ///
    /// The Source root, for the storage a console saves the current revision
    /// into.
    ///
    #[cfg(feature = "persistence")]
    pub fn source(&self) -> &SourceCommander {
        &self.source
    }

    ///
    /// Takes the Playback diagnostics recorded since the last drain, for the
    /// console to report, and withdraws a request the engine has just said it
    /// is no longer carrying out.
    ///
    /// It answers with diagnostics alone. Lifecycle state is no longer handed
    /// back beside them and cached here: per ADR 0041 the engine publishes it,
    /// and whoever needs it reads the published value at the moment it is
    /// asked rather than the value the last frame happened to carry away.
    ///
    /// `playback_requested` answers what this Orcvs has *asked* Playback to
    /// be, which is the only fact Space can toggle against within one input
    /// batch. A run the engine ended by itself — an adapter panicking out of a
    /// delivery, a grid it can no longer schedule — leaves that request
    /// standing for nothing, and the next Space cancels a run that is already
    /// over: a press the user sees nothing come of, and a second one needed
    /// before the app tries to play at all. `ClockFailure` is the engine
    /// saying exactly that, on the one ordered stream it reports failures on,
    /// so this is where the request is withdrawn. Every other diagnostic
    /// leaves it alone: an Overrun or a refused device is a run continuing,
    /// not a run ending.
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

    pub fn bpm(&self) -> Bpm {
        self.opts.bpm
    }

    pub fn set_bpm(&mut self, bpm: Bpm) {
        if self.playback_requested
            && let Err(error) = self.playback.retune(Duration::from_millis(bpm.delay_ms()))
        {
            self.playback.report_retune_error(error);
            return;
        }
        self.opts.bpm = bpm;
    }

    ///
    /// Moves the Cursor to `position`, refusing one minted by another Grid.
    ///
    pub fn select(&mut self, position: Position) {
        self.grid.assert_owns(position);
        self.cursor.select(position);
    }

    ///
    /// The Grid this running Orcvs's Source occupies.
    ///
    /// The only thing that mints a Position [`select`](Self::select) will
    /// accept. `Grid` is `Copy`, and the same Grid is already reachable as
    /// `render_frame().grid()` — this answers it without deriving a Frame.
    ///
    pub fn grid(&self) -> Grid {
        self.grid
    }

    ///
    /// writes s to the current cursor position
    /// triggers parse of expression
    ///
    pub fn write(&mut self, s: &str) {
        let cell = self.grid.index(self.cursor.position());

        match self.source.set(cell, s) {
            Ok(_) => self.cursor.select(self.grid.right(self.cursor.position())),
            Err(e) => error!("rejected edit: {e}"),
        }
    }

    ///
    /// Empties the Cell under the Cursor and steps left.
    ///
    /// The Cursor sits on a Position this Grid minted, so the Cell it names
    /// exists and emptying it cannot be refused.
    ///
    fn delete(&mut self) {
        self.source.unset(self.grid.index(self.cursor.position()));
        self.cursor.select(self.grid.left(self.cursor.position()));
    }

    pub fn render_frame(&self) -> RenderFrame {
        RenderFrame::derive(
            self.source.read_revision(),
            self.cursor.position(),
            self.cursor.on,
            RenderFrameConfig {
                marker_spacing: self.opts.marker_spacing,
                highlight_dot_spacing: self.opts.highlight_dot_spacing,
            },
        )
    }

    pub fn advance_cursor_blink(&mut self) {
        self.cursor.blink();
    }

    pub fn remaining_cursor_blink_delay(&self) -> Duration {
        self.cursor.remaining_blink_delay()
    }

    ///
    /// Handles event and returns boolean indicating if repating is required
    ///
    pub fn event_handler(&mut self, events: Vec<InputEvent>) -> bool {
        let mut repaint = false;
        for event in &events {
            match event {
                InputEvent::KeyPressed(InputKey::ArrowDown) => {
                    self.cursor.select(self.grid.down(self.cursor.position()))
                }
                InputEvent::KeyPressed(InputKey::ArrowLeft) => {
                    self.cursor.select(self.grid.left(self.cursor.position()))
                }
                InputEvent::KeyPressed(InputKey::ArrowRight) => {
                    self.cursor.select(self.grid.right(self.cursor.position()))
                }
                InputEvent::KeyPressed(InputKey::ArrowUp) => {
                    self.cursor.select(self.grid.up(self.cursor.position()))
                }
                InputEvent::KeyPressed(InputKey::Backspace | InputKey::Delete) => self.delete(),
                InputEvent::KeyPressed(InputKey::Space) => {
                    if self.playback_requested {
                        self.stop();
                    } else {
                        self.play();
                    }
                }
                InputEvent::Text(text_to_insert)
                    if text_to_insert.len() == 1 && text_to_insert != " " =>
                {
                    self.write(text_to_insert);
                    repaint = true;
                }
                InputEvent::Text(_) => {}
            }
        }
        repaint
    }

    fn stop(&mut self) {
        self.playback_requested = false;
        self.playback.stop();
    }

    fn play(&mut self) {
        let ms = self.opts.bpm.delay_ms();
        // A start failure is already recorded as a Playback diagnostic, which
        // `drain_playback_diagnostics` hands to the console; reporting it again
        // here would put one failure on two channels. What is left to do with
        // the answer is to not raise a request the engine refused: Space
        // toggles against what has been asked for, so a refused start must
        // leave nothing standing for the next press to cancel.
        if self.playback.start(Duration::from_millis(ms)).is_ok() {
            self.playback_requested = true;
        }
    }
}

impl<B: crate::midi::MidiBackend + 'static> Orcvs<crate::midi::MidiOutputAdapter<B>> {
    /// Returns the MIDI configuration capability without exposing Playback
    /// lifecycle control.
    ///
    /// A running Orcvs does not hand its complete Playback Engine to callers:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16).unwrap();
    /// let _playback = orcvs.playback_engine();
    /// ```
    ///
    /// The MIDI selection handle cannot start Playback:
    ///
    /// ```compile_fail
    /// use std::time::Duration;
    /// let orcvs = orcvs::app::Orcvs::new(16, 16).unwrap();
    /// orcvs
    ///     .midi_selection_handle()
    ///     .start(Duration::from_millis(100));
    /// ```
    ///
    /// It cannot stop or disconnect Playback:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16).unwrap();
    /// orcvs.midi_selection_handle().stop();
    /// ```
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16).unwrap();
    /// orcvs.midi_selection_handle().disconnect();
    /// ```
    ///
    /// It cannot read the Playback lifecycle state or drain its diagnostics:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16).unwrap();
    /// let _state = orcvs.midi_selection_handle().state();
    /// ```
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16).unwrap();
    /// let _diagnostics = orcvs.midi_selection_handle().drain_diagnostics();
    /// ```
    pub fn midi_selection_handle(&self) -> crate::playback::MidiSelectionHandle<B> {
        crate::playback::MidiSelectionHandle::new(&self.playback)
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::Orcvs;
    use crate::opts::Bpm;
    use crate::test::trace;
    use crate::{
        glyph::{Glyph, GlyphString},
        opts::{DEFAULT_MARKER_SPACING, MarkerSpacing},
    };

    ///
    /// An output adapter that dies on its first delivery, which is how a
    /// Playback Engine's task dies without being asked to.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[derive(Default)]
    struct PanickingOutputAdapter;

    #[cfg(not(target_arch = "wasm32"))]
    impl crate::playback::OutputAdapter for PanickingOutputAdapter {
        fn submit(
            &mut self,
            _commands: &[crate::playback::OutputCommand],
        ) -> Result<(), crate::playback::OutputAdapterError> {
            panic!("test output panic");
        }

        fn safety_reset(&mut self) -> Result<(), crate::playback::OutputAdapterError> {
            Ok(())
        }
    }

    ///
    /// Space asks to play again once the engine has reported that its run
    /// ended without being asked to.
    ///
    /// Space toggles against what Playback has been *asked* to be, which is
    /// the only fact an input batch can read without the engine having had a
    /// turn. A run the engine ended by itself leaves that request standing for
    /// nothing: the next press cancels a run that is already over, does
    /// nothing a user can see, and a second press is needed before the app
    /// even tries to play. The engine says so on the one channel it has, so
    /// the request is withdrawn where that report is read.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn space_after_the_engine_ended_its_own_run_asks_to_play_again() {
        use crate::playback::PlaybackDiagnostic;

        let mut orcvs =
            Orcvs::with_output_adapter(10, 6, PanickingOutputAdapter).expect("the test runtime");
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

        orcvs.event_handler(vec![super::InputEvent::KeyPressed(super::InputKey::Space)]);

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

        orcvs.event_handler(vec![super::InputEvent::KeyPressed(super::InputKey::Space)]);

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
    async fn user_can_change_the_tempo() {
        let mut orcvs =
            Orcvs::with_output_adapter(2, 1, crate::playback::InMemoryOutputAdapter::default())
                .expect("the test runtime");

        orcvs.set_bpm(Bpm::new(120).unwrap());

        assert_eq!(orcvs.bpm().beats_per_minute(), 120);
    }

    #[tokio::test(start_paused = true)]
    async fn repeated_tempo_changes_preserve_the_current_beat_phase() {
        let adapter = crate::playback::InMemoryOutputAdapter::default();
        let mut orcvs =
            Orcvs::with_output_adapter(2, 1, adapter.clone()).expect("the test runtime");
        orcvs.event_handler(vec![super::InputEvent::KeyPressed(super::InputKey::Space)]);
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
    /// Two Space events in one batch are a toggle and its cancellation, and
    /// leave Playback where they found it.
    ///
    /// A batch is a whole frame's events, handled with nothing awaited
    /// between them, so the Playback Engine's task has had no turn in which to
    /// apply what the first Space asked for. Held Space is how a user delivers
    /// this: egui reports auto-repeat as further key presses, and any frame
    /// longer than the repeat interval carries two.
    ///
    /// The engine coalesces two starts into one run deliberately — that is
    /// where idempotence belongs — so a second Space that asked to start again
    /// is not corrected downstream. It has to not ask.
    ///
    #[tokio::test]
    async fn a_second_space_in_one_batch_cancels_the_first() {
        let adapter = crate::playback::InMemoryOutputAdapter::default();
        let mut orcvs =
            Orcvs::with_output_adapter(2, 1, adapter.clone()).expect("the test runtime");

        orcvs.event_handler(vec![
            super::InputEvent::KeyPressed(super::InputKey::Space),
            super::InputEvent::KeyPressed(super::InputKey::Space),
        ]);
        tokio::task::yield_now().await;

        assert_eq!(
            orcvs.playback.state(),
            crate::playback::PlaybackState::Stopped,
            "an even number of Space presses leaves Playback stopped"
        );
        assert!(
            adapter.command_lists().is_empty(),
            "a run cancelled before the engine's turn delivers no Tick"
        );
    }

    ///
    /// Native only: what this states is a running Orcvs that finds no Tokio
    /// runtime, and staging that means owning the decision about whether one
    /// is entered. `tokio::runtime::Runtime::new` is the multi-threaded
    /// builder, which the `[target.'cfg(not(target_arch = "wasm32"))'
    /// .dependencies]` table pulls in and a browser target never has.
    ///
    /// ADR 0041 makes the Playback Engine a task, so a runtime is what
    /// building a running Orcvs requires and construction is where the absence
    /// of one is answered. There is nothing half-built left over: an Orcvs
    /// that could not spawn its engine is not returned at all.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
fn a_running_orcvs_is_refused_when_there_is_no_runtime_to_run_on() {
        let adapter = crate::playback::InMemoryOutputAdapter::default();

        let outside = Orcvs::with_output_adapter(2, 1, adapter.clone());

        assert_eq!(
            outside.err(),
            Some(crate::playback::PlaybackStartError::RuntimeUnavailable)
        );

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _runtime = runtime.enter();

        assert!(Orcvs::with_output_adapter(2, 1, adapter).is_ok());
    }

    #[tokio::test]
    async fn render_frame_reflects_a_write_and_the_selection() {
        let mut app = orcvs();
        app.write("x");

        let frame = app.render_frame();

        assert_eq!(frame.grid().rows(), 1);
        assert_eq!(frame.grid().columns(), 2);
        assert_eq!(frame.at(app.grid.origin()).content(), Some('x'));
        assert_eq!(
            frame.at(app.grid.position(1, 0).unwrap()).position(),
            app.grid.position(1, 0).unwrap()
        );
        assert!(frame.at(app.grid.position(1, 0).unwrap()).selected());
    }

    ///
    /// The Cursor is the Position `derive` was given, carried rather than found.
    ///
    #[tokio::test]
    async fn render_frame_answers_the_cursor_it_was_derived_for() {
        let mut app = Orcvs::new(4, 3).expect("the test runtime");
        let origin = app.grid.origin();
        assert_eq!(app.render_frame().cursor(), origin);

        let moved = app.grid.position(2, 1).unwrap();
        app.select(moved);
        assert_eq!(app.render_frame().cursor(), moved);
    }

    #[tokio::test]
    async fn deriving_a_render_frame_does_not_advance_cursor_blink_state() {
        let mut app = orcvs();
        app.cursor.on = true;

        let first = app.render_frame();
        let second = app.render_frame();

        assert!(first.at(app.grid.origin()).cursor_visible());
        assert!(second.at(app.grid.origin()).cursor_visible());
        assert!(app.cursor.on);
    }

    ///
    /// A two-by-one running Orcvs on the test's own runtime.
    ///
    /// Every test that builds one is running on a runtime, which is what a
    /// running Orcvs needs to spawn its Playback Engine onto.
    ///
    fn orcvs() -> Orcvs {
        Orcvs::new(2, 1).expect("the test runtime")
    }

    fn app() -> Orcvs {
        let rows = 1; // * (DEFAULT_MARKER_SPACING as usize);
        let cols = DEFAULT_MARKER_SPACING;

        Orcvs::new(cols, rows).expect("the test runtime")
    }

    fn rendered(app: &Orcvs, position: crate::grid::Position) -> GlyphString {
        let frame = app.render_frame();
        let cell = frame.at(position);
        GlyphString::new(
            cell.content().map(|content| content.to_string()),
            cell.glyph(),
        )
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
            self.cursor.select(position);
        }

        pub fn delete_at(&mut self, x: usize, y: usize) {
            self.select_or_panic(x, y);
            self.delete()
        }

        pub fn set_at(&mut self, x: usize, y: usize, s: &str) {
            self.select_or_panic(x, y);
            self.write(s);
        }
    }

    #[tokio::test]
    async fn test_get_reads_the_cell_at_the_position() {
        trace();

        // 4 columns, 2 rows: transposing the axes addresses a different Cell.
        let mut app = Orcvs::new(4, 2).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        // written into the second row
        app.set_at(0, 1, ".");
        app.set_at(1, 1, "+");

        assert_eq!(
            rendered(&app, at(0, 1)),
            GlyphString::new(Some(".".to_string()), Glyph::Function)
        );
        let written = GlyphString::new(Some("+".to_string()), Glyph::Function);
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
        assert_eq!(
            rendered(&app, at(0, 0)),
            GlyphString::new(Some(".".to_string()), Glyph::Function)
        );
        assert_eq!(
            rendered(&app, at(1, 0)),
            GlyphString::new(Some("+".to_string()), Glyph::Function)
        );
    }

    #[tokio::test]
    async fn test_editing_an_operand_hint_never_renders_an_occupied_cell_as_empty() {
        trace();

        let mut app = Orcvs::new(10, 1).expect("the test runtime");
        let position = app.grid.position(5, 0).expect("inside the grid");
        app.set_at(0, 0, ".");
        app.set_at(1, 0, "+");

        app.set_at(5, 0, "x");

        // Cell 5 is the second Cell of the Addition's second operand — ADR
        // 0033 has `.+` claim six Cells whatever they hold — so it is
        // presented as the operand it is. What this test is about is that the
        // character survives the classification: an operand-slot Glyph never
        // renders an occupied Cell as empty.
        assert_eq!(
            rendered(&app, position),
            GlyphString::new(Some("x".to_string()), Glyph::Number)
        );
    }

    #[tokio::test]
    async fn test_select_moves_the_cursor_to_the_position() {
        trace();

        let mut app = app();
        let target = app.grid.position(3, 0).expect("inside the grid");

        app.select(target);

        assert_eq!(app.cursor.position(), target);
    }

    #[tokio::test]
    async fn test_write_moves_cursor_right() {
        trace();

        let mut app = app();
        app.select_or_panic(0, 0);

        app.write("+");

        assert_eq!(app.cursor.position(), app.grid.position(1, 0).unwrap());
    }

    #[tokio::test]
    async fn test_delete_clears_cell_and_moves_cursor_left() {
        trace();

        let mut app = app();
        let grid = app.grid;

        app.set_at(0, 0, "+");
        app.set_at(1, 0, "+");

        app.delete_at(1, 0);

        assert_eq!(app.cursor.position(), grid.position(0, 0).unwrap());
        // The refused `+` owns only Cell 0; its neighbour is empty again.
        assert_eq!(
            rendered(&app, grid.position(1, 0).expect("inside the grid")),
            GlyphString::space()
        );
    }

    #[tokio::test]
    async fn test_empty_source_cells_remain_empty_without_marker_glyphs() {
        trace();

        let mut app = app();
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        app.select_or_panic(7, 0);

        assert!((0..=2).all(|x| rendered(&app, at(x, 0)) == GlyphString::space()));
    }

    #[tokio::test]
    async fn test_sector_edges_use_one_whole_cell_spacing_without_marker_glyphs() {
        let mut app = Orcvs::new(7, 3).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(6, 2));
        app.opts.marker_spacing = MarkerSpacing::new(2).unwrap();

        let frame = app.render_frame();
        assert_eq!(
            (0..7)
                .map(|x| frame.at(at(x, 0)).sector_left_strength().is_some())
                .collect::<Vec<_>>(),
            vec![false, false, true, false, true, false, true]
        );
        assert!((0..7).all(|x| frame.at(at(x, 0)).glyph() == Glyph::Space));

        app.opts.marker_spacing = MarkerSpacing::new(1).unwrap();
        let frame = app.render_frame();
        assert_eq!(frame.at(at(0, 0)).sector_left_strength(), None);
        assert!((1..7).all(|x| frame.at(at(x, 0)).sector_left_strength().is_some()));
    }

    ///
    /// A Comment claims every remaining Cell of its row, empty Cells
    /// included, so those Cells now carry a Glyph where before they carried
    /// none. The Glyph decides the colour and nothing else: a Comment hints
    /// at no spelling, so a Cell it claims renders the character the Source
    /// holds, and an empty one renders empty.
    ///
    /// This is the distinction the `s: None` fallback draws. An empty operand
    /// Cell renders `h` or `n` because a signature says what belongs there,
    /// which `test_editing_an_operand_hint_never_renders_an_occupied_cell_as_empty`
    /// pins from the other side. A Comment declares nothing, so a placeholder
    /// there would be a character the user never typed.
    ///
    #[tokio::test]
    async fn test_a_comment_renders_its_own_text_and_leaves_its_empty_cells_empty() {
        trace();

        let mut app = Orcvs::new(10, 1).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.src("||hi there");

        // Every Cell of the row belongs to the one Comment, so every Cell
        // carries its Glyph, the space between the two words included.
        let frame = app.render_frame();
        assert!((0..10).all(|x| frame.at(at(x, 0)).glyph() == Glyph::Comment));
        // And each renders what the Source holds there, no more.
        assert_eq!(
            (0..10)
                .map(|x| rendered(&app, at(x, 0)).to_string())
                .collect::<String>(),
            "||hi there",
        );

        // The same row with a tail the text does not reach. A second Grid
        // places its own Positions, so the Cells are named through it.
        let mut app = Orcvs::new(10, 1).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.src("||hi");

        assert_eq!(
            (0..10)
                .map(|x| rendered(&app, at(x, 0)).to_string())
                .collect::<String>(),
            "||hi      ",
        );
    }

    #[tokio::test]
    async fn test_empty_cells_between_markers_remain_spaces() {
        let mut app = Orcvs::new(24, 16).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(8, 8));

        assert_eq!(rendered(&app, at(14, 10)), GlyphString::space());
        assert_eq!(rendered(&app, at(16, 10)), GlyphString::space());
        assert_eq!(rendered(&app, at(17, 10)), GlyphString::space());
    }
}
