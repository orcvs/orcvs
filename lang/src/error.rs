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

    #[error("a terminal Function is valid only at the root of an Expression")]
    NestedTerminalFunction,

    #[error("MIDI channel {0:02X} is outside the range 00–0F")]
    MidiChannel(u8),

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
