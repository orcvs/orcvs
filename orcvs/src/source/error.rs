use std::error::Error;
use std::fmt;

///
/// Why an edit was rejected. The Source is never mutated when an error is returned.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceError {
    /// A Cell holds exactly one printable single-byte ASCII character.
    InvalidCell { content: String },
    /// The accepted edit would create an Expression the parser cannot hold.
    ExpressionTooLong {
        start: usize,
        end: usize,
        capacity: usize,
    },
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::InvalidCell { content } => {
                write!(
                    f,
                    "a Cell holds exactly one printable single-byte ASCII character, got {content:?}"
                )
            }
            SourceError::ExpressionTooLong {
                start,
                end,
                capacity,
            } => write!(
                f,
                "Expression at Cells {start}..={end} exceeds the parser capacity of {capacity} atoms"
            ),
        }
    }
}

impl Error for SourceError {}
