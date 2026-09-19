//!
//! The per-Cell decision of how one Render Frame is drawn: background, border,
//! foreground, sector seams and the character shown.
//!
//! # No geometry
//!
//! Nothing here is a `Rect`, and nothing here reads a viewport's geometry.
//! Where a Cell *is* on screen is `GridViewport`'s arithmetic and is asserted
//! there; a `Rect` in this layer would make every assertion about a colour
//! acquire a viewport before it could be written. Sector Seam strengths are
//! decided here from the Frame's spacing answer; `sector_line` turns a strength into a colour as a pure reading of
//! the palette, while the stroke *widths* the same seams are drawn with scale
//! with the Cell side and so belong to the step that has one.
//!
//! A [`VisiblePositions`] does cross into this layer, and it is that rule kept
//! rather than an exception to it. It is two `Range<usize>` over the Grid's own
//! column and row numbers, and the clip `Rect` they were resolved from is
//! consumed in `grid_viewport` and never arrives here. Naming an index range
//! costs an assertion no viewport, which is the whole of what the rule is for.
//! [`BackgroundRun`] already answered columns rather than a rectangle for
//! exactly this reason.
//!
//! A [`FramePaint`] pairs a Render Frame with those ranges at one seam:
//! [`Paint::derive`] takes the pair, not the two values separately.
//!
//! # Only the Positions the console draws
//!
//! A Paint covers the Positions `GridViewport::visible_positions` answers and
//! no others, so a console showing a tenth of a Grid pays for a tenth of it:
//! neither the per-Cell call to `cell_visuals_with_cursor_colour` nor the `Vec` holding its
//! answers follows the Source's size. Everything the derivation reads is local
//! to its own Cell, which makes the cull a saving rather than a change of answer.
//!
//! # Why the Cells are flat
//!
//! A [`Paint`] holds one `Vec<CellPaint>` in row-major order over the drawn
//! Positions. `Grid::index` is deliberately not that arithmetic: it addresses a
//! Cell of the whole Grid, and this `Vec` holds a sub-rectangle of one. The
//! Render Frame is flat too, over the whole Grid; the difference is only which
//! Positions each covers.
//!

use std::collections::HashMap;
use std::ops::Range;

use egui::Color32;

use orcvs::{
    grid::{Grid, Position},
    render_frame::RenderFrame,
    source::Claim,
};

use crate::{
    marks::{sector_left_strength, sector_top_strength},
    source_paint::SourcePaintSettings,
    style::{PALETTE, cell_visuals_with_cursor_colour, sector_line},
};

pub use crate::grid_viewport::VisiblePositions;

///
/// What one Cell of a Render Frame is drawn as.
///
/// Flat. The background is already decided at `cell_visuals_with_cursor_colour`: `None` means the
/// panel behind the Grid has painted the Source colour, and `Some` means this
/// Cell needs a fill of its own.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CellPaint {
    pub background: Option<Color32>,
    pub border: Color32,
    pub foreground: Color32,
    pub sector_left: Option<Color32>,
    pub sector_top: Option<Color32>,
    pub character: char,
}

///
/// One run of consecutive Cells of a single row that share one background
/// colour: the colour, the row, and the columns it covers.
///
/// Columns rather than a rectangle. Turning a run into the rectangle that
/// fills it is the viewport's arithmetic, and putting a `Rect` here would make
/// every assertion about coalescing acquire a viewport first.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundRun {
    pub colour: Color32,
    pub row: usize,
    pub columns: Range<usize>,
}

///
/// A Render Frame and the Positions the console draws from it, paired at one
/// seam before [`Paint::derive`].
///
/// Minted only through [`Self::new`], [`Self::range`], or [`Self::whole`], which
/// refuse a [`VisiblePositions`] minted for another Grid.
///
pub struct FramePaint<'a> {
    frame: &'a RenderFrame,
    drawn: VisiblePositions,
}

impl<'a> FramePaint<'a> {
    ///
    /// Pairs `frame` with `drawn` after refusing a range minted for another Grid.
    ///
    pub fn new(frame: &'a RenderFrame, drawn: VisiblePositions) -> Self {
        if let Some(identity) = drawn.grid() {
            assert!(
                frame.grid().owns_identity(identity),
                "VisiblePositions belong to another Grid"
            );
        }

        Self { frame, drawn }
    }

    ///
    /// Pairs `frame` with a column and row range clamped to its Grid.
    ///
    pub fn range(frame: &'a RenderFrame, columns: Range<usize>, rows: Range<usize>) -> Self {
        Self::new(
            frame,
            VisiblePositions::for_grid(frame.grid(), columns, rows),
        )
    }

    ///
    /// Pairs `frame` with every Position its Grid holds.
    ///
    pub fn whole(frame: &'a RenderFrame) -> Self {
        let grid = frame.grid();

        Self::new(
            frame,
            VisiblePositions::for_grid(grid, 0..grid.columns(), 0..grid.rows()),
        )
    }
}

///
/// Whether any Cell of `claim`'s own slot (`claim.cells`) holds written
/// content.
///
/// `Claim { cells, token, atom }` cannot answer Pending versus Invalid on its
/// own — both cover `atom: None` alike — so this reads the Render Frame's
/// own Cell contents over the claim's range, converting each raw index
/// through the Frame's Grid the way `orcvs::source::Span::positions` already
/// does (ADR 0044). A slot with any written Cell is Invalid; one that is
/// entirely blank is Pending — `cell_visuals_with_cursor_colour` is where
/// that distinction is spent.
///
fn slot_written(frame: &RenderFrame, claim: &Claim) -> bool {
    let grid = frame.grid();

    claim.cells.clone().any(|index| {
        grid.cell_index(index)
            .is_some_and(|cell_index| frame.at(grid.position_at(cell_index)).content().is_some())
    })
}

///
/// How one Render Frame is drawn, Cell by Cell.
///
/// Derived from a Render Frame and nothing else — no running Orcvs, no
/// `egui::Context`. That is what makes a question about what a Cell looks like
/// answerable without the harness a font atlas needs.
///
#[derive(Clone, Debug)]
pub struct Paint {
    grid: Grid,
    /// The Positions this Paint covers: what the console draws, and one Cell
    /// more in every direction the Grid has one.
    drawn: VisiblePositions,
    cursor: Option<Position>,
    /// Whether the Region spans more than one Cell. The lasso outlines it
    /// then, and the Cursor's Cell is framed by nothing of its own.
    region_spans: bool,
    cells: Vec<CellPaint>,
}

impl Paint {
    ///
    /// Reads a paired Render Frame and drawn range and answers what each Cell
    /// is drawn as.
    ///
    /// `cell_visuals_with_cursor_colour` is called once per drawn Cell and is
    /// unchanged: this decides what to do with its answer, not what the
    /// answer is. It reads the claim on the Cell (`RenderCell::claim`),
    /// whether that claim's slot holds written content — `slot_written`,
    /// below, answers the latter once per claim rather than once per Cell.
    ///
    /// The range is the console's decision, not this layer's. It comes from
    /// `GridViewport::visible_positions` already clamped to the Grid, which is
    /// why the two slices below are indexed rather than reached through `get`:
    /// a range that outran this Render Frame would mean a Frame that is not
    /// the rectangle its own Grid says it is, and answering an empty slice
    /// there would turn that into rows that silently go unpainted. Pairing is
    /// checked in [`FramePaint::new`].
    ///
    pub fn derive(input: FramePaint<'_>) -> Self {
        Self::derive_with_colours(
            input,
            Some(PALETTE.selection_fill),
            crate::cursor_effects::DEFAULT_REGION_COLOUR,
            None,
            SourcePaintSettings::default(),
        )
    }

    ///
    /// [`Self::derive`] with the Theme's colours. `cursor_colour` fills the
    /// Cursor's Cell. While a Region spans more than one Cell,
    /// `region_colour` fills the rest of it, and `region_cursor_colour` fills
    /// the Cursor's Cell independently — or, when `None`, `cursor_colour`
    /// still does. Only the fill carries over: the Cursor's Cell in such a
    /// Region takes the grid line and its sector seams, as every other Cell
    /// of the Region does.
    ///
    pub fn derive_with_colours(
        input: FramePaint<'_>,
        cursor_colour: Option<Color32>,
        region_colour: Color32,
        region_cursor_colour: Option<Color32>,
        source_paint: SourcePaintSettings,
    ) -> Self {
        let FramePaint { frame, drawn } = input;
        let grid = frame.grid();
        // The Cursor is the Position the Render Frame was derived for. A Paint
        // covers a viewport, so `None` here means that Position is outside the
        // drawn range — not that the Frame selected nothing.
        let frame_cursor = frame.cursor();
        let cursor_visible = frame.cursor_visible();
        let sector_seam_spacing = frame.sector_seam_spacing().cells();
        let cursor = (drawn.columns.contains(&frame_cursor.x())
            && drawn.rows.contains(&frame_cursor.y()))
        .then_some(frame_cursor);
        // The Region's extent, read once. A Region of one Cell is the Cursor's
        // own Cell and is tinted nowhere, so it answers empty ranges rather
        // than a per-Cell question the answer to which is always no.
        let region = frame.region();
        let region_spans = !region.is_one_cell();
        let (region_columns, region_rows) = if region_spans {
            (region.columns(), region.rows())
        } else {
            (0..0, 0..0)
        };
        // Sized up front. The drawn count is known exactly, so collecting into
        // a `Vec` need not grow by doubling across the walk.
        let mut cells = Vec::with_capacity(drawn.count());
        // `slot_written` walks every Cell of a claim's slot, so a claim
        // spanning many Cells — a whole-row Comment among them — is answered
        // once here rather than once per Cell it covers. Every Cell of one
        // claim shares one `Arc` (ADR 0044), so the claim's own address is
        // the cache key.
        let mut written_cache: HashMap<*const Claim, bool> = HashMap::new();
        for row in drawn.rows.clone() {
            for column in drawn.columns.clone() {
                let position = grid
                    .position(column, row)
                    .expect("a drawn Position is one the visible range clamped to this Grid");
                let cell = frame.at(position);
                // While a Region spans more than one Cell the lasso around it
                // is the Cursor's border, so the Cursor's Cell keeps an
                // ordinary one. Its fill is its own: the Region's Cursor
                // colour, or the Cursor's colour when that is unset.
                let is_cursor = position == frame_cursor;
                let selected = is_cursor && !region_spans;
                let claim = cell.claim();
                let written = claim.is_some_and(|claim| {
                    *written_cache
                        .entry(claim as *const Claim)
                        .or_insert_with(|| slot_written(frame, claim))
                });
                let visuals = cell_visuals_with_cursor_colour(
                    claim,
                    written,
                    selected,
                    selected && cursor_visible,
                    cursor_colour,
                    source_paint,
                );
                let in_region = region_columns.contains(&column) && region_rows.contains(&row);
                let background = if is_cursor && region_spans {
                    region_cursor_colour.or(cursor_colour)
                } else {
                    visuals.background.or(in_region.then_some(region_colour))
                };

                cells.push(CellPaint {
                    background,
                    border: visuals.border,
                    foreground: visuals.foreground,
                    // A sector seam is suppressed on the Cursor's Cell, so the
                    // Cursor's frame is never crossed by one. While a Region
                    // spans more than one Cell the lasso is the frame and the
                    // Cursor's Cell is not `selected`, so it keeps its seams
                    // as every other Cell of the Region does. It is decided here rather
                    // than left to the step that draws it: in the loop this
                    // replaced the rule was structural — the selected Cell took
                    // a branch the seams were not in — and a rule that survives
                    // only as a branch shape is a rule the next reader has to
                    // rediscover.
                    //
                    // `sector_line` is pure, so the strength becomes a colour
                    // here. The stroke widths are geometry and stay out of this
                    // layer.
                    sector_left: (!selected)
                        .then(|| {
                            sector_left_strength(position, sector_seam_spacing).map(sector_line)
                        })
                        .flatten(),
                    sector_top: (!selected)
                        .then(|| {
                            sector_top_strength(position, sector_seam_spacing).map(sector_line)
                        })
                        .flatten(),
                    // A Cell's own content when it has one; the space
                    // otherwise, which `place_glyphs` (`console.rs`) draws no
                    // Glyph for. `syntax-highlighting/03` retired the blank
                    // spelling table that used to stand a placeholder letter
                    // in here: an empty claimed operand Cell now answers the
                    // same space an empty unclaimed one always has, and reads
                    // as its Token tint alone (`cell_visuals_with_cursor_
                    // colour`'s `fill_tint_colour`), never as a spelled
                    // letter.
                    character: cell.content().unwrap_or(' '),
                });
            }
        }

        Self {
            grid,
            drawn: drawn.clone(),
            cursor,
            region_spans,
            cells,
        }
    }

    ///
    /// What the Cell at `position` is drawn as.
    ///
    /// `position` has to be one of the drawn Positions. One this Paint does
    /// not cover is refused rather than indexed: the offset is a subtraction
    /// from the drawn range's own corner, so a Position left of or above that
    /// corner wraps, and a wrapped offset can land back inside `cells` and
    /// answer some other Cell's paint in silence.
    ///
    /// Callers that already hold a Position — the colour tests among them —
    /// look up here. Production walks go through [`Self::cells`] or
    /// [`Self::background_runs`] so they never pay this offset, and this
    /// lookup is compiled only for tests so it is not a shipped seam.
    ///
    #[cfg(test)]
    pub(crate) fn at(&self, position: Position) -> &CellPaint {
        let offset = self
            .offset(position)
            .unwrap_or_else(|| panic!("{position:?} is not one of the drawn Positions"));

        &self.cells[offset]
    }

    ///
    /// Where `position`'s paint sits in `cells`, or `None` when this Paint
    /// does not cover it.
    ///
    /// Row-major within the drawn range rather than `Grid::index`, which
    /// addresses a Cell of the whole Grid where this `Vec` holds one
    /// sub-rectangle of it.
    ///
    #[cfg(test)]
    fn offset(&self, position: Position) -> Option<usize> {
        let column = position.x().checked_sub(self.drawn.columns.start)?;
        let row = position.y().checked_sub(self.drawn.rows.start)?;

        (column < self.drawn.columns.len() && row < self.drawn.rows.len())
            .then(|| row * self.drawn.columns.len() + column)
    }

    ///
    /// The drawn Positions in row order, each with what it is drawn as.
    ///
    /// The one way to walk a Paint. It zips the Positions against the `Vec`,
    /// so the walk the shape step makes on every Render Frame does no offset
    /// arithmetic at all, and the row-major agreement between Positions and
    /// Cells is kept here rather than restated at each caller.
    ///
    pub(crate) fn cells(&self) -> impl Iterator<Item = (Position, &CellPaint)> {
        self.positions().zip(&self.cells)
    }

    ///
    /// How many Cells this Paint covers, which is the drawn count and never
    /// the Grid's.
    ///
    pub(crate) fn count(&self) -> usize {
        self.cells.len()
    }

    ///
    /// The drawn Positions in row order.
    ///
    fn positions(&self) -> impl Iterator<Item = Position> {
        self.drawn.rows.clone().flat_map(|row| self.row(row))
    }

    ///
    /// The drawn Positions of one row, left to right.
    ///
    /// `Grid::position` is the only way to mint one and it refuses a pair the
    /// Grid does not own, which `visible_positions` has already clamped
    /// against — so the `expect` restates that clamp rather than guarding a
    /// case that arises.
    ///
    fn row(&self, row: usize) -> impl Iterator<Item = Position> {
        let grid = self.grid;

        self.drawn.columns.clone().map(move |column| {
            grid.position(column, row)
                .expect("a drawn Position is one the visible range clamped to this Grid")
        })
    }

    ///
    /// Where the Cursor is, when this Paint covers it.
    ///
    /// One Position for the whole Paint rather than a flag on every Cell:
    /// [`RenderFrame::cursor`] answers the Position `derive` was given, so
    /// exactly one exists, and a per-Cell bool would re-open a state the layer
    /// below has closed. `None` says the Cursor is outside the viewport this
    /// Paint covers, not that the Render Frame selected nothing.
    ///
    pub(crate) fn cursor(&self) -> Option<Position> {
        self.cursor
    }

    ///
    /// Whether the Region spans more than one Cell, when the lasso around it
    /// replaces the Cursor's own Cell border.
    ///
    pub(crate) fn region_spans(&self) -> bool {
        self.region_spans
    }

    ///
    /// The consecutive Cells of each row that share one background, coalesced.
    ///
    /// Derived on demand rather than stored: the per-Cell backgrounds are the
    /// one truth, and a stored copy of their runs is a second one to keep
    /// agreeing with them.
    ///
    /// A run ends at a Cell wanting a different colour, at a Cell wanting no
    /// background at all, and at the end of its row — Cells are consecutive
    /// within a row, and the row below starts somewhere else entirely.
    ///
    /// "Its row" is the drawn part of a row, so a run opens at the first drawn
    /// Cell and is flushed at the last. Coalescing the whole row and clipping
    /// the answer would paint the same rectangles — `cell_rect` is a function
    /// of one column index, so both hand the shape step the same two endpoints
    /// — but it would walk Cells the console does not draw, which is the cost
    /// this covers only the drawn range to avoid.
    ///
    pub fn background_runs(&self) -> Vec<BackgroundRun> {
        let mut runs = Vec::new();
        let width = self.drawn.columns.len();
        let first_column = self.drawn.columns.start;

        // Row-major slices of `cells`: the fold already owns the drawn ranges,
        // and re-deriving each Cell's offset through a Position mint would pay
        // a lookup on every Cell of every frame.
        for (row_offset, row) in self.drawn.rows.clone().enumerate() {
            let start = row_offset * width;
            let row_cells = &self.cells[start..start + width];
            // The run so far: its colour, and the columns it covers.
            let mut open: Option<(Color32, Range<usize>)> = None;

            for (column_offset, cell) in row_cells.iter().enumerate() {
                let column = first_column + column_offset;
                let background = cell.background;
                open = match (open, background) {
                    (Some((colour, columns)), Some(background)) if colour == background => {
                        Some((colour, columns.start..column + 1))
                    }
                    (finished, background) => {
                        if let Some((colour, columns)) = finished {
                            runs.push(BackgroundRun {
                                colour,
                                row,
                                columns,
                            });
                        }
                        background.map(|colour| (colour, column..column + 1))
                    }
                };
            }

            if let Some((colour, columns)) = open {
                runs.push(BackgroundRun {
                    colour,
                    row,
                    columns,
                });
            }
        }

        runs
    }
}

#[cfg(test)]
mod tests {
    use super::{BackgroundRun, CellPaint, FramePaint, Paint, slot_written};
    use crate::grid_viewport::VisiblePositions;
    use crate::marks::{sector_left_strength, sector_top_strength};
    use crate::source_paint::{DEFAULT_ORDINARY, DEFAULT_SOURCE_BACKGROUND, SourcePaintSettings};
    use crate::style::{PALETTE, cell_visuals_with_cursor_colour, sector_line};
    use egui::Color32;
    use orcvs::source::Token;
    use orcvs::{app::Orcvs, grid::Grid, render_frame::RenderFrame};
    use std::ops::Range;

    fn running_orcvs(cols: usize, rows: usize) -> Orcvs {
        Orcvs::new(cols, rows).expect("the test runtime")
    }

    ///
    /// A Paint over every Position of a Render Frame's Grid.
    ///
    /// This is the range a fitted console hands `derive` —
    /// `GridViewport::visible_positions` answers exactly it when the whole
    /// Grid is on screen, which `console.rs` asserts through a console pass.
    /// Stated directly here so that a test about what colour a Cell is still
    /// needs no viewport to say "all of them".
    ///
    fn whole(frame: &RenderFrame) -> Paint {
        Paint::derive(FramePaint::whole(frame))
    }

    ///
    /// What `cell_visuals_with_cursor_colour` answers for `cell`, reading its
    /// claim and its slot's own written fact straight from `frame` and
    /// `cell` — the same two inputs
    /// `Paint::derive_with_colours` reads, so a test comparing against this
    /// needs no `SourcePaintSettings::default()`-only shim.
    ///
    fn expected_visuals(
        frame: &RenderFrame,
        cell: &orcvs::render_frame::RenderCell,
        selected: bool,
        cursor_visible: bool,
        cursor_colour: Option<Color32>,
        source_paint: SourcePaintSettings,
    ) -> crate::style::CellVisuals {
        let claim = cell.claim();
        let written = claim.is_some_and(|claim| slot_written(frame, claim));
        cell_visuals_with_cursor_colour(
            claim,
            written,
            selected,
            cursor_visible,
            cursor_colour,
            source_paint,
        )
    }

    ///
    /// Every Position answers the Cell the Grid indexes, not its neighbour.
    ///
    /// The Cursor grades its selected Cell and written Cells carry Glyph
    /// colours of their own, so an answer off by one Cell —
    /// in either axis — differs from the answer asked for.
    ///
    #[tokio::test]
    async fn a_cell_is_answered_at_the_position_the_grid_indexes() {
        let mut orcvs = running_orcvs(6, 4);
        for (x, character) in "#a#".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x + 1, 2).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.grid().position(4, 1).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let mut borders = std::collections::BTreeSet::new();
        let mut foregrounds = std::collections::BTreeSet::new();
        let cursor = frame.cursor();

        for cell in frame.cells() {
            let position = cell.position();
            let selected = position == cursor;
            let visuals = expected_visuals(
                &frame,
                cell,
                selected,
                selected && frame.cursor_visible(),
                Some(PALETTE.selection_fill),
                SourcePaintSettings::default(),
            );
            let painted = paint.at(position);

            assert_eq!(
                painted.border,
                visuals.border,
                "the border at {:?}",
                cell.position()
            );
            assert_eq!(
                painted.foreground,
                visuals.foreground,
                "the foreground at {:?}",
                cell.position()
            );
            borders.insert(painted.border.to_array());
            foregrounds.insert(painted.foreground.to_array());
        }

        // A Grid painted one colour throughout would pass the loop above with
        // every answer off by any number of Cells.
        assert!(borders.len() > 1, "every border was the same colour");
        assert!(
            foregrounds.len() > 1,
            "every foreground was the same colour"
        );
    }

    ///
    /// The Cursor is one Position the Paint answers, not a flag to be found by
    /// searching the Cells.
    ///
    #[tokio::test]
    async fn the_cursor_is_the_selected_position() {
        let mut orcvs = running_orcvs(9, 5);
        let selected = orcvs.grid().position(6, 3).expect("inside the grid");
        orcvs.select(selected);

        let paint = whole(&orcvs.render_frame());

        assert_eq!(paint.cursor(), Some(selected));
    }

    ///
    /// A Region larger than one Cell fills every Cell it covers but the
    /// Cursor's with the Theme's Region colour, and gives every Cell of it,
    /// the Cursor's included, the grid line: the lasso around the Region is
    /// the Cursor's border then. A Region of one Cell fills none.
    ///
    /// The Cursor's Cell takes its own fill. Unset, it is the Cursor's cell
    /// colour, as with no Region; set, it is independent of both.
    ///
    #[tokio::test]
    async fn a_region_larger_than_one_cell_is_filled_around_an_independently_coloured_cursor() {
        let mut orcvs = running_orcvs(6, 4);
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        let fill = Color32::from_rgba_unmultiplied(255, 255, 255, 43);
        let cursor_fill = Color32::from_rgb(1, 2, 3);
        let own = Color32::from_rgba_unmultiplied(0, 0, 255, 200);

        let painted = |orcvs: &Orcvs, cursor: Option<Color32>, region_cursor| {
            Paint::derive_with_colours(
                FramePaint::whole(&orcvs.render_frame()),
                cursor,
                fill,
                region_cursor,
                SourcePaintSettings::default(),
            )
        };

        orcvs.select(at(1, 1));
        let collapsed = painted(&orcvs, None, Some(own));
        assert!(
            collapsed.cells().all(|(_, cell)| cell.background.is_none()),
            "a Region of one Cell was tinted"
        );
        assert!(!collapsed.region_spans());

        orcvs.extend(at(3, 2));
        for (cursor, region_cursor, expected) in [
            (None, None, None),
            (Some(cursor_fill), None, Some(cursor_fill)),
            (Some(cursor_fill), Some(own), Some(own)),
        ] {
            let spanned = painted(&orcvs, cursor, region_cursor);
            assert!(spanned.region_spans());
            for (position, cell) in spanned.cells() {
                let inside = (1..4).contains(&position.x()) && (1..3).contains(&position.y());
                let wanted = if position == at(3, 2) {
                    expected
                } else {
                    inside.then_some(fill)
                };
                assert_eq!(cell.background, wanted, "the background at {position:?}");
                assert_eq!(cell.border, PALETTE.grid_line, "the border at {position:?}");
            }
        }
        assert_eq!(
            painted(&orcvs, None, None).background_runs(),
            vec![run(fill, 1, 1..4), run(fill, 2, 1..3)]
        );
    }

    ///
    /// Sector seams stand where Paint derives them, and nowhere on the Cursor's
    /// own Cell while it is framed on its own.
    ///
    /// The suppression is the derive's, so no later step learns the rule. The
    /// Cursor is put on a Cell that wants a seam — the corner of a sector,
    /// where the seam is at full strength — because a Cursor parked where no
    /// seam was wanted would prove nothing.
    ///
    /// Everywhere else the colour is `sector_line`'s answer for the strength
    /// Paint computed. `sector_line` is pure, so the derive resolves it here;
    /// the stroke widths are geometry and are not in this layer.
    ///
    #[tokio::test]
    async fn seams_stand_where_paint_asks_and_never_on_the_cursor() {
        let mut orcvs = running_orcvs(24, 24);
        // A sector corner at the default Sector Seam spacing of eight.
        let corner = orcvs.grid().position(8, 8).expect("inside the grid");
        orcvs.select(corner);

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let spacing = frame.sector_seam_spacing().cells();
        let cursor = frame.cursor();
        let mut seams = 0;

        for cell in frame.cells() {
            let position = cell.position();
            let painted = paint.at(position);
            let left = sector_left_strength(position, spacing);
            let top = sector_top_strength(position, spacing);

            if position == cursor {
                assert!(
                    left.is_some() && top.is_some(),
                    "the Cursor must sit where seams are wanted for their absence to mean anything"
                );
                assert_eq!(painted.sector_left, None);
                assert_eq!(painted.sector_top, None);
                continue;
            }

            assert_eq!(
                painted.sector_left,
                left.map(sector_line),
                "the left seam at {position:?}"
            );
            assert_eq!(
                painted.sector_top,
                top.map(sector_line),
                "the top seam at {position:?}"
            );
            seams += usize::from(painted.sector_left.is_some());
            seams += usize::from(painted.sector_top.is_some());
        }

        assert!(seams > 0, "no Cell but the Cursor's was asked for a seam");
    }

    ///
    /// While a Region spans more than one Cell the lasso is the Cursor's frame,
    /// so the Cursor's Cell keeps the sector seams it would carry as any other
    /// Cell of the Region.
    ///
    #[tokio::test]
    async fn a_cursor_inside_a_spanning_region_keeps_its_seams() {
        let mut orcvs = running_orcvs(24, 24);
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        // A sector corner at the default Sector Seam spacing of eight.
        orcvs.select(at(4, 4));
        orcvs.extend(at(8, 8));

        let frame = orcvs.render_frame();
        let spacing = frame.sector_seam_spacing().cells();
        let painted = *whole(&frame).at(at(8, 8));

        let left = sector_left_strength(at(8, 8), spacing);
        let top = sector_top_strength(at(8, 8), spacing);
        assert!(
            left.is_some() && top.is_some(),
            "the Cursor is off a sector corner"
        );
        assert_eq!(painted.sector_left, left.map(sector_line));
        assert_eq!(painted.sector_top, top.map(sector_line));
    }

    ///
    /// Each Cell shows exactly its own content, or the space when it holds
    /// none — never a placeholder letter standing in for its Token.
    ///
    /// `syntax-highlighting/03` retired the blank spelling table
    /// `each_cell_shows_the_character_the_table_answers` used to pin: what a
    /// Cell shows is answered from the Render Frame alone, with no Token
    /// lookup at all. The Grid carries an Addition, whose claim reaches past
    /// the two Cells it is spelled in and leaves classified but empty operand
    /// Cells behind it — this is the Cell this test is about, and it answers
    /// the space like any other empty Cell, not a letter the deleted table
    /// used to spell for it.
    ///
    #[tokio::test]
    async fn every_cell_shows_only_its_own_content_or_the_space() {
        let mut orcvs = running_orcvs(8, 8);
        for (x, character) in ".+".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 2).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.grid().position(5, 5).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);

        for cell in frame.cells() {
            let shown = paint.at(cell.position()).character;

            assert_eq!(
                shown,
                cell.content().unwrap_or(' '),
                "the character at {:?}",
                cell.position()
            );
        }
    }

    ///
    /// An empty claimed operand Cell of every Token a signature can declare —
    /// Number, Note, Atom, Sequence — shows no character at all: the tint
    /// `style::fill_tint_colour` paints is the whole of what marks it as
    /// Pending, and the space it shows is the same one an empty unclaimed
    /// Cell always has. `Token::Char` is not one of the Tokens exercised
    /// here: `style::fill_tint_colour`'s own doc explains why no operand ever
    /// declares it, so there is no empty *claimed* Char Cell to write this
    /// test against — a Leftover Char is never empty, since content is what
    /// makes it Char at all.
    ///
    /// Number and Note come from an Addition (`.+`, two Number operands) and
    /// a `.v` (one Note operand) left unfilled. Atom and Sequence have no
    /// Function whose *first* operand declares them without also demanding a
    /// nested Function earlier in the row, so they are asserted against
    /// `cell_visuals_with_cursor_colour` directly in `style::tests`, which this test does not
    /// repeat.
    ///
    #[tokio::test]
    async fn an_empty_claimed_number_or_note_operand_shows_tint_and_no_character() {
        let mut orcvs = running_orcvs(8, 2);
        for (x, character) in ".+".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        for (x, character) in ".v".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 1).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.grid().position(7, 1).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();

        // The Addition's two Number operands, columns 2-5 of row 0.
        for x in 2..6 {
            let position = orcvs.grid().position(x, 0).expect("inside the grid");
            let cell = frame.at(position);
            assert_eq!(cell.content(), None, "operand Cell {x} was not empty");
            assert_eq!(cell.claim().map(|claim| claim.token), Some(Token::Number));
            let painted = paint.at(position);
            assert_eq!(painted.character, ' ', "operand Cell {x} spelled a letter");
            assert_eq!(
                painted.background,
                expected_visuals(&frame, cell, false, false, None, source_paint).background,
                "operand Cell {x} did not carry the Number tint"
            );
        }

        // `.v`'s one Note operand, columns 2-3 of row 1.
        for x in 2..4 {
            let position = orcvs.grid().position(x, 1).expect("inside the grid");
            let cell = frame.at(position);
            assert_eq!(cell.content(), None, "operand Cell {x} was not empty");
            assert_eq!(cell.claim().map(|claim| claim.token), Some(Token::Note));
            let painted = paint.at(position);
            assert_eq!(painted.character, ' ', "operand Cell {x} spelled a letter");
            assert_eq!(
                painted.background,
                expected_visuals(&frame, cell, false, false, None, source_paint).background,
                "operand Cell {x} did not carry the Note tint"
            );
        }
    }

    ///
    /// A slot cut off by the end of its row carries its declared Token for
    /// every one of its Cells the Grid holds, the same as any other claimed
    /// operand Cell — no per-Cell classifier beside the Language Map is
    /// needed, because `LanguageMap::token_at` already answers it: `lang::
    /// Parser::take_token`'s error path still records the Cells the row's
    /// tail actually held (`PositionedEntry::cells`), even though the
    /// operand itself cannot bind, so the truncated Cell reaches this layer
    /// exactly like an ordinary Pending Operand.
    ///
    /// `.+01` written into a 5-wide Grid is an Add whose second Number
    /// operand needs columns 4-5, and column 4 is the last column the Grid
    /// has: one Cell of the slot exists, and it is empty (a space at the row
    /// edge, left at its default rather than written), so this is the
    /// Pending case rather than an Invalid one.
    ///
    #[tokio::test]
    async fn a_row_truncated_operand_cell_carries_its_declared_token_tint_and_no_character() {
        let mut orcvs = running_orcvs(5, 1);
        for (x, character) in ".+01".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();
        let position = orcvs.grid().position(4, 0).expect("inside the grid");
        let cell = frame.at(position);

        assert_eq!(cell.content(), None, "the truncated Cell was not empty");
        assert_eq!(
            cell.claim().map(|claim| claim.token),
            Some(Token::Number),
            "the truncated Cell did not carry the operand's declared Token"
        );
        let painted = paint.at(position);
        assert_eq!(
            painted.character, ' ',
            "the truncated Cell spelled a letter"
        );
        assert_eq!(
            painted.background,
            expected_visuals(&frame, cell, false, false, None, source_paint).background,
            "the truncated Cell did not carry the Number tint"
        );
    }

    ///
    /// A Paint over `rows` of backgrounds and nothing else that matters.
    ///
    /// `background_runs` folds over the per-Cell backgrounds alone, so the
    /// rest of each Cell is filler: stating it from a Render Frame would make
    /// every case below an exercise in arranging production state around it.
    ///
    fn paint_of(rows: &[&[Option<Color32>]]) -> Paint {
        let grid = Grid::new(rows[0].len(), rows.len());
        assert!(
            rows.iter().all(|row| row.len() == rows[0].len()),
            "a Grid's rows are all one length"
        );

        Paint {
            grid,
            drawn: VisiblePositions::for_grid(grid, 0..grid.columns(), 0..grid.rows()),
            cursor: Some(grid.origin()),
            region_spans: false,
            cells: rows
                .iter()
                .flat_map(|row| row.iter())
                .map(|&background| CellPaint {
                    background,
                    border: PALETTE.grid_line,
                    foreground: DEFAULT_ORDINARY,
                    sector_left: None,
                    sector_top: None,
                    character: ' ',
                })
                .collect(),
        }
    }

    fn run(colour: Color32, row: usize, columns: Range<usize>) -> BackgroundRun {
        BackgroundRun {
            colour,
            row,
            columns,
        }
    }

    ///
    /// `syntax-highlighting/02`'s Fill tint, walked end to end from written
    /// Source through `Paint::derive` rather than through `cell_visuals_with_cursor_colour`
    /// alone: a nested Function's own Cells tint like its parent's, an
    /// Operand Cell tints with its declared Token's colour whether or not
    /// its slot binds, and adjacent tinted Cells of one colour — a Function
    /// and its nested Function here, and three Number operand entries there
    /// — coalesce into one `BackgroundRun` apiece rather than one per Cell or
    /// per Language Unit.
    ///
    /// `.+.x010203` is Add of a nested Multiply and a Number: `.+` and `.x`
    /// are each a Function's own two-Cell spelling (columns 0-1 and 2-3),
    /// and `01`, `02`, `03` are three Number operand entries (columns 4-5,
    /// 6-7, 8-9) — Multiply's two and Add's own. The Cursor is parked on the
    /// untouched row below so its own fill cannot stand in for a tint this
    /// test is about.
    ///
    #[tokio::test]
    async fn nested_function_and_operand_cells_tint_and_adjacent_same_colour_cells_merge_into_one_run()
     {
        let mut orcvs = running_orcvs(10, 2);
        for (x, character) in ".+.x010203".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.grid().position(0, 1).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();
        // The real bound claims this fixture already carries: `.+`'s own
        // Function claim (columns 0-1) and Multiply's first Number operand
        // (columns 4-5), rather than a claim built by hand — both are bound,
        // so `written` plays no part in either tint.
        let function_cell = frame.at(orcvs.grid().position(0, 0).expect("inside the grid"));
        let number_cell = frame.at(orcvs.grid().position(4, 0).expect("inside the grid"));
        let function_tint =
            expected_visuals(&frame, function_cell, false, false, None, source_paint).background;
        let number_tint =
            expected_visuals(&frame, number_cell, false, false, None, source_paint).background;
        assert!(function_tint.is_some() && number_tint.is_some());
        assert_ne!(function_tint, number_tint);

        for x in 0..4 {
            let position = orcvs.grid().position(x, 0).expect("inside the grid");
            assert_eq!(
                paint.at(position).background,
                function_tint,
                "Function Cell {x} was not tinted, nested included"
            );
        }
        for x in 4..10 {
            let position = orcvs.grid().position(x, 0).expect("inside the grid");
            assert_eq!(
                paint.at(position).background,
                number_tint,
                "Number Operand Cell {x} was not tinted"
            );
        }

        let row_zero_runs: Vec<_> = paint
            .background_runs()
            .into_iter()
            .filter(|run| run.row == 0)
            .collect();
        assert_eq!(
            row_zero_runs,
            vec![
                run(function_tint.expect("checked above"), 0, 0..4),
                run(number_tint.expect("checked above"), 0, 4..10),
            ],
            "the outer and nested Function merged into one run, and so did the three Number entries"
        );
    }

    #[test]
    fn a_run_ends_where_the_next_cell_wants_a_different_colour() {
        let first = PALETTE.selection_fill;
        let second = DEFAULT_SOURCE_BACKGROUND;
        let paint = paint_of(&[&[Some(first), Some(first), Some(second), Some(second)]]);

        assert_eq!(
            paint.background_runs(),
            vec![run(first, 0, 0..2), run(second, 0, 2..4)]
        );
    }

    #[test]
    fn a_run_ends_where_the_next_cell_wants_no_background() {
        let colour = PALETTE.selection_fill;
        let paint = paint_of(&[&[Some(colour), Some(colour), None, Some(colour)]]);

        assert_eq!(
            paint.background_runs(),
            vec![run(colour, 0, 0..2), run(colour, 0, 3..4)],
            "the gap parts two runs of one colour rather than joining them"
        );
    }

    ///
    /// A run ends at the end of its row, and the identical run in the row
    /// below is a run of its own.
    ///
    #[test]
    fn a_run_ends_at_the_end_of_its_row() {
        let colour = PALETTE.selection_fill;
        let paint = paint_of(&[&[Some(colour), Some(colour)], &[Some(colour), Some(colour)]]);

        assert_eq!(
            paint.background_runs(),
            vec![run(colour, 0, 0..2), run(colour, 1, 0..2)]
        );
    }

    ///
    /// One Cell wanting a background between two that want none is a run, not
    /// a Cell the fold forgets to flush.
    ///
    #[test]
    fn one_cell_wanting_a_background_alone_is_a_run_of_one() {
        let colour = PALETTE.selection_fill;
        let paint = paint_of(&[&[None, Some(colour), None]]);

        assert_eq!(paint.background_runs(), vec![run(colour, 0, 1..2)]);
    }

    #[test]
    fn a_row_wanting_no_background_contributes_no_run() {
        let paint = paint_of(&[&[None, None, None]]);

        assert_eq!(paint.background_runs(), Vec::new());
    }

    ///
    /// A Paint decides the Cells the console draws and no others, and decides
    /// each of them exactly as a Paint of the whole Grid would.
    ///
    /// This is the Cell-iteration half of the claim `source-grid-rendering/05`
    /// made — the cost of a Render Frame follows the viewport rather than the
    /// Source. `05` could only count it through a console pass, because the
    /// per-Cell decision had no surface of its own; here the `CellPaint`s are
    /// the count, with no `egui::Context` and no window.
    ///
    /// The second assertion is the half that matters for correctness. Nothing
    /// the derivation reads reaches past its own Cell, so a culled Paint has
    /// to answer what an unculled one answers even when the Cursor is outside
    /// the drawn range.
    ///
    #[tokio::test]
    async fn a_paint_decides_the_drawn_cells_and_answers_them_unchanged() {
        let mut orcvs = running_orcvs(40, 30);
        orcvs.select(orcvs.grid().position(9, 10).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let grid = frame.grid();
        let drawn = VisiblePositions::for_grid(grid, 12..20, 8..14);
        let culled = Paint::derive(FramePaint::new(&frame, drawn.clone()));
        let every = whole(&frame);

        assert_eq!(culled.count(), drawn.count());
        assert!(
            culled.count() * 10 < every.count(),
            "{} of {} Cells is not a cull worth asserting about",
            culled.count(),
            every.count()
        );

        let walked = culled.cells().count();
        assert_eq!(walked, culled.count(), "the walk and the count disagree");

        let mut graded = 0;
        for (position, cell) in culled.cells() {
            assert!(
                drawn.columns.contains(&position.x()) && drawn.rows.contains(&position.y()),
                "{position:?} is outside the range the Paint was asked for"
            );
            assert_eq!(cell, every.at(position), "the paint at {position:?}");
            graded += usize::from(cell.background.is_some());
        }

        assert_eq!(graded, 0, "Cursor effects belong to console geometry");
    }

    ///
    /// The runs a culled Paint answers are the whole Grid's runs clipped to
    /// the range it covers.
    ///
    /// Coalescing over the drawn columns and coalescing the whole row and
    /// clipping the answer produce the same rectangles: the shape step builds
    /// a run's rectangle from `GridViewport::cell_rect` on the run's first and
    /// last column, and `cell_rect` is a function of one column index, so both
    /// folds hand it the same two endpoints. The choice between them is cost
    /// alone.
    ///
    /// That is an equality rather than a resemblance, so it is asserted as one
    /// instead of left to a comment — which is what `3c2b640` asked for when
    /// it pinned the one-Cell margin's purpose with assertions.
    ///
    #[tokio::test]
    async fn the_runs_a_culled_paint_answers_are_the_whole_grids_clipped_to_it() {
        let mut orcvs = running_orcvs(32, 24);
        orcvs.select(orcvs.grid().position(15, 11).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let grid = frame.grid();
        let drawn = VisiblePositions::for_grid(grid, 10..22, 7..16);
        let clipped = whole(&frame)
            .background_runs()
            .into_iter()
            .filter(|run| drawn.rows.contains(&run.row))
            .filter_map(|run| {
                let columns = run.columns.start.max(drawn.columns.start)
                    ..run.columns.end.min(drawn.columns.end);

                (!columns.is_empty()).then_some(BackgroundRun { columns, ..run })
            })
            .collect::<Vec<_>>();

        assert_eq!(
            Paint::derive(FramePaint::new(&frame, drawn)).background_runs(),
            clipped
        );
    }

    ///
    /// A Paint answers the Cursor's Position where it covers it, and `None`
    /// where it does not.
    ///
    /// `None` is a statement about the viewport rather than about the Render
    /// Frame: the same Frame answers a Cursor to a Paint whose range reaches
    /// it. A derivation that searched its own Cells and insisted on finding
    /// one would panic on every zoomed console whose Cursor is off screen.
    ///
    #[tokio::test]
    async fn the_cursor_is_answered_only_where_the_paint_covers_it() {
        let mut orcvs = running_orcvs(20, 20);
        let selected = orcvs.grid().position(5, 4).expect("inside the grid");
        orcvs.select(selected);

        let frame = orcvs.render_frame();
        let grid = frame.grid();
        let reaching = VisiblePositions::for_grid(grid, 3..8, 2..7);
        let past = VisiblePositions::for_grid(grid, 10..16, 12..18);

        assert_eq!(whole(&frame).cursor(), Some(selected));
        assert_eq!(
            Paint::derive(FramePaint::new(&frame, reaching)).cursor(),
            Some(selected)
        );
        assert_eq!(Paint::derive(FramePaint::new(&frame, past)).cursor(), None);
    }

    ///
    /// A console showing none of its Grid paints nothing, and that is a state
    /// rather than an error.
    ///
    /// `GridViewport::visible_positions` answers `VisiblePositions::empty()`
    /// for a clip that misses the Grid, for a Grid with no Cell and for a
    /// degenerate viewport, so this is a range the shipped path reaches rather
    /// than one only a test can build.
    ///
    #[tokio::test]
    async fn a_paint_over_no_positions_is_empty_rather_than_impossible() {
        let orcvs = running_orcvs(8, 8);
        let frame = orcvs.render_frame();
        let paint = Paint::derive(FramePaint::new(&frame, VisiblePositions::empty()));

        assert_eq!(paint.count(), 0);
        assert_eq!(paint.cells().count(), 0);
        assert_eq!(paint.cursor(), None);
        assert_eq!(paint.background_runs(), Vec::new());
    }

    ///
    /// Ranges past the Grid are clamped before `Paint::derive` walks them,
    /// so an out-of-bounds request is empty or partial rather than a panic.
    ///
    #[tokio::test]
    async fn a_paint_clamps_out_of_bounds_ranges_before_it_walks_them() {
        let orcvs = running_orcvs(8, 8);
        let frame = orcvs.render_frame();
        let grid = frame.grid();
        let drawn = VisiblePositions::for_grid(grid, 6..100, 2..100);

        assert_eq!(drawn.columns, 6..8);
        assert_eq!(drawn.rows, 2..8);

        let paint = Paint::derive(FramePaint::new(&frame, drawn.clone()));

        assert_eq!(paint.count(), drawn.count());
    }

    ///
    /// Pairing refuses a range minted for another Grid, even when the shape
    /// matches the Render Frame's.
    ///
    #[tokio::test]
    #[should_panic(expected = "VisiblePositions belong to another Grid")]
    async fn frame_paint_refuses_visible_positions_minted_for_another_grid() {
        let orcvs = running_orcvs(8, 8);
        let frame = orcvs.render_frame();
        let other = Grid::new(8, 8);
        let drawn = VisiblePositions::for_grid(other, 0..8, 0..8);

        let _ = FramePaint::new(&frame, drawn);
    }
}
