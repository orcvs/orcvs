mod ast;
mod parser;

// use std::fmt;

// use miette::diagnostic;
// use miette::Diagnostic;
use thiserror::Error;

///
/// play channel octave note velocity
/// p 11 C4 100
///
///  pass function as param
///  x y 1 2 3 4
///  x (y 1 2 3) 4
///

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Play {
    pub channel: u8,
    pub velocity: u8,
    pub note: String,
}

impl Play {
    pub fn new(
        channel: u8,
        note: String,
        velocity: u8,
    ) -> Play {
        Play {
            channel,
            note,
            velocity,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expression {
    Function(Function, Atom),
    Play(Play),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Atom {
    Function(Box<Function>),
    Char(char),
    Hex(char),
    Num(u8),
    List(Vec<Atom>),
    String(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Function {
    Play(Atom, Atom, Atom),
    Take,
    End,
    Ident(Atom),
    X,
    Y,
}

// impl fmt::Display for Function {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             Function::Play(a, b, c) => {
//                 write!(f, "Play {} {} {}", a, b, c)
//             }
//             Function::Take => write!(f, "Take"),
//             Function::End => write!(f, "End"),
//             Function::Ident(x) => write!(f, "Ident {}", x),
//             Function::X => write!(f, "X"),
//             Function::Y => write!(f, "Y"),
//         }
//     }
// }

// impl fmt::Display for Atom {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             Atom::Num() => {
//                 write!(f, "Play {} {} {}", a, b, c)
//             }

//         }
//     }
// }

// fn _play(ch: u8, oct: u8, note: &String, vel: u8) -> Result<(), VthaError> {

#[derive(Error, Debug)]
pub enum VthaError {
    #[error(transparent)]
    // #[diagnostic(transparent)]
    ArgumentError(#[from] ArgumentError),

    #[error(transparent)]
    // #[diagnostic(transparent)]
    SyntaxError(#[from] SyntaxError),

    #[error("{0:?}")]
    EvalError(String),
}

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("unknown function {f:?}")]
    // #[diagnostic(code(SyntaxError))]
    UnknownFunction { f: String },
}

#[derive(Error, Debug)]
pub enum ArgumentError {
    #[error("invalid number of arguments (expected {expected:?}, found {found:?})")]
    // #[diagnostic(code(ArgumentError))]
    Arity { expected: usize, found: usize },

    #[error("{0:?} should be a number")]
    NumberExpected(&'static str),

    #[error("{0:?} should be a string")]
    StringExpected(&'static str),
}
// #[diagnostic(code(ArgumentError), url("https://my_website.com/error"))]

#[macro_export]
macro_rules! list {
    ($($x:expr),*) => {
        Atom::List(vec![$($x),*])
        // Atom::List(array_vec![$($x),*])
    };
}

#[macro_export]
macro_rules! empty_list {
    () => {
        Atom::List(vec![])
    };
}
