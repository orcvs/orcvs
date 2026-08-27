use egui::{Event, Key};

use std::time::Duration;
use tracing::error;

use crate::glyph::GlyphString;
use crate::opts::{MarkerSpacing, Opts};

use crate::cursor::Cursor;
use crate::grid::{Grid, Position};
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
use crate::playback::InMemoryOutputAdapter;
use crate::playback::{PlaybackDiagnostic, PlaybackEngine, PlaybackState};
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
    pub opts: Opts,
    pub cursor: Cursor,
    pub grid: Grid,

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
    pub fn write(&mut self, s: &String) {
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

    ///
    /// The GlyphString rendered at `position`. Total: a Position can only come
    /// from a Grid, so every Position names a Cell that exists.
    ///
    pub fn get(&self, position: Position) -> GlyphString {
        let idx = self.index(position);

        let (s, g) = self.source.get(idx);
        match g {
            Some(g) => GlyphString::new(s, g),
            None => self.terminator(position),
        }
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
                Event::Text(text_to_insert) => {
                    if text_to_insert.len() == 1 && text_to_insert != " " {
                        self.write(text_to_insert);

                        repaint = true;
                    }
                }
                _ => {
                    // info!("Pressed");
                }
            }
        }
        repaint
    }

    ///
    /// The purely visual GlyphString for a Cell the Source leaves empty.
    ///
    fn terminator(&self, position: Position) -> GlyphString {
        let (x, y) = (position.x(), position.y());

        // Markers
        let marker_spacing = self.opts.marker_spacing.cells();
        if x % marker_spacing == 0 && y % marker_spacing == 0 {
            return GlyphString::marker();
        }

        // Highlight
        if in_marker_block(self.cursor.position(), position, self.opts.marker_spacing) {
            if x % self.opts.highlight_dot_spacing == 0 && y % self.opts.highlight_dot_spacing == 0
            {
                return GlyphString::highlight();
            }
        }

        return GlyphString::space();
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

///
/// True when `target` falls inside the marker block containing `cursor`.
/// Purely visual: `spacing` is the marker spacing, not a Source dimension.
///
fn in_marker_block(cursor: Position, target: Position, spacing: MarkerSpacing) -> bool {
    let x = target.x();
    let y = target.y();
    let cursor_x = cursor.x();
    let cursor_y = cursor.y();
    let spacing = spacing.cells();

    let min_x = cursor_x / spacing * spacing;
    let max_x = (cursor_x / spacing + 1).saturating_mul(spacing);
    let min_y = cursor_y / spacing * spacing;
    let max_y = (cursor_y / spacing + 1).saturating_mul(spacing);

    x >= min_x && x <= max_x && y >= min_y && y <= max_y
}

#[cfg(test)]
mod test {

    use super::{in_marker_block, App};
    use crate::{
        glyph::{Glyph, GlyphString},
        grid::Grid,
        opts::{MarkerSpacing, DEFAULT_MARKER_SPACING},
        test::trace,
    };

    fn app() -> App {
        let rows = 1; // * (DEFAULT_MARKER_SPACING as usize);
        let cols = DEFAULT_MARKER_SPACING;

        App::new(cols, rows)
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
            self.write(&s.to_owned());
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
        assert_eq!(app.get(at(0, 1)), written);
        assert_eq!(app.get(at(1, 1)), written);

        // and it is those Cells' content, not another's
        for row in grid.rows() {
            for position in row {
                if position == at(0, 1) || position == at(1, 1) {
                    continue;
                }
                assert_ne!(
                    app.get(position),
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
        assert_eq!(app.get(at(0, 0)), expected);
        assert_eq!(app.get(at(1, 0)), expected);
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
            app.get(position),
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

        app.write(&"+".to_string());

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
            app.get(grid.position(1, 0).expect("inside the grid")),
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

        let g = app.get(at(0, 0));
        assert_eq!(g, GlyphString::marker());

        let g = app.get(at(1, 0));
        assert_eq!(g, GlyphString::space());

        let g = app.get(at(2, 0));
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
            (0..7).map(|x| app.get(at(x, 0))).collect::<Vec<_>>(),
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
        assert!((0..7).all(|x| app.get(at(x, 0)) == GlyphString::marker()));
    }

    #[test]
    fn test_in_marker_block() {
        trace();
        let spacing = MarkerSpacing::new(8).unwrap();
        // Rectangular, and large enough to mint every position probed below
        let grid = Grid::new(64, 60);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        let selected = at(5, 5);
        assert!(!in_marker_block(selected, at(10, 10), spacing));
        for x in 0..spacing.cells() {
            for y in 0..spacing.cells() {
                assert!(in_marker_block(selected, at(x, y), spacing));
            }
        }

        let selected = at(8, 8);
        assert!(!in_marker_block(selected, at(1, 1), spacing));

        for x in 8..=16 {
            for y in 8..=16 {
                assert!(in_marker_block(selected, at(x, y), spacing));
            }
        }

        // Marker block X 5 Y 6
        let selected = at(42, 51);
        assert!(!in_marker_block(selected, at(1, 1), spacing));
        for x in 0..=spacing.cells() {
            for y in 0..=spacing.cells() {
                let x = x + 40;
                let y = y + 48;
                assert!(in_marker_block(selected, at(x, y), spacing));
            }
        }
    }

    #[test]
    fn test_marker_block_accepts_the_largest_whole_cell_spacing() {
        let grid = Grid::new(2, 2);
        let origin = grid.origin();

        assert!(in_marker_block(
            origin,
            origin,
            MarkerSpacing::new(usize::MAX).unwrap()
        ));
    }
}
