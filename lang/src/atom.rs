use std::fmt;

use crate::{Error, TypeError, midi_note_to_number, midi_number_to_note, str_to_num};

pub type Atoms = Vec<Atom>;

/// The MIDI note domain: `00`–`7F`.
///
/// Ordered as well as compared, because the Playback Engine keys a Timed
/// Play's ownership by the two domain types it sounds on, channel and note.
/// Monophonic Play keys by channel alone and carries the note in the claim
/// instead, so a Note reaches the schedule either way. Ordering a note by its
/// number is the protocol's own order, and carrying the key as the domain
/// types keeps the engine from re-deriving either domain from a byte.
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
/// the two together key a Timed Play's ownership inside the Playback Engine,
/// and the channel alone keys a Monophonic Play's.
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
            /// A constant rather than a conversion a caller has to unwrap.
            /// `Velocity::ZERO` is the one with callers: the Playback Engine
            /// delivers a scheduled Note Off as MIDI's zero-velocity stop, and
            /// a `try_from(0)` there would be an unreachable failure path
            /// inside a Tick. Every other role inherits the constant from this
            /// macro rather than because a caller asked for it.
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
    /// The controller a Control Change addresses.
    Controller => "controller",
    /// The value a Control Change sends to the controller beside it.
    ///
    /// Named for the control it belongs to because `Value` is the Sequence
    /// value model's, and named for that rather than for MIDI because ADR 0016
    /// defers OSC and UDP output and notes a type reads better named for its
    /// domain than for its protocol: a control's value is what this is on any
    /// wire, and `MidiValue` would have to be renamed the day a second one
    /// arrives.
    ControlValue => "value",
    /// The low seven bits of a Pitch Bend, which precede the high seven on the
    /// wire.
    ///
    /// A bend is one fourteen-bit value the protocol splits in two, and Orcvs
    /// sends the halves as the Source wrote them rather than assembling them
    /// into a number it would have to take apart again. `Lsb` alone would name
    /// the half of nothing in particular; the bend is what makes this half
    /// meaningful, and what keeps it out of the next fourteen-bit pair's
    /// positions.
    BendLsb => "lsb",
    /// The high seven bits of a Pitch Bend.
    BendMsb => "msb",
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
    /// Every Activation, in the order the variants are declared.
    ///
    /// `Function::ALL` is generated from the Function table, and the sweeps
    /// that read it stay honest when a Function is added. The Activations are
    /// a hand-written enum, so this is the same guarantee written out once:
    /// a test that reads it covers a fifth Activation the day one is declared,
    /// rather than passing while never testing it.
    pub const ALL: &'static [Self] = &[Self::North, Self::South, Self::West, Self::East];

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
/// ADR 0028 states that an instruction answers either a value or an effect,
/// never both and never neither, so that is the distinction this enum draws. A
/// Value Function answers with a language value the surrounding Expression can
/// consume. A Function that answers an effect performs something and answers
/// with nothing, so it is valid only where no value is required. Every
/// Function states which it is in the canonical definitions below, and nothing
/// else is allowed to decide: a spelling table that disagreed with the
/// interpreter would silently make an effect Function usable as an operand.
///
/// The effect a Function performs is carried inside the variant rather than
/// being the variant, because ADR 0029 records that Terminal Output is one
/// effect kind and not the definition of effect. Every caller that means
/// "answers no value" therefore asks [`FunctionKind::answers_value`] and stays
/// correct when the Source-writing effects of ADR 0004 arrive, instead of
/// borrowing a narrower question that coincides with it only while Terminal
/// Output is the one effect defined.
#[derive(Clone, Copy)]
enum FunctionKind {
    Value,
    Effect(EffectKind),
}

impl FunctionKind {
    #[inline(always)]
    const fn answers_value(self) -> bool {
        matches!(self, Self::Value)
    }

    #[inline(always)]
    const fn performs_terminal_output(self) -> bool {
        matches!(self, Self::Effect(EffectKind::TerminalOutput))
    }
}

/// Which effect a Function that answers an effect performs.
///
/// Named for the kind rather than for the Effect itself, because CONTEXT.md
/// gives Effect to what a Producer contributes to the Tick Plan and this is a
/// property a Function declares before any Tick runs. One variant today: it is
/// a type of its own rather than a second arm of [`FunctionKind`] so that the
/// Halt, Directional Bang, and Jump Functions of ADR 0004 are added here, where
/// they answer no value by construction, rather than beside `Value`, where each
/// would have to be re-excluded at every caller.
#[derive(Clone, Copy)]
enum EffectKind {
    /// The `!` family of ADR 0016: a Play Command delivered to the Playback
    /// Engine, with nothing written back into the Source.
    TerminalOutput,
}

/// The kind column of the canonical definitions, mapped to the declaration it
/// names.
///
/// A row names its effect rather than the word "effect", so the table says what
/// each Function does, and this is the one place a new effect is related to the
/// value-or-effect rule. It is a macro arm for the same reason `operand_token!`
/// is: the column stays one identifier per row while the shape it expands to is
/// free to grow.
macro_rules! function_kind {
    (Value) => {
        FunctionKind::Value
    };
    (TerminalOutput) => {
        FunctionKind::Effect(EffectKind::TerminalOutput)
    };
}

/// Whether a Function extends across a Sequence operand or requires a scalar
/// one.
///
/// ADR 0007 makes pervasive extension the rule for the Atomic Functions, and
/// ADR 0012 makes Increment and Interpolation exceptions to it because element
/// identity across Ticks would need hidden state their one visible Atom cannot
/// hold. An exception that arrived by omission would therefore be silent, so
/// this is declared beside every other property of a Function rather than
/// inferred from a family prefix or assumed from a signature: two Functions of
/// the same family and the same signature can differ in it. ADR 0036 is that
/// case built — Clock `~.` and Delay `~*` share the Tick family and share a
/// signature of two Numbers, and one broadcasts while the other refuses.
#[derive(Clone, Copy)]
enum Pervasion {
    Pervasive,
    /// Declared by ADR 0036's Delay `~*` and Euclidean `~%`. Each answers a
    /// pulse, so a widened operation would need one answer per element and an
    /// element that does not Bang has only the Absence Marker to offer, which
    /// ADR 0025 refuses as a Sequence member; reducing the elements to one
    /// answer instead would fix a meaning for layered rhythms that could not
    /// later be changed without breaking Source, so the operand is refused.
    ///
    /// ADR 0012's Increment and Interpolation are the other exception this
    /// column exists for — they state it on their own terms and are unbuilt —
    /// and each arrives by declaring this rather than by a check written beside
    /// its body.
    Scalar,
}

/// How wide an answer a Function gives: one Atom, as many Atoms as its operands
/// carry, or a Sequence whatever they carry.
///
/// [`Pervasion`] above says whether a Sequence operand is admitted at all; this
/// says what reaches the answer when one is. The two are independent, and
/// Equality is why. ADR 0011 makes it "a whole-value predicate": it broadcasts
/// to find its comparison pairs, so it is `Pervasive`, and it still returns one
/// scalar Bang or no value at all, so its answer is one Atom however wide its
/// operands were. Deriving the width from the pervasion column would make
/// Equality answer a Sequence it never returns, and deriving it from the family
/// prefix would do the same to every `.`-spelled row.
///
/// Tick scheduling reads this, per ADR 0036, to decide how many Cells one
/// result can reach before any Function has evaluated. That is why the answer
/// is declared rather than observed: a schedule is fixed before a width exists.
#[derive(Clone, Copy)]
enum Answer {
    /// One Atom, whatever its operands carry. Equality is the row that declares
    /// this today; ADR 0012's Increment and Interpolation, which refuse a
    /// Sequence operand outright, will declare it beside `Pervasion::Scalar`.
    Atom,
    /// One answer per element, so as wide as the widest operand: an Atom for
    /// Atom operands and a Sequence of the same length for a Sequence one. This
    /// is ADR 0007's pervasive extension seen from the result, so a row
    /// declaring it must also declare `Pervasion::Pervasive` — a Function that
    /// refuses a Sequence operand can never widen over one — and a test below
    /// holds the two columns to that.
    Elementwise,
    /// A Sequence, whatever its operands carry. No row declares this today: it
    /// is the answer ADR 0007's Range, Reverse, Concatenate, and Replace give,
    /// and none of the four is built. Declaring it now is what lets ADR 0036's
    /// scheduling reserve Cells for a width nothing can yet produce, so those
    /// Functions arrive as one table row each rather than as a scheduling
    /// change. `expect` rather than `allow`, so the first of them turns this
    /// attribute into the error that deletes it.
    #[expect(
        dead_code,
        reason = "the Sequence Functions of ADR 0007 are unbuilt: this is the answer they will declare"
    )]
    Sequence,
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
    (Controller) => {
        crate::Token::Number
    };
    (ControlValue) => {
        crate::Token::Number
    };
    (BendLsb) => {
        crate::Token::Number
    };
    (BendMsb) => {
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
    (Controller) => {
        crate::Controller
    };
    (ControlValue) => {
        crate::ControlValue
    };
    (BendLsb) => {
        crate::BendLsb
    };
    (BendMsb) => {
        crate::BendMsb
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
    // Every domain declared over a Number binds the same way, so the arm is
    // written once and the declared types forward to it. That buys brevity and
    // nothing else, and in particular it is not what keeps one role from
    // binding another role's domain. `define_functions!` initialises each field
    // of the generated operand struct straight from this macro, so an arm that
    // converted to the wrong domain of the same token fails to compile at that
    // field — `expected BendMsb, found BendLsb` — whether the body is written
    // here once or repeated six times. Six near-identical bodies would have
    // been exactly as safe and merely longer.
    (@number_domain $domain:ty, $operand:expr, $role:ident) => {
        match $operand {
            Some(crate::Atom::Number(value)) => {
                <$domain>::try_from(value).map_err(crate::Error::from)
            }
            _ => unreachable!(concat!(
                "typed extraction guarantees a Number for the ",
                stringify!($role),
                " operand"
            )),
        }
    };
    (MidiChannel, $operand:expr, $role:ident) => {
        operand_bind!(@number_domain crate::MidiChannel, $operand, $role)
    };
    (Velocity, $operand:expr, $role:ident) => {
        operand_bind!(@number_domain crate::Velocity, $operand, $role)
    };
    (Controller, $operand:expr, $role:ident) => {
        operand_bind!(@number_domain crate::Controller, $operand, $role)
    };
    (ControlValue, $operand:expr, $role:ident) => {
        operand_bind!(@number_domain crate::ControlValue, $operand, $role)
    };
    (BendLsb, $operand:expr, $role:ident) => {
        operand_bind!(@number_domain crate::BendLsb, $operand, $role)
    };
    (BendMsb, $operand:expr, $role:ident) => {
        operand_bind!(@number_domain crate::BendMsb, $operand, $role)
    };
    // The one declared domain that is the whole byte, so this converts where
    // the others validate. It has an arm of its own rather than forwarding to
    // `@number_domain` above: every byte is a length, so `Length` converts
    // infallibly and has no `TryFrom` to share. What makes a length a length is
    // the type it arrives as, not a check it passed.
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
    ($($variant:ident => ($spelling:literal, $kind:ident, $pervasion:ident, $answer:ident, $bang:literal, [$($role:ident: $operand:ident),* $(,)?])),+ $(,)?) => {
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
                    $(Self::$variant => function_kind!($kind),)+
                }
            }

            const fn pervasion(self) -> Pervasion {
                match self {
                    $(Self::$variant => Pervasion::$pervasion,)+
                }
            }

            const fn answer(self) -> Answer {
                match self {
                    $(Self::$variant => Answer::$answer,)+
                }
            }

            /// Whether this Function answers a language value the surrounding
            /// Expression can consume, rather than performing an effect.
            ///
            /// This is the one question the Interpreter's nesting guard, tick
            /// planning's activation gate, and Sequence membership each ask, so
            /// a Function declared with an effect kind joins all three by its
            /// definition alone. None of them asks which effect: ADR 0029
            /// records that they would each be borrowing a narrower question
            /// that happens to coincide with the one they mean.
            #[inline(always)]
            pub const fn answers_value(self) -> bool {
                self.kind().answers_value()
            }

            /// Whether this Function performs the Terminal Output effect of
            /// ADR 0016: a Play Command delivered to the Playback Engine, with
            /// nothing written back into the Source.
            ///
            /// This is the narrow question, and it is asked only where the
            /// rule is about Terminal Output rather than about answering an
            /// effect. Having no Cell destination is such a rule: ADR 0004
            /// gives a Source-writing Function a validated write bundle and
            /// ADR 0009 lets it resolve multiple Portals, so a gate that
            /// refused a Portal to every Function answering an effect would
            /// deny the Halt, Directional Bang, and Jump Functions their
            /// destinations. Ask [`Function::answers_value`] instead wherever
            /// the rule is that nothing consumes the answer.
            #[inline(always)]
            pub const fn performs_terminal_output(self) -> bool {
                self.kind().performs_terminal_output()
            }

            /// Whether this Function can return Bang, even when the current
            /// operands produce no result. Scheduling uses this declaration to
            /// wait for activation producers before deciding whether to perform.
            pub const fn can_emit_bang(self) -> bool {
                match self {
                    $(Self::$variant => $bang,)+
                }
            }

            /// Whether this Function extends pervasively across a Sequence
            /// operand instead of requiring one Atom per position.
            ///
            /// The Operand Stack asks this before it decides the shape of an
            /// operation, so broadcasting is something a Function declares
            /// rather than something the shape of its operands decides for it:
            /// a Sequence reaching a Scalar Function is refused with the same
            /// diagnostic whether that Function is Terminal or, like Delay, an
            /// ordinary value Function that ADR 0036 keeps scalar.
            #[inline(always)]
            pub const fn is_pervasive(self) -> bool {
                matches!(self.pervasion(), Pervasion::Pervasive)
            }

            /// Whether this Function answers a Sequence whatever its operands
            /// carry.
            ///
            /// Per ADR 0036 this is one of the two questions Tick scheduling
            /// asks to decide how many Cells a result can reach, and it is the
            /// one that needs no operand: a Range answers a Sequence from two
            /// Number bounds. No Function answers `true` today, so every
            /// scheduled result is still as wide as its operands make it.
            #[inline(always)]
            pub const fn answers_sequence(self) -> bool {
                matches!(self.answer(), Answer::Sequence)
            }

            /// Whether a Sequence operand widens this Function's answer into a
            /// Sequence, rather than being consumed into one Atom.
            ///
            /// The other question ADR 0036's scheduling asks, and the one that
            /// separates the Atomic Functions from Equality: each of them
            /// broadcasts over a Sequence operand, and Equality alone answers
            /// one Atom when it has. A Function that refuses a Sequence operand
            /// outright answers `false` here as well, because there is no
            /// operand to widen from.
            #[inline(always)]
            pub const fn widens_over_a_sequence_operand(self) -> bool {
                matches!(self.answer(), Answer::Elementwise)
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
                    let mut stack = Stack::new(16);

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
    AbsoluteDifference => (".|", Value, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Add => (".+", Value, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Clock => ("~.", Value, Pervasive, Elementwise, false, [rate: Number, modulus: Number]),
    ControlChange => ("!c", TerminalOutput, Pervasive, Elementwise, false, [channel: MidiChannel, controller: Controller, value: ControlValue]),
    ConvertToNote => (".^", Value, Pervasive, Elementwise, false, [value: Number]),
    ConvertToNumber => (".v", Value, Pervasive, Elementwise, false, [value: Note]),
    Delay => ("~*", Value, Scalar, Atom, true, [rate: Number, modulus: Number]),
    Divide => ("./", Value, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Equality => (".=", Value, Pervasive, Atom, true, [left: Number, right: Number]),
    Euclidean => ("~%", Value, Scalar, Atom, true, [hits: Number, steps: Number]),
    Maximum => (".>", Value, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Minimum => (".<", Value, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Modulo => (".%", Value, Pervasive, Elementwise, false, [left: Number, right: Number]),
    MonophonicPlay => ("!%", TerminalOutput, Pervasive, Elementwise, false, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
    Multiply => (".x", Value, Pervasive, Elementwise, false, [left: Number, right: Number]),
    PitchBend => ("!b", TerminalOutput, Pervasive, Elementwise, false, [channel: MidiChannel, lsb: BendLsb, msb: BendMsb]),
    RawPlay => ("!>", TerminalOutput, Pervasive, Elementwise, false, [channel: MidiChannel, velocity: Velocity, note: Note]),
    Subtract => (".-", Value, Pervasive, Elementwise, false, [left: Number, right: Number]),
    TimedPlay => ("!~", TerminalOutput, Pervasive, Elementwise, false, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
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
    use super::{
        Activation, Atom, BendLsb, BendMsb, ControlValue, Controller, Function, Length,
        MidiChannel, Note, Velocity, to_atom_num,
    };

    #[test]
    fn every_activation_variant_is_named_in_all() {
        // The match is the guard: a fifth Activation stops this compiling,
        // which is what a hand-written `ALL` needs in place of the generation
        // that keeps `Function::ALL` honest. The membership check is the other
        // half — a variant declared but left out of `ALL` fails here rather
        // than going untested wherever `ALL` is swept.
        for activation in [
            Activation::North,
            Activation::South,
            Activation::West,
            Activation::East,
        ] {
            let spelling = match activation {
                Activation::North => "^^",
                Activation::South => "vv",
                Activation::West => "<<",
                Activation::East => ">>",
            };
            assert_eq!(activation.spelling(), spelling);
            assert!(
                Activation::ALL.contains(&activation),
                "{activation:?} is not named in Activation::ALL",
            );
        }
        assert_eq!(Activation::ALL.len(), 4);
    }

    #[test]
    fn exactly_the_pulse_answering_functions_declare_that_they_can_emit_bang() {
        // Three Functions answer a Bang rather than a value: ADR 0011's
        // Equality and ADR 0012's Delay and Euclidean. Tick scheduling trusts
        // the declaration to decide which roots can supply activation, so the
        // list is stated whole — a fourth Function that began answering Bang
        // without declaring it would build no activation edge, and the
        // neighbouring terminal root would fall silent with no diagnostic
        // anywhere. `only_a_function_that_declares_it_ever_answers_with_bang`
        // is the other half, checking each declaration against what the
        // Interpreter actually answers.
        assert_eq!(
            Function::ALL
                .iter()
                .copied()
                .filter(|function| function.can_emit_bang())
                .collect::<Vec<_>>(),
            vec![Function::Delay, Function::Equality, Function::Euclidean]
        );
    }

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
            assert_eq!(
                Controller::try_from(value).is_ok(),
                value <= 0x7F,
                "{value:02X}"
            );
            assert_eq!(
                ControlValue::try_from(value).is_ok(),
                value <= 0x7F,
                "{value:02X}"
            );
            assert_eq!(
                BendLsb::try_from(value).is_ok(),
                value <= 0x7F,
                "{value:02X}"
            );
            assert_eq!(
                BendMsb::try_from(value).is_ok(),
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
            assert_eq!(Controller::try_from(value).unwrap().value(), value);
            assert_eq!(ControlValue::try_from(value).unwrap().value(), value);
            assert_eq!(BendLsb::try_from(value).unwrap().value(), value);
            assert_eq!(BendMsb::try_from(value).unwrap().value(), value);
            assert_eq!(Note::try_from(value).unwrap().value(), value);
        }
    }

    #[test]
    fn a_rejected_data_byte_names_the_operand_role_that_supplied_it() {
        // The role word is a property of the type rather than an argument at
        // the call site, so which word a rejected byte answers with is fixed
        // where the role is minted and not at whatever body constructed it.
        // Every role is asserted by its own type: a role that inherited
        // another's word would be a diagnostic naming an operand the Source
        // never wrote.
        assert_eq!(
            Velocity::try_from(0x80).unwrap_err().to_string(),
            "MIDI velocity 80 is outside the range 00\u{2013}7F"
        );
        assert_eq!(
            Controller::try_from(0x80).unwrap_err().to_string(),
            "MIDI controller 80 is outside the range 00\u{2013}7F"
        );
        assert_eq!(
            ControlValue::try_from(0x80).unwrap_err().to_string(),
            "MIDI value 80 is outside the range 00\u{2013}7F"
        );
        assert_eq!(
            BendLsb::try_from(0x80).unwrap_err().to_string(),
            "MIDI lsb 80 is outside the range 00\u{2013}7F"
        );
        assert_eq!(
            BendMsb::try_from(0x80).unwrap_err().to_string(),
            "MIDI msb 80 is outside the range 00\u{2013}7F"
        );

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
        assert_eq!(Function::MonophonicPlay.to_string(), "!%");
    }

    #[test]
    fn control_change_and_pitch_bend_display_with_the_terminal_output_family_spellings() {
        assert_eq!(Function::ControlChange.to_string(), "!c");
        assert_eq!(Function::PitchBend.to_string(), "!b");
    }

    #[test]
    fn every_function_declares_whether_it_answers_a_value_or_an_effect() {
        // ADR 0028 gives every Function exactly one of two answers, and ADR
        // 0029 requires the declaration to be read rather than derived: an
        // enumerated check naming spellings would be a second place to keep in
        // step with the definitions, and reading the `!` family prefix would
        // classify the Source-writing Functions of ADR 0004 as answering a
        // value the day they are spelled `*^` or `*!`. So this match is
        // exhaustive over `Function` with no wildcard, the way pervasion's is
        // below: a Function added later has to be classified here as well as in
        // the table, and a copied row that answers the wrong kind fails here
        // rather than standing where an operand belongs.
        for function in Function::ALL.iter().copied() {
            let expected = match function {
                Function::AbsoluteDifference
                | Function::Add
                | Function::Clock
                | Function::ConvertToNote
                | Function::ConvertToNumber
                | Function::Delay
                | Function::Divide
                | Function::Equality
                | Function::Euclidean
                | Function::Maximum
                | Function::Minimum
                | Function::Modulo
                | Function::Multiply
                | Function::Subtract => true,
                Function::ControlChange
                | Function::MonophonicPlay
                | Function::PitchBend
                | Function::RawPlay
                | Function::TimedPlay => false,
            };

            assert_eq!(function.answers_value(), expected, "{function:?}");

            // Terminal Output is the one effect kind declared today, so the
            // two classifications are exact complements. That coincidence is
            // why the narrow question needs a predicate of its own rather than
            // a negation of the wide one: the day a Source-writing effect
            // Function of ADR 0004 is declared, this assertion fails and names
            // the Function whose callers must each choose again which question
            // they mean.
            assert_eq!(
                function.performs_terminal_output(),
                !expected,
                "{function:?}"
            );
        }
    }

    #[test]
    fn every_function_declares_whether_it_extends_over_a_sequence() {
        // ADR 0007 makes pervasive extension the rule for Atomic Functions and
        // ADR 0012 makes Increment and Interpolation exceptions to it, so the
        // property cannot be inferred from a family prefix. ADR 0030 settles
        // the other family the same way: the Terminal Output Functions extend
        // as well, so pervasion is not a property of answering a value either,
        // and a `!`-spelled row is no more predictable from its spelling than a
        // `.`-spelled one. Nor is it a property of a signature: ADR 0036 keeps
        // Delay and Euclidean scalar while Clock, which shares Delay's family
        // and its two Number operands, broadcasts. It is declared per Function
        // instead, and this match is exhaustive over `Function` with no
        // wildcard: a Function added later has to be classified here as well as
        // in the table, so neither an omission nor a copied row can make it
        // broadcast by accident.
        for function in Function::ALL.iter().copied() {
            let expected = match function {
                Function::AbsoluteDifference
                | Function::Add
                | Function::Clock
                | Function::ControlChange
                | Function::ConvertToNote
                | Function::ConvertToNumber
                | Function::Divide
                | Function::Equality
                | Function::Maximum
                | Function::Minimum
                | Function::Modulo
                | Function::MonophonicPlay
                | Function::Multiply
                | Function::PitchBend
                | Function::RawPlay
                | Function::Subtract
                | Function::TimedPlay => true,
                Function::Delay | Function::Euclidean => false,
            };

            assert_eq!(function.is_pervasive(), expected, "{function:?}");
        }
    }

    #[test]
    fn every_function_declares_how_wide_an_answer_it_gives() {
        // ADR 0036 schedules a result's Cells before any Function evaluates, so
        // the width of an answer is declared rather than observed. Equality is
        // the row that makes this a column of its own: ADR 0011 makes it a
        // whole-value predicate that broadcasts to find its comparison pairs
        // and still answers one scalar, so it is `Pervasive` like the other
        // ten `.`-spelled rows and is the only one of them whose answer stays
        // one Atom. Neither the family prefix nor the pervasion column can tell
        // it apart, which is why this match is exhaustive with no wildcard.
        //
        // That exhaustiveness is also what holds ADR 0036's premise that no
        // Function answers a Sequence yet. Every arm below declares `sequence`
        // false, and a row added to the table has to be given an arm here, so
        // the day ADR 0007's Range is declared this test fails and names it.
        //
        // The `!`-spelled rows widen too, per ADR 0030: one Expression answers
        // an ordered group of Play Commands over a Sequence operand, and that
        // widening reaches the Playback Engine rather than a Cell. Scheduling
        // reads their declaration all the same — `reserved_for` asks every node
        // it derives a reservation for, Terminal Output included, without first
        // asking what kind of answer its Function gives. What that reservation
        // cannot do is reach a Cell: a Terminal Output Function is given no
        // Portal, so it has no destination for a reservation to be measured
        // from and no write for one to order. They declare the column because
        // it says how wide an answer is, not how wide a write is.
        for function in Function::ALL.iter().copied() {
            let (sequence, widens) = match function {
                Function::Equality => (false, false),
                // ADR 0036's two pulses answer one Atom as well, and now for a
                // different reason than Equality's. Equality broadcasts to
                // find its comparison pairs and reduces them; these refuse a
                // Sequence operand outright, so there is no width to reduce
                // from. They are what the assertion below is about — an answer
                // that does not widen, declared beside the pervasion that
                // cannot widen.
                Function::Delay | Function::Euclidean => (false, false),
                Function::AbsoluteDifference
                | Function::Add
                | Function::Clock
                | Function::ControlChange
                | Function::ConvertToNote
                | Function::ConvertToNumber
                | Function::Divide
                | Function::Maximum
                | Function::Minimum
                | Function::Modulo
                | Function::MonophonicPlay
                | Function::Multiply
                | Function::PitchBend
                | Function::RawPlay
                | Function::Subtract
                | Function::TimedPlay => (false, true),
            };

            assert_eq!(function.answers_sequence(), sequence, "{function:?}");
            assert_eq!(
                function.widens_over_a_sequence_operand(),
                widens,
                "{function:?}"
            );

            // The two columns are independent but not free of each other: a
            // Function that refuses a Sequence operand has none to widen from,
            // so `Elementwise` beside `Pervasion::Scalar` would declare a
            // widening that can never happen. ADR 0012's Increment is the row
            // that will first be able to break this, and it should fail here
            // rather than reserve Cells for a Sequence it refuses.
            assert!(
                !function.widens_over_a_sequence_operand() || function.is_pervasive(),
                "{function:?} widens over an operand it refuses",
            );
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
