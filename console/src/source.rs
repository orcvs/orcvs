use arrayvec::ArrayVec;
use lang::{Atom, Expression, Function, Parsed, Parser, Token};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::io::repeat;
use std::iter;
use std::rc::Rc;
use tracing::{debug, error, info};
// use arrayvec::ArrayVec;

pub const TERMINATOR: &str = " ";
pub const TERMINATOR_BYTES: &[u8] = TERMINATOR.as_bytes();

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Source {
    pub inner: String,
    pub parsed: HashMap<usize, Parsed<Expression>, nohash_hasher::BuildNoHashHasher<usize>>,
    pub glyphs: Vec<Glyph>,
    map: Vec<Option<Rc<RefCell<SourceExpression>>>>,
    cols: usize,
    rows: usize,
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct SourceExpression {
    start: usize,
    end: usize,

    // #[serde(skip_deserializing, skip_serializing)]
    valid: Option<bool>,
    function: Option<Function>,
}

impl SourceExpression {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            valid: None,
            function: None,
        }
    }
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Glyph {
    Function,
    Number,
    Note,
    String,
    Terminator(Terminator),
}
pub type G = Glyph;

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Terminator {
    Dot,
    Marker,
    Space,
}

impl Glyph {
    pub fn highlight() -> Self {
        Glyph::Terminator(Terminator::Dot)
    }
    pub fn marker() -> Self {
        Glyph::Terminator(Terminator::Marker)
    }
}

impl Default for Glyph {
    fn default() -> Self {
        Glyph::Terminator(Terminator::default())
    }
}

impl Default for Terminator {
    fn default() -> Self {
        Terminator::Space
    }
}

impl From<Glyph> for String {
    fn from(g: Glyph) -> Self {
        match g {
            Glyph::Function => "F".to_string(),
            Glyph::Number => "h".to_string(),
            Glyph::Note => "n".to_string(),
            Glyph::String => "s".to_string(),
            Glyph::Terminator(t) => t.into(),
        }
    }
}

impl From<Terminator> for String {
    fn from(t: Terminator) -> Self {
        match t {
            Terminator::Dot => ".".to_string(),
            Terminator::Marker => "+".to_string(),
            Terminator::Space => " ".to_string(),
        }
    }
}

fn to_glyphs(parsed_exps: &Parsed<Expression>) -> Vec<Glyph> {
    parsed_exps
        .0
        .iter()
        .filter_map(|exp| match exp.atom {
            Some(Atom::Empty) => None,
            Some(Atom::Function(Function::Empty)) => None,
            _ => Some(exp.token),
        })
        .flat_map(|t| {
            let (g, n) = match t {
                Token::Function => (G::Function, 2),
                Token::Note => (G::Note, 2),
                Token::Number => (G::Number, 2),
                Token::Number1 => (G::Number, 1),
                Token::String => (G::String, 2),
            };
            iter::repeat(g).take(n)
        })
        .collect()
}

impl Source {
    pub fn new(cols: usize, rows: usize) -> Self {
        let n = cols * rows;
        let inner = TERMINATOR.to_string().repeat(n);

        let map = vec![None; n];
        let glyphs = vec![Glyph::default(); n];

        let parsed = HashMap::with_capacity_and_hasher(n, BuildHasherDefault::default());

        Self {
            inner,
            glyphs,
            parsed,
            map,
            cols,
            rows,
        }
    }

    pub fn get_at(&self, x: usize, y: usize) -> String {
        let idx = self.to_idx(x, y);
        // SAFELY UNSAFE
        // all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds
        let s = unsafe { self.inner.get_unchecked(idx..(idx + 1)) };
        s.to_owned()
    }

    #[inline(always)]
    pub fn get_exp_str(&self, exp: &Rc<RefCell<SourceExpression>>) -> &str {
        let exp = exp.borrow();
        let start = exp.start;
        let end = exp.end;

        assert!(
            end <= self.len(),
            "expression out of bounds [start:{start}, end:{end}]"
        );

        // SAFELY UNSAFE
        // all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds
        unsafe { self.inner.get_unchecked(start..(end + 1)) }
    }

    pub fn set_exp(&mut self, idx: usize, exp: Rc<RefCell<SourceExpression>>) {
        self.parse_exp(&exp);
        self.map[idx] = Some(exp);
    }

    pub fn remove_exp(&mut self, idx: usize) {
        // self.glyphs.remove(&idx);
        self.map[idx] = None;
    }

    pub fn get_glyph_at(&self, x: usize, y: usize) -> Glyph {
        let idx = self.to_idx(x, y);
        let s = self.get_at(x, y);

        // info!("{:?}", self.glyphs);
        // if let Some(exp) = &self.map[idx] {
        //     let exp = exp.borrow();
        //     let key = exp.start;

        //     if let Some(parsed) = self.glyphs.get(&key) {
        //         info!("{:?}", parsed);
        //         let pos = idx - key;
        //         let g = *parsed.get(pos).unwrap_or(&Glyph::default());
        //         info!("{:?}", g);
        //         return g;
        //     }
        // }

        Glyph::default()
    }

    pub fn parse_exp(&mut self, exp: &Rc<RefCell<SourceExpression>>) {
        let mut s = self.get_exp_str(&exp).to_owned();

        let parsed = Parser::from(&mut s).parse().take();

        let glyphs = to_glyphs(&parsed);

        let exp = exp.borrow();
        let idx = exp.start;

        debug!("parsed {parsed:?}");
        debug!("glyphs {glyphs:?}");

        self.parsed.insert(idx, parsed);

        for (i, g) in glyphs.iter().enumerate() {
            let pos = idx + i;
            self.glyphs[pos] = *g;
        }
    }

    ///
    /// Sets `s` at the x, y coords of the grid and recalculates expressions.
    ///
    /// ```
    /// use console::source::Source;
    /// let mut source = Source::new(10, 10);
    /// let idx = source.set_at(3, 3, "!");
    ///
    /// let s = source.get_at(3, 3);
    /// assert_eq!(s.as_str(), "!");
    /// ```
    ///
    pub fn set_at(&mut self, x: usize, y: usize, s: &str) {
        let idx = self.set_at_uncalculated(x, y, s);

        // info!("{}", self.inner);

        self.calculate_at(idx, s);
    }

    ///
    /// Sets `s` at the x, y coords of the grid without recalculating expressions.
    /// Returns the index position of the updated character
    ///
    /// ```
    /// use console::source::Source;
    /// let mut source = Source::new(10, 10);
    /// let idx = source.set_at_uncalculated(3, 3, "!");
    ///
    /// assert_eq!(idx, 33);
    /// ```
    ///
    pub fn set_at_uncalculated(&mut self, x: usize, y: usize, s: &str) -> usize {
        let idx = self.to_idx(x, y);

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
    /// Alias of `set_at_uncalculated``
    ///
    ///
    pub fn set_terminator_at(&mut self, x: usize, y: usize, s: &str) {
        self.set_at_uncalculated(x, y, s);
    }

    pub fn unset_at(&mut self, x: usize, y: usize) {
        let idx = self.to_idx(x, y);

        // SAFELY UNSAFE
        //   all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds

        unsafe {
            let bytes = self.inner.as_bytes_mut();
            bytes[idx] = TERMINATOR_BYTES[0];
        };

        self.calculate_at(idx, TERMINATOR);

        // let recalculate = unsafe {
        //     let bytes = self.inner.as_bytes_mut();
        //     let b = bytes[idx];

        //     info!("{b}");
        //     bytes[idx] = s_bytes[0];

        //     is_terminator_bytes(b)
        // };

        // if recalculate {
        //     self.calculate_at(idx, s);
        // }
    }

    pub fn calculate_at(&mut self, idx: usize, s: &str) {
        let (lft_idx, lft_exp) = if idx > 0 {
            let idx = idx - 1;
            let exp = self.map[idx].clone();
            (idx, exp)
        } else {
            // Index 0 has no left expression
            (0, None)
        };

        let (rgt_idx, rgt_exp) = if idx < self.len() - 1 {
            let idx = idx + 1;
            let exp = self.map[idx].clone();
            (idx, exp)
        } else {
            // Last index  has no right expression
            (self.len(), None)
        };

        let terminator = is_terminator(s);
        let glyph = !terminator;

        /*
            // ... => .I.
            // Create a new expression if the lhs and rhs are None
            //   exp.start = idx
            //   exp.end   = idx

            // .I. => .ID.
            // Append to the lhs expression if Some(lft_exp) and None(rhs)
            //   lhs.end = idx

            // .IDAA. => .ID0A.
            // Replace if Some(lft) and Some(rhs)
            //   // noop

            // .IDAA. => .ID.A.
            // Split if Terminator and Some(lft) and Some(rhs)
            //

            // .ID.A. => .IDAA.
            // Join if None(idx) and Some(lft_exp) and Some(rhs)
            //  lft.end = rgt.end
            //  rgt = lft
            //  self.map[idx] = lft

            // ..DAA. => .IDAA.
            // Prepend if None(lhs) and Some(rhs)
            //  exp.start = idx
        */

        // info!("{:?}", lft_exp);
        // info!("{:?}", rgt_exp);

        match (lft_exp, rgt_exp) {
            (Some(lft_exp), Some(ref mut rgt_exp)) => {
                // Split the expression
                if terminator {
                    // Remove current expression from idx
                    self.remove_exp(idx);

                    // In some cases lft and rgt may refer to the same expression
                    // We cannot have multiple mutable borrows in the same scope
                    // So we split the borrows into separate scopes

                    // Right must be first as we want to preserve the end idx
                    // The left expression will be modified
                    {
                        let rgt = rgt_exp.borrow();
                        let exp = Rc::new(RefCell::new(SourceExpression::new(rgt_idx, rgt.end)));
                        // self.map[rgt_idx] = Some(exp);
                        self.set_exp(rgt_idx, exp);
                    }
                    {
                        let mut exp = lft_exp.borrow_mut();
                        exp.end = lft_idx;
                    }
                };

                // Join or Replace the expression
                // Replace is a noop
                if glyph {
                    let idx_exp = &self.map[idx];

                    // Join the lft and rgt expressions
                    if idx_exp.is_none() {
                        // In some cases lft and rgt may refer to the same expression
                        // We cannot have multiple mutable borrows in the same scope
                        // So we split the borrows into separate scopes

                        // Right must be first as we want to preserve the end idx
                        // The left expression will be modified
                        let rgt = rgt_exp.borrow();
                        let end = rgt.end;
                        {
                            let mut exp = lft_exp.borrow_mut();
                            exp.end = end;
                        }

                        // Iterate the map until the rgt end and set the lft expression
                        for i in idx..(end + 1) {
                            self.set_exp(i, lft_exp.clone());
                        }
                    }
                }
            }
            (Some(lft_exp), None) => {
                {
                    let mut exp = lft_exp.borrow_mut();
                    exp.end = idx;
                }
                // Append to the expression
                self.set_exp(idx, lft_exp);
            }
            (None, Some(rgt_exp)) => {
                // Prepend to the expression
                {
                    let mut exp = rgt_exp.borrow_mut();
                    exp.start = idx;
                }
                self.set_exp(idx, rgt_exp);
            }
            (None, None) => {
                if terminator {
                    // Remove expression
                    self.remove_exp(idx);
                }

                if glyph {
                    // New Expression
                    let exp = Rc::new(RefCell::new(SourceExpression::new(idx, idx)));
                    self.set_exp(idx, exp);
                }
            }
        }
    }

    ///
    /// Convert x, y coordinates to a linear index
    /// panic if the index is out of bounds
    ///
    #[inline]
    pub fn to_idx(&self, x: usize, y: usize) -> usize {
        let idx = y * self.cols + x;
        assert!(idx <= self.len(), "index {idx} out of bounds for [{x},{y}]");
        idx
    }

    ///
    /// Current length of the source
    ///
    fn len(&self) -> usize {
        self.cols * self.rows
    }

    #[cfg(test)]
    fn from_source(cols: usize, rows: usize, source: impl Into<String>) -> Self {
        let n = cols * rows;
        let inner = source.into();
        let len = inner.len();

        assert!(len == n, "source length {len}, expected {n}");

        let mut source = Self::new(cols, rows);

        // Iterate through inner and call set_at
        for (idx, &byte) in inner.as_bytes().iter().enumerate() {
            let x = idx % cols;
            let y = idx / cols;
            source.set_at(x, y, &(byte as char).to_string());
        }

        source
    }

    #[cfg(test)]
    fn print_exp(&self) {
        self.map.iter().for_each(|m| {
            info!("map: {:?}", m);
        });
        self.glyphs.iter().for_each(|m| {
            info!("parsed: {:?}", m);
        });
    }
}

#[inline]
pub fn is_terminator(s: &str) -> bool {
    match s {
        "." => true,
        " " => true,
        // "+" => true,
        _ => false,
    }
}

#[inline]
pub fn is_terminator_bytes(b: u8) -> bool {
    match b {
        46 => true,
        32 => true,
        // 43 => true,
        _ => false,
    }
}

#[inline]
pub fn is_character(s: &str) -> bool {
    !is_terminator(s)
}

#[cfg(test)]
mod test {
    use crate::source::is_terminator;
    use crate::source::is_terminator_bytes;
    use crate::source::Glyph;
    use crate::source::Source;
    use crate::source::SourceExpression;
    use crate::source::Terminator;
    use crate::test::trace;
    use std::fmt::Debug;
    use tracing::{debug, info};

    fn source_from(s: &str) -> Source {
        let mut source = Source::new(10, 1);

        let y = 0;

        for (x, c) in s.chars().enumerate() {
            source.set_at(x, y, &c.to_string());

            //
            // I ACTUALLY UNDERSTAND THIS
            //
            // Reference to Option (the Map owns the data)
            //
            let opt_exp: &Option<std::rc::Rc<std::cell::RefCell<SourceExpression>>> =
                &source.map[0];
            // Get Reference to the RC the Option is wrapping
            // Swap for Option<&RC>
            opt_exp
                .as_ref()
                // and map Option to get the &RC
                .map(|exp: &std::rc::Rc<std::cell::RefCell<SourceExpression>>| {
                    // Now we can borrow the actual Exp we are interested in.
                    let end = exp.borrow().end;
                    assert_eq!(end, x);
                });
        }

        // append '.' to s to fil the source to len 10
        let l = s.len();
        let mut expected = String::from(s);
        expected.push_str(&".".repeat(10 - l));

        debug!("source.inner {:?}", source.inner);
        assert_eq!(source.inner, expected);
        source
    }

    #[test]
    fn test_get_glyph_at() {
        trace();

        let source = source_from("  ++0101  ");

        let glyph = source.get_glyph_at(0, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));

        let glyph = source.get_glyph_at(2, 0);
        assert_eq!(glyph, Glyph::Function);

        let glyph = source.get_glyph_at(3, 0);
        assert_eq!(glyph, Glyph::Function);

        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Number);

        let glyph = source.get_glyph_at(9, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));
    }

    #[test]
    fn test_is_terminator() {
        let t = is_terminator(".");
        assert!(t);

        let t = is_terminator(" ");
        assert!(t);

        let t = is_terminator("+");
        assert!(t == false);

        let t = is_terminator("..");
        assert!(t == false);

        let t = is_terminator("!");
        assert!(t == false);
    }

    #[test]
    fn test_is_terminator_bytes() {
        let b = ".".as_bytes();
        let t = is_terminator_bytes(b[0]);
        assert!(t);

        let b = " ".as_bytes();
        let t = is_terminator_bytes(b[0]);
        assert!(t);

        let b = "+".as_bytes();
        let t = is_terminator_bytes(b[0]);
        assert!(t == false);

        let b = "!".as_bytes();
        let t = is_terminator_bytes(b[0]);
        assert!(t == false);
    }

    #[test]
    fn test_expression_delete_last() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(5, 0, "C");
        source.set_at(7, 0, "B");
        source.set_at(9, 0, "A");

        assert_eq!(source.inner, "     C B A");

        assert_eq!(source.glyphs.len(), 3);

        source.set_at(5, 0, ".");
        source.set_at(7, 0, ".");
        source.set_at(9, 0, ".");
        assert_eq!(source.inner, "     . . .");

        assert_eq!(source.glyphs.len(), 0);
    }

    #[test]
    fn test_get_expression_as_str() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "i");
        source.set_at(1, 0, "d");
        source.set_at(2, 0, "0");
        source.set_at(3, 0, "A");

        assert_eq!(source.inner, "id0A      ");

        source.print_exp();

        let exp = source
            .map
            .iter()
            .find_map(|o| o.as_ref().map(|e| e))
            .unwrap();

        source.print_exp();

        let s = source.get_exp_str(&exp);

        assert_eq!(s, "id0A");
    }

    // #[test]
    fn test_expressions_list_with_delete() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "A");
        source.set_at(2, 0, "B");
        source.set_at(4, 0, "C");
        assert_eq!(source.inner, "A B C     ");

        assert_eq!(source.glyphs.len(), 3);

        source.set_at(4, 0, ".");
        assert_eq!(source.glyphs.len(), 2);

        source.set_at(2, 0, ".");
        assert_eq!(source.glyphs.len(), 1);

        source.set_at(0, 0, ".");

        assert_eq!(source.glyphs.len(), 0);

        // source.parsed.iter().for_each(|m| {
        //     info!("map: {:?}", m);
        // });
    }

    #[test]
    fn test_expressions_list() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "A");
        // source.set_at(2, 0, "B");
        // source.set_at(4, 0, "C");
        // assert_eq!(source.inner, "A B C     ");

        info!("{:?}", source.inner);
        info!("{:?}", source.glyphs);
        // assert_eq!(source.glyphs.len(), 3);
    }

    #[test]
    fn test_map_expression_with_join() {
        trace();

        // let mut source = Source::from_source(10, 1, "IDAA......");
        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "I");
        source.set_at(1, 0, "D");
        source.set_at(3, 0, "A");
        assert_eq!(source.inner, "ID A      ");

        assert_ne!(source.map[0], source.map[2]);

        source.set_at(2, 0, "A");
        assert_eq!(source.inner, "IDAA      ");

        // Single expression
        assert_eq!(source.map[0], source.map[1]);
        assert_eq!(source.map[0], source.map[2]);
        assert_eq!(source.map[0], source.map[3]);
    }

    #[test]
    fn test_map_expression_at_max_idx() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(6, 0, "I");
        source.set_at(7, 0, "D");
        source.set_at(8, 0, "A");
        source.set_at(9, 0, "A");
        assert_eq!(source.inner, "      IDAA");
    }

    #[test]
    fn test_map_expression_with_prepend() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(6, 0, "I");
        source.set_at(7, 0, "D");
        source.set_at(8, 0, "A");
        source.set_at(9, 0, "A");
        assert_eq!(source.inner, "      IDAA");

        source.set_at(5, 0, "X");
        assert_eq!(source.inner, "     XIDAA");

        assert_eq!(source.map[5], source.map[6]);

        let exp = source.map[5].as_ref().unwrap().as_ref();
        assert_eq!(exp.borrow().start, 5);
    }

    #[test]
    fn test_map_expression_with_split() {
        trace();

        // let mut source = Source::from_source(10, 1, "IDAA......");
        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "I");
        source.set_at(1, 0, "D");
        source.set_at(2, 0, "A");
        source.set_at(3, 0, "A");
        assert_eq!(source.inner, "IDAA      ");

        source.set_at(2, 0, ".");
        assert_eq!(source.inner, "ID.A      ");

        assert_eq!(source.map[0], source.map[1]);
        assert_ne!(source.map[0], source.map[2]);

        // Start of new expression
        assert_ne!(source.map[0], source.map[3]);

        // Now empty
        assert_eq!(source.map[2], None);
    }

    #[test]
    fn test_map_expression_with_insert() {
        trace();

        // let mut source = Source::from_source(10, 1, "IDAA......");
        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "I");
        source.set_at(1, 0, "D");
        source.set_at(2, 0, "A");
        source.set_at(3, 0, "A");
        assert_eq!(source.inner, "IDAA      ");

        source.map.iter().for_each(|m| {
            info!("map: {:?}", m);
        });

        source.set_at(2, 0, "0");
        assert_eq!(source.inner, "ID0A      ");

        assert_eq!(source.map[0], source.map[1]);
        assert_eq!(source.map[0], source.map[2]);
        assert_eq!(source.map[0], source.map[3]);
    }

    #[test]
    fn test_map_expression() {
        trace();

        let mut source = Source::new(10, 1);
        source.set_at(0, 0, "I");

        assert_eq!(source.inner, "I         ");

        source.set_at(1, 0, "D");

        assert_eq!(source.inner, "ID        ");
        assert_eq!(source.map[0], source.map[1]);

        let ptr_0 = source.map[0].as_ref().unwrap().as_ref();
        let ptr_1 = source.map[1].as_ref().unwrap().as_ref();

        assert!(std::ptr::eq(ptr_0, ptr_1));

        source.set_at(2, 0, "0");
        source.set_at(3, 0, "1");

        assert_eq!(source.map[0], source.map[1]);
        assert_eq!(source.map[0], source.map[2]);
        assert_eq!(source.map[0], source.map[3]);
    }

    #[test]
    fn test_set_at() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "T");

        assert_eq!(source.inner, "T         ");

        source.set_at(7, 0, "X");

        assert_eq!(source.inner, "T      X  ");
    }

    #[test]
    fn test_get_at() {
        trace();

        let source = Source::from_source(10, 1, "T......X..");

        let s = source.get_at(0, 0);

        assert_eq!(s, "T");

        let s = source.get_at(7, 0);

        assert_eq!(s, "X");
    }

    #[test]
    #[should_panic(expected = "source length 10, expected 100")]
    fn test_source_() {
        let _source = Source::from_source(10, 10, "..........");
    }

    #[test]
    fn test_to_idx() {
        trace();

        let source = Source::new(10, 10);

        let idx = source.to_idx(0, 0);
        assert_eq!(idx, 0);

        let idx = source.to_idx(5, 5);
        assert_eq!(idx, 55);
    }

    #[test]
    #[should_panic(expected = "index 121 out of bounds for [11,11]")]
    fn test_to_idx_out_of_bounds() {
        let source = Source::new(10, 10);
        let _idx = source.to_idx(11, 11); // This should panic
    }
}
