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
use std::cell::RefCell;
use std::rc::Rc;

use tracing::info;

pub struct Source {
    inner: String,
    cols: usize,
    rows: usize,
    // map: Vec<Rc<RefCell<Option<Expression>>>>,
    // expressions: Vec<Rc<RefCell<Option<Expression>>>>,
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
        let expressions = vec![None; n];

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

        let lhs = idx.checked_sub(1).unwrap_or(0);
        let rhs = std::cmp::min(idx + 1, self.len() - 1);

        /*

            ch:  I
            idx: 0
            lhs: 0
            rhs: 1

            if None at idx 0
                new expression

            if Some at idx 0
                if end == lhs
                    append to expression


           ch:  D
           idx: 1
           lhs: 0
           rhs: 2

        */

        // if lhs == 0 {
        //     info!("first");
        // }

        // if rhs == idx {
        //     info!("last");
        // }

        let lhs_exp = self.map[lhs].clone();
        let rhs_exp = self.map[rhs].clone();

        info!("lhs: {:?}", lhs);
        info!("expression: {:?}", lhs_exp);

        let end_idx = match rhs_exp {
            Some(exp) => {
                // Inserting into an existing expression
                exp.borrow().end
            }
            None => {
                // Appending to the expression
                idx
            }
        };

        match lhs_exp {
            Some(exp) => {
                self.map[idx] = Some(exp.clone());

                let mut exp = exp.borrow_mut();
                if exp.end == lhs {
                    exp.end = idx;
                }
            }
            None => {
                let exp = Rc::new(RefCell::new(Expression {
                    start: idx,
                    end: idx,
                }));
                self.map[idx] = Some(exp);
            }
        }

        // let mut expression = expression.borrow_mut();

        // match *expression {
        //     Some(ref mut expression) => {
        //         // if the current expression end is the lhs, append to the expression
        //         if expression.end == lhs {
        //             expression.end = idx;
        //         }
        //     }
        //     None => {
        //         let exp = Expression {
        //             start: idx,
        //             end: idx,
        //         };
        //         info!("expression start: {:?}", exp);
        //         *expression = Some(exp);
        //         info!("expression: {:?}", expression);
        //     }
        // };

        // match self.map.get(lhs) {
        //     Some(expression) => {
        //         expression.borrow_mut().end = idx;
        //         self.map[idx] = expression.clone();
        //     }
        //     None => {
        //         let exp = Rc::new(RefCell::new(Expression {
        //             start: idx,
        //             end: idx,
        //         }));
        //         self.map.insert(idx, exp);
        //     }
        // }
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

// let mut expression_start_idx: Option<usize> = None;
// The literal source string for evaluation
// let mut source_start_idx: Option<usize> = None;

// // iterate each byte in self.src.as_bytes
// self.src.as_bytes().iter().for_each(|b| {
// info!("b: {}", b);
// });

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
    fn test_map_expression_with_break() {
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
        assert_eq!(source.map[0], source.map[2]);

        // assert_eq!(source.map[0], source.map[3]);
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
