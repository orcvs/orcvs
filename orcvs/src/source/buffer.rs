use std::ops::Range;
use std::sync::Arc;

use super::CellContent;
use crate::grid::CellIndex;

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
/// Which Cells an edit has written since the last committed Tick: one bit per
/// Cell, in Grid order, so the shipped 256 × 256 Grid holds 8 KiB.
///
/// A clone shares the bits, as a [`SourceBuffer`] clone shares its Cells, so a
/// revision captures them for a reference count. Marking a Cell already
/// marked writes nothing, and marking one copies the bits first only while a
/// revision still shares them.
///
#[derive(Clone)]
pub(super) struct EditedCells {
    bits: Arc<[u64]>,
    /// The Cells the set covers. The last word's bits past it are never
    /// marked.
    len: usize,
    /// Whether any bit is set, so a set no edit has marked answers without a
    /// walk and is cleared without one.
    any: bool,
}

impl EditedCells {
    /// No Cell of `len` edited.
    pub(super) fn none(len: usize) -> Self {
        Self {
            bits: std::iter::repeat_n(0, len.div_ceil(64)).collect(),
            len,
            any: false,
        }
    }

    /// Marks the Cell at `index` edited. `index` must name one of the Cells
    /// the set covers, which a debug build asserts.
    pub(super) fn mark(&mut self, index: usize) {
        debug_assert!(index < self.len, "Cell {index} of {}", self.len);
        let (word, bit) = (index / 64, 1 << (index % 64));
        if self.bits[word] & bit == 0 {
            Arc::make_mut(&mut self.bits)[word] |= bit;
            self.any = true;
        }
    }

    ///
    /// No Cell edited, over the same Cells. The bits are cleared in place
    /// where no revision shares them, and only where an edit marked one, so a
    /// commit after a Tick with no edit writes nothing; a shared set is left
    /// to its revisions and replaced.
    ///
    pub(super) fn cleared(mut self) -> Self {
        if self.any {
            match Arc::get_mut(&mut self.bits) {
                Some(bits) => bits.fill(0),
                None => return Self::none(self.len),
            }
            self.any = false;
        }
        self
    }

    /// Whether any Cell in `range` is marked. `range` must lie within the
    /// Cells the set covers, which a debug build asserts.
    pub(super) fn any_in(&self, range: Range<usize>) -> bool {
        debug_assert!(range.end <= self.len, "Cells {range:?} of {}", self.len);
        self.any
            && range
                .into_iter()
                .any(|index| self.bits[index / 64] & (1 << (index % 64)) != 0)
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

    /// How many Cells the view holds.
    pub(super) fn len(self) -> usize {
        self.0.len()
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
    pub(super) fn of(bytes: &'a [u8]) -> Self {
        Self::checked(bytes).expect("a fixture's Cells are printable ASCII")
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

    /// The Cells in `range` as text, checked over `range` alone. Panics when
    /// `range` runs past the last Cell.
    pub(super) fn text(&self, range: Range<usize>) -> &str {
        self.cells().slice(range).as_str()
    }

    /// Whether the Cell at `index` is empty. Panics when `index` is past the
    /// last Cell.
    pub(super) fn is_empty_at(&self, index: usize) -> bool {
        self.0[index] == CellContent::SPACE.byte()
    }

    /// Writes `content` into `cell`. Panics when `cell` is past the last Cell.
    pub(super) fn write(&mut self, cell: CellIndex, content: CellContent) {
        self.0[cell.get()] = content.byte();
    }
}

#[cfg(test)]
mod tests {
    use super::{Cells, EditedCells, SourceBuffer};
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
    fn bytes_outside_printable_ascii_are_not_cells() {
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

    #[test]
    fn edited_cells_answer_each_mark_and_a_shared_set_keeps_its_own() {
        let mut edited = EditedCells::none(130);
        assert!(!edited.any_in(0..130));
        edited.mark(64);
        edited.mark(129);
        assert!(edited.any_in(64..65));
        assert!(edited.any_in(100..130));
        assert!(!edited.any_in(0..64));
        assert!(!edited.any_in(65..129));

        // A revision's copy keeps the marks it was read with, while the
        // Source's set is cleared for the next commit and marked again.
        let revision = edited.clone();
        let mut next = edited.cleared();
        assert!(!next.any_in(0..130));
        next.mark(3);
        assert!(revision.any_in(64..65));
        assert!(!revision.any_in(3..4));
        assert!(next.any_in(3..4));
    }
}
