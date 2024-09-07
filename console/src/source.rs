use lang::Parser;
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use tracing::{debug, error, info};

use crate::coord::Coord;
use crate::glyph::Glyph;
// use arrayvec::ArrayVec;

pub const TERMINATOR: &str = " ";
pub const TERMINATOR_BYTES: &[u8] = TERMINATOR.as_bytes();

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Source {
    pub inner: String,
    map: Vec<Option<Rc<RefCell<Expression>>>>,
    cols: usize,
    rows: usize,
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Expression {
    start: usize,
    end: usize,
}

impl Expression {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        (self.end - self.start) + 1
    }

    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
}

impl Source {
    pub fn new(cols: usize, rows: usize) -> Self {
        assert!(cols > 0, "cols must be greater than zero");
        assert!(rows > 0, "rows must be greater than zero");

        let n = cols * rows;

        // info!("n: {n}");

        let inner = TERMINATOR.to_string().repeat(n);
        let map = vec![None; n];

        Self {
            inner,
            map,
            cols,
            rows,
        }
    }

    ///
    /// Sets `s` at the x, y coords of the grid and recalculates expressions.
    ///
    /// ```
    /// use console::source::Source;
    /// let mut source = source(10, 10);
    /// let idx = source.set_at(Coord::new(3, 3, 10, 1), "!");
    ///
    /// let s = source.get_at(Coord::new(3, 3, 10, 10));
    /// assert_eq!(s.as_str(), "!");
    /// ```
    ///
    pub fn set_at(&mut self, coord: Coord, s: &str) {
        let idx = self.set_at_inner(coord, s);

        // info!("{}", self.inner);
        self.calculate_at(idx, s);
    }

    pub fn get_at(&self, coord: Coord) -> String {
        // info!("x: {x}, y: {y}");

        let idx = coord.index();

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
    pub fn unset_at(&mut self, coord: Coord) {
        self.set_at(coord, TERMINATOR);
    }

    pub fn get_exp_at(&self, coord: Coord) -> Option<Expression> {
        let idx = coord.index();

        if let Some(exp) = &self.map[idx] {
            let exp = exp.borrow();
            Some(*exp)
        } else {
            None
        }
    }

    pub fn get_exp_with_src_at(&self, coord: Coord) -> Option<(Expression, String)> {
        let idx = coord.index();

        if let Some(exp) = &self.map[idx] {
            let exp = exp.borrow();
            let src = self.get_exp_src(exp.start, exp.end);
            Some((*exp, src))
        } else {
            None
        }
    }

    #[inline]
    fn get_exp_src(&self, from: usize, to: usize) -> String {
        // SAFELY UNSAFE
        // all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds
        let s = unsafe { self.inner.get_unchecked(from..(to + 1)) };
        s.to_owned()
    }

    fn set_exp_from(&mut self, start: usize, end: usize, exp: &Rc<RefCell<Expression>>) {
        for i in start..(end + 1) {
            self.map[i] = Some(exp.clone());
        }
    }

    fn set_exp(&mut self, idx: usize, exp: Rc<RefCell<Expression>>) {
        self.map[idx] = Some(exp);
    }

    fn end_exp(&mut self, idx: usize, exp: &Rc<RefCell<Expression>>) {
        let mut exp = exp.borrow_mut();
        exp.end = idx;
    }

    fn remove_exp(&mut self, idx: usize) {
        self.map[idx] = None;
    }

    fn start_exp(&mut self, idx: usize, exp: &Rc<RefCell<Expression>>) {
        let mut exp = exp.borrow_mut();
        exp.start = idx;
    }

    fn join_exp(
        &mut self,
        idx: usize,
        lft_exp: &Rc<RefCell<Expression>>,
        rgt_exp: &Rc<RefCell<Expression>>,
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
        lft_exp: &Rc<RefCell<Expression>>,
        rgt_idx: usize,
        rgt_exp: &Rc<RefCell<Expression>>,
    ) {
        {
            // lft and rgt often refer to the same expression
            // We cannot have multiple mutable borrows in the same scope
            // So we split the borrows into separate scopes
            // Right must be first as we want to capture the end value before modifying the left value
            let rgt = rgt_exp.borrow();
            let rgt_end = rgt.end;
            let exp = Rc::new(RefCell::new(Expression::new(rgt_idx, rgt_end)));
            self.set_exp_from(rgt_idx, rgt_end, &exp);
        }
        // Update A-1 (Left)
        // Expression now ends at A-1 (Left)
        self.end_exp(lft_idx, &lft_exp);
    }

    fn append_exp(&mut self, idx: usize, exp: Rc<RefCell<Expression>>) {
        self.end_exp(idx, &exp);
        self.set_exp(idx, exp);
    }

    fn prepend_exp(&mut self, idx: usize, exp: Rc<RefCell<Expression>>) {
        self.start_exp(idx, &exp);
        self.set_exp(idx, exp);
    }

    fn new_exp(&mut self, idx: usize) {
        let exp = Rc::new(RefCell::new(Expression::new(idx, idx)));
        self.set_exp(idx, exp);
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
    fn set_at_inner(&mut self, coord: Coord, s: &str) -> usize {
        let idx = coord.index();

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

    fn calculate_at(&mut self, idx: usize, s: &str) {
        // let exp = self.map[idx].clone();

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
    /// Current length of the source
    ///
    fn len(&self) -> usize {
        self.cols * self.rows
    }
}

#[cfg(test)]
mod test {
    use crate::coord::Coord;
    use crate::source::Expression;
    use crate::source::Glyph;
    use crate::source::Source;
    use crate::test::trace;
    use tracing::{debug, info};

    fn source(cols: usize, rows: usize) -> Source {
        Source::new(cols, rows)
    }
    // ===========================================================================

    #[test]
    fn expression_delete_last() {
        trace();

        let mut source = source(10, 1);

        source.set_at(Coord::new(5, 0, 10, 1), "C");
        source.set_at(Coord::new(7, 0, 10, 1), "B");
        source.set_at(Coord::new(9, 0, 10, 1), "A");

        assert_eq!(source.inner, "     C B A");

        let count = source
            .map
            .iter()
            .filter_map(|o| o.as_deref())
            .collect::<Vec<_>>()
            .len();

        assert_eq!(count, 3);

        source.set_at(Coord::new(5, 0, 10, 1), " ");
        source.set_at(Coord::new(7, 0, 10, 1), " ");
        source.set_at(Coord::new(9, 0, 10, 1), " ");
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

        let mut source = source(10, 1);

        source.set_at(Coord::new(0, 0, 10, 1), "i");
        source.set_at(Coord::new(1, 0, 10, 1), "d");
        source.set_at(Coord::new(2, 0, 10, 1), "0");
        source.set_at(Coord::new(3, 0, 10, 1), "A");

        assert_eq!(source.inner, "id0A      ");

        source.print_exp();

        let exp = source
            .map
            .iter()
            .find_map(|o| o.as_ref().map(|e| e))
            .unwrap();

        source.print_exp();
        let exp = exp.borrow();
        let s = source.get_exp_src(exp.start, exp.end);

        assert_eq!(s, "id0A");
    }

    #[test]
    fn test_map_expression_with_join() {
        trace();

        // let mut source = Source::from_source(10, 1, "IDAA......");
        let mut source = source(10, 1);

        source.set_at(Coord::new(0, 0, 10, 1), "I");
        source.set_at(Coord::new(1, 0, 10, 1), "D");
        source.set_at(Coord::new(3, 0, 10, 1), "A");
        assert_eq!(source.inner, "ID A      ");

        assert_ne!(source.map[0], source.map[2]);
        assert_ne!(source.map[2], source.map[3]);

        source.set_at(Coord::new(2, 0, 10, 1), "A");
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

        let mut source = source(10, 1);

        source.set_at(Coord::new(6, 0, 10, 1), "I");
        source.set_at(Coord::new(7, 0, 10, 1), "D");
        source.set_at(Coord::new(8, 0, 10, 1), "A");
        source.set_at(Coord::new(9, 0, 10, 1), "A");
        assert_eq!(source.inner, "      IDAA");
    }

    #[test]
    fn test_map_expression_with_prepend() {
        trace();

        let mut source = source(10, 1);

        source.set_at(Coord::new(6, 0, 10, 1), "I");
        source.set_at(Coord::new(7, 0, 10, 1), "D");
        source.set_at(Coord::new(8, 0, 10, 1), "A");
        source.set_at(Coord::new(9, 0, 10, 1), "A");
        assert_eq!(source.inner, "      IDAA");

        source.set_at(Coord::new(5, 0, 10, 1), "X");
        assert_eq!(source.inner, "     XIDAA");

        assert_eq!(source.map[5], source.map[6]);

        let exp = source.map[5].as_ref().unwrap().as_ref();
        assert_eq!(exp.borrow().start, 5);
    }

    #[test]
    fn test_map_expression_with_split() {
        trace();

        // let mut source = Source::from_source(10, 1, "IDAA......");
        let mut source = source(10, 1);

        source.set_at(Coord::new(0, 0, 10, 1), "+");
        source.set_at(Coord::new(1, 0, 10, 1), "+");
        source.set_at(Coord::new(2, 0, 10, 1), "A");
        source.set_at(Coord::new(3, 0, 10, 1), "A");
        source.set_at(Coord::new(4, 0, 10, 1), "B");
        source.set_at(Coord::new(5, 0, 10, 1), "B");
        assert_eq!(source.inner, "++AABB    ");

        source.set_at(Coord::new(2, 0, 10, 1), " ");
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
        let mut source = source(10, 1);

        source.set_at(Coord::new(0, 0, 10, 1), "I");
        source.set_at(Coord::new(1, 0, 10, 1), "D");
        source.set_at(Coord::new(2, 0, 10, 1), "A");
        source.set_at(Coord::new(3, 0, 10, 1), "A");
        assert_eq!(source.inner, "IDAA      ");

        source.map.iter().for_each(|m| {
            info!("map: {:?}", m);
        });

        source.set_at(Coord::new(2, 0, 10, 1), "0");
        assert_eq!(source.inner, "ID0A      ");

        assert_eq!(source.map[0], source.map[1]);
        assert_eq!(source.map[0], source.map[2]);
        assert_eq!(source.map[0], source.map[3]);
    }

    #[test]
    fn test_map_expression() {
        trace();

        let mut source = source(10, 1);
        source.set_at(Coord::new(0, 0, 10, 1), "I");

        assert_eq!(source.inner, "I         ");

        source.set_at(Coord::new(1, 0, 10, 1), "D");

        assert_eq!(source.inner, "ID        ");
        assert_eq!(source.map[0], source.map[1]);

        let ptr_0 = source.map[0].as_ref().unwrap().as_ref();
        let ptr_1 = source.map[1].as_ref().unwrap().as_ref();

        assert!(std::ptr::eq(ptr_0, ptr_1));

        source.set_at(Coord::new(2, 0, 10, 1), "0");
        source.set_at(Coord::new(3, 0, 10, 1), "1");

        assert_eq!(source.map[0], source.map[1]);
        assert_eq!(source.map[0], source.map[2]);
        assert_eq!(source.map[0], source.map[3]);
    }

    #[test]
    fn test_set_at() {
        trace();

        let mut source = source(10, 1);

        source.set_at(Coord::new(0, 0, 10, 1), "T");

        assert_eq!(source.inner, "T         ");

        source.set_at(Coord::new(7, 0, 10, 1), "X");

        assert_eq!(source.inner, "T      X  ");
    }

    #[test]
    fn test_get_at() {
        trace();

        let source = Source::from_source(10, 1, "T......X..");

        let s = source.get_at(Coord::new(0, 0, 10, 10));

        assert_eq!(s, "T");

        let s = source.get_at(Coord::new(7, 0, 10, 10));

        assert_eq!(s, "X");
    }

    #[test]
    fn test_expression_len() {
        let exp = Expression { start: 0, end: 0 };
        assert_eq!(exp.len(), 1);

        let exp = Expression { start: 0, end: 1 };
        assert_eq!(exp.len(), 2);
    }

    #[test]
    #[should_panic(expected = "source length 10, expected 100")]
    fn test_source_() {
        let _source = Source::from_source(10, 10, "..........");
    }

    impl Source {
        fn from_source(cols: usize, rows: usize, s: impl Into<String>) -> Self {
            let n = cols * rows;
            let inner = s.into();
            let len = inner.len();

            assert!(len == n, "source length {len}, expected {n}");

            let mut source = source(cols, rows);

            // Iterate through inner and call set_at
            for (idx, &byte) in inner.as_bytes().iter().enumerate() {
                let x = idx % cols;
                let y = idx / cols;
                source.set_at(Coord::new(x, y, 10, 1), &(byte as char).to_string());
            }

            source
        }

        fn print_exp(&self) {
            self.map.iter().for_each(|m| {
                info!("map: {:?}", m);
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
}
