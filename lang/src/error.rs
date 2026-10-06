use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Argument(#[from] ArgumentError),

    #[error(transparent)]
    Syntax(#[from] SyntaxError),

    #[error(transparent)]
    Type(#[from] TypeError),

    #[error(transparent)]
    Interpretation(#[from] InterpretationError),
}

#[derive(Error, Debug)]
pub enum InterpretationError {
    #[error("cannot divide by zero")]
    DivisionByZero,

    #[error("cannot modulo by zero")]
    ModuloByZero,

    #[error("Number {0:02X} cannot be converted to a Note")]
    NoteConversion(u8),

    /// A Tick-reading Function handed a zero where its cycle needs a length.
    ///
    /// Clock's and Delay's cycle is `rate * modulus` and Euclidean's is
    /// `steps`, and none of the three has a cycle at all once a
    /// factor is zero: there is no step to be at, no period to be on, and no
    /// position for an onset to fall in. Each therefore diagnoses rather than
    /// inventing a cycle, exactly as [`InterpretationError::DivisionByZero`]
    /// and [`InterpretationError::ModuloByZero`] refuse to invent a quotient.
    ///
    /// One variant carries all three faults because they are one fault under
    /// three role names, and the Source is told which Function it wrote and
    /// which of its operands held the zero — the same thing Division and
    /// Modulo achieve by being separate variants for a family of two, and the
    /// same shape [`InterpretationError::MidiDataByte`] uses for the roles
    /// that share the MIDI data-byte domain, down to the field name.
    /// `function` is the declared [`crate::Function`] rather than a spelling
    /// written down here, so the diagnostic renders the Cells the Source
    /// actually holds.
    #[error("{function} cannot count a cycle with a zero {role}")]
    ZeroCycle {
        function: crate::Function,
        role: &'static str,
    },

    /// Clock computed a step its Number answer cannot hold.
    ///
    /// Unreachable while the step formula stands: the step is
    /// `(Tick / rate) % modulus`, a remainder of a modulus that arrived as a
    /// Number, so it is below `FF` however far the Tick has counted. The
    /// variant exists for what the alternatives to it would cost. A panic
    /// states the invariant and is ruled out, because the narrowing runs
    /// inside a Tick under the Source write guard ADR 0028 forbids panicking
    /// under. A fallback Number is worse still: `00` is the first step
    /// of every cycle, so a formula that stopped being total would write a
    /// legal-looking step and leave no trace of having done it. Diagnosing is
    /// the remaining option: it costs the Expression its Cell write rather
    /// than Playback, and names the step that could not be answered.
    #[error("{} cannot answer the step {step} as a Number", crate::Function::Clock)]
    ClockStepOutOfRange { step: u64 },

    /// Euclidean asked to place more onsets than its cycle has positions.
    ///
    /// Zero steps is validated first and this second, so `(00, 00)` is a
    /// cycle of no length rather than a pattern of no hits. Both operands are
    /// named because either one is the Cell pair the Source would edit. The
    /// Function is named in the message rather than carried in a field, unlike
    /// [`InterpretationError::ZeroCycle`] above: only one Function can raise
    /// this, so there is nothing for a field to distinguish. It is still
    /// rendered from the declaration rather than spelled out in prose, because
    /// a Source shown `~% cannot count a cycle with a zero step count` beside
    /// `Euclidean cannot fit ...` has been told about two Functions where it
    /// wrote one: the spelling is what its Cells hold.
    #[error(
        "{} cannot fit {hits:02X} hits into {steps:02X} steps",
        crate::Function::Euclidean
    )]
    EuclideanOverfull { hits: u8, steps: u8 },

    /// Increment or Track handed a zero where its wrap needs a length.
    ///
    /// Track wraps its index at its count. The Parser refuses a count of
    /// `00` before any Turn, so evaluation raises this only for operands
    /// handed to it directly.
    ///
    /// Increment's `(previous + step) % modulus` has no wrap once the modulus
    /// is zero, so Increment diagnoses rather than inventing one. The variant
    /// is its own rather than [`InterpretationError::ZeroCycle`] because
    /// Increment is not counting a cycle: it is wrapping a running Number, and
    /// a Source shown "cannot count a cycle" would be told about a Function it
    /// did not write. It is also not [`InterpretationError::ModuloByZero`]:
    /// that names Modulo, and a Source shown two messages about one Cell pair
    /// should not have to work out that they name one Function. `function` is
    /// the declared Function so the message renders the Cells the Source holds.
    #[error("{function} cannot wrap at a zero {role}")]
    ZeroWrap {
        function: crate::Function,
        role: &'static str,
    },

    /// Increment or Interpolation found a previous that is not a Number.
    ///
    /// An empty Portal reads as Number `00` and any other present Language
    /// Unit diagnoses. Portal binding raises this after cell operand
    /// validation, so the fault is named here rather than
    /// as a [`crate::TypeError`] about operand Cells the Source did not write
    /// as an operand. `function` is the declared Function and `role` names
    /// what was read, so the message says which of the two feedback Functions
    /// the Source wrote.
    #[error("{function} cannot read a {role} that is not a Number")]
    PortalInputNotNumber {
        function: crate::Function,
        role: &'static str,
    },

    /// A Jump's input Portal did not hold one complete aligned Language Unit.
    ///
    /// Empty and Bang are legal inputs; a partial pair or a slice across two
    /// units is not. `function` is the Jump the Source
    /// wrote so the message names those Cells.
    #[error("{function} has partial or invalid input")]
    JumpInput { function: crate::Function },

    /// Increment or Interpolation computed a Number their answer cannot hold.
    ///
    /// Unreachable while the formulas stand: Increment's result is a
    /// remainder of a modulus that arrived as a Number, and Interpolation
    /// moves a Number toward a Number, so both stay below `FF`. The variant
    /// exists for the reason [`InterpretationError::ClockStepOutOfRange`]
    /// does: a panic is ruled out under the Source write guard, and a
    /// fallback Number would write a legal-looking answer with no trace.
    #[error("{function} cannot answer {value} as a Number")]
    TickNumberOutOfRange {
        function: crate::Function,
        value: u64,
    },

    #[error("MIDI channel {0:02X} is outside the range 00–0F")]
    MidiChannel(u8),

    /// The Operand Stack had no slot left for a value.
    ///
    /// The Interpreter refuses an operand list of any length other than its
    /// Function's signature before building the stack, and the stack holds
    /// the widest signature, so no evaluation it accepts reaches this. It
    /// exists because that proof alone still leaves the push one edit away
    /// from panicking inside a Tick.
    #[error("the Operand Stack cannot hold more than {capacity} values")]
    OperandStackExhausted { capacity: usize },

    /// `role` names the operand the Source supplied so one diagnostic serves
    /// every data byte in the Terminal Output family: a Play velocity, a
    /// Control Change controller or value, a Pitch Bend LSB or MSB.
    #[error("MIDI {role} {value:02X} is outside the range 00–7F")]
    MidiDataByte { role: &'static str, value: u8 },
}

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("expected a note, found {0:?}")]
    Note(String),

    #[error("expected a number, found {0:?}")]
    Number(String),
}

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("expected a function")]
    ExpectedFunction,

    #[error("expected a token")]
    ExpectedToken,

    #[error("unknown function {0:?}")]
    UnknownFunction(String),

    #[error("unexpected trailing content {0:?}")]
    UnexpectedTrailingContent(String),

    /// A nested Function returns one two-Cell answer to the operand it
    /// stands in, so a Function that answers an effect can stand only where
    /// nothing consumes an answer. This names the rule rather than any one
    /// effect family, so every effect Function raises it by its declared
    /// kind alone, and the Source shows it before any Tick runs.
    #[error("a Function that answers an effect is valid only at the root of an Expression")]
    NestedEffectFunction,

    /// A List Function's count where the Source does not spell a Number
    /// literal. The count decides how many Items the claim holds before any
    /// Function evaluates, so a nested Function cannot supply it.
    #[error("a List count is a literal Number")]
    ListCountNotLiteral,

    /// A List Function's count of `00`. A List holds at least one Item, so
    /// there is nothing to select and no claim to establish.
    #[error("a List count of 00 holds no Item")]
    EmptyList,

    /// A Comment where a value was required. A Comment is a complete
    /// Language Unit that is not a value: it records a Token and no
    /// Atom, so permissive analysis completes and strict parsing, which
    /// yields Atoms, has nothing to yield.
    #[error("a Comment is a Language Unit rather than a value")]
    CommentIsNotAValue,
}

#[derive(Error, Debug)]
pub enum ArgumentError {
    #[error("invalid number of arguments (expected {expected:?}, found {found:?})")]
    Arity { expected: usize, found: usize },
}
