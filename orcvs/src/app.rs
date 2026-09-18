use std::time::Duration;
use tracing::error;

use crate::midi::{MidiOutputAdapter, MidiSelectionHandle};
use crate::opts::{Bpm, Opts};

use crate::cursor::Cursor;
use crate::grid::{Grid, Position};
use crate::playback::{OutputOnlyAdapter, PlaybackDiagnostic, PlaybackEngine, PlaybackStartError};
use crate::region::Region;
use crate::render_frame::{RenderFrame, RenderFrameConfig};
use crate::source::{CellContent, CellWrite, Source, SourceCommander};

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

///
/// One of the four directions an arrow key moves the Cursor.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    Down,
    Left,
    Right,
    Up,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    KeyPressed(InputKey),
    ///
    /// Shift with an arrow: moves the Cursor and keeps the anchor, so the
    /// Region grows or shrinks from the Cell it was spanned from.
    ///
    Extend(Arrow),
    ///
    /// Command `A`: spans the Region across the whole Grid.
    ///
    SelectAll,
    ///
    /// Escape: collapses the Region onto the Cursor.
    ///
    Collapse,
    ///
    /// Command Enter: the next event, when it is a character, fills every
    /// Cell of the Region with it. Any other event disarms the fill.
    ///
    Fill,
    Text(String),
}

///
/// One running Orcvs: its options, Source and Grid, Cursor, and Playback
/// lifecycle. Output-device discovery and selection belong to the console.
///
/// `S` is the selection capability, not the output backend. MIDI construction
/// supplies a [`MidiSelectionHandle`]; output-only construction uses `()`.
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
pub struct Orcvs<S = MidiSelectionHandle> {
    opts: Opts,
    cursor: Cursor,
    ///
    /// The Cell the Region is spanned from; the Cursor is its other end.
    ///
    /// A Position rather than a `Region`, because the Cursor already holds the
    /// live end and a second copy of it would be a second truth to keep in
    /// step. It is running state and never stored with the Source.
    ///
    anchor: Position,
    ///
    /// Whether command Enter has armed a fill for the next character.
    ///
    /// Held across input batches, because the chord and the character are
    /// two presses a viewer makes one after the other, and those may arrive in
    /// different frames.
    ///
    fill_armed: bool,
    grid: Grid,

    source: SourceCommander,
    playback: PlaybackEngine,
    selection: S,
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
        Self::with_midi_output_adapter(cols, rows, MidiOutputAdapter::new())
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
    /// ```compile_fail
    /// use orcvs::{app::Orcvs, playback::InMemoryOutputAdapter};
    /// let app = Orcvs::with_output_adapter(1, 1, InMemoryOutputAdapter::default()).unwrap();
    /// app.midi_selection_handle();
    /// ```
    ///
    /// ```compile_fail
    /// use orcvs::app::Orcvs;
    /// use orcvs::midi::MidiOutputAdapter;
    ///
    /// let _orcvs = Orcvs::with_output_adapter(1, 1, MidiOutputAdapter::new()).unwrap();
    /// ```
    pub fn with_output_adapter<A: OutputOnlyAdapter + Send + 'static>(
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
            cursor: Cursor::new(grid.origin()),
            anchor: grid.origin(),
            fill_armed: false,
            grid,
            opts,
            source,
            playback,
            selection,
            playback_requested: false,
        }
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

    ///
    /// The observation the Playback Engine last published, read without
    /// awaiting and without reaching the engine.
    ///
    /// ADR 0041 has the console read this while drawing a frame rather than
    /// asking the engine a question: the browser main thread has no blocking
    /// receive, so a frame cannot wait for an answer at all.
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
    /// Moves the Cursor to `position` and collapses the Region onto it,
    /// refusing a Position minted by another Grid.
    ///
    pub fn select(&mut self, position: Position) {
        self.grid.assert_owns(position);
        self.collapse_to(position);
    }

    ///
    /// Moves the Cursor to `position` and keeps the anchor, so the Region
    /// spans from the anchor to `position`.
    ///
    /// ```
    /// use orcvs::app::Orcvs;
    ///
    /// # let runtime = tokio::runtime::Runtime::new().unwrap();
    /// # let _runtime = runtime.enter();
    /// let mut orcvs = Orcvs::new(8, 4).expect("a Tokio runtime");
    /// let grid = orcvs.grid();
    /// let at = |x, y| grid.position(x, y).expect("inside the Grid");
    ///
    /// orcvs.select(at(5, 3));
    /// orcvs.extend(at(2, 1));
    ///
    /// let region = orcvs.render_frame().region();
    /// assert_eq!((region.columns(), region.rows()), (2..6, 1..4));
    /// ```
    ///
    pub fn extend(&mut self, position: Position) {
        self.grid.assert_owns(position);
        self.cursor.select(position);
    }

    ///
    /// The Region from the anchor to the Cursor.
    ///
    pub fn region(&self) -> Region {
        Region::span(self.grid, self.anchor, self.cursor.position())
    }

    ///
    /// Moves the Cursor to `position` and the anchor with it, so the Region
    /// is that one Cell.
    ///
    fn collapse_to(&mut self, position: Position) {
        self.cursor.select(position);
        self.anchor = position;
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
            Ok(_) => self.collapse_to(self.grid.right(self.cursor.position())),
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
        self.collapse_to(self.grid.left(self.cursor.position()));
    }

    pub fn render_frame(&self) -> RenderFrame {
        RenderFrame::derive(
            self.source.read_revision(),
            self.region(),
            self.cursor.on,
            RenderFrameConfig {
                sector_seam_spacing: self.opts.sector_seam_spacing,
                cursor_bloom_radius: self.opts.cursor_bloom_radius,
            },
        )
    }

    ///
    /// Handles event and returns boolean indicating if repating is required
    ///
    pub fn event_handler(&mut self, events: Vec<InputEvent>) -> bool {
        let mut repaint = false;
        for event in &events {
            // A fill is armed for exactly one event: the character it fills
            // with, or whatever else arrived instead and so disarmed it.
            let fill_armed = std::mem::take(&mut self.fill_armed);
            match event {
                InputEvent::KeyPressed(InputKey::ArrowDown) => {
                    self.collapse_to(self.stepped(Arrow::Down))
                }
                InputEvent::KeyPressed(InputKey::ArrowLeft) => {
                    self.collapse_to(self.stepped(Arrow::Left))
                }
                InputEvent::KeyPressed(InputKey::ArrowRight) => {
                    self.collapse_to(self.stepped(Arrow::Right))
                }
                InputEvent::KeyPressed(InputKey::ArrowUp) => {
                    self.collapse_to(self.stepped(Arrow::Up))
                }
                InputEvent::Extend(arrow) => self.cursor.select(self.stepped(*arrow)),
                InputEvent::SelectAll => {
                    let whole = Region::whole(self.grid);
                    self.anchor = whole.anchor();
                    self.cursor.select(whole.cursor());
                }
                InputEvent::Collapse => self.anchor = self.cursor.position(),
                InputEvent::KeyPressed(InputKey::Backspace | InputKey::Delete) => {
                    if self.region().is_one_cell() {
                        self.delete();
                    } else {
                        self.fill(CellContent::SPACE);
                    }
                    repaint = true;
                }
                InputEvent::Fill => self.fill_armed = true,
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
                    match CellContent::new(text_to_insert.as_bytes()[0]) {
                        Some(content) if fill_armed => self.fill(content),
                        _ => self.write(text_to_insert),
                    }
                    repaint = true;
                }
                InputEvent::Text(_) => {}
            }
        }
        repaint
    }

    ///
    /// Writes `content` into every Cell of the Region, in one revision, and
    /// keeps the Region.
    ///
    fn fill(&mut self, content: CellContent) {
        let writes: Vec<CellWrite> = self
            .region()
            .positions_by_row()
            .flatten()
            .map(|position| CellWrite {
                cell: self.grid.index(position),
                content,
            })
            .collect();
        self.source.write_cells(&writes);
    }

    ///
    /// The Cell one step from the Cursor towards `arrow`, clamped at the
    /// Grid's edge.
    ///
    fn stepped(&self, arrow: Arrow) -> Position {
        let cursor = self.cursor.position();
        match arrow {
            Arrow::Down => self.grid.down(cursor),
            Arrow::Left => self.grid.left(cursor),
            Arrow::Right => self.grid.right(cursor),
            Arrow::Up => self.grid.up(cursor),
        }
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

impl Orcvs {
    /// A running Orcvs with MIDI discovery and selection connected to Playback.
    pub fn with_midi_output_adapter(
        cols: usize,
        rows: usize,
        adapter: MidiOutputAdapter,
    ) -> Result<Self, PlaybackStartError> {
        Self::with_source_and_midi_output_adapter(Source::new(Grid::new(cols, rows)), adapter)
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
    pub fn midi_selection_handle(&self) -> MidiSelectionHandle {
        self.selection.clone()
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::Orcvs;
    use crate::opts::Bpm;
    use crate::playback::PlaybackState;
    use crate::region::Region;
    use crate::source::Tick;
    use crate::test::trace;
    use crate::{opts::DEFAULT_SECTOR_SEAM_SPACING, source::Token};

    #[tokio::test]
    async fn playback_observation_is_tick_zero_stopped_and_run_clock_zero_before_the_first_run() {
        let orcvs =
            Orcvs::with_output_adapter(2, 1, crate::playback::InMemoryOutputAdapter::default())
                .expect("the test runtime");

        let observation = orcvs.playback_observation();

        assert_eq!(observation.state, PlaybackState::Stopped);
        assert_eq!(observation.tick, Tick::ZERO);
        assert!(observation.on_beat);
        assert_eq!(observation.run_clock(), Duration::ZERO);
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
    async fn a_fresh_orcvs_opens_at_one_hundred_and_twenty_bpm() {
        let orcvs =
            Orcvs::with_output_adapter(2, 1, crate::playback::InMemoryOutputAdapter::default())
                .expect("the test runtime");

        assert_eq!(orcvs.bpm().beats_per_minute(), 120);
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
        assert_eq!(frame.cursor(), app.grid.position(1, 0).unwrap());
    }

    ///
    /// The Cursor is the Position `derive` was given, carried rather than found.
    ///
    #[tokio::test]
    async fn render_frame_answers_the_cursor_it_was_derived_for() {
        let mut app = Orcvs::new(4, 3).expect("the test runtime");
        let origin = app.grid.origin();
        let initial = app.render_frame();
        assert_eq!(initial.cursor(), origin);
        assert!(!initial.cursor_visible());

        let moved = app.grid.position(2, 1).unwrap();
        app.select(moved);
        let moved_frame = app.render_frame();
        assert_eq!(moved_frame.cursor(), moved);
        assert!(!moved_frame.cursor_visible());
    }

    #[tokio::test]
    async fn a_fresh_orcvs_has_a_region_of_the_cursors_one_cell() {
        let app = Orcvs::new(4, 3).expect("the test runtime");

        let region = app.render_frame().region();

        assert!(region.is_one_cell());
        assert_eq!(region.cursor(), app.grid.origin());
    }

    ///
    /// `extend` keeps the anchor, `select` moves it with the Cursor, and a
    /// write collapses the Region onto the Cell it stepped to.
    ///
    #[tokio::test]
    async fn extend_spans_a_region_that_select_and_a_write_collapse() {
        let mut app = Orcvs::new(6, 4).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");

        app.select(at(1, 1));
        app.extend(at(3, 2));
        let spanned = app.render_frame().region();
        assert_eq!((spanned.columns(), spanned.rows()), (1..4, 1..3));
        assert_eq!(app.render_frame().cursor(), at(3, 2));

        app.select(at(4, 0));
        assert_eq!(app.region(), Region::at(grid, at(4, 0)));

        app.extend(at(0, 0));
        app.write("x");
        assert_eq!(app.region(), Region::at(grid, at(1, 0)));
    }

    ///
    /// Shift with an arrow moves the Cursor and keeps the anchor, and passing
    /// the anchor flips the Region; a bare arrow collapses it and moves.
    ///
    #[tokio::test]
    async fn shift_arrows_extend_the_region_and_a_bare_arrow_collapses_it() {
        use super::{Arrow, InputEvent, InputKey};

        let mut app = Orcvs::new(8, 6).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(3, 3));

        app.event_handler(vec![
            InputEvent::Extend(Arrow::Right),
            InputEvent::Extend(Arrow::Right),
            InputEvent::Extend(Arrow::Down),
        ]);
        let region = app.region();
        assert_eq!((region.columns(), region.rows()), (3..6, 3..5));
        assert_eq!(region.cursor(), at(5, 4));

        app.event_handler(vec![InputEvent::Extend(Arrow::Left); 4]);
        let flipped = app.region();
        assert_eq!((flipped.columns(), flipped.rows()), (1..4, 3..5));
        assert_eq!(flipped.anchor(), at(3, 3));

        app.event_handler(vec![InputEvent::KeyPressed(InputKey::ArrowUp)]);
        assert_eq!(app.region(), Region::at(grid, at(1, 3)));
    }

    ///
    /// Command `A` spans the whole Grid and Escape collapses the Region onto
    /// the Cursor. Neither writes to the Source.
    ///
    #[tokio::test]
    async fn select_all_spans_the_grid_and_collapse_returns_to_the_cursor() {
        use super::{Arrow, InputEvent};

        let mut app = Orcvs::new(5, 4).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        let before = app.source.snapshot();

        app.event_handler(vec![InputEvent::SelectAll]);
        assert_eq!(app.region(), Region::whole(grid));

        app.event_handler(vec![InputEvent::Extend(Arrow::Right), InputEvent::Collapse]);
        assert_eq!(app.region(), Region::at(grid, at(1, 0)));

        assert_eq!(
            app.source.snapshot(),
            before,
            "a Region chord wrote to the Source"
        );
    }

    ///
    /// The Source's rows, as a test reads them back.
    ///
    fn rows(app: &Orcvs) -> Vec<String> {
        let snapshot = app.source.snapshot();
        snapshot
            .as_bytes()
            .chunks(app.grid.columns())
            .map(|row| String::from_utf8(row.to_vec()).expect("printable ASCII"))
            .collect()
    }

    ///
    /// An Orcvs of `text`'s rows, each padded to the widest.
    ///
    fn written(text: &[&str]) -> Orcvs {
        let columns = text.iter().map(|row| row.len()).max().unwrap_or(1);
        let mut app = Orcvs::new(columns, text.len()).expect("the test runtime");
        for (y, row) in text.iter().enumerate() {
            for (x, character) in row.chars().enumerate() {
                app.set_at(x, y, &character.to_string());
            }
        }
        app
    }

    #[tokio::test]
    async fn backspace_and_delete_empty_a_region_larger_than_one_cell_and_keep_it() {
        use super::{InputEvent, InputKey};

        for key in [InputKey::Backspace, InputKey::Delete] {
            let mut app = written(&["abcd", "efgh", "ijkl"]);
            let grid = app.grid;
            let at = |x, y| grid.position(x, y).expect("inside the Grid");
            let (anchor, cursor) = (at(2, 2), at(1, 0));
            app.select(anchor);
            app.extend(cursor);

            app.event_handler(vec![InputEvent::KeyPressed(key)]);

            assert_eq!(rows(&app), ["a  d", "e  h", "i  l"], "{key:?}");
            assert_eq!(app.region(), Region::span(grid, anchor, cursor));
        }
    }

    #[tokio::test]
    async fn backspace_and_delete_on_one_cell_empty_it_and_step_left() {
        use super::{InputEvent, InputKey};

        for key in [InputKey::Backspace, InputKey::Delete] {
            let mut app = written(&["abcd"]);
            let grid = app.grid;
            let at = |x| grid.position(x, 0).expect("inside the Grid");
            app.select(at(2));

            app.event_handler(vec![InputEvent::KeyPressed(key)]);

            assert_eq!(rows(&app), ["ab d"], "{key:?}");
            assert_eq!(app.region(), Region::at(grid, at(1)));
        }
    }

    #[tokio::test]
    async fn typing_in_a_region_writes_at_the_cursor_steps_right_and_collapses() {
        use super::InputEvent;

        let mut app = written(&["....", "...."]);
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(0, 0));
        app.extend(at(2, 1));

        app.event_handler(vec![InputEvent::Text("x".to_owned())]);

        assert_eq!(rows(&app), ["....", "..x."]);
        assert_eq!(app.region(), Region::at(grid, at(3, 1)));
    }

    ///
    /// Command Enter arms a fill that the next character carries into every
    /// Cell of the Region. A keystroke on its own never fills, and any other
    /// event between the chord and the character disarms it.
    ///
    #[tokio::test]
    async fn command_enter_then_a_character_fills_the_region() {
        use super::{Arrow, InputEvent};

        let mut app = written(&["....", "....", "...."]);
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(1, 0));
        app.extend(at(2, 1));
        let region = app.region();

        app.event_handler(vec![InputEvent::Fill]);
        app.event_handler(vec![InputEvent::Text("#".to_owned())]);

        assert_eq!(rows(&app), [".##.", ".##.", "...."]);
        assert_eq!(app.region(), region, "a fill keeps the Region");

        // Disarmed by an arrow between the chord and the character: the
        // character is typed rather than filled.
        app.event_handler(vec![
            InputEvent::Fill,
            InputEvent::Extend(Arrow::Down),
            InputEvent::Text("*".to_owned()),
        ]);
        assert_eq!(rows(&app), [".##.", ".##.", "..*."]);

        // A plain keystroke with a Region never fills.
        app.select(at(0, 0));
        app.extend(at(3, 2));
        app.event_handler(vec![InputEvent::Text("=".to_owned())]);
        assert_eq!(rows(&app), [".##.", ".##.", "..*="]);
    }

    #[tokio::test]
    async fn deriving_a_render_frame_does_not_change_cursor_visibility() {
        let mut app = orcvs();
        app.cursor.on = false;

        let first = app.render_frame();
        let second = app.render_frame();

        assert!(!first.cursor_visible());
        assert!(!second.cursor_visible());
        assert!(!app.cursor.on);
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
        let rows = 1; // * (DEFAULT_SECTOR_SEAM_SPACING as usize);
        let cols = DEFAULT_SECTOR_SEAM_SPACING;

        Orcvs::new(cols, rows).expect("the test runtime")
    }

    fn rendered(app: &Orcvs, position: crate::grid::Position) -> (Option<char>, Option<Token>) {
        let frame = app.render_frame();
        let cell = frame.at(position);
        (cell.content(), cell.token())
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
            self.collapse_to(position);
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

        let mut app = Orcvs::new(10, 1).expect("the test runtime");
        let position = app.grid.position(5, 0).expect("inside the grid");
        app.set_at(0, 0, ".");
        app.set_at(1, 0, "+");

        app.set_at(5, 0, "x");

        // Cell 5 is the second Cell of the Addition's second operand — ADR
        // 0033 has `.+` claim six Cells whatever they hold — so it is
        // presented as the operand it is. What this test is about is that the
        // character survives the classification: an operand-slot Token never
        // renders an occupied Cell as empty.
        assert_eq!(rendered(&app, position), (Some('x'), Some(Token::Number)));
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
            (None, None)
        );
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
    /// A Comment claims every remaining Cell of its row, empty Cells
    /// included, so those Cells now carry a Token where before they carried
    /// none. The Token decides the colour and nothing else: a Comment hints
    /// at no spelling, so a Cell it claims renders the character the Source
    /// holds, and an empty one renders empty.
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

        let mut app = Orcvs::new(10, 1).expect("the test runtime");
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.src("||hi there");

        // Every Cell of the row belongs to the one Comment, so every Cell
        // carries its Token, the space between the two words included.
        let frame = app.render_frame();
        assert!((0..10).all(|x| frame.at(at(x, 0)).token() == Some(Token::Comment)));
        // And each renders what the Source holds there, no more.
        assert_eq!(
            (0..10)
                .map(|x| rendered(&app, at(x, 0)).0.unwrap_or(' '))
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
                .map(|x| rendered(&app, at(x, 0)).0.unwrap_or(' '))
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

        assert_eq!(rendered(&app, at(14, 10)), (None, None));
        assert_eq!(rendered(&app, at(16, 10)), (None, None));
        assert_eq!(rendered(&app, at(17, 10)), (None, None));
    }
}
