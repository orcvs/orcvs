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
    #[error("a Play Function is valid only at the root of an Expression")]
    NestedPlay,

    #[error("Play velocity {0:02X} is outside the MIDI range 00–7F")]
    PlayVelocity(u8),
}

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("expected a function, found {0:?}")]
    Function(String),

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
