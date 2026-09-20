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
//! [`Paint::derive_with_colours`] takes the pair, not the two values
//! separately.
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
    style::{cell_visuals_with_cursor_colour, sector_line},
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
/// seam before [`Paint::derive_with_colours`].
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
    /// is drawn as, in the colours it is handed.
    ///
    /// `cursor_colour` fills the Cursor's Cell. While a Region spans more than
    /// one Cell, `region_colour` fills the rest of it, and
    /// `region_cursor_colour` fills the Cursor's Cell independently — or, when
    /// `None`, `cursor_colour` still does. Only the fill carries over: the
    /// Cursor's Cell in such a Region takes the grid line and its sector
    /// seams, as every other Cell of the Region does. `source_paint` is the
    /// viewer's live `Theme → Source colours` value, so every Token colour
    /// here is a setting rather than a constant
    /// (`.scratch/syntax-highlighting/issues/01`).
    ///
    /// `cell_visuals_with_cursor_colour` is called once per drawn Cell and is
    /// unchanged: this decides what to do with its answer, not what the
    /// answer is. It reads the claim on the Cell (`RenderCell::claim`),
    /// whether that claim's slot holds written content — `slot_written`,
    /// below, answers the latter once per claim rather than once per Cell —
    /// and whether the Cell lies in a root Function's Output Portal
    /// Reservation (`RenderCell::output_portal`, `.scratch/syntax-
    /// highlighting/issues/06`).
    ///
    /// The range is the console's decision, not this layer's. It comes from
    /// `GridViewport::visible_positions` already clamped to the Grid, which is
    /// why the two slices below are indexed rather than reached through `get`:
    /// a range that outran this Render Frame would mean a Frame that is not
    /// the rectangle its own Grid says it is, and answering an empty slice
    /// there would turn that into rows that silently go unpainted. Pairing is
    /// checked in [`FramePaint::new`].
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
                    cell.output_portal(),
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
    /// [`Self::derive_with_colours`] at the Theme's Cursor and Region colours
    /// and the Source Paint defaults.
    ///
    /// Compiled only for tests, like [`Self::at`], so it is not a shipped
    /// seam: no console path derives a Paint from defaults, because
    /// `show_source` always holds the viewer's live settings. What this saves
    /// is a test having to name five colours to ask a question about one, and
    /// `console/benches/paint.rs` — a separate crate, which never sees this —
    /// states its own for the same reason
    /// (`.scratch/syntax-highlighting/issues/07`).
    ///
    #[cfg(test)]
    pub(crate) fn derive(input: FramePaint<'_>) -> Self {
        Self::derive_with_colours(
            input,
            Some(crate::style::PALETTE.selection_fill),
            crate::cursor_effects::DEFAULT_REGION_COLOUR,
            None,
            SourcePaintSettings::default(),
        )
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

    /// Writes `text` into `row`, one Cell at a time, the way every other
    /// fixture in this module does. A space writes the Cell's own blank
    /// content (`CellContent::SPACE`), which reads back identically to a
    /// Cell never written, so a row with gaps can be spelled as one
    /// literal.
    fn write_row(orcvs: &mut Orcvs, row: usize, text: &str) {
        let grid = orcvs.grid();
        for (x, character) in text.chars().enumerate() {
            orcvs.select(grid.position(x, row).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
    }

    /// `source_paint`'s own Fill tint mix of `colour`, restated
    /// independently of `style::fill_tint_colour` so a broken mix is
    /// caught rather than mirrored — the same reading `style::tests`
    /// pins its tint assertions with.
    fn tinted(source_paint: SourcePaintSettings, colour: Color32) -> Color32 {
        let strength = f32::from(source_paint.fill_tint()) / 100.0;
        source_paint
            .source_background()
            .lerp_to_gamma(colour, strength)
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
    /// claim, its slot's own written fact, and its own Output Portal fact
    /// straight from `frame` and `cell` — the same three inputs
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
            cell.output_portal(),
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
        for (x, character) in ".+a".chars().enumerate() {
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
    /// a `.v` (one Note operand) left unfilled. Atom and Sequence are not
    /// repeated here: `an_operand_cell_of_every_token_a_source_can_claim_is_
    /// tinted_with_its_own_colour` below covers all four Tokens from Source,
    /// drawing its Atom and Sequence slots from `:&` and `:<`, whose first
    /// operands declare them with no nested Function anywhere.
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
    /// Characters that spell no Function where an Expression could start —
    /// `hi`, a written `07`, a lone `|` — are claims the Parser records as
    /// `(Token::Function, atom: None)`. No signature declared anything
    /// there, so nothing was expected and nothing failed: they paint as
    /// Ordinary text with no tint. Diagnostic stays with an operand slot a
    /// signature declared and its written content did not satisfy — `c4` in
    /// `.+c401`'s first Number slot.
    ///
    #[tokio::test]
    async fn text_that_spells_no_function_is_ordinary_while_an_invalid_operand_stays_diagnostic() {
        let mut orcvs = running_orcvs(10, 2);
        for (row, text) in [(0, "hi 07 |"), (1, ".+c401")] {
            for (x, character) in text.chars().enumerate() {
                orcvs.select(orcvs.grid().position(x, row).expect("inside the grid"));
                orcvs.write(&character.to_string());
            }
        }
        orcvs.select(orcvs.grid().position(9, 1).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();

        for x in [0, 1, 3, 4, 6] {
            let position = orcvs.grid().position(x, 0).expect("inside the grid");
            let claim = frame.at(position).claim().expect("a claimed Cell");
            assert_eq!(
                (claim.token, claim.atom),
                (Token::Function, None),
                "column {x} was not a Function claim that bound nothing"
            );
            let painted = paint.at(position);
            assert_eq!(painted.foreground, source_paint.ordinary(), "column {x}");
            assert_eq!(painted.background, None, "column {x}");
        }

        for x in 2..4 {
            let position = orcvs.grid().position(x, 1).expect("inside the grid");
            let painted = paint.at(position);
            assert_eq!(painted.foreground, source_paint.diagnostic(), "column {x}");
            assert_eq!(
                painted.background,
                expected_visuals(&frame, frame.at(position), false, false, None, source_paint)
                    .background,
                "column {x} did not keep the Number tint"
            );
            assert!(painted.background.is_some(), "column {x} lost its tint");
        }
    }

    ///
    /// `.scratch/syntax-highlighting/issues/04`: `**` and `||` written into a
    /// typed operand slot are read as that slot's declared Token and refused.
    /// Neither is a nested Bang or a Comment at an operand position — the
    /// Parser only asks whether a Function comes next where an Expression
    /// could start — so both record an unbound `Token::Number` entry, keep
    /// the declared Number tint, and draw their glyphs in the Diagnostic
    /// colour. The valid operand beside each is unaffected: Number on the
    /// same Number tint.
    ///
    /// All four operand Cells of each Expression are asserted, because a
    /// rule stated only over the rejected pair cannot say that the operand
    /// beside it was left alone. The tint is read as one answer shared by the
    /// four Cells and distinct from `.+`'s own Function tint, rather than as
    /// a colour restated from `style::fill_tint_colour`'s mix.
    ///
    /// The two Expressions sit two rows apart so that neither one's Output
    /// Portal Reservation — row 1 for the first, row 3 for the second, both
    /// two Cells wide at columns 0-1 — reaches a Cell asserted here:
    /// `06`'s Output Portal paint takes precedence over the claim's own, so
    /// an operand under a Reservation would be answering a different rule.
    ///
    #[tokio::test]
    async fn a_rejected_bang_or_comment_in_an_operand_slot_draws_diagnostic_on_its_number_tint() {
        let mut orcvs = running_orcvs(8, 4);
        for (row, text) in [(0, ".+**01"), (2, ".+||02")] {
            for (x, character) in text.chars().enumerate() {
                orcvs.select(orcvs.grid().position(x, row).expect("inside the grid"));
                orcvs.write(&character.to_string());
            }
        }
        orcvs.select(orcvs.grid().position(7, 3).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();

        for (row, operands) in [(0, "**01"), (2, "||02")] {
            let mut tints = Vec::new();

            for (offset, character) in operands.chars().enumerate() {
                // The two operands of `.+` stand at columns 2-3 and 4-5; the
                // first two characters spell the rejected one.
                let x = offset + 2;
                let rejected = offset < 2;
                let position = orcvs.grid().position(x, row).expect("inside the grid");
                let claim = frame.at(position).claim().expect("a claimed operand Cell");

                assert_eq!(
                    claim.token,
                    Token::Number,
                    "row {row} column {x} was not claimed as a Number operand"
                );
                assert_eq!(
                    claim.atom.is_none(),
                    rejected,
                    "row {row} column {x} did not bind the way its operand is written"
                );

                let painted = paint.at(position);

                assert_eq!(painted.character, character, "row {row} column {x}");
                assert_eq!(
                    painted.foreground,
                    if rejected {
                        source_paint.diagnostic()
                    } else {
                        source_paint.number()
                    },
                    "row {row} column {x} drew the wrong glyph colour"
                );
                tints.push(painted.background);
            }

            let number_tint = tints[0];
            assert!(
                number_tint.is_some(),
                "row {row}: the rejected operand lost its tint"
            );
            assert!(
                tints.iter().all(|tint| *tint == number_tint),
                "row {row}: the four operand Cells did not share one tint: {tints:?}"
            );
            let function_tint = paint
                .at(orcvs.grid().position(0, row).expect("inside the grid"))
                .background;
            assert_ne!(
                number_tint, function_tint,
                "row {row}: the operand Cells took `.+`'s Function tint"
            );
        }
    }

    ///
    /// The Cursor's own fill wins outright on its Cell
    /// (`.scratch/syntax-highlighting/issues/01`), and an Invalid operand is
    /// no exception: the Cell under the Cursor fills with the Cursor's colour
    /// in place of its Number tint, while its glyph stays Diagnostic. Its
    /// twin one column over is the control — the other Cell of the same
    /// rejected `**`, unselected — so this says the Cursor replaced the fill
    /// rather than that the fill was never there.
    ///
    #[tokio::test]
    async fn the_cursor_fills_over_an_invalid_operands_tint_and_leaves_its_diagnostic_glyph() {
        let mut orcvs = running_orcvs(8, 2);
        for (x, character) in ".+**01".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        let cursor = orcvs.grid().position(2, 0).expect("inside the grid");
        orcvs.select(cursor);

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();
        let selected = paint.at(cursor);
        let unselected = paint.at(orcvs.grid().position(3, 0).expect("inside the grid"));

        assert_eq!(paint.cursor(), Some(cursor));
        assert_eq!(selected.character, '*');
        assert_eq!(
            selected.foreground,
            source_paint.diagnostic(),
            "the Cursor's Cell lost the Invalid operand's Diagnostic glyph"
        );
        assert_eq!(
            selected.background,
            Some(PALETTE.selection_fill),
            "the Cursor's Cell did not take the Cursor's own fill"
        );
        assert_eq!(
            unselected.foreground,
            source_paint.diagnostic(),
            "the unselected half of the same rejected operand"
        );
        assert!(
            unselected.background.is_some() && unselected.background != selected.background,
            "the unselected half did not keep a tint of its own: {:?}",
            unselected.background
        );
    }

    ///
    /// One Expression per Operand Token the tint distinguishes, written from
    /// Source: `.+` for Number, `:#C4D4` for Note, `:&` for Atom and `:<XY`
    /// for Sequence.
    ///
    /// The four rows stand two apart. `06`'s Output Portal paint takes
    /// precedence over an operand's own tint, and a root Function's
    /// Reservation lands one row south — the whole row south, for the three
    /// Sequence-answering roots here — so Expressions on adjacent rows would
    /// have row `N + 1`'s operands answering `06`'s rule instead of `02`'s.
    /// Two rows apart puts every Reservation on an empty row, and the tests
    /// below assert `!output_portal()` on every Cell they read rather than
    /// trusting this paragraph.
    ///
    /// The Cursor parks at the east end of row 0, which no Expression claims
    /// and no Reservation covers, so the Cursor's own fill cannot stand in
    /// for a tint.
    ///
    /// # Which binding states a Source can actually reach
    ///
    /// The four rows are not four Pending slots: they are the three binding
    /// states a Source can reach, spread across the four Tokens. `.+`'s
    /// Number slots and `:&`'s Atom slots are Pending, `:#C4D4`'s Note slots
    /// are bound, and `:<XY`'s Sequence slot is Invalid.
    ///
    /// A bound Atom or Sequence slot is deliberately absent, because no
    /// Source produces one. `Token::decode` refuses both outright
    /// (`lang/src/expression.rs`) — an Atom operand "has no literal reading"
    /// and a Sequence "has no literal spelling at all" — so the only thing
    /// that can satisfy either is a nested Function. And when a nested
    /// Function stands there, `take_language_unit`'s `is_function_next()`
    /// branch records the entry as `Token::Function` (`lang/src/parser.rs`),
    /// so the declared Token never reaches the Render Frame at all:
    /// `:<:-0104` carries Function and Number claims and no Sequence claim
    /// anywhere. Every `Token::Atom` or `Token::Sequence` claim a Source can
    /// produce is therefore unbound — Pending when its Cells are blank,
    /// Invalid when they are written.
    ///
    fn operand_token_sampler() -> Orcvs {
        let mut orcvs = running_orcvs(8, 8);
        for (row, text) in [(0, ".+"), (2, ":#C4D4"), (4, ":&"), (6, ":<XY")] {
            write_row(&mut orcvs, row, text);
        }
        orcvs.select(orcvs.grid().position(7, 0).expect("inside the grid"));
        orcvs
    }

    ///
    /// `syntax-highlighting/02`'s Fill tint on an Operand Cell of every Token
    /// a Source can claim, walked from written Source through the Render
    /// Frame rather than from a `Claim` built by hand: Number, Note, Atom and
    /// Sequence each tint with their own colour, and the Function's own
    /// two-Cell spelling beside them tints with Function's.
    ///
    /// The glyph colour is asserted alongside, because the tint alone cannot
    /// say that the right rule produced it: a bound Note draws Note, a
    /// Pending Number draws Number, a Pending Atom draws Ordinary, and
    /// `:<XY`'s Invalid Sequence draws Diagnostic while keeping the Sequence
    /// tint.
    ///
    /// The five tints are finally required to be five colours. Four operand
    /// arms that all answered one tint — the mix collapsing, or every Token
    /// reaching the same colour — would satisfy every assertion above.
    ///
    #[tokio::test]
    async fn an_operand_cell_of_every_token_a_source_can_claim_is_tinted_with_its_own_colour() {
        let orcvs = operand_token_sampler();
        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();
        let grid = orcvs.grid();
        let mut tints = std::collections::BTreeSet::new();

        for (row, operands, token, bound, tint, foreground) in [
            (
                0,
                2..6,
                Token::Number,
                false,
                source_paint.number(),
                source_paint.number(),
            ),
            (
                2,
                2..6,
                Token::Note,
                true,
                source_paint.note(),
                source_paint.note(),
            ),
            (
                4,
                2..6,
                Token::Atom,
                false,
                source_paint.ordinary(),
                source_paint.ordinary(),
            ),
            (
                6,
                2..4,
                Token::Sequence,
                false,
                source_paint.sequence(),
                source_paint.diagnostic(),
            ),
        ] {
            for x in operands {
                let position = grid.position(x, row).expect("inside the grid");
                let cell = frame.at(position);

                assert!(
                    !cell.output_portal(),
                    "row {row} column {x} is inside a Reservation, so `06`'s rule answers it"
                );
                let claim = cell.claim().expect("a claimed operand Cell");
                assert_eq!(
                    claim.token, token,
                    "row {row} column {x} was not claimed as the operand it declares"
                );
                assert_eq!(
                    claim.atom.is_some(),
                    bound,
                    "row {row} column {x} did not bind the way its operand is written"
                );

                let painted = paint.at(position);
                assert_eq!(
                    painted.background,
                    Some(tinted(source_paint, tint)),
                    "row {row} column {x} was not tinted with its own Token's colour"
                );
                assert_eq!(
                    painted.foreground, foreground,
                    "row {row} column {x} drew the wrong glyph colour"
                );
            }

            // The root Function's own two Cells, on the same row: the
            // Function tint, not the operand's.
            for x in 0..2 {
                let position = grid.position(x, row).expect("inside the grid");
                let painted = paint.at(position);
                assert_eq!(
                    painted.background,
                    Some(tinted(source_paint, source_paint.function())),
                    "row {row} column {x} was not tinted with the Function colour"
                );
                assert_eq!(
                    painted.foreground,
                    source_paint.function(),
                    "row {row} column {x} did not draw the Function colour"
                );
            }

            tints.insert(tinted(source_paint, tint).to_array());
        }

        tints.insert(tinted(source_paint, source_paint.function()).to_array());
        assert_eq!(
            tints.len(),
            5,
            "the four Operand tints and the Function tint were not five colours: {tints:?}"
        );
    }

    ///
    /// A Fill tint of `0` paints no background anywhere — not on a Function
    /// Cell, not on an Operand Cell of any Token, and not on an Output Portal
    /// Cell either, since `06`'s paint mixes at the same strength. The
    /// assertion is over every Cell of the Grid rather than over the claimed
    /// ones, so a role that acquired a fill of its own would fail here.
    ///
    /// The same Source at the default strength is the control: each of the 22
    /// Cells an Expression claims does carry a background there, so `0`
    /// answering `None` is the strength doing it and not the fixture having
    /// nothing to tint. `None` rather than `Some(source_background)` is the
    /// point of the rule — a Cell tinted at `0%` costs `background_runs`
    /// nothing to walk.
    ///
    /// The Cursor's colour is handed in as `None` here rather than as the
    /// Theme's fill, so the Cursor's own Cell has no fill to contribute
    /// either; its precedence over the tint is
    /// `the_cursors_own_fill_wins_over_a_function_cells_tint`'s subject.
    ///
    #[tokio::test]
    async fn zero_percent_fill_tint_paints_no_tint_on_any_cell_a_source_claims() {
        let orcvs = operand_token_sampler();
        let frame = orcvs.render_frame();
        let grid = orcvs.grid();
        let mut source_paint = SourcePaintSettings::default();
        let painted = |source_paint| {
            Paint::derive_with_colours(
                FramePaint::whole(&frame),
                None,
                crate::cursor_effects::DEFAULT_REGION_COLOUR,
                None,
                source_paint,
            )
        };

        assert_ne!(source_paint.fill_tint(), 0);
        let default_strength = painted(source_paint);
        let claimed: Vec<_> = frame
            .cells()
            .iter()
            .filter(|cell| cell.claim().is_some())
            .map(|cell| cell.position())
            .collect();
        assert_eq!(
            claimed.len(),
            22,
            "the fixture stopped claiming what it did"
        );
        for position in &claimed {
            assert!(
                default_strength.at(*position).background.is_some(),
                "{position:?} had nothing to tint at the default strength"
            );
        }

        *source_paint.fill_tint_mut() = 0;
        let no_strength = painted(source_paint);
        for row in 0..grid.rows() {
            for column in 0..grid.columns() {
                let position = grid.position(column, row).expect("inside the grid");
                assert_eq!(
                    no_strength.at(position).background,
                    None,
                    "{position:?} was tinted at 0%"
                );
            }
        }
    }

    ///
    /// Comment, Bang and an empty unclaimed Cell take no tint, from Source
    /// rather than from a hand-built claim: `||hello` claims its whole row as
    /// one Comment, a standalone `**` is one bound Bang claim, and the Cells
    /// east of it are claimed by nothing at all. Each keeps its own glyph
    /// colour and answers no background.
    ///
    /// Read at the default Fill tint strength, which is asserted non-zero, so
    /// a missing exclusion cannot hide behind a strength of zero. None of the
    /// three roles is inside a Reservation — a Comment reserves nothing and a
    /// standalone Bang is not a root Function — which the test asserts rather
    /// than assumes, since `06`'s paint would otherwise supply the tint that
    /// is supposed to be absent.
    ///
    #[tokio::test]
    async fn a_comment_a_bang_and_an_unclaimed_cell_take_no_tint_from_source() {
        let mut orcvs = running_orcvs(8, 4);
        write_row(&mut orcvs, 0, "||hello");
        write_row(&mut orcvs, 2, "**");
        orcvs.select(orcvs.grid().position(7, 3).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();
        let grid = orcvs.grid();
        assert_ne!(source_paint.fill_tint(), 0);

        for (row, columns, token, foreground) in [
            (0, 0..8, Some(Token::Comment), source_paint.comment()),
            (2, 0..2, Some(Token::Bang), source_paint.bang()),
            (2, 2..8, None, source_paint.ordinary()),
        ] {
            for x in columns {
                let position = grid.position(x, row).expect("inside the grid");
                let cell = frame.at(position);

                assert!(
                    !cell.output_portal(),
                    "row {row} column {x} is inside a Reservation, so `06`'s rule answers it"
                );
                assert_eq!(
                    cell.claim().map(|claim| claim.token),
                    token,
                    "row {row} column {x} was not claimed the way this row is written"
                );

                let painted = paint.at(position);
                assert_eq!(painted.background, None, "row {row} column {x} was tinted");
                assert_eq!(
                    painted.foreground, foreground,
                    "row {row} column {x} drew the wrong glyph colour"
                );
            }
        }
    }

    ///
    /// The Cursor's own fill wins over the Fill tint on its Cell
    /// (`syntax-highlighting/02`), driven from Source on a Function Cell:
    /// `.+0102`'s `.` is the Cursor's Cell and fills with the Cursor's
    /// colour, while its twin `+` — the other Cell of the same Function
    /// spelling, unselected — keeps the Function tint. The control is what
    /// says the Cursor replaced a fill rather than that there was none.
    ///
    /// The glyph is untouched: a Cursor changes which fill a Cell takes and
    /// not what colour the Function spells in.
    /// `the_cursor_fills_over_an_invalid_operands_tint_and_leaves_its_
    /// diagnostic_glyph` is the same rule over an Operand Cell.
    ///
    #[tokio::test]
    async fn the_cursors_own_fill_wins_over_a_function_cells_tint() {
        let mut orcvs = running_orcvs(6, 2);
        write_row(&mut orcvs, 0, ".+0102");
        let cursor = orcvs.grid().position(0, 0).expect("inside the grid");
        orcvs.select(cursor);

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let source_paint = SourcePaintSettings::default();
        let function_tint = tinted(source_paint, source_paint.function());
        let selected = paint.at(cursor);
        let unselected = paint.at(orcvs.grid().position(1, 0).expect("inside the grid"));

        assert_eq!(paint.cursor(), Some(cursor));
        assert_ne!(PALETTE.selection_fill, function_tint);
        assert_eq!(
            selected.background,
            Some(PALETTE.selection_fill),
            "the Cursor's Cell did not take the Cursor's own fill"
        );
        assert_eq!(
            unselected.background,
            Some(function_tint),
            "the unselected half of the same Function spelling lost its tint"
        );
        assert_eq!(
            selected.foreground,
            source_paint.function(),
            "the Cursor's Cell stopped spelling in the Function colour"
        );
    }

    ///
    /// `.scratch/syntax-highlighting/issues/06`: end-to-end Output Portal
    /// paint tests, built from Source text through the Render Frame. Every
    /// scenario writes the answer directly into Source rather than running a
    /// Tick, since the highlight comes from the current revision alone
    /// (`.scratch/syntax-highlighting/issues/05`'s Answer): a written value
    /// south of a Function is indistinguishable from one a Tick wrote.
    ///
    mod output_portal_paint {
        use super::{SourcePaintSettings, running_orcvs, tinted, whole, write_row};

        ///
        /// A scalar answer south of a Function draws in the Output Portal
        /// colour on the Output Portal tint: `07` left south of `.+0304` re-
        /// parses as two one-Cell unbound Function claims (`.scratch/syntax-
        /// highlighting/issues/05`'s Answer), and the Output Portal fact
        /// paints over that Diagnostic-shaped claim rather than leaving it
        /// Diagnostic.
        ///
        #[tokio::test]
        async fn a_scalar_answer_paints_in_the_output_portal_colour() {
            let mut orcvs = running_orcvs(6, 3);
            write_row(&mut orcvs, 0, ".+0304");
            write_row(&mut orcvs, 1, "07");
            orcvs.select(orcvs.grid().position(0, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let output_tinted = tinted(source_paint, source_paint.output_portal());
            let grid = orcvs.grid();

            for x in 0..2 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(frame.at(position).output_portal(), "column {x}");
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground,
                    source_paint.output_portal(),
                    "column {x}"
                );
                assert_eq!(painted.background, Some(output_tinted), "column {x}");
            }
            // Past the scalar Reservation's pair: an ordinary, untinted blank
            // Cell, the same as if `.+0304` were not there.
            for x in 2..6 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(!frame.at(position).output_portal(), "column {x}");
                assert_eq!(paint.at(position).background, None, "column {x}");
            }
        }

        ///
        /// The other scalar-answer example `06` names: `C4` left south of
        /// `.^3C` (ConvertToNote) paints the same way — the rule reads the
        /// Output Portal fact and the claim's shape, not which Function or
        /// which characters produced it.
        ///
        #[tokio::test]
        async fn a_note_shaped_scalar_answer_paints_in_the_output_portal_colour_too() {
            let mut orcvs = running_orcvs(4, 3);
            write_row(&mut orcvs, 0, ".^3C");
            write_row(&mut orcvs, 1, "C4");
            orcvs.select(orcvs.grid().position(0, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let output_tinted = tinted(source_paint, source_paint.output_portal());
            let grid = orcvs.grid();

            for x in 0..2 {
                let position = grid.position(x, 1).expect("inside the grid");
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground,
                    source_paint.output_portal(),
                    "column {x}"
                );
                assert_eq!(painted.background, Some(output_tinted), "column {x}");
            }
        }

        ///
        /// A Sequence answer is painted the same way across every one of its
        /// Cells, and the highlight stops where the answer does rather than
        /// running to the end of the row. `:<:-0104` (Reverse of NumberRange
        /// 01..04) answers a Sequence outright, so its Reservation runs to
        /// the end of the destination row (`.scratch/syntax-highlighting/
        /// issues/10`'s Answer) — but the highlight is fitted to the written
        /// answer, `04030201` (`12`), so the row's last two Cells are the
        /// ordinary untinted blanks they would be with no root above them,
        /// even though the Reservation still covers them.
        ///
        #[tokio::test]
        async fn a_sequence_answer_paints_every_cell_and_stops_where_the_answer_does() {
            let mut orcvs = running_orcvs(10, 2);
            write_row(&mut orcvs, 0, ":<:-0104");
            write_row(&mut orcvs, 1, "04030201");
            orcvs.select(orcvs.grid().position(8, 0).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let output_tinted = tinted(source_paint, source_paint.output_portal());
            let grid = orcvs.grid();

            for x in 0..8 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(frame.at(position).output_portal(), "column {x}");
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground,
                    source_paint.output_portal(),
                    "column {x}"
                );
                assert_eq!(painted.background, Some(output_tinted), "column {x}");
            }
            // Past the answer: the Reservation still covers these Cells, and
            // the highlight no longer does, so they draw as ordinary blanks.
            for x in 8..10 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(!frame.at(position).output_portal(), "column {x}");
                assert_eq!(frame.at(position).content(), None, "column {x}");
                let painted = paint.at(position);
                assert_eq!(painted.character, ' ', "column {x}");
                assert_eq!(painted.background, None, "column {x}");
            }
        }

        ///
        /// A Bang answer keeps the Bang glyph colour rather than the Output
        /// Portal colour, but takes the Output Portal tint in place of
        /// Bang's usual bare `None`: `**` left south of `~*0401` (Delay)
        /// reads as a genuine bound Bang claim (`RenderFrame`'s own
        /// `a_standalone_bang_is_one_bound_bang_claim`), and the tint is what
        /// tells it apart from an ordinary Bang elsewhere on the Grid — the
        /// second `**`, on the row below, sits outside any Reservation and
        /// stays untinted.
        ///
        #[tokio::test]
        async fn a_bang_answer_keeps_its_glyph_colour_but_takes_the_output_portal_tint() {
            let mut orcvs = running_orcvs(6, 3);
            write_row(&mut orcvs, 0, "~*0401");
            write_row(&mut orcvs, 1, "**");
            write_row(&mut orcvs, 2, "**");
            orcvs.select(orcvs.grid().position(4, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let output_tinted = tinted(source_paint, source_paint.output_portal());
            let grid = orcvs.grid();

            for x in 0..2 {
                let portal_position = grid.position(x, 1).expect("inside the grid");
                assert!(frame.at(portal_position).output_portal(), "column {x}");
                let portal_painted = paint.at(portal_position);
                assert_eq!(portal_painted.foreground, source_paint.bang(), "column {x}");
                assert_eq!(portal_painted.background, Some(output_tinted), "column {x}");

                let ordinary_position = grid.position(x, 2).expect("inside the grid");
                assert!(!frame.at(ordinary_position).output_portal(), "column {x}");
                let ordinary_painted = paint.at(ordinary_position);
                assert_eq!(
                    ordinary_painted.foreground,
                    source_paint.bang(),
                    "column {x}"
                );
                assert_eq!(
                    ordinary_painted.background, None,
                    "an ordinary Bang outside a Reservation is untinted, column {x}"
                );
            }
        }

        ///
        /// An empty Output Portal Cell shows the Output Portal tint and no
        /// glyph, including before the first Tick: nothing ever runs a Tick
        /// in this test, so the highlight can only have come from the
        /// current Source revision, as `.scratch/syntax-highlighting/
        /// issues/05`'s Answer states.
        ///
        #[tokio::test]
        async fn an_empty_output_portal_cell_shows_the_tint_and_no_glyph_before_any_tick() {
            let mut orcvs = running_orcvs(6, 3);
            write_row(&mut orcvs, 0, ".+0102");
            orcvs.select(orcvs.grid().position(0, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let output_tinted = tinted(source_paint, source_paint.output_portal());
            let grid = orcvs.grid();

            for x in 0..2 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(frame.at(position).output_portal(), "column {x}");
                assert_eq!(frame.at(position).content(), None, "column {x}");
                let painted = paint.at(position);
                assert_eq!(painted.character, ' ', "column {x}");
                assert_eq!(painted.background, Some(output_tinted), "column {x}");
            }
            for x in 2..6 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(!frame.at(position).output_portal(), "column {x}");
                assert_eq!(paint.at(position).background, None, "column {x}");
            }
        }

        ///
        /// A Terminal Output Function paints nothing at its Output Portal:
        /// `!>` (RawPlay) answers Play, not a Cell, so `output_portal()` is
        /// `None` for it (`.scratch/syntax-highlighting/issues/10`'s
        /// Answer) and the row south of it reads exactly as if no root stood
        /// north of it.
        ///
        #[tokio::test]
        async fn a_terminal_output_functions_south_row_is_ordinary() {
            let mut orcvs = running_orcvs(8, 2);
            write_row(&mut orcvs, 0, "!>007F");
            orcvs.select(orcvs.grid().position(6, 0).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let grid = orcvs.grid();

            for x in 0..8 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(!frame.at(position).output_portal(), "column {x}");
                let painted = paint.at(position);
                assert_eq!(painted.foreground, source_paint.ordinary(), "column {x}");
                assert_eq!(painted.background, None, "column {x}");
                assert_eq!(painted.character, ' ', "column {x}");
            }
        }

        ///
        /// Halt paints nothing at its Output Portal: it locks the root there
        /// rather than writing (`.scratch/syntax-highlighting/issues/10`'s
        /// Answer), so the row south of `*!` reads as ordinary.
        ///
        #[tokio::test]
        async fn halts_south_row_is_ordinary() {
            let mut orcvs = running_orcvs(4, 3);
            write_row(&mut orcvs, 0, "*!");
            orcvs.select(orcvs.grid().position(0, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let grid = orcvs.grid();

            for x in 0..4 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(!frame.at(position).output_portal(), "column {x}");
                let painted = paint.at(position);
                assert_eq!(painted.foreground, source_paint.ordinary(), "column {x}");
                assert_eq!(painted.background, None, "column {x}");
            }
        }

        ///
        /// A Self-Banging (Source-writing) Function paints nothing at its
        /// Output Portal: its Advance's writes, including the Cells it
        /// moves onto, are its declared Source effect rather than an answer
        /// through an Output Portal (`.scratch/syntax-highlighting/
        /// issues/10`'s Answer). `>>` is `SelfBangingEast`; the Cells east
        /// of its own anchor, which its Advance would move onto during a
        /// Tick, carry no Output Portal fact even though nothing has ticked
        /// yet to prove that by writing there.
        ///
        #[tokio::test]
        async fn a_self_banging_functions_advance_path_is_never_an_output_portal() {
            let mut orcvs = running_orcvs(6, 1);
            write_row(&mut orcvs, 0, ">>");
            // The Function's own anchor, outside the asserted range 2..6.
            orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let grid = orcvs.grid();

            for x in 2..6 {
                let position = grid.position(x, 0).expect("inside the grid");
                assert!(!frame.at(position).output_portal(), "column {x}");
                let painted = paint.at(position);
                assert_eq!(painted.foreground, source_paint.ordinary(), "column {x}");
                assert_eq!(painted.background, None, "column {x}");
            }
        }

        ///
        /// The paint precedence where an Output Portal covers another
        /// Expression's claimed Cells (`.scratch/syntax-highlighting/
        /// issues/06`'s precedence decision): `:-0102` (NumberRange) answers
        /// a Sequence outright, so its Reservation runs the whole of the row
        /// south, where `.+0304` (Add) stands as a second, independent root.
        /// Add's own two-Cell spelling is a bound Function claim and keeps
        /// its Function paint outright — the "another root" case — while
        /// Add's own Number operand Cells, which the Reservation also
        /// covers, take the Output Portal colour and tint instead of their
        /// declared Number role — the "a consumer's operand" case. Both
        /// named overlaps from `.scratch/syntax-highlighting/issues/05`'s
        /// Answer are exercised by this one Source.
        ///
        #[tokio::test]
        async fn a_bound_function_spelling_wins_but_its_operands_take_the_output_portal() {
            let mut orcvs = running_orcvs(6, 2);
            write_row(&mut orcvs, 0, ":-0102");
            write_row(&mut orcvs, 1, ".+0304");
            // Writing leaves the Cursor at the last Cell it wrote (row 1,
            // column 5) — one of the Cells this test asserts about. Move it
            // back onto row 0, which no assertion below reads.
            orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let function_tinted = tinted(source_paint, source_paint.function());
            let output_tinted = tinted(source_paint, source_paint.output_portal());
            let grid = orcvs.grid();

            for x in 0..6 {
                assert!(
                    frame
                        .at(grid.position(x, 1).expect("inside the grid"))
                        .output_portal(),
                    "column {x}: NumberRange answers a Sequence outright"
                );
            }

            for x in 0..2 {
                let position = grid.position(x, 1).expect("inside the grid");
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground,
                    source_paint.function(),
                    "Add's own spelling, column {x}"
                );
                assert_eq!(
                    painted.background,
                    Some(function_tinted),
                    "Add's own spelling, column {x}"
                );
            }
            for x in 2..6 {
                let position = grid.position(x, 1).expect("inside the grid");
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground,
                    source_paint.output_portal(),
                    "Add's own Number operand, column {x}"
                );
                assert_eq!(
                    painted.background,
                    Some(output_tinted),
                    "Add's own Number operand, column {x}"
                );
            }
        }

        ///
        /// The regression `.scratch/syntax-highlighting/issues/12` was
        /// opened for, from a screenshot on 2026-09-19: a two-column layout
        /// where Sequence-capable roots on the left share rows with scalar
        /// roots on the right.
        ///
        /// With the highlight covering the whole Reservation, each left
        /// root tinted its destination row to the Grid's right edge, so the
        /// tint ran under the right column's Tick Expressions — `.+0304` on
        /// row 1 and `.^3C` on row 3 — and past the right column's own
        /// two-Cell Output Portals. Fitted to the answer, each left root
        /// stops at its own written Cells and the right column is left
        /// entirely to its own roots.
        ///
        /// The right column is offset one row from the left so that a left
        /// root's destination row is a right root's Expression row; that is
        /// the arrangement the screenshot had, and it is what makes the
        /// defect visible rather than hidden under a Cell both roots cover.
        ///
        #[tokio::test]
        async fn a_left_columns_sequence_tint_never_reaches_the_right_columns_roots() {
            let mut orcvs = running_orcvs(20, 5);
            //                        01234567890123456789
            write_row(&mut orcvs, 0, ":-0104              ");
            write_row(&mut orcvs, 1, "01020304    .+0304  ");
            write_row(&mut orcvs, 2, ":#C4D4      07      ");
            write_row(&mut orcvs, 3, "C4c4D4      .^3C    ");
            write_row(&mut orcvs, 4, "            C4      ");
            orcvs.select(orcvs.grid().position(19, 0).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let source_paint = SourcePaintSettings::default();
            let output_tinted = tinted(source_paint, source_paint.output_portal());
            let number_tinted = tinted(source_paint, source_paint.number());
            let grid = orcvs.grid();
            let highlighted = |y: usize| -> String {
                (0..grid.columns())
                    .map(|x| {
                        let position = grid.position(x, y).expect("inside the grid");
                        if frame.at(position).output_portal() {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect()
            };

            // Each left root covers its own answer and stops: `01020304` is
            // eight Cells, `C4c4D4` six. Each right root covers its own Cell
            // pair. Nothing else on the Grid is highlighted.
            assert_eq!(highlighted(0), "....................");
            assert_eq!(highlighted(1), "########............");
            assert_eq!(highlighted(2), "............##......");
            assert_eq!(highlighted(3), "######..............");
            assert_eq!(highlighted(4), "............##......");

            // The right roots' own Number operands keep the Number colour on
            // the Number tint. Under the whole-row highlight they took the
            // Output Portal colour on the Output Portal tint, because the
            // left root's Reservation covered them.
            for (y, columns) in [(1_usize, 14..18_usize), (3, 14..16)] {
                for x in columns {
                    let position = grid.position(x, y).expect("inside the grid");
                    let painted = paint.at(position);
                    assert_eq!(
                        painted.foreground,
                        source_paint.number(),
                        "right operand at ({x}, {y})"
                    );
                    assert_eq!(
                        painted.background,
                        Some(number_tinted),
                        "right operand at ({x}, {y})"
                    );
                }
            }

            // The right roots' own answers still read as Output Portals, and
            // the Cells past each pair are ordinary blanks rather than the
            // left root's tint reaching on.
            for y in [2_usize, 4] {
                for x in 12..14 {
                    let position = grid.position(x, y).expect("inside the grid");
                    let painted = paint.at(position);
                    assert_eq!(
                        painted.foreground,
                        source_paint.output_portal(),
                        "right answer at ({x}, {y})"
                    );
                    assert_eq!(
                        painted.background,
                        Some(output_tinted),
                        "right answer at ({x}, {y})"
                    );
                }
                for x in 14..20 {
                    let position = grid.position(x, y).expect("inside the grid");
                    assert_eq!(paint.at(position).background, None, "blank at ({x}, {y})");
                }
            }
        }
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
