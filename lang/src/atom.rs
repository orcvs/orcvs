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
    let n = str_to_num(s)?;
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
    /// Delegates to `Display` so the two renderings can never drift apart.
    #[inline(always)]
    fn from(atom: Atom) -> Self {
        atom.to_string()
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
            // Numbers are hexadecimal: rendered results are written back into the
            // Source and re-parsed as two Cells, so they must round trip as hex
            Atom::Number(n) => write!(f, "{:02X}", n),
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

#[cfg(test)]
mod test {
    use super::{to_atom_num, Atom, Function};

    #[test]
    fn test_number_displays_as_two_uppercase_hex_digits() {
        // Numbers are hexadecimal: the parser reads a Number operand as two Cells
        assert_eq!(Atom::Number(0).to_string(), "00");
        assert_eq!(Atom::Number(3).to_string(), "03");
        assert_eq!(Atom::Number(10).to_string(), "0A");
        assert_eq!(Atom::Number(15).to_string(), "0F");
        assert_eq!(Atom::Number(16).to_string(), "10");
        assert_eq!(Atom::Number(100).to_string(), "64");
        assert_eq!(Atom::Number(255).to_string(), "FF");
    }

    #[test]
    fn test_number_display_round_trips_through_the_parser() {
        // Results are written back into the Source and re-parsed as source text,
        // so every Number must survive a render/parse round trip.
        for n in 0..=u8::MAX {
            let atom = Atom::Number(n);
            let rendered = atom.to_string();

            assert_eq!(rendered.len(), 2, "Number({n}) rendered as {rendered:?}");
            assert_eq!(
                to_atom_num(&rendered).unwrap(),
                atom,
                "Number({n}) did not round trip through {rendered:?}"
            );
        }
    }

    #[test]
    fn test_atom_to_string_matches_display() {
        // TypeError::Number(atom.into()) must report the same rendering the grid shows
        assert_eq!(
            String::from(Atom::Number(10)),
            Atom::Number(10).to_string(),
            "String::from disagreed with Display"
        );

        for n in 0..=u8::MAX {
            for atom in [Atom::Number(n), Atom::Note(n)] {
                assert_eq!(
                    String::from(atom),
                    atom.to_string(),
                    "String::from disagreed with Display for {atom:?}"
                );
            }
        }

        for atom in [
            Atom::Char('v'),
            Atom::Empty,
            Atom::Function(Function::Add),
            Atom::Function(Function::Play),
        ] {
            assert_eq!(String::from(atom), atom.to_string(), "{atom:?}");
        }
    }

    #[test]
    fn test_notes_render_distinctly_from_numbers() {
        // Notes render via midi_number_to_note and are unaffected by hex Numbers
        assert_eq!(Atom::Note(60).to_string(), "C4");
        assert_eq!(Atom::Note(69).to_string(), "A4");
        assert_eq!(Atom::Note(21).to_string(), "A0");

        assert_ne!(Atom::Note(60).to_string(), Atom::Number(60).to_string());
        assert_eq!(Atom::Number(60).to_string(), "3C");
    }
}
