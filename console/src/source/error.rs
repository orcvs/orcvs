use std::error::Error;
use std::fmt;

///
/// Why an edit was rejected. The Source is never mutated when an error is returned.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceError {
    /// The index does not name a Cell in this Source.
    OutOfRange { idx: usize, len: usize },
    /// A Cell holds exactly one printable single-byte ASCII character.
    InvalidCell { content: String },
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::OutOfRange { idx, len } => {
                write!(f, "index {idx} is out of range for a Source of {len} Cells")
            }
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
