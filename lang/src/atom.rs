use arrayvec::ArrayVec;
use std::fmt;

use crate::{EXP_LEN, Error, TypeError, midi_note_to_number, midi_number_to_note, str_to_num};

pub type Atoms = ArrayVec<Atom, EXP_LEN>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note(u8);

impl Note {
    #[inline(always)]
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Note {
    type Error = crate::InterpretationError;

    #[inline(always)]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00..=0x7F => Ok(Self(value)),
            _ => Err(crate::InterpretationError::NoteConversion(value)),
        }
    }
}

// #[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Atom {
    Activation(Activation),
    Bang,
    Char(char),
    Empty,
    Function(Function),
    Note(Note),
    Number(u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Activation {
    North,
    South,
    West,
    East,
}

impl Activation {
    pub fn spelling(self) -> &'static str {
        match self {
            Self::North => "^^",
            Self::South => "vv",
            Self::West => "<<",
            Self::East => ">>",
        }
    }
}

impl TryFrom<&str> for Activation {
    type Error = ();

    fn try_from(spelling: &str) -> Result<Self, Self::Error> {
        match spelling {
            "^^" => Ok(Self::North),
            "vv" => Ok(Self::South),
            "<<" => Ok(Self::West),
            ">>" => Ok(Self::East),
            _ => Err(()),
        }
    }
}

// #[derive(serde::Deserialize, serde::Serialize)]
macro_rules! define_functions {
    ($($variant:ident => ($spelling:literal, [$($operand:expr),* $(,)?])),+ $(,)?) => {
        $(const _: () = assert!(
            $spelling.len() == 2 && $spelling.is_ascii(),
            "a Function spelling must be exactly two ASCII Cells",
        );)+

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum Function {
            $($variant,)+
        }

        impl Function {
            /// Every real Function, generated from the canonical definitions above.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            pub(crate) const fn spelling(self) -> &'static str {
                match self {
                    $(Self::$variant => $spelling,)+
                }
            }

            pub(crate) fn signature(self) -> &'static [crate::Token] {
                match self {
                    $(Self::$variant => &[$($operand,)*],)+
                }
            }
        }

        impl TryFrom<&str> for Function {
            type Error = Error;

            /// Two definitions sharing a spelling would generate a duplicate arm here and
            /// leave the later variant unreachable from the parser. Denying the lint turns
            /// that into a compile error rather than a warning the build would accept.
            #[deny(unreachable_patterns)]
            #[inline(always)]
            fn try_from(spelling: &str) -> Result<Self, Self::Error> {
                match spelling {
                    $($spelling => Ok(Self::$variant),)+
                    _ => Err(crate::SyntaxError::UnknownFunction(spelling.to_string()).into()),
                }
            }
        }
    };
}

define_functions! {
    Add => (".+", [crate::Token::Number, crate::Token::Number]),
    ConvertToNote => (".^", [crate::Token::Number]),
    ConvertToNumber => (".v", [crate::Token::Note]),
    Divide => ("./", [crate::Token::Number, crate::Token::Number]),
    Multiply => (".x", [crate::Token::Number, crate::Token::Number]),
    Play => ("!>", [crate::Token::Number, crate::Token::Number, crate::Token::Note]),
    Subtract => (".-", [crate::Token::Number, crate::Token::Number]),
}

#[inline(always)]
pub fn to_atom_note(s: &str) -> Result<Atom, Error> {
    match midi_note_to_number(s) {
        Some(n) => Ok(Atom::Note(Note(n))),
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
        f.write_str(self.spelling())
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atom::Activation(activation) => f.write_str(activation.spelling()),
            Atom::Bang => write!(f, "**"),
            // Numbers are hexadecimal: rendered results are written back into the
            // Source and re-parsed as two Cells, so they must round trip as hex
            Atom::Number(n) => write!(f, "{:02X}", n),
            Atom::Note(n) => match midi_number_to_note(n.value()) {
                Some(note) => write!(f, "{note}"),
                None => Err(fmt::Error),
            },
            Atom::Char(c) => write!(f, "{c}"),
            Atom::Function(fun) => write!(f, "{fun}"),
            Atom::Empty => write!(f, "_"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Activation, Atom, Function, Note, to_atom_num};

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
            let atom = Atom::Number(n);
            assert_eq!(
                String::from(atom),
                atom.to_string(),
                "String::from disagreed with Display for {atom:?}"
            );
        }

        for n in 0..=0x7F {
            let atom = Atom::Note(Note::try_from(n).unwrap());
            assert_eq!(String::from(atom), atom.to_string());
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
    fn note_construction_enforces_the_midi_domain_before_rendering() {
        assert!(Note::try_from(0x80).is_err());

        let note = Atom::Note(Note::try_from(0x7F).unwrap());
        assert_eq!(String::from(note), "G9");
    }

    #[test]
    fn arithmetic_functions_display_with_the_dot_family_spellings() {
        assert_eq!(Function::Add.to_string(), ".+");
        assert_eq!(Function::Subtract.to_string(), ".-");
        assert_eq!(Function::Multiply.to_string(), ".x");
        assert_eq!(Function::Divide.to_string(), "./");
    }

    #[test]
    fn numeric_conversion_functions_display_with_their_dot_family_spellings() {
        assert_eq!(Function::ConvertToNumber.to_string(), ".v");
        assert_eq!(Function::ConvertToNote.to_string(), ".^");
    }

    #[test]
    fn play_function_displays_with_the_terminal_output_family_spelling() {
        assert_eq!(Function::Play.to_string(), "!>");
    }

    #[test]
    fn bang_and_activation_display_with_their_complete_spellings() {
        assert_eq!(Atom::Bang.to_string(), "**");
        assert_eq!(Atom::Activation(Activation::North).to_string(), "^^");
        assert_eq!(Atom::Activation(Activation::South).to_string(), "vv");
        assert_eq!(Atom::Activation(Activation::West).to_string(), "<<");
        assert_eq!(Atom::Activation(Activation::East).to_string(), ">>");
    }

    #[test]
    fn test_notes_render_distinctly_from_numbers() {
        // Notes render via midi_number_to_note and are unaffected by hex Numbers
        assert_eq!(Atom::Note(Note::try_from(60).unwrap()).to_string(), "C4");
        assert_eq!(Atom::Note(Note::try_from(69).unwrap()).to_string(), "A4");
        assert_eq!(Atom::Note(Note::try_from(21).unwrap()).to_string(), "A0");

        assert_ne!(
            Atom::Note(Note::try_from(60).unwrap()).to_string(),
            Atom::Number(60).to_string()
        );
        assert_eq!(Atom::Number(60).to_string(), "3C");
    }
}
