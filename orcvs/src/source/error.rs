use std::error::Error;
use std::fmt;

///
/// Why an edit was rejected. The Source is never mutated when an error is returned.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceError {
    /// A Cell holds exactly one printable single-byte ASCII character.
    InvalidCell { content: String },
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
        }
    }
}

impl Error for SourceError {}
