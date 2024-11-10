use arrayvec::ArrayVec;
use std::fmt;

use crate::{midi_note_to_number, midi_number_to_note, str_to_num, Error, TypeError, EXP_LEN};

pub type Atoms = ArrayVec<Atom, EXP_LEN>;

// #[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Atom {
    Char(char),
    Empty,
    Function(Function),
    Note(u8),
    Number(u8),
}

// #[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Function {
    Add,
    Divide,
    Empty,
    Id,
    Multiply,
    Play,
    Subtract,
}

#[inline(always)]
pub fn to_atom_note(s: &str) -> Result<Atom, Error> {
    match midi_note_to_number(s) {
        Some(n) => {
            let a = Atom::Note(n);
            Ok(a)
        }
        None => Err(TypeError::Note(s.to_string()))?,
    }
}

#[inline(always)]
pub fn to_atom_num(s: &str) -> Result<Atom, Error> {
    let n = str_to_num(&s)?;
    Ok(Atom::Number(n))
}

#[inline(always)]
pub fn to_atom_char(s: &str) -> Result<Atom, Error> {
    match s.chars().next() {
        Some(c) => Ok(Atom::Char(c)),
        None => Err(TypeError::Char(s.to_string()))?,
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
            Atom::Char(c) => c.to_string(),
            Atom::Function(fun) => format!("{}", fun),
            Atom::Empty => "_".to_string(),
        }
    }
}

impl From<Function> for Atom {
    #[inline(always)]
    fn from(f: Function) -> Self {
        Atom::Function(f)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::Add => write!(f, "++"),
            Function::Empty => write!(f, "__"),
            Function::Divide => write!(f, "//"),
            Function::Id => write!(f, "id"),
            Function::Multiply => write!(f, "**"),
            Function::Play => write!(f, ">>"),
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
            Atom::Char(c) => write!(f, "{c}"),
            Atom::Function(ref fun) => write!(f, "{fun}"),
            Atom::Empty => write!(f, "_"),
        }
    }
}
