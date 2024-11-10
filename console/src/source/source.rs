use lang::{Atoms, Expression, Parser};
use std::cell::{Ref, RefCell};
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task;
use tracing::{debug, error, info};

use crate::coord::Coord;
use crate::glyph::{to_glyphs, Glyph};

use super::{Command, ExpressionMap};

// use arrayvec::ArrayVec;

pub const TERMINATOR: &str = " ";
pub const TERMINATOR_BYTES: &[u8] = TERMINATOR.as_bytes();

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Source {
    inner: String,
    map: ExpressionMap,
    glyphs: Vec<Glyph>,
    parsed: Vec<Option<Atoms>>,
    sender: Sender<Command>,
}

impl Source {
    pub fn new(size: usize, sender: Sender<Command>) -> Self {
        let inner = TERMINATOR.to_string().repeat(size);
        let map = ExpressionMap::new(size);

        let glyphs = vec![Glyph::default(); size];
        let parsed = vec![None; size];

        Self {
            inner,
            map,
            glyphs,
            parsed,
            sender,
        }
    }

    ///
    /// Sets `s` at the x, y coords of the grid and recalculates expressions.
    ///
    /// ```
    /// use console::source::Source;
    /// let mut source = source(10, 10);
    /// let idx = source.set_at(Coord::ne, 10, 1), "!");
    ///
    /// let s = source.get_at(Coord::new(3, 3, 10, 10));
    /// assert_eq!(s.as_str(), "!");
    /// ```
    ///
    pub fn set_at(&mut self, idx: usize, s: &str) {
        self.unparse(idx);
        self.set_at_inner(idx, s);

        let glyph = Glyph::is_glyph(s);
        if glyph {
            self.map.set(idx);
        } else {
            self.map.unset(idx);
        }

        self.parse(idx);
    }

    pub fn get_at(&self, idx: usize) -> String {
        // info!("idx: {idx}");

        // SAFELY UNSAFE
        // all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds
        let s = unsafe { self.inner.get_unchecked(idx..(idx + 1)) };
        s.to_owned()
    }

    ///
    /// Unset at {x, y}
    /// Sets a `TERMINATOR` at the position`
    ///
    // pub fn unset_at(&mut self, idx: usize) {
    //     self.set_at(idx, TERMINATOR);
    // }

    // pub fn get_exp_with_src_at(&self, idx: usize) -> Option<(ExpressionIndex, String)> {
    //     if let Some(exp) = &self.map[idx] {
    //         let exp = exp.borrow();
    //         let src = self.get_exp_src(exp.start(), exp.end());
    //         Some((*exp, src))
    //     } else {
    //         None
    //     }
    // }

    #[inline]
    fn get_exp_src(&self, from: usize, to: usize) -> String {
        // SAFELY UNSAFE
        // all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds
        let s = unsafe { self.inner.get_unchecked(from..(to + 1)) };
        s.to_owned()
    }

    ///
    /// Sets `s` at the x, y coords of the grid without recalculating expressions.
    /// Returns the index position of the updated character
    ///
    /// ```
    /// use console::source::Source;
    /// let mut source = source(10, 10);
    /// let idx = source.set_at_uncalculated(3, 3, "!");
    ///
    /// assert_eq!(idx, 33);
    /// ```
    ///
    fn set_at_inner(&mut self, idx: usize, s: &str) -> usize {
        let b = s.as_bytes();

        // SAFELY UNSAFE
        //   all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            bytes[idx] = b[0];
        }

        idx
    }

    ///
    /// Unsets the expresion glyphs and parsed atom for the expression at cursor.coord
    ///
    fn unparse(&mut self, idx: usize) {
        if let Some(exp) = self.map.get(idx) {
            //     let start = exp.start();

            //     self.unset_glyphs(start);
            //     self.parsed[start] = None;
        }
    }

    fn parse(&mut self, idx: usize) {
        // if let Some((exp, mut src)) = self.get_exp_with_src_at(idx) {
        //     let mut parsed: Expression = Parser::from(&mut src).parse();
        //     let start = exp.start();

        //     let glyphs = to_glyphs(parsed.take_tokens());
        //     let atoms = parsed.take_atoms();

        //     self.parsed[start] = Some(atoms);

        //     self.set_glyphs(start, glyphs);
        // }
    }

    pub fn get_glyph_at(&self, idx: usize) -> Glyph {
        self.glyphs[idx]
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

    pub fn get_sender(&self) -> Sender<Command> {
        self.sender.clone()
    }

    // pub fn inner(&self) -> String {
    //     self.inner.to_owned()
    // }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod test {

    // fn source(cols: usize, rows: usize) -> Source {
    //     Source::new(cols, rows)
    // }
    // ===========================================================================

    // #[test]
    // fn test_get_expression_as_str() {
    //     trace();

    //     let cols = 10;
    //     let mut source = source(10, 1);

    //     source.set_at(Coord::new(0, 0).index(cols), "i");
    //     source.set_at(Coord::new(1, 0).index(cols), "d");
    //     source.set_at(Coord::new(2, 0).index(cols), "0");
    //     source.set_at(Coord::new(3, 0).index(cols), "A");

    //     assert_eq!(source.inner, "id0A      ");

    //     source.print_exp();

    //     let exp = source
    //         .map
    //         .iter()
    //         .find_map(|o| o.as_ref().map(|e| e))
    //         .unwrap();

    //     source.print_exp();
    //     let exp = exp.borrow();
    //     let s = source.get_exp_src(exp.start, exp.end);

    //     assert_eq!(s, "id0A");
    // }

    // #[test]
    // fn test_map_expression_at_max_idx() {
    //     trace();

    //     let cols = 10;
    //     let mut source = source(10, 1);

    //     source.set_at(Coord::new(6, 0).index(cols), "I");
    //     source.set_at(Coord::new(7, 0).index(cols), "D");
    //     source.set_at(Coord::new(8, 0).index(cols), "A");
    //     source.set_at(Coord::new(9, 0).index(cols), "A");
    //     assert_eq!(source.inner, "      IDAA");
    // }

    // #[test]
    // fn test_map_expression() {
    //     trace();

    //     let cols = 10;
    //     let mut source = source(10, 1);

    //     source.set_at(Coord::new(0, 0).index(cols), "I");

    //     assert_eq!(source.inner, "I         ");

    //     source.set_at(Coord::new(1, 0).index(cols), "D");

    //     assert_eq!(source.inner, "ID        ");
    //     assert_eq!(source.map[0], source.map[1]);

    //     let ptr_0 = source.map[0].as_ref().unwrap().as_ref();
    //     let ptr_1 = source.map[1].as_ref().unwrap().as_ref();

    //     assert!(std::ptr::eq(ptr_0, ptr_1));

    //     source.set_at(Coord::new(2, 0).index(cols), "0");
    //     source.set_at(Coord::new(3, 0).index(cols), "1");

    //     assert_eq!(source.map[0], source.map[1]);
    //     assert_eq!(source.map[0], source.map[2]);
    //     assert_eq!(source.map[0], source.map[3]);
    // }

    // #[test]
    // fn test_set_at() {
    //     trace();

    //     let cols = 10;
    //     let mut source = source(10, 1);

    //     source.set_at(Coord::new(0, 0).index(cols), "T");

    //     assert_eq!(source.inner, "T         ");

    //     source.set_at(Coord::new(7, 0).index(cols), "X");

    //     assert_eq!(source.inner, "T      X  ");
    // }

    // #[test]
    // fn test_get_at() {
    //     trace();

    //     let source = Source::from_source(10, 1, "T......X..");

    //     let s = source.get_at(0);

    //     assert_eq!(s, "T");

    //     let s = source.get_at(7);

    //     assert_eq!(s, "X");
    // }

    // #[test]
    // fn test_expression_len() {
    //     let exp = ExpressionMap { start: 0, end: 0 };
    //     assert_eq!(exp.len(), 1);

    //     let exp = ExpressionMap { start: 0, end: 1 };
    //     assert_eq!(exp.len(), 2);
    // }

    // #[test]
    // #[should_panic(expected = "source length 10, expected 100")]
    // fn test_source_() {
    //     let _source = Source::from_source(10, 10, "..........");
    // }

    // impl Source {
    //     fn from_source(cols: usize, rows: usize, s: impl Into<String>) -> Self {
    //         let n = cols * rows;
    //         let inner = s.into();
    //         let len = inner.len();

    //         assert!(len == n, "source length {len}, expected {n}");

    //         let mut source = source(cols, rows);

    //         // Iterate through inner and call set_at
    //         for (idx, &byte) in inner.as_bytes().iter().enumerate() {
    //             let x = idx % cols;
    //             let y = idx / cols;
    //             source.set_at(Coord::new(x, y).index(cols), &(byte as char).to_string());
    //         }

    //         source
    //     }

    //     fn print_exp(&self) {
    //         self.map.iter().for_each(|m| {
    //             info!("map: {:?}", m);
    //         });
    //     }

    //     fn map_len(&self) -> usize {
    //         self.map
    //             .iter()
    //             .filter(|e| e.is_some())
    //             .collect::<Vec<_>>()
    //             .len()
    //     }
    // }
}
