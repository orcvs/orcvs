/// One printable ASCII character held by a Source Cell, including space.
///
/// Construction validates the content once; planned writes retain that proof
/// until commit. This describes content, not a Cell's Grid position.
///
/// ```
/// use orcvs::source::CellContent;
///
/// let content = CellContent::new(b'A').unwrap();
/// assert_eq!(content.as_char(), 'A');
/// assert!(CellContent::new(b'\n').is_none());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellContent(u8);

impl CellContent {
    pub(super) const SPACE: Self = Self(b' ');

    /// Accepts exactly the printable ASCII bytes, including space.
    pub fn new(byte: u8) -> Option<Self> {
        (0x20..=0x7e).contains(&byte).then_some(Self(byte))
    }

    /// The character to display for this Cell content.
    pub fn as_char(self) -> char {
        char::from(self.0)
    }

    pub(super) fn byte(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::CellContent;

    #[test]
    fn every_byte_is_classified_and_accepted_content_is_preserved() {
        for byte in u8::MIN..=u8::MAX {
            let content = CellContent::new(byte);
            assert_eq!(content.is_some(), byte == b' ' || byte.is_ascii_graphic());
            if let Some(content) = content {
                assert_eq!(content.byte(), byte);
                assert_eq!(content.as_char(), char::from(byte));
            }
        }
    }
}
