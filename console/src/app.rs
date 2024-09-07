use egui::{Event, FontId, Key};
use lang::EXP_LEN;
use lang::{Atom, Atoms, Parser};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;
use std::sync::RwLock;
// use tokio::sync::RwLock;

use std::time::{Duration, Instant};
use tokio::{task, time};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::{
    coord::Coord,
    cursor::Cursor,
    glyph::{to_glyphs, Glyph, Terminator},
    source::Source,
};

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

#[derive(Clone, Debug)]
struct Parsed {
    len: usize,
    inner: Atoms,
}

// pub type ExpressionMap = HashMap<usize, Atoms, nohash_hasher::BuildNoHashHasher<usize>>;
// https://draft.ryhl.io/blog/shared-mutable-state/
#[derive(Clone, Debug)]
struct SharedMap {
    inner: Arc<RwLock<ExpressionMap>>,
}

#[derive(Clone, Debug)]
struct ExpressionMap {
    data: HashMap<usize, Parsed, nohash_hasher::BuildNoHashHasher<usize>>,
}

impl SharedMap {
    pub fn new(capacity: usize) -> Self {
        let map = ExpressionMap {
            data: HashMap::with_capacity_and_hasher(capacity, BuildHasherDefault::default()),
        };

        Self {
            inner: Arc::new(RwLock::new(map)),
        }
    }

    pub fn insert(&self, idx: usize, parsed: Parsed) {
        let mut lock = self.inner.write().unwrap();
        lock.data.insert(idx, parsed);
    }

    pub fn remove(&self, idx: &usize) {
        let mut lock = self.inner.write().unwrap();
        lock.data.remove(idx);
    }

    pub fn get(&self, idx: &usize) -> Option<Parsed> {
        let lock = self.inner.read().unwrap();
        lock.data.get(idx).cloned()
    }

    pub fn len_at(&self, idx: &usize) -> usize {
        let lock = self.inner.read().unwrap();
        lock.data.get(idx).map_or(0, |p| p.len)
    }
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct App {
    pub opts: Opts,
    pub cursor: Cursor,

    exp: SharedMap,
    glyphs: Vec<Glyph>,
    src: Source,
    token: Option<CancellationToken>,
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
        let count = cols * rows;

        let glyphs = vec![Glyph::default(); count];
        let exp = SharedMap::new(count / 6);

        let src = Source::new(cols, rows);

        Self {
            cursor: Cursor::new(cols, rows, opts.cursor_delay),
            src,
            exp,
            glyphs,
            opts,
            token: None,
        }
    }

    ///
    /// writes s to the current cursor position
    /// triggers parse of expression
    ///
    pub fn write(&mut self, s: &String) {
        self.unparse();
        self.src.set_at(self.cursor.coord, s);
        self.parse();
        self.cursor.right();
    }

    #[inline]
    pub fn delete(&mut self) {
        self.unparse();
        self.src.unset_at(self.cursor.coord);
        self.parse();
        self.cursor.left();
    }

    ///   [1] =>
    fn parse(&mut self) {
        if let Some((exp, mut src)) = self.src.get_exp_with_src_at(self.cursor.coord) {
            let mut parsed: lang::Expression = Parser::from(&mut src).parse();
            let start = exp.start();

            let glyphs = to_glyphs(parsed.take_tokens());
            let atoms = parsed.take_atoms();

            self.exp.insert(
                start,
                Parsed {
                    inner: atoms,
                    len: parsed.len(),
                },
            );

            self.set_glyphs(start, glyphs);
        }
    }

    ///
    /// Unsets the expresion glyphs and parsed atom for the expression at cursor.coord
    ///
    fn unparse(&mut self) {
        if let Some(exp) = self.src.get_exp_at(self.cursor.coord) {
            self.unset_glyphs(exp.start());
            self.exp.remove(&exp.start());
        }
    }

    pub fn get_glyph_at(&self, coord: Coord) -> Glyph {
        self.glyphs[coord.index()]
    }

    pub fn get_at(&self, x: usize, y: usize) -> (String, Glyph) {
        let coord = Coord::new(x, y, self.opts.cols, self.opts.rows);

        let mut s = self.src.get_at(coord);
        let mut g = self.get_glyph_at(coord);

        if Glyph::is_terminator(&s) {
            if matches!(g, Glyph::Terminator(_)) {
                g = self.terminator(x, y);
                s = g.to_string()
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
                    if text_to_insert.len() == 1 {
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

    fn set_glyphs(&mut self, start: usize, glyphs: Vec<Glyph>) {
        for (i, g) in glyphs.iter().enumerate() {
            let pos = start + i;
            self.glyphs[pos] = *g;
        }
    }

    fn unset_glyphs(&mut self, start: usize) {
        let end = self.glyphs.len();

        for i in start..end {
            match self.glyphs.get(i) {
                Some(Glyph::Terminator(_)) => break,
                _ => {
                    self.glyphs[i] = Glyph::default();
                }
            };
        }
    }

    fn playing(&self) -> bool {
        self.token.is_some()
    }

    fn pause(&mut self) {
        if let Some(token) = &self.token {
            token.cancel();
        }
    }

    fn stop(&mut self) {
        if let Some(token) = &self.token {
            token.cancel();
        }
    }

    fn play(&mut self) {
        let token = CancellationToken::new();
        let cln_token = token.clone();
        self.token = Some(token);
        info!("play");

        let ms = self.opts.bpm.delay_ms();
        let exp = self.exp.clone();
        task::spawn(async move {
            info!("spawn");
            tokio::select! {
                _ = cln_token.cancelled() => {
                    info!("cancelled");
                }
                _ = Self::ticker(ms, exp) => {
                    info!("done");
                }
            }
        });
    }

    async fn ticker(ms: u64, exp: SharedMap) {
        let mut interval = time::interval(Duration::from_millis(ms));
        loop {
            let exp = exp.get(&0);
            info!("exp {exp:?}");

            interval.tick().await;
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

// pub fn get_glyph_at(&self, x: usize, y: usize) -> Glyph {
//     let idx = self.to_idx(x, y);
//     *self.glyphs.get(idx).unwrap()
// }

// async fn parse_exp(&mut self, exp: &Rc<RefCell<Expression>>) {
//     let mut s = self.get_exp_str(&exp).to_owned();

//     let mut expression = Parser::from(&mut s).parse();

//     let glyphs = to_glyphs(expression.take_tokens());

//     let atoms = expression.take_atoms();

//     let mut exp = exp.borrow_mut();
//     let idx = exp.start;
//     let len = exp.parsed_len;

//     self.unset_glyphs(idx, len);

//     if !glyphs.is_empty() {
//         exp.parsed_len = glyphs.len();

//         {
//             let mut write_lock = self.exp.blocking_write();
//             write_lock.insert(idx, atoms);
//         }

//         for (i, g) in glyphs.iter().enumerate() {
//             let pos = idx + i;
//             self.glyphs[pos] = *g;
//         }
//     }
// }

pub struct Bpm(usize);

impl Bpm {
    fn delay_ms(&self) -> u64 {
        let ms = (60000 / self.0) / 4;
        ms as u64
    }
}

#[cfg(test)]
mod test {

    use crate::{
        coord::Coord,
        glyph::{Glyph, Terminator},
        test::trace,
    };
    use lang::{Atom, Function};
    use tokio::time;
    use tracing::info;

    use super::{App, DEFAULT_GRID_SIZE};

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
            Coord::new(x, y, 100, 100)
        }
    }

    #[test]
    fn test_edit_complex() {
        trace();

        let mut app = app();

        app.set_at(2, 0, "+");
        assert_eq!(app.src.inner, "  +     ");

        // `+` is not a function (yet)
        let glyph = app.get_glyph_at((2, 0).into());
        assert_eq!(glyph, Glyph::default());

        app.set_at(3, 0, "+");
        assert_eq!(app.src.inner, "  ++    ");

        // `++` is a function
        let exp = app.exp.get(&2).unwrap();
        assert_eq!(exp.inner[0], Atom::Function(Function::Add));

        let glyph = app.get_glyph_at((2, 0).into());
        assert_eq!(glyph, Glyph::Function);

        let glyph = app.get_glyph_at((3, 0).into());
        assert_eq!(glyph, Glyph::Function);

        let glyph = app.get_glyph_at((4, 0).into());
        assert_eq!(glyph, Glyph::Number);

        app.set_at(4, 0, "0");
        assert_eq!(app.src.inner, "  ++0   ");

        app.set_at(5, 0, "1");
        assert_eq!(app.src.inner, "  ++01  ");

        app.set_at(6, 0, "0");
        assert_eq!(app.src.inner, "  ++010 ");
        app.set_at(7, 0, "2");
        assert_eq!(app.src.inner, "  ++0102");

        let exp = app.exp.get(&2).unwrap();
        assert_eq!(exp.inner[0], Atom::Function(Function::Add));
        assert_eq!(exp.inner[1], Atom::Number(1));
        assert_eq!(exp.inner[2], Atom::Number(2));

        // Invalidate the function
        app.delete_at(3, 0);
        assert_eq!(app.src.inner, "  + 0102");

        assert!(app.glyphs.iter().all(|g| *g == Glyph::default()));

        // Recreate the function
        app.set_at(3, 0, "+");
        assert_eq!(app.src.inner, "  ++0102");

        // `++` is a function
        let exp = app.exp.get(&2).unwrap();
        assert_eq!(exp.inner[0], Atom::Function(Function::Add));
    }

    #[test]
    fn test_edit_simple() {
        trace();

        let mut app = app();

        app.set_at(0, 0, "i");
        assert!(app.src.inner.starts_with('i'));

        // `i` is not a function yet
        let glyph = app.get_glyph_at((0, 0).into());
        assert_eq!(glyph, Glyph::default());

        // id
        app.set_at(1, 0, "d");
        assert!(app.src.inner.starts_with("id"));

        let glyph = app.get_glyph_at((0, 0).into());
        assert_eq!(glyph, Glyph::Function);

        let glyph = app.get_glyph_at((1, 0).into());
        assert_eq!(glyph, Glyph::Function);

        let glyph = app.get_glyph_at((2, 0).into());
        assert_eq!(glyph, Glyph::String);

        // Delete invalidates expression, and resets glyphs
        app.delete_at(0, 0);
        for x in 0..3 {
            let glyph = app.get_glyph_at((x, 0).into());
            assert_eq!(glyph, Glyph::default());
        }
    }

    #[test]
    fn test_terminator() {
        trace();

        let mut app = app();
        app.cursor.select_at(7, 0);

        let g = app.terminator(0, 0);
        assert_eq!(g, Glyph::Terminator(Terminator::Marker));

        let g = app.terminator(1, 0);
        assert_eq!(g, Glyph::Terminator(Terminator::Space));

        let g = app.terminator(2, 0);
        assert_eq!(g, Glyph::Terminator(Terminator::Dot));
    }
}

/*
    fn source_from(s: &str) -> Source {
        let mut source = source(10, 1);

        let y = 0;

        for (x, c) in s.chars().enumerate() {
            source.set_at(x, y, &c.to_string());

            //
            // I ACTUALLY UNDERSTAND THIS
            //
            // Reference to Option (the Map owns the data)
            //
            let opt_exp: &Option<std::rc::Rc<std::cell::RefCell<Expression>>> = &source.map[0];
            // Get Reference to the RC the Option is wrapping
            // Swap for Option<&RC>
            opt_exp
                .as_ref()
                // and map Option to get the &RC
                .map(|exp: &std::rc::Rc<std::cell::RefCell<Expression>>| {
                    // Now we can borrow the actual Exp we are interested in.
                    let end = exp.borrow().end;
                    assert_eq!(end, x);
                });
        }

        // append '.' to s to fil the source to len 10
        let l = s.len();
        let mut expected = String::from(s);
        expected.push_str(&".".repeat(10 - l));

        assert_eq!(source.inner, expected);
        source
    }
*/
