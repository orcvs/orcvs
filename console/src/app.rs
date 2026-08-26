use egui::{Event, Key};

use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::{task, time};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::glyph::GlyphString;
use crate::opts::Opts;

use crate::source::{Command, SourceCommander};
use crate::{coord::Coord, cursor::Cursor};

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct App {
    pub opts: Opts,
    pub cursor: Cursor,

    token: Option<CancellationToken>,
    source: SourceCommander,
}

impl App {
    pub fn new(cols: usize, rows: usize) -> Self {
        assert!(cols > 0, "cols must be greater than zero");
        assert!(rows > 0, "rows must be greater than zero");

        let opts = Opts::new(cols, rows);

        let source = SourceCommander::spawn(opts.clone());

        Self {
            cursor: Cursor::new(cols, rows, opts.cursor_delay),
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
            Ok(_) => self.cursor.right(),
            Err(e) => error!("rejected edit: {e}"),
        }
    }

    fn delete(&mut self) {
        let idx = self.cursor_index();

        match self.source.unset(idx) {
            Ok(_) => self.cursor.left(),
            Err(e) => error!("rejected delete: {e}"),
        }
    }

    pub fn cursor_index(&self) -> usize {
        self.index(self.cursor.coord)
    }

    ///
    /// Convert x, y coordinates to a linear index
    /// panic if the index is out of bounds
    ///
    pub fn index(&self, coord: Coord) -> usize {
        let idx = coord.y * self.opts.cols + coord.x;
        assert!(
            idx <= self.opts.cols * self.opts.rows,
            "index {idx} out of bounds for [{},{}]",
            coord.x,
            coord.y,
        );
        idx
    }

    pub fn get(&self, x: usize, y: usize) -> GlyphString {
        let idx = self.index(Coord::new(x, y));

        let (s, g) = self.source.get(idx);
        match g {
            Some(g) => GlyphString::new(s, g),
            None => self.terminator(x, y),
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
                } => self.cursor.down(),
                Event::Key {
                    key: Key::ArrowLeft,
                    pressed: true,
                    ..
                } => self.cursor.left(),
                Event::Key {
                    key: Key::ArrowRight,
                    pressed: true,
                    ..
                } => self.cursor.right(),
                Event::Key {
                    key: Key::ArrowUp,
                    pressed: true,
                    ..
                } => self.cursor.up(),
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

    fn terminator(&self, x: usize, y: usize) -> GlyphString {
        // Grid markers
        if x as f32 % self.opts.grid_size == 0.0 && y as f32 % self.opts.grid_size == 0.0 {
            return GlyphString::marker();
        }

        // Highlight
        if self.cursor.coord.in_grid(x, y, self.opts.grid_size) {
            if x % self.opts.grid_selected_dot_spacing == 0
                && y % self.opts.grid_selected_dot_spacing == 0
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

#[cfg(test)]
mod test {

    use super::App;
    use crate::{
        coord::Coord,
        glyph::{Glyph, GlyphString},
        opts::DEFAULT_GRID_SIZE,
        test::trace,
    };

    fn app() -> App {
        let rows = 1; // * (DEFAULT_GRID_SIZE as usize);
        let cols = 1 * (DEFAULT_GRID_SIZE as usize);

        App::new(cols, rows)
    }

    impl App {
        pub fn src(&mut self, src: &str) {
            for (i, c) in src.chars().enumerate() {
                self.set_at(i, 0, &c.to_string())
            }
        }

        pub fn delete_at(&mut self, x: usize, y: usize) {
            self.cursor.select_at(x, y);
            self.delete()
        }

        pub fn set_at(&mut self, x: usize, y: usize, s: &str) {
            self.cursor.select_at(x, y);
            self.write(&s.to_owned());
        }
    }

    impl From<(usize, usize)> for Coord {
        fn from((x, y): (usize, usize)) -> Self {
            Coord::new(x, y)
        }
    }

    #[tokio::test]
    async fn test_to_idx() {
        trace();
        let app = App::new(10, 10);

        let coord = Coord::new(0, 0);
        let idx = app.index(coord);
        assert_eq!(idx, 0);

        let coord = Coord::new(5, 5);
        let idx = app.index(coord);
        assert_eq!(idx, 55);
    }

    #[tokio::test]
    #[should_panic(expected = "index 121 out of bounds for [11,11]")]
    async fn test_to_idx_out_of_bounds() {
        trace();
        let app = App::new(10, 10);

        let coord = Coord::new(11, 11);

        let _ = app.index(coord);
    }

    #[tokio::test]
    async fn test_write_renders_cell_and_glyph_immediately() {
        trace();

        let mut app = app();

        app.set_at(0, 0, "+");
        app.set_at(1, 0, "+");

        // the accepted edits are observable as soon as write returns
        let expected = GlyphString::new(Some("+".to_string()), Glyph::Function);
        assert_eq!(app.get(0, 0), expected);
        assert_eq!(app.get(1, 0), expected);
    }

    #[tokio::test]
    async fn test_write_moves_cursor_right() {
        trace();

        let mut app = app();
        app.cursor.select_at(0, 0);

        app.write(&"+".to_string());

        assert_eq!(app.cursor.coord, Coord::new(1, 0));
    }

    #[tokio::test]
    async fn test_delete_clears_cell_and_moves_cursor_left() {
        trace();

        let mut app = app();
        app.set_at(0, 0, "+");
        app.set_at(1, 0, "+");

        app.delete_at(1, 0);

        assert_eq!(app.cursor.coord, Coord::new(0, 0));
        assert_eq!(app.get(1, 0), GlyphString::space());
    }

    #[tokio::test]
    async fn test_terminator() {
        trace();

        let mut app = app();
        app.cursor.select_at(7, 0);

        let g = app.terminator(0, 0);
        assert_eq!(g, GlyphString::marker());

        let g = app.terminator(1, 0);
        assert_eq!(g, GlyphString::space());

        let g = app.terminator(2, 0);
        assert_eq!(g, GlyphString::highlight());
    }
}
