//! The operand types a Function declares, as Rust types.
//!
//! A Function's row in the table names one of these types per role. The type
//! decides three things together: the [`Token`] the Parser reads a literal in
//! that slot as, the payload extraction yields for it, and the value the
//! Function body receives. [`Operand::bind`] takes exactly the payload its
//! [`Operand::Token`] yields, so an operand whose token and bind disagree does
//! not compile:
//!
//! ```
//! use lang::operand::Operand;
//! use lang::{Error, MidiChannel};
//!
//! enum Channel {}
//!
//! impl Operand for Channel {
//!     type Token = lang::operand::Number;
//!     type Bound = MidiChannel;
//!
//!     fn bind(number: u8) -> Result<MidiChannel, Error> {
//!         Ok(MidiChannel::try_from(number)?)
//!     }
//! }
//! ```
//!
//! The same operand declared over the Note token, while its bind still reads a
//! Number, is refused by the type checker: `E0053`, `bind` expected
//! `lang::Note` and found `u8`. The two examples differ in that one line, so
//! the mismatch is the only thing the second can fail on.
//!
//! ```compile_fail,E0053
//! use lang::operand::Operand;
//! use lang::{Error, MidiChannel};
//!
//! enum Channel {}
//!
//! impl Operand for Channel {
//!     type Token = lang::operand::Note;
//!     type Bound = MidiChannel;
//!
//!     fn bind(number: u8) -> Result<MidiChannel, Error> {
//!         Ok(MidiChannel::try_from(number)?)
//!     }
//! }
//! ```
//!
//! Every type here is uninhabited: it is a name for a declaration, never a
//! value.

use crate::{Error, SequenceError, Token, TypeError, Value};

/// What a Parser token is at evaluation: the [`Token`] a literal in the slot
/// is read as, and the payload one operand yields once checked against it.
pub trait TokenKind {
    /// The token a signature names for this slot.
    const TOKEN: Token;

    /// What checking one operand against this token yields.
    type Payload;

    /// Checks one Atom, answering the payload it carries.
    ///
    /// This is the element reading: a pervasive Function reads each member of
    /// a Sequence operand here, one element at a time.
    fn from_atom(atom: crate::Atom) -> Result<Self::Payload, Error>;

    /// Checks one whole value, answering the payload it carries.
    ///
    /// This is the whole-value reading a Function that consumes its operands
    /// intact binds through. A Sequence satisfies no Atom-shaped token, so it
    /// diagnoses unless the token overrides this to consume one.
    #[inline(always)]
    fn from_value(value: Value) -> Result<Self::Payload, Error> {
        match value {
            Value::Atom(atom) => Self::from_atom(atom),
            Value::Sequence(sequence) => {
                Err(SequenceError::ExpectedAtom(sequence.to_string()).into())
            }
        }
    }
}

/// One operand type a Function table row may declare.
pub trait Operand {
    /// The token this operand is read as, which fixes the payload [`bind`]
    /// receives.
    ///
    /// [`bind`]: Operand::bind
    type Token: TokenKind;

    /// What the Function body receives for this operand.
    type Bound;

    /// The token a signature names for this operand.
    const TOKEN: Token = <Self::Token as TokenKind>::TOKEN;

    /// Narrows the checked payload to the operand's declared domain.
    ///
    /// Fallible because a domain may be narrower than its token: a MIDI
    /// channel is read as a Number and is a channel only once this admits it.
    fn bind(payload: <Self::Token as TokenKind>::Payload) -> Result<Self::Bound, Error>;
}

/// A token an Operand Literal can be read as, so a [`Numeric`] operand can
/// name the literal its slot reads.
pub trait Literal: TokenKind {}

/// Either numeric Atom, as a [`Numeric`] operand receives it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumericValue {
    Note(crate::Note),
    Number(u8),
}

/// A Number: `00`–`FF`.
pub enum Number {}

impl TokenKind for Number {
    const TOKEN: Token = Token::Number;
    type Payload = u8;

    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<u8, Error> {
        match atom {
            crate::Atom::Number(number) => Ok(number),
            atom => Err(TypeError::Number(atom.into()).into()),
        }
    }
}

impl Literal for Number {}

impl Operand for Number {
    type Token = Self;
    type Bound = u8;

    #[inline(always)]
    fn bind(number: u8) -> Result<u8, Error> {
        Ok(number)
    }
}

/// A MIDI Note: `00`–`7F`.
pub enum Note {}

impl TokenKind for Note {
    const TOKEN: Token = Token::Note;
    type Payload = crate::Note;

    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<crate::Note, Error> {
        match atom {
            crate::Atom::Note(note) => Ok(note),
            atom => Err(TypeError::Note(atom.into()).into()),
        }
    }
}

impl Literal for Note {}

impl Operand for Note {
    type Token = Self;
    type Bound = crate::Note;

    #[inline(always)]
    fn bind(note: crate::Note) -> Result<crate::Note, Error> {
        Ok(note)
    }
}

/// Any Atom: the replacement of Replace, which may differ in type from the
/// member it displaces.
///
/// It declares no type, so no Atom fails it. Every Atom with no place at an
/// operand position is refused by a rule stated elsewhere: an Atom that cannot
/// be a Sequence member by [`crate::Sequence::new`], and an effect Function by
/// the Interpreter's nesting guard. Restating either here would give that rule
/// a second place to disagree from.
pub enum Atom {}

impl TokenKind for Atom {
    const TOKEN: Token = Token::Atom;
    type Payload = crate::Atom;

    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<crate::Atom, Error> {
        Ok(atom)
    }
}

impl Operand for Atom {
    type Token = Self;
    type Bound = crate::Atom;

    #[inline(always)]
    fn bind(atom: crate::Atom) -> Result<crate::Atom, Error> {
        Ok(atom)
    }
}

/// A whole Sequence, consumed intact.
///
/// No Atom satisfies it and none is promoted: promotion is a Function's
/// decision, which [`AtomOrSequence`] declares, so an Atom here diagnoses
/// rather than widening into a singleton.
pub enum Sequence {}

impl TokenKind for Sequence {
    const TOKEN: Token = Token::Sequence;
    type Payload = crate::Sequence;

    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<crate::Sequence, Error> {
        Err(SequenceError::ExpectedSequence(atom.into()).into())
    }

    #[inline(always)]
    fn from_value(value: Value) -> Result<crate::Sequence, Error> {
        crate::Sequence::try_from(value)
    }
}

impl Operand for Sequence {
    type Token = Self;
    type Bound = crate::Sequence;

    #[inline(always)]
    fn bind(sequence: crate::Sequence) -> Result<crate::Sequence, Error> {
        Ok(sequence)
    }
}

/// A Sequence, or an Atom promoted to the Sequence of that one member.
///
/// Read as an Atom by the Parser, because a literal in this slot is one Atom.
pub enum AtomOrSequence {}

impl TokenKind for AtomOrSequence {
    const TOKEN: Token = Token::Atom;
    type Payload = crate::Sequence;

    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<crate::Sequence, Error> {
        crate::Sequence::promote(atom)
    }

    #[inline(always)]
    fn from_value(value: Value) -> Result<crate::Sequence, Error> {
        match value {
            Value::Sequence(sequence) => Ok(sequence),
            Value::Atom(atom) => Self::from_atom(atom),
        }
    }
}

impl Operand for AtomOrSequence {
    type Token = Self;
    type Bound = crate::Sequence;

    #[inline(always)]
    fn bind(sequence: crate::Sequence) -> Result<crate::Sequence, Error> {
        Ok(sequence)
    }
}

/// A Number or a Note, whichever arrives, with its literal read as `L`.
///
/// ADR 0021 gives each numeric conversion a monomorphic literal signature and
/// an evaluation that is idempotent over its own result type, so a value
/// already converted, arriving from nested evaluation or a Sequence, is
/// accepted rather than refused. `L` is the literal reading; the payload is
/// either numeric type.
pub struct Numeric<L>(core::marker::PhantomData<L>, core::convert::Infallible);

impl<L: Literal> TokenKind for Numeric<L> {
    const TOKEN: Token = L::TOKEN;
    type Payload = NumericValue;

    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<NumericValue, Error> {
        match atom {
            crate::Atom::Note(note) => Ok(NumericValue::Note(note)),
            crate::Atom::Number(number) => Ok(NumericValue::Number(number)),
            atom => Err(TypeError::Numeric(atom.into()).into()),
        }
    }
}

impl<L: Literal> Operand for Numeric<L> {
    type Token = Self;
    type Bound = NumericValue;

    #[inline(always)]
    fn bind(value: NumericValue) -> Result<NumericValue, Error> {
        Ok(value)
    }
}

/// A domain `D` read from a Number literal, admitted by `D`'s own conversion.
///
/// The domain is a property of `D`, so its bind is `D::try_from`: a Number
/// outside the domain diagnoses with the error that conversion reports.
pub struct Domain<D>(core::marker::PhantomData<D>, core::convert::Infallible);

impl<D> Operand for Domain<D>
where
    D: TryFrom<u8>,
    Error: From<D::Error>,
{
    type Token = Number;
    type Bound = D;

    #[inline(always)]
    fn bind(number: u8) -> Result<D, Error> {
        Ok(D::try_from(number)?)
    }
}

/// The MIDI channel domain over a Number literal.
pub type MidiChannel = Domain<crate::MidiChannel>;

/// The Play velocity domain over a Number literal.
pub type Velocity = Domain<crate::Velocity>;

/// The Control Change controller domain over a Number literal.
pub type Controller = Domain<crate::Controller>;

/// The Control Change value domain over a Number literal.
pub type ControlValue = Domain<crate::ControlValue>;

/// The Pitch Bend low half over a Number literal.
pub type BendLsb = Domain<crate::BendLsb>;

/// The Pitch Bend high half over a Number literal.
pub type BendMsb = Domain<crate::BendMsb>;

/// A Play lifetime over a Number literal. Every byte is a length, so this
/// converts where the MIDI domains validate.
pub enum Length {}

impl Operand for Length {
    type Token = Number;
    type Bound = crate::Length;

    #[inline(always)]
    fn bind(number: u8) -> Result<crate::Length, Error> {
        Ok(crate::Length::from(number))
    }
}

/// Checks every Atom `operand` holds against `O`'s token, in member order.
///
/// A Sequence operand is walked member by member, because an element reading
/// broadcasts across it.
#[inline(always)]
pub(crate) fn check<O: Operand>(operand: &Value) -> Result<(), Error> {
    match operand {
        Value::Atom(atom) => O::Token::from_atom(*atom).map(drop),
        Value::Sequence(sequence) => {
            for atom in sequence {
                O::Token::from_atom(*atom)?;
            }
            Ok(())
        }
    }
}

/// Checks a scalar operand against `O`'s domain, where no element binds it.
///
/// A Sequence operand answers nothing here: at the width this is asked at, it
/// is empty.
#[inline(always)]
pub(crate) fn check_domain<O: Operand>(operand: &Value) -> Result<(), Error> {
    match operand {
        Value::Atom(atom) => bind_atom::<O>(*atom).map(drop),
        Value::Sequence(_) => Ok(()),
    }
}

/// Binds one element Atom to `O`.
#[inline(always)]
pub(crate) fn bind_atom<O: Operand>(atom: crate::Atom) -> Result<O::Bound, Error> {
    O::bind(O::Token::from_atom(atom)?)
}

/// Binds one whole value to `O`.
#[inline(always)]
pub(crate) fn bind_value<O: Operand>(value: Value) -> Result<O::Bound, Error> {
    O::bind(O::Token::from_value(value)?)
}
