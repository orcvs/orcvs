use std::cell::RefCell;
use std::rc::Rc;

use tracing::info;

const TERMINATOR: &str = ".";

pub struct Source {
    inner: String,
    cols: usize,
    rows: usize,
    map: Vec<Option<Rc<RefCell<Expression>>>>,
    expressions: Vec<Option<Rc<RefCell<Expression>>>>,
}

#[derive(Debug, PartialEq)]
pub struct Expression {
    start: usize,
    end: usize,
}

impl<'a> Source {
    fn new(cols: usize, rows: usize) -> Self {
        let n = cols * rows;
        let inner = '.'.to_string().repeat(n);

        let map = vec![None; n];
        let expressions = Vec::with_capacity(n / 6);

        Self {
            cols,
            rows,
            inner,
            map,
            expressions,
        }
    }

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

    pub fn get_at(&self, idx: usize) -> &str {
        assert!(idx <= self.len(), "index out of bounds {idx}");
        unsafe { self.inner.get_unchecked(idx..=idx) }
    }

    pub fn set_at(&mut self, x: usize, y: usize, s: &str) {
        let idx = self.to_idx(x, y);

        let b = s.as_bytes();

        unsafe {
            let bytes = self.inner.as_bytes_mut();
            bytes[idx] = b[0];
        }

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

        let terminator = s == TERMINATOR;
        let alphanumeric = !terminator;

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

        // info!("idx {:?} | {:?}", lft_exp, rgt_exp);

        match (lft_exp, rgt_exp) {
            (Some(lft_exp), Some(ref mut rgt_exp)) => {
                // Split the expression
                if terminator {
                    self.map[idx] = None;
                    // In some cases lft and rgt may refer to the same expression
                    // We cannot have multiple mutable borrows in the same scope
                    // So we split the borrows into separate scopes

                    // Right must be first as we want to preserve the end idx
                    // The left expression will be modified
                    {
                        let rgt = rgt_exp.borrow();
                        let exp = Rc::new(RefCell::new(Expression {
                            start: rgt_idx,
                            end: rgt.end,
                        }));
                        self.map[rgt_idx] = Some(exp);
                    }
                    {
                        let mut exp = lft_exp.borrow_mut();
                        exp.end = lft_idx;
                    }
                };

                // Join or Replace the expression
                // Replace is a noop
                if alphanumeric {
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
                        for i in idx..=end {
                            self.map[i] = Some(lft_exp.clone());
                        }
                    }
                }
            }
            (Some(lft_exp), None) => {
                // Append to the expression

                self.map[idx] = Some(lft_exp.clone());

                let mut exp = lft_exp.borrow_mut();
                exp.end = idx;
            }
            (None, Some(rgt_exp)) => {
                // Prepend to the expression
                self.map[idx] = Some(rgt_exp.clone());

                let mut exp = rgt_exp.borrow_mut();

                exp.start = idx;
            }
            (None, None) => {
                if terminator {
                    // Remove expression
                    self.map[idx] = None;
                    self.expressions.pop();
                }

                if alphanumeric {
                    // New Expression
                    let exp = Rc::new(RefCell::new(Expression {
                        start: idx,
                        end: idx,
                    }));
                    self.expressions.push(Some(exp.clone()));
                    self.map[idx] = Some(exp.clone());
                }
            }
        }
    }

    ///
    /// Convert x, y coordinates to a linear index
    /// panic if the index is out of bounds
    ///
    #[inline(always)]
    pub fn to_idx(&self, x: usize, y: usize) -> usize {
        let idx = y * self.cols + x;
        assert!(idx <= self.len(), "index out of bounds {idx} for [{x},{y}]");
        idx
    }

    ///
    /// Current length of the source
    ///
    #[inline(always)]
    fn len(&self) -> usize {
        self.cols * self.rows
    }
}

/*


        ID: [f][f][n][n]

        [ ][ ][ ][ ][ ][ ][ ][ ][ ][ ]

            I
        [ ][ ][ ][ ][ ][ ][ ][ ][ ][ ]
            |
    idx     5
    idx 4 => nil
    new src idx 5

    if map[idx-1] == None
       map[idx] = idx

               D
        [ ][5][5][ ][ ][ ][ ][ ][ ][ ]
               |
    idx        6
    idx 5 => 5


    let start = map[idx-1]
    if Some(start)
       map[idx] = start

*/
#[cfg(test)]
mod test {
    use std::sync::Once;

    use tracing::info;

    use crate::source::Expression;

    use super::Source;

    #[allow(dead_code)]
    static INIT: Once = Once::new();

    #[allow(dead_code)]
    fn trace() {
        INIT.call_once(|| {
            use tracing_subscriber::FmtSubscriber;

            let subscriber = FmtSubscriber::builder()
                .with_max_level(tracing::Level::DEBUG) // Set the maximum level of tracing events that should be logged.
                .with_line_number(true)
                .with_target(true)
                .finish();

            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");
        });
    }

    #[test]
    fn test_expression_delete_last() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(5, 0, "C");
        source.set_at(7, 0, "B");
        source.set_at(9, 0, "A");
        assert_eq!(source.inner, ".....C.B.A");

        assert_eq!(source.expressions.len(), 3);

        source.set_at(5, 0, ".");
        source.set_at(7, 0, ".");
        source.set_at(9, 0, ".");
        assert_eq!(source.inner, "..........");

        // source.expressions.iter().for_each(|m| {
        //     info!("map: {:?}", m);
        // });

        assert_eq!(source.expressions.len(), 0);
    }

    #[test]
    fn test_expressions_list_with_delete() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "A");
        source.set_at(2, 0, "B");
        source.set_at(4, 0, "C");
        assert_eq!(source.inner, "A.B.C.....");

        assert_eq!(source.expressions.len(), 3);

        source.set_at(4, 0, ".");
        assert_eq!(source.expressions.len(), 2);

        source.set_at(2, 0, ".");
        assert_eq!(source.expressions.len(), 1);

        source.set_at(0, 0, ".");

        assert_eq!(source.expressions.len(), 0);

        source.expressions.iter().for_each(|m| {
            info!("map: {:?}", m);
        });
    }

    #[test]
    fn test_expressions_list() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "A");
        source.set_at(2, 0, "B");
        source.set_at(4, 0, "C");
        assert_eq!(source.inner, "A.B.C.....");

        assert_eq!(source.expressions.len(), 3);
    }

    #[test]
    fn test_map_expression_with_join() {
        trace();

        // let mut source = Source::from_source(10, 1, "IDAA......");
        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "I");
        source.set_at(1, 0, "D");
        source.set_at(3, 0, "A");
        assert_eq!(source.inner, "ID.A......");

        assert_ne!(source.map[0], source.map[2]);

        source.set_at(2, 0, "A");
        assert_eq!(source.inner, "IDAA......");

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
        assert_eq!(source.inner, "......IDAA");
    }

    #[test]
    fn test_map_expression_with_prepend() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(6, 0, "I");
        source.set_at(7, 0, "D");
        source.set_at(8, 0, "A");
        source.set_at(9, 0, "A");
        assert_eq!(source.inner, "......IDAA");

        source.set_at(5, 0, "X");
        assert_eq!(source.inner, ".....XIDAA");

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
        assert_eq!(source.inner, "IDAA......");

        source.set_at(2, 0, ".");
        assert_eq!(source.inner, "ID.A......");

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
        assert_eq!(source.inner, "IDAA......");

        source.map.iter().for_each(|m| {
            info!("map: {:?}", m);
        });

        source.set_at(2, 0, "0");
        assert_eq!(source.inner, "ID0A......");

        assert_eq!(source.map[0], source.map[1]);
        assert_eq!(source.map[0], source.map[2]);
        assert_eq!(source.map[0], source.map[3]);
    }

    #[test]
    fn test_map_expression() {
        trace();

        let mut source = Source::new(10, 1);
        source.set_at(0, 0, "I");
        assert_eq!(source.inner, "I.........");

        source.set_at(1, 0, "D");
        assert_eq!(source.inner, "ID........");

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

        assert_eq!(source.inner, "T.........");

        source.set_at(7, 0, "X");

        assert_eq!(source.inner, "T......X..");
    }

    #[test]
    fn test_get_at() {
        trace();

        let source = Source::from_source(10, 1, "T......X..");

        let s = source.get_at(0);

        assert_eq!(s, "T");

        let s = source.get_at(7);

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
    #[should_panic(expected = "index out of bounds")]
    fn test_to_idx_out_of_bounds() {
        let source = Source::new(10, 10);
        let _idx = source.to_idx(11, 11); // This should panic
    }
}
