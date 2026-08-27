use egui::{Event, Key};

use std::time::Duration;
use tracing::error;

use crate::opts::Opts;

use crate::cursor::Cursor;
use crate::grid::{Grid, Position};
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
use crate::playback::InMemoryOutputAdapter;
use crate::playback::{PlaybackDiagnostic, PlaybackEngine, PlaybackState};
use crate::render_frame::{RenderFrame, RenderFrameConfig};
use crate::source::SourceCommander;

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
use crate::midi::{MidiDestination, MidiDestinationId};
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
use crate::native_midi::{MidirBackend, NativeMidiOutputAdapter};

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
type AppOutputAdapter = NativeMidiOutputAdapter;
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
type AppOutputAdapter = InMemoryOutputAdapter;

///
/// The console's editing state: the Grid that is the Source's shape, the
/// Cursor selecting one of its Positions, and the commander that owns the
/// Source itself.
///
/// Selection names a Position, and only a Grid mints one. A pair outside the
/// Grid never becomes a Position at all, so `select` has no rejection to make
/// and cannot silently leave the Cursor on the Cell it was already on and send
/// the next write there:
///
/// ```
/// use console::grid::Grid;
///
/// let grid = Grid::new(16, 16);
///
/// // the Grid refuses a pair outside itself, so there is no Position to select
/// assert_eq!(grid.position(99, 99), None);
///
/// // every Position `select` can be handed is one the Grid minted
/// let position = grid.position(15, 15).expect("inside the grid");
/// assert_eq!((position.x(), position.y()), (15, 15));
/// ```
///
pub struct App {
    opts: Opts,
    cursor: Cursor,
    grid: Grid,

    source: SourceCommander,
    playback: PlaybackEngine<AppOutputAdapter>,
    playback_state: PlaybackState,
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    midi_destinations: Vec<MidiDestination>,
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    midi_status: Option<String>,
}

impl App {
    pub fn new(cols: usize, rows: usize) -> Self {
        let grid = Grid::new(cols, rows);

        let opts = Opts::new();

        let source = SourceCommander::new(grid);
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        let adapter = NativeMidiOutputAdapter::new(MidirBackend);
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let adapter = InMemoryOutputAdapter::default();
        let playback = PlaybackEngine::new(source.clone(), adapter);

        Self {
            cursor: Cursor::new(grid.origin(), opts.cursor_delay),
            grid,
            opts,
            source,
            playback,
            playback_state: PlaybackState::Stopped,
            #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
            midi_destinations: Vec::new(),
            #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
            midi_status: None,
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    pub fn refresh_midi_destinations(&mut self) {
        match self.playback.midi_destinations() {
            Ok(destinations) => {
                self.midi_destinations = destinations;
                self.midi_status = None;
            }
            Err(error) => self.midi_status = Some(error.message),
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    pub fn midi_destinations(&self) -> &[MidiDestination] {
        &self.midi_destinations
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    pub fn select_midi_destination(&mut self, destination_id: &MidiDestinationId) {
        match self.playback.select_midi_destination(destination_id) {
            Ok(()) => self.midi_status = None,
            Err(error) => self.midi_status = Some(error.message),
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    pub fn selected_midi_destination_id(&self) -> Option<MidiDestinationId> {
        self.playback.selected_midi_destination_id()
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    pub fn midi_status(&self) -> Option<String> {
        self.midi_status.clone()
    }

    pub fn observe_playback(&mut self) {
        let observation = self.playback.observe();
        self.playback_state = observation.state;
        for diagnostic in observation.diagnostics {
            match diagnostic {
                PlaybackDiagnostic::OutputFailure(error) => {
                    self.record_playback_failure(error.message)
                }
                PlaybackDiagnostic::ClockFailure { message } => {
                    self.record_playback_failure(message)
                }
                PlaybackDiagnostic::Overrun { .. } => {}
            }
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    fn record_playback_failure(&mut self, message: String) {
        self.midi_status = Some(message);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    fn record_playback_failure(&mut self, message: String) {
        error!("Playback failure: {message}");
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
        let idx = self.cursor_index();

        match self.source.set(idx, s) {
            Ok(_) => self.cursor.select(self.grid.right(self.cursor.position())),
            Err(e) => error!("rejected edit: {e}"),
        }
    }

    fn delete(&mut self) {
        let idx = self.cursor_index();

        match self.source.unset(idx) {
            Ok(_) => self.cursor.select(self.grid.left(self.cursor.position())),
            Err(e) => error!("rejected delete: {e}"),
        }
    }

    pub fn cursor_index(&self) -> usize {
        self.index(self.cursor.position())
    }

    ///
    /// Convert a Position into a linear index.
    /// Total: a Position can only come from a Grid, so it is in range for the
    /// Grid that minted it.
    ///
    pub fn index(&self, position: Position) -> usize {
        self.grid.index(position)
    }

    pub fn render_frame(&self) -> RenderFrame {
        RenderFrame::derive(
            self.source.read_revision_cells(),
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

    ///
    /// Handles event and returns boolean indicating if repating is required
    ///
    pub fn event_handler(&mut self, events: Vec<Event>) -> bool {
        let mut repaint = false;
        for event in &events {
            match event {
                Event::Key {
                    key: Key::ArrowDown,
                    pressed: true,
                    ..
                } => self.cursor.select(self.grid.down(self.cursor.position())),
                Event::Key {
                    key: Key::ArrowLeft,
                    pressed: true,
                    ..
                } => self.cursor.select(self.grid.left(self.cursor.position())),
                Event::Key {
                    key: Key::ArrowRight,
                    pressed: true,
                    ..
                } => self.cursor.select(self.grid.right(self.cursor.position())),
                Event::Key {
                    key: Key::ArrowUp,
                    pressed: true,
                    ..
                } => self.cursor.select(self.grid.up(self.cursor.position())),
                Event::Key {
                    key: Key::Backspace,
                    pressed: true,
                    ..
                } => self.delete(),
                Event::Key {
                    key: Key::Delete,
                    pressed: true,
                    ..
                } => self.delete(),
                Event::Key {
                    key: Key::Space,
                    pressed: true,
                    ..
                } => {
                    if self.playing() {
                        self.stop();
                    } else {
                        self.play();
                    }
                }
                Event::Text(text_to_insert)
                    if text_to_insert.len() == 1 && text_to_insert != " " =>
                {
                    self.write(text_to_insert);
                    repaint = true;
                }
                _ => {
                    // info!("Pressed");
                }
            }
        }
        repaint
    }

    fn playing(&self) -> bool {
        self.playback_state == PlaybackState::Playing
    }

    fn stop(&mut self) {
        self.playback.stop();
        self.observe_playback();
    }

    fn play(&mut self) {
        let ms = self.opts.bpm.delay_ms();
        if let Err(error) = self.playback.start(Duration::from_millis(ms)) {
            #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
            {
                self.midi_status = Some(error.to_string());
            }
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
            error!("Playback did not start: {error}");
        }
        self.observe_playback();
    }
}

#[cfg(test)]
mod test {

    use super::App;
    use crate::{
        glyph::{Glyph, GlyphString},
        opts::{MarkerSpacing, DEFAULT_MARKER_SPACING},
        test::trace,
    };

    #[test]
    fn app_exposes_a_render_frame_without_leaking_its_grid_or_cursor() {
        let mut app = App::new(2, 1);
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
        let mut app = App::new(2, 1);
        app.cursor.on = true;

        let first = app.render_frame();
        let second = app.render_frame();

        assert!(first.rows()[0][0].cursor_visible());
        assert!(second.rows()[0][0].cursor_visible());
        assert!(app.cursor.on);
    }

    fn app() -> App {
        let rows = 1; // * (DEFAULT_MARKER_SPACING as usize);
        let cols = DEFAULT_MARKER_SPACING;

        App::new(cols, rows)
    }

    fn rendered(app: &App, position: crate::grid::Position) -> GlyphString {
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

    impl App {
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
        let app = App::new(10, 4);

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
        let mut app = App::new(4, 2);
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        // written into the second row
        app.set_at(0, 1, "+");
        app.set_at(1, 1, "+");

        let written = GlyphString::new(Some("+".to_string()), Glyph::Function);
        assert_eq!(rendered(&app, at(0, 1)), written);
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

        app.set_at(0, 0, "+");
        app.set_at(1, 0, "+");

        // the accepted edits are observable as soon as write returns
        let expected = GlyphString::new(Some("+".to_string()), Glyph::Function);
        assert_eq!(rendered(&app, at(0, 0)), expected);
        assert_eq!(rendered(&app, at(1, 0)), expected);
    }

    #[tokio::test]
    async fn test_editing_an_operand_hint_never_renders_an_occupied_cell_as_empty() {
        trace();

        let mut app = App::new(10, 1);
        let position = app.grid.position(5, 0).expect("inside the grid");
        app.set_at(0, 0, "+");
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
    async fn test_empty_source_exposes_marker_space_and_highlight_glyphs() {
        trace();

        let mut app = app();
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        app.select_or_panic(7, 0);

        let g = rendered(&app, at(0, 0));
        assert_eq!(g, GlyphString::marker());

        let g = rendered(&app, at(1, 0));
        assert_eq!(g, GlyphString::space());

        let g = rendered(&app, at(2, 0));
        assert_eq!(g, GlyphString::highlight());
    }

    #[tokio::test]
    async fn test_marker_placement_uses_one_whole_cell_spacing() {
        let mut app = App::new(7, 3);
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(6, 2));
        app.opts.marker_spacing = MarkerSpacing::new(2).unwrap();

        assert_eq!(
            (0..7).map(|x| rendered(&app, at(x, 0))).collect::<Vec<_>>(),
            vec![
                GlyphString::marker(),
                GlyphString::space(),
                GlyphString::marker(),
                GlyphString::space(),
                GlyphString::marker(),
                GlyphString::space(),
                GlyphString::marker(),
            ]
        );

        app.opts.marker_spacing = MarkerSpacing::new(1).unwrap();
        assert!((0..7).all(|x| rendered(&app, at(x, 0)) == GlyphString::marker()));
    }

    #[tokio::test]
    async fn test_highlight_does_not_reach_into_the_next_marker_block() {
        let mut app = App::new(24, 16);
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        app.select(at(8, 8));

        assert_eq!(rendered(&app, at(14, 10)), GlyphString::highlight());
        assert_eq!(rendered(&app, at(16, 10)), GlyphString::space());
    }
}
