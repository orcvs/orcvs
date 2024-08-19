use std::{
    ops::Deref,
    time::{Duration, Instant},
};

use egui::{Event, FontId, Key};
use tokio::{task, time};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::{cursor::Cursor, glyph::Glyph, source::Source};

pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_GRID_SIZE: f32 = 8.0;

pub const DEFAULT_COL_COUNT: usize = 2 * (DEFAULT_GRID_SIZE as usize);
pub const DEFAULT_ROW_COUNT: usize = 2 * (DEFAULT_GRID_SIZE as usize);

pub const DEFAULT_GRID_SELECTED_DOT_SPACING: usize = 2;

pub const DEFAULT_CURSOR_DELAY: u64 = 800;

enum Command {
    Set(usize, usize, String),
    Unset(usize, usize),
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct App {
    pub opts: Opts,
    pub cursor: Cursor,

    src: Source,
    token: Option<CancellationToken>,
    // Append-only log of commands
    // cmd: Vec<Command>,
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Opts {
    pub bpm: Bpm,
    pub cols: usize,
    pub cursor_delay: u64,
    pub font_id: FontId,
    pub grid_selected_dot_spacing: usize,
    pub grid_size: f32,
    pub mode: Mode,
    pub rows: usize,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum Mode {
    Insert,
    Command,
}

impl Opts {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            bpm: Bpm(120),
            cols,
            cursor_delay: DEFAULT_CURSOR_DELAY,
            font_id: egui::FontId::monospace(DEFAULT_FONT_SIZE),
            grid_selected_dot_spacing: DEFAULT_GRID_SELECTED_DOT_SPACING,
            grid_size: DEFAULT_GRID_SIZE,
            mode: Mode::Insert,
            rows,
        }
    }
}

impl App {
    pub fn new(cols: usize, rows: usize) -> Self {
        let opts = Opts::new(cols, rows);

        Self {
            cursor: Cursor::new(cols, rows, opts.cursor_delay),
            src: Source::new(cols, rows),
            opts,
            token: None,
        }
    }

    #[inline]
    pub fn delete(&mut self) {
        self.src.unset_at(self.cursor.coord.x, self.cursor.coord.y);
        self.cursor.left();
    }

    pub fn get_glyph_at(&self, x: usize, y: usize) -> Glyph {
        self.src.get_glyph_at(x, y)
    }

    pub fn get_at(&self, x: usize, y: usize) -> (String, Glyph) {
        let mut s = self.src.get_at(x, y);
        let mut g = self.get_glyph_at(x, y);

        if Glyph::is_terminator(&s) {
            if matches!(g, Glyph::Terminator(_)) {
                g = self.terminator(x, y);
                s = g.into()
            }
        }

        (s, g)
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

                Event::Text(text_to_insert) => {
                    if text_to_insert.len() == 1 {
                        self.src
                            .set_at(self.cursor.coord.x, self.cursor.coord.y, text_to_insert);

                        // if self.opts.mode == Mode::Insert {
                        self.cursor.right();
                        repaint = true;
                    }

                    // }
                }

                _ => {
                    // info!("Pressed");
                }
            }
        }
        repaint
    }

    async fn play(&mut self) {
        let token = CancellationToken::new();
        let cln_token = token.clone();
        self.token = Some(token);

        let ms = self.opts.bpm.delay_ms();

        // info!("{ms}");
        // let state = self.state;
        task::spawn(async move {
            info!("spawn");
            tokio::select! {
                _ = cln_token.cancelled() => {
                    info!("cancelled");
                }
                _ = Self::ticker(ms) => {
                    info!("done");
                }
            }
        });
        info!("here");
    }

    async fn ticker(ms: u64) {
        let mut interval = time::interval(Duration::from_millis(ms));
        loop {
            info!("interval");
            interval.tick().await;
            // tick().await;
        }
    }

    fn terminator(&self, x: usize, y: usize) -> Glyph {
        // Grid markers
        if x as f32 % self.opts.grid_size == 0.0 && y as f32 % self.opts.grid_size == 0.0 {
            return Glyph::marker();
        }

        // Highlight
        if self.cursor.coord.in_grid(x, y, self.opts.grid_size) {
            if x % self.opts.grid_selected_dot_spacing == 0
                && y % self.opts.grid_selected_dot_spacing == 0
            {
                return Glyph::highlight();
            }
        }

        Glyph::default()
    }
}

struct Bpm(usize);

impl Bpm {
    fn delay_ms(&self) -> u64 {
        let ms = (60000 / self.0) / 4;
        ms as u64
    }
}

#[cfg(test)]
mod test {

    use crate::{
        glyph::{Glyph, Terminator},
        test::trace,
    };

    use super::{App, DEFAULT_GRID_SIZE};

    #[test]
    fn test_terminator() {
        trace();

        let rows = 3 * (DEFAULT_GRID_SIZE as usize);
        let cols = 3 * (DEFAULT_GRID_SIZE as usize);

        let mut app = App::new(cols, rows);
        app.cursor.select_at(3, 3);

        let g = app.terminator(0, 0);
        assert_eq!(g, Glyph::Terminator(Terminator::Marker));
    }
}
