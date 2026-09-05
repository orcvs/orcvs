use std::time::Duration;
use tracing::error;

use crate::opts::{Bpm, Opts};

use crate::cursor::Cursor;
use crate::grid::{Grid, Position};
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
use crate::playback::InMemoryOutputAdapter;
use crate::playback::{OutputAdapter, PlaybackDiagnostic, PlaybackEngine, PlaybackState};
use crate::render_frame::{RenderFrame, RenderFrameConfig};
use crate::source::SourceCommander;

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
use crate::native_midi::{MidirBackend, NativeMidiOutputAdapter};

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

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
pub type OrcvsOutputAdapter = NativeMidiOutputAdapter;
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub type OrcvsOutputAdapter = InMemoryOutputAdapter;

///
/// One running Orcvs: its options, Source and Grid, Cursor, and Playback
/// lifecycle. Output-device discovery and selection belong to the shell.
///
/// Selection names a Position, and only a Grid mints one. A pair outside the
/// Grid never becomes a Position at all, so `select` has no rejection to make
/// and cannot silently leave the Cursor on the Cell it was already on and send
/// the next write there:
///
/// ```
/// use orcvs::app::Orcvs;
/// use orcvs::grid::Grid;
///
/// let orcvs = Orcvs::new(16, 16);
/// let grid = Grid::new(16, 16);
///
/// // the Grid refuses a pair outside itself, so there is no Position to select
/// assert_eq!(grid.position(99, 99), None);
///
/// // every Position `select` can be handed is one the Grid minted
/// let position = grid.position(15, 15).expect("inside the grid");
/// assert_eq!((position.x(), position.y()), (15, 15));
/// assert_eq!(orcvs.render_frame().rows().len(), 16);
/// ```
///
pub struct Orcvs<A: OutputAdapter = OrcvsOutputAdapter> {
    opts: Opts,
    cursor: Cursor,
    grid: Grid,

    source: SourceCommander,
    playback: PlaybackEngine<A>,
    playback_state: PlaybackState,
}

impl Orcvs {
    pub fn new(cols: usize, rows: usize) -> Self {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        let adapter = NativeMidiOutputAdapter::new(MidirBackend);
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let adapter = InMemoryOutputAdapter::default();
        Self::with_output_adapter(cols, rows, adapter)
    }
}

impl<A: OutputAdapter + Send + 'static> Orcvs<A> {
    pub fn with_output_adapter(cols: usize, rows: usize, adapter: A) -> Self {
        let grid = Grid::new(cols, rows);
        let opts = Opts::new();
        let source = SourceCommander::new(grid);
        let playback = PlaybackEngine::new(source.clone(), adapter);

        Self {
            cursor: Cursor::new(grid.origin(), opts.cursor_delay),
            grid,
            opts,
            source,
            playback,
            playback_state: PlaybackState::Stopped,
        }
    }

    pub fn observe_playback(&mut self) -> Vec<PlaybackDiagnostic> {
        let observation = self.playback.observe();
        self.playback_state = observation.state;
        observation.diagnostics
    }

    pub fn bpm(&self) -> Bpm {
        self.opts.bpm
    }

    pub fn set_bpm(&mut self, bpm: Bpm) {
        if self.playing()
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

    ///
    /// Convert a Position into a linear index.
    /// Total: a Position can only come from a Grid, so it is in range for the
    /// Grid that minted it.
    ///
    pub fn index(&self, position: Position) -> usize {
        self.grid.index(position).get()
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
                    if self.playing() {
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

    fn playing(&self) -> bool {
        self.playback_state == PlaybackState::Playing
    }

    fn stop(&mut self) {
        self.playback.stop();
        self.playback_state = PlaybackState::Stopped;
    }

    fn play(&mut self) {
        let ms = self.opts.bpm.delay_ms();
        // A start failure is already recorded as a Playback diagnostic, which
        // `observe_playback` hands to the shell; reporting it again here would
        // put one failure on two channels.
        if self.playback.start(Duration::from_millis(ms)).is_ok() {
            self.playback_state = PlaybackState::Playing;
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
impl<B: crate::midi::MidiBackend + 'static> Orcvs<crate::midi::MidiOutputAdapter<B>> {
    /// Returns the MIDI configuration capability without exposing Playback
    /// lifecycle control.
    ///
    /// A running Orcvs does not hand its complete Playback Engine to callers:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16);
    /// let _playback = orcvs.playback_engine();
    /// ```
    ///
    /// The MIDI selection handle cannot start Playback:
    ///
    /// ```compile_fail
    /// use std::time::Duration;
    /// let orcvs = orcvs::app::Orcvs::new(16, 16);
    /// orcvs
    ///     .midi_selection_handle()
    ///     .start(Duration::from_millis(100));
    /// ```
    ///
    /// It cannot stop or disconnect Playback:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16);
    /// orcvs.midi_selection_handle().stop();
    /// ```
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16);
    /// orcvs.midi_selection_handle().disconnect();
    /// ```
    ///
    /// It cannot observe Playback or drain its diagnostics:
    ///
    /// ```compile_fail
    /// let orcvs = orcvs::app::Orcvs::new(16, 16);
    /// let _observation = orcvs.midi_selection_handle().observe();
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

    #[test]
    fn user_can_change_the_tempo() {
        let mut orcvs =
            Orcvs::with_output_adapter(2, 1, crate::playback::InMemoryOutputAdapter::default());

        orcvs.set_bpm(Bpm::new(120).unwrap());

        assert_eq!(orcvs.bpm().beats_per_minute(), 120);
    }

    #[tokio::test(start_paused = true)]
    async fn repeated_tempo_changes_preserve_the_current_beat_phase() {
        let adapter = crate::playback::InMemoryOutputAdapter::default();
        let mut orcvs = Orcvs::with_output_adapter(2, 1, adapter.clone());
        orcvs.event_handler(vec![super::InputEvent::KeyPressed(super::InputKey::Space)]);
        tokio::task::yield_now().await;
        assert_eq!(adapter.command_lists().len(), 1);

        tokio::time::advance(Duration::from_millis(500)).await;
        for _ in 0..5 {
            orcvs.set_bpm(Bpm::new(20).unwrap());
            tokio::time::advance(Duration::from_millis(40)).await;
            tokio::task::yield_now().await;
        }

        assert_eq!(adapter.all_notes_off_count(), 0);
        assert_eq!(adapter.command_lists().len(), 1);
        orcvs.observe_playback();
        assert!(orcvs.playing());

        tokio::time::advance(Duration::from_millis(49)).await;
        tokio::task::yield_now().await;
        assert_eq!(adapter.command_lists().len(), 1);

        tokio::time::advance(Duration::from_millis(1)).await;
        tokio::task::yield_now().await;
        assert_eq!(adapter.command_lists().len(), 2);
    }

    ///
    /// Native only: the retune failure this states is a Playback Engine that
    /// finds no Tokio runtime, and staging it means building one by hand.
    /// `tokio::runtime::Runtime::new` is the multi-threaded builder, which the
    /// `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` table pulls
    /// in and a browser target never has.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn failed_tempo_retune_keeps_existing_playback_running() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let adapter = crate::playback::InMemoryOutputAdapter::default();
        let mut orcvs = Orcvs::with_output_adapter(2, 1, adapter.clone());
        runtime.block_on(async {
            orcvs.event_handler(vec![super::InputEvent::KeyPressed(super::InputKey::Space)]);
            tokio::task::yield_now().await;
        });

        orcvs.set_bpm(Bpm::new(120).unwrap());

        let diagnostics = orcvs.observe_playback();
        assert!(orcvs.playing());
        assert_eq!(orcvs.bpm().beats_per_minute(), 20);
        assert_eq!(adapter.all_notes_off_count(), 0);
        assert_eq!(
            diagnostics,
            vec![crate::playback::PlaybackDiagnostic::RetuneFailure {
                message: "Playback requires a Tokio runtime".to_owned(),
            }]
        );
    }

    #[test]
    fn app_exposes_a_render_frame_without_leaking_its_grid_or_cursor() {
        let mut app = Orcvs::new(2, 1);
        app.write("x");

        let frame = app.render_frame();

        assert_eq!(frame.rows().len(), 1);
        assert_eq!(frame.rows()[0].len(), 2);
        assert_eq!(frame.rows()[0][0].content(), Some('x'));
        assert_eq!(
            frame.rows()[0][1].position(),
            app.grid.position(1, 0).unwrap()
        );
        assert!(frame.rows()[0][1].selected());
    }

    #[test]
    fn deriving_a_render_frame_does_not_advance_cursor_blink_state() {
        let mut app = Orcvs::new(2, 1);
        app.cursor.on = true;

        let first = app.render_frame();
        let second = app.render_frame();

        assert!(first.rows()[0][0].cursor_visible());
        assert!(second.rows()[0][0].cursor_visible());
        assert!(app.cursor.on);
    }

    fn app() -> Orcvs {
        let rows = 1; // * (DEFAULT_MARKER_SPACING as usize);
        let cols = DEFAULT_MARKER_SPACING;

        Orcvs::new(cols, rows)
    }

    fn rendered(app: &Orcvs, position: crate::grid::Position) -> GlyphString {
        let frame = app.render_frame();
        let cell = frame
            .rows()
            .iter()
            .flatten()
            .find(|cell| cell.position() == position)
            .expect("Render Frame contains every Grid Position");
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
    async fn test_to_idx() {
        trace();
        let app = Orcvs::new(10, 4);

        let position = app.grid.position(0, 0).expect("inside the grid");
        let idx = app.index(position);
        assert_eq!(idx, 0);

        let position = app.grid.position(5, 3).expect("inside the grid");
        let idx = app.index(position);
        assert_eq!(idx, 35);
    }

    #[tokio::test]
    async fn test_get_reads_the_cell_at_the_position() {
        trace();

        // 4 columns, 2 rows: transposing the axes addresses a different Cell.
        let mut app = Orcvs::new(4, 2);
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
        for row in grid.rows() {
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

        let mut app = Orcvs::new(10, 1);
        let position = app.grid.position(5, 0).expect("inside the grid");
        app.set_at(0, 0, ".");
        app.set_at(1, 0, "+");

        app.set_at(5, 0, "x");

        assert_eq!(
            rendered(&app, position),
            GlyphString::new(Some("x".to_string()), Glyph::Char)
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
        let mut app = Orcvs::new(7, 3);
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(6, 2));
        app.opts.marker_spacing = MarkerSpacing::new(2).unwrap();

        let frame = app.render_frame();
        assert_eq!(
            frame.rows()[0]
                .iter()
                .map(|cell| cell.sector_left_strength().is_some())
                .collect::<Vec<_>>(),
            vec![false, false, true, false, true, false, true]
        );
        assert!(
            frame.rows()[0]
                .iter()
                .all(|cell| cell.glyph() == Glyph::Space)
        );

        app.opts.marker_spacing = MarkerSpacing::new(1).unwrap();
        let frame = app.render_frame();
        assert_eq!(frame.rows()[0][0].sector_left_strength(), None);
        assert!(
            frame.rows()[0][1..]
                .iter()
                .all(|cell| cell.sector_left_strength().is_some())
        );
    }

    #[tokio::test]
    async fn test_empty_cells_between_markers_remain_spaces() {
        let mut app = Orcvs::new(24, 16);
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(8, 8));

        assert_eq!(rendered(&app, at(14, 10)), GlyphString::space());
        assert_eq!(rendered(&app, at(16, 10)), GlyphString::space());
        assert_eq!(rendered(&app, at(17, 10)), GlyphString::space());
    }
}
