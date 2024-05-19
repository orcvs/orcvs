mod ast;
mod parser;

// use std::fmt;

use lazy_static::lazy_static;
use std::collections::HashMap;

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

// Glyph Enum with an option for all alphanumeric characters
#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Glyph {
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    A,
    a,
    B,
    b,
    C,
    c,
    D,
    d,
    E,
    e,
    F,
    f,
    G,
    g,
    H,
    h,
    I,
    i,
    J,
    j,
    K,
    k,
    L,
    l,
    M,
    m,
    N,
    n,
    O,
    o,
    P,
    p,
    Q,
    q,
    R,
    r,
    S,
    s,
    T,
    t,
    U,
    u,
    V,
    v,
    W,
    w,
    X,
    x,
    Y,
    y,
    Z,
    z,
    Unknown,
}

// Convert a Glyphe &str to a Glyph enum
impl From<char> for Glyph {
    fn from(c: char) -> Self {
        match c {
            '0' => Glyph::_0,
            '1' => Glyph::_1,
            '2' => Glyph::_2,
            '3' => Glyph::_3,
            '4' => Glyph::_4,
            '5' => Glyph::_5,
            '6' => Glyph::_6,
            '7' => Glyph::_7,
            '8' => Glyph::_8,
            '9' => Glyph::_9,
            'A' => Glyph::A,
            'a' => Glyph::a,
            'B' => Glyph::B,
            'b' => Glyph::b,
            'C' => Glyph::C,
            'c' => Glyph::c,
            'D' => Glyph::D,
            'd' => Glyph::d,
            'E' => Glyph::E,
            'e' => Glyph::e,
            'F' => Glyph::F,
            'f' => Glyph::f,
            'G' => Glyph::G,
            'g' => Glyph::g,
            'H' => Glyph::H,
            'h' => Glyph::h,
            'I' => Glyph::I,
            'i' => Glyph::i,
            'J' => Glyph::J,
            'j' => Glyph::j,
            'K' => Glyph::K,
            'k' => Glyph::k,
            'L' => Glyph::L,
            'l' => Glyph::l,
            'M' => Glyph::M,
            'm' => Glyph::m,
            'N' => Glyph::N,
            'n' => Glyph::n,
            'O' => Glyph::O,
            'o' => Glyph::o,
            'P' => Glyph::P,
            'p' => Glyph::p,
            'Q' => Glyph::Q,
            'q' => Glyph::q,
            'R' => Glyph::R,
            'r' => Glyph::r,
            'S' => Glyph::S,
            's' => Glyph::s,
            'T' => Glyph::T,
            't' => Glyph::t,
            'U' => Glyph::U,
            'u' => Glyph::u,
            'V' => Glyph::V,
            'v' => Glyph::v,
            'W' => Glyph::W,
            'w' => Glyph::w,
            'X' => Glyph::X,
            'x' => Glyph::x,
            'Y' => Glyph::Y,
            'y' => Glyph::y,
            'Z' => Glyph::Z,
            'z' => Glyph::z,
            _ => Glyph::Unknown,
        }
    }
}

// Convert a Glyphe &str to a Glyph enum
impl From<&str> for Glyph {
    fn from(s: &str) -> Self {
        match s {
            "0" => Glyph::_0,
            "1" => Glyph::_1,
            "2" => Glyph::_2,
            "3" => Glyph::_3,
            "4" => Glyph::_4,
            "5" => Glyph::_5,
            "6" => Glyph::_6,
            "7" => Glyph::_7,
            "8" => Glyph::_8,
            "9" => Glyph::_9,
            "A" => Glyph::A,
            "B" => Glyph::B,
            "C" => Glyph::C,
            "D" => Glyph::D,
            "E" => Glyph::E,
            "F" => Glyph::F,
            "G" => Glyph::G,
            "H" => Glyph::H,
            "I" => Glyph::I,
            "J" => Glyph::J,
            "K" => Glyph::K,
            "L" => Glyph::L,
            "M" => Glyph::M,
            "N" => Glyph::N,
            "O" => Glyph::O,
            "P" => Glyph::P,
            "Q" => Glyph::Q,
            "R" => Glyph::R,
            "S" => Glyph::S,
            "T" => Glyph::T,
            "U" => Glyph::U,
            "V" => Glyph::V,
            "W" => Glyph::W,
            "X" => Glyph::X,
            "Y" => Glyph::Y,
            "Z" => Glyph::Z,
            _ => Glyph::Unknown,
        }
    }
}

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
    Note(u8),
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

lazy_static! {
    static ref MIDI_NOTES: HashMap<&'static str, u8> = {
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
