use arrayvec::ArrayVec;
use lang::{Atom, Expression, Function, Parsed, Parser, Token};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::io::repeat;
use std::iter;
use std::rc::Rc;
use tracing::{debug, error, info, warn};
// use arrayvec::ArrayVec;

pub const TERMINATOR: &str = " ";
pub const TERMINATOR_BYTES: &[u8] = TERMINATOR.as_bytes();

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Source {
    pub glyphs: Vec<Glyph>,
    pub src: Vec<String>,
    cols: usize,
    rows: usize,
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
        assert!(cols > 0, "cols must be greater than zero");
        assert!(rows > 0, "rows must be greater than zero");

        let n = cols * rows;

        let glyphs = vec![Glyph::default(); n];
        let src = vec![TERMINATOR.to_string(); n];

        Self {
            glyphs,
            src,
            cols,
            rows,
        }
    }

    pub fn get_at(&self, x: usize, y: usize) -> String {
        let idx = self.to_idx(x, y);
        self.src[idx].to_owned()
    }

    pub fn get_glyph_at(&self, x: usize, y: usize) -> Glyph {
        let idx = self.to_idx(x, y);
        // let s = self.get_at(x, y);

        *self.glyphs.get(idx).unwrap()
    }

    pub fn parse(&mut self, _idx: usize) {
        let mut exp = Vec::<String>::new();
        let mut glyph_map: HashMap<usize, Vec<Glyph>, nohash_hasher::BuildNoHashHasher<usize>> =
            HashMap::with_capacity_and_hasher(self.len(), BuildHasherDefault::default());

        let mut start = 0;

        for (idx, s) in self.src.iter().enumerate() {
            self.glyphs[idx] = Glyph::Terminator(Terminator::Space);

            if is_terminator(s) {
                if !exp.is_empty() {
                    let mut s = exp
                        .iter()
                        .map(|s| s.to_owned())
                        .collect::<Vec<_>>()
                        .join("");

                    let parsed = Parser::from(&mut s).parse().take();
                    let glyphs = to_glyphs(&parsed);

                    glyph_map.insert(start, glyphs);

                    exp.clear();
                }
            } else {
                if exp.is_empty() {
                    start = idx;
                }
                exp.push(s.to_owned());
            }
        }

        for (idx, glyphs) in glyph_map.iter() {
            for (i, g) in glyphs.iter().enumerate() {
                let pos = idx + i;
                self.glyphs[pos] = *g;
            }
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
        let idx = self.to_idx(x, y);

        self.src[idx] = s.to_owned();
        self.parse(idx);
    }

    ///
    /// Unset at {x, y}
    /// Sets a `TERMINATOR` at the position`
    ///
    pub fn unset_at(&mut self, x: usize, y: usize) {
        self.set_at(x, y, TERMINATOR);
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

    pub fn as_string(&self) -> String {
        self.src
            .iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
            .join("")
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
    use crate::source::Terminator;
    use crate::source::G;
    use crate::test::trace;
    use std::fmt::Debug;
    use tracing::{debug, info};

    // ===========================================================================
    #[test]
    fn test_get_glyph_at_with_edit() {
        trace();

        // let mut source = source_from("  ++0101  ");
        let mut source = Source::new(10, 1);

        source.set_at(2, 0, "+");
        assert_eq!(source.as_string(), "  +       ");

        // `+` is not a function yet
        let glyph = source.get_glyph_at(2, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));
        // assert_eq!(source.map_len(), 1);

        source.set_at(3, 0, "+");
        assert_eq!(source.as_string(), "  ++      ");

        // Now `++` is a function yet
        let glyph = source.get_glyph_at(3, 0);
        assert_eq!(glyph, Glyph::Function);
        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Number);

        source.set_at(4, 0, "1");
        assert_eq!(source.as_string(), "  ++1     ");

        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Number);

        source.set_at(5, 0, "1");
        assert_eq!(source.as_string(), "  ++11    ");

        let glyph = source.get_glyph_at(5, 0);
        assert_eq!(glyph, Glyph::Number);

        source.set_at(6, 0, "2");
        assert_eq!(source.as_string(), "  ++112   ");

        let glyph = source.get_glyph_at(6, 0);
        assert_eq!(glyph, Glyph::Number);

        source.set_at(7, 0, "2");
        assert_eq!(source.as_string(), "  ++1122  ");

        let glyph = source.get_glyph_at(7, 0);
        assert_eq!(glyph, Glyph::Number);

        // --

        source.unset_at(7, 0);
        assert_eq!(source.as_string(), "  ++112   ");

        let glyph = source.get_glyph_at(7, 0);
        assert_eq!(glyph, Glyph::Number);

        // --

        source.unset_at(4, 0);
        assert_eq!(source.as_string(), "  ++ 12   ");

        // `++` is valid, glyphs map to the expanded function
        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Number);

        // --

        source.unset_at(3, 0);
        assert_eq!(source.as_string(), "  +  12   ");

        // Now `+` is not a valid function, no glyphs
        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));

        let glyph = source.get_glyph_at(3, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));

        // --

        source.unset_at(2, 0);
        assert_eq!(source.as_string(), "     12   ");

        let glyph = source.get_glyph_at(2, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));

        source.set_at(3, 0, "+");
        source.set_at(4, 0, "+");
        assert_eq!(source.as_string(), "   ++12   ");

        let glyph = source.get_glyph_at(3, 0);
        assert_eq!(glyph, Glyph::Function);

        let glyph = source.get_glyph_at(5, 0);
        assert_eq!(glyph, Glyph::Number);
    }

    // ===========================================================================
    // ===========================================================================
    // ===========================================================================
    // ===========================================================================

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

        let glyph = source.get_glyph_at(6, 0);
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

        assert_eq!(source.as_string(), "     C B A");

        source.set_at(5, 0, " ");
        source.set_at(7, 0, " ");
        source.set_at(9, 0, " ");
        assert_eq!(source.as_string(), "          ");
    }

    #[test]
    fn test_get_expression_as_str() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "i");
        source.set_at(1, 0, "d");
        source.set_at(2, 0, "0");
        source.set_at(3, 0, "A");

        assert_eq!(source.as_string(), "id0A      ");
    }

    // #[test]
    fn _test_expressions_list_with_delete() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "A");
        source.set_at(2, 0, "B");
        source.set_at(4, 0, "C");
        assert_eq!(source.as_string(), "A B C     ");

        assert_eq!(source.glyphs.len(), 3);

        source.set_at(4, 0, ".");
        assert_eq!(source.glyphs.len(), 2);

        source.set_at(2, 0, ".");
        assert_eq!(source.glyphs.len(), 1);

        source.set_at(0, 0, ".");

        assert_eq!(source.glyphs.len(), 0);
    }

    #[test]
    fn test_glyphs_with_invalid() {
        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "A");
        source.set_at(2, 0, "B");
        source.set_at(4, 0, "C");
        assert_eq!(source.as_string(), "A B C     ");

        for g in source.glyphs {
            assert!(matches!(g, Glyph::Terminator(_)));
        }
    }

    #[test]
    fn test_glyphs_with_incomplete_function() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "i");
        source.set_at(1, 0, "d");
        assert_eq!(source.as_string(), "id        ");

        let expected = vec![G::Function, G::Function, G::String, G::String];
        let results: Vec<Glyph> = source.glyphs.into_iter().take(4).collect();
        assert_eq!(results, expected)
    }

    #[test]
    fn test_glyphs_with_incomplete_function_edited() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "i");
        source.set_at(1, 0, "d");
        assert_eq!(source.as_string(), "id        ");

        let expected = vec![&G::Function, &G::Function, &G::String, &G::String];
        let results: Vec<&Glyph> = source.glyphs.iter().take(4).collect();

        assert_eq!(results, expected);

        source.print_exp();

        source.set_at(0, 0, "i");
        source.set_at(1, 0, " ");
        assert_eq!(source.as_string(), "i         ");

        let glyph = source.get_glyph_at(0, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));
    }

    #[test]
    fn test_map_expression_at_max_idx() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(6, 0, "I");
        source.set_at(7, 0, "D");
        source.set_at(8, 0, "A");
        source.set_at(9, 0, "A");
        assert_eq!(source.as_string(), "      IDAA");
    }

    #[test]
    fn test_map_expression_with_prepend() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(6, 0, "I");
        source.set_at(7, 0, "D");
        source.set_at(8, 0, "A");
        source.set_at(9, 0, "A");
        assert_eq!(source.as_string(), "      IDAA");

        source.set_at(5, 0, "X");
        assert_eq!(source.as_string(), "     XIDAA");
    }

    #[test]
    fn test_map_expression_with_split() {
        trace();

        // let mut source = Source::from_source(10, 1, "IDAA......");
        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "+");
        source.set_at(1, 0, "+");
        source.set_at(2, 0, "A");
        source.set_at(3, 0, "A");
        source.set_at(4, 0, "B");
        source.set_at(5, 0, "B");
        assert_eq!(source.as_string(), "++AABB    ");

        source.set_at(2, 0, " ");
        assert_eq!(source.as_string(), "++ ABB    ");

        // ++
        let glyph = source.get_glyph_at(0, 0);
        assert_eq!(glyph, Glyph::Function);

        let glyph = source.get_glyph_at(2, 0);
        assert_eq!(glyph, Glyph::Number);
    }

    #[test]
    fn test_set_at() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "T");

        assert_eq!(source.as_string(), "T         ");

        source.set_at(7, 0, "X");

        assert_eq!(source.as_string(), "T      X  ");
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

    impl Source {
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

        fn print_exp(&self) {
            info!("{:?}", self.as_string());

            self.glyphs.iter().for_each(|m| {
                info!("parsed: {:?}", m);
            });
        }
    }

    fn source_from(s: &str) -> Source {
        let mut source = Source::new(10, 1);

        let y = 0;

        for (x, c) in s.chars().enumerate() {
            source.set_at(x, y, &c.to_string());
        }

        // append '.' to s to fil the source to len 10
        let l = s.len();
        let mut expected = String::from(s);
        expected.push_str(&".".repeat(10 - l));

        assert_eq!(source.as_string(), expected);
        source
    }
}
