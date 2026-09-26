use std::sync::Arc;

use super::CellContent;

///
/// The Cells of one Source: one printable ASCII byte per Cell, in Grid order.
///
/// A clone shares the bytes rather than copying them, so a planning snapshot
/// or a Source revision read from the Source costs a reference count. A write
/// copies them first only while such a reader still holds them, which leaves
/// every reader with the Cells it took, whatever the Source writes after.
///
/// Every byte is a [`CellContent`]'s: the buffer starts as empty Cells or as
/// text each byte of which was checked, and a write takes a `CellContent`.
///
#[derive(Clone)]
pub(super) struct SourceBuffer(Arc<[u8]>);

impl SourceBuffer {
    /// `len` empty Cells.
    pub(super) fn empty(len: usize) -> Self {
        Self(std::iter::repeat_n(CellContent::SPACE.byte(), len).collect())
    }

    /// The Cells `text` holds, or `None` when one of its bytes is not a Cell's.
    #[cfg(feature = "persistence")]
    pub(super) fn from_text(text: &str) -> Option<Self> {
        text.bytes()
            .all(|byte| CellContent::new(byte).is_some())
            .then(|| Self(Arc::from(text.as_bytes())))
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.0
    }

    ///
    /// The Cells as text, borrowed.
    ///
    /// Checked rather than assumed: every byte is printable ASCII, so the
    /// check never fails, but that proof lives in the constructors and
    /// [`Self::write`] rather than in the type.
    ///
    pub(super) fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("every Cell is printable ASCII")
    }

    ///
    /// Writes `content` into the Cell at `index`.
    ///
    /// Copies the Cells first when a reader shares them, so the reader keeps
    /// the revision it took. Panics when `index` is past the last Cell; the
    /// Source only passes an index its own Grid minted.
    ///
    pub(super) fn write(&mut self, index: usize, content: CellContent) {
        Arc::make_mut(&mut self.0)[index] = content.byte();
    }
}

#[cfg(test)]
mod tests {
    use super::SourceBuffer;
    use crate::source::CellContent;

    fn content(byte: u8) -> CellContent {
        CellContent::new(byte).expect("printable ASCII")
    }

    fn buffer(text: &str) -> SourceBuffer {
        let mut buffer = SourceBuffer::empty(text.len());
        for (index, byte) in text.bytes().enumerate() {
            buffer.write(index, content(byte));
        }
        buffer
    }

    #[test]
    fn a_clone_shares_the_cells() {
        let buffer = buffer("abc");

        let reader = buffer.clone();

        assert_eq!(reader.bytes().as_ptr(), buffer.bytes().as_ptr());
    }

    #[test]
    fn a_write_while_a_reader_holds_the_cells_leaves_the_reader_unchanged() {
        let mut buffer = buffer("abc");
        let reader = buffer.clone();

        buffer.write(1, content(b'x'));

        assert_eq!(reader.as_str(), "abc");
        assert_eq!(buffer.as_str(), "axc");
        assert_ne!(reader.bytes().as_ptr(), buffer.bytes().as_ptr());
    }

    #[test]
    fn a_write_no_reader_shares_is_made_in_place() {
        let mut buffer = SourceBuffer::empty(3);
        let before = buffer.bytes().as_ptr();

        buffer.write(2, content(b'!'));

        assert_eq!(buffer.bytes().as_ptr(), before);
        assert_eq!(buffer.as_str(), "  !");
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn text_holding_a_byte_no_cell_holds_is_refused() {
        assert!(SourceBuffer::from_text("a\nb").is_none());
        assert!(SourceBuffer::from_text("é").is_none());
        assert!(SourceBuffer::from_text("a b~").is_some());
    }
}
