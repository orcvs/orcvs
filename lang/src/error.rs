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
    /// Raised where a scalar signature requires one Atom. Issue 02 replaces
    /// this diagnostic for Atomic Functions with broadcasting; the Functions
    /// that stay scalar keep it.
    #[error("expected an Atom, found the Sequence {0:?}")]
    ExpectedAtom(String),

    #[error("expected a Sequence, found {0:?}")]
    ExpectedSequence(String),

    /// An Atom with no place in a Sequence: a Self-Banging Function, which is
    /// a root-only Source effect, or the absence marker, which has no Source
    /// encoding of its own.
    #[error("{0:?} cannot be a Sequence member")]
    Member(String),

    /// Two non-scalar operands of unequal length. ADR 0007 pairs equal-length
    /// Sequences element-wise and diagnoses everything else; issue 02 raises
    /// this when it broadcasts.
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

    #[error("a terminal Function is valid only at the root of an Expression")]
    NestedTerminalFunction,

    #[error("MIDI channel {0:02X} is outside the range 00–0F")]
    MidiChannel(u8),

    /// The Operand Stack had no slot left for a value.
    ///
    /// No Expression the parser accepts can raise this: the `Args` declaration
    /// carries the proof. It exists because ADR 0028 requires every bound the
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

    #[error("expression exceeds the parser capacity of {capacity} atoms")]
    ExpressionTooLong { capacity: usize },
}

#[derive(Error, Debug)]
pub enum ArgumentError {
    #[error("invalid number of arguments (expected {expected:?}, found {found:?})")]
    // #[diagnostic(code(ArgumentError))]
    Arity { expected: usize, found: usize },

    #[error("expected an argument")]
    Expected,

    #[error("expected a function")]
    ExpectedFunction,
}
