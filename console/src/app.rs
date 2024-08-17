use std::{
    ops::Deref,
    time::{Duration, Instant},
};

use egui::{Event, FontId, Key};
use tracing::{error, info};

use crate::{executor::Executor, glyph::Glyph, source::Source, Coord};

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
    exe: Executor,
    // Append-only log of commands
    // cmd: Vec<Command>,
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Opts {
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

pub struct Cursor {
    pub on: bool,

    coord: Coord,
    at: Instant,
    delay: u64,
}

impl Cursor {
    fn new(cols: usize, rows: usize, delay: u64) -> Self {
        Self {
            coord: Coord::new(0, 0, cols, rows),
            at: Instant::now(),
            on: false,
            delay,
        }
    }

    pub fn blink(&mut self) {
        if self.at.elapsed() >= Duration::from_millis(self.delay) {
            self.at = Instant::now();
            self.on = !self.on;
        }
    }

    #[inline]
    pub fn select(&mut self, selected: Coord) {
        self.coord = selected;
        self.on = false;
        self.at = Instant::now();
    }

    #[inline]
    pub fn select_at(&mut self, x: usize, y: usize) {
        self.select(self.coord.at(x, y));
    }

    #[inline]
    pub fn up(&mut self) {
        self.select(self.coord.up());
    }

    #[inline]
    pub fn down(&mut self) {
        self.select(self.coord.down());
    }

    #[inline]
    pub fn left(&mut self) {
        self.select(self.coord.left());
    }

    #[inline]
    pub fn right(&mut self) {
        self.select(self.coord.right());
    }
}

impl Deref for Cursor {
    type Target = Coord;
    fn deref(&self) -> &Self::Target {
        &self.coord
    }
}

impl App {
    pub fn new(cols: usize, rows: usize) -> Self {
        let opts = Opts::new(cols, rows);

        Self {
            cursor: Cursor::new(cols, rows, opts.cursor_delay),
            exe: Executor::default(),
            src: Source::new(cols, rows),
            opts,
        }
    }

    #[inline]
    pub fn delete(&mut self) {
        self.src.unset_at(self.cursor.x, self.cursor.y);
        self.cursor.left();
    }

    pub fn get_glyph_at(&self, x: usize, y: usize) -> Glyph {
        self.src.get_glyph_at(x, y)
    }

    pub fn terminator(&self, x: usize, y: usize) -> Glyph {
        // Highlight
        if self.cursor.in_grid(x, y, self.opts.grid_size) {
            if x % self.opts.grid_selected_dot_spacing == 0
                && y % self.opts.grid_selected_dot_spacing == 0
            {
                return Glyph::highlight();
            }
        }

        // Grid markers
        if x as f32 % self.opts.grid_size == 0.0 && y as f32 % self.opts.grid_size == 0.0 {
            return Glyph::marker();
        }

        Glyph::default()
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
                    self.src
                        .set_at(self.cursor.x, self.cursor.y, text_to_insert);

                    // if self.opts.mode == Mode::Insert {
                    self.cursor.right();
                    repaint = true;
                    // }
                }

                _ => {
                    // info!("Pressed");
                }
            }
        }
        repaint
    }
}

#[cfg(test)]
mod test {

    use tracing::info;

    use crate::test::trace;

    use super::App;

    #[test]
    fn test_highlight() {
        trace();

        let mut app = App::new(10, 10);

        // app.set_at(0, 0, "i");
        // app.set_at(1, 0, "d");

        // // info!(":{:?}", app.src.inner);

        // // let (s, g) = app.render(1, 0);
        // // info!("{}:{:?}", s, g);

        // // let (s, g) = app.render(2, 0);
        // // info!("{}:{:?}", s, g);
        // assert_eq!(&s, "s");
        // assert_eq!(g, Glyph::String);
    }
}
