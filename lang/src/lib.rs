mod interpreter;
mod parser;

pub use interpreter::Interpreter;
pub use parser::Parser;

use arrayvec::ArrayVec;
use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::Once;
use thiserror::Error;

pub struct MaybeAtom(Option<Atom>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Note,
    Number,
    String,
    // Number(u8),
}
pub type T = Token;

pub type Tokens = ArrayVec<Token, 8>;

macro_rules! array_vec {
    ($($items:tt),*) => {
        {
            let mut ary = ArrayVec::new();
            $(
                for item in $items.iter() {
                    ary.push(*item);
                }
            )*
            ary
        }
    };
}

impl From<&Function> for Tokens {
    #[inline(always)]
    fn from(f: &Function) -> Self {
        match f {
            Function::Add => array_vec!([T::Number, T::Number]),
            Function::Divide => array_vec!([T::Number, T::Number]),
            Function::Id => array_vec!([T::String]),
            Function::Multiply => array_vec!([T::Number, T::Number]),
            Function::Subtract => array_vec!([T::Number, T::Number]),
            _ => array_vec!(([])),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Stack<const N: usize> {
    pub(crate) inner: ArrayVec<Atom, N>,
}

impl<const N: usize> Stack<N> {
    pub fn new() -> Self {
        Self {
            inner: ArrayVec::new(),
        }
    }

    pub fn new_with(array_vec: ArrayVec<Atom, N>) -> Self {
        Self { inner: array_vec }
    }

    #[inline(always)]
    pub fn push(&mut self, atom: Atom) {
        self.inner.push(atom);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> MaybeAtom {
        MaybeAtom(self.inner.pop())
    }

    #[inline(always)]
    pub fn peek(&self) -> Option<&Atom> {
        self.inner.last()
    }

    #[inline(always)]
    pub fn try_pop<T: TryFrom<MaybeAtom, Error = Error>>(
        &mut self,
        expected: usize,
        count: usize,
    ) -> Result<T, Error> {
        self.pop()
            .try_into()
            .map_err(|err| map_arity(err, expected, count))
    }
}

impl<const N: usize> From<&[Atom]> for Stack<N> {
    fn from(array: &[Atom]) -> Self {
        let mut inner: ArrayVec<Atom, N> = ArrayVec::new();
        for item in array.into_iter() {
            inner.push(item.clone());
        }

        Stack { inner }
    }
}

impl<const N: usize> Deref for Stack<N> {
    type Target = [Atom];

    fn deref(&self) -> &[Atom] {
        &self.inner[..]
    }
}

impl<const N: usize> DerefMut for Stack<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[inline(always)]
pub fn to_atom_string(s: &str) -> Result<Atom, Error> {
    let a = Atom::String(s.to_string());
    Ok(a)
}

#[inline(always)]
pub fn to_atom_note(s: &str) -> Result<Atom, Error> {
    match midi_note_to_number(s) {
        Some(n) => {
            let a = Atom::Note(n);
            Ok(a)
        }
        None => Err(TypeError::Note(s.to_string()).into()),
    }
}

#[inline(always)]
pub fn to_atom_num(s: &str) -> Result<Atom, Error> {
    let n = str_to_num(&s)?;
    Ok(Atom::Number(n))
}

#[inline(always)]
fn str_to_num(s: &str) -> Result<u8, Error> {
    match u8::from_str_radix(&s, 16) {
        Ok(n) => Ok(n),
        Err(_) => Err(TypeError::Number(s.to_string()).into()),
    }
}

impl From<MaybeAtom> for Atom {
    #[inline(always)]
    fn from(maybe_atom: MaybeAtom) -> Self {
        match maybe_atom.0 {
            Some(a) => a,
            None => Atom::Empty,
        }
    }
}

impl TryFrom<MaybeAtom> for u8 {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(Atom::Number(n) | Atom::Note(n)) => Ok(n),
            Some(Atom::String(s)) => str_to_num(&s),
            Some(atom) => Err(TypeError::Number(atom.into()).into()),
            None => Err(ArgumentError::Expected.into()),
        }
    }
}

impl TryFrom<MaybeAtom> for String {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(a) => Ok(a.into()),
            None => Err(ArgumentError::Expected.into()),
        }
    }
}

impl TryFrom<MaybeAtom> for Function {
    type Error = Error;

    #[inline(always)]
    fn try_from(maybe_atom: MaybeAtom) -> Result<Self, Self::Error> {
        match maybe_atom.0 {
            Some(Atom::Function(f)) => Ok(f),
            _ => Err(ArgumentError::Expected.into()),
        }
    }
}

impl TryFrom<&str> for Function {
    type Error = Error;

    #[inline(always)]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "++" => Ok(Function::Add),
            "//" => Ok(Function::Divide),
            "**" => Ok(Function::Multiply),
            "--" => Ok(Function::Subtract),
            "id" => Ok(Function::Id),
            s => Err(SyntaxError::UnknownFunction(s.to_string()).into()),
        }
    }
}

impl From<Atom> for String {
    #[inline(always)]
    fn from(atom: Atom) -> Self {
        match atom {
            Atom::Number(n) => n.to_string(),
            Atom::Note(n) => match midi_number_to_note(n) {
                Some(note) => note.to_string(),
                None => n.to_string(),
            },
            Atom::String(s) => s.to_owned(),
            Atom::Function(fun) => format!("{}", fun),
            Atom::Empty => "_".to_string(),
        }
    }
}

// impl From<MaybeAtom> for Function {
//     #[inline(always)]
//     fn from(maybe_atom: MaybeAtom) -> Self {
//         match maybe_atom.0 {
//             Some(Atom::Function(f)) => f,
//             _ => Function::Empty,
//         }
//     }
// }

// impl From<Option<&Atom>> for Atom {
//     #[inline(always)]
//     fn from(atom: Option<&Atom>) -> Self {
//         match atom {
//             Some(a) => a.clone(),
//             None => Atom::Empty,
//         }
//     }
// }

// #[inline(always)]
fn map_arity(err: Error, expected: usize, found: usize) -> Error {
    match err {
        Error::Argument(ArgumentError::Expected) => ArgumentError::Arity { expected, found }.into(),
        _ => err.into(),
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Atom {
    Empty,
    Function(Function),
    Note(u8),
    Number(u8),
    String(String),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Function {
    Add,
    Divide,
    Empty,
    Id,
    Multiply,
    Play,
    Subtract,
}

impl From<Function> for Atom {
    fn from(f: Function) -> Self {
        Atom::Function(f)
    }
}

impl From<&Atom> for Function {
    fn from(a: &Atom) -> Self {
        match a {
            Atom::Function(f) => f.clone(),
            _ => panic!("expected a function"),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Argument(#[from] ArgumentError),

    #[error(transparent)]
    Syntax(#[from] SyntaxError),

    #[error(transparent)]
    Type(#[from] TypeError),
}

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("expected a function, found {0:?}")]
    Function(String),

    #[error("expected a note, found {0:?}")]
    Note(String),

    #[error("expected a number, found {0:?}")]
    Number(String),

    #[error("expected a string, found {0:?}")]
    String(String),
}

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("expected a function")]
    ExpectedFunction,

    #[error("expected a token")]
    ExpectedToken,

    #[error("unknown function {0:?}")]
    UnknownFunction(String),
}

#[derive(Error, Debug)]
pub enum ArgumentError {
    #[error("invalid number of arguments (expected {expected:?}, found {found:?})")]
    // #[diagnostic(code(ArgumentError))]
    Arity { expected: usize, found: usize },

    #[error("expected an argument")]
    Expected,

    #[error("expected a function")]
    ExpectedFunction,
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

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::Add => write!(f, "++"),
            Function::Empty => write!(f, "__"),
            Function::Divide => write!(f, "//"),
            Function::Id => write!(f, "id"),
            Function::Multiply => write!(f, "**"),
            Function::Play => write!(f, "pl"),
            Function::Subtract => write!(f, "--"),
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atom::Number(n) => {
                write!(f, "{n}")
            }
            Atom::Note(n) => match midi_number_to_note(*n) {
                Some(note) => write!(f, "{note}"),
                None => write!(f, "{n}"),
            },
            Atom::String(ref s) => write!(f, "{s}"),
            Atom::Function(ref fun) => write!(f, "{fun}"),
            Atom::Empty => write!(f, "_"),
        }
    }
}

#[must_use]
pub fn midi_note_to_number(note: &str) -> Option<u8> {
    match note {
        "A0" => Some(21),
        "a0" => Some(22),
        "B0" => Some(23),
        "C1" => Some(24),
        "c1" => Some(25),
        "D1" => Some(26),
        "d1" => Some(27),
        "E1" => Some(28),
        "F1" => Some(29),
        "f1" => Some(30),
        "G1" => Some(31),
        "g1" => Some(32),
        "A1" => Some(33),
        "a1" => Some(34),
        "B1" => Some(35),
        "C2" => Some(36),
        "c2" => Some(37),
        "D2" => Some(38),
        "d2" => Some(39),
        "E2" => Some(40),
        "F2" => Some(41),
        "f2" => Some(42),
        "G2" => Some(43),
        "g2" => Some(44),
        "A2" => Some(45),
        "a2" => Some(46),
        "B2" => Some(47),
        "C3" => Some(48),
        "c3" => Some(49),
        "D3" => Some(50),
        "d3" => Some(51),
        "E3" => Some(52),
        "F3" => Some(53),
        "f3" => Some(54),
        "G3" => Some(55),
        "g3" => Some(56),
        "A3" => Some(57),
        "a3" => Some(58),
        "B3" => Some(59),
        "C4" => Some(60),
        "c4" => Some(61),
        "D4" => Some(62),
        "d4" => Some(63),
        "E4" => Some(64),
        "F4" => Some(65),
        "f4" => Some(66),
        "G4" => Some(67),
        "g4" => Some(68),
        "A4" => Some(69),
        "a4" => Some(70),
        "B4" => Some(71),
        "C5" => Some(72),
        "c5" => Some(73),
        "D5" => Some(74),
        "d5" => Some(75),
        "E5" => Some(76),
        "F5" => Some(77),
        "f5" => Some(78),
        "G5" => Some(79),
        "g5" => Some(80),
        "A5" => Some(81),
        "a5" => Some(82),
        "B5" => Some(83),
        "C6" => Some(84),
        "c6" => Some(85),
        "D6" => Some(86),
        "d6" => Some(87),
        "E6" => Some(88),
        "F6" => Some(89),
        "f6" => Some(90),
        "G6" => Some(91),
        "g6" => Some(92),
        "A6" => Some(93),
        "a6" => Some(94),
        "B6" => Some(95),
        "C7" => Some(96),
        "c7" => Some(97),
        "D7" => Some(98),
        "d7" => Some(99),
        "E7" => Some(100),
        "F7" => Some(101),
        "f7" => Some(102),
        "G7" => Some(103),
        "g7" => Some(104),
        "A7" => Some(105),
        "a7" => Some(106),
        "B7" => Some(107),
        "C8" => Some(108),
        "c8" => Some(109),
        "D8" => Some(110),
        "d8" => Some(111),
        "E8" => Some(112),
        "F8" => Some(113),
        "f8" => Some(114),
        "G8" => Some(115),
        "g8" => Some(116),
        "A8" => Some(117),
        "a8" => Some(118),
        "B8" => Some(119),
        "C9" => Some(120),
        "c9" => Some(121),
        "D9" => Some(122),
        "d9" => Some(123),
        "E9" => Some(124),
        "F9" => Some(125),
        "f9" => Some(126),
        "G9" => Some(127),
        _ => None,
    }
}

#[must_use]
fn midi_number_to_note(note: u8) -> Option<&'static str> {
    match note {
        21 => Some("A0"),
        22 => Some("a0"),
        23 => Some("B0"),
        24 => Some("C1"),
        25 => Some("c1"),
        26 => Some("D1"),
        27 => Some("d1"),
        28 => Some("E1"),
        29 => Some("F1"),
        30 => Some("f1"),
        31 => Some("G1"),
        32 => Some("g1"),
        33 => Some("A1"),
        34 => Some("a1"),
        35 => Some("B1"),
        36 => Some("C2"),
        37 => Some("c2"),
        38 => Some("D2"),
        39 => Some("d2"),
        40 => Some("E2"),
        41 => Some("F2"),
        42 => Some("f2"),
        43 => Some("G2"),
        44 => Some("g2"),
        45 => Some("A2"),
        46 => Some("a2"),
        47 => Some("B2"),
        48 => Some("C3"),
        49 => Some("c3"),
        50 => Some("D3"),
        51 => Some("d3"),
        52 => Some("E3"),
        53 => Some("F3"),
        54 => Some("f3"),
        55 => Some("G3"),
        56 => Some("g3"),
        57 => Some("A3"),
        58 => Some("a3"),
        59 => Some("B3"),
        60 => Some("C4"),
        61 => Some("c4"),
        62 => Some("D4"),
        63 => Some("d4"),
        64 => Some("E4"),
        65 => Some("F4"),
        66 => Some("f4"),
        67 => Some("G4"),
        68 => Some("g4"),
        69 => Some("A4"),
        70 => Some("a4"),
        71 => Some("B4"),
        72 => Some("C5"),
        73 => Some("c5"),
        74 => Some("D5"),
        75 => Some("d5"),
        76 => Some("E5"),
        77 => Some("F5"),
        78 => Some("f5"),
        79 => Some("G5"),
        80 => Some("g5"),
        81 => Some("A5"),
        82 => Some("a5"),
        83 => Some("B5"),
        84 => Some("C6"),
        85 => Some("c6"),
        86 => Some("D6"),
        87 => Some("d6"),
        88 => Some("E6"),
        89 => Some("F6"),
        90 => Some("f6"),
        91 => Some("G6"),
        92 => Some("g6"),
        93 => Some("A6"),
        94 => Some("a6"),
        95 => Some("B6"),
        96 => Some("C7"),
        97 => Some("c7"),
        98 => Some("D7"),
        99 => Some("d7"),
        100 => Some("E7"),
        101 => Some("F7"),
        102 => Some("f7"),
        103 => Some("G7"),
        104 => Some("g7"),
        105 => Some("A7"),
        106 => Some("a7"),
        107 => Some("B7"),
        108 => Some("C8"),
        109 => Some("c8"),
        110 => Some("D8"),
        111 => Some("d8"),
        112 => Some("E8"),
        113 => Some("F8"),
        114 => Some("f8"),
        115 => Some("G8"),
        116 => Some("g8"),
        117 => Some("A8"),
        118 => Some("a8"),
        119 => Some("B8"),
        120 => Some("C9"),
        121 => Some("c9"),
        122 => Some("D9"),
        123 => Some("d9"),
        124 => Some("E9"),
        125 => Some("F9"),
        126 => Some("f9"),
        127 => Some("G9"),
        _ => None,
    }
}
