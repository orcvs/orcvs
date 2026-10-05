use std::fmt;

use crate::{Error, TypeError, midi_note_to_number, midi_number_to_note};

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
/// call site has to remember, so each role is one line here and inherits both
/// the domain and the diagnostic wording.
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
    /// Named for the control it belongs to rather than for MIDI because ADR
    /// 0016 defers OSC and UDP output and notes a type reads better named for
    /// its domain than for its protocol: a control's value is what this is on any
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

/// One parsed value or operation: what an Expression is made of and what a
/// value Function answers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Atom {
    Bang,
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
/// "answers no value" therefore asks [`FunctionKind::answers_value`], not a
/// narrower question such as whether a Function performs Terminal Output,
/// which answers differently for every Source-writing and locking Function.
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

    const fn locks_root(self) -> bool {
        matches!(self, Self::Effect(EffectKind::Lock))
    }
}

/// Which effect a Function that answers an effect performs.
///
/// Named for the kind rather than for the Effect itself, because CONTEXT.md
/// gives Effect to what a Producer contributes to the Tick Plan and this is a
/// property a Function declares before any Tick runs. It is a type of its own
/// rather than a second arm of [`FunctionKind`] so that every effect is
/// declared here, where it answers no value by construction, rather than
/// beside `Value`, where each would have to be re-excluded at every caller.
#[derive(Clone, Copy)]
enum EffectKind {
    /// The `!` family: a Play Command delivered to the Playback Engine, with
    /// nothing written back into the Source.
    TerminalOutput,
    /// A Source-writing effect: Cells written back into the Source through one
    /// validated Portal bundle, with no Play Command and no value.
    ///
    /// The whole effect rides inside the variant because that is what this type
    /// is for — the effect a Function performs, not merely that it performs
    /// one. [`crate::SourceEffect`] says why its displacement is an offset
    /// rather than a named direction.
    SourceWrite(crate::SourceEffect),
    /// A root lock through one Portal, with no Cell write, no Play Command,
    /// and no value. The Portal is the Function's Output Portal.
    Lock,
}

/// The kind column of the canonical definitions, mapped to the declaration it
/// names.
///
/// A row names its effect rather than the word "effect", so the table says what
/// each Function does, and this is the one place a new effect is related to the
/// value-or-effect rule. It is a macro arm so the column stays one identifier
/// per row while the shape it expands to is free to grow.
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
    // one column per property and gives every Source-writing row one home.
    //
    // The two groups are written as one block on purpose. Each `SelfBang` arm
    // sits beside the `Bang` arm that emits it, and the pair differs in the
    // bundle and — in the table's activation column — in where the Turn comes
    // from. That is the activation asymmetry as two lines rather than as a
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
            spelling: Some(Function::SelfBangingNorth.spelling()),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangNorth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: -1,
            spelling: Some(Function::SelfBangingNorth.spelling()),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangSouth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: 1,
            spelling: Some(Function::SelfBangingSouth.spelling()),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangSouth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: 1,
            spelling: Some(Function::SelfBangingSouth.spelling()),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangWest) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: -1,
            rows: 0,
            spelling: Some(Function::SelfBangingWest.spelling()),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangWest) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: -2,
            rows: 0,
            spelling: Some(Function::SelfBangingWest.spelling()),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangEast) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 1,
            rows: 0,
            spelling: Some(Function::SelfBangingEast.spelling()),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangEast) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 2,
            rows: 0,
            spelling: Some(Function::SelfBangingEast.spelling()),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (Halt) => {
        FunctionKind::Effect(EffectKind::Lock)
    };
}

/// Where a root Function's activation comes from.
///
/// An ordinary root Expression is inert until Bang activation. A Self-Banging
/// Function is the one exception: at its own Source-order turn it receives Bang
/// activation without a Source-resident `**`. That asymmetry is a property of
/// the Function and of nothing else, so it is declared beside every other
/// property rather than derived from one of them: `^^` and `*^` write the same
/// spelling to the same geometry and differ here.
///
/// It is not [`FunctionKind`]. `^^` answers an effect and takes its Turn
/// without a Bang, so whether a Function answers a value does not say where
/// its activation comes from, and this is the question Tick scheduling means.
///
/// Only a root is asked. A nested Function takes its Turn from the root that
/// owns it, so what a nested Function declares here is never read.
#[derive(Clone, Copy)]
enum ActivationSource {
    /// The root takes a Turn at its own Source-order turn with nothing
    /// delivered to it. Every Function that answers a value declares this,
    /// and so does every Self-Banging Function.
    Intrinsic,
    /// The root is inert until a Bang reaches it, which is the ordinary rule.
    Bang,
}

// The declared type of a Portal input, mapped to the input it decodes. Number
// is the one type a Portal input declares, so a row naming another type does
// not match an arm and fails to compile.
macro_rules! portal_input {
    ($role:literal, Number) => {
        crate::PortalInput::number($role)
    };
}

// One Function's operand struct and its binds, from the row's roles and their
// operand types.
//
// A row that declares no operand gets nothing: there is nothing to extract, so
// no body reads such a struct. A Portal input on such a row matches no arm and
// fails to compile, because its Portal is bound beside the operands.
macro_rules! operands {
    ($variant:ident, []) => {};
    ($variant:ident, [$($role:ident: $operand:ty),+] $(, $portal_role:literal: $portal_type:ident)?) => {
        pub(crate) struct $variant {
            $(pub(crate) $role: <$operand as crate::operand::Operand>::Bound,)+
        }

        impl crate::stack::Operands for $variant {
            const FUNCTION: crate::Function = crate::Function::$variant;

            #[inline(always)]
            fn check(operands: &[crate::Atom]) -> Result<(), crate::Error> {
                let [$($role),+] = operands else {
                    return Err(crate::stack::arity(Self::FUNCTION, operands.len()));
                };
                $(crate::operand::check::<$operand>(*$role)?;)+
                Ok(())
            }

            // Field initialisers evaluate in signature order, so the first
            // operand outside its domain is the one that diagnoses.
            #[inline(always)]
            fn bind(operands: &[crate::Atom]) -> Result<Self, crate::Error> {
                let [$($role),+] = operands else {
                    return Err(crate::stack::arity(Self::FUNCTION, operands.len()));
                };
                Ok(Self {
                    $($role: crate::operand::bind_atom::<$operand>(*$role)?,)+
                })
            }
        }

        $(impl crate::portal::PortalOperands for $variant {
            const PORTAL: crate::PortalInput = portal_input!($portal_role, $portal_type);
        })?
    };
}

// Checks and binds `values` through `$variant`'s operand struct, for the table
// sweep in the tests below. A row that declares no operand has nothing to bind.
#[cfg(test)]
macro_rules! bind_declared {
    ($variant:ident, [], $values:expr) => {
        Ok(())
    };
    ($variant:ident, [$($role:ident),+], $values:expr) => {{
        use crate::stack::Operands as _;
        let values: &[crate::Atom] = $values;
        operands::$variant::check(values).and_then(|()| operands::$variant::bind(values).map(drop))
    }};
}

macro_rules! define_functions {
    ($($variant:ident => ($spelling:literal, $kind:ident, $activation:ident, $bang:literal, [$($role:ident: $operand:ident),* $(,)?] $(, portal: $portal_role:literal : $portal_type:ident)?)),+ $(,)?) => {
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

            /// The Function `spelling` spells, if it spells one.
            ///
            /// Borrows the spelling and builds nothing on refusal, so a caller
            /// that only asks whether two Cells spell a Function allocates
            /// nothing to learn that they do not. [`TryFrom<&str>`](TryFrom)
            /// answers through this and builds its
            /// [`SyntaxError::UnknownFunction`](crate::SyntaxError::UnknownFunction)
            /// only where a refusal is reported.
            ///
            /// Two definitions sharing a spelling would generate a duplicate arm
            /// here and leave the later variant unreachable from the parser.
            /// Denying the lint turns that into a compile error rather than a
            /// warning the build would accept.
            #[deny(unreachable_patterns)]
            #[inline(always)]
            pub(crate) fn from_spelling(spelling: &str) -> Option<Self> {
                match spelling {
                    $($spelling => Some(Self::$variant),)+
                    _ => None,
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

            /// Whether this Function answers a language value the surrounding
            /// Expression can consume, rather than performing an effect.
            ///
            /// This is the one question the Interpreter's nesting guard and tick
            /// planning's activation gate each ask, so a Function declared with
            /// an effect kind joins both by its definition alone. Neither asks
            /// which effect: ADR 0029 records that they would each be borrowing
            /// a narrower question that happens to coincide with the one they
            /// mean.
            #[inline(always)]
            pub const fn answers_value(self) -> bool {
                self.kind().answers_value()
            }

            /// Whether this Function, standing at an Expression root, takes a
            /// Turn without a Bang delivered to it.
            ///
            /// This is the question Tick scheduling's activation seed asks, and
            /// the only one that decides it. It is not
            /// [`Function::answers_value`]: `^^` answers an effect and takes
            /// its Turn anyway, so a caller that asked the value question would
            /// leave `^^` inert forever.
            #[inline(always)]
            pub const fn is_intrinsically_active(self) -> bool {
                matches!(self.activation_source(), ActivationSource::Intrinsic)
            }

            /// Whether this Function performs the Terminal Output effect: a
            /// Play Command delivered to the Playback Engine, with nothing
            /// written back into the Source.
            ///
            /// This is the narrow question, and it is asked only where the rule
            /// is about Terminal Output rather than about answering an effect.
            /// Having no Cell destination is such a rule: a Source-writing
            /// Function writes where its [`crate::SourceEffect`] resolves, and
            /// Halt locks through its Output Portal, so a gate that refused a
            /// Portal to every Function answering an effect would deny them
            /// their destinations. Ask
            /// [`Function::answers_value`] instead wherever the rule is that
            /// nothing consumes the answer.
            #[inline(always)]
            pub const fn performs_terminal_output(self) -> bool {
                self.kind().performs_terminal_output()
            }

            /// The Source write this Function performs, or `None` for a
            /// Function that performs none.
            ///
            /// `orcvs` resolves the displacement against the Grid, as
            /// [`crate::SourceEffect`] describes. Advance and Emit declare the
            /// whole write: scheduling reserves the displaced Span, and the
            /// Interpreter answers the same declaration.
            #[inline(always)]
            pub const fn source_effect(self) -> Option<crate::SourceEffect> {
                self.kind().source_effect()
            }

            /// Whether this Function locks the Expression root at its Output
            /// Portal.
            #[inline(always)]
            pub const fn locks_root(self) -> bool {
                self.kind().locks_root()
            }

            /// Whether this Function can return Bang, even when the current
            /// operands produce no result. Scheduling uses this declaration to
            /// wait for activation producers before deciding whether to perform.
            pub const fn can_emit_bang(self) -> bool {
                match self {
                    $(Self::$variant => $bang,)+
                }
            }

            /// Whether this Function declares no operand at all.
            ///
            /// One name for a question both crates ask: `signature()` is
            /// `pub(crate)`, so `orcvs` cannot ask the slice. It is a fact
            /// about the declaration and not about a Function group — the
            /// Jumps, Halt and the Directional Bang Functions declare no
            /// operand either — so a caller that means "is a Self-Banging
            /// Function" should ask [`Function::source_effect`] instead, and
            /// read the bundle it answers: both groups declare an effect, and
            /// it is the `Advance` a Self-Banging Function declares that tells
            /// it from the `Emit` of a Directional Bang one.
            #[inline(always)]
            pub const fn takes_no_operand(self) -> bool {
                self.signature().is_empty()
            }

            /// The Portal input resolved at Turn, after cell operand validation.
            pub const fn portal_input(self) -> Option<crate::PortalInput> {
                match self {
                    $($(Self::$variant => Some(portal_input!($portal_role, $portal_type)),)?)+
                    _ => None,
                }
            }

            /// The token each declared operand is read as, in signature order.
            pub(crate) const fn signature(self) -> &'static [crate::Token] {
                match self {
                    $(Self::$variant => const {
                        &[$(<<crate::operand::$operand as crate::operand::Operand>::Token as crate::operand::TokenKind>::TOKEN,)*]
                    },)+
                }
            }
        }

        /// The operands each Function declares, one struct per Function that
        /// declares any, with a field named for the role that position plays.
        ///
        /// A Function body destructures the struct its Function declares, so an
        /// operand's position is written once — here, beside the role name and
        /// the type — and never restated in the body that reads it. Transposing
        /// two same-typed operands is therefore an edit to the declaration
        /// rather than a silent edit inside a body.
        pub(crate) mod operands {
            $(operands! {
                $variant,
                [$($role: crate::operand::$operand),*]
                $(, $portal_role: $portal_type)?
            })+
        }

        #[cfg(test)]
        impl Function {
            /// Checks and binds `values` as this Function's declared operands,
            /// through the reading its operand struct names.
            fn bind_declared(self, values: &[Atom]) -> Result<(), Error> {
                match self {
                    $(Self::$variant => bind_declared!($variant, [$($role),*], values),)+
                }
            }
        }

        impl TryFrom<&str> for Function {
            type Error = Error;

            #[inline(always)]
            fn try_from(spelling: &str) -> Result<Self, Self::Error> {
                Self::from_spelling(spelling).ok_or_else(|| {
                    crate::SyntaxError::UnknownFunction(spelling.to_string()).into()
                })
            }
        }
    };
}

define_functions! {
    AbsoluteDifference => (".|", Value, Intrinsic, false, [left: Number, right: Number]),
    Add => (".+", Value, Intrinsic, false, [left: Number, right: Number]),
    Clock => ("~.", Value, Intrinsic, false, [rate: Number, modulus: Number]),
    ControlChange => ("!c", TerminalOutput, Bang, false, [channel: MidiChannel, controller: Controller, value: ControlValue]),
    ConvertToNote => (".^", Value, Intrinsic, false, [value: Number]),
    ConvertToNumber => (".v", Value, Intrinsic, false, [value: Note]),
    Delay => ("~*", Value, Intrinsic, true, [rate: Number, modulus: Number]),
    DirectionalBangEast => ("*>", BangEast, Bang, false, []),
    DirectionalBangNorth => ("*^", BangNorth, Bang, false, []),
    DirectionalBangSouth => ("*v", BangSouth, Bang, false, []),
    DirectionalBangWest => ("*<", BangWest, Bang, false, []),
    Divide => ("./", Value, Intrinsic, false, [left: Number, right: Number]),
    Equality => (".=", Value, Intrinsic, true, [left: Number, right: Number]),
    Euclidean => ("~%", Value, Intrinsic, true, [hits: Number, steps: Number]),
    Halt => ("*!", Halt, Bang, false, []),
    Increment => ("~+", Value, Intrinsic, false, [step: Number, modulus: Number], portal: "previous value": Number),
    Interpolation => ("~>", Value, Intrinsic, false, [rate: Number, target: Number], portal: "previous value": Number),
    JumpEast => ("&>", Value, Intrinsic, true, []),
    JumpNorth => ("&^", Value, Intrinsic, true, []),
    JumpSouth => ("&v", Value, Intrinsic, true, []),
    JumpWest => ("&<", Value, Intrinsic, true, []),
    Maximum => (".>", Value, Intrinsic, false, [left: Number, right: Number]),
    Minimum => (".<", Value, Intrinsic, false, [left: Number, right: Number]),
    Modulo => (".%", Value, Intrinsic, false, [left: Number, right: Number]),
    MonophonicPlay => ("!%", TerminalOutput, Bang, false, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
    Multiply => (".x", Value, Intrinsic, false, [left: Number, right: Number]),
    PitchBend => ("!b", TerminalOutput, Bang, false, [channel: MidiChannel, lsb: BendLsb, msb: BendMsb]),
    Random => ("~?", Value, Intrinsic, false, [seed: Number, minimum: Number, maximum: Number]),
    RawPlay => ("!>", TerminalOutput, Bang, false, [channel: MidiChannel, velocity: Velocity, note: Note]),
    SelfBangingEast => (">>", SelfBangEast, Intrinsic, false, []),
    SelfBangingNorth => ("^^", SelfBangNorth, Intrinsic, false, []),
    SelfBangingSouth => ("vv", SelfBangSouth, Intrinsic, false, []),
    SelfBangingWest => ("<<", SelfBangWest, Intrinsic, false, []),
    Subtract => (".-", Value, Intrinsic, false, [left: Number, right: Number]),
    TimedPlay => ("!~", TerminalOutput, Bang, false, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
}

/// Declares every fact a Function replacement is refused for changing, minting
/// the enum, the complete list of its variants, and the wording a refusal
/// states, from one row each.
///
/// The `Display` `match` is exhaustive, so the wording cannot be forgotten, and
/// generating [`ReplacementChange::ALL`] from the same rows makes leaving a
/// variant out of the list not expressible rather than merely untested — no
/// test can see a variant that has its `Display` arm and no list entry. It is
/// what `define_functions!` does for [`Function::ALL`], and the reason neither
/// list needs a completeness test.
macro_rules! define_replacement_changes {
    ($($(#[$doc:meta])* $variant:ident => $wording:literal),+ $(,)?) => {
        /// Which fact an incoming Function changes about the Function a
        /// computation is already running, when a Function replacement is
        /// refused.
        ///
        /// ADR 0032 fixes a Tick's schedule before any Function evaluates and
        /// reserves a result's Cells from the Function found at each anchor, so
        /// a replacement is admitted only where the incoming Function agrees
        /// with the running one on every fact those derivations read. This type
        /// names the four so that a refusal states which one differed, and so
        /// that a test can tell the terms apart.
        ///
        /// Declaration order is the order the comparison applies. A replacement
        /// differing on several facts reports the first.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum ReplacementChange {
            $($(#[$doc])* $variant,)+
        }

        impl ReplacementChange {
            /// Every fact a Function replacement is refused for changing,
            /// generated from the declarations above and so complete by
            /// construction.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }

        impl fmt::Display for ReplacementChange {
            /// The fact, worded as CONTEXT.md's Function Replacement entry
            /// words it, so that a refusal reads as the rule's own sentence
            /// continued.
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(match self {
                    $(Self::$variant => $wording,)+
                })
            }
        }
    };
}

define_replacement_changes! {
    /// Whether the Function answers a value the surrounding Expression can
    /// consume, rather than performing an effect.
    ///
    /// The value-or-effect distinction, and the one the Interpreter's nesting
    /// guard and tick planning both read. The schedule was
    /// derived from the answer the Function found here gave, so a replacement
    /// that changed it would leave Turns ordered from edges that no longer
    /// describe what runs.
    AnswerKind => "whether it answers a value",
    /// Where the Function's activation comes from: taken on its own, or
    /// delivered by a Bang.
    ///
    /// The question scheduling's activation seed asks. It is not
    /// [`ReplacementChange::AnswerKind`] — `^^` answers an effect and takes its
    /// Turn without a Bang — so admitting a replacement that changed it
    /// would leave the schedule holding a root that now needs no Bang, or one
    /// waiting on a Bang no edge delivers.
    Activation => "where its activation comes from",
    /// Whether the Function can answer Bang even where its operands produce no
    /// result.
    ///
    /// Scheduling reads this declaration to decide which roots can supply
    /// activation, and orders the Functions waiting on them before deciding
    /// whether to perform. A replacement that changed it would leave those
    /// edges standing on a declaration that no longer holds.
    BangEmission => "whether it can emit Bang",
    /// The Source write the Function declares: the displacement, the spelling
    /// and the bundle, compared whole.
    ///
    /// A Source-writing Function states where it writes in its declaration, and
    /// `computations` reads that declaration to reserve the destination before
    /// any Turn. The Portal it declares is therefore read twice: once by
    /// scheduling, which reserves the Cells it resolves to, and once at the
    /// Turn, which writes through it. A replacement that moves the offset
    /// separates the two, so the write lands at Cells no dependency edge names.
    /// `^^` and `>>` agree on every other fact,
    /// so this is the only one that tells them apart. The whole effect is
    /// compared rather than its fields because every field of it is read at the
    /// Turn: the offsets resolve the Portal and the bundle decides how many.
    Write => "the Source write it declares",
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
    /// byte-identical duplicate as a dead arm Rust does not warn about, while a
    /// table is a value a test can count, which is what
    /// `each_named_change_is_compared_exactly_once` does.
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
                || replacement.output_portal() != running.output_portal()
                || replacement.input_portal() != running.input_portal()
        }),
    ];

    /// The Output Portal this Function names, or `None` when it names none.
    ///
    /// Terminal Output and Source-writing Functions name none here: the former
    /// has no Cell destination, and the latter keeps its destinations on
    /// [`Function::source_effect`]. Every other Function names one row south
    /// unless it is a Jump, which names the Portal its direction writes
    /// through.
    pub const fn output_portal(self) -> Option<crate::PortalCoords> {
        if self.performs_terminal_output() || self.source_effect().is_some() {
            return None;
        }
        Some(match self {
            Self::JumpEast => crate::PortalCoords {
                columns: 2,
                rows: 0,
            },
            Self::JumpWest => crate::PortalCoords {
                columns: -2,
                rows: 0,
            },
            Self::JumpNorth => crate::PortalCoords {
                columns: 0,
                rows: -1,
            },
            _ => crate::PortalCoords::SOUTH,
        })
    }

    /// The Input Portal this Function names, or `None` when it names none.
    ///
    /// Jump names the Portal opposite its Output Portal. Increment and
    /// Interpolation name one row south, the same site as their Output Portal.
    pub const fn input_portal(self) -> Option<crate::PortalCoords> {
        match self {
            Self::JumpEast => Some(crate::PortalCoords {
                columns: -2,
                rows: 0,
            }),
            Self::JumpWest => Some(crate::PortalCoords {
                columns: 2,
                rows: 0,
            }),
            Self::JumpNorth => Some(crate::PortalCoords::SOUTH),
            Self::JumpSouth => Some(crate::PortalCoords {
                columns: 0,
                rows: -1,
            }),
            _ if self.portal_input().is_some() => Some(crate::PortalCoords::SOUTH),
            _ => None,
        }
    }

    /// Whether this Function copies a Language Unit from its Input Portal.
    ///
    /// Jump names an Input Portal and binds no typed Portal input. Increment
    /// and Interpolation name the same south site as a Number Portal input, so
    /// they are not this: the Cells they read are a value, not a Language Unit.
    pub const fn copies_language_unit(self) -> bool {
        self.input_portal().is_some() && self.portal_input().is_none()
    }

    /// Which declared fact this Function changes about `running`, the Function
    /// a computation is running, or `None` where it changes none of them.
    ///
    /// The first difference under the table's order rather than every
    /// difference: a replacement is refused once and names one fact. `None` is
    /// an admitted replacement: every value Function reserves the same one
    /// Atom's Cell pair, so no fact beyond these four can separate two
    /// Functions a schedule has already ordered.
    pub fn replacing(self, running: Self) -> Option<ReplacementChange> {
        Self::DECLARED_CHANGES
            .iter()
            .find(|(_, differs)| differs(self, running))
            .map(|&(change, _)| change)
    }
}

/// The Note Atom two Cells spell, borrowing them and building nothing on
/// refusal. [`to_atom_note`] answers through this and builds its
/// [`TypeError::Note`] only where a refusal is reported.
pub(crate) fn note_atom_from_spelling(s: &str) -> Option<Atom> {
    midi_note_to_number(s).map(|n| Atom::Note(Note(n)))
}

/// The Number Atom two Cells spell, borrowing them and building nothing on
/// refusal. [`to_atom_num`] answers through this and builds its
/// [`TypeError::Number`] only where a refusal is reported.
pub(crate) fn number_atom_from_spelling(s: &str) -> Option<Atom> {
    crate::number_from_spelling(s).map(Atom::Number)
}

#[inline(always)]
pub fn to_atom_note(s: &str) -> Result<Atom, Error> {
    note_atom_from_spelling(s).ok_or_else(|| TypeError::Note(s.to_string()).into())
}

#[inline(always)]
pub fn to_atom_num(s: &str) -> Result<Atom, Error> {
    number_atom_from_spelling(s).ok_or_else(|| TypeError::Number(s.to_string()).into())
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

    /// The lowest value a literal of `token` carries, as the operand the
    /// Parser would hand a signature position declared as that token.
    fn lowest(token: crate::Token) -> Atom {
        match token {
            crate::Token::Number => Atom::Number(0),
            crate::Token::Note => Atom::Note(Note::try_from(0).expect("00 is a Note")),
            other => panic!("no operand is declared as {other:?}"),
        }
    }

    #[test]
    fn every_declared_operand_binds_the_lowest_value_its_token_reads() {
        // The type checker holds a token and its bind to one payload type, but
        // not a bind to accepting the values that token reads: a domain that
        // refused every literal in its slot would compile and diagnose at
        // evaluation, every Tick. Every declared domain contains its
        // token's minimum, so binding each signature from its own tokens'
        // lowest values finds that the day the row is declared; a domain that
        // excluded its minimum would need its own witness here.
        for function in Function::ALL.iter().copied() {
            let values: Vec<Atom> = function.signature().iter().copied().map(lowest).collect();

            assert!(
                function.bind_declared(&values).is_ok(),
                "{function:?} declares a token whose lowest value its bind refuses: {:?}",
                function.bind_declared(&values),
            );
        }
    }

    #[test]
    fn each_named_change_is_compared_exactly_once() {
        // A chain of `||` terms has no value to count and a repeated term is
        // not a pattern Rust warns about. A table has both, so each fact is
        // compared once or the count says so.
        for change in ReplacementChange::ALL.iter().copied() {
            let compared = Function::DECLARED_CHANGES
                .iter()
                .filter(|(named, _)| *named == change)
                .count();
            assert_eq!(
                compared, 1,
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
            assert!(
                reached,
                "no pair reports {change:?} as its first difference"
            );
        }

        // The weaker set: only these two facts are ever the *sole* difference
        // between a pair. A pair differing on the answer kind differs on
        // activation or on the write as well, and a pair differing on
        // activation differs on one of the others, so a fixture naming either
        // of those cannot be a fixture that changes one thing.
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
        // The Source-writing effects live in `function_kind!`, so this is the
        // test that keeps that macro in step with the spellings. It is
        // exhaustive over the Source-writing rows rather than a list, so a
        // later row is drawn the day it is declared, the way `Function::ALL`
        // keeps every other sweep honest.
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
                "^^" => (0, -1, Some("^^"), crate::SourceBundle::Advance),
                "vv" => (0, 1, Some("vv"), crate::SourceBundle::Advance),
                "<<" => (-1, 0, Some("<<"), crate::SourceBundle::Advance),
                ">>" => (1, 0, Some(">>"), crate::SourceBundle::Advance),
                "*^" => (0, -1, Some("^^"), crate::SourceBundle::Emit),
                "*v" => (0, 1, Some("vv"), crate::SourceBundle::Emit),
                "*<" => (-2, 0, Some("<<"), crate::SourceBundle::Emit),
                "*>" => (2, 0, Some(">>"), crate::SourceBundle::Emit),
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
    fn exactly_the_bang_capable_functions_declare_that_they_can_emit_bang() {
        // Equality, Delay and Euclidean answer a Bang or Absence as their
        // result, and a Jump copies a Bang from its Input Portal. Tick
        // scheduling trusts the declaration to decide which roots can supply
        // activation, so the list is stated whole — a Function that began
        // returning Bang without declaring it would build no activation edge,
        // and the neighbouring terminal root would fall silent with no
        // diagnostic anywhere.
        // `only_a_function_that_declares_it_ever_answers_with_bang` is the
        // other half for operand-reading Functions; the Jumps are exercised on
        // their own path, because a Jump reads its Portal.
        assert_eq!(
            Function::ALL
                .iter()
                .copied()
                .filter(|function| function.can_emit_bang())
                .collect::<Vec<_>>(),
            vec![
                Function::Delay,
                Function::Equality,
                Function::Euclidean,
                Function::JumpEast,
                Function::JumpNorth,
                Function::JumpSouth,
                Function::JumpWest,
            ]
        );
    }

    #[test]
    fn every_function_names_its_portals() {
        use crate::PortalCoords;

        for function in Function::ALL.iter().copied() {
            let output = function.output_portal();
            let input = function.input_portal();
            match function {
                Function::JumpEast => {
                    assert_eq!(
                        output,
                        Some(PortalCoords {
                            columns: 2,
                            rows: 0
                        })
                    );
                    assert_eq!(
                        input,
                        Some(PortalCoords {
                            columns: -2,
                            rows: 0
                        })
                    );
                }
                Function::JumpWest => {
                    assert_eq!(
                        output,
                        Some(PortalCoords {
                            columns: -2,
                            rows: 0
                        })
                    );
                    assert_eq!(
                        input,
                        Some(PortalCoords {
                            columns: 2,
                            rows: 0
                        })
                    );
                }
                Function::JumpNorth => {
                    assert_eq!(
                        output,
                        Some(PortalCoords {
                            columns: 0,
                            rows: -1
                        })
                    );
                    assert_eq!(input, Some(PortalCoords::SOUTH));
                }
                Function::JumpSouth => {
                    assert_eq!(output, Some(PortalCoords::SOUTH));
                    assert_eq!(
                        input,
                        Some(PortalCoords {
                            columns: 0,
                            rows: -1
                        })
                    );
                }
                Function::Increment | Function::Interpolation => {
                    assert_eq!(output, Some(PortalCoords::SOUTH));
                    assert_eq!(input, Some(PortalCoords::SOUTH));
                }
                Function::Halt => {
                    assert_eq!(output, Some(PortalCoords::SOUTH));
                    assert_eq!(input, None);
                    assert!(function.locks_root());
                }
                _ if function.performs_terminal_output() || function.source_effect().is_some() => {
                    assert_eq!(output, None, "{function:?}");
                    assert_eq!(input, None, "{function:?}");
                }
                _ => {
                    assert_eq!(output, Some(PortalCoords::SOUTH), "{function:?}");
                    assert_eq!(input, None, "{function:?}");
                    assert!(!function.locks_root(), "{function:?}");
                }
            }
        }
    }

    #[test]
    fn increment_replacing_add_is_a_write() {
        assert_eq!(
            Function::Increment.replacing(Function::Add),
            Some(ReplacementChange::Write)
        );
    }

    #[test]
    fn each_midi_domain_type_accepts_exactly_its_protocol_range() {
        // The domain is a property of the type, so the conversion has to hold
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
        // A length spans `00`–`FF`, so unlike every MIDI domain beside it there
        // is no value to refuse — and none to alter either.
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
        // ADR 0028 gives every Function exactly one of two answers, and the
        // declaration is read rather than derived: an enumerated check naming
        // spellings would be a second place to keep in step with the
        // definitions, and reading the `!` family prefix would classify the
        // effect Functions spelled `*^` or `*!` as answering a value. So this
        // match is exhaustive over `Function` with no wildcard: a Function
        // added later has to be classified
        // here as well as in the table, and a copied row that answers the wrong
        // kind fails here rather than standing where an operand belongs.
        for function in Function::ALL.iter().copied() {
            // Three questions, asked separately, because none is the
            // complement of another. Terminal Output is one effect kind among
            // several, so "answers no value" and "performs Terminal Output"
            // select different rows, and the rows below are the witness that a
            // caller asking the narrow question where it means the wide one is
            // wrong. The activation source is not the value question either: a
            // root that answers a value takes its Turn with nothing delivered
            // to it, and so does a Self-Banging Function that answers an
            // effect, so the third column is not the first one read again.
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
                | Function::Increment
                | Function::Interpolation
                | Function::JumpEast
                | Function::JumpNorth
                | Function::JumpSouth
                | Function::JumpWest
                | Function::Maximum
                | Function::Minimum
                | Function::Modulo
                | Function::Multiply
                | Function::Random
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
                // four above are the table's whole record of the activation
                // asymmetry, and reading them as one group is the mistake this
                // column exists to make impossible.
                Function::DirectionalBangEast
                | Function::DirectionalBangNorth
                | Function::DirectionalBangSouth
                | Function::DirectionalBangWest => (false, false, false),
                // The same activation as a Directional Bang Function — inert
                // until Bang — and a different effect kind: a lock, not a
                // Source write.
                Function::Halt => (false, false, false),
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
    fn bang_and_the_self_banging_functions_display_with_their_complete_spellings() {
        assert_eq!(Atom::Bang.to_string(), "**");
        assert_eq!(Atom::Function(Function::SelfBangingNorth).to_string(), "^^");
        assert_eq!(Atom::Function(Function::SelfBangingSouth).to_string(), "vv");
        assert_eq!(Atom::Function(Function::SelfBangingWest).to_string(), "<<");
        assert_eq!(Atom::Function(Function::SelfBangingEast).to_string(), ">>");
        assert_eq!(Atom::Function(Function::JumpNorth).to_string(), "&^");
        assert_eq!(Atom::Function(Function::JumpSouth).to_string(), "&v");
        assert_eq!(Atom::Function(Function::JumpWest).to_string(), "&<");
        assert_eq!(Atom::Function(Function::JumpEast).to_string(), "&>");
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
