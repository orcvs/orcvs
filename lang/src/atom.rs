use std::fmt;

use crate::{Error, TypeError, midi_note_to_number};

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
            /// compares a Play Command's velocity against it to recognise the
            /// Source's explicit stop, and a `try_from(0)` there would be an
            /// unreachable failure path inside a Tick. Every other role
            /// inherits the constant from this macro rather than because a
            /// caller asked for it.
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

/// One closed set of two-Cell names, as a Copy enum with the spelling each
/// variant fills.
///
/// Each generated enum names its members once, in one table, so a name cannot
/// have a spelling and no variant or the reverse. A spelling is checked at
/// compile time to be two printable Cells, and two rows sharing a spelling are
/// an unreachable arm the denied lint turns into a compile error.
macro_rules! define_named_literal {
    ($(#[$doc:meta])* $name:ident { $($variant:ident => $spelling:literal),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $($variant,)+
        }

        impl $name {
            /// Every name of this type, in table order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            /// The two Cells this name spells.
            pub const fn cells(self) -> Spelling {
                match self {
                    $(Self::$variant => const { Spelling::checked($spelling) },)+
                }
            }

            /// The name `spelling` spells, if it spells one. A half-blank pair
            /// spells none, so partial input is not read as a name.
            #[deny(unreachable_patterns)]
            pub(crate) fn from_spelling(spelling: &str) -> Option<Self> {
                match spelling {
                    $($spelling => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

/// One [`IntervalSet`] row per name, giving the spelling and the semitones
/// above the root beside each other.
macro_rules! define_interval_sets {
    ($(#[$doc:meta])* $($variant:ident => ($spelling:literal, [$($semitone:literal),+])),+ $(,)?) => {
        define_named_literal! {
            $(#[$doc])*
            IntervalSet {
                $($variant => $spelling,)+
            }
        }

        impl IntervalSet {
            /// The semitones above the root, lowest first.
            pub const fn semitones(self) -> &'static [u8] {
                match self {
                    $(Self::$variant => &[$($semitone),+],)+
                }
            }
        }
    };
}

define_interval_sets! {
    /// A list of semitones above a root, named by a chord or a scale.
    ///
    /// One type serves both, so an operand taking an Interval Set accepts a
    /// chord or a scale. Every name fills both Cells, a one-character chord
    /// name taking the filler `_`, so a half-typed pair stays recognisably
    /// partial.
    Major => ("M_", [0, 4, 7]),
    MajorSeventh => ("M7", [0, 4, 7, 11]),
    MajorNinth => ("M9", [0, 4, 7, 11, 14]),
    MajorThirteenth => ("MD", [0, 4, 7, 11, 14, 21]),
    MajorSixth => ("M6", [0, 4, 7, 9]),
    SixNine => ("69", [0, 4, 7, 9, 14]),
    AugmentedMajorSeventh => ("+M", [0, 4, 8, 11]),
    Minor => ("m_", [0, 3, 7]),
    MinorSeventh => ("m7", [0, 3, 7, 10]),
    MinorMajorSeventh => ("mM", [0, 3, 7, 11]),
    MinorSixth => ("m6", [0, 3, 7, 9]),
    MinorNinth => ("m9", [0, 3, 7, 10, 14]),
    MinorEleventh => ("mB", [0, 3, 7, 10, 14, 17]),
    MinorThirteenth => ("mD", [0, 3, 7, 10, 14, 21]),
    Diminished => ("o_", [0, 3, 6]),
    DiminishedSeventh => ("o7", [0, 3, 6, 9]),
    HalfDiminished => ("h7", [0, 3, 6, 10]),
    DominantFlatNinth => ("b9", [0, 4, 7, 10, 13]),
    DominantSeventh => ("7_", [0, 4, 7, 10]),
    DominantNinth => ("9_", [0, 4, 7, 10, 14]),
    DominantThirteenth => ("13", [0, 4, 7, 10, 14, 21]),
    SuspendedSecond => ("s2", [0, 2, 7]),
    SuspendedFourth => ("s4", [0, 5, 7]),
    Augmented => ("+_", [0, 4, 8]),
    AddNine => ("a9", [0, 4, 7, 14]),
    MajorScale => ("MA", [0, 2, 4, 5, 7, 9, 11]),
    NaturalMinorScale => ("mi", [0, 2, 3, 5, 7, 8, 10]),
    DorianScale => ("do", [0, 2, 3, 5, 7, 9, 10]),
    PhrygianScale => ("ph", [0, 1, 3, 5, 7, 8, 10]),
    LydianScale => ("ly", [0, 2, 4, 6, 7, 9, 11]),
    MixolydianScale => ("mx", [0, 2, 4, 5, 7, 9, 10]),
    LocrianScale => ("lo", [0, 1, 3, 5, 6, 8, 10]),
    HarmonicMinorScale => ("hm", [0, 2, 3, 5, 7, 8, 11]),
    MelodicMinorScale => ("mm", [0, 2, 3, 5, 7, 9, 11]),
    MajorPentatonicScale => ("p5", [0, 2, 4, 7, 9]),
    MinorPentatonicScale => ("p3", [0, 3, 5, 7, 10]),
    BluesScale => ("bl", [0, 3, 5, 6, 7, 10]),
    WholeToneScale => ("wt", [0, 2, 4, 6, 8, 10]),
    ChromaticScale => ("ch", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
}

/// Whether two semitone lists are the same list, in a form a `const` item can
/// evaluate.
const fn same_semitones(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

// Every Interval Set is non-empty, starts at its root, rises strictly, and is
// the only one naming its list, so the tones a set places above a root include
// the root, are distinct and rise, and a list of semitones names at most one
// set. A table row breaking any of these fails the build.
const _: () = {
    let mut index = 0;
    while index < IntervalSet::ALL.len() {
        let semitones = IntervalSet::ALL[index].semitones();
        assert!(
            !semitones.is_empty() && semitones[0] == 0,
            "an Interval Set starts at its root"
        );
        let mut tone = 1;
        while tone < semitones.len() {
            assert!(
                semitones[tone - 1] < semitones[tone],
                "an Interval Set rises strictly"
            );
            tone += 1;
        }
        let mut other = index + 1;
        while other < IntervalSet::ALL.len() {
            assert!(
                !same_semitones(semitones, IntervalSet::ALL[other].semitones()),
                "no two Interval Sets name the same semitones"
            );
            other += 1;
        }
        index += 1;
    }
};

define_named_literal! {
    /// How a chord's tones are arranged once its Interval Set has placed them.
    Voicing {
        Close => "CL",
        Open => "OP",
        DropTwo => "D2",
        DropThree => "D3",
        DropFive => "D5",
        DropSix => "D6",
        Shell => "SH",
        Rootless => "RL",
    }
}

/// Whether two spellings are the same two Cells, in a form a `const` item can
/// evaluate.
const fn same_cells(left: Spelling, right: Spelling) -> bool {
    let [left_west, left_east] = left.cells();
    let [right_west, right_east] = right.cells();
    left_west == right_west && left_east == right_east
}

/// Whether any spelling in `left` is also one in `right`.
const fn shares_a_spelling(left: &[Spelling], right: &[Spelling]) -> bool {
    let mut index = 0;
    while index < left.len() {
        let mut other = 0;
        while other < right.len() {
            if same_cells(left[index], right[other]) {
                return true;
            }
            other += 1;
        }
        index += 1;
    }
    false
}

/// The spelling of every name of type `$name`, in table order, as an array a
/// `const` item can build.
macro_rules! spellings_of {
    ($name:ty) => {{
        const COUNT: usize = <$name>::ALL.len();
        let mut spellings = [Spelling::checked("  "); COUNT];
        let mut index = 0;
        while index < COUNT {
            spellings[index] = <$name>::ALL[index].cells();
            index += 1;
        }
        spellings
    }};
}

// A spelling names at most one Function, Interval Set or Voicing. Uniqueness
// within one type is the denied `unreachable_patterns` lint in each
// `from_spelling`; this covers the pairs across types, because a reading that
// tries one type before another would otherwise shadow the later name without
// a diagnostic. Numbers and Notes are not checked here: they are decoded by
// operand position, and `69`, `13` and `D2` deliberately spell one as well.
const _: () = {
    let functions = &spellings_of!(Function);
    let interval_sets = &spellings_of!(IntervalSet);
    let voicings = &spellings_of!(Voicing);
    assert!(
        !shares_a_spelling(interval_sets, voicings),
        "an Interval Set and a Voicing share a spelling"
    );
    assert!(
        !shares_a_spelling(functions, interval_sets),
        "a Function and an Interval Set share a spelling"
    );
    assert!(
        !shares_a_spelling(functions, voicings),
        "a Function and a Voicing share a spelling"
    );
};

/// One parsed value or operation: what an Expression is made of and what a
/// value Function answers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Atom {
    Bang,
    Function(Function),
    IntervalSet(IntervalSet),
    Note(Note),
    Number(u8),
    Voicing(Voicing),
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
            spelling: Some(Function::SelfBangingNorth.cells()),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangNorth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: -1,
            spelling: Some(Function::SelfBangingNorth.cells()),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangSouth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: 1,
            spelling: Some(Function::SelfBangingSouth.cells()),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangSouth) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: 1,
            spelling: Some(Function::SelfBangingSouth.cells()),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangWest) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: -1,
            rows: 0,
            spelling: Some(Function::SelfBangingWest.cells()),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangWest) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: -2,
            rows: 0,
            spelling: Some(Function::SelfBangingWest.cells()),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    (SelfBangEast) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 1,
            rows: 0,
            spelling: Some(Function::SelfBangingEast.cells()),
            bundle: crate::SourceBundle::Advance,
        }))
    };
    (BangEast) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 2,
            rows: 0,
            spelling: Some(Function::SelfBangingEast.cells()),
            bundle: crate::SourceBundle::Emit,
        }))
    };
    // The Bang Function writes no spelling and is displaced nowhere: its one
    // Portal is its own Span, which it clears.
    (Fire) => {
        FunctionKind::Effect(EffectKind::SourceWrite(crate::SourceEffect {
            columns: 0,
            rows: 0,
            spelling: None,
            bundle: crate::SourceBundle::Fire,
        }))
    };
    (Halt) => {
        FunctionKind::Effect(EffectKind::Lock)
    };
}

/// Where a root Function's activation comes from.
///
/// A root that answers an effect is inert until Bang activation, with two
/// exceptions that take their Turn with nothing delivered: a Self-Banging
/// Function, which receives Bang activation at its own turn without a
/// Source-resident `**`, and the Bang Function `**` itself. That asymmetry is a
/// property of the Function and of nothing else, so it is declared beside every
/// other property rather than derived from one of them: `^^` and `*^` write the
/// same spelling to the same geometry and differ here.
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
    /// and so do every Self-Banging Function and the Bang Function.
    Intrinsic,
    /// The root is inert until a Bang reaches it, which is the ordinary rule.
    Bang,
}

/// Which of a Function's Portals the `distance:` or `position:` rule its row
/// declares selects: the Input Portal a Read reads or the Output Portal a
/// Write writes.
#[derive(Clone, Copy)]
enum SelectedPortal {
    Input,
    Output,
}

/// Whether a Function's answer can be a Bang: one column of the Function
/// table, read by [`Function::can_emit_bang`].
#[derive(Clone, Copy)]
enum BangAnswer {
    /// It never answers Bang.
    Never,
    /// A Bang is one of the answers it can give.
    May,
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
            fn check(operands: &[Option<crate::Atom>]) -> Result<(), crate::Error> {
                let [$($role),+] = operands else {
                    return Err(crate::stack::arity(Self::FUNCTION, operands.len()));
                };
                let mut slots = 0..;
                $(crate::operand::check::<$operand>(*$role, crate::stack::next_slot(&mut slots))?;)+
                Ok(())
            }

            // Field initialisers evaluate in signature order, so the first
            // operand outside its domain is the one that diagnoses.
            #[inline(always)]
            fn bind(operands: &[Option<crate::Atom>]) -> Result<Self, crate::Error> {
                let [$($role),+] = operands else {
                    return Err(crate::stack::arity(Self::FUNCTION, operands.len()));
                };
                let mut slots = 0..;
                Ok(Self {
                    $($role: crate::operand::bind_atom::<$operand>(
                        *$role,
                        crate::stack::next_slot(&mut slots),
                    )?,)+
                })
            }
        }

        $(impl crate::portal::PortalOperands for $variant {
            const PORTAL: crate::PortalInput = portal_input!($portal_role, $portal_type);
        })?
    };
}

// Implements `Carried` for `$variant`'s operand struct when its row declares a
// `value: Untyped` role: the Language Unit a Write or Push carries to its pair.
macro_rules! carried {
    ($variant:ident, []) => {};
    ($variant:ident, [value: Untyped $(, $($rest:tt)*)?]) => {
        impl crate::functions::write::Carried for $variant {
            fn value(self) -> crate::Atom {
                self.value
            }
        }
    };
    ($variant:ident, [$role:ident: $operand:ident $(<$param:ident>)? $(, $($rest:tt)*)?]) => {
        carried!($variant, [$($($rest)*)?]);
    };
}

/// Whether two names spell the same bytes, where `==` on `str` is not const.
const fn same_name(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut at = 0;
    while at < left.len() {
        if left[at] != right[at] {
            return false;
        }
        at += 1;
    }
    true
}

// A row's Roles: one per declared operand, except that a `grouped:` Role stands
// for the consecutive operands it names. A group `grouped_roles` refuses fails
// to compile at the row that declares it.
macro_rules! function_roles {
    ([$($role:ident: $operand:ident),*], [$($group:ident: $($member:ident)+),*], [$($position:ident)*]) => {{
        const NAMES: &[&str] = &[$(stringify!($role)),*];
        const OPERANDS: &[&str] = &[$(stringify!($operand)),*];
        const GROUPS: &[RoleGroup] = &[$(RoleGroup {
            name: stringify!($group),
            members: &[$(stringify!($member)),+],
        }),*];
        const _: () = assert_position_grouped(&[$(stringify!($position)),*], GROUPS);
        const ROLES: [Role; role_count(NAMES.len(), GROUPS)] =
            grouped_roles(NAMES, OPERANDS, GROUPS);
        &ROLES
    }};
}

/// One `grouped:` Role of a row: its name and the operands it covers, as the
/// row names them.
#[derive(Clone, Copy)]
struct RoleGroup {
    name: &'static str,
    members: &'static [&'static str],
}

/// How many Roles a row with `operands` declared operands has once `groups`
/// stand for their members.
const fn role_count(operands: usize, groups: &[RoleGroup]) -> usize {
    let (mut count, mut index) = (operands, 0);
    while index < groups.len() {
        assert!(
            groups[index].members.len() >= 2,
            "a grouped Role covers at least two operands"
        );
        count -= groups[index].members.len() - 1;
        index += 1;
    }
    count
}

/// A row's Roles in signature order, evaluated where the row is declared.
/// `names` and `operands` are its declared roles and their operand types.
///
/// A group covers consecutive declared operands in signature order, each one a
/// Number, as a Position's column and row are.
const fn grouped_roles<const N: usize>(
    names: &[&'static str],
    operands: &[&str],
    groups: &[RoleGroup],
) -> [Role; N] {
    let mut roles = [Role {
        name: "",
        operands: 0,
    }; N];
    let (mut filled, mut operand) = (0, 0);
    while operand < names.len() {
        let role = match group_starting_at(groups, names[operand]) {
            Some(group) => {
                assert_group_covers(group, names, operands, operand);
                Role {
                    name: group.name,
                    operands: group.members.len(),
                }
            }
            None => Role {
                name: names[operand],
                operands: 1,
            },
        };
        // `N` counts every group as standing for its members, so a group no
        // declared operand starts leaves one Role too many.
        assert!(
            filled < N,
            "a grouped Role must start at a declared operand outside any other group"
        );
        roles[filled] = role;
        filled += 1;
        operand += role.operands;
    }
    roles
}

/// Refuses a row whose `position:` operands are not one `grouped:` Role, since
/// a reader sees a Position as one operand. `position` is empty on a row that
/// declares no `position:` clause.
const fn assert_position_grouped(position: &[&str], groups: &[RoleGroup]) {
    if position.is_empty() {
        return;
    }
    let mut index = 0;
    while index < groups.len() {
        let members = groups[index].members;
        if members.len() == position.len() {
            let mut member = 0;
            while member < members.len() && same_name(members[member], position[member]) {
                member += 1;
            }
            if member == members.len() {
                return;
            }
        }
        index += 1;
    }
    panic!("a `position:` clause's operands must be one grouped Role");
}

/// The group whose first member is the declared operand `name`.
const fn group_starting_at(groups: &[RoleGroup], name: &str) -> Option<RoleGroup> {
    let mut index = 0;
    while index < groups.len() {
        if same_name(groups[index].members[0], name) {
            return Some(groups[index]);
        }
        index += 1;
    }
    None
}

/// Refuses `group` unless its members are the declared operands from `start`
/// on, in order, and every one is a Number.
const fn assert_group_covers(group: RoleGroup, names: &[&str], operands: &[&str], start: usize) {
    let mut member = 0;
    while member < group.members.len() {
        let index = start + member;
        assert!(
            index < names.len() && same_name(names[index], group.members[member]),
            "a grouped Role covers consecutive declared operands in signature order"
        );
        assert!(
            same_name(operands[index], "Number"),
            "a grouped Role covers only Number operands"
        );
        member += 1;
    }
}

// Whether a row declares the optional fact its tokens stand for.
macro_rules! declares {
    () => {
        false
    };
    ($($fact:tt)+) => {
        true
    };
}

// Checks `operands` through `$variant`'s operand struct. A row that declares no
// operand admits only an empty list.
macro_rules! check_declared {
    ($variant:ident, [], $operands:expr) => {{
        let operands: &[Option<crate::Atom>] = $operands;
        if operands.is_empty() {
            Ok(())
        } else {
            Err(crate::stack::arity(Function::$variant, operands.len()))
        }
    }};
    ($variant:ident, [$($role:ident),+], $operands:expr) => {{
        use crate::stack::Operands as _;
        operands::$variant::check($operands)
    }};
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
        let values: Vec<Option<crate::Atom>> = $values.iter().copied().map(Some).collect();
        operands::$variant::checked_bind(&values).map(drop)
    }};
}

// Two tables: the Functions a spelling names wherever a Function may stand,
// and the Functions the Parser names itself. Both mint variants and every
// declared fact alike; only the first reaches `from_spelling`. The Bang
// Function is the second kind: `**` is also the Bang value an untyped operand
// carries, so an operand slot must not read it as a nested Function, and the
// Parser names it only where an Expression starts.
macro_rules! define_functions {
    (
        spelled: { $($s_variant:ident => ($s_spelling:literal, $($s_row:tt)*)),+ $(,)? }
        named: { $($n_variant:ident => ($n_spelling:literal, $($n_row:tt)*)),+ $(,)? }
    ) => {
        define_functions! {
            @spelled [$($s_spelling => $s_variant),+]
            $($s_variant => ($s_spelling, $($s_row)*)),+,
            $($n_variant => ($n_spelling, $($n_row)*)),+
        }
    };
    (@spelled [$($from:literal => $to:ident),+] $($variant:ident => ($spelling:literal, $name:literal, $summary:literal, $kind:ident, $activation:ident, $bang:ident, [$($role:ident: $operand:ident $(<$param:ident>)?),* $(,)?] $(, portal: $portal_role:literal : $portal_type:ident)? $(, distance: $d_portal:ident $d_direction:ident $d_distance:ident)? $(, position: $p_portal:ident $p_column:ident $p_row:ident)? $(, grouped: [$($g_role:ident: $($g_operand:ident)+),+])?)),+ $(,)?) => {
        $(const _: Spelling = Spelling::checked($spelling);)+

        // A `portal:` row's Input Portal is the static site south of it, which
        // a selection rule would contradict, and one row selects one pair.
        $(const _: () = assert!(
            !(declares!($($portal_role)?) && (declares!($($d_portal)?) || declares!($($p_portal)?))),
            "a row that declares `portal:` cannot also declare a selection rule",
        );
        const _: () = assert!(
            !(declares!($($d_portal)?) && declares!($($p_portal)?)),
            "a row declares at most one of `distance:` and `position:`",
        );)+

        // A `portal:` row reads back, from the static site south of it, the
        // previous value it wrote there, so neither of its Portals may be
        // selected by a rule; and one row selects one pair.
        $(const _: () = assert!(
            !(declares!($($portal_role)?) && (declares!($($d_portal)?) || declares!($($p_portal)?))),
            "a row that declares `portal:` cannot also declare a selection rule",
        );
        const _: () = assert!(
            !(declares!($($d_portal)?) && declares!($($p_portal)?)),
            "a row declares at most one of `distance:` and `position:`",
        );)+

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

            /// This Function's name, as its heading in `docs/functions.md`
            /// gives it: `.|` is `Absolute Difference`.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }

            /// One short line on what this Function answers or does, in the
            /// Function reference's voice: `.+` `answers left + right`.
            pub const fn summary(self) -> &'static str {
                match self {
                    $(Self::$variant => $summary,)+
                }
            }

            /// This Function's Roles, in signature order. Together they cover
            /// every operand its signature declares.
            pub const fn roles(self) -> &'static [Role] {
                match self {
                    $(Self::$variant => function_roles!(
                        [$($role: $operand),*],
                        [$($($g_role: $($g_operand)+),+)?],
                        [$($p_column $p_row)?]
                    ),)+
                }
            }

            /// The two Cells this Function's spelling occupies.
            pub const fn cells(self) -> Spelling {
                match self {
                    $(Self::$variant => const { Spelling::checked($spelling) },)+
                }
            }

            /// The Function `spelling` spells, if it spells one wherever a
            /// Function may stand. The Bang Function is not among them: the
            /// Parser names it where an Expression starts.
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
                    $($from => Some(Self::$to),)+
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

            /// Whether this Function can return Bang, even when the current
            /// operands produce no result. Scheduling uses this declaration to
            /// wait for activation producers before deciding whether to perform.
            pub const fn can_emit_bang(self) -> bool {
                match self {
                    $(Self::$variant => !matches!(BangAnswer::$bang, BangAnswer::Never),)+
                }
            }

            /// Whether this Function declares no operand at all.
            ///
            /// One name for a question both crates ask: `signature()` is
            /// `pub(crate)`, so `orcvs` cannot ask the slice. It is a fact
            /// about the declaration and not about a Function group — the
            /// Copies, Halt, the Bang Function and the Directional Bang
            /// Functions declare no operand either — so a caller that means "is a Self-Banging
            /// Function" should match [`Function::output`]'s
            /// [`crate::Output::Source`] instead, and read the bundle it carries: both groups declare an effect, and
            /// it is the `Advance` a Self-Banging Function declares that tells
            /// it from the `Emit` of a Directional Bang one.
            #[inline(always)]
            pub const fn takes_no_operand(self) -> bool {
                self.signature().is_empty()
            }

            /// Whether this Function sends a Payload: the Cells after its
            /// spelling through the next `!` on its row.
            pub const fn takes_payload(self) -> bool {
                matches!(self, Self::Udp | Self::Osc)
            }

            /// The Portal input resolved at Turn, after cell operand validation.
            pub const fn portal_input(self) -> Option<crate::PortalInput> {
                match self {
                    $($(Self::$variant => Some(portal_input!($portal_role, $portal_type)),)?)+
                    _ => None,
                }
            }

            /// The rule by which this Function's operands select its pair, and
            /// the Portal it selects, when its row declares one.
            const fn declared_selection(self) -> Option<(SelectedPortal, crate::PairSelection)> {
                match self {
                    $($(Self::$variant => Some((
                        SelectedPortal::$d_portal,
                        crate::PairSelection::Distance(crate::DistanceRule::$variant),
                    )),)?)+
                    $($(Self::$variant => Some((
                        SelectedPortal::$p_portal,
                        crate::PairSelection::Position(crate::PositionRule::$variant),
                    )),)?)+
                    _ => None,
                }
            }

            /// The token each declared operand is read as, in signature order.
            pub(crate) const fn signature(self) -> &'static [crate::Token] {
                match self {
                    $(Self::$variant => const {
                        &[$(<<crate::operand::$operand $(<crate::operand::$param>)? as crate::operand::Operand>::Token as crate::operand::TokenKind>::TOKEN,)*]
                    },)+
                }
            }

            /// Whether each declared operand admits a slot that holds no
            /// value, in signature order: what its operand type answers.
            pub(crate) const fn blank_admitted(self) -> &'static [bool] {
                match self {
                    $(Self::$variant => const {
                        &[$(crate::operand::admits_blank::<crate::operand::$operand $(<crate::operand::$param>)?>(),)*]
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
                [$($role: crate::operand::$operand $(<crate::operand::$param>)?),*]
                $(, $portal_role: $portal_type)?
            })+

            $(carried!($variant, [$($role: $operand $(<$param>)?),*]);)+
        }

        /// How a directional Read's or Write's operands count the distance to
        /// its pair: one rule per row that declares `distance:`, binding that
        /// row's own operands.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum DistanceRule {
            $($(
                #[doc = concat!(
                    "`", $spelling, "` selects its ", stringify!($d_portal), " Portal's pair `",
                    stringify!($d_distance), "` Portals ", stringify!($d_direction), " from its `",
                    stringify!($d_distance), "` operand's slot."
                )]
                $variant,
            )?)+
        }

        impl DistanceRule {
            /// The direction this rule counts in.
            pub const fn direction(self) -> crate::Direction {
                match self {
                    $($(Self::$variant => crate::Direction::$d_direction,)?)+
                }
            }

            /// The distance this rule counts, from the operands its Function's
            /// Turn resolved, checked against that Function's own signature so
            /// they diagnose as its evaluation would: a Write's `value` follows
            /// its `distance` and is never read.
            pub(crate) fn distance(self, values: &[Option<Atom>]) -> Result<u8, Error> {
                match self {
                    $($(Self::$variant => {
                        use crate::stack::Operands as _;
                        operands::$variant::checked_bind(values)
                            .map(|operands::$variant { $d_distance, .. }| $d_distance)
                    })?)+
                }
            }
        }

        /// How an absolute Read's or Write's operands name the Position of its
        /// pair: one rule per row that declares `position:`, binding that row's
        /// own operands.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum PositionRule {
            $($(
                #[doc = concat!(
                    "`", $spelling, "` names its ", stringify!($p_portal), " Portal's Position by its `",
                    stringify!($p_column), "` and `", stringify!($p_row), "` operands."
                )]
                $variant,
            )?)+
        }

        impl PositionRule {
            /// The Position, as `(column, row)`, from the operands its
            /// Function's Turn resolved, checked against that Function's own
            /// signature so they diagnose as its evaluation would: a Write's
            /// `value` follows the Position and is never read.
            pub(crate) fn position(self, values: &[Option<Atom>]) -> Result<(u8, u8), Error> {
                match self {
                    $($(Self::$variant => {
                        use crate::stack::Operands as _;
                        operands::$variant::checked_bind(values)
                            .map(|operands::$variant { $p_column, $p_row, .. }| ($p_column, $p_row))
                    })?)+
                }
            }
        }

        impl Function {
            /// Checks `operands`, given in signature order, as this
            /// Function's evaluation checks them before it reads any domain
            /// or Portal: their count, and each slot against its declared
            /// token. A slot that holds no value is refused here by an operand
            /// type that requires one, with
            /// [`ArgumentError::Absent`](crate::ArgumentError::Absent).
            ///
            /// ```
            /// use lang::{ArgumentError, Atom, Error, Function};
            /// assert!(Function::Add.check_operands(&[Some(Atom::Number(1)); 2]).is_ok());
            /// assert!(matches!(
            ///     Function::Add.check_operands(&[Some(Atom::Number(1)), None]),
            ///     Err(Error::Argument(ArgumentError::Absent { slot: 1, .. }))
            /// ));
            /// ```
            pub fn check_operands(self, operands: &[Option<Atom>]) -> Result<(), Error> {
                match self {
                    $(Self::$variant => check_declared!($variant, [$($role),*], operands),)+
                }
            }
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

/// What one or more consecutive operands of a Function are to a reader: `.+`
/// has the Roles `left` and `right`, and `=$` the two Position Roles `from`
/// and `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Role {
    /// The Role's name in the Function reference.
    pub name: &'static str,
    /// How many of the Function's operands, in signature order, this Role
    /// covers. A Position is spelled as two Numbers, column then row, and is
    /// one Role.
    pub operands: usize,
}

define_functions! {
    spelled: {
    AbsoluteCopy => ("=$", "Absolute Copy", "copies the pair at from onto the pair at to", Value, Intrinsic, May, [from_column: Number, from_row: Number, to_column: Number, to_row: Number], grouped: [from: from_column from_row, to: to_column to_row]),
    AbsoluteDifference => (".|", "Absolute Difference", "answers the distance between left and right", Value, Intrinsic, Never, [left: Number, right: Number]),
    AbsoluteRead => ("&$", "Absolute Read", "reads the pair at a Position", Value, Intrinsic, May, [column: Number, row: Number], position: Input column row, grouped: [at: column row]),
    AbsoluteWrite => ("@$", "Absolute Write", "writes value at a Position", Value, Intrinsic, May, [column: Number, row: Number, value: Untyped], position: Output column row, grouped: [at: column row]),
    Add => (".+", "Add", "answers left + right", Value, Intrinsic, Never, [left: Number, right: Number]),
    Clock => ("~.", "Clock", "answers which of its steps the Tick is on", Value, Intrinsic, Never, [length: Number, steps: Number]),
    ControlChange => ("!c", "Control Change", "sends value to controller on channel", TerminalOutput, Bang, Never, [channel: MidiChannel, controller: Controller, value: ControlValue]),
    ConvertToNote => (".^", "Convert to Note", "answers the Note for a Number", Value, Intrinsic, Never, [number: Number]),
    ConvertToNumber => (".v", "Convert to Number", "answers the MIDI note number of a Note", Value, Intrinsic, Never, [note: Note]),
    CopyEast => ("=>", "Copy East", "copies the pair before it to the pair after it", Value, Intrinsic, May, []),
    CopyNorth => ("=^", "Copy North", "copies the pair south of it to the pair north of it", Value, Intrinsic, May, []),
    CopySouth => ("=v", "Copy South", "copies the pair north of it to the pair south of it", Value, Intrinsic, May, []),
    CopyWest => ("=<", "Copy West", "copies the pair after it to the pair before it", Value, Intrinsic, May, []),
    Delay => ("~*", "Delay", "answers a Bang every length * steps Ticks, else a Clear", Value, Intrinsic, May, [length: Number, steps: Number]),
    DirectionalBangEast => ("*>", "Bang East", "emits >> after it when a Bang reaches it", BangEast, Bang, Never, []),
    DirectionalBangNorth => ("*^", "Bang North", "emits ^^ one row north when a Bang reaches it", BangNorth, Bang, Never, []),
    DirectionalBangSouth => ("*v", "Bang South", "emits vv one row south when a Bang reaches it", BangSouth, Bang, Never, []),
    DirectionalBangWest => ("*<", "Bang West", "emits << before it when a Bang reaches it", BangWest, Bang, Never, []),
    Divide => ("./", "Divide", "answers left / right, rounded down", Value, Intrinsic, Never, [left: Number, right: Number]),
    Down => ("#-", "Down", "answers the Note number semitones below note", Value, Intrinsic, Never, [note: Note, number: Number]),
    Equality => (".=", "Equality", "answers a Bang when left equals right, else a Clear", Value, Intrinsic, May, [left: Number, right: Number]),
    Euclidean => ("~%", "Euclidean", "answers a Bang on hits of every steps Ticks, else a Clear", Value, Intrinsic, May, [hits: Number, steps: Number]),
    Halt => ("*!", "Halt", "locks the root south of it when a Bang reaches it", Halt, Bang, Never, []),
    Increment => ("~+", "Increment", "answers previous value + amount, wrapped at modulus", Value, Intrinsic, Never, [amount: Number, modulus: Number], portal: "previous value": Number),
    Interpolation => ("~>", "Interpolation", "answers previous value moved toward target by at most amount", Value, Intrinsic, Never, [amount: Number, target: Number], portal: "previous value": Number),
    Maximum => (".>", "Maximum", "answers the larger of left and right", Value, Intrinsic, Never, [left: Number, right: Number]),
    Minimum => (".<", "Minimum", "answers the smaller of left and right", Value, Intrinsic, Never, [left: Number, right: Number]),
    Modulo => (".%", "Modulo", "answers the remainder of left / right", Value, Intrinsic, Never, [left: Number, right: Number]),
    MonophonicPlay => ("!%", "Monophonic Play", "replaces its channel's Monophonic note for length Ticks", TerminalOutput, Bang, Never, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
    Multiply => (".x", "Multiply", "answers left * right", Value, Intrinsic, Never, [left: Number, right: Number]),
    Osc => ("!o", "OSC", "sends its Payload as one OSC message", TerminalOutput, Bang, Never, []),
    Pick => ("#p", "Pick", "answers the Note at position index of set above root", Value, Intrinsic, Never, [root: Note, set: IntervalSet, index: Number]),
    PitchBend => ("!b", "Pitch Bend", "sends lsb and msb as a Pitch Bend on channel", TerminalOutput, Bang, Never, [channel: MidiChannel, lsb: BendLsb, msb: BendMsb]),
    Push => ("@t", "Push", "writes value at pair index % count of the row below", Value, Intrinsic, May, [index: Number, count: Number, value: Untyped]),
    Random => ("~?", "Random", "answers a Number from minimum to maximum", Value, Intrinsic, Never, [seed: Number, minimum: Number, maximum: Number]),
    RawChordPlay => ("!:", "Raw Chord Play", "sends a Note On for each tone of a chord", TerminalOutput, Bang, Never, [channel: MidiChannel, velocity: Velocity, root: Note, set: IntervalSet, voicing: Voicing, inversion: Number]),
    RawPlay => ("!>", "Raw Play", "sends a Note On, or a Note Off at velocity 00", TerminalOutput, Bang, Never, [channel: MidiChannel, velocity: Velocity, note: Optional<Note>]),
    ReadEast => ("&>", "Read East", "reads the pair distance pairs east of its distance operand", Value, Intrinsic, May, [distance: Number], distance: Input East distance),
    ReadNorth => ("&^", "Read North", "reads the pair distance rows north of its distance operand", Value, Intrinsic, May, [distance: Number], distance: Input North distance),
    ReadSouth => ("&v", "Read South", "reads the pair distance rows south of its distance operand", Value, Intrinsic, May, [distance: Number], distance: Input South distance),
    ReadWest => ("&<", "Read West", "reads the pair distance pairs west of its distance operand", Value, Intrinsic, May, [distance: Number], distance: Input West distance),
    SelfBangingEast => (">>", "Move East", "moves itself one Cell east each Tick", SelfBangEast, Intrinsic, Never, []),
    SelfBangingNorth => ("^^", "Move North", "moves itself one row north each Tick", SelfBangNorth, Intrinsic, Never, []),
    SelfBangingSouth => ("vv", "Move South", "moves itself one row south each Tick", SelfBangSouth, Intrinsic, Never, []),
    SelfBangingWest => ("<<", "Move West", "moves itself one Cell west each Tick", SelfBangWest, Intrinsic, Never, []),
    Stack => ("#s", "Stack", "answers the chord of size tones built on degree of set", Value, Intrinsic, Never, [set: IntervalSet, degree: Number, size: Number]),
    Subtract => (".-", "Subtract", "answers left - right", Value, Intrinsic, Never, [left: Number, right: Number]),
    TimedChordPlay => ("!#", "Timed Chord Play", "sends a chord, each tone stopped after length Ticks", TerminalOutput, Bang, Never, [channel: MidiChannel, velocity: Velocity, root: Note, set: IntervalSet, voicing: Voicing, inversion: Number, length: Length]),
    TimedPlay => ("!~", "Timed Play", "sends a Note On and its Note Off after length Ticks", TerminalOutput, Bang, Never, [channel: MidiChannel, velocity: Velocity, note: Note, length: Length]),
    Track => ("&t", "Track", "reads pair index % count after its operands", Value, Intrinsic, May, [index: Number, count: Number]),
    Udp => ("!u", "UDP", "sends its Payload as one UDP datagram", TerminalOutput, Bang, Never, []),
    Up => ("#+", "Up", "answers the Note number semitones above note", Value, Intrinsic, Never, [note: Note, number: Number]),
    WriteEast => ("@>", "Write East", "writes value distance pairs east of its distance operand", Value, Intrinsic, May, [distance: Number, value: Untyped], distance: Output East distance),
    WriteNorth => ("@^", "Write North", "writes value distance rows north of its distance operand", Value, Intrinsic, May, [distance: Number, value: Untyped], distance: Output North distance),
    WriteSouth => ("@v", "Write South", "writes value distance rows south of its distance operand", Value, Intrinsic, May, [distance: Number, value: Untyped], distance: Output South distance),
    WriteWest => ("@<", "Write West", "writes value distance pairs west of its distance operand", Value, Intrinsic, May, [distance: Number, value: Untyped], distance: Output West distance),
    }
    named: {
    Bang => ("**", "Bang", "activates the roots aligned with it", Fire, Intrinsic, May, []),
    }
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
            replacement.output() != running.output()
                || replacement.input_portal() != running.input_portal()
        }),
    ];

    /// What this Function does through its Output Portal.
    ///
    /// The kind column decides it for every effect: Terminal Output writes no
    /// Cell, a Source write writes its declared effect, and a lock locks
    /// through the Portal one row south. A value answer is written at the pair
    /// its operands select when the Function declares a selection rule, and
    /// otherwise at its static offset: one row south unless it is a Copy,
    /// which writes through the Portal its direction names.
    pub const fn output(self) -> crate::Output {
        use crate::Output;
        match self.kind() {
            FunctionKind::Effect(EffectKind::TerminalOutput) => Output::Terminal,
            FunctionKind::Effect(EffectKind::SourceWrite(effect)) => Output::Source(effect),
            FunctionKind::Effect(EffectKind::Lock) => Output::Lock(self.static_offset()),
            FunctionKind::Value => match self.selection() {
                Some(selection) => Output::Dynamic(selection),
                None => Output::Static(self.static_offset()),
            },
        }
    }

    /// The offset a static Output Portal stands at.
    const fn static_offset(self) -> crate::PortalCoords {
        match self {
            Self::CopyEast => crate::PortalCoords {
                columns: 2,
                rows: 0,
            },
            Self::CopyWest => crate::PortalCoords {
                columns: -2,
                rows: 0,
            },
            Self::CopyNorth => crate::PortalCoords {
                columns: 0,
                rows: -1,
            },
            _ => crate::PortalCoords::SOUTH,
        }
    }

    /// How this Function's operands select the pair its dynamic Output Portal
    /// writes, or `None` when its Output Portal, if any, is static.
    ///
    /// Each Write's row declares the rule its address operands select its
    /// pair by, the rule the Read with the same arrow, or the absolute Read,
    /// declares for the pair it reads: the address operands lead the
    /// signature and `value` follows them. Push selects a pair of
    /// the lane below it, and the absolute Copy the destination Position its
    /// last two operands name.
    const fn selection(self) -> Option<crate::PairSelection> {
        use crate::PairSelection::{DestinationPosition, Lane};
        match self {
            Self::AbsoluteCopy => Some(DestinationPosition),
            Self::Push => Some(Lane),
            _ => match self.declared_selection() {
                Some((SelectedPortal::Output, selection)) => Some(selection),
                _ => None,
            },
        }
    }

    /// The Input Portal this Function names, or `None` when it names none.
    ///
    /// Copy names the Portal opposite its Output Portal. Increment and
    /// Interpolation name one row south, the same site as their Output Portal.
    /// Those are static. Track's, every Read's and the absolute Copy's are
    /// dynamic, and each names the rule by which its operands select the pair
    /// it reads; a Read's row declares its rule.
    pub const fn input_portal(self) -> Option<crate::InputPortal> {
        use crate::InputPortal::{Dynamic, Static};
        use crate::PairSelection::{IndexModuloCount, SourcePosition};
        match self {
            Self::AbsoluteCopy => Some(Dynamic(SourcePosition)),
            Self::CopyEast => Some(Static(crate::PortalCoords {
                columns: -2,
                rows: 0,
            })),
            Self::CopyWest => Some(Static(crate::PortalCoords {
                columns: 2,
                rows: 0,
            })),
            Self::CopyNorth => Some(Static(crate::PortalCoords::SOUTH)),
            Self::CopySouth => Some(Static(crate::PortalCoords {
                columns: 0,
                rows: -1,
            })),
            Self::Track => Some(Dynamic(IndexModuloCount)),
            _ if self.portal_input().is_some() => Some(Static(crate::PortalCoords::SOUTH)),
            _ => match self.declared_selection() {
                Some((SelectedPortal::Input, selection)) => Some(Dynamic(selection)),
                _ => None,
            },
        }
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

/// The Interval Set Atom two Cells spell, borrowing them and building nothing
/// on refusal.
pub(crate) fn interval_set_atom_from_spelling(s: &str) -> Option<Atom> {
    IntervalSet::from_spelling(s).map(Atom::IntervalSet)
}

/// The Voicing Atom two Cells spell, borrowing them and building nothing on
/// refusal.
pub(crate) fn voicing_atom_from_spelling(s: &str) -> Option<Atom> {
    Voicing::from_spelling(s).map(Atom::Voicing)
}

/// The Atom an untyped slot holds: the Language Unit two Cells spell, a Bang
/// included.
///
/// The slot carries those Cells on and never interprets them. Every reading
/// that can apply renders back as the same two Cells, so where two readings
/// coincide, as `C4` spells both a Number and a Note, either carries the
/// same encoding. Cells that spell no Language Unit, an empty pair among
/// them, are not a value to carry.
pub(crate) fn untyped_atom(s: &str) -> Result<Atom, Error> {
    if s == "**" {
        return Ok(Atom::Bang);
    }
    crate::functions::copy::copied_atom(s).ok_or_else(|| TypeError::Unit(s.to_string()).into())
}

#[inline(always)]
pub fn to_atom_note(s: &str) -> Result<Atom, Error> {
    note_atom_from_spelling(s).ok_or_else(|| TypeError::Note(s.to_string()).into())
}

#[inline(always)]
pub fn to_atom_num(s: &str) -> Result<Atom, Error> {
    number_atom_from_spelling(s).ok_or_else(|| TypeError::Number(s.to_string()).into())
}

#[inline(always)]
pub(crate) fn to_atom_interval_set(s: &str) -> Result<Atom, Error> {
    interval_set_atom_from_spelling(s).ok_or_else(|| TypeError::IntervalSet(s.to_string()).into())
}

#[inline(always)]
pub(crate) fn to_atom_voicing(s: &str) -> Result<Atom, Error> {
    voicing_atom_from_spelling(s).ok_or_else(|| TypeError::Voicing(s.to_string()).into())
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
        self.spelling().fmt(f)
    }
}

impl Atom {
    /// The two Source Cells this Atom spells, which a Portal writes and a
    /// later Turn decodes.
    pub fn spelling(self) -> Spelling {
        match self {
            Atom::Bang => const { Spelling::checked("**") },
            // Numbers are hexadecimal: rendered results are written back into
            // the Source and re-parsed as two Cells, so they round trip as hex.
            Atom::Number(n) => Spelling::number(n),
            Atom::Note(note) => Spelling::note(note),
            Atom::Function(function) => function.cells(),
            Atom::IntervalSet(set) => set.cells(),
            Atom::Voicing(voicing) => voicing.cells(),
        }
    }
}

pub use spelling::Spelling;

/// Holds the only constructors of a [`Spelling`]. Its field is private to this
/// module, so nothing else in `lang` can build one from unchecked bytes.
mod spelling {
    use std::fmt;

    use super::Note;
    use crate::{NOTE_PITCHES, octave_cell};

    ///
    /// The two Source Cells an Atom spells.
    ///
    /// Both are printable ASCII, space through `~`, which is what a Cell
    /// holds. Every constructor builds from a literal checked at compile time
    /// or from a table whose every entry is, so a `Spelling` that exists needs
    /// no second check.
    ///
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Spelling([u8; 2]);

    /// The hexadecimal digits a Number spells, most significant first.
    const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

    /// The highest octave a Note reaches: `0x7F / 12`.
    const TOP_OCTAVE: u8 = 0x7F / 12;

    // Every table a `Spelling` is built from holds only printable Cells, so the
    // constructors below index them without checking what they find.
    const _: () = {
        assert!(all_printable(HEX_DIGITS));
        assert!(all_printable(NOTE_PITCHES));
        let mut octave = 0;
        while octave <= TOP_OCTAVE {
            assert!(Spelling::admits(octave_cell(octave)));
            octave += 1;
        }
    };

    impl Spelling {
        /// A spelling stated as a literal, refused at compile time unless it
        /// is two printable ASCII Cells. Every caller evaluates it in a const
        /// context.
        pub(super) const fn checked(spelling: &str) -> Self {
            let bytes = spelling.as_bytes();
            assert!(
                bytes.len() == 2 && Self::admits(bytes[0]) && Self::admits(bytes[1]),
                "a spelling must be exactly two printable ASCII Cells",
            );
            Self([bytes[0], bytes[1]])
        }

        /// A Number's two hexadecimal digits.
        pub(super) fn number(n: u8) -> Self {
            Self([
                HEX_DIGITS[usize::from(n >> 4)],
                HEX_DIGITS[usize::from(n & 0x0F)],
            ])
        }

        /// A Note's pitch letter and octave. A Note is at most `0x7F`, so its
        /// octave is at most `TOP_OCTAVE`.
        pub(super) fn note(note: Note) -> Self {
            let value = note.value();
            Self([
                NOTE_PITCHES[usize::from(value % 12)],
                octave_cell(value / 12),
            ])
        }

        /// Whether `byte` is one a Cell holds: printable ASCII, space through
        /// `~`.
        pub const fn admits(byte: u8) -> bool {
            matches!(byte, 0x20..=0x7E)
        }

        /// The two Cells, west first.
        pub const fn cells(self) -> [u8; 2] {
            self.0
        }
    }

    const fn all_printable(table: &[u8]) -> bool {
        let mut index = 0;
        while index < table.len() {
            if !Spelling::admits(table[index]) {
                return false;
            }
            index += 1;
        }
        true
    }

    impl fmt::Display for Spelling {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let [west, east] = self.0;
            write!(f, "{}{}", char::from(west), char::from(east))
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        Atom, BendLsb, BendMsb, ControlValue, Controller, Function, Length, MidiChannel, Note,
        ReplacementChange, Role, RoleGroup, Velocity, assert_position_grouped, grouped_roles,
        role_count, to_atom_num,
    };
    use crate::{Error, IntervalSet, Token, TypeError, Voicing};

    /// The normative names and semitones, stated from the specification rather
    /// than read from the table under test.
    const INTERVAL_SETS: [(&str, &[u8]); 39] = [
        ("M_", &[0, 4, 7]),
        ("M7", &[0, 4, 7, 11]),
        ("M9", &[0, 4, 7, 11, 14]),
        ("MD", &[0, 4, 7, 11, 14, 21]),
        ("M6", &[0, 4, 7, 9]),
        ("69", &[0, 4, 7, 9, 14]),
        ("+M", &[0, 4, 8, 11]),
        ("m_", &[0, 3, 7]),
        ("m7", &[0, 3, 7, 10]),
        ("mM", &[0, 3, 7, 11]),
        ("m6", &[0, 3, 7, 9]),
        ("m9", &[0, 3, 7, 10, 14]),
        ("mB", &[0, 3, 7, 10, 14, 17]),
        ("mD", &[0, 3, 7, 10, 14, 21]),
        ("o_", &[0, 3, 6]),
        ("o7", &[0, 3, 6, 9]),
        ("h7", &[0, 3, 6, 10]),
        ("b9", &[0, 4, 7, 10, 13]),
        ("7_", &[0, 4, 7, 10]),
        ("9_", &[0, 4, 7, 10, 14]),
        ("13", &[0, 4, 7, 10, 14, 21]),
        ("s2", &[0, 2, 7]),
        ("s4", &[0, 5, 7]),
        ("+_", &[0, 4, 8]),
        ("a9", &[0, 4, 7, 14]),
        ("MA", &[0, 2, 4, 5, 7, 9, 11]),
        ("mi", &[0, 2, 3, 5, 7, 8, 10]),
        ("do", &[0, 2, 3, 5, 7, 9, 10]),
        ("ph", &[0, 1, 3, 5, 7, 8, 10]),
        ("ly", &[0, 2, 4, 6, 7, 9, 11]),
        ("mx", &[0, 2, 4, 5, 7, 9, 10]),
        ("lo", &[0, 1, 3, 5, 6, 8, 10]),
        ("hm", &[0, 2, 3, 5, 7, 8, 11]),
        ("mm", &[0, 2, 3, 5, 7, 9, 11]),
        ("p5", &[0, 2, 4, 7, 9]),
        ("p3", &[0, 3, 5, 7, 10]),
        ("bl", &[0, 3, 5, 6, 7, 10]),
        ("wt", &[0, 2, 4, 6, 8, 10]),
        ("ch", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
    ];

    /// The eight Voicing names, stated from the specification.
    const VOICINGS: [&str; 8] = ["CL", "OP", "D2", "D3", "D5", "D6", "SH", "RL"];

    #[test]
    fn every_interval_set_name_decodes_to_its_semitones() {
        for (name, semitones) in INTERVAL_SETS {
            let atom = Token::IntervalSet.decode(name).expect(name);
            let Atom::IntervalSet(set) = atom else {
                panic!("{name} decoded to {atom:?}");
            };
            assert_eq!(set.semitones(), semitones, "{name}");
            assert_eq!(atom.to_string(), name, "{name} spells itself back");
        }
        assert_eq!(IntervalSet::ALL.len(), INTERVAL_SETS.len());
    }

    #[test]
    fn every_voicing_name_decodes_and_spells_itself_back() {
        for name in VOICINGS {
            let atom = Token::Voicing.decode(name).expect(name);
            assert!(
                matches!(atom, Atom::Voicing(_)),
                "{name} decoded to {atom:?}"
            );
            assert_eq!(atom.to_string(), name, "{name} spells itself back");
        }
        assert_eq!(Voicing::ALL.len(), VOICINGS.len());
    }

    /// Whether `token` refuses `cells` with its own type error, carrying the
    /// Cells it was given.
    fn refused_as_its_type(token: Token, cells: &str) -> bool {
        match token.decode(cells) {
            Err(Error::Type(TypeError::IntervalSet(found))) => {
                token == Token::IntervalSet && found == cells
            }
            Err(Error::Type(TypeError::Voicing(found))) => {
                token == Token::Voicing && found == cells
            }
            _ => false,
        }
    }

    #[test]
    fn an_unknown_or_half_blank_name_is_refused_by_both_literal_types() {
        for token in [Token::IntervalSet, Token::Voicing] {
            for refused in [
                "  ", "M ", " M", "C ", " L", "zz", "CM", "", "M", "M_ ", "**",
            ] {
                assert!(
                    refused_as_its_type(token, refused),
                    "{token:?} admitted {refused:?}"
                );
            }
        }
        for near_miss in ["m8", "MX", "M7 "] {
            assert!(
                refused_as_its_type(Token::IntervalSet, near_miss),
                "{near_miss:?}"
            );
        }
        for near_miss in ["cl", "D4", "D1", "CLL"] {
            assert!(
                refused_as_its_type(Token::Voicing, near_miss),
                "{near_miss:?}"
            );
        }
    }

    /// A valid Number or Note spelling that names no member is refused, so
    /// neither type falls back to the Number or Note reading of its Cells.
    #[test]
    fn a_number_or_note_spelling_that_is_no_name_is_refused_by_both_literal_types() {
        for token in [Token::IntervalSet, Token::Voicing] {
            for spelling in ["C4", "7F", "00"] {
                assert!(
                    Token::Number.decode(spelling).is_ok() || Token::Note.decode(spelling).is_ok(),
                    "{spelling:?} spells a Number or a Note"
                );
                assert!(
                    refused_as_its_type(token, spelling),
                    "{token:?} admitted {spelling:?}"
                );
            }
        }
    }

    #[test]
    fn an_interval_set_and_a_voicing_refuse_each_others_names() {
        for voicing in Voicing::ALL {
            let name = voicing.cells().to_string();
            assert!(refused_as_its_type(Token::IntervalSet, &name), "{name}");
        }
        for set in IntervalSet::ALL {
            let name = set.cells().to_string();
            assert!(refused_as_its_type(Token::Voicing, &name), "{name}");
        }
    }

    #[test]
    fn a_name_that_also_spells_a_number_or_note_is_read_by_the_position_receiving_it() {
        let set = |name| Token::IntervalSet.decode(name).expect(name);
        assert_eq!(Token::Number.decode("69").unwrap(), Atom::Number(0x69));
        assert!(matches!(set("69"), Atom::IntervalSet(_)));
        assert_eq!(Token::Number.decode("13").unwrap(), Atom::Number(0x13));
        assert!(matches!(set("13"), Atom::IntervalSet(_)));

        let d2 = Note::try_from(0x26).unwrap();
        assert_eq!(Token::Note.decode("D2").unwrap(), Atom::Note(d2));
        assert_eq!(Token::Number.decode("D2").unwrap(), Atom::Number(0xD2));
        assert!(matches!(
            Token::Voicing.decode("D2").unwrap(),
            Atom::Voicing(_)
        ));
    }

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
            // An untyped slot carries any Language Unit, so the lowest Number
            // is one of the values it reads.
            crate::Token::Untyped => Atom::Number(0),
            crate::Token::IntervalSet => Atom::IntervalSet(IntervalSet::ALL[0]),
            crate::Token::Voicing => Atom::Voicing(Voicing::ALL[0]),
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
            let crate::Output::Source(effect) = function.output() else {
                continue;
            };
            seen += 1;
            // The last two columns are the bundle's two facts: whether it
            // clears the producer's own Span, and whether it writes a spelling
            // at that Span displaced. Every bundle rule the Portal, the schedule
            // and execution apply is one of them, its negation, or both.
            let (columns, rows, spelling, bundle, clears, displaced) = match function.spelling() {
                "^^" => (0, -1, Some("^^"), crate::SourceBundle::Advance, true, true),
                "vv" => (0, 1, Some("vv"), crate::SourceBundle::Advance, true, true),
                "<<" => (-1, 0, Some("<<"), crate::SourceBundle::Advance, true, true),
                ">>" => (1, 0, Some(">>"), crate::SourceBundle::Advance, true, true),
                "*^" => (0, -1, Some("^^"), crate::SourceBundle::Emit, false, true),
                "*v" => (0, 1, Some("vv"), crate::SourceBundle::Emit, false, true),
                "*<" => (-2, 0, Some("<<"), crate::SourceBundle::Emit, false, true),
                "*>" => (2, 0, Some(">>"), crate::SourceBundle::Emit, false, true),
                "**" => (0, 0, None, crate::SourceBundle::Fire, true, false),
                other => panic!("{other} declares a Source write with no stated effect"),
            };
            assert_eq!(
                effect,
                crate::SourceEffect {
                    columns,
                    rows,
                    spelling: spelling.map(super::Spelling::checked),
                    bundle,
                },
                "{function:?} writes something its spelling does not name",
            );
            assert_eq!(
                (
                    effect.bundle.clears_own_span(),
                    effect.bundle.writes_displaced()
                ),
                (clears, displaced),
                "{function:?}'s bundle states the wrong facts about its own Span",
            );
            assert!(
                function.takes_no_operand(),
                "{function:?} declares an operand a Source-writing Function does not take",
            );
        }
        assert_eq!(
            seen, 9,
            "the Source-writing rows are no longer the nine expected"
        );
    }

    #[test]
    fn exactly_the_bang_capable_functions_declare_that_they_can_emit_bang() {
        // Equality, Delay and Euclidean answer a Bang or a Clear as their
        // result, a Copy, Track or Read copies a Bang from its Input
        // Portal, a Write carries a Bang in its `value`, and the Bang
        // Function's Turn delivers the Bang it stands for. Tick scheduling
        // trusts the declaration to decide which roots
        // can supply activation, so the list is stated whole — a Function that
        // began returning Bang without declaring it would build no activation
        // edge, and the neighbouring terminal root would fall silent with no
        // diagnostic anywhere.
        // `only_a_function_that_declares_it_ever_answers_with_bang` is the
        // other half for operand-reading Functions; the Copies, Track and Read
        // are exercised on their own path, because each reads its Portal.
        assert_eq!(
            Function::ALL
                .iter()
                .copied()
                .filter(|function| function.can_emit_bang())
                .collect::<Vec<_>>(),
            vec![
                Function::AbsoluteCopy,
                Function::AbsoluteRead,
                Function::AbsoluteWrite,
                Function::CopyEast,
                Function::CopyNorth,
                Function::CopySouth,
                Function::CopyWest,
                Function::Delay,
                Function::Equality,
                Function::Euclidean,
                Function::Push,
                Function::ReadEast,
                Function::ReadNorth,
                Function::ReadSouth,
                Function::ReadWest,
                Function::Track,
                Function::WriteEast,
                Function::WriteNorth,
                Function::WriteSouth,
                Function::WriteWest,
                Function::Bang,
            ]
        );
    }

    #[test]
    fn every_declared_selection_is_the_one_its_portal_selects_by() {
        // A row's hard-coded Portal arm would shadow a rule it also declares,
        // so every declared rule must be the one its Portal reports.
        use super::SelectedPortal;
        use crate::{InputPortal, Output};
        for function in Function::ALL.iter().copied() {
            match function.declared_selection() {
                Some((SelectedPortal::Input, selection)) => assert_eq!(
                    function.input_portal(),
                    Some(InputPortal::Dynamic(selection)),
                    "{function:?}"
                ),
                Some((SelectedPortal::Output, selection)) => {
                    assert_eq!(
                        function.output(),
                        Output::Dynamic(selection),
                        "{function:?}"
                    );
                }
                None => {}
            }
        }
    }

    #[test]
    fn every_declared_distance_or_position_binds_its_own_functions_operands() {
        use crate::{ArgumentError, Direction, Error, InputPortal, Output, PairSelection};
        let mut declared = 0;
        for function in Function::ALL.iter().copied() {
            let input = match function.input_portal() {
                Some(InputPortal::Dynamic(selection)) => Some(('&', selection)),
                _ => None,
            };
            let output = match function.output() {
                Output::Dynamic(selection) => Some(('@', selection)),
                _ => None,
            };
            for (family, selection) in input.into_iter().chain(output) {
                // A Read `&` selects the pair its Input Portal reads and a
                // Write `@` the pair its Output Portal writes, so a rule
                // declared on the wrong Portal shows here; a distance counts
                // in the direction the spelling's arrow names.
                let mut spelling = function.spelling().chars();
                let named = matches!(
                    selection,
                    PairSelection::Distance(_) | PairSelection::Position(_)
                );
                if named {
                    assert_eq!(spelling.next(), Some(family), "{function:?}");
                }
                if let PairSelection::Distance(rule) = selection {
                    let arrow = match rule.direction() {
                        Direction::North => '^',
                        Direction::South => 'v',
                        Direction::East => '>',
                        Direction::West => '<',
                    };
                    assert_eq!(spelling.next(), Some(arrow), "{function:?}");
                }
                let bind = |values: &[Atom]| {
                    let values: Vec<Option<Atom>> = values.iter().copied().map(Some).collect();
                    match selection {
                        PairSelection::Distance(rule) => Some(rule.distance(&values).map(drop)),
                        PairSelection::Position(rule) => Some(rule.position(&values).map(drop)),
                        _ => None,
                    }
                };
                let arity = function.signature().len();
                let Some(own) = bind(&vec![Atom::Number(1); arity]) else {
                    continue;
                };
                declared += 1;
                assert!(own.is_ok(), "{function:?}: {own:?}");
                let extra = bind(&vec![Atom::Number(1); arity + 1]);
                assert!(
                    matches!(
                        extra,
                        Some(Err(Error::Argument(ArgumentError::Arity { expected, .. })))
                            if expected == arity
                    ),
                    "{function:?}: {extra:?}"
                );
            }
        }
        // The four directional Reads and Writes, and the absolute Read and Write.
        assert_eq!(declared, 10);
    }

    #[test]
    fn every_function_names_its_input_portal() {
        use crate::{
            Direction,
            InputPortal::{Dynamic, Static},
            PairSelection, PortalCoords, PositionRule,
        };
        // The arrow each Read's spelling names.
        const READS: [(Function, Direction); 4] = [
            (Function::ReadEast, Direction::East),
            (Function::ReadNorth, Direction::North),
            (Function::ReadSouth, Direction::South),
            (Function::ReadWest, Direction::West),
        ];

        for function in Function::ALL.iter().copied() {
            let input = function.input_portal();
            match function {
                Function::CopyEast => assert_eq!(
                    input,
                    Some(Static(PortalCoords {
                        columns: -2,
                        rows: 0
                    }))
                ),
                Function::CopyWest => assert_eq!(
                    input,
                    Some(Static(PortalCoords {
                        columns: 2,
                        rows: 0
                    }))
                ),
                Function::CopyNorth => assert_eq!(input, Some(Static(PortalCoords::SOUTH))),
                Function::CopySouth => assert_eq!(
                    input,
                    Some(Static(PortalCoords {
                        columns: 0,
                        rows: -1
                    }))
                ),
                // Increment and Interpolation read where they write.
                Function::Increment | Function::Interpolation => {
                    assert_eq!(input, Some(Static(PortalCoords::SOUTH)));
                }
                Function::Track => {
                    assert_eq!(input, Some(Dynamic(PairSelection::IndexModuloCount)));
                }
                Function::ReadEast
                | Function::ReadNorth
                | Function::ReadSouth
                | Function::ReadWest => {
                    let (_, direction) = READS
                        .iter()
                        .find(|(read, _)| *read == function)
                        .expect("every Read is listed");
                    assert!(
                        matches!(
                            input,
                            Some(Dynamic(PairSelection::Distance(rule)))
                                if rule.direction() == *direction
                        ),
                        "{function:?}: {input:?}"
                    );
                }
                Function::AbsoluteRead => assert_eq!(
                    input,
                    Some(Dynamic(PairSelection::Position(PositionRule::AbsoluteRead)))
                ),
                // The absolute Copy reads one Position and writes another.
                Function::AbsoluteCopy => {
                    assert_eq!(input, Some(Dynamic(PairSelection::SourcePosition)));
                }
                _ => assert_eq!(input, None, "{function:?}"),
            }
        }
    }

    #[test]
    fn every_function_does_one_thing_through_its_output_portal() {
        use crate::{DistanceRule, Output, PairSelection, PortalCoords, PositionRule};
        let east = PortalCoords {
            columns: 2,
            rows: 0,
        };
        let west = PortalCoords {
            columns: -2,
            rows: 0,
        };
        let north = PortalCoords {
            columns: 0,
            rows: -1,
        };

        for function in Function::ALL.iter().copied() {
            let output = function.output();
            // `output` reads a selection rule only for a value answer, so an
            // effect that declared one would lose it without a word.
            assert!(
                function.selection().is_none() || function.answers_value(),
                "{function:?}"
            );
            match function {
                Function::RawChordPlay
                | Function::RawPlay
                | Function::TimedChordPlay
                | Function::TimedPlay
                | Function::MonophonicPlay
                | Function::ControlChange
                | Function::PitchBend
                | Function::Udp
                | Function::Osc => assert_eq!(output, Output::Terminal, "{function:?}"),
                Function::SelfBangingEast
                | Function::SelfBangingNorth
                | Function::SelfBangingSouth
                | Function::SelfBangingWest
                | Function::DirectionalBangEast
                | Function::DirectionalBangNorth
                | Function::DirectionalBangSouth
                | Function::DirectionalBangWest
                | Function::Bang => {
                    assert!(matches!(output, Output::Source(_)), "{function:?}");
                }
                Function::Halt => assert_eq!(output, Output::Lock(PortalCoords::SOUTH)),
                Function::CopyEast => assert_eq!(output, Output::Static(east)),
                Function::CopyWest => assert_eq!(output, Output::Static(west)),
                Function::CopyNorth => assert_eq!(output, Output::Static(north)),
                Function::WriteEast => assert_eq!(
                    output,
                    Output::Dynamic(PairSelection::Distance(DistanceRule::WriteEast))
                ),
                Function::WriteNorth => assert_eq!(
                    output,
                    Output::Dynamic(PairSelection::Distance(DistanceRule::WriteNorth))
                ),
                Function::WriteSouth => assert_eq!(
                    output,
                    Output::Dynamic(PairSelection::Distance(DistanceRule::WriteSouth))
                ),
                Function::WriteWest => assert_eq!(
                    output,
                    Output::Dynamic(PairSelection::Distance(DistanceRule::WriteWest))
                ),
                Function::AbsoluteWrite => {
                    assert_eq!(
                        output,
                        Output::Dynamic(PairSelection::Position(PositionRule::AbsoluteWrite))
                    );
                }
                Function::Push => assert_eq!(output, Output::Dynamic(PairSelection::Lane)),
                Function::AbsoluteCopy => {
                    assert_eq!(output, Output::Dynamic(PairSelection::DestinationPosition));
                }
                _ => assert_eq!(output, Output::Static(PortalCoords::SOUTH), "{function:?}"),
            }
        }
    }

    #[test]
    fn replacing_a_read_with_track_another_read_or_a_copy_is_a_write() {
        // Each names its own rule for selecting its dynamic Input Portal, a
        // directional Copy declares static Portals, and the absolute Copy
        // also declares a dynamic Output Portal.
        const READS: [Function; 5] = [
            Function::AbsoluteRead,
            Function::ReadEast,
            Function::ReadNorth,
            Function::ReadSouth,
            Function::ReadWest,
        ];
        const COPIES: [Function; 5] = [
            Function::AbsoluteCopy,
            Function::CopyEast,
            Function::CopyNorth,
            Function::CopySouth,
            Function::CopyWest,
        ];
        for read in READS {
            assert_eq!(read.replacing(read), None, "{read:?}");
            let others = READS
                .into_iter()
                .filter(|other| *other != read)
                .chain(COPIES)
                .chain([Function::Track]);
            for other in others {
                assert_eq!(
                    other.replacing(read),
                    Some(ReplacementChange::Write),
                    "{other:?} replacing {read:?}"
                );
                assert_eq!(
                    read.replacing(other),
                    Some(ReplacementChange::Write),
                    "{read:?} replacing {other:?}"
                );
            }
        }
    }

    #[test]
    fn replacing_a_write_with_another_write_a_read_or_a_value_function_is_a_write() {
        // Each Write, and the absolute Copy, names its own rule for selecting
        // its dynamic Output Portal; a Read and Equality answer through a
        // static one.
        const WRITES: [Function; 7] = [
            Function::AbsoluteCopy,
            Function::AbsoluteWrite,
            Function::Push,
            Function::WriteEast,
            Function::WriteNorth,
            Function::WriteSouth,
            Function::WriteWest,
        ];
        for write in WRITES {
            assert_eq!(write.replacing(write), None, "{write:?}");
            let others = WRITES.into_iter().filter(|other| *other != write).chain([
                Function::AbsoluteRead,
                Function::Track,
                Function::ReadEast,
                Function::Equality,
            ]);
            for other in others {
                assert_eq!(
                    other.replacing(write),
                    Some(ReplacementChange::Write),
                    "{other:?} replacing {write:?}"
                );
                assert_eq!(
                    write.replacing(other),
                    Some(ReplacementChange::Write),
                    "{write:?} replacing {other:?}"
                );
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
        // The Playback Engine recognises a velocity `00` stop inside a Tick,
        // where a fallible conversion would be an unreachable failure path.
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
    fn each_function_names_its_operand_roles_in_signature_order() {
        let names = |function: Function| -> Vec<&str> {
            function.roles().iter().map(|role| role.name).collect()
        };
        assert_eq!(names(Function::Add), ["left", "right"]);
        assert_eq!(
            names(Function::TimedPlay),
            ["channel", "velocity", "note", "length"]
        );
        let covers = |function: Function| -> Vec<usize> {
            function.roles().iter().map(|role| role.operands).collect()
        };
        assert_eq!(names(Function::AbsoluteCopy), ["from", "to"]);
        assert_eq!(covers(Function::AbsoluteCopy), [2, 2]);
        assert_eq!(names(Function::AbsoluteRead), ["at"]);
        assert_eq!(covers(Function::AbsoluteRead), [2]);
        assert_eq!(names(Function::AbsoluteWrite), ["at", "value"]);
        assert_eq!(covers(Function::AbsoluteWrite), [2, 1]);
        assert!(Function::CopyEast.roles().is_empty());
        for function in Function::ALL {
            let roles = names(*function);
            let mut signature = function.signature().iter();
            for role in function.roles() {
                assert!(
                    role.operands >= 1,
                    "`{function}`'s `{}` covers no operand",
                    role.name
                );
                let covered: Vec<_> = signature.by_ref().take(role.operands).collect();
                assert_eq!(
                    covered.len(),
                    role.operands,
                    "`{function}`'s `{}` runs past its signature",
                    role.name
                );
                assert!(
                    role.operands == 1
                        || covered
                            .iter()
                            .all(|token| matches!(token, crate::Token::Number)),
                    "`{function}`'s grouped `{}` covers only Numbers",
                    role.name
                );
            }
            assert!(
                signature.next().is_none(),
                "`{function}`'s Roles cover each operand it declares"
            );
            for (index, role) in roles.iter().enumerate() {
                assert!(
                    !role.is_empty() && role.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                    "`{function}` names an operand `{role}`"
                );
                assert!(
                    !roles[..index].contains(role),
                    "`{function}` names `{role}` twice"
                );
            }
        }
    }

    const COLUMN_ROW: RoleGroup = RoleGroup {
        name: "at",
        members: &["column", "row"],
    };

    #[test]
    fn a_grouped_role_stands_for_the_operands_it_covers() {
        let roles: [Role; 2] = grouped_roles(
            &["column", "row", "value"],
            &["Number", "Number", "Untyped"],
            &[COLUMN_ROW],
        );
        assert_eq!(
            roles,
            [
                Role {
                    name: "at",
                    operands: 2
                },
                Role {
                    name: "value",
                    operands: 1
                },
            ]
        );
    }

    #[test]
    #[should_panic(expected = "a grouped Role covers only Number operands")]
    fn a_grouped_role_refuses_an_operand_that_is_not_a_number() {
        let _: [Role; 1] = grouped_roles(&["column", "row"], &["Number", "Note"], &[COLUMN_ROW]);
    }

    #[test]
    #[should_panic(expected = "a grouped Role covers consecutive declared operands")]
    fn a_grouped_role_refuses_operands_out_of_signature_order() {
        let _: [Role; 1] = grouped_roles(
            &["column", "value", "row"],
            &["Number", "Number", "Number"],
            &[COLUMN_ROW],
        );
    }

    #[test]
    #[should_panic(expected = "a `position:` clause's operands must be one grouped Role")]
    fn a_position_that_is_not_one_grouped_role_is_refused() {
        assert_position_grouped(&["column", "row"], &[]);
    }

    #[test]
    fn a_position_grouped_as_one_role_is_accepted() {
        assert_position_grouped(&["column", "row"], &[COLUMN_ROW]);
        assert_position_grouped(&[], &[]);
    }

    #[test]
    #[should_panic(expected = "a grouped Role must start at a declared operand")]
    fn a_grouped_role_refuses_a_group_the_row_does_not_declare() {
        let _: [Role; 1] = grouped_roles(&["left", "right"], &["Number", "Number"], &[COLUMN_ROW]);
    }

    #[test]
    #[should_panic(expected = "a grouped Role covers at least two operands")]
    fn a_grouped_role_refuses_a_single_operand() {
        role_count(
            1,
            &[RoleGroup {
                name: "at",
                members: &["column"],
            }],
        );
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
    fn pitch_functions_display_with_the_hash_family_spellings() {
        assert_eq!(Function::Up.to_string(), "#+");
        assert_eq!(Function::Down.to_string(), "#-");
        assert_eq!(Function::Pick.to_string(), "#p");
        assert_eq!(Function::Stack.to_string(), "#s");
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
                Function::AbsoluteCopy
                | Function::AbsoluteDifference
                | Function::AbsoluteRead
                | Function::Add
                | Function::Clock
                | Function::ConvertToNote
                | Function::ConvertToNumber
                | Function::Delay
                | Function::Divide
                | Function::Down
                | Function::Equality
                | Function::Euclidean
                | Function::Increment
                | Function::Interpolation
                | Function::CopyEast
                | Function::CopyNorth
                | Function::CopySouth
                | Function::CopyWest
                | Function::Maximum
                | Function::Minimum
                | Function::Modulo
                | Function::Multiply
                | Function::Pick
                | Function::Random
                | Function::ReadEast
                | Function::ReadNorth
                | Function::ReadSouth
                | Function::ReadWest
                | Function::Stack
                | Function::Subtract
                | Function::Track
                | Function::Up
                | Function::AbsoluteWrite
                | Function::Push
                | Function::WriteEast
                | Function::WriteNorth
                | Function::WriteSouth
                | Function::WriteWest => (true, false, true),
                Function::ControlChange
                | Function::MonophonicPlay
                | Function::PitchBend
                | Function::RawChordPlay
                | Function::RawPlay
                | Function::TimedChordPlay
                | Function::TimedPlay
                | Function::Udp
                | Function::Osc => (false, true, false),
                // An effect that is not Terminal Output, taken without a Bang.
                // These four rows are the whole reason the three predicates are
                // asked separately.
                Function::SelfBangingEast
                | Function::SelfBangingNorth
                | Function::SelfBangingSouth
                | Function::SelfBangingWest => (false, false, true),
                // Taken without a Bang too: the Bang Function's Turn is what
                // delivers one.
                Function::Bang => (false, false, true),
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
                function.output() == crate::Output::Terminal,
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
        assert_eq!(Atom::Function(Function::CopyNorth).to_string(), "=^");
        assert_eq!(Atom::Function(Function::CopySouth).to_string(), "=v");
        assert_eq!(Atom::Function(Function::CopyWest).to_string(), "=<");
        assert_eq!(Atom::Function(Function::CopyEast).to_string(), "=>");
    }

    #[test]
    fn test_notes_render_distinctly_from_numbers() {
        // Notes spell a pitch and an octave and are unaffected by hex Numbers
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
