mod eval;
mod parser;

use arrayvec::ArrayVec;
use lazy_static::lazy_static;
use std::collections::HashMap;

use std::fmt;
use std::fmt::Debug;
use std::sync::Once;
use thiserror::Error;
// use miette::diagnostic;
// use miette::Diagnostic;

pub use parser::Parser;

///
/// play channel octave note velocity
/// p 11 C4 100
///
///  pass function as param
///  x y 1 2 3 4
///  x (y 1 2 3) 4
///

// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub struct AtomRef(usize);

// impl From Atom for String
impl From<Atom> for String {
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

// type VecStack = Vec<Atom>;

// use smallvec::{smallvec, SmallVec};
// type VecStack = SmallVec<[Atom; 32]>;
// use arrayvec::ArrayVec;
type VecStack = ArrayVec<Atom, 48>;

fn new_vec() -> VecStack {
    ArrayVec::new()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
    Function(Function),
    Note(u8),
    Number(u8),
    String(String),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Add,
    Ident,
    Play,
    Sub,
    Empty,
}

impl From<Function> for Atom {
    fn from(f: Function) -> Self {
        Atom::Function(f)
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
            Function::Add => write!(f, "add()"),
            Function::Ident => write!(f, "ident()"),
            Function::Play => write!(f, "play()"),
            Function::Sub => write!(f, "sub()"),
            Function::Empty => write!(f, "_()"),
        }
    }
}

// impl<T: AtomTrait> fmt::Display for AtomRef<T> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.index)
//     }
// }

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

lazy_static! {
    pub static ref MIDI_NOTE_TO_NUMBER: HashMap<&'static str, u8> = {
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
        m.insert("f6", 90);
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
    pub static ref MIDI_NUMBER_TO_NOTE: HashMap<u8, &'static str> = {
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

pub fn midi_note_to_number_hash(note: &str) -> Option<&u8> {
    MIDI_NOTE_TO_NUMBER.get(note)
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
