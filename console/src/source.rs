use std::cell::{Ref, RefCell};
use std::rc::Rc;

use arrayvec::ArrayVec;
use lang::{Atom, Function, Parser, Stack};
use tracing::info;

const TERMINATOR: &str = ".";

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Source {
    inner: String,
    cols: usize,
    rows: usize,
    map: Vec<Option<Rc<RefCell<Expression>>>>,
    expressions: Vec<Rc<RefCell<Expression>>>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Expression {
    start: usize,
    end: usize,

    #[serde(skip_deserializing, skip_serializing)]
    stack: Option<Stack>,

    #[serde(skip_deserializing, skip_serializing)]
    valid: Option<bool>,

    function: Option<Function>,
}

impl Expression {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            stack: None,
            valid: None,
            function: None,
        }
    }

    pub fn to_map(&self) -> Map {
        let mut map: Map = ArrayVec::new();

        if let Some(stack) = &self.stack {
            for (idx, exp) in stack.into_iter().rev().enumerate() {
                info!("====================");
                info!("{}: {:?}", idx, exp);
                info!("map {:?}", map);

                match exp {
                    Atom::Function(f) => {
                        let f_map = get_function_map(&f);

                        info!("----------------------");
                        info!("f_map {:?}", f_map);

                        // First function
                        if map.is_empty() {
                            map.extend(f_map.into_iter());
                        } else {
                            info!("idx {:?}", idx);

                            // remove the element at idx
                            map.pop_at(idx);
                            info!("map {:?}", map);

                            let mut pos = idx;

                            // Insert the function map at idx
                            for glyph in f_map.into_iter() {
                                map.insert(pos, glyph);
                                pos += 1;
                            }

                            // [Function, Number, Function, Number, Number]
                            // info!("count {:?}", count);
                            info!("map {:?}", map);
                            // Remove the element at idx +
                        }
                    }
                    _ => {}
                }
            }
        }
        map
    }
}

macro_rules! map_from {
    ($($items:tt),*) => {
        {
            let mut ary: Map = ArrayVec::new();
            $(
                for item in $items.iter() {
                    ary.push(*item);
                }
            )*
            ary
        }
    };
}

#[must_use]
fn get_function_map(f: &Function) -> Map {
    match f {
        Function::Add => map_from!([G::Function, G::Number, G::Number]),
        Function::Id => map_from!([G::Function, G::Number]),
        _ => map_from!(([])),
    }
}

impl<'a> Source {
    pub fn new(cols: usize, rows: usize) -> Self {
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

    fn map(&mut self) {
        for exp in self.expressions.iter() {
            let s = self.get_expression(exp.borrow());
            let mut s = String::from(s);

            let mut parser = Parser::new(&mut s);
            let result = parser.parse();

            match result {
                Ok(valid) => {
                    let mut exp = exp.borrow_mut();
                    // let f = parser.stack.last();
                    // if let Some(f) = f {
                    // exp.function = Some(f.into());
                    // }
                    exp.stack = Some(parser.stack);
                    exp.valid = Some(valid);
                }
                Err(e) => {
                    info!("error: {:?}", e);
                }
            }
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

    pub fn get_at(&self, x: usize, y: usize) -> &str {
        let idx = self.to_idx(x, y);
        // SAFELY UNSAFE
        // all characters are single-byte ASCII
        //   the idx is always in range
        //      - to_index will panic if the index is out of bounds
        unsafe { self.inner.get_unchecked(idx..=idx) }
    }

    pub fn get_expression(&self, exp: Ref<'_, Expression>) -> &str {
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
        unsafe { self.inner.get_unchecked(start..=end) }
    }

    pub fn set_at(&mut self, x: usize, y: usize, s: &str) {
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
                        let exp = Rc::new(RefCell::new(Expression::new(rgt_idx, rgt.end)));
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
                    let exp = Rc::new(RefCell::new(Expression::new(idx, idx)));
                    self.expressions.push(exp.clone());
                    self.map[idx] = Some(exp.clone());
                }
            }
        }
    }

    ///
    /// Convert x, y coordinates to a linear index
    /// panic if the index is out of bounds
    ///
    pub fn to_idx(&self, x: usize, y: usize) -> usize {
        let idx = y * self.cols + x;
        assert!(idx <= self.len(), "index out of bounds {idx} for [{x},{y}]");
        idx
    }

    ///
    /// Current length of the source
    ///
    fn len(&self) -> usize {
        self.cols * self.rows
    }

    fn print_exp(&self) {
        self.expressions.iter().for_each(|m| {
            info!("map: {:?}", m);
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Glyph {
    Function,
    Note,
    Number,
    String,
    FunctionN(u8),
    NoteN(u8),
    NumberN(u8),
    StringN(u8),
}

type G = Glyph;

// type InnerArrayVec<T, const N: usize> = ArrayVec<T, N>;
// type OuterArrayVec<T, const N: usize, const M: usize> = ArrayVec<InnerArrayVec<T, N>, M>;

// type GlyphMap = ArrayVec<G, 32>;

type Map = ArrayVec<G, 32>;

struct FunctionMap {
    map: Map,
    parameters: usize,
}

impl FunctionMap {
    fn new(map: Map, parameters: usize) -> Self {
        Self { map, parameters }
    }
}

// type Map = ArrayVec<ArrayVec<G, 32>, 32>;
// type NestedArrayVec<T, const N: usize> = ArrayVec<NestedArrayVec<T, N - 1>, N>;
// type NestedArrayVec<T, 0> = ArrayVec<T, N>;

#[cfg(test)]
mod test {
    use super::Source;
    use crate::source::get_function_map;
    use crate::source::Expression;
    use crate::source::Map;

    use crate::source::{Glyph, G};
    use arrayvec::ArrayVec;
    use lang::Parser;
    use lang::{Atom, Function, Stack};
    use std::sync::Once;
    use tracing::info;

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

        assert_eq!(source.inner, expected);

        source
    }

    #[test]
    fn _test_expression_map_alt() {
        trace();

        let array: &[Atom] = &[
            Atom::Empty,
            Atom::Function(Function::Id),
            Atom::Empty,
            Atom::Function(Function::Add),
        ];

        // co0
        // for exp in stack.pop
        //    Empty => positions += 1
        //    Empty => positions += 1
        //
        //    Function::Id
        //      [G::Function, G::Number]
        //      positions -= 1
        //
        //   Function::Add
        //      [G::Function, G::Number, G::Number]
        //
        //
        //   Prepend function
        //      [G::Number, G::Number]
        //      [G::Function, G::Function, G::Number]
        //
        //   Append parameters
        //
        //

        let mut stack: Stack = Stack::from(array);

        let mut map: Map = ArrayVec::new();

        info!("map: {:?}", map);

        let mut count = 0;
        for exp in stack.into_iter() {
            info!("--------------------------");
            info!("exp: {:?}", exp);

            info!("count: {:?}", count);

            match exp {
                Atom::Function(f) => {
                    let mut f_map = get_function_map(&f);

                    count += 1;

                    info!("f_map: {:?}", f_map);

                    if map.is_empty() {
                        map.extend(f_map.into_iter());
                    } else {
                        // Wrap the function
                        let f = f_map.remove(0);
                        map.insert(0, f);

                        for x in 1..=count {
                            info!("x: {:?}", x);
                        }

                        // if let Some(f) = f_map.first() {}
                        // if let Some((first, rest)) = f_map.split_first_mut() {
                        //     //     info!("first: {:?}", first);
                        //     //     info!("rest: {:?}", rest);

                        //     //     map.insert(0, *first);

                        //     //     assert!(count <= rest.len());

                        //     //     // map.extend(rest[..count].copy_from_slice(src));
                        // }
                    }

                    // Decrement by the number of fgunction parameters
                    match f {
                        Function::Id => {
                            count -= 1;
                        }
                        Function::Add => {
                            count -= 2;
                        }
                        _ => {}
                    };
                    info!("count: {:?}", count);

                    // if arity <= positions {
                    //     map.extend(f_map.into_iter());
                    // } else {
                    //     // Wrap the function
                    //     let f = f_map.pop();

                    //     map.extend(f_map.into_iter());
                    // }
                }
                _ => {}
            }
        }
        info!("positions: {:?}", count);
        info!("map: {:?}", map);
        // let expected: Map = map_from!([G::Function, G::Function, G::Number, G::Number]);
        // assert_eq!(map, expected);
    }

    #[test]
    fn test_expression_map_one() {
        trace();

        // let array: &[Atom] = &[
        //     Atom::Empty,
        //     Atom::Empty,
        //     Atom::Function(Function::Id),
        //     Atom::Function(Function::Add),
        // ];

        // let mut stack: Stack = Stack::from(array);

        // let source = source_from("++id0A0A");
        let source = source_from("++id0A0A");
        // source.map();

        let exp = source.expressions.first().unwrap();
        info!("exp: {:?}", exp);

        let s = source.get_expression(exp.borrow());
        info!("s: {:?}", s);

        let mut s = String::from(s);

        let mut parser = Parser::new(&mut s);
        let result = parser.parse();

        info!("stack: {:?}", parser.stack);

        let mut exp = exp.borrow_mut();
        exp.stack = Some(parser.stack);

        // let map = exp.to_map();
        // info!(" map: {:?}", map);
        let mut map = Map::new();
        if let Some(stack) = &exp.stack {
            for (idx, exp) in stack.into_iter().rev().enumerate() {
                info!("====================");
                info!("{}: {:?}", idx, exp);
                info!("map {:?}", map);

                match exp {
                    Atom::Function(f) => {
                        let f_map = get_function_map(&f);

                        info!("----------------------");
                        info!("f_map {:?}", f_map);

                        // First function
                        if map.is_empty() {
                            map.extend(f_map.into_iter());
                        } else {
                            info!("idx {:?}", idx);

                            // let l = f_map.len();

                            // remove the element at idx
                            map.pop_at(idx);
                            info!("map {:?}", map);

                            let mut pos = idx;

                            // Insert the function map at idx
                            for glyph in f_map.into_iter() {
                                map.insert(pos, glyph);
                                pos += 1;
                            }

                            // [Function, Number, Function, Number, Number]

                            // info!("count {:?}", count);
                            info!("map {:?}", map);
                            // Remove the element at idx +
                        }
                    }
                    _ => {}
                }
            }
        }

        // let expected: Map = map_from!([G::Function, G::Number, G::Function, G::Number]);
        // assert_eq!(map, expected);

        let expected: Map = map_from!([G::Function, G::Function, G::Number, G::Number]);
        assert_eq!(map, expected);

        info!("map {:?}", map);

        // let map = exp.borrow().to_map();

        // info!("map: {:?}", map);

        // let mut map: Map = ArrayVec::new();

        // info!("map: {:?}", map);

        // for (idx, exp) in stack.into_iter().enumerate() {
        //     info!("{}: {:?}", idx, exp);

        //     match exp {
        //         Atom::Function(f) => {
        //             let f_map = get_function_map(&f);

        //             // First function
        //             if idx == 0 {
        //                 map.extend(f_map.into_iter());
        //             } else {
        //                 map.pop_at(idx);

        //                 for (inner_idx, glyph) in f_map.into_iter().enumerate() {
        //                     let pos = idx + inner_idx;
        //                     map.insert(pos, glyph);
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // }
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

        assert_eq!(source.expressions.len(), 0);
    }

    #[test]
    fn test_get_expression_as_str() {
        trace();

        let mut source = Source::new(10, 1);

        source.set_at(0, 0, "i");
        source.set_at(1, 0, "d");
        source.set_at(2, 0, "0");
        source.set_at(3, 0, "A");
        assert_eq!(source.inner, "id0A......");

        source.print_exp();

        let exp = source.expressions.first().unwrap();
        let exp = exp.borrow();

        let s = source.get_expression(exp);

        assert_eq!(s, "id0A");
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
    #[should_panic(expected = "index out of bounds")]
    fn test_to_idx_out_of_bounds() {
        let source = Source::new(10, 10);
        let _idx = source.to_idx(11, 11); // This should panic
    }

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
}
