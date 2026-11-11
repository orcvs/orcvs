//! The operand types a Function declares, as Rust types.
//!
//! A Function's row in the table names one of these types per role. The type
//! decides four things together: the [`Token`] the Parser reads a literal in
//! that slot as, the payload extraction yields for it, the value the
//! Function body receives, and whether the slot may hold no value
//! ([`Operand::ABSENT`]). [`Operand::bind`] takes exactly the payload its
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
//! The token a signature names is read off [`Operand::Token`] too, so an
//! operand cannot name one token to the Parser while binding another's
//! payload: an `Operand` declares no token of its own to override, `E0438`.
//!
//! ```compile_fail,E0438
//! use lang::operand::{Number, Operand};
//! use lang::{Error, MidiChannel, Token};
//!
//! enum Channel {}
//!
//! impl Operand for Channel {
//!     type Token = Number;
//!     type Bound = MidiChannel;
//!     const TOKEN: Token = Token::Note;
//!
//!     fn bind(number: u8) -> Result<MidiChannel, Error> {
//!         Ok(MidiChannel::try_from(number)?)
//!     }
//! }
//! ```
//!
//! Every operand type here is uninhabited: it is a name for a declaration,
//! never a value.

use crate::{Error, Token, TypeError};

/// What a Parser token is at evaluation: the [`Token`] a literal in the slot
/// is read as, and the payload one operand yields once checked against it.
pub trait TokenKind {
    /// The token a signature names for this slot.
    const TOKEN: Token;

    /// What checking one operand against this token yields.
    type Payload;

    /// Checks one Atom, answering the payload it carries.
    fn from_atom(atom: crate::Atom) -> Result<Self::Payload, Error>;
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

    /// Narrows the checked payload to the operand's declared domain.
    ///
    /// Fallible because a domain may be narrower than its token: a MIDI
    /// channel is read as a Number and is a channel only once this admits it.
    fn bind(payload: <Self::Token as TokenKind>::Payload) -> Result<Self::Bound, Error>;

    /// What a slot that holds no value binds to — blank Cells, or a nested
    /// Function that answered a Clear — or `None` where this operand requires
    /// a value and refuses the slot.
    ///
    /// Whether an operand admits a blank slot is this answer, so the Parser,
    /// which reads a blank slot as absent only where it is admitted, and
    /// binding cannot disagree. Only [`Optional`] admits one. It is a function
    /// rather than the value, so the Function table can ask whether it is
    /// present at compile time without dropping a generic `Bound`.
    const ABSENT: Option<fn() -> Self::Bound> = None;
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

impl Operand for Note {
    type Token = Self;
    type Bound = crate::Note;

    #[inline(always)]
    fn bind(note: crate::Note) -> Result<crate::Note, Error> {
        Ok(note)
    }
}

/// An Interval Set name: the semitones a Chord Play stacks on its root.
pub enum IntervalSet {}

impl TokenKind for IntervalSet {
    const TOKEN: Token = Token::IntervalSet;
    type Payload = crate::IntervalSet;

    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<crate::IntervalSet, Error> {
        match atom {
            crate::Atom::IntervalSet(set) => Ok(set),
            atom => Err(TypeError::IntervalSet(atom.into()).into()),
        }
    }
}

impl Operand for IntervalSet {
    type Token = Self;
    type Bound = crate::IntervalSet;

    #[inline(always)]
    fn bind(set: crate::IntervalSet) -> Result<crate::IntervalSet, Error> {
        Ok(set)
    }
}

/// A Voicing name: how a Chord Play arranges the tones of its Interval Set.
pub enum Voicing {}

impl TokenKind for Voicing {
    const TOKEN: Token = Token::Voicing;
    type Payload = crate::Voicing;

    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<crate::Voicing, Error> {
        match atom {
            crate::Atom::Voicing(voicing) => Ok(voicing),
            atom => Err(TypeError::Voicing(atom.into()).into()),
        }
    }
}

impl Operand for Voicing {
    type Token = Self;
    type Bound = crate::Voicing;

    #[inline(always)]
    fn bind(voicing: crate::Voicing) -> Result<crate::Voicing, Error> {
        Ok(voicing)
    }
}

/// The untyped operand: any Language Unit, carried as the Atom its two Cells
/// spell. Only a Write's `value` declares it.
pub enum Untyped {}

impl TokenKind for Untyped {
    const TOKEN: Token = Token::Untyped;
    type Payload = crate::Atom;

    /// Every Atom is a Language Unit's encoding, so every Atom is carried.
    #[inline(always)]
    fn from_atom(atom: crate::Atom) -> Result<crate::Atom, Error> {
        Ok(atom)
    }
}

impl Operand for Untyped {
    type Token = Self;
    type Bound = crate::Atom;

    #[inline(always)]
    fn bind(atom: crate::Atom) -> Result<crate::Atom, Error> {
        Ok(atom)
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

/// `O`, or no value: a slot that admits blank Cells.
///
/// It is read as `O`'s token and binds a present value through `O::bind`, so
/// a value `O` refuses is refused with `O`'s diagnostic. A slot that holds no
/// value binds to `None` rather than being refused.
pub struct Optional<O>(core::marker::PhantomData<O>, core::convert::Infallible);

impl<O: Operand> Operand for Optional<O> {
    type Token = O::Token;
    type Bound = Option<O::Bound>;

    #[inline(always)]
    fn bind(payload: <O::Token as TokenKind>::Payload) -> Result<Option<O::Bound>, Error> {
        O::bind(payload).map(Some)
    }

    const ABSENT: Option<fn() -> Option<O::Bound>> = Some(|| None);
}

/// Whether `O` admits a slot that holds no value, for the Function table's
/// signature constants.
pub(crate) const fn admits_blank<O: Operand>() -> bool {
    O::ABSENT.is_some()
}

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

/// The two Cells a slot that holds no value shows.
pub(crate) const BLANK_CELLS: &str = "  ";

/// `O`'s refusal of operand slot `slot`, which holds no value.
#[inline(always)]
fn absent<O: Operand>(slot: usize) -> Error {
    let refusal = match O::Token::TOKEN.decode(BLANK_CELLS) {
        Err(refusal) => refusal,
        Ok(atom) => unreachable!("every Atom spells a written Language Unit, not {atom}"),
    };
    crate::ArgumentError::Absent {
        slot,
        refusal: Box::new(refusal),
    }
    .into()
}

/// Checks operand slot `slot` against `O`'s token. A slot that holds no value
/// passes only where `O` admits one.
#[inline(always)]
pub(crate) fn check<O: Operand>(atom: Option<crate::Atom>, slot: usize) -> Result<(), Error> {
    match atom {
        Some(atom) => O::Token::from_atom(atom).map(drop),
        None if admits_blank::<O>() => Ok(()),
        None => Err(absent::<O>(slot)),
    }
}

/// Binds operand slot `slot` to `O`.
#[inline(always)]
pub(crate) fn bind_atom<O: Operand>(
    atom: Option<crate::Atom>,
    slot: usize,
) -> Result<O::Bound, Error> {
    match atom {
        Some(atom) => O::bind(O::Token::from_atom(atom)?),
        None => O::ABSENT
            .map(|blank| blank())
            .ok_or_else(|| absent::<O>(slot)),
    }
}
