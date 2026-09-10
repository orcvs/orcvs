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
    Sequence(#[from] SequenceError),

    #[error(transparent)]
    Interpretation(#[from] InterpretationError),
}

/// Problems with the shape of a language value rather than the type of an
/// Operand Literal.
///
/// These live apart from [`TypeError`] because `TypeError` answers "these two
/// Source Cells do not read as the type this operand position requires", which
/// is a question about text. A Sequence is never spelled in Source: it exists
/// only between Functions, so every diagnostic here is about a value one
/// Function handed another — whether it was one Atom or many, whether an Atom
/// may be a member at all, and whether two operands have compatible lengths.
#[derive(Error, Debug)]
pub enum SequenceError {
    /// A Sequence at an operand position of a Function that declares it does
    /// not pervade. No Function declares that today: ADR 0030 made the
    /// Terminal Output family pervasive, which were the last rows that did not
    /// broadcast, and `Pervasion::Scalar` is now a declaration the table can
    /// express and nothing uses. ADR 0012's Increment and Interpolation are
    /// the candidates to use it when they arrive, and each would be refused by
    /// its declared pervasion rather than by a check written beside it.
    ///
    /// The variant is not thereby unreachable. The scalar pop `Stack::pop`
    /// offers outside Function evaluation raises it under the same rule: a
    /// Sequence has no scalar reading, and answering with its first Atom would
    /// silently discard the rest.
    #[error("expected an Atom, found the Sequence {0:?}")]
    ExpectedAtom(String),

    #[error("expected a Sequence, found {0:?}")]
    ExpectedSequence(String),

    /// An Atom with no place in a Sequence: a Self-Banging Function, which is
    /// a root-only Source effect; the absence marker, which has no Source
    /// encoding of its own; or a Function that answers an effect rather than a
    /// value, which ADR 0029 refuses by its declared kind.
    #[error("{0:?} cannot be a Sequence member")]
    Member(String),

    /// Two non-scalar operands of unequal length, named in signature order.
    /// ADR 0007 pairs equal-length Sequences element-wise and diagnoses
    /// everything else, including an empty Sequence against a non-empty one:
    /// an empty operand is a length rather than a scalar that repeats. The
    /// shape of an operation is settled before any of its elements is read, so
    /// this precedes every diagnostic about one element.
    #[error("incompatible Sequence lengths {left} and {right}")]
    IncompatibleLengths { left: usize, right: usize },

    /// An empty Sequence where indexing needs a member to reach. Select and
    /// Replace raise this in issue 03.
    #[error("expected a non-empty Sequence")]
    EmptyNotAllowed,
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
    /// ADR 0012 measures Clock's and Delay's cycle as `rate * modulus` and
    /// Euclidean's as `steps`, and none of the three has a cycle at all once a
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
    /// Unreachable while ADR 0012's formula stands: the step is
    /// `(Tick / rate) % modulus`, a remainder of a modulus that arrived as a
    /// Number, so it is below `FF` however far the Tick has counted. The
    /// variant exists for what the alternatives to it would cost. A panic
    /// states the invariant and is ruled out, because the narrowing runs
    /// inside a Tick under the Source write guard ADR 0028 forbids panicking
    /// under. A fallback Number is worse still: `00` is the first step
    /// of every cycle, so a formula that stopped being total would write a
    /// legal-looking step and leave no trace of having done it. Diagnosing is
    /// the remaining option, and it is the trade `Stack::convert` already
    /// makes — its fallback is the absence marker *because* the absence marker
    /// is not numeric, so an impossible state costs a type diagnostic rather
    /// than Playback. Here it costs the Expression its Cell write and names
    /// the step that could not be answered.
    #[error("{} cannot answer the step {step} as a Number", crate::Function::Clock)]
    ClockStepOutOfRange { step: u64 },

    /// Euclidean asked to place more onsets than its cycle has positions.
    ///
    /// ADR 0012 validates zero steps first and this second, so `(00, 00)` is a
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

    /// ADR 0028 states that an instruction answers either a value or an
    /// effect, so a Function answering an effect can stand only where nothing
    /// consumes an answer. Terminal Output is the one effect kind built today
    /// and this names the rule rather than that family, so the Source-writing
    /// Functions of ADR 0004 raise it by their declared kind alone.
    #[error("a Function that answers an effect is valid only at the root of an Expression")]
    NestedEffectFunction,

    #[error("MIDI channel {0:02X} is outside the range 00–0F")]
    MidiChannel(u8),

    /// The Operand Stack had no slot left for a value.
    ///
    /// Evaluation reserves one slot per Atom, enough for every accepted
    /// Expression. It exists because ADR 0028 requires every bound the
    /// machine relies on to be proven or diagnosed, and a proof alone still
    /// leaves the push one edit away from panicking inside a Tick.
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
    #[error("expected a function, found {0:?}")]
    Function(String),

    #[error("expected a bang, found {0:?}")]
    Bang(String),

    #[error("expected a number or note, found {0:?}")]
    Numeric(String),

    #[error("expected a note, found {0:?}")]
    Note(String),

    #[error("expected a number, found {0:?}")]
    Number(String),

    #[error("expected a char, found {0:?}")]
    Char(String),

    #[error("expected a string, found {0:?}")]
    String(String),
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

    /// A Comment where a value was required. ADR 0035 makes a Comment a
    /// complete Language Unit that is not a value: it records a Token and no
    /// Atom, so permissive analysis completes and strict parsing, which
    /// yields Atoms, has nothing to yield.
    #[error("a Comment is a Language Unit rather than a value")]
    CommentIsNotAValue,
}

#[derive(Error, Debug)]
pub enum ArgumentError {
    #[error("invalid number of arguments (expected {expected:?}, found {found:?})")]
    // #[diagnostic(code(ArgumentError))]
    Arity { expected: usize, found: usize },

    #[error("expected a function")]
    ExpectedFunction,
}
