use arrayvec::ArrayVec;
use lang::{Atom, Expression, Function, Parsed, Parser, Token};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::io::repeat;
use std::iter;
use std::rc::Rc;
use tracing::{debug, error, info};

use crate::glyph::{Glyph, Terminator, G};
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
    parsed_len: usize,
    // #[serde(skip_deserializing, skip_serializing)]
    // valid: Option<bool>,
    // function: Option<Function>,
}

impl SourceExpression {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            parsed_len: Default::default(),
            // valid: None,
            // function: None,
        }
    }
}

// Parsed([Expression { token: Function, atom: Some(Function(Id)) }, Expression { token: String, atom: Some(Empty) }])

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

        info!("n: {n}");

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
        info!("x: {x}, y: {y}");

        let idx = self.to_idx(x, y);

        info!("idx: {idx}");

        // SAFELY UNSAFE
        // all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds
        let s = unsafe { self.inner.get_unchecked(idx..(idx + 1)) };
        s.to_owned()
    }

    #[inline]
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

    pub fn set_exp_from(&mut self, start: usize, end: usize, exp: &Rc<RefCell<SourceExpression>>) {
        for i in start..(end + 1) {
            self.map[i] = Some(exp.clone());
        }
        self.parse_exp(&exp);
    }

    pub fn set_exp(&mut self, idx: usize, exp: Rc<RefCell<SourceExpression>>) {
        self.map[idx] = Some(exp);
    }

    pub fn end_exp(&mut self, idx: usize, exp: &Rc<RefCell<SourceExpression>>) {
        {
            let mut exp = exp.borrow_mut();
            exp.end = idx;
        }
        self.parse_exp(exp);
    }

    pub fn start_exp(&mut self, idx: usize, exp: &Rc<RefCell<SourceExpression>>) {
        {
            let mut exp = exp.borrow_mut();
            exp.start = idx;
        }
        self.parse_exp(exp);
    }

    fn join_exp(
        &mut self,
        idx: usize,
        lft_exp: &Rc<RefCell<SourceExpression>>,
        rgt_exp: &Rc<RefCell<SourceExpression>>,
    ) {
        let idx_exp = &self.map[idx];
        if idx_exp.is_none() {
            // In some cases lft and rgt may refer to the same expression
            // We cannot have multiple mutable borrows in the same scope
            // So we split the borrows into separate scopes

            // Right must be first as we want to preserve the end idx
            // The left expression will be modified
            let rgt = rgt_exp.borrow();
            let end = rgt.end;
            {
                self.end_exp(end, &lft_exp);
            }
            // Iterate the map until the rgt end and set the lft expression
            self.set_exp_from(idx, end, lft_exp);
        }
    }

    fn split_exp(
        &mut self,
        lft_idx: usize,
        lft_exp: &Rc<RefCell<SourceExpression>>,
        rgt_idx: usize,
        rgt_exp: &Rc<RefCell<SourceExpression>>,
    ) {
        {
            // lft and rgt often refer to the same expression
            // We cannot have multiple mutable borrows in the same scope
            // So we split the borrows into separate scopes
            // Right must be first as we want to capture the end value before modifying the left value
            let rgt = rgt_exp.borrow();
            let rgt_end = rgt.end;
            let exp = Rc::new(RefCell::new(SourceExpression::new(rgt_idx, rgt_end)));
            self.set_exp_from(rgt_idx, rgt_end, &exp);
        }
        // Update A-1 (Left)
        // Expression now ends at A-1 (Left)
        self.end_exp(lft_idx, &lft_exp);
    }

    fn append_exp(&mut self, idx: usize, exp: Rc<RefCell<SourceExpression>>) {
        self.end_exp(idx, &exp);
        self.set_exp(idx, exp);
    }

    fn prepend_exp(&mut self, idx: usize, exp: Rc<RefCell<SourceExpression>>) {
        self.start_exp(idx, &exp);
        self.set_exp(idx, exp);
    }

    fn new_exp(&mut self, idx: usize) {
        let exp = Rc::new(RefCell::new(SourceExpression::new(idx, idx)));
        self.set_exp(idx, exp);
    }

    pub fn remove_exp(&mut self, idx: usize) {
        self.map[idx] = None;
    }

    pub fn get_glyph_at(&self, x: usize, y: usize) -> Glyph {
        let idx = self.to_idx(x, y);
        // let s = self.get_at(x, y);
        // if x == 2 || x == 3 {
        //     info!("glyphs {:?}", self.glyphs.iter().take(10));
        // }
        *self.glyphs.get(idx).unwrap()
    }

    fn unset_glyphs(&mut self, start: usize, len: usize) {
        for i in 0..len {
            let pos = start + i;
            self.glyphs[pos] = Glyph::Terminator(Terminator::Space);
        }
    }

    pub fn parse_exp(&mut self, exp: &Rc<RefCell<SourceExpression>>) {
        let mut s = self.get_exp_str(&exp).to_owned();

        let parsed = Parser::from(&mut s).parse().take();

        let glyphs = to_glyphs(&parsed);

        let mut exp = exp.borrow_mut();

        let idx = exp.start;
        let len = exp.parsed_len;

        self.unset_glyphs(idx, len);

        if !glyphs.is_empty() {
            exp.parsed_len = glyphs.len();
            self.parsed.insert(idx, parsed);
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
    /// Unset at {x, y}
    /// Sets a `TERMINATOR` at the position`
    ///
    pub fn unset_at(&mut self, x: usize, y: usize) {
        self.set_at(x, y, TERMINATOR);
    }

    pub fn calculate_at(&mut self, idx: usize, s: &str) {
        let exp = self.map[idx].clone();

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

        let terminator = Glyph::is_terminator(s);
        let glyph = !terminator;

        if terminator {
            self.remove_exp(idx);
        }

        match (lft_exp, rgt_exp) {
            (Some(lft_exp), Some(ref mut rgt_exp)) => {
                if glyph {
                    self.join_exp(idx, &lft_exp, &rgt_exp);
                } else {
                    self.split_exp(lft_idx, &lft_exp, rgt_idx, &rgt_exp);
                }
            }
            (Some(lft_exp), None) => {
                if glyph {
                    // self.end_exp(idx, &lft_exp);
                    // self.set_exp(idx, lft_exp);
                    self.append_exp(idx, lft_exp);
                } else {
                    self.end_exp(lft_idx, &lft_exp);
                }
            }
            (None, Some(rgt_exp)) => {
                if glyph {
                    // self.start_exp(idx, &rgt_exp);
                    // self.set_exp(idx, rgt_exp);
                    self.prepend_exp(idx, rgt_exp);
                } else {
                    self.start_exp(rgt_idx, &rgt_exp);
                }
            }
            (None, None) => {
                if glyph {
                    self.new_exp(idx);
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
}

#[cfg(test)]
mod test {
    use crate::source::Glyph;
    use crate::source::Source;
    use crate::source::SourceExpression;
    use crate::source::Terminator;
    use crate::source::G;
    use crate::test::trace;
    use std::fmt::Debug;
    use tracing::{debug, info};

    // ===========================================================================
    #[test]
    fn test_simple_edit() {
        trace();

        // let mut source = source_from("  ++0101  ");
        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "i");
        assert_eq!(source.inner, "i         ");

        // `+` is not a function yet
        let glyph = source.get_glyph_at(0, 0);
        assert_eq!(glyph, Glyph::default());
        assert_eq!(source.map_len(), 1);

        source.set_at(1, 0, "d");
        assert_eq!(source.inner, "id        ");
        assert_eq!(source.map_len(), 2);

        let glyph = source.get_glyph_at(2, 0);
        assert_eq!(glyph, Glyph::String);

        source.unset_at(1, 0);

        for x in 0..3 {
            let glyph = source.get_glyph_at(x, 0);
            assert_eq!(glyph, Glyph::default());
        }

        source.unset_at(0, 0);

        for x in 0..3 {
            let glyph = source.get_glyph_at(x, 0);
            assert_eq!(glyph, Glyph::default());
        }

        source.print_exp();

        // // Now `++` is a function yet
        // let glyph = source.get_glyph_at(3, 0);
        // assert_eq!(glyph, Glyph::Function);
        // let glyph = source.get_glyph_at(4, 0);
        // assert_eq!(glyph, Glyph::Number);
        // assert_eq!(source.map_len(), 2);
    }

    #[test]
    fn test_get_glyphs_with_edit() {
        trace();

        // let mut source = source_from("  ++0101  ");
        let mut source = Source::new(10, 1);

        source.set_at(2, 0, "+");
        assert_eq!(source.inner, "  +       ");

        // `+` is not a function yet
        let glyph = source.get_glyph_at(2, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));
        assert_eq!(source.map_len(), 1);

        source.set_at(3, 0, "+");
        assert_eq!(source.inner, "  ++      ");

        // Now `++` is a function yet
        let glyph = source.get_glyph_at(3, 0);
        assert_eq!(glyph, Glyph::Function);
        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Number);
        assert_eq!(source.map_len(), 2);

        source.set_at(4, 0, "1");
        assert_eq!(source.inner, "  ++1     ");

        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Number);
        assert_eq!(source.map_len(), 3);

        source.set_at(5, 0, "1");
        assert_eq!(source.inner, "  ++11    ");

        let glyph = source.get_glyph_at(5, 0);
        assert_eq!(glyph, Glyph::Number);
        assert_eq!(source.map_len(), 4);

        source.set_at(6, 0, "2");
        assert_eq!(source.inner, "  ++112   ");

        let glyph = source.get_glyph_at(6, 0);
        assert_eq!(glyph, Glyph::Number);
        assert_eq!(source.map_len(), 5);

        source.set_at(7, 0, "2");
        assert_eq!(source.inner, "  ++1122  ");

        let glyph = source.get_glyph_at(7, 0);
        assert_eq!(glyph, Glyph::Number);
        assert_eq!(source.map_len(), 6);

        // --

        source.unset_at(7, 0);
        assert_eq!(source.inner, "  ++112   ");
        assert_eq!(source.map_len(), 5);

        let glyph = source.get_glyph_at(7, 0);
        assert_eq!(glyph, Glyph::Number);

        // --

        source.unset_at(4, 0);
        assert_eq!(source.inner, "  ++ 12   ");

        assert_eq!(source.map_len(), 4);
        // `++`` and `12` are distinct expressions`
        assert_ne!(source.map[2], source.map[4]);

        // `++` is valid, glyphs map to the expanded function
        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Number);

        // --

        source.unset_at(3, 0);
        assert_eq!(source.inner, "  +  12   ");

        assert_eq!(source.map_len(), 3);

        // Now `+` is not a valid function, no glyphs
        let glyph = source.get_glyph_at(4, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));

        let glyph = source.get_glyph_at(3, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));

        // --

        source.unset_at(2, 0);
        assert_eq!(source.inner, "     12   ");
        assert_eq!(source.map_len(), 2);

        let glyph = source.get_glyph_at(2, 0);
        assert_eq!(glyph, Glyph::Terminator(Terminator::Space));

        source.set_at(3, 0, "+");
        source.set_at(4, 0, "+");
        assert_eq!(source.inner, "   ++12   ");
        assert_eq!(source.map_len(), 4);

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
        let t = Glyph::is_terminator(".");
        assert!(t);

        let t = Glyph::is_terminator(" ");
        assert!(t);

        let t = Glyph::is_terminator("+");
        assert!(t == false);

        let t = Glyph::is_terminator("..");
        assert!(t == false);

        let t = Glyph::is_terminator("!");
        assert!(t == false);
    }

    #[test]
    fn test_is_terminator_bytes() {
        let b = ".".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(t);

        let b = " ".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(t);

        let b = "+".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(t == false);

        let b = "!".as_bytes();
        let t = Glyph::is_terminator_bytes(b[0]);
        assert!(t == false);
    }

    #[test]
    fn expression_delete_last() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(5, 0, "C");
        source.set_at(7, 0, "B");
        source.set_at(9, 0, "A");

        assert_eq!(source.inner, "     C B A");

        let count = source
            .map
            .iter()
            .filter_map(|o| o.as_deref())
            .collect::<Vec<_>>()
            .len();

        assert_eq!(count, 3);

        source.set_at(5, 0, " ");
        source.set_at(7, 0, " ");
        source.set_at(9, 0, " ");
        assert_eq!(source.inner, "          ");

        let count = source
            .map
            .iter()
            .filter_map(|o| o.as_deref())
            .collect::<Vec<_>>()
            .len();
        tracing::debug!("source.map {:?}", count);
        assert_eq!(count, 0);
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
    fn _test_expressions_list_with_delete() {
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
    fn test_glyphs_with_invalid() {
        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "A");
        source.set_at(2, 0, "B");
        source.set_at(4, 0, "C");
        assert_eq!(source.inner, "A B C     ");

        info!("{:?}", source.inner);
        info!("{:?}", source.glyphs);

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
        assert_eq!(source.inner, "id        ");

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
        assert_eq!(source.inner, "id        ");

        let expected = vec![&G::Function, &G::Function, &G::String, &G::String];
        let results: Vec<&Glyph> = source.glyphs.iter().take(4).collect();

        assert_eq!(results, expected);

        source.set_at(0, 0, "i");
        source.set_at(1, 0, " ");
        assert_eq!(source.inner, "i         ");

        // You are here
        // The glyphs need to be reset when a new Parsed is invalid/empty
        // i is nothing from highligh perspective
        // info!("{:?}", source.inner);
        info!("{:?}", source.glyphs);
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
        assert_ne!(source.map[2], source.map[3]);

        source.set_at(2, 0, "A");
        assert_eq!(source.inner, "IDAA      ");

        source.print_exp();
        // info!("source.map {:?}", source.map);
        // info!("source.glyphs {:?}", source.glyphs);

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

        source.set_at(0, 0, "+");
        source.set_at(1, 0, "+");
        source.set_at(2, 0, "A");
        source.set_at(3, 0, "A");
        source.set_at(4, 0, "B");
        source.set_at(5, 0, "B");
        assert_eq!(source.inner, "++AABB    ");

        source.set_at(2, 0, " ");
        assert_eq!(source.inner, "++ ABB    ");

        // ++
        assert_eq!(source.map[0], source.map[1]);

        // ` `
        assert_eq!(source.map[2], None);

        // + and ` `
        assert_ne!(source.map[0], source.map[2]);

        // + and A
        assert_ne!(source.map[0], source.map[3]);

        // ABB
        assert_eq!(source.map[3], source.map[4]);
        assert_eq!(source.map[3], source.map[5]);

        // end
        assert_eq!(source.map[6], None);
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
            self.map.iter().for_each(|m| {
                info!("map: {:?}", m);
            });
            self.glyphs.iter().for_each(|m| {
                info!("parsed: {:?}", m);
            });
        }

        fn map_len(&self) -> usize {
            self.map
                .iter()
                .filter(|e| e.is_some())
                .collect::<Vec<_>>()
                .len()
        }
    }

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

        assert_eq!(source.inner, expected);
        source
    }
}
