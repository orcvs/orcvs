mod ast;
mod parser;

use std::fmt;

use miette::diagnostic;
use miette::Diagnostic;
use thiserror::Error;
use tinyvec::ArrayVec;

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
    pub channel: i32,
    pub velocity: i32,
    pub note: String,
}

impl Play {
    pub fn new(channel: i32, note: String, velocity: i32) -> Play {
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
    Function(Function),
    Char(char),
    Hex(char),
    Num(i32),
    List(Vec<Atom>),
    String(String),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Function {
    Play,
    Take,
    End,
    X,
    Y,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Function::Play => write!(f, "Play"),
            Function::Take => write!(f, "Take"),
            Function::End => write!(f, "End"),
            Function::X => write!(f, "X"),
            Function::Y => write!(f, "Y"),
        }
    }
}

// fn _play(ch: u8, oct: u8, note: &String, vel: u8) -> Result<(), VthaError> {

#[derive(Error, Diagnostic, Debug)]
pub enum VthaError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ArgumentError(#[from] ArgumentError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    SyntaxError(#[from] SyntaxError),

    #[error("{0:?}")]
    EvalError(String),
}

#[derive(Error, Diagnostic, Debug)]
pub enum SyntaxError {
    #[error("unknown function {f:?}")]
    #[diagnostic(code(SyntaxError))]
    UnknownFunction { f: String },
}

#[derive(Error, Diagnostic, Debug)]
pub enum ArgumentError {
    #[error("invalid number of arguments (expected {expected:?}, found {found:?})")]
    #[diagnostic(code(ArgumentError))]
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
