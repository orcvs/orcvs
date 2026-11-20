use tracing::error;

use crate::grid::{Grid, Position};
use crate::opts::SectorSeamSpacing;
use crate::region::Region;
use crate::source::{CellContent, CellWrite, SourceCommander};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKey {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Backspace,
    Delete,
    Space,
    Tab,
}

///
/// One of the four directions an arrow key moves the Cursor.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    Down,
    Left,
    Right,
    Up,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputEvent {
    KeyPressed(InputKey),
    ///
    /// Shift with an arrow: moves the Cursor and keeps the anchor, so the
    /// Region grows or shrinks from the Cell it was spanned from.
    ///
    Extend(Arrow),
    ///
    /// Shift Tab: moves the Cursor to its Sector's first Cell, or from
    /// there to the previous Sector's, the way a bare Tab moves it on to the
    /// next.
    ///
    PreviousSector,
    ///
    /// Command `A`: spans the Region across the whole Grid.
    ///
    SelectAll,
    ///
    /// Escape: collapses the Region onto the Cursor.
    ///
    Collapse,
    ///
    /// Command Enter: the next event, when it is a character, fills every
    /// Cell of the Region with it. Any other event disarms the fill.
    ///
    Fill,
    ///
    /// Puts the Region's rows on the clipboard.
    ///
    Copy,
    ///
    /// Puts the Region's rows on the clipboard, then empties the Region.
    ///
    Cut,
    ///
    /// Writes the text from the Region's top-left and spans the Region over
    /// what landed.
    ///
    Paste(String),
    ///
    /// Command Shift `R`: makes Tick `0` the next Tick Playback executes,
    /// without playing or stopping it. Space then resumes from there.
    ///
    Rewind,
    Text(String),
}

///
/// What one event the [`Editor`] handled did beyond moving the Region.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Edit {
    /// The event was an edit, and the Source may have changed: a refused
    /// write or an empty paste answers it too, having changed nothing.
    Wrote,
    /// The event put the Region's rows on the clipboard and wrote nothing.
    Copied(String),
    /// The event put the Region's rows on the clipboard, then emptied them.
    Cut(String),
}

///
/// The Region, with its Cursor, and the edits a viewer makes through it: one
/// running Orcvs's editing state, apart from its Playback.
///
/// The Editor writes through the [`SourceCommander`] each edit is handed
/// rather than holding one, so the running Orcvs stays the one owner of the
/// Source root and the Editor needs no runtime to be built or tested.
///
/// Selection names a Position, and only a Grid mints one. A pair outside the
/// Grid never becomes a Position at all, so `select` has no rejection to make
/// and cannot silently leave the Cursor on the Cell it was already on and send
/// the next write there:
///
/// ```
/// use orcvs::editor::Editor;
/// use orcvs::grid::Grid;
/// use orcvs::opts::Opts;
///
/// let grid = Grid::new();
/// let mut editor = Editor::new(grid, Opts::new().sector_seam_spacing);
///
/// // the Grid refuses a pair outside itself, so there is no Position to select
/// assert_eq!(grid.position(256, 256), None);
///
/// // every Position `select` can be handed is one this Grid minted
/// let position = grid.position(15, 15).expect("inside the grid");
/// editor.select(position);
/// assert_eq!(editor.region().cursor(), position);
/// ```
///
#[derive(Debug)]
pub struct Editor {
    grid: Grid,
    sector_seam_spacing: SectorSeamSpacing,
    region: Region,
    ///
    /// Whether command Enter has armed a fill for the next character.
    ///
    /// Held across input batches, because the chord and the character are
    /// two presses a viewer makes one after the other, and those may arrive in
    /// different frames.
    ///
    fill_armed: bool,
}

impl Editor {
    ///
    /// An Editor on `grid` with the Region the one Cell at its origin. Tab and
    /// Shift Tab step a Sector of `sector_seam_spacing` Cells at a time.
    ///
    pub fn new(grid: Grid, sector_seam_spacing: SectorSeamSpacing) -> Self {
        Self {
            grid,
            sector_seam_spacing,
            region: Region::at(grid, grid.origin()),
            fill_armed: false,
        }
    }

    ///
    /// The Region from the anchor to its live end, with the Cursor on one of
    /// its Cells.
    ///
    pub fn region(&self) -> Region {
        self.region
    }

    ///
    /// Moves the Cursor to `position` and collapses the Region onto it,
    /// refusing a Position minted by another Grid. Disarms a fill, as any
    /// event between command Enter and its character does.
    ///
    pub fn select(&mut self, position: Position) {
        self.grid.assert_owns(position);
        self.fill_armed = false;
        self.region = Region::at(self.grid, position);
    }

    ///
    /// Moves the Cursor to `position` and keeps the anchor, so the Region
    /// spans from the anchor to `position`. Disarms a fill, as
    /// [`select`](Self::select) does.
    ///
    /// ```
    /// use orcvs::editor::Editor;
    /// use orcvs::grid::Grid;
    /// use orcvs::opts::Opts;
    ///
    /// let grid = Grid::new();
    /// let mut editor = Editor::new(grid, Opts::new().sector_seam_spacing);
    /// let at = |x, y| grid.position(x, y).expect("inside the Grid");
    ///
    /// editor.select(at(5, 3));
    /// editor.extend(at(2, 1));
    ///
    /// let region = editor.region();
    /// assert_eq!((region.columns(), region.rows()), (2..6, 1..4));
    /// ```
    ///
    pub fn extend(&mut self, position: Position) {
        self.grid.assert_owns(position);
        self.fill_armed = false;
        self.region = Region::span(self.grid, self.region.anchor(), position);
    }

    ///
    /// Disarms a fill armed by command Enter, for input that went to
    /// something other than the Source and so never reaches
    /// [`Orcvs::event_handler`](crate::app::Orcvs::event_handler) as the event
    /// that would have disarmed it.
    ///
    pub fn disarm_fill(&mut self) {
        self.fill_armed = false;
    }

    ///
    /// Handles one event against `source`, and answers what it wrote or put on
    /// the clipboard, or nothing when it only moved the Region or was not an
    /// edit at all.
    ///
    /// Every event disarms a fill armed by the event before it, Space and
    /// Rewind included: a fill is armed for exactly one event, the character
    /// it fills with or whatever else arrived instead. Space and Rewind are
    /// otherwise Playback requests, which the Editor leaves to its caller, so
    /// only the running Orcvs that answers them hands the Editor events.
    ///
    /// Refuses a `source` on another Grid: the Editor indexes it with
    /// Positions its own Grid minted.
    ///
    pub(crate) fn handle(&mut self, event: &InputEvent, source: &SourceCommander) -> Option<Edit> {
        assert_eq!(
            source.grid(),
            self.grid,
            "the Source is on another Grid than the Editor's"
        );
        let fill_armed = std::mem::take(&mut self.fill_armed);
        let cursor = self.region.cursor();
        match event {
            InputEvent::KeyPressed(InputKey::ArrowDown) => {
                self.collapse_to(self.stepped(cursor, Arrow::Down))
            }
            InputEvent::KeyPressed(InputKey::ArrowLeft) => {
                self.collapse_to(self.stepped(cursor, Arrow::Left))
            }
            InputEvent::KeyPressed(InputKey::ArrowRight) => {
                self.collapse_to(self.stepped(cursor, Arrow::Right))
            }
            InputEvent::KeyPressed(InputKey::ArrowUp) => {
                self.collapse_to(self.stepped(cursor, Arrow::Up))
            }
            InputEvent::KeyPressed(InputKey::Tab) => {
                self.collapse_to(self.grid.next_sector(cursor, self.sector_seam_spacing))
            }
            InputEvent::PreviousSector => {
                self.collapse_to(self.grid.previous_sector(cursor, self.sector_seam_spacing))
            }
            // Stepped from the live end rather than the Cursor, which
            // command A may have left inside the Region; the Cursor
            // rejoins the live end.
            InputEvent::Extend(arrow) => {
                let end = self.stepped(self.region.end(), *arrow);
                self.region = Region::span(self.grid, self.region.anchor(), end);
            }
            InputEvent::SelectAll => self.region = Region::whole(self.grid, cursor),
            InputEvent::Collapse => self.collapse_to(cursor),
            InputEvent::KeyPressed(InputKey::Backspace | InputKey::Delete) => {
                if self.region.is_one_cell() {
                    self.delete(source);
                } else {
                    self.fill(CellContent::SPACE, source);
                }
                return Some(Edit::Wrote);
            }
            InputEvent::Fill => self.fill_armed = true,
            InputEvent::Copy => return Some(Edit::Copied(self.region_text(source))),
            InputEvent::Cut => {
                let text = self.region_text(source);
                self.fill(CellContent::SPACE, source);
                return Some(Edit::Cut(text));
            }
            InputEvent::Paste(text) => {
                self.paste(text, source);
                return Some(Edit::Wrote);
            }
            InputEvent::KeyPressed(InputKey::Space) | InputEvent::Rewind => {}
            InputEvent::Text(text_to_insert)
                if text_to_insert.len() == 1 && text_to_insert != " " =>
            {
                match CellContent::new(text_to_insert.as_bytes()[0]) {
                    Some(content) if fill_armed => self.fill(content, source),
                    _ => self.write(text_to_insert, source),
                }
                return Some(Edit::Wrote);
            }
            InputEvent::Text(_) => {}
        }
        None
    }

    ///
    /// Moves the Cursor to `position` and the anchor with it, so the Region
    /// is that one Cell.
    ///
    fn collapse_to(&mut self, position: Position) {
        self.region = Region::at(self.grid, position);
    }

    ///
    /// Writes `s` into the Cell under the Cursor and collapses the Region onto
    /// the next Cell right, clamped at the last column. A refused edit is
    /// logged and moves nothing.
    ///
    fn write(&mut self, s: &str, source: &SourceCommander) {
        let cursor = self.region.cursor();
        match source.set(self.grid.index(cursor), s) {
            Ok(_) => self.collapse_to(self.grid.right(cursor)),
            Err(e) => error!("rejected edit: {e}"),
        }
    }

    ///
    /// Empties the Cell under the Cursor and steps left.
    ///
    /// The Cursor sits on a Position this Grid minted, so the Cell it names
    /// exists and emptying it cannot be refused.
    ///
    fn delete(&mut self, source: &SourceCommander) {
        let cursor = self.region.cursor();
        source.unset(self.grid.index(cursor));
        self.collapse_to(self.grid.left(cursor));
    }

    ///
    /// The Region's rows as text joined by newlines, an empty Cell as a
    /// space, so trailing spaces keep the Region's shape.
    ///
    fn region_text(&self, source: &SourceCommander) -> String {
        let revision = source.read_revision();
        let rows: Vec<String> = self
            .region
            .positions_by_row()
            .map(|row| {
                row.map(|position| revision.content_at(position).unwrap_or(' '))
                    .collect()
            })
            .collect();
        rows.join("\n")
    }

    ///
    /// Writes `text` from the Region's top-left, in one revision, and spans
    /// the Region over the rectangle that landed.
    ///
    /// The text lands as a block, spaces included, so it replaces what was
    /// under it: a row shorter than the longest lands empty Cells to the
    /// block's width, which is what makes the Region the rectangle that
    /// landed and a copy of it the text that was pasted. A character that
    /// cannot be a Cell lands as an empty Cell rather than shifting the rest
    /// of its row. `\r\n` and `\n` both break a row, and a break at the very
    /// end adds no row of its own. What runs past the Grid's right or bottom
    /// edge is dropped. The Cursor stays on the top-left, so the Source View
    /// does not move to follow it.
    ///
    fn paste(&mut self, text: &str, source: &SourceCommander) {
        let text = text.trim_suffix('\n');
        let text = text.trim_suffix('\r');
        let top_left = self.region.top_left();
        let rows: Vec<Vec<char>> = text
            .split('\n')
            .map(|line| line.trim_suffix('\r').chars().collect())
            .take(self.grid.rows() - top_left.y())
            .collect();
        let width = rows
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .min(self.grid.columns() - top_left.x());
        if width == 0 {
            return;
        }
        let mut writes = Vec::with_capacity(width * rows.len());
        for (dy, row) in rows.iter().enumerate() {
            for dx in 0..width {
                let position = self
                    .grid
                    .position(top_left.x() + dx, top_left.y() + dy)
                    .expect("the block is clipped to the Grid");
                let content = row
                    .get(dx)
                    .and_then(|character| u8::try_from(*character).ok())
                    .and_then(CellContent::new)
                    .unwrap_or(CellContent::SPACE);
                writes.push(CellWrite {
                    cell: self.grid.index(position),
                    content,
                });
            }
        }
        source.write_cells(&writes);
        let far_corner = self
            .grid
            .position(top_left.x() + width - 1, top_left.y() + rows.len() - 1)
            .expect("the far corner of the block is clipped to the Grid");
        self.region = Region::span(self.grid, far_corner, top_left);
    }

    ///
    /// Writes `content` into every Cell of the Region, in one revision, and
    /// keeps the Region.
    ///
    fn fill(&mut self, content: CellContent, source: &SourceCommander) {
        let writes: Vec<CellWrite> = self
            .region
            .positions_by_row()
            .flatten()
            .map(|position| CellWrite {
                cell: self.grid.index(position),
                content,
            })
            .collect();
        source.write_cells(&writes);
    }

    ///
    /// The Cell one step from `from` towards `arrow`, clamped at the
    /// Grid's edge.
    ///
    fn stepped(&self, from: Position, arrow: Arrow) -> Position {
        match arrow {
            Arrow::Down => self.grid.down(from),
            Arrow::Left => self.grid.left(from),
            Arrow::Right => self.grid.right(from),
            Arrow::Up => self.grid.up(from),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Arrow, Edit, Editor, InputEvent, InputKey};
    use crate::grid::{Grid, Position};
    use crate::opts::{DEFAULT_SECTOR_SEAM_SPACING, SectorSeamSpacing};
    use crate::region::Region;
    use crate::source::SourceCommander;
    use crate::source::rows::commander;
    use crate::test::trace;

    ///
    /// An Editor and the Source it edits, as a running Orcvs pairs them.
    ///
    struct Editing {
        editor: Editor,
        source: SourceCommander,
        grid: Grid,
    }

    impl Editing {
        ///
        /// An Editor over an empty Source of `cols` by `rows`.
        ///
        fn blank(cols: usize, rows: usize) -> Self {
            Self::over(SourceCommander::new(Grid::with_shape(cols, rows)))
        }

        ///
        /// An Editor over a Source of `text`'s rows, each padded to the widest.
        ///
        fn written(text: &[&str]) -> Self {
            Self::over(commander(text))
        }

        fn over(source: SourceCommander) -> Self {
            let spacing =
                SectorSeamSpacing::new(DEFAULT_SECTOR_SEAM_SPACING).expect("a positive spacing");
            let grid = source.grid();
            Self {
                editor: Editor::new(grid, spacing),
                source,
                grid,
            }
        }

        ///
        /// The same Source with Sectors `cells` wide.
        ///
        fn with_sectors(mut self, cells: usize) -> Self {
            let spacing = SectorSeamSpacing::new(cells).expect("a positive spacing");
            self.editor = Editor::new(self.grid, spacing);
            self
        }

        fn at(&self, x: usize, y: usize) -> Position {
            self.grid.position(x, y).expect("inside the Grid")
        }

        ///
        /// Handles `events` in order and answers the edits they made.
        ///
        fn handle(&mut self, events: impl IntoIterator<Item = InputEvent>) -> Vec<Edit> {
            events
                .into_iter()
                .filter_map(|event| self.editor.handle(&event, &self.source))
                .collect()
        }

        fn select(&mut self, x: usize, y: usize) {
            let position = self.at(x, y);
            self.editor.select(position);
        }

        fn extend(&mut self, x: usize, y: usize) {
            let position = self.at(x, y);
            self.editor.extend(position);
        }

        fn region(&self) -> Region {
            self.editor.region()
        }

        ///
        /// The Source's rows, as a test reads them back.
        ///
        fn rows(&self) -> Vec<String> {
            self.source
                .snapshot()
                .as_bytes()
                .chunks(self.grid.columns())
                .map(|row| String::from_utf8(row.to_vec()).expect("printable ASCII"))
                .collect()
        }
    }

    fn text(s: &str) -> InputEvent {
        InputEvent::Text(s.to_owned())
    }

    #[test]
    fn a_new_editor_has_a_region_of_the_origins_one_cell() {
        let editing = Editing::blank(4, 3);

        assert_eq!(
            editing.region(),
            Region::at(editing.grid, editing.grid.origin())
        );
    }

    #[test]
    fn test_select_moves_the_cursor_to_the_position() {
        trace();

        let mut editing = Editing::blank(DEFAULT_SECTOR_SEAM_SPACING, 1);

        editing.select(3, 0);

        assert_eq!(editing.region().cursor(), editing.at(3, 0));
    }

    #[test]
    fn test_write_moves_cursor_right() {
        trace();

        let mut editing = Editing::blank(DEFAULT_SECTOR_SEAM_SPACING, 1);

        let edits = editing.handle([text("+")]);

        assert_eq!(edits, [Edit::Wrote]);
        assert_eq!(editing.rows(), ["+       "]);
        assert_eq!(editing.region().cursor(), editing.at(1, 0));
    }

    #[test]
    fn test_delete_clears_cell_and_moves_cursor_left() {
        trace();

        let mut editing = Editing::written(&["++      "]);
        editing.select(1, 0);

        let edits = editing.handle([InputEvent::KeyPressed(InputKey::Backspace)]);

        assert_eq!(edits, [Edit::Wrote]);
        assert_eq!(editing.rows(), ["+       "]);
        assert_eq!(editing.region().cursor(), editing.at(0, 0));
    }

    ///
    /// `extend` keeps the anchor, `select` moves it with the Cursor, and a
    /// write collapses the Region onto the Cell it stepped to.
    ///
    #[test]
    fn extend_spans_a_region_that_select_and_a_write_collapse() {
        let mut editing = Editing::blank(6, 4);

        editing.select(1, 1);
        editing.extend(3, 2);
        let spanned = editing.region();
        assert_eq!((spanned.columns(), spanned.rows()), (1..4, 1..3));
        assert_eq!(spanned.cursor(), editing.at(3, 2));

        editing.select(4, 0);
        assert_eq!(editing.region(), Region::at(editing.grid, editing.at(4, 0)));

        editing.extend(0, 0);
        editing.handle([text("x")]);
        assert_eq!(editing.region(), Region::at(editing.grid, editing.at(1, 0)));
    }

    ///
    /// Shift with an arrow moves the Cursor and keeps the anchor, and passing
    /// the anchor flips the Region; a bare arrow collapses it and moves.
    ///
    #[test]
    fn shift_arrows_extend_the_region_and_a_bare_arrow_collapses_it() {
        let mut editing = Editing::blank(8, 6);
        editing.select(3, 3);

        editing.handle([
            InputEvent::Extend(Arrow::Right),
            InputEvent::Extend(Arrow::Right),
            InputEvent::Extend(Arrow::Down),
        ]);
        let region = editing.region();
        assert_eq!((region.columns(), region.rows()), (3..6, 3..5));
        assert_eq!(region.cursor(), editing.at(5, 4));

        editing.handle(vec![InputEvent::Extend(Arrow::Left); 4]);
        let flipped = editing.region();
        assert_eq!((flipped.columns(), flipped.rows()), (1..4, 3..5));
        assert_eq!(flipped.anchor(), editing.at(3, 3));

        editing.handle([InputEvent::KeyPressed(InputKey::ArrowUp)]);
        assert_eq!(editing.region(), Region::at(editing.grid, editing.at(1, 3)));
    }

    ///
    /// Tab moves the Cursor to the first Cell of the next Sector on its row,
    /// and stays put in the Grid's last Sector — cut short at the right edge
    /// when the column count is not a whole multiple of the spacing.
    ///
    #[test]
    fn tab_steps_to_the_next_sector_and_stays_in_the_last_one() {
        // 10 columns and spacing 3: sectors at 0-2, 3-5, 6-8, and a last
        // Sector cut short to the single Cell at column 9.
        let mut editing = Editing::blank(10, 1).with_sectors(3);
        let grid = editing.grid;
        editing.select(1, 0);
        let tab = || InputEvent::KeyPressed(InputKey::Tab);

        editing.handle([tab()]);
        assert_eq!(editing.region(), Region::at(grid, editing.at(3, 0)));

        editing.handle([tab()]);
        assert_eq!(editing.region(), Region::at(grid, editing.at(6, 0)));

        editing.handle([tab()]);
        assert_eq!(
            editing.region(),
            Region::at(grid, editing.at(9, 0)),
            "reached the cut-short last Sector"
        );

        editing.handle([tab()]);
        assert_eq!(
            editing.region(),
            Region::at(grid, editing.at(9, 0)),
            "the last Sector stays put"
        );
    }

    ///
    /// Shift Tab moves the Cursor to the first Cell of its own Sector, then,
    /// once there, to the previous Sector's first Cell, staying put at the
    /// Grid's first Sector.
    ///
    #[test]
    fn shift_tab_steps_to_the_sector_start_then_the_previous_sector() {
        let mut editing = Editing::blank(10, 1).with_sectors(3);
        let grid = editing.grid;
        editing.select(4, 0);

        editing.handle([InputEvent::PreviousSector]);
        assert_eq!(
            editing.region(),
            Region::at(grid, editing.at(3, 0)),
            "moved to its own Sector's first Cell"
        );

        editing.handle([InputEvent::PreviousSector]);
        assert_eq!(
            editing.region(),
            Region::at(grid, editing.at(0, 0)),
            "already there: moved to the previous Sector's first Cell"
        );

        editing.handle([InputEvent::PreviousSector]);
        assert_eq!(
            editing.region(),
            Region::at(grid, editing.at(0, 0)),
            "the first Sector's first Cell stays put"
        );
    }

    ///
    /// Tab and Shift Tab collapse a multi-Cell Region onto the Cursor's
    /// stepped Position, the way a plain arrow does.
    ///
    #[test]
    fn tab_and_shift_tab_collapse_a_multi_cell_region_to_the_cursor() {
        let mut editing = Editing::blank(10, 4).with_sectors(3);
        let grid = editing.grid;

        editing.select(1, 1);
        editing.extend(4, 3);
        assert!(
            !editing.region().is_one_cell(),
            "test setup spanned no Region"
        );

        editing.handle([InputEvent::KeyPressed(InputKey::Tab)]);
        assert_eq!(
            editing.region(),
            Region::at(grid, editing.at(6, 3)),
            "Tab did not collapse the Region to the Cursor's new Sector"
        );

        editing.select(1, 1);
        editing.extend(4, 3);
        editing.handle([InputEvent::PreviousSector]);
        assert_eq!(
            editing.region(),
            Region::at(grid, editing.at(3, 3)),
            "Shift Tab did not collapse the Region to the Cursor's new Sector"
        );
    }

    ///
    /// Command `A` spans the whole Grid and Escape collapses the Region onto
    /// the Cursor. Neither writes to the Source.
    ///
    #[test]
    fn select_all_spans_the_grid_and_collapse_returns_to_the_cursor() {
        let mut editing = Editing::blank(5, 4);
        editing.select(2, 1);
        let before = editing.source.snapshot();

        // Command A spans the Grid around the Cursor and leaves it where it
        // was, so nothing follows it and the next keystroke writes there.
        let edits = editing.handle([InputEvent::SelectAll]);
        let whole = editing.region();
        assert_eq!((whole.columns(), whole.rows()), (0..5, 0..4));
        assert_eq!(whole.cursor(), editing.at(2, 1));

        let edits = [edits, editing.handle([InputEvent::Collapse])].concat();
        assert_eq!(editing.region(), Region::at(editing.grid, editing.at(2, 1)));

        assert_eq!(edits, [], "a Region chord answered an edit");
        assert_eq!(
            editing.source.snapshot(),
            before,
            "a Region chord wrote to the Source"
        );
    }

    ///
    /// After command A, Shift with an arrow moves the Region's live corner,
    /// and the Cursor rejoins it there.
    ///
    #[test]
    fn a_shift_arrow_after_select_all_moves_the_live_corner() {
        let mut editing = Editing::blank(5, 4);
        editing.select(2, 1);

        editing.handle([InputEvent::SelectAll, InputEvent::Extend(Arrow::Left)]);

        let region = editing.region();
        assert_eq!((region.columns(), region.rows()), (0..4, 0..4));
        assert_eq!(region.cursor(), editing.at(3, 3));
    }

    ///
    /// Typing after command A writes at the Cursor, steps right, and
    /// collapses the Region.
    ///
    #[test]
    fn typing_after_select_all_writes_at_the_cursor() {
        let mut editing = Editing::blank(5, 2);
        editing.select(2, 1);

        editing.handle([InputEvent::SelectAll, text("x")]);

        assert_eq!(editing.rows(), ["     ", "  x  "]);
        assert_eq!(editing.region(), Region::at(editing.grid, editing.at(3, 1)));
    }

    #[test]
    fn backspace_and_delete_empty_a_region_larger_than_one_cell_and_keep_it() {
        for key in [InputKey::Backspace, InputKey::Delete] {
            let mut editing = Editing::written(&["abcd", "efgh", "ijkl"]);
            editing.select(2, 2);
            editing.extend(1, 0);

            let edits = editing.handle([InputEvent::KeyPressed(key)]);

            assert_eq!(edits, [Edit::Wrote], "{key:?}");
            assert_eq!(editing.rows(), ["a  d", "e  h", "i  l"], "{key:?}");
            assert_eq!(
                editing.region(),
                Region::span(editing.grid, editing.at(2, 2), editing.at(1, 0))
            );
        }
    }

    #[test]
    fn backspace_and_delete_on_one_cell_empty_it_and_step_left() {
        for key in [InputKey::Backspace, InputKey::Delete] {
            let mut editing = Editing::written(&["abcd"]);
            editing.select(2, 0);

            editing.handle([InputEvent::KeyPressed(key)]);

            assert_eq!(editing.rows(), ["ab d"], "{key:?}");
            assert_eq!(editing.region(), Region::at(editing.grid, editing.at(1, 0)));
        }
    }

    #[test]
    fn typing_in_a_region_writes_at_the_cursor_steps_right_and_collapses() {
        let mut editing = Editing::written(&["....", "...."]);
        editing.select(0, 0);
        editing.extend(2, 1);

        editing.handle([text("x")]);

        assert_eq!(editing.rows(), ["....", "..x."]);
        assert_eq!(editing.region(), Region::at(editing.grid, editing.at(3, 1)));
    }

    ///
    /// Command Enter arms a fill that the next character carries into every
    /// Cell of the Region. A keystroke on its own never fills, and any other
    /// event between the chord and the character disarms it.
    ///
    #[test]
    fn command_enter_then_a_character_fills_the_region() {
        let mut editing = Editing::written(&["....", "....", "...."]);
        editing.select(1, 0);
        editing.extend(2, 1);
        let region = editing.region();

        editing.handle([InputEvent::Fill]);
        editing.handle([text("#")]);

        assert_eq!(editing.rows(), [".##.", ".##.", "...."]);
        assert_eq!(editing.region(), region, "a fill keeps the Region");

        // Disarmed by an arrow between the chord and the character: the
        // character is typed rather than filled.
        editing.handle([InputEvent::Fill, InputEvent::Extend(Arrow::Down), text("*")]);
        assert_eq!(editing.rows(), [".##.", ".##.", "..*."]);

        // A plain keystroke with a Region never fills.
        editing.select(0, 0);
        editing.extend(3, 2);
        editing.handle([text("=")]);
        assert_eq!(editing.rows(), [".##.", ".##.", "..*="]);
    }

    ///
    /// Space and Rewind are Playback requests the Editor leaves to its caller,
    /// and still disarm a fill: the character after them is typed.
    ///
    #[test]
    fn space_and_rewind_disarm_a_fill_and_answer_no_edit() {
        for request in [InputEvent::KeyPressed(InputKey::Space), InputEvent::Rewind] {
            let mut editing = Editing::written(&["....", "...."]);
            editing.select(0, 0);
            editing.extend(1, 1);

            let edits = editing.handle([InputEvent::Fill, request.clone()]);
            assert_eq!(edits, [], "{request:?}");

            editing.handle([text("x")]);
            assert_eq!(editing.rows(), ["....", ".x.."], "{request:?}");
        }
    }

    ///
    /// A pointer that moves the Region between the chord and the character
    /// is an event like any other, so it disarms the fill: the character is
    /// typed at the Cursor, which steps right.
    ///
    #[test]
    fn selecting_or_extending_between_command_enter_and_a_character_disarms_the_fill() {
        let mut editing = Editing::written(&["....", "...."]);
        let grid = editing.grid;

        // A drag: the Region is spanned anew after the chord.
        editing.handle([InputEvent::Fill]);
        editing.select(0, 0);
        editing.extend(1, 1);
        editing.handle([text("x")]);
        assert_eq!(editing.rows(), ["....", ".x.."]);
        assert_eq!(editing.region(), Region::at(grid, editing.at(2, 1)));

        // A click: the Region collapses onto one Cell after the chord.
        editing.handle([InputEvent::Fill]);
        editing.select(2, 0);
        editing.handle([text("y")]);
        assert_eq!(editing.rows(), ["..y.", ".x.."]);
        assert_eq!(editing.region(), Region::at(grid, editing.at(3, 0)));

        // A Shift click: the Region grows from the anchor after the chord.
        editing.handle([InputEvent::Fill]);
        editing.extend(0, 0);
        editing.handle([text("z")]);
        assert_eq!(editing.rows(), ["z.y.", ".x.."]);
        assert_eq!(editing.region(), Region::at(grid, editing.at(1, 0)));
    }

    ///
    /// Keys another control took between the chord and the character are
    /// events the Source never sees, so its presenter disarms the fill for
    /// them and the character is typed at the Cursor.
    ///
    #[test]
    fn a_fill_disarmed_for_input_elsewhere_types_the_next_character() {
        let mut editing = Editing::written(&["....", "...."]);
        editing.select(0, 0);
        editing.extend(1, 1);

        editing.handle([InputEvent::Fill]);
        editing.editor.disarm_fill();
        editing.handle([text("x")]);

        assert_eq!(editing.rows(), ["....", ".x.."]);
        assert_eq!(editing.region(), Region::at(editing.grid, editing.at(2, 1)));
    }

    #[test]
    fn copy_puts_the_regions_rows_on_the_clipboard_with_trailing_spaces() {
        let mut editing = Editing::written(&["ab  ", "c d ", "    "]);
        editing.select(3, 2);
        editing.extend(0, 0);
        let before = editing.rows();

        let edits = editing.handle([InputEvent::Copy]);

        assert_eq!(edits, [Edit::Copied("ab  \nc d \n    ".to_owned())]);
        assert_eq!(editing.rows(), before, "a copy wrote to the Source");
        assert_eq!(
            editing.region(),
            Region::span(editing.grid, editing.at(3, 2), editing.at(0, 0))
        );
    }

    #[test]
    fn cut_copies_and_then_empties_the_region() {
        let mut editing = Editing::written(&["abcd", "efgh"]);
        editing.select(1, 0);
        editing.extend(2, 1);

        let edits = editing.handle([InputEvent::Cut]);

        assert_eq!(edits, [Edit::Cut("bc\nfg".to_owned())]);
        assert_eq!(editing.rows(), ["a  d", "e  h"]);
        assert_eq!(
            editing.region(),
            Region::span(editing.grid, editing.at(1, 0), editing.at(2, 1))
        );
    }

    ///
    /// A paste writes from the Region's top-left, spaces included, clips at
    /// the Grid's edges, and leaves the Region on the rectangle that landed.
    ///
    #[test]
    fn paste_writes_from_the_top_left_clipped_and_spans_what_landed() {
        let mut editing = Editing::written(&["....", "....", "...."]);
        editing.select(3, 2);
        editing.extend(2, 1);

        let edits = editing.handle([InputEvent::Paste("a b\nxyz\n123".to_owned())]);

        // Two columns and two rows of the three-by-three block fit.
        assert_eq!(edits, [Edit::Wrote]);
        assert_eq!(editing.rows(), ["....", "..a ", "..xy"]);
        let landed = editing.region();
        assert_eq!((landed.columns(), landed.rows()), (2..4, 1..3));
    }

    ///
    /// A character that cannot be a Cell lands as an empty Cell, `\r\n` is one
    /// row break, and a trailing break adds no row.
    ///
    #[test]
    fn a_pasted_character_that_cannot_be_a_cell_lands_empty() {
        let mut editing = Editing::written(&["....", "....", "...."]);
        editing.select(0, 0);

        editing.handle([InputEvent::Paste("aé\tb\r\ncd\r\n".to_owned())]);

        // The shorter row lands empty Cells to the width of what landed, so
        // the Region holds exactly the block that was pasted.
        assert_eq!(editing.rows(), ["a  b", "cd  ", "...."]);
        let landed = editing.region();
        assert_eq!((landed.columns(), landed.rows()), (0..4, 0..2));
        assert_eq!(landed.cursor(), editing.at(0, 0));
    }

    #[test]
    #[should_panic(expected = "the Source is on another Grid than the Editor's")]
    fn an_event_against_a_source_on_another_grid_is_refused() {
        let mut editing = Editing::blank(4, 1);
        let elsewhere = SourceCommander::new(Grid::with_shape(8, 2));

        editing.editor.handle(&text("x"), &elsewhere);
    }

    ///
    /// Text of more than one character, and a space, is not an edit: Space
    /// arrives as its own key, and a Cell holds one character.
    ///
    #[test]
    fn text_that_is_not_one_printable_character_writes_nothing() {
        let mut editing = Editing::written(&["ab.."]);

        let edits = editing.handle([text("xy"), text(" "), text("")]);

        assert_eq!(edits, []);
        assert_eq!(editing.rows(), ["ab.."]);
        assert_eq!(editing.region(), Region::at(editing.grid, editing.at(0, 0)));
    }
}
