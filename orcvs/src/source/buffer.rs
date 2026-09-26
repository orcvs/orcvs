use std::ops::Range;
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
        Cells::checked(text.as_bytes()).map(|cells| Self(Arc::from(cells.0)))
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// The Cells, lent as a view that carries their invariant.
    pub(super) fn cells(&self) -> Cells<'_> {
        Cells(&self.0)
    }

    /// The Cells as text, borrowed, through [`Cells::as_str`]'s check.
    pub(super) fn as_str(&self) -> &str {
        self.cells().as_str()
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

///
/// A borrowed run of Cells: one printable ASCII byte per Cell, in Grid order.
///
/// Tick planning, the Language Map and Claim lookup read the Cells through
/// this view rather than through bare bytes, which carry no invariant. Every
/// view starts from a [`SourceBuffer`], from [`WorkingCells`], or from bytes
/// [`Self::checked`] tested, and a narrower view is a range of one of those.
///
#[derive(Clone, Copy)]
pub(super) struct Cells<'a>(&'a [u8]);

impl<'a> Cells<'a> {
    /// `bytes` as Cells, or `None` when one of them is not a Cell's.
    pub(super) fn checked(bytes: &'a [u8]) -> Option<Self> {
        bytes
            .iter()
            .all(|&byte| CellContent::new(byte).is_some())
            .then_some(Self(bytes))
    }

    pub(super) fn bytes(self) -> &'a [u8] {
        self.0
    }

    /// The Cells in `range`. Panics when `range` runs past the last Cell.
    pub(super) fn slice(self, range: Range<usize>) -> Self {
        Self(&self.0[range])
    }

    /// The Cells in runs of `columns`, one per row.
    pub(super) fn rows(self, columns: usize) -> impl Iterator<Item = Self> {
        self.0.chunks_exact(columns).map(Self)
    }

    ///
    /// The Cells as text, borrowed.
    ///
    /// Checked at runtime rather than assumed: every byte is printable ASCII,
    /// so the check never fails, but that proof lives in the constructors
    /// rather than in a type the compiler can read. The check is linear in
    /// the view, so text is read from the narrowest view that holds it.
    ///
    pub(super) fn as_str(self) -> &'a str {
        std::str::from_utf8(self.0).expect("every Cell is printable ASCII")
    }
}

#[cfg(test)]
impl<'a> Cells<'a> {
    /// `bytes` as Cells, for a test whose fixture states them as text.
    /// Panics when one of them is not a Cell's.
    pub(super) fn of(bytes: &'a (impl AsRef<[u8]> + ?Sized)) -> Self {
        Self::checked(bytes.as_ref()).expect("a fixture's Cells are printable ASCII")
    }
}

///
/// The Cells one Tick writes as it executes, starting from the revision it
/// plans against. A write takes a [`CellContent`], so these stay Cells.
///
pub(super) struct WorkingCells(Vec<u8>);

impl WorkingCells {
    pub(super) fn new(cells: Cells<'_>) -> Self {
        Self(cells.0.to_vec())
    }

    pub(super) fn cells(&self) -> Cells<'_> {
        Cells(&self.0)
    }

    /// Writes `content` into the Cell at `index`. Panics when `index` is past
    /// the last Cell.
    pub(super) fn write(&mut self, index: usize, content: CellContent) {
        self.0[index] = content.byte();
    }
}

#[cfg(test)]
mod tests {
    use super::{Cells, SourceBuffer};
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

    #[test]
    fn bytes_no_cell_holds_are_not_cells() {
        assert!(Cells::checked(b"a\nb").is_none());
        assert!(Cells::checked("é".as_bytes()).is_none());
        assert_eq!(Cells::checked(b".+01 ").map(Cells::as_str), Some(".+01 "));
    }

    #[test]
    fn a_narrower_view_reads_only_its_cells() {
        let buffer = buffer(".+0102  ");

        let rows: Vec<_> = buffer.cells().rows(4).map(Cells::as_str).collect();

        assert_eq!(rows, [".+01", "02  "]);
        assert_eq!(buffer.cells().slice(2..6).as_str(), "0102");
    }
}
