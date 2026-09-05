use arrayvec::ArrayVec;
use std::fmt;

use crate::{EXP_LEN, Error, TypeError, midi_note_to_number, midi_number_to_note, str_to_num};

pub type Atoms = ArrayVec<Atom, EXP_LEN>;

/// The MIDI note domain: `00`–`7F`.
///
/// Ordered as well as compared, because the Playback Engine keys a Timed
/// Play's ownership by the channel and note it sounds on. Ordering a note by
/// its number is the protocol's own order, and carrying the key as the two
/// domain types keeps the engine from re-deriving either domain from a byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

/// The MIDI channel domain: a Number in `00`–`0F`.
///
/// Orcvs sends direct hexadecimal MIDI values, so an operand outside the
/// protocol range is a Source error rather than something to scale or clamp.
/// Carrying the domain in the type rather than proving it and handing back a
/// `u8` is what lets [`crate::PlayCommand`] and the output adapter rely on the
/// range instead of re-deriving it. Ordered for the same reason [`Note`] is:
/// the two together key a Timed Play's ownership inside the Playback Engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MidiChannel(u8);

impl MidiChannel {
    #[inline(always)]
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for MidiChannel {
    type Error = crate::InterpretationError;

    #[inline(always)]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00..=0x0F => Ok(Self(value)),
            _ => Err(crate::InterpretationError::MidiChannel(value)),
        }
    }
}

/// The MIDI data-byte domain, `00`–`7F`, shared privately by every role that
/// occupies it.
///
/// A Play velocity, a Control Change controller or value, and a Pitch Bend LSB
/// or MSB all take the same range; only the word the diagnostic uses differs.
/// Sharing the predicate here and minting one public type per role below is
/// what keeps two operands of the same Function from being assignable to one
/// another. One shared public data-byte type would validate the range just as
/// well and leave a controller-for-value swap invisible, which is the failure
/// the role types exist to stop.
#[inline(always)]
fn midi_data_byte(role: &'static str, value: u8) -> Result<u8, crate::InterpretationError> {
    match value {
        0x00..=0x7F => Ok(value),
        _ => Err(crate::InterpretationError::MidiDataByte { role, value }),
    }
}

/// Mints one MIDI data-byte role as a distinct public type over the shared
/// private predicate.
///
/// The role word becomes a property of the type rather than an argument every
/// call site has to remember, so a role that arrives with Control Change or
/// Pitch Bend is one line here and inherits both the domain and the diagnostic
/// wording.
macro_rules! define_data_byte_roles {
    ($($(#[$doc:meta])* $name:ident => $role:literal),+ $(,)?) => {$(
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(u8);

        impl $name {
            /// The zero data byte, which every role in this domain contains.
            ///
            /// A constant rather than a conversion a caller has to unwrap: the
            /// Playback Engine delivers a scheduled Note Off as MIDI's
            /// zero-velocity stop, and a `try_from(0)` there would be an
            /// unreachable failure path inside a Tick.
            pub const ZERO: Self = Self(0);

            #[inline(always)]
            pub const fn value(self) -> u8 {
                self.0
            }
        }

        impl TryFrom<u8> for $name {
            type Error = crate::InterpretationError;

            #[inline(always)]
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match midi_data_byte($role, value) {
                    Ok(value) => Ok(Self(value)),
                    Err(error) => Err(error),
                }
            }
        }
    )+};
}

define_data_byte_roles! {
    /// A Play velocity. `00` is not an absent note but MIDI's explicit stop,
    /// so the domain starts at zero like every other data byte.
    Velocity => "velocity",
}

/// The Timed and Monophonic Play lifetime: a Number in `00`–`FF`.
///
/// Every byte is a length, so this converts where the MIDI domains validate.
/// It is a type of its own regardless, because a length is a count of Ticks
/// rather than a MIDI value: nothing else keeps it out of a data-byte position,
/// and nothing else says that the Playback Engine, not the output adapter, is
/// what reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Length(u8);

impl Length {
    /// The lifetime that starts no note.
    pub const ZERO: Self = Self(0);

    /// This length's Number, as the Source wrote it.
    #[inline(always)]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// How many Ticks this length lasts.
    #[inline(always)]
    pub const fn ticks(self) -> u64 {
        self.0 as u64
    }
}

impl From<u8> for Length {
    #[inline(always)]
    fn from(value: u8) -> Self {
        Self(value)
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

/// Whether a Function extends across a Sequence operand or requires a scalar
/// one.
///
/// ADR 0007 makes pervasive extension the rule for the Atomic Functions, and
/// ADR 0012 makes Increment and Interpolation exceptions to it because element
/// identity across Ticks would need hidden state their one visible Atom cannot
/// hold. An exception that arrived by omission would therefore be silent, so
/// this is declared beside every other property of a Function rather than
/// inferred from a family prefix or assumed from a signature: `is_terminal`
/// can be read off the `!` family, and this cannot, because two Functions of
/// the same family and the same signature differ in it.
#[derive(Clone, Copy)]
enum Pervasion {
    Pervasive,
    /// No row declares this today. ADR 0030 removed the last two that did —
    /// the Terminal Output Functions were declared scalar from an
    /// implementation brief rather than from a decision — and ADR 0012's
    /// Increment and Interpolation, which are the rows that state the exception
    /// on their own terms, are unbuilt. Deleting the answer would leave the
    /// column with one value and make the exception impossible to declare,
    /// which is the opposite of what declaring it beside the signature is for.
    /// `expect` rather than `allow`, so the first Function to declare it turns
    /// this attribute into the error that deletes it.
    #[expect(
        dead_code,
        reason = "the exception ADR 0012 states has no built Function yet: this is the answer it will declare"
    )]
    Scalar,
}

// An operand's declared type decides three things, one per macro below: the
// `Token` its signature is checked against, the Rust value a Function body
// receives for it, and how the checked `Atom` becomes that value. A new operand
// type needs one arm in each of the three, so a Function definition cannot name
// a type the extraction does not already know how to check and bind.
//
// The column therefore answers two questions rather than one. `Token` is what
// the parser reads from two Cells; the bound type is the domain the interpreter
// accepts, which may be narrower. `MidiChannel` and `Velocity` are both read as
// a `Number` and are neither a `Number` nor each other once bound. That is one
// refinement chain from Cells to Number to a domain, declared where the role is
// declared, which is what lets a new MIDI terminal Function inherit its
// validation from the table instead of from a body that remembers to ask.
macro_rules! operand_token {
    (Number) => {
        crate::Token::Number
    };
    (Note) => {
        crate::Token::Note
    };
    (MidiChannel) => {
        crate::Token::Number
    };
    (Velocity) => {
        crate::Token::Number
    };
    (Length) => {
        crate::Token::Number
    };
}

// The domain half of a bind, for one Atom, with the bound value discarded.
//
// It is written through `operand_bind!` rather than beside it so a declared
// domain still has exactly one definition: an arm added here could narrow
// differently from the arm that binds, and the two are asked the same question
// about the same Atom. The three arms above stay the whole cost of a new
// operand type.
macro_rules! operand_domain {
    ($operand:ident, $role:ident) => {
        (|atom: crate::Atom| -> Result<(), crate::Error> {
            operand_bind!($operand, Some(atom), $role).map(|_| ())
        }) as fn(crate::Atom) -> Result<(), crate::Error>
    };
}

macro_rules! operand_type {
    (Number) => {
        u8
    };
    (Note) => {
        crate::Note
    };
    (MidiChannel) => {
        crate::MidiChannel
    };
    (Velocity) => {
        crate::Velocity
    };
    (Length) => {
        crate::Length
    };
}

// A bind answers a `Result` because a declared domain is narrower than the
// `Token` the parser checked: `Stack::extract` proves the operand is a Number,
// and only the conversion here proves it is a channel. The domain diagnostic is
// therefore raised by the declaration rather than by a Function body.
macro_rules! operand_bind {
    (Number, $operand:expr, $role:ident) => {
        match $operand {
            Some(crate::Atom::Number(value)) => Ok::<_, crate::Error>(value),
            _ => unreachable!(concat!(
                "typed extraction guarantees a Number for the ",
                stringify!($role),
                " operand"
            )),
        }
    };
    (Note, $operand:expr, $role:ident) => {
        match $operand {
            Some(crate::Atom::Note(value)) => Ok::<_, crate::Error>(value),
            _ => unreachable!(concat!(
                "typed extraction guarantees a Note for the ",
                stringify!($role),
                " operand"
            )),
        }
    };
    (MidiChannel, $operand:expr, $role:ident) => {
        match $operand {
            Some(crate::Atom::Number(value)) => {
                crate::MidiChannel::try_from(value).map_err(crate::Error::from)
            }
            _ => unreachable!(concat!(
                "typed extraction guarantees a Number for the ",
                stringify!($role),
                " operand"
            )),
        }
    };
    (Velocity, $operand:expr, $role:ident) => {
        match $operand {
            Some(crate::Atom::Number(value)) => {
                crate::Velocity::try_from(value).map_err(crate::Error::from)
            }
            _ => unreachable!(concat!(
                "typed extraction guarantees a Number for the ",
                stringify!($role),
                " operand"
            )),
        }
    };
    // The one declared domain that is the whole byte, so this converts where
    // the others validate. It still binds through the same arm, because what
    // makes a length a length is the type it arrives as, not a check it passed.
    (Length, $operand:expr, $role:ident) => {
        match $operand {
            Some(crate::Atom::Number(value)) => Ok::<_, crate::Error>(crate::Length::from(value)),
            _ => unreachable!(concat!(
                "typed extraction guarantees a Number for the ",
                stringify!($role),
                " operand"
            )),
        }
    };
}

// A Function of exactly one declared role gets the `UnaryOperands` marker and
// every other Function gets nothing, decided by which arm the role list matches
// rather than by a second list to keep in step. The single-role arm is written
// first because a one-element list matches both.
macro_rules! unary_operands {
    ($variant:ident, [$role:ident]) => {
        impl crate::stack::UnaryOperands for $variant {}
    };
    ($variant:ident, [$($role:ident),*]) => {};
}

// #[derive(serde::Deserialize, serde::Serialize)]
macro_rules! define_functions {
    ($($variant:ident => ($spelling:literal, $kind:ident, $pervasion:ident, [$($role:ident: $operand:ident),* $(,)?])),+ $(,)?) => {
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

            const fn pervasion(self) -> Pervasion {
                match self {
                    $(Self::$variant => Pervasion::$pervasion,)+
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

            /// Whether this Function extends pervasively across a Sequence
            /// operand instead of requiring one Atom per position.
            ///
            /// The Operand Stack asks this before it decides the shape of an
            /// operation, so broadcasting is something a Function declares
            /// rather than something the shape of its operands decides for it:
            /// a Sequence reaching a Scalar Function is refused with the same
            /// diagnostic whether that Function is Terminal or, like Increment,
            /// an ordinary value Function that ADR 0012 keeps scalar.
            #[inline(always)]
            pub const fn is_pervasive(self) -> bool {
                matches!(self.pervasion(), Pervasion::Pervasive)
            }

            pub(crate) const fn signature(self) -> &'static [crate::Token] {
                match self {
                    $(Self::$variant => &[$(operand_token!($operand),)*],)+
                }
            }

            /// One domain check per declared operand, in signature order.
            ///
            /// The narrowing a declaration states — a `MidiChannel` is a
            /// `Number` the parser read and a channel only once its domain
            /// admits it — is ordinarily answered as an element binds, which
            /// covers every operand at every width but one. At width zero no
            /// element binds, so the Operand Stack asks here instead, and a
            /// scalar operand beside an empty Sequence is checked against the
            /// domain it declares rather than only against its `Token`.
            pub(crate) fn domains(self) -> &'static [fn(crate::Atom) -> Result<(), crate::Error>] {
                match self {
                    $(Self::$variant => const { &[$(operand_domain!($operand, $role),)*] },)+
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
            use crate::{Error, Function, stack::{Extracted, Operands}};

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
                    fn from_operands(operands: Extracted<'_>) -> Result<Self, Error> {
                        let mut operands = operands.atoms().iter().copied();

                        // Field initialisers evaluate in signature order, so
                        // the first operand outside its domain is the one that
                        // diagnoses, exactly as the pop loop above it.
                        Ok(Self {
                            $($role: operand_bind!($operand, operands.next(), $role)?,)*
                        })
                    }
                }

                unary_operands!($variant, [$($role),*]);
            )+
        }

        /// Every Function's declared operand token and its bind must agree.
        ///
        /// `operand_token!` decides what `Stack::extract` accepts and
        /// `operand_bind!` decides what it then reads. They are separate arms
        /// keyed on the same declared type, so a disagreement between them is
        /// not a compile error: the bind falls through to its `unreachable!`
        /// and panics inside Tick planning, under the Source lock, which is
        /// exactly the third option ADR 0028 rules out. Extracting every
        /// Function once from operands built out of its own signature turns
        /// that into a test failure at the moment the operand type is added.
        #[cfg(test)]
        mod declaration_agreement {
            use crate::{Atom, Note, Stack, Token};

            /// The lowest value each token can carry. Every domain declared
            /// over a token so far contains it; a domain that excluded its
            /// token's minimum would fail here and need its own witness, which
            /// is the right way to find that out.
            fn lowest(token: Token) -> Atom {
                match token {
                    Token::Number => Atom::Number(0),
                    Token::Note => Atom::Note(Note::try_from(0).expect("00 is a Note")),
                    other => panic!("no operand is declared as {other:?}"),
                }
            }

            #[test]
            fn every_declared_operand_binds_the_atom_its_token_accepts() {
                $({
                    let function = crate::Function::$variant;
                    let mut stack: Stack<16> = Stack::new();

                    // Pushed in reverse so extraction pops them in signature order.
                    for token in function.signature().iter().copied().rev() {
                        stack.push(lowest(token)).unwrap();
                    }

                    assert!(
                        stack.extract::<super::operands::$variant>().is_ok(),
                        "{function:?} declares a token its bind does not read",
                    );
                })+
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
    AbsoluteDifference => (".|", Value, Pervasive, [left: Number, right: Number]),
    Add => (".+", Value, Pervasive, [left: Number, right: Number]),
    ConvertToNote => (".^", Value, Pervasive, [value: Number]),
    ConvertToNumber => (".v", Value, Pervasive, [value: Note]),
    Divide => ("./", Value, Pervasive, [left: Number, right: Number]),
    Equality => (".=", Value, Pervasive, [left: Number, right: Number]),
    Maximum => (".>", Value, Pervasive, [left: Number, right: Number]),
    Minimum => (".<", Value, Pervasive, [left: Number, right: Number]),
    Modulo => (".%", Value, Pervasive, [left: Number, right: Number]),
    Multiply => (".x", Value, Pervasive, [left: Number, right: Number]),
    RawPlay => ("!>", Terminal, Pervasive, [channel: MidiChannel, velocity: Velocity, note: Note]),
    Subtract => (".-", Value, Pervasive, [left: Number, right: Number]),
    TimedPlay => ("!~", Terminal, Pervasive, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
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
    use super::{Activation, Atom, Function, Length, MidiChannel, Note, Velocity, to_atom_num};
    use crate::InterpretationError;

    #[test]
    fn each_midi_domain_type_accepts_exactly_its_protocol_range() {
        // Relocated from the two validator functions this replaced. The domain
        // is now a property of the type, so the conversion is what has to hold
        // over the whole byte, and every input is cheap enough to enumerate.
        for value in 0..=u8::MAX {
            assert_eq!(
                MidiChannel::try_from(value).is_ok(),
                value <= 0x0F,
                "{value:02X}"
            );
            assert_eq!(
                Velocity::try_from(value).is_ok(),
                value <= 0x7F,
                "{value:02X}"
            );
            assert_eq!(Note::try_from(value).is_ok(), value <= 0x7F, "{value:02X}");
        }
    }

    #[test]
    fn the_timed_play_length_domain_is_the_whole_byte() {
        // ADR 0016 gives length `00`–`FF`, so unlike every MIDI domain beside
        // it there is no value to refuse — and none to alter either.
        for value in 0..=u8::MAX {
            assert_eq!(Length::from(value).value(), value, "{value:02X}");
            assert_eq!(Length::from(value).ticks(), u64::from(value), "{value:02X}");
        }

        assert_eq!(Length::ZERO, Length::from(0));
    }

    #[test]
    fn the_zero_data_byte_is_available_without_a_conversion_to_unwrap() {
        // The Playback Engine delivers a scheduled Note Off as MIDI's
        // zero-velocity stop, inside a Tick, where a fallible conversion would
        // be an unreachable failure path.
        assert_eq!(Velocity::ZERO, Velocity::try_from(0).unwrap());
        assert_eq!(Velocity::ZERO.value(), 0);
    }

    #[test]
    fn each_midi_domain_type_carries_the_value_it_was_given() {
        // A newtype that quietly altered its value would satisfy the range test
        // above and still be wrong, so the accepted half is checked too.
        for value in 0..=0x0F {
            assert_eq!(MidiChannel::try_from(value).unwrap().value(), value);
        }
        for value in 0..=0x7F {
            assert_eq!(Velocity::try_from(value).unwrap().value(), value);
            assert_eq!(Note::try_from(value).unwrap().value(), value);
        }
    }

    #[test]
    fn a_rejected_data_byte_names_the_operand_role_that_supplied_it() {
        // The role word moved from an argument at the call site to a property
        // of the type. Control Change and Pitch Bend mint their roles from the
        // same private predicate, so the diagnostic each one answers is fixed
        // here rather than at whatever body happens to construct it.
        assert_eq!(
            Velocity::try_from(0x80).unwrap_err().to_string(),
            "MIDI velocity 80 is outside the range 00\u{2013}7F"
        );

        for role in ["velocity", "controller", "value", "lsb", "msb"] {
            assert_eq!(
                InterpretationError::MidiDataByte { role, value: 0x80 }.to_string(),
                format!("MIDI {role} 80 is outside the range 00\u{2013}7F")
            );
        }

        assert_eq!(
            MidiChannel::try_from(0x10).unwrap_err().to_string(),
            "MIDI channel 10 is outside the range 00\u{2013}0F"
        );
    }

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
            Atom::Function(Function::RawPlay),
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
    fn play_functions_display_with_the_terminal_output_family_spellings() {
        assert_eq!(Function::RawPlay.to_string(), "!>");
        assert_eq!(Function::TimedPlay.to_string(), "!~");
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

        assert!(Function::RawPlay.is_terminal());
        assert!(!Function::Add.is_terminal());
    }

    #[test]
    fn every_function_declares_whether_it_extends_over_a_sequence() {
        // ADR 0007 makes pervasive extension the rule for Atomic Functions and
        // ADR 0012 makes Increment and Interpolation exceptions to it, so the
        // property cannot be inferred from a family prefix the way
        // `is_terminal` can. ADR 0030 settles the other family the same way:
        // the Terminal Output Functions extend as well, so pervasion is not a
        // property of answering a value either, and a `!`-spelled row is no
        // more predictable from its spelling than a `.`-spelled one. It is
        // declared per Function instead, and this match is exhaustive over
        // `Function` with no wildcard: a Function added later has to be
        // classified here as well as in the table, so neither an omission nor a
        // copied row can make it broadcast by accident.
        for function in Function::ALL.iter().copied() {
            let expected = match function {
                Function::AbsoluteDifference
                | Function::Add
                | Function::ConvertToNote
                | Function::ConvertToNumber
                | Function::Divide
                | Function::Equality
                | Function::Maximum
                | Function::Minimum
                | Function::Modulo
                | Function::Multiply
                | Function::RawPlay
                | Function::Subtract
                | Function::TimedPlay => true,
            };

            assert_eq!(function.is_pervasive(), expected, "{function:?}");
        }
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
