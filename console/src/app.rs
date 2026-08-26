use egui::{Event, Key};

use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::{task, time};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::glyph::GlyphString;
use crate::opts::Opts;

use crate::cursor::Cursor;
use crate::grid::{Grid, Position};
use crate::source::{Command, SourceCommander};

///
/// The console's editing state: the Grid that is the Source's shape, the
/// Cursor selecting one of its Positions, and the commander that owns the
/// Source itself.
///
/// Selection names a Position, and only a Grid mints one. There is no
/// coordinate-taking selection to hand a pair the Grid would refuse, so a
/// rejected selection cannot silently leave the Cursor on the Cell it was
/// already on and send the next write there:
///
/// ```compile_fail
/// use console::app::App;
///
/// let mut app = App::new(16, 16);
///
/// app.select_at(99, 99);
/// ```
///
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct App {
    pub opts: Opts,
    pub cursor: Cursor,
    pub grid: Grid,

    token: Option<CancellationToken>,
    source: SourceCommander,
}

impl App {
    pub fn new(cols: usize, rows: usize) -> Self {
        let grid = Grid::new(cols, rows);

        let opts = Opts::new();

        let source = SourceCommander::spawn(grid);

        Self {
            cursor: Cursor::new(grid.origin(), opts.cursor_delay),
            grid,
            opts,
            source,
            token: None,
        }
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
        if x as f32 % self.opts.marker_spacing == 0.0 && y as f32 % self.opts.marker_spacing == 0.0
        {
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
        self.token.is_some()
    }

    // TODO(issue 05): unwired — no input reaches it, and it cancels the token
    // without clearing `self.token`, so `playing()` still reports true afterwards.
    // Playback lifecycle belongs to the Playback Engine.
    // See .scratch/source-playback-engine/issues/05-run-live-editing-through-the-playback-engine.md
    fn pause(&mut self) {
        if let Some(token) = &self.token {
            token.cancel();
        }
    }

    fn stop(&mut self) {
        if let Some(token) = &self.token {
            token.cancel();
            self.token = None;
        }
    }

    fn play(&mut self) {
        let token = CancellationToken::new();
        let cln_token = token.clone();
        self.token = Some(token);

        let ms = self.opts.bpm.delay_ms();

        let snd = self.source.sender();

        task::spawn(async move {
            tokio::select! {
                _ = cln_token.cancelled() => {
                    info!("cancelled");
                }
                _ = Self::ticker(ms, snd) => {
                    info!("done");
                }
            }
        });
    }

    async fn ticker(ms: u64, snd: Sender<Command>) {
        let mut interval = time::interval(Duration::from_millis(ms));
        info!("ticker");
        loop {
            match snd.send(Command::Tick).await {
                Ok(_) => {}
                Err(e) => {
                    error!("error {e:?}");
                }
            }
            interval.tick().await;
        }
    }
}

///
/// True when `target` falls inside the marker block containing `cursor`.
/// Purely visual: `spacing` is the marker spacing, not a Source dimension.
///
fn in_marker_block(cursor: Position, target: Position, spacing: f32) -> bool {
    let x = target.x() as i32;
    let y = target.y() as i32;
    let cursor_x = cursor.x() as i32;
    let cursor_y = cursor.y() as i32;
    // Narrowed first: a spacing between 0 and 1 truncates to zero, and so does
    // NaN, so this is the value the divisions below actually use
    let spacing = spacing as i32;
    assert!(spacing > 0, "marker spacing must be at least one Cell");

    let min_x = ((cursor_x / spacing) * spacing) - 1;
    let max_x = (1 + (cursor_x / spacing)) * spacing;
    let min_y = ((cursor_y / spacing) * spacing) - 1;
    let max_y = (1 + (cursor_y / spacing)) * spacing;

    x > min_x && (x) <= max_x && y > min_y && (y) <= max_y
}

#[cfg(test)]
mod test {

    use super::{in_marker_block, App};
    use crate::{
        glyph::{Glyph, GlyphString},
        grid::Grid,
        opts::DEFAULT_MARKER_SPACING,
        test::trace,
    };

    fn app() -> App {
        let rows = 1; // * (DEFAULT_MARKER_SPACING as usize);
        let cols = 1 * (DEFAULT_MARKER_SPACING as usize);

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
    async fn test_terminator() {
        trace();

        let mut app = app();
        let grid = app.grid;
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        app.select_or_panic(7, 0);

        let g = app.terminator(at(0, 0));
        assert_eq!(g, GlyphString::marker());

        let g = app.terminator(at(1, 0));
        assert_eq!(g, GlyphString::space());

        let g = app.terminator(at(2, 0));
        assert_eq!(g, GlyphString::highlight());
    }

    #[test]
    fn test_in_marker_block() {
        trace();
        let spacing = 8.0;
        // Rectangular, and large enough to mint every position probed below
        let grid = Grid::new(64, 60);
        let at = |x, y| grid.position(x, y).expect("inside the grid");

        let selected = at(5, 5);
        assert!(!in_marker_block(selected, at(10, 10), spacing));
        for x in 0..spacing as usize {
            for y in 0..spacing as usize {
                assert!(in_marker_block(selected, at(x, y), spacing));
            }
        }

        let selected = at(8, 8);
        assert!(!in_marker_block(selected, at(1, 1), spacing));

        for x in 8..=16 as usize {
            for y in 8..=16 as usize {
                assert!(in_marker_block(selected, at(x, y), spacing));
            }
        }

        // Marker block X 5 Y 6
        let selected = at(42, 51);
        assert!(!in_marker_block(selected, at(1, 1), spacing));
        for x in 0..=spacing as usize {
            for y in 0..=spacing as usize {
                let x = x + 40;
                let y = y + 48;
                assert!(in_marker_block(selected, at(x, y), spacing));
            }
        }
    }
}
