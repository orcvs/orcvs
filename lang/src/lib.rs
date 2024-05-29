mod eval;
mod parser;

use lazy_static::lazy_static;
use std::collections::HashMap;

use std::fmt;
use std::sync::Once;

use thiserror::Error;
// use miette::diagnostic;
// use miette::Diagnostic;

pub use eval::eval;
pub use parser::parse;

///
/// play channel octave note velocity
/// p 11 C4 100
///
///  pass function as param
///  x y 1 2 3 4
///  x (y 1 2 3) 4
///

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Atom {
    Function(Box<Function>),
    Num(u8),
    Note(u8),
    String(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Function {
    Add(Atom, Atom),
    Ident(Atom),
    Play(Atom, Atom, Atom),
    Sub(Atom, Atom),
}

impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Atom::String(s.to_owned())
    }
}

impl From<Function> for Atom {
    fn from(f: Function) -> Self {
        Atom::Function(Box::new(f))
    }
}

impl From<u8> for Atom {
    fn from(n: u8) -> Self {
        Atom::Num(n)
    }
}

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

    #[error("expected token")]
    ExpectedToken {},
}

#[derive(Error, Debug)]
pub enum ArgumentError {
    #[error("invalid number of arguments (expected {expected:?}, found {found:?})")]
    // #[diagnostic(code(ArgumentError))]
    Arity { expected: usize, found: usize },

    #[error("{0:?} should be a number")]
    NumberExpected(String),

    #[error("{0:?} should be a string")]
    StringExpected(String),

    #[error("{0:?} should be a note")]
    NoteExpected(String),
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

// #[diagnostic(code(ArgumentError), url("https://my_website.com/error"))]

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::Add(a, b) => {
                write!(f, "Add {a} {b}")
            }
            Function::Sub(a, b) => {
                write!(f, "Sub {a} {b}")
            }
            _ => write!(f, "Function"),
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atom::Num(n) => {
                write!(f, "{n}")
            }
            Atom::Note(n) => match MIDI_NUMBER_TO_NOTE.get(n) {
                Some(note) => write!(f, "{note}"),
                None => write!(f, "{n}"),
            },
            Atom::String(ref s) => write!(f, "{s}"),
            Atom::Function(ref fun) => write!(f, "{fun}"),
        }
    }
}

lazy_static! {
    static ref MIDI_NOTE_TO_NUMBER: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();
        m.insert("A0", 21);
        m.insert("a0", 22);
        m.insert("B0", 23);
        m.insert("C1", 24);
        m.insert("c1", 25);
        m.insert("D1", 26);
        m.insert("d1", 27);
        m.insert("E1", 28);
        m.insert("F1", 29);
        m.insert("f1", 30);
        m.insert("G1", 31);
        m.insert("g1", 32);
        m.insert("A1", 33);
        m.insert("a1", 34);
        m.insert("B1", 35);
        m.insert("C2", 36);
        m.insert("c2", 37);
        m.insert("D2", 38);
        m.insert("d2", 39);
        m.insert("E2", 40);
        m.insert("F2", 41);
        m.insert("f2", 42);
        m.insert("G2", 43);
        m.insert("g2", 44);
        m.insert("A2", 45);
        m.insert("a2", 46);
        m.insert("B2", 47);
        m.insert("C3", 48);
        m.insert("c3", 49);
        m.insert("D3", 50);
        m.insert("d3", 51);
        m.insert("E3", 52);
        m.insert("F3", 53);
        m.insert("f3", 54);
        m.insert("G3", 55);
        m.insert("g3", 56);
        m.insert("A3", 57);
        m.insert("a3", 58);
        m.insert("B3", 59);
        m.insert("C4", 60);
        m.insert("c4", 61);
        m.insert("D4", 62);
        m.insert("d4", 63);
        m.insert("E4", 64);
        m.insert("F4", 65);
        m.insert("f4", 66);
        m.insert("G4", 67);
        m.insert("g4", 68);
        m.insert("A4", 69);
        m.insert("a4", 70);
        m.insert("B4", 71);
        m.insert("C5", 72);
        m.insert("c5", 73);
        m.insert("D5", 74);
        m.insert("d5", 75);
        m.insert("E5", 76);
        m.insert("F5", 77);
        m.insert("f5", 78);
        m.insert("G5", 79);
        m.insert("g5", 80);
        m.insert("A5", 81);
        m.insert("a5", 82);
        m.insert("B5", 83);
        m.insert("C6", 84);
        m.insert("c6", 85);
        m.insert("D6", 86);
        m.insert("d6", 87);
        m.insert("E6", 88);
        m.insert("F6", 89);
        m.insert("d6", 90);
        m.insert("G6", 91);
        m.insert("g6", 92);
        m.insert("A6", 93);
        m.insert("a6", 94);
        m.insert("B6", 95);
        m.insert("C7", 96);
        m.insert("c7", 97);
        m.insert("D7", 98);
        m.insert("d7", 99);
        m.insert("E7", 100);
        m.insert("F7", 101);
        m.insert("f7", 102);
        m.insert("G7", 103);
        m.insert("g7", 104);
        m.insert("A7", 105);
        m.insert("a7", 106);
        m.insert("B7", 107);
        m.insert("C8", 108);
        m.insert("c8", 109);
        m.insert("D8", 110);
        m.insert("d8", 111);
        m.insert("E8", 112);
        m.insert("F8", 113);
        m.insert("f8", 114);
        m.insert("G8", 115);
        m.insert("g8", 116);
        m.insert("A8", 117);
        m.insert("a8", 118);
        m.insert("B8", 119);
        m.insert("C9", 120);
        m.insert("c9", 121);
        m.insert("D9", 122);
        m.insert("d9", 123);
        m.insert("E9", 124);
        m.insert("F9", 125);
        m.insert("f9", 126);
        m.insert("G9", 127);
        m
    };
}

lazy_static! {
    static ref MIDI_NUMBER_TO_NOTE: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(21, "A0");
        m.insert(22, "a0");
        m.insert(23, "B0");
        m.insert(24, "C1");
        m.insert(25, "c1");
        m.insert(26, "D1");
        m.insert(27, "d1");
        m.insert(28, "E1");
        m.insert(29, "F1");
        m.insert(30, "f1");
        m.insert(31, "G1");
        m.insert(32, "g1");
        m.insert(33, "A1");
        m.insert(34, "a1");
        m.insert(35, "B1");
        m.insert(36, "C2");
        m.insert(37, "c2");
        m.insert(38, "D2");
        m.insert(39, "d2");
        m.insert(40, "E2");
        m.insert(41, "F2");
        m.insert(42, "f2");
        m.insert(43, "G2");
        m.insert(44, "g2");
        m.insert(45, "A2");
        m.insert(46, "a2");
        m.insert(47, "B2");
        m.insert(48, "C3");
        m.insert(49, "c3");
        m.insert(50, "D3");
        m.insert(51, "d3");
        m.insert(52, "E3");
        m.insert(53, "F3");
        m.insert(54, "f3");
        m.insert(55, "G3");
        m.insert(56, "g3");
        m.insert(57, "A3");
        m.insert(58, "a3");
        m.insert(59, "B3");
        m.insert(60, "C4");
        m.insert(61, "c4");
        m.insert(62, "D4");
        m.insert(63, "d4");
        m.insert(64, "E4");
        m.insert(65, "F4");
        m.insert(66, "f4");
        m.insert(67, "G4");
        m.insert(68, "g4");
        m.insert(69, "A4");
        m.insert(70, "a4");
        m.insert(71, "B4");
        m.insert(72, "C5");
        m.insert(73, "c5");
        m.insert(74, "D5");
        m.insert(75, "d5");
        m.insert(76, "E5");
        m.insert(77, "F5");
        m.insert(78, "f5");
        m.insert(79, "G5");
        m.insert(80, "g5");
        m.insert(81, "A5");
        m.insert(82, "a5");
        m.insert(83, "B5");
        m.insert(84, "C6");
        m.insert(85, "c6");
        m.insert(86, "D6");
        m.insert(87, "d6");
        m.insert(88, "E6");
        m.insert(89, "F6");
        m.insert(90, "d6");
        m.insert(91, "G6");
        m.insert(92, "g6");
        m.insert(93, "A6");
        m.insert(94, "a6");
        m.insert(95, "B6");
        m.insert(96, "C7");
        m.insert(97, "c7");
        m.insert(98, "D7");
        m.insert(99, "d7");
        m.insert(100, "E7");
        m.insert(101, "F7");
        m.insert(102, "f7");
        m.insert(103, "G7");
        m.insert(104, "g7");
        m.insert(105, "A7");
        m.insert(106, "a7");
        m.insert(107, "B7");
        m.insert(108, "C8");
        m.insert(109, "c8");
        m.insert(110, "D8");
        m.insert(111, "d8");
        m.insert(112, "E8");
        m.insert(113, "F8");
        m.insert(114, "f8");
        m.insert(115, "G8");
        m.insert(116, "g8");
        m.insert(117, "A8");
        m.insert(118, "a8");
        m.insert(119, "B8");
        m.insert(120, "C9");
        m.insert(121, "c9");
        m.insert(122, "D9");
        m.insert(123, "d9");
        m.insert(124, "E9");
        m.insert(125, "F9");
        m.insert(126, "f9");
        m.insert(127, "G9");
        m
    };
}
