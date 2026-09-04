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

/// What a Function contributes to the Expression that contains it.
///
/// A Value Function answers with a language value the surrounding Expression
/// can consume. A Terminal Function performs a Terminal Output effect and
/// answers with nothing, so it is valid only where no value is required.
/// Every Function states which it is in the canonical definitions below, and
/// nothing else is allowed to decide: a spelling table that disagreed with the
/// interpreter would silently make a terminal Function usable as an operand.
#[derive(Clone, Copy)]
enum FunctionKind {
    Value,
    Terminal,
}

// An operand's declared type decides three things, one per macro below: the
// `Token` its signature is checked against, the Rust value a Function body
// receives for it, and the `Atom` that carries it. A new operand type needs one
// arm in each of the three, so a Function definition cannot name a type the
// extraction does not already know how to check and bind.
macro_rules! operand_token {
    (Number) => {
        crate::Token::Number
    };
    (Note) => {
        crate::Token::Note
    };
}

macro_rules! operand_type {
    (Number) => {
        u8
    };
    (Note) => {
        crate::Note
    };
}

macro_rules! operand_bind {
    (Number, $operand:expr, $role:ident) => {
        match $operand {
            Some(crate::Atom::Number(value)) => value,
            _ => unreachable!(concat!(
                "typed extraction guarantees a Number for the ",
                stringify!($role),
                " operand"
            )),
        }
    };
    (Note, $operand:expr, $role:ident) => {
        match $operand {
            Some(crate::Atom::Note(value)) => value,
            _ => unreachable!(concat!(
                "typed extraction guarantees a Note for the ",
                stringify!($role),
                " operand"
            )),
        }
    };
}

// #[derive(serde::Deserialize, serde::Serialize)]
macro_rules! define_functions {
    ($($variant:ident => ($spelling:literal, $kind:ident, [$($role:ident: $operand:ident),* $(,)?])),+ $(,)?) => {
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

            const fn kind(self) -> FunctionKind {
                match self {
                    $(Self::$variant => FunctionKind::$kind,)+
                }
            }

            /// Whether this Function performs a Terminal Output effect instead
            /// of producing a value. Both the Interpreter's nesting guard and
            /// tick planning's activation gate ask this rather than naming
            /// individual Functions, so a new terminal spelling joins both by
            /// its definition alone.
            #[inline(always)]
            pub const fn is_terminal(self) -> bool {
                matches!(self.kind(), FunctionKind::Terminal)
            }

            pub(crate) fn signature(self) -> &'static [crate::Token] {
                match self {
                    $(Self::$variant => &[$(operand_token!($operand),)*],)+
                }
            }
        }

        /// The operands each Function declares, one struct per Function, with a
        /// field named for the role that position plays.
        ///
        /// A Function body destructures the struct its Function declares, so an
        /// operand's position is written once — here, beside the role name and
        /// the type — and never restated in the body that reads it. Transposing
        /// two same-typed operands is therefore an edit to the declaration
        /// rather than a silent edit inside a body.
        pub(crate) mod operands {
            use crate::{Function, stack::{Extracted, Operands}};

            $(
                // Every Function in the table gets a struct, including the two
                // whose evaluation deliberately takes a numeric value rather
                // than the single type their signature declares: ADR 0021's
                // idempotence for nested values, which `lang-foundations/06`
                // records as an exclusion. Those two structs are generated and
                // unread, which is the table staying uniform rather than dead
                // code to delete — dropping them would mean the declaration no
                // longer covered every Function.
                #[allow(dead_code)]
                pub(crate) struct $variant {
                    $(pub(crate) $role: operand_type!($operand),)*
                }

                impl Operands for $variant {
                    const FUNCTION: Function = Function::$variant;

                    #[inline(always)]
                    fn from_operands(operands: Extracted<'_>) -> Self {
                        let mut operands = operands.atoms().iter().copied();

                        Self {
                            $($role: operand_bind!($operand, operands.next(), $role),)*
                        }
                    }
                }
            )+
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
    AbsoluteDifference => (".|", Value, [left: Number, right: Number]),
    Add => (".+", Value, [left: Number, right: Number]),
    ConvertToNote => (".^", Value, [value: Number]),
    ConvertToNumber => (".v", Value, [value: Note]),
    Divide => ("./", Value, [left: Number, right: Number]),
    Equality => (".=", Value, [left: Number, right: Number]),
    Maximum => (".>", Value, [left: Number, right: Number]),
    Minimum => (".<", Value, [left: Number, right: Number]),
    Modulo => (".%", Value, [left: Number, right: Number]),
    Multiply => (".x", Value, [left: Number, right: Number]),
    Play => ("!>", Terminal, [channel: Number, velocity: Number, note: Note]),
    Subtract => (".-", Value, [left: Number, right: Number]),
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
        assert_eq!(Function::AbsoluteDifference.to_string(), ".|");
        assert_eq!(Function::Modulo.to_string(), ".%");
        assert_eq!(Function::Minimum.to_string(), ".<");
        assert_eq!(Function::Maximum.to_string(), ".>");
        assert_eq!(Function::Equality.to_string(), ".=");
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
    fn exactly_the_terminal_output_family_is_classified_terminal() {
        // The two families are visible in the spellings a user types: the dot
        // family answers with a value, and the `!` family performs. A
        // definition whose classification contradicted its spelling would let
        // a terminal Function stand where an operand belongs.
        for function in Function::ALL.iter().copied() {
            assert_eq!(
                function.is_terminal(),
                function.spelling().starts_with('!'),
                "{function:?} spells {:?}",
                function.spelling()
            );
        }

        assert!(Function::Play.is_terminal());
        assert!(!Function::Add.is_terminal());
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
