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
    Bang,
    Char(char),
    Empty,
    Function(Function),
    Note(Note),
    Number(u8),
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

    /// The Source write this Function performs, or `None` for a Function that
    /// performs none.
    #[inline(always)]
    const fn source_effect(self) -> Option<crate::SourceEffect> {
        match self {
            Self::Effect(EffectKind::SourceWrite(effect)) => Some(effect),
            _ => None,
        }
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
    /// ADR 0004's Source-writing effect: Cells written back into the Source
    /// through one validated Portal bundle, with no Play Command and no value.
    ///
    /// The whole effect rides inside the variant because that is what this type
    /// is for — the effect a Function performs, not merely that it performs
    /// one. The displacement is a whole-Cell offset rather than a named
    /// direction: ADR 0006 states this geometry in coordinates already, north
    /// `(x, y-1)` and west `(x-2, y)`, and a Portal is an output property every
    /// Function has, with `Portal::ordinary_result` one row south as the
    /// default. These Functions decline the default and say by how much.
    SourceWrite(crate::SourceEffect),
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
    // One arm per Source-writing Function rather than one `SourceWrite` arm
    // taking arguments: the kind column of the table is a single identifier,
    // and this macro is already "the one place a new effect is related to the
    // value-or-effect rule". Declaring the whole effect here keeps the table to
    // one column per property and gives the eight of them one home.
    //
    // The two groups are written as one block on purpose. Each `SelfBang` arm
    // sits beside the `Bang` arm that emits it, and the pair differs in the
    // bundle and — in the table's activation column — in where the Turn comes
    // from. That is ADR 0029's asymmetry as two lines rather than as a
    // paragraph, and the horizontal offsets say the rest: a Self-Banging
    // Function moves one Cell, and a Directional Bang Function emits two, which
    // is outside its own Span. A direction name would have hidden the
    // difference the numbers state.
    //
    // The spelling is read from the emitted Function rather than written out,
    // so `^^` has one home whichever of the two rows names it.
    (SelfBangNorth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: -1,
            spelling: Function::SelfBangingNorth.spelling(),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangNorth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: -1,
            spelling: Function::SelfBangingNorth.spelling(),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangSouth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: 1,
            spelling: Function::SelfBangingSouth.spelling(),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangSouth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: 1,
            spelling: Function::SelfBangingSouth.spelling(),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangWest) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: -1,
            rows: 0,
            spelling: Function::SelfBangingWest.spelling(),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangWest) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: -2,
            rows: 0,
            spelling: Function::SelfBangingWest.spelling(),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangEast) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 1,
            rows: 0,
            spelling: Function::SelfBangingEast.spelling(),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangEast) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 2,
            rows: 0,
            spelling: Function::SelfBangingEast.spelling(),
            bundle: crate::SourceBundle::Emit,
        }))
    };
}

/// Where a root Function's activation comes from.
///
/// ADR 0006 states the rule and its one exception together: "An ordinary root
/// Expression is inert until Bang activation. A Self-Banging Function is the
/// explicit exception in activation source: at its own Source-order turn it
/// intrinsically receives Bang activation without creating a Source-resident
/// `**`." That asymmetry is a property of the Function and of nothing else, so
/// it is declared beside every other property rather than derived from one of
/// them. ADR 0029 calls it "the whole reason both forms exist": `^^` and `*^`
/// write the same spelling to the same geometry and differ here.
///
/// It was read off [`FunctionKind`] until the Self-Banging Functions were
/// declared, because a value Function was exactly a Function that took its Turn
/// without a Bang. `^^` answers an effect and takes its Turn anyway, so the two
/// questions came apart and this is the one Tick scheduling means.
///
/// Only a root is asked. A nested Function takes its Turn from the root that
/// owns it, per ADR 0006's "Activation recursively includes nested Functions",
/// so what a nested Function declares here is never read.
#[derive(Clone, Copy)]
enum ActivationSource {
    /// The root takes a Turn at its own Source-order turn with nothing
    /// delivered to it. Every Function that answers a value declares this,
    /// and so does every Self-Banging Function.
    Intrinsic,
    /// The root is inert until a Bang reaches it, which is the ordinary rule.
    Bang,
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
/// the same family and the same signature can differ in it. ADR 0039 is that
/// case built — Clock `~.` and Delay `~*` share the Tick family and share a
/// signature of two Numbers, and one broadcasts while the other refuses.
#[derive(Clone, Copy)]
enum Pervasion {
    Pervasive,
    /// Declared by ADR 0039's Delay `~*` and Euclidean `~%`. Each answers a
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
    ($($variant:ident => ($spelling:literal, $kind:ident, $activation:ident, $pervasion:ident, $answer:ident, $bang:literal, [$($role:ident: $operand:ident),* $(,)?])),+ $(,)?) => {
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

            const fn activation_source(self) -> ActivationSource {
                match self {
                    $(Self::$variant => ActivationSource::$activation,)+
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

            /// Whether this Function, standing at an Expression root, takes a
            /// Turn without a Bang delivered to it.
            ///
            /// This is the question Tick scheduling's activation seed asks, and
            /// the only one that decides it. It is not
            /// [`Function::answers_value`]: those two selected the same rows
            /// until the Self-Banging Functions were declared, and a caller
            /// that kept asking the value question would leave `^^` inert
            /// forever.
            #[inline(always)]
            pub const fn is_intrinsically_active(self) -> bool {
                matches!(self.activation_source(), ActivationSource::Intrinsic)
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

            /// The Source write this Function performs, or `None` for a
            /// Function that performs none.
            ///
            /// `orcvs` resolves the displacement against the Grid, because
            /// ADR 0009 keeps destination resolution there and this crate holds
            /// no Grid. It is read before the Tick, to reserve the Cells the
            /// bundle can reach, and answered again as the interpretation:
            /// these Functions take no operand and read no Context, so the
            /// whole of the effect is declared here.
            #[inline(always)]
            pub const fn source_effect(self) -> Option<crate::SourceEffect> {
                self.kind().source_effect()
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
            /// ordinary value Function that ADR 0039 keeps scalar.
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

            /// Whether this Function declares no operand at all.
            ///
            /// One name for a question both crates ask and neither could
            /// spell the same way: `signature()` is `pub(crate)`, so `orcvs`
            /// reached it through `Tokens::from(..).is_empty()` while `lang`
            /// asked the slice directly. It is a fact about the declaration
            /// and not about a Function group — the Directional Bang
            /// Functions `spatial-tick-planning/06` adds declare no operand
            /// either — so a caller that means "is a Self-Banging Function"
            /// should ask [`Function::source_effect`] instead, and read the
            /// bundle it answers: both groups declare an effect, and it is the
            /// `Advance` a Self-Banging Function declares that tells it from
            /// the `Emit` of a Directional Bang one.
            #[inline(always)]
            pub const fn takes_no_operand(self) -> bool {
                self.signature().is_empty()
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
                    // A row declaring no operand expands to a struct with no
                    // field, so nothing reads or advances the iterator and both
                    // lints fire on a binding the other rows need. The narrow
                    // suppression is on the generated body, where the arity is
                    // a property of the row rather than of this code; a written
                    // `from_operands` that ignored its operands would still be
                    // caught, because no row writes one.
                    #[allow(unused_mut, unused_variables)]
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
    AbsoluteDifference => (".|", Value, Intrinsic, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Add => (".+", Value, Intrinsic, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Clock => ("~.", Value, Intrinsic, Pervasive, Elementwise, false, [rate: Number, modulus: Number]),
    ControlChange => ("!c", TerminalOutput, Bang, Pervasive, Elementwise, false, [channel: MidiChannel, controller: Controller, value: ControlValue]),
    ConvertToNote => (".^", Value, Intrinsic, Pervasive, Elementwise, false, [value: Number]),
    ConvertToNumber => (".v", Value, Intrinsic, Pervasive, Elementwise, false, [value: Note]),
    Delay => ("~*", Value, Intrinsic, Scalar, Atom, true, [rate: Number, modulus: Number]),
    DirectionalBangEast => ("*>", BangEast, Bang, Scalar, Atom, false, []),
    DirectionalBangNorth => ("*^", BangNorth, Bang, Scalar, Atom, false, []),
    DirectionalBangSouth => ("*v", BangSouth, Bang, Scalar, Atom, false, []),
    DirectionalBangWest => ("*<", BangWest, Bang, Scalar, Atom, false, []),
    Divide => ("./", Value, Intrinsic, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Equality => (".=", Value, Intrinsic, Pervasive, Atom, true, [left: Number, right: Number]),
    Euclidean => ("~%", Value, Intrinsic, Scalar, Atom, true, [hits: Number, steps: Number]),
    Maximum => (".>", Value, Intrinsic, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Minimum => (".<", Value, Intrinsic, Pervasive, Elementwise, false, [left: Number, right: Number]),
    Modulo => (".%", Value, Intrinsic, Pervasive, Elementwise, false, [left: Number, right: Number]),
    MonophonicPlay => ("!%", TerminalOutput, Bang, Pervasive, Elementwise, false, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
    Multiply => (".x", Value, Intrinsic, Pervasive, Elementwise, false, [left: Number, right: Number]),
    PitchBend => ("!b", TerminalOutput, Bang, Pervasive, Elementwise, false, [channel: MidiChannel, lsb: BendLsb, msb: BendMsb]),
    RawPlay => ("!>", TerminalOutput, Bang, Pervasive, Elementwise, false, [channel: MidiChannel, velocity: Velocity, note: Note]),
    SelfBangingEast => (">>", SelfBangEast, Intrinsic, Scalar, Atom, false, []),
    SelfBangingNorth => ("^^", SelfBangNorth, Intrinsic, Scalar, Atom, false, []),
    SelfBangingSouth => ("vv", SelfBangSouth, Intrinsic, Scalar, Atom, false, []),
    SelfBangingWest => ("<<", SelfBangWest, Intrinsic, Scalar, Atom, false, []),
    Subtract => (".-", Value, Intrinsic, Pervasive, Elementwise, false, [left: Number, right: Number]),
    TimedPlay => ("!~", TerminalOutput, Bang, Pervasive, Elementwise, false, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
}

/// Which fact an incoming Function changes about the Function a computation is
/// already running, when a Function replacement is refused.
///
/// ADR 0032 fixes a Tick's schedule before any Function evaluates and ADR 0036
/// reserves a result's Cells from the Function found at each anchor, so a
/// replacement is admitted only where the incoming Function agrees with the
/// running one on every fact those two derivations read. This type names the
/// five so that a refusal states which one differed: one diagnostic string
/// answered all of them before, and a byte-identical duplicate of one term
/// stood in the guard unnoticed because no test could tell the terms apart.
///
/// Declaration order is the order the comparison applies, which is the order
/// the guard has always applied. A replacement differing on several facts
/// reports the first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReplacementChange {
    /// Whether the Function answers a value the surrounding Expression can
    /// consume, rather than performing an effect.
    ///
    /// ADR 0028's distinction, and the one Sequence membership, the
    /// Interpreter's nesting guard and tick planning all read. The schedule was
    /// derived from the answer the Function found here gave, so a replacement
    /// that changed it would leave Turns ordered from edges that no longer
    /// describe what runs.
    AnswerKind,
    /// Where the Function's activation comes from: taken on its own, or
    /// delivered by a Bang.
    ///
    /// The question scheduling's activation seed asks, and the term the
    /// Self-Banging Functions added to this guard. It is not
    /// [`ReplacementChange::AnswerKind`] — those two selected the same rows
    /// until `^^` was declared — so admitting a replacement that changed it
    /// would leave the schedule holding a root that now needs no Bang, or one
    /// waiting on a Bang no edge delivers.
    Activation,
    /// Whether the Function can answer Bang even where its operands produce no
    /// result.
    ///
    /// Scheduling reads this declaration to decide which roots can supply
    /// activation, and orders the Functions waiting on them before deciding
    /// whether to perform. A replacement that changed it would leave those
    /// edges standing on a declaration that no longer holds.
    BangEmission,
    /// The Source write the Function declares: the displacement, the spelling
    /// and the bundle, compared whole.
    ///
    /// ADR 0004 has a Source-writing Function state where it writes in its
    /// declaration, and `computations` reads that declaration to reserve the
    /// destination before any Turn. The Portal it declares is therefore read
    /// twice: once by scheduling, which reserves the Cells it resolves to, and
    /// once at the Turn, which writes through it. A replacement that moves the
    /// offset separates the two, so the write lands at Cells no dependency edge
    /// names — the same ADR 0036 defect [`ReplacementChange::Width`] refuses,
    /// stated about direction rather than extent. `^^` and `>>` agree on every
    /// other fact, so this is the only one that tells them apart. The whole
    /// effect is compared rather than its fields because every field of it is
    /// read at the Turn: the offsets resolve the Portal and the bundle decides
    /// how many.
    Write,
    /// How wide a result the Function reserves.
    ///
    /// ADR 0036: a schedule reserves Cells from the Function it found at each
    /// anchor, so a replacement that would widen or narrow that reservation is
    /// refused with the ones that change activation or answer kind. What it is
    /// compared against stays the settled reservation, because the Turns were
    /// ordered from that one and this same guard is what keeps every admitted
    /// replacement inside it.
    ///
    /// The one fact this crate cannot answer: a reservation is derived from the
    /// schedule and from the widths a computation's children settled, and
    /// `lang` holds neither. `orcvs` composes this comparison onto
    /// [`Function::replacing`], appended last, which is where the order this
    /// enum states is completed. No declared pair differs here in any case —
    /// no Function answers a Sequence, so nothing derives a width wider than a
    /// Cell pair — and the variant is stated with the other four so that one
    /// type names all five facts and [`ReplacementChange::ALL`] is a complete
    /// list.
    Width,
}

impl ReplacementChange {
    /// Every fact a Function replacement is refused for changing.
    pub const ALL: &'static [Self] = &[
        Self::AnswerKind,
        Self::Activation,
        Self::BangEmission,
        Self::Write,
        Self::Width,
    ];
}

impl fmt::Display for ReplacementChange {
    /// The fact, worded as CONTEXT.md's Function Replacement entry words it, so
    /// that a refusal reads as the rule's own sentence continued.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AnswerKind => "whether it answers a value",
            Self::Activation => "where its activation comes from",
            Self::BangEmission => "whether it can emit Bang",
            Self::Write => "the Source write it declares",
            Self::Width => "how wide a result it reserves",
        })
    }
}

/// One fact a Function declares, beside the comparison that answers whether a
/// replacement changes it.
///
/// Named so that [`Function::DECLARED_CHANGES`] reads as the table it is rather
/// than as its element type spelled out, which is also what keeps the row shape
/// stated once for the tests that walk it.
type DeclaredChange = (ReplacementChange, fn(Function, Function) -> bool);

impl Function {
    /// The four facts a declaration states, each beside the comparison that
    /// answers whether a replacement changes it, in the order the guard applies
    /// them.
    ///
    /// A table rather than a chain of `||` terms. A chain admits a
    /// byte-identical duplicate as a dead arm Rust does not warn about — one
    /// stood in the replacement guard, reached through a merge — while a table
    /// is a value a test can count, which is what
    /// `each_named_change_is_compared_exactly_once` does.
    /// [`ReplacementChange::Width`] is absent because it is not a fact a
    /// declaration states; `orcvs` appends that comparison after these.
    const DECLARED_CHANGES: &'static [DeclaredChange] = &[
        (ReplacementChange::AnswerKind, |replacement, running| {
            replacement.answers_value() != running.answers_value()
        }),
        (ReplacementChange::Activation, |replacement, running| {
            replacement.is_intrinsically_active() != running.is_intrinsically_active()
        }),
        (ReplacementChange::BangEmission, |replacement, running| {
            replacement.can_emit_bang() != running.can_emit_bang()
        }),
        (ReplacementChange::Write, |replacement, running| {
            replacement.source_effect() != running.source_effect()
        }),
    ];

    /// Which declared fact this Function changes about `running`, the Function
    /// a computation is running, or `None` where it changes none of them.
    ///
    /// The first difference under the table's order rather than every
    /// difference: a replacement is refused once and names one fact. `None` is
    /// not yet an admitted replacement — `orcvs` asks its own width comparison
    /// after this one, which is the fifth fact and the last.
    pub fn replacing(self, running: Self) -> Option<ReplacementChange> {
        Self::DECLARED_CHANGES
            .iter()
            .find(|(_, differs)| differs(self, running))
            .map(|&(change, _)| change)
    }
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
        Atom, BendLsb, BendMsb, ControlValue, Controller, Function, Length, MidiChannel, Note,
        ReplacementChange, Velocity, to_atom_num,
    };

    /// Every ordered pair of Functions, which is what a replacement is: the
    /// incoming Function beside the one a computation is running. A Function
    /// paired with itself is included, because that pair is the admitted
    /// replacement every fixture that replaces `.+` with `.-` relies on.
    fn replacement_pairs() -> impl Iterator<Item = (Function, Function)> {
        Function::ALL.iter().copied().flat_map(|replacement| {
            Function::ALL
                .iter()
                .copied()
                .map(move |running| (replacement, running))
        })
    }

    /// Every declared fact the pair differs on, in the table's order, which is
    /// how a sole difference is told from a first one.
    fn declared_differences(replacement: Function, running: Function) -> Vec<ReplacementChange> {
        Function::DECLARED_CHANGES
            .iter()
            .filter(|(_, differs)| differs(replacement, running))
            .map(|&(change, _)| change)
            .collect()
    }

    #[test]
    fn each_named_change_is_compared_exactly_once() {
        // The test the duplicated term would have failed. `deliver_output`
        // compared the declared Source write twice, byte for byte, and the
        // suite could not see it: a chain of `||` terms has no value to count
        // and a repeated term is not a pattern Rust warns about. A table has
        // both, so each fact is compared once or the count says so.
        //
        // `Width` is expected zero times here because it is not a fact a
        // declaration states. `orcvs` appends that one comparison to this
        // table's answer, so across the two every variant is compared exactly
        // once, and its absence here is the half of that this crate can hold.
        for change in ReplacementChange::ALL.iter().copied() {
            let compared = Function::DECLARED_CHANGES
                .iter()
                .filter(|(named, _)| *named == change)
                .count();
            assert_eq!(
                compared,
                usize::from(change != ReplacementChange::Width),
                "{change:?} is compared {compared} times by the declaration table",
            );
        }
    }

    #[test]
    fn every_declared_change_is_the_first_difference_for_some_pair() {
        // What each variant is worth: a fact some pair of real Functions
        // actually differs on first, so a diagnostic naming it can be produced
        // rather than merely spelled.
        for change in ReplacementChange::ALL.iter().copied() {
            let reached = replacement_pairs()
                .any(|(replacement, running)| replacement.replacing(running) == Some(change));
            if change == ReplacementChange::Width {
                // Not reachable, and not only because this crate cannot answer
                // it. `reserved_for` answers a Row only where a Function
                // declares a Sequence answer or widens over one, and no row in
                // the definitions above declares `Sequence` — so no schedule
                // derived from a declaration ever holds a width wider than a
                // Cell pair, and no pair of Functions differs on it. The one
                // thing that mints a Row is a width a test states, and the
                // fixture that states one refuses to combine it with a stated
                // Function replacement. The variant is stated so the five facts
                // have one home, not because a pair reaches it.
                assert!(!reached, "no declared pair differs on the reserved width");
                continue;
            }
            assert!(
                reached,
                "no pair reports {change:?} as its first difference"
            );
        }

        // The weaker set, recorded because the difference between the two is
        // what made an existing test's comment wrong: only these two facts are
        // ever the *sole* difference between a pair. A pair differing on the
        // answer kind differs on activation or on the write as well, and a pair
        // differing on activation differs on one of the others, so a fixture
        // naming either of those cannot be a fixture that changes one thing.
        let sole = ReplacementChange::ALL
            .iter()
            .copied()
            .filter(|&change| {
                replacement_pairs().any(|(replacement, running)| {
                    declared_differences(replacement, running) == [change]
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            sole,
            vec![ReplacementChange::BangEmission, ReplacementChange::Write],
            "the facts a pair can differ on alone are no longer the two expected",
        );
    }

    #[test]
    fn every_source_writing_function_declares_the_effect_its_spelling_names() {
        // The eight effects live in `function_kind!`, so this is the test that
        // keeps that macro in step with the spellings. It is exhaustive over
        // the Source-writing rows rather than a list, so a ninth is drawn the
        // day it is declared, the way `Function::ALL` keeps every other sweep
        // honest.
        //
        // The pairs are stated together because the pairing is the design.
        // `*^` and `^^` write `^^` and differ in two declared things: the
        // bundle, here, and the activation source, in the sweep below. The
        // horizontal rows carry the third difference the offsets state — a
        // Self-Banging Function moves one Cell and a Directional Bang Function
        // emits two, which is the first Cell outside its own Span — and a test
        // reading `(-1, 0)` where it expected `(-2, 0)` is the whole point of
        // writing them out.
        let mut seen = 0;
        for function in Function::ALL.iter().copied() {
            let Some(effect) = function.source_effect() else {
                continue;
            };
            seen += 1;
            let (columns, rows, spelling, bundle) = match function.spelling() {
                "^^" => (0, -1, "^^", crate::SourceBundle::Advance),
                "vv" => (0, 1, "vv", crate::SourceBundle::Advance),
                "<<" => (-1, 0, "<<", crate::SourceBundle::Advance),
                ">>" => (1, 0, ">>", crate::SourceBundle::Advance),
                "*^" => (0, -1, "^^", crate::SourceBundle::Emit),
                "*v" => (0, 1, "vv", crate::SourceBundle::Emit),
                "*<" => (-2, 0, "<<", crate::SourceBundle::Emit),
                "*>" => (2, 0, ">>", crate::SourceBundle::Emit),
                other => panic!("{other} declares a Source write with no stated effect"),
            };
            assert_eq!(
                effect,
                crate::SourceEffect {
                    columns,
                    rows,
                    spelling,
                    bundle,
                },
                "{function:?} writes something its spelling does not name",
            );
            assert!(
                function.takes_no_operand(),
                "{function:?} declares an operand a Source-writing Function does not take",
            );
        }
        assert_eq!(
            seen, 8,
            "the Source-writing rows are no longer the eight expected"
        );
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
            // Two questions, asked separately, because they stopped being
            // complements. Until the Self-Banging Functions were declared,
            // Terminal Output was the one effect kind, so "answers no value"
            // and "performs Terminal Output" selected the same rows and this
            // test could assert one as the negation of the other. The version
            // that did so named the day it would fail: "the day a Source-writing
            // effect Function of ADR 0004 is declared, this assertion fails and
            // names the Function whose callers must each choose again which
            // question they mean." It failed on `SelfBangingEast`. The rows
            // below are now the witness that a caller asking the narrow
            // question where it means the wide one is wrong today, and not
            // merely wrong in principle.
            //
            // Three questions now, for the same reason: the activation source
            // came apart from the value question on the same four rows. A root
            // that answers a value takes its Turn with nothing delivered to it,
            // and so does a Self-Banging Function that answers an effect, so
            // the third column is not the first one read again.
            let (answers_value, terminal_output, intrinsically_active) = match function {
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
                | Function::Subtract => (true, false, true),
                Function::ControlChange
                | Function::MonophonicPlay
                | Function::PitchBend
                | Function::RawPlay
                | Function::TimedPlay => (false, true, false),
                // An effect that is not Terminal Output, taken without a Bang.
                // These four rows are the whole reason the three predicates are
                // asked separately.
                Function::SelfBangingEast
                | Function::SelfBangingNorth
                | Function::SelfBangingSouth
                | Function::SelfBangingWest => (false, false, true),
                // The same effect kind, waiting for a Bang. These four and the
                // four above are the table's whole record of ADR 0029's
                // asymmetry, and reading them as one group is the mistake this
                // column exists to make impossible.
                Function::DirectionalBangEast
                | Function::DirectionalBangNorth
                | Function::DirectionalBangSouth
                | Function::DirectionalBangWest => (false, false, false),
            };

            assert_eq!(function.answers_value(), answers_value, "{function:?}");
            assert_eq!(
                function.performs_terminal_output(),
                terminal_output,
                "{function:?}"
            );
            assert_eq!(
                function.is_intrinsically_active(),
                intrinsically_active,
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
        // `.`-spelled one. Nor is it a property of a signature: ADR 0039 keeps
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
                // The Source-writing Functions declare no operand, so there is
                // no operand for pervasion to widen over. They are `Scalar` for
                // the reason ADR 0039's two pulses are not: those refuse a
                // Sequence they could have been handed, while these are never
                // handed anything.
                Function::DirectionalBangEast
                | Function::DirectionalBangNorth
                | Function::DirectionalBangSouth
                | Function::DirectionalBangWest
                | Function::SelfBangingEast
                | Function::SelfBangingNorth
                | Function::SelfBangingSouth
                | Function::SelfBangingWest => false,
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
                // ADR 0039's two pulses answer one Atom as well, and now for a
                // different reason than Equality's. Equality broadcasts to
                // find its comparison pairs and reduces them; these refuse a
                // Sequence operand outright, so there is no width to reduce
                // from. They are what the assertion below is about — an answer
                // that does not widen, declared beside the pervasion that
                // cannot widen.
                Function::Delay | Function::Euclidean => (false, false),
                // A Source-writing Function answers an effect, so it answers no
                // Sequence and widens over nothing. It declares the column all
                // the same, because the column says how wide an answer is and
                // scheduling reads it before any Function has evaluated.
                Function::DirectionalBangEast
                | Function::DirectionalBangNorth
                | Function::DirectionalBangSouth
                | Function::DirectionalBangWest
                | Function::SelfBangingEast
                | Function::SelfBangingNorth
                | Function::SelfBangingSouth
                | Function::SelfBangingWest => (false, false),
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
    fn bang_and_the_self_banging_functions_display_with_their_complete_spellings() {
        assert_eq!(Atom::Bang.to_string(), "**");
        assert_eq!(Atom::Function(Function::SelfBangingNorth).to_string(), "^^");
        assert_eq!(Atom::Function(Function::SelfBangingSouth).to_string(), "vv");
        assert_eq!(Atom::Function(Function::SelfBangingWest).to_string(), "<<");
        assert_eq!(Atom::Function(Function::SelfBangingEast).to_string(), ">>");
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
