//!
//! The per-Cell decision of how one Render Frame is drawn: background, border,
//! foreground, sector seams and the character shown.
//!
//! # No geometry
//!
//! Nothing here is a `Rect`, and nothing here reads a viewport's geometry.
//! Where a Cell *is* on screen is `GridViewport`'s arithmetic and is asserted
//! there; a `Rect` in this layer would make every assertion about a colour
//! acquire a viewport before it could be written. Which Cell edges carry a
//! Sector Seam is decided here from the Frame's spacing answer, in the
//! Theme's `sector.seam`, while the stroke *widths* the same seams are drawn
//! with belong to the step that has the geometry.
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
//! [`Paint::derive_with_theme`] takes the pair, not the two values
//! separately.
//!
//! # Only the Positions the console draws
//!
//! A Paint covers the Positions `GridViewport::visible_positions` answers and
//! no others, so a console showing a tenth of a Grid pays for a tenth of it:
//! neither the per-Cell call to `CellColours::colours` nor the `Vec` holding its
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

use std::ops::Range;

use egui::{Color32, Stroke};

use orcvs::{
    grid::{Grid, Position},
    render_frame::RenderFrame,
};

use crate::{
    marks::{sector_left, sector_top},
    style::{CellColours, CursorPlacement},
    theme::{GridWidth, Theme},
};

pub use crate::grid_viewport::VisiblePositions;

///
/// What one Cell of a Render Frame is drawn as.
///
/// Flat. The background is already decided by `CellColours::colours`: `None` means the
/// panel behind the Grid has painted the Source colour, and `Some` means this
/// Cell needs a fill of its own.
///
///
/// `border` is the Cell's own border fact as `crate::style::CellVisuals::border`
/// resolved it, `None` where the Cell has none; never `Eq` since a `Stroke`
/// carries an `f32`, unlike this struct's other fields.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CellPaint {
    pub background: Option<Color32>,
    pub border: Option<EdgeLine>,
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
/// Which fact draws a Cell edge, lowest first. An edge shows the line of the
/// highest-ranked fact either Cell beside it carries on that side.
///
/// The Portal roles rank as [`crate::style`] orders them for paint: Output
/// over Input over Locked.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeRank {
    GridLine,
    /// A Source role's own border.
    Role,
    /// A Bang's own border, which a Bang keeps on all four of its edges
    /// against any other role's.
    Bang,
    Diagnostic,
    Locked,
    InputPortal,
    OutputPortal,
    /// The Cursor's own Cell inside a Region larger than one Cell, carrying
    /// the border its fact gives it.
    RegionCursor,
    /// The selection outline.
    Selection,
}

///
/// The line a fact draws on an edge: its rank, and its colour already
/// composited over the facts below it on its own Cell, at its width in
/// points.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeLine {
    pub rank: EdgeRank,
    pub stroke: Stroke,
    /// Whether a Sector Seam is composited into `stroke`.
    pub seamed: bool,
}

impl EdgeLine {
    ///
    /// The line `rank` draws at `width` in `colour`, unseamed, or `None` where
    /// `width` is zero: a zero width draws no line. Only the width
    /// decides. A transparent line of nonzero width is drawn, and still wins
    /// its edge over the facts below it.
    ///
    pub(crate) fn drawn(rank: EdgeRank, width: GridWidth, colour: Color32) -> Option<Self> {
        (width.points() > 0.0).then(|| Self {
            rank,
            stroke: Stroke::new(width.points(), colour),
            seamed: false,
        })
    }

    ///
    /// The line an edge shows where neither Cell beside it carries a fact, or
    /// `None` where `grid.border.width` is zero.
    ///
    pub(crate) fn grid(theme: &Theme) -> Option<Self> {
        Self::drawn(
            EdgeRank::GridLine,
            theme.grid_border_width,
            theme.grid_border,
        )
    }
}

///
/// Consecutive edges along one row line or column line that show the same
/// line: the boundary's index, the Cells it runs along, and the line.
///
/// A row line `line` is the top edge of row `line`, and runs along columns; a
/// column line is the left edge of column `line`, and runs along rows. The
/// last line of each axis is the far edge of the last drawn Cell.
///
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeRun {
    pub line: usize,
    pub span: Range<usize>,
    pub edge: EdgeLine,
}

///
/// The selection outline: the Cursor's Cell, or a Region larger than one
/// Cell, as the Grid columns and rows it encloses, and the line on every
/// edge of its perimeter.
///
#[derive(Clone, Debug, PartialEq)]
struct Outline {
    line: EdgeLine,
    columns: Range<usize>,
    rows: Range<usize>,
}

impl Outline {
    /// Whether the edge along row line `row` above or below `column` is on
    /// this outline.
    fn across(&self, row: usize, column: usize) -> bool {
        (row == self.rows.start || row == self.rows.end) && self.columns.contains(&column)
    }

    /// Whether the edge along column line `column` beside `row` is on this
    /// outline.
    fn down(&self, column: usize, row: usize) -> bool {
        (column == self.columns.start || column == self.columns.end) && self.rows.contains(&row)
    }
}

///
/// Every drawn Cell edge, coalesced into runs along the row lines and the
/// column lines.
///
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Edges {
    pub rows: Vec<EdgeRun>,
    pub columns: Vec<EdgeRun>,
}

///
/// A Render Frame and the Positions the console draws from it, paired at one
/// seam before [`Paint::derive_with_theme`].
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
    /// The selection outline, or `None` where its border's width is zero.
    outline: Option<Outline>,
    /// The line an edge shows where neither Cell beside it carries a fact,
    /// or `None` where `grid.border.width` is zero.
    grid_line: Option<EdgeLine>,
    /// `sector.seam.width`, in points: a Cell's seams are absent where it is
    /// zero.
    sector_seam_width: f32,
    cells: Vec<CellPaint>,
}

impl Paint {
    ///
    /// Reads a paired Render Frame and drawn range and answers what each Cell
    /// is drawn as, resolved from `theme`.
    ///
    /// `theme.cursor_background` fills the Cursor's Cell, and
    /// `theme.cursor_foreground`, when set, colours its glyph. While a Region
    /// spans more than one Cell, `theme.region_background` fills the rest of
    /// it, and `theme.region_cursor_background` fills the Cursor's Cell
    /// independently — or, when `None`, the ordinary Cursor fill still does.
    /// Its glyph takes `theme.region_cursor_glyph()` the same way
    /// (`style::CellColours::colours` decides both glyphs).
    /// `theme.cell_background` is the uniform base every one of those fills
    /// composites over, and — being independent of whether a role
    /// contributed one — is also what a Cell with no fill of its own falls
    /// back to, so `.scratch/theming/schema.md`'s "does not count as a fact
    /// fill when evaluating Region fallback" holds even when a custom
    /// Theme's Cell base is not transparent. `theme` is the resolved Theme
    /// the console currently shows the Source with, so every Token colour
    /// here is a Theme channel rather than a constant.
    ///
    /// The outline is `theme.cursor_border` at `theme.cursor_border_width`
    /// around the Cursor's Cell, or `theme.region_border` at
    /// `theme.region_border_width` around a spanning Region, whose Cursor's
    /// Cell keeps its own border and sector seams.
    ///
    /// `style::CellColours` decides each Cell's colours: this decides where
    /// a Cell sits relative to the Cursor and Region, and what to do with the
    /// answer, not what the answer is. It reads the finished language fact
    /// on the Cell (`RenderCell::source_paint`), the Portal roles the Cell
    /// plays (`RenderCell::portal_roles`), and whether the Cell holds
    /// content, which decides whether an unclaimed Cell's Ordinary border can
    /// show.
    ///
    /// `theme` is resolved once here, before the loop, rather than once per
    /// Cell: a per-Cell resolution would add its cost to every drawn Cell of
    /// every frame. `style::CellColours` holds the prepared fills and the
    /// answer for each fact and painting Portal the walk asks for, so a Cell
    /// costs one indexed load rather than the role match, Diagnostic/Portal
    /// blends, `cell.background` composite and border priority match.
    ///
    /// The range is the console's decision, not this layer's. It comes from
    /// `GridViewport::visible_positions` already clamped to the Grid, which is
    /// why the two slices below are indexed rather than reached through `get`:
    /// a range that outran this Render Frame would mean a Frame that is not
    /// the rectangle its own Grid says it is, and answering an empty slice
    /// there would turn that into rows that silently go unpainted. Pairing is
    /// checked in [`FramePaint::new`].
    ///
    pub fn derive_with_theme(input: FramePaint<'_>, theme: &Theme) -> Self {
        let FramePaint { frame, drawn } = input;
        let grid = frame.grid();
        // The Cursor is the Position the Render Frame was derived for, inside
        // the drawn range or not.
        let frame_cursor = frame.cursor();
        let sector_seam_spacing = frame.sector_seam_spacing().cells();
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
        let sector_seam = theme.sector_seam;
        let sector_seam_width = theme.sector_seam_width.points();
        let seams = sector_seam_width > 0.0;
        let mut colours = CellColours::new(theme);
        // Sized up front. The drawn count is known exactly, so collecting into
        // a `Vec` need not grow by doubling across the walk.
        let mut cells = Vec::with_capacity(drawn.count());
        for row in drawn.rows.clone() {
            for column in drawn.columns.clone() {
                let position = grid
                    .position(column, row)
                    .expect("a drawn Position is one the visible range clamped to this Grid");
                let cell = frame.at(position);
                // While a Region spans more than one Cell the outline is the
                // Region's, and the Cursor's Cell is selected only by its
                // fill.
                let placement = if position == frame_cursor {
                    if region_spans {
                        CursorPlacement::RegionCursor
                    } else {
                        CursorPlacement::Cursor
                    }
                } else if region_columns.contains(&column) && region_rows.contains(&row) {
                    CursorPlacement::Region
                } else {
                    CursorPlacement::Plain
                };
                let visuals = colours.colours(
                    cell.source_paint(),
                    cell.portal_roles(),
                    cell.content().is_some(),
                    placement,
                );

                // The Cursor's own Cell inside a spanning Region keeps its
                // Diagnostic or Portal border, ranked over every other fact.
                // A role border keeps its own rank, so it never hides a
                // neighbour's Diagnostic or Portal frame.
                let border = match visuals.border {
                    Some(line)
                        if placement == CursorPlacement::RegionCursor
                            && line.rank >= EdgeRank::Diagnostic =>
                    {
                        Some(EdgeLine {
                            rank: EdgeRank::RegionCursor,
                            ..line
                        })
                    }
                    border => border,
                };
                cells.push(CellPaint {
                    background: visuals.background,
                    border,
                    foreground: visuals.foreground,
                    // The colour is `sector_seam`, read from `theme` once
                    // above the loop. Whether an edge shows it is
                    // `Self::edges`' answer.
                    sector_left: (seams && sector_left(position, sector_seam_spacing))
                        .then_some(sector_seam),
                    sector_top: (seams && sector_top(position, sector_seam_spacing))
                        .then_some(sector_seam),
                    // A Cell's own content when it has one; the space
                    // otherwise, which `place_glyphs` (`console::shapes`) draws no
                    // Glyph for. An empty claimed operand Cell answers the same
                    // space an empty unclaimed one does, and reads as its Token
                    // channel alone (`crate::style::source_paint_visuals`),
                    // never as a spelled letter.
                    character: cell.content().unwrap_or(' '),
                });
            }
        }

        let grid_line = EdgeLine::grid(theme);

        // The Region is the Cursor's own Cell when it spans no more.
        let (outline_width, outline_colour) = if region_spans {
            (theme.region_border_width, theme.region_border)
        } else {
            (theme.cursor_border_width, theme.cursor_border)
        };
        let outline =
            EdgeLine::drawn(EdgeRank::Selection, outline_width, outline_colour).map(|line| {
                Outline {
                    line,
                    columns: region.columns(),
                    rows: region.rows(),
                }
            });

        Self {
            grid,
            drawn: drawn.clone(),
            outline,
            grid_line,
            sector_seam_width,
            cells,
        }
    }

    ///
    /// [`Self::derive_with_theme`] at the dark default Theme, with an
    /// otherwise-unset Cursor fill overridden to a visible colour so a test
    /// asking about Cursor precedence need not build its own Theme to see
    /// one — `.scratch/theming/schema.md`'s optional-fill default leaves
    /// `cursor.background` unset, which every test in this module but the
    /// ones about that default itself would otherwise have to work around.
    ///
    /// Compiled only for tests, like [`Self::at`], so it is not a shipped
    /// seam: no console path derives a Paint from a test-only Theme, because
    /// `show_source` always holds the console's own resolved one. What this
    /// saves is a test having to build a Theme to ask a question about one
    /// Cell. `console/benches/paint.rs`, a separate crate that never sees
    /// this, walks the dark default through `source_shapes::dark_default_theme`.
    ///
    #[cfg(test)]
    pub(crate) fn derive(input: FramePaint<'_>) -> Self {
        Self::derive_with_theme(input, &test_theme())
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
    /// The Positions this Paint covers.
    ///
    pub(crate) fn drawn(&self) -> &VisiblePositions {
        &self.drawn
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

    ///
    /// Every edge of the drawn Cells, each showing the line of the
    /// highest-ranked fact either Cell beside it carries, or the Grid line,
    /// coalesced into runs along each row line and column line.
    ///
    /// Derived on demand, as [`Self::background_runs`] is. An edge on the
    /// drawn range's boundary has one drawn Cell beside it, and takes that
    /// Cell's fact alone. Of two facts of one rank, the Cell below or right of
    /// the edge draws it. The selection outline outranks every fact, and
    /// where its border's width is zero its edges fall to the facts beside
    /// them. A Sector Seam, carried on the left or top edge of the Cell it
    /// starts, is composited over the line that won every edge but the
    /// outline's.
    ///
    pub fn edges(&self) -> Edges {
        let outline = self.outline.as_ref();
        let (columns, rows) = (self.drawn.columns.len(), self.drawn.rows.len());
        let (first_column, first_row) = (self.drawn.columns.start, self.drawn.rows.start);
        let cell = |column: Option<usize>, row: Option<usize>| {
            let (column, row) = (column?, row?);
            (column < columns && row < rows).then(|| &self.cells[row * columns + column])
        };
        // The outline owns its edges outright, so neither a fact beside it
        // nor a seam lies there. Elsewhere a seam lies over the line that won
        // the edge, at the wider of the two widths, and stands alone where no
        // line does. `max_by_key` answers the last of equal maxima, so of two
        // facts of one rank `other`, the Cell below or right, draws the edge.
        let winner = |outline: Option<&Outline>,
                      one: Option<&CellPaint>,
                      other: Option<&CellPaint>,
                      seam: Option<Color32>| {
            if let Some(outline) = outline {
                return Some(outline.line);
            }
            let line = [
                one.and_then(|cell| cell.border),
                other.and_then(|cell| cell.border),
                self.grid_line,
            ]
            .into_iter()
            .flatten()
            .max_by_key(|line| line.rank);
            let Some(seam) = seam else {
                return line;
            };
            Some(match line {
                Some(line) => EdgeLine {
                    rank: line.rank,
                    stroke: Stroke::new(
                        line.stroke.width.max(self.sector_seam_width),
                        line.stroke.color.blend(seam),
                    ),
                    seamed: true,
                },
                None => EdgeLine {
                    rank: EdgeRank::GridLine,
                    stroke: Stroke::new(self.sector_seam_width, seam),
                    seamed: true,
                },
            })
        };

        let mut edges = Edges::default();
        if columns == 0 || rows == 0 {
            return edges;
        }
        for row in 0..=rows {
            let line = (0..columns).map(|column| {
                let below = cell(Some(column), Some(row));
                winner(
                    outline
                        .filter(|outline| outline.across(first_row + row, first_column + column)),
                    cell(Some(column), row.checked_sub(1)),
                    below,
                    below.and_then(|cell| cell.sector_top),
                )
            });
            coalesce(&mut edges.rows, first_row + row, first_column, line);
        }
        for column in 0..=columns {
            let line = (0..rows).map(|row| {
                let right = cell(Some(column), Some(row));
                winner(
                    outline.filter(|outline| outline.down(first_column + column, first_row + row)),
                    cell(column.checked_sub(1), Some(row)),
                    right,
                    right.and_then(|cell| cell.sector_left),
                )
            });
            coalesce(&mut edges.columns, first_column + column, first_row, line);
        }
        edges
    }
}

///
/// Appends the runs of equal lines along one boundary, `line`, whose edges
/// start at `first` and answer `edges` in order; an edge with no line ends a
/// run and opens none.
///
fn coalesce(
    runs: &mut Vec<EdgeRun>,
    line: usize,
    first: usize,
    edges: impl Iterator<Item = Option<EdgeLine>>,
) {
    let mut open: Option<EdgeRun> = None;
    for (offset, edge) in edges.enumerate() {
        let at = first + offset;
        match (&mut open, edge) {
            (Some(run), Some(edge)) if run.edge == edge => run.span.end = at + 1,
            (_, edge) => {
                runs.extend(open.take());
                open = edge.map(|edge| EdgeRun {
                    line,
                    span: at..at + 1,
                    edge,
                });
            }
        }
    }
    runs.extend(open);
}

///
/// [`Paint::derive`]'s own Theme: the dark default with its otherwise unset
/// `cursor.background` given a visible colour, so a test that asks about
/// Cursor precedence sees one without building a Theme of its own. Not a
/// shipped seam — see [`Paint::derive`]'s own doc.
///
#[cfg(test)]
fn test_theme() -> Theme {
    let base = crate::theme_registry::tests_support::default_theme(crate::theme::Appearance::Dark);
    Theme {
        cursor_background: Some(base.selection_background),
        ..base
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackgroundRun, CellPaint, EdgeLine, EdgeRank, EdgeRun, FramePaint, Outline, Paint, Theme,
        test_theme,
    };
    use crate::grid_viewport::VisiblePositions;
    use crate::marks::{sector_left, sector_top};
    use crate::style::{CellColours, CursorPlacement};
    use crate::theme::{Appearance, GridWidth};
    use crate::theme_registry::tests_support::{dark, default_theme, id};
    use egui::{Color32, Stroke};
    use orcvs::source::{OperandState, SourcePaint, Token};
    use orcvs::{
        app::Orcvs,
        grid::Grid,
        render_frame::{PortalRole, RenderFrame},
    };
    use std::ops::Range;

    fn running_orcvs(cols: usize, rows: usize) -> Orcvs {
        Orcvs::with_shape(cols, rows).expect("the test runtime")
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

    /// The background a role's own foreground `colour` composites to, read
    /// back from `theme` rather than recomputed: `.scratch/theming/schema.md`
    /// stores a background per role, so what a test asks is which stored value a
    /// Cell answers, not a mix this layer would have to reimplement to
    /// check. Every foreground `CellColours::colours` can answer
    /// is distinct in the dark default.
    fn tinted(theme: &Theme, colour: Color32) -> Color32 {
        for (foreground, background) in [
            (theme.source_function, theme.source_function_background),
            (theme.source_number, theme.source_number_background),
            (theme.source_note, theme.source_note_background),
        ] {
            if foreground == colour {
                return background;
            }
        }
        panic!("tinted: {colour:?} is not one of the Theme's tinted role foregrounds")
    }

    ///
    /// A Paint over every Position of a Render Frame's Grid.
    ///
    /// This is the range a fitted console hands `derive` —
    /// `GridViewport::visible_positions` answers exactly it when the whole
    /// Grid is on screen, which `console::tests` asserts through a console pass.
    /// Stated directly here so that a test about what colour a Cell is still
    /// needs no viewport to say "all of them".
    ///
    fn whole(frame: &RenderFrame) -> Paint {
        Paint::derive(FramePaint::whole(frame))
    }

    ///
    /// What `CellColours::colours` answers for `cell` at
    /// `CursorPlacement::Plain`, reading its claim, its own Portal roles and
    /// its occupancy straight from `cell` — the same three per-Cell inputs
    /// `Paint::derive_with_theme` reads, so a test comparing against this
    /// needs no Theme-specific shim.
    ///
    fn expected_visuals(
        cell: &orcvs::render_frame::RenderCell,
        theme: &Theme,
    ) -> crate::style::CellVisuals {
        CellColours::new(theme).colours(
            cell.source_paint(),
            cell.portal_roles(),
            cell.content().is_some(),
            CursorPlacement::Plain,
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
        // The expected answer is each Cell's own visuals, so the Cursor's
        // glyph colour is cleared: it would recolour the selected Cell.
        let theme = Theme {
            cursor_foreground: None,
            ..default_theme(Appearance::Dark)
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let mut borders = std::collections::BTreeSet::new();
        let mut foregrounds = std::collections::BTreeSet::new();
        for cell in frame.cells() {
            let position = cell.position();
            let visuals = expected_visuals(cell, &theme);
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
            borders.insert(painted.border.map(|border| border.stroke.color.to_array()));
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
    /// The Cursor's outline encloses the selected Position, as one extent the
    /// Paint answers rather than a flag to be found by searching the Cells.
    ///
    #[tokio::test]
    async fn the_cursor_outline_encloses_the_selected_position() {
        let mut orcvs = running_orcvs(9, 5);
        let selected = orcvs.grid().position(6, 3).expect("inside the grid");
        orcvs.select(selected);

        let outline = whole(&orcvs.render_frame()).outline.expect("an outline");

        assert_eq!((outline.columns, outline.rows), (6..7, 3..4));
    }

    ///
    /// A Region larger than one Cell fills every Cell it covers but the
    /// Cursor's with the Theme's Region colour, and gives no Cell of it, the
    /// Cursor's included, a border of its own, so the Grid line shows: the
    /// lasso around the Region is the Cursor's border then. A Region of one
    /// Cell fills none.
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

        let painted = |orcvs: &Orcvs, cursor: Option<Color32>, region_cursor: Option<Color32>| {
            let theme = Theme {
                cursor_background: cursor,
                region_background: fill,
                region_cursor_background: region_cursor,
                ..default_theme(Appearance::Dark)
            };
            Paint::derive_with_theme(FramePaint::whole(&orcvs.render_frame()), &theme)
        };

        orcvs.select(at(1, 1));
        let collapsed = painted(&orcvs, None, Some(own));
        assert!(
            collapsed.cells().all(|(_, cell)| cell.background.is_none()),
            "a Region of one Cell was tinted"
        );
        assert!(!outline_spans(&collapsed));

        orcvs.extend(at(3, 2));
        for (cursor, region_cursor, expected) in [
            (None, None, None),
            (Some(cursor_fill), None, Some(cursor_fill)),
            (Some(cursor_fill), Some(own), Some(own)),
        ] {
            let spanned = painted(&orcvs, cursor, region_cursor);
            assert!(outline_spans(&spanned));
            for (position, cell) in spanned.cells() {
                let inside = (1..4).contains(&position.x()) && (1..3).contains(&position.y());
                let wanted = if position == at(3, 2) {
                    expected
                } else {
                    inside.then_some(fill)
                };
                assert_eq!(cell.background, wanted, "the background at {position:?}");
                assert_eq!(cell.border, None, "the border at {position:?}");
            }
        }
        assert_eq!(
            painted(&orcvs, None, None).background_runs(),
            vec![run(fill, 1, 1..4), run(fill, 2, 1..3)]
        );
    }

    ///
    /// The selection outline is the Cursor's Cell in `cursor.border` at
    /// `cursor.border.width`, or a spanning Region in `region.border` at
    /// `region.border.width`. The Cursor's own Cell inside that Region keeps
    /// its own border, ranked just below the outline. Neither Cell's border
    /// is `selection.border`.
    ///
    #[tokio::test]
    async fn the_selection_outline_is_the_cursor_or_the_spanning_region() {
        let mut orcvs = running_orcvs(6, 4);
        write_row(&mut orcvs, 1, ".+");
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        // The Output Portal is given a visible border, so the Cursor's Cell
        // has a fact of its own to keep.
        let theme = Theme {
            output_portal_border_width: GridWidth::from_points(1.0).expect("within 0..=1"),
            ..default_theme(Appearance::Dark)
        };
        let painted = |orcvs: &Orcvs| {
            Paint::derive_with_theme(FramePaint::whole(&orcvs.render_frame()), &theme)
        };
        let portal = painted(&orcvs).at(at(1, 2)).border;
        assert!(
            portal.is_some(),
            "`.+` reserves its Output Portal below `+`"
        );

        orcvs.select(at(1, 2));
        let single = painted(&orcvs);
        assert_eq!(
            single.outline,
            Some(Outline {
                line: EdgeLine {
                    rank: EdgeRank::Selection,
                    stroke: Stroke::new(theme.cursor_border_width.points(), theme.cursor_border),
                    seamed: false,
                },
                columns: 1..2,
                rows: 2..3,
            })
        );
        assert_eq!(
            single.at(at(1, 2)).border,
            portal,
            "the Cursor's Cell keeps its fact"
        );

        orcvs.select(at(0, 0));
        orcvs.extend(at(1, 2));
        let spanning = painted(&orcvs);
        assert_eq!(
            spanning.outline,
            Some(Outline {
                line: EdgeLine {
                    rank: EdgeRank::Selection,
                    stroke: Stroke::new(theme.region_border_width.points(), theme.region_border),
                    seamed: false,
                },
                columns: 0..2,
                rows: 0..3,
            })
        );
        assert_eq!(
            spanning.at(at(1, 2)).border,
            portal.map(|line| EdgeLine {
                rank: EdgeRank::RegionCursor,
                ..line
            })
        );
    }

    ///
    /// A single-Cell Cursor with no fill of its own paints no fill, whatever
    /// the Region fill: only a Region spanning more than one Cell takes it.
    ///
    #[tokio::test]
    async fn a_single_cell_cursor_with_no_fill_takes_no_region_fill() {
        let mut orcvs = running_orcvs(4, 3);
        let at = orcvs.grid().position(1, 1).expect("inside the grid");
        orcvs.select(at);
        let theme = Theme {
            cursor_background: None,
            cell_background: Color32::TRANSPARENT,
            region_background: Color32::from_rgb(7, 8, 9),
            ..default_theme(Appearance::Dark)
        };

        let paint = Paint::derive_with_theme(FramePaint::whole(&orcvs.render_frame()), &theme);

        assert_eq!(paint.at(at).background, None);
    }

    ///
    /// A Cursor or Region border of width zero withdraws the outline, and
    /// its edges fall to the facts and Grid line beneath it.
    ///
    #[tokio::test]
    async fn a_zero_width_outline_withdraws_to_the_edges_beneath_it() {
        let mut orcvs = running_orcvs(4, 3);
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        let zero = crate::theme::GridWidth::from_points(0.0).expect("zero is a width");
        let theme = Theme {
            cursor_border_width: zero,
            region_border_width: zero,
            ..default_theme(Appearance::Dark)
        };
        let painted = |orcvs: &Orcvs| {
            Paint::derive_with_theme(FramePaint::whole(&orcvs.render_frame()), &theme)
        };

        orcvs.select(at(1, 1));
        let single = painted(&orcvs);
        assert_eq!(single.outline, None);
        assert!(
            single
                .edges()
                .columns
                .iter()
                .chain(&single.edges().rows)
                .all(|run| run.edge.rank == EdgeRank::GridLine),
            "a blank Grid with no outline shows only Grid lines"
        );

        orcvs.extend(at(2, 2));
        assert_eq!(painted(&orcvs).outline, None);
    }

    ///
    /// A custom Theme document's `region.cursor.background` in each of its
    /// three states, painted for the Cursor inside a Region larger than one
    /// Cell. Omitted, it inherits the parent's absent fill and `none` clears
    /// it; both fall back to the ordinary Cursor fill. An explicit transparent
    /// colour is a supplied fill and suppresses that fallback, leaving only
    /// the uniform `cell.background` beneath it.
    ///
    #[tokio::test]
    async fn an_explicit_transparent_region_cursor_fill_suppresses_the_cursor_fill_fallback() {
        let mut orcvs = running_orcvs(6, 4);
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        orcvs.select(at(1, 1));
        orcvs.extend(at(3, 2));
        let frame = orcvs.render_frame();
        let cursor_fill = Color32::from_rgb(1, 2, 3);
        let base = Color32::from_rgba_unmultiplied(10, 20, 30, 128);

        let painted = |region_cursor: Option<&str>, cell_background: &str| {
            let mut text = format!(
                "{}[style]\n\
                 \"cursor.background\" = \"#010203\"\n\
                 \"cell.background\" = \"{cell_background}\"\n",
                dark("Custom")
            );
            if let Some(value) = region_cursor {
                text.push_str(&format!("\"region.cursor.background\" = \"{value}\"\n"));
            }
            let document = crate::theme_document::decode("custom.toml", text.as_bytes())
                .expect("a valid document");
            // The parent leaves the Region's Cursor unfilled, so omitting
            // the key inherits an absent fill.
            let parent = Theme {
                region_cursor_background: None,
                ..default_theme(Appearance::Dark)
            };
            let theme = crate::theme_document::resolve(&[parent], &id("custom"), &document)
                .expect("resolves");
            let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
            assert!(outline_spans(&paint));
            paint.at(at(3, 2)).background
        };

        for (region_cursor, cell_background, expected) in [
            (None, "#00000000", Some(cursor_fill)),
            (Some("none"), "#00000000", Some(cursor_fill)),
            (Some("#FFFFFF00"), "#00000000", None),
            (None, "#0A141E80", Some(base.blend(cursor_fill))),
            (Some("none"), "#0A141E80", Some(base.blend(cursor_fill))),
            (Some("#FFFFFF00"), "#0A141E80", Some(base)),
        ] {
            assert_eq!(
                painted(region_cursor, cell_background),
                expected,
                "the Region's Cursor with region.cursor.background {region_cursor:?} \
                 over cell.background {cell_background}"
            );
        }
    }

    ///
    /// "Other Region Cells use Region fill only when no visible effective
    /// fact fill exists after role, Diagnostic and Portal composition"
    /// (`.scratch/theming/schema.md`). A Function's own two-Cell spelling and
    /// the Number operands beside it keep their own role backgrounds inside a
    /// Region that spans them; the blank row above, which claims nothing,
    /// falls back to the Region fill. The Function sits on the *last* row so
    /// its Output Portal Reservation — the row south of a scalar-only
    /// Function's anchor — has no row to land on and cannot
    /// be mistaken for "claims nothing" on a row this test never asked about.
    ///
    #[tokio::test]
    async fn region_role_backgrounds_win_over_the_region_fill() {
        let mut orcvs = running_orcvs(6, 2);
        write_row(&mut orcvs, 1, ".+0102");
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        // The Region spans both rows; the Cursor lands on the blank row's
        // far corner, away from every role-bearing Cell of row 1.
        orcvs.select(at(0, 1));
        orcvs.extend(at(5, 0));

        let frame = orcvs.render_frame();
        // The Region's Cursor is left unfilled, so its Cell has no fill of
        // its own Theme value to answer.
        let theme = Theme {
            region_cursor_background: None,
            ..default_theme(Appearance::Dark)
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);

        assert!(outline_spans(&paint));
        assert_eq!(frame.cursor(), at(5, 0));

        // Function's own two-Cell spelling: `.+`.
        for x in 0..2 {
            assert_eq!(
                paint.at(at(x, 1)).background,
                Some(theme.source_function_background),
                "Function Cell {x} took the Region fill instead of its own tint"
            );
        }
        // The two Number operand entries: `01` and `02`.
        for x in 2..6 {
            assert_eq!(
                paint.at(at(x, 1)).background,
                Some(theme.source_number_background),
                "Number operand Cell {x} took the Region fill instead of its own tint"
            );
        }
        // The blank row above claims nothing, so every Cell but the
        // Cursor's own falls back to the Region fill.
        for x in 0..5 {
            assert_eq!(
                paint.at(at(x, 0)).background,
                Some(theme.region_background),
                "blank Region Cell {x} of row 0 did not fall back to the Region fill"
            );
        }
        // The Cursor's own Cell inside the spanning Region: no fill of its
        // own theme value, and Region fallback never applies to it either —
        // `derive_with_theme`'s `is_cursor && region_spans` branch answers it
        // outright.
        assert_eq!(paint.at(at(5, 0)).background, None);
    }

    ///
    /// `cell.background` "does not count as a fact fill when evaluating
    /// Region fallback, so setting it cannot suppress Region highlighting"
    /// (`.scratch/theming/schema.md`) — proven here with a `cell_background`
    /// that is opaque, not the built-in's transparent default, so the
    /// distinction between "no role contributed" and "the uniform base alone
    /// is visible" cannot be collapsed by coincidence. Every Cell of the
    /// Grid is checked, the same way `a_region_larger_than_one_cell_is_
    /// filled_around_an_independently_coloured_cursor` above checks every
    /// Cell rather than a hand-picked few, so a mistaken corner cannot slip
    /// past unnoticed.
    ///
    #[tokio::test]
    async fn region_fallback_applies_over_a_nontransparent_cell_background() {
        let mut orcvs = running_orcvs(6, 4);
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        let base = Color32::from_rgba_unmultiplied(10, 20, 30, 255);
        // The Region's Cursor is left unfilled, so its Cell shows the base
        // alone.
        let theme = Theme {
            cell_background: base,
            region_cursor_background: None,
            ..default_theme(Appearance::Dark)
        };
        let composed_region_fill = base.blend(theme.region_background);
        assert_ne!(
            composed_region_fill, base,
            "the fixture's Region fill must actually change the composite"
        );

        orcvs.select(at(1, 1));
        orcvs.extend(at(3, 2));
        let cursor = at(3, 2);

        let frame = orcvs.render_frame();
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);

        assert!(outline_spans(&paint));
        assert_eq!(frame.cursor(), cursor);

        for (position, cell) in paint.cells() {
            let inside = (1..4).contains(&position.x()) && (1..3).contains(&position.y());
            let wanted = if position == cursor {
                // The Cursor's own Cell inside a spanning Region: no fill of
                // its own Theme value, composited over the opaque base.
                Some(base)
            } else if inside {
                Some(composed_region_fill)
            } else {
                Some(base)
            };
            assert_eq!(cell.background, wanted, "the background at {position:?}");
        }
    }

    ///
    /// `cell.background` at transparent, partial-alpha and opaque values,
    /// composited beneath an ordinary Cell (no role fill — the `base_fill`
    /// fallback above), a tinted Cell (a role background composited over it)
    /// and a selected Cell outside a Region (the Cursor's own fill
    /// composited over it).
    /// `region_fallback_applies_over_a_nontransparent_cell_background` above
    /// already sweeps an *opaque* base across a Region; this sweeps every
    /// alpha level and adds a tinted Cell and a non-Region selected Cell,
    /// which that fixture's blank, single-Cell-Cursor-free Grid has neither
    /// of. Both the role background and the Cursor fill are themselves given
    /// partial alpha, so an opaque top layer cannot make the comparison pass
    /// by coincidence (an opaque colour wins outright over any base,
    /// `Color32::blend`, so it would never expose a base that was not
    /// actually composited under it).
    ///
    #[tokio::test]
    async fn cell_background_composites_beneath_ordinary_tinted_and_selected_cells_at_every_alpha()
    {
        let mut orcvs = running_orcvs(6, 3);
        write_row(&mut orcvs, 0, ".+0102");
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        // The Function's own two-Cell spelling of row 0.
        let function = at(0, 0);
        let ordinary = at(4, 2);
        let selected = at(5, 2);
        orcvs.select(selected);
        let cursor_fill = Color32::from_rgba_unmultiplied(9, 8, 7, 111);
        let role_fill = Color32::from_rgba_unmultiplied(200, 100, 50, 128);

        for base in [
            Color32::TRANSPARENT,
            Color32::from_rgba_unmultiplied(10, 20, 30, 128),
            Color32::from_rgba_unmultiplied(10, 20, 30, 255),
        ] {
            let theme = Theme {
                cell_background: base,
                cursor_background: Some(cursor_fill),
                source_function_background: role_fill,
                ..default_theme(Appearance::Dark)
            };
            let frame = orcvs.render_frame();
            let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);

            assert!(!outline_spans(&paint), "a single-Cell Cursor never spans");
            assert_eq!(frame.cursor(), selected);

            assert_eq!(
                paint.at(ordinary).background,
                (base.a() != 0).then_some(base),
                "an ordinary Cell at cell.background alpha {}",
                base.a()
            );
            assert_eq!(
                paint.at(function).background,
                Some(base.blend(role_fill)),
                "a tinted Cell at cell.background alpha {}",
                base.a()
            );
            assert_eq!(
                paint.at(selected).background,
                Some(base.blend(cursor_fill)),
                "a selected Cell at cell.background alpha {}",
                base.a()
            );
        }
    }

    ///
    /// Sector seams stand on every Cell Paint derives them for, the Cursor's
    /// included, in the Theme's `sector.seam`.
    /// The Cursor's outline owns its Cell's edges, so neither of its seams
    /// shows there.
    ///
    /// The Cursor is put on a Cell that wants a seam — the corner of a
    /// sector — because a Cursor parked where no seam was wanted would
    /// prove nothing.
    ///
    /// Everywhere the colour is the Theme's own `sector.seam`.
    ///
    #[tokio::test]
    async fn seams_stand_where_paint_asks_and_the_outline_owns_its_edges() {
        let mut orcvs = running_orcvs(24, 24);
        // A sector corner at the default Sector Seam spacing of eight.
        let corner = orcvs.grid().position(8, 8).expect("inside the grid");
        orcvs.select(corner);

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let spacing = frame.sector_seam_spacing().cells();
        let cursor = frame.cursor();
        // `whole` derives against `test_theme`, so its `sector_seam` is every
        // seam's colour.
        let theme = test_theme();
        let base = theme.sector_seam;
        let mut seams = 0;

        for cell in frame.cells() {
            let position = cell.position();
            let painted = paint.at(position);
            let left = sector_left(position, spacing);
            let top = sector_top(position, spacing);

            if position == cursor {
                assert!(
                    left && top,
                    "the Cursor must sit where seams are wanted for their absence to mean anything"
                );
            }
            assert_eq!(
                painted.sector_left,
                left.then_some(base),
                "the left seam at {position:?}"
            );
            assert_eq!(
                painted.sector_top,
                top.then_some(base),
                "the top seam at {position:?}"
            );
            seams += usize::from(painted.sector_left.is_some());
            seams += usize::from(painted.sector_top.is_some());
        }

        assert!(seams > 0, "no Cell was asked for a seam");

        let outline = EdgeLine {
            rank: EdgeRank::Selection,
            stroke: Stroke::new(theme.cursor_border_width.points(), theme.cursor_border),
            seamed: false,
        };
        let edges = paint.edges();
        let on = |runs: &[EdgeRun], line: usize, at: usize| {
            runs.iter()
                .find(|run| run.line == line && run.span.contains(&at))
                .map(|run| run.edge)
        };
        assert_eq!(
            on(&edges.columns, 8, 8),
            Some(outline),
            "the Cursor's left edge"
        );
        assert_eq!(
            on(&edges.rows, 8, 8),
            Some(outline),
            "the Cursor's top edge"
        );
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

        let left = sector_left(at(8, 8), spacing);
        let top = sector_top(at(8, 8), spacing);
        let base = test_theme().sector_seam;
        assert!(left && top, "the Cursor is off a sector corner");
        assert_eq!(painted.sector_left, left.then_some(base));
        assert_eq!(painted.sector_top, top.then_some(base));
    }

    ///
    /// Each Cell shows exactly its own content, or the space when it holds
    /// none — never a placeholder letter standing in for its Token.
    ///
    /// What a Cell shows is answered from the Render Frame alone, with no Token
    /// lookup at all. The Grid carries an Addition, whose claim reaches past
    /// the two Cells it is spelled in and leaves classified but empty operand
    /// Cells behind it — this is the Cell this test is about, and it answers
    /// the space like any other empty Cell, not a letter spelled for its Token.
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
    /// Number and Note — shows no character at all: the tint
    /// `style::source_paint_visuals` paints is the whole of what marks it as
    /// Pending, and the space it shows is the same one an empty unclaimed
    /// Cell always has. `Token::Char` is not one of the Tokens exercised
    /// here: `style::source_paint_visuals`'s own doc explains why no operand ever
    /// declares it, so there is no empty *claimed* Char Cell to write this
    /// test against — a Leftover Char is never empty, since content is what
    /// makes it Char at all.
    ///
    /// Number and Note come from an Addition (`.+`, two Number operands) and
    /// a `.v` (one Note operand) left unfilled.
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
        let theme = default_theme(Appearance::Dark);

        // The Addition's two Number operands, columns 2-5 of row 0.
        for x in 2..6 {
            let position = orcvs.grid().position(x, 0).expect("inside the grid");
            let cell = frame.at(position);
            assert_eq!(cell.content(), None, "operand Cell {x} was not empty");
            assert_eq!(
                cell.source_paint(),
                SourcePaint::Operand {
                    token: Token::Number,
                    state: OperandState::Pending,
                }
            );
            let painted = paint.at(position);
            assert_eq!(painted.character, ' ', "operand Cell {x} spelled a letter");
            assert_eq!(
                painted.background,
                expected_visuals(cell, &theme).background,
                "operand Cell {x} did not carry the Number tint"
            );
        }

        // `.v`'s one Note operand, columns 2-3 of row 1.
        for x in 2..4 {
            let position = orcvs.grid().position(x, 1).expect("inside the grid");
            let cell = frame.at(position);
            assert_eq!(cell.content(), None, "operand Cell {x} was not empty");
            assert_eq!(
                cell.source_paint(),
                SourcePaint::Operand {
                    token: Token::Note,
                    state: OperandState::Pending,
                }
            );
            let painted = paint.at(position);
            assert_eq!(painted.character, ' ', "operand Cell {x} spelled a letter");
            assert_eq!(
                painted.background,
                expected_visuals(cell, &theme).background,
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
        let theme = default_theme(Appearance::Dark);
        let position = orcvs.grid().position(4, 0).expect("inside the grid");
        let cell = frame.at(position);

        assert_eq!(cell.content(), None, "the truncated Cell was not empty");
        assert_eq!(
            cell.source_paint(),
            SourcePaint::Operand {
                token: Token::Number,
                state: OperandState::Pending,
            },
            "the truncated Cell did not carry the operand's declared Token"
        );
        let painted = paint.at(position);
        assert_eq!(
            painted.character, ' ',
            "the truncated Cell spelled a letter"
        );
        assert_eq!(
            painted.background,
            expected_visuals(cell, &theme).background,
            "the truncated Cell did not carry the Number tint"
        );
    }

    ///
    /// A Pending operand Cell of every Token a signature can declare draws no
    /// glyph, through the real paint path — `console::shapes`'s `place_glyphs` skips
    /// a Cell whose `character` is `' '`, and a Pending slot's Cells are
    /// never anything else, since a Pending slot is blank by definition.
    /// The contrast validator relies on this: a Pending Cell has no foreground
    /// for it to measure, so `contrast::Role::SOURCE_ROLES` carries no Pending
    /// fact. If this test ever fails — a Pending Cell starts drawing a character
    /// — add Pending states to `contrast::Role::SOURCE_ROLES`, since the "no
    /// foreground to measure" reasoning that leaves them out would no longer
    /// hold.
    ///
    #[tokio::test]
    async fn every_pending_operand_token_draws_no_glyph_through_the_real_paint_path() {
        let mut orcvs = running_orcvs(6, 2);
        for (row, text) in [(0, ".+"), (1, ".v")] {
            write_row(&mut orcvs, row, text);
        }
        orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let grid = orcvs.grid();

        for (row, operands, token) in [(0, 2..6, Token::Number), (1, 2..4, Token::Note)] {
            for x in operands {
                let position = grid.position(x, row).expect("inside the grid");
                let cell = frame.at(position);
                assert_eq!(
                    cell.source_paint(),
                    SourcePaint::Operand {
                        token,
                        state: OperandState::Pending,
                    },
                    "row {row} column {x} was not the Pending fixture this test expects"
                );
                assert_eq!(
                    cell.content(),
                    None,
                    "row {row} column {x}'s Pending slot held content"
                );

                let painted = paint.at(position);
                assert_eq!(
                    painted.character, ' ',
                    "row {row} column {x} ({token:?}) drew a glyph despite being Pending — \
                     restore Pending states to contrast::Role::SOURCE_ROLES, since measuring a \
                     Pending foreground is no longer an impossible state"
                );
            }
        }
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
        let theme = default_theme(Appearance::Dark);

        for x in [0, 1, 3, 4, 6] {
            let position = orcvs.grid().position(x, 0).expect("inside the grid");
            assert_eq!(
                frame.at(position).source_paint(),
                SourcePaint::Unclaimed,
                "column {x} was not a Function claim that bound nothing"
            );
            let painted = paint.at(position);
            assert_eq!(painted.foreground, theme.source_ordinary, "column {x}");
            assert_eq!(painted.background, None, "column {x}");
        }

        for x in 2..4 {
            let position = orcvs.grid().position(x, 1).expect("inside the grid");
            let painted = paint.at(position);
            assert_eq!(
                painted.foreground, theme.diagnostic_foreground,
                "column {x}"
            );
            assert_eq!(
                painted.background,
                expected_visuals(frame.at(position), &theme).background,
                "column {x} did not keep the Number tint"
            );
            assert!(painted.background.is_some(), "column {x} lost its tint");
        }
    }

    ///
    /// `**` and `||` written into a typed operand slot are read as that slot's
    /// declared Token and refused. Neither is a nested Bang or a Comment at an
    /// operand position — the Parser only asks whether a Function comes next
    /// where an Expression could start — so both record an unbound
    /// `Token::Number` entry, keep the declared Number tint, and draw their
    /// glyphs in the Diagnostic colour. The valid operand beside each is
    /// unaffected: Number on the same Number tint.
    ///
    /// All four operand Cells of each Expression are asserted, because a
    /// rule stated only over the rejected pair cannot say that the operand
    /// beside it was left alone. The tint is read as one answer shared by the
    /// four Cells and distinct from `.+`'s own Function tint, rather than as
    /// a colour restated from the Theme.
    ///
    /// The two Expressions sit two rows apart so that neither one's Output
    /// Portal Reservation — row 1 for the first, row 3 for the second, both
    /// two Cells wide at columns 0-1 — reaches a Cell asserted here:
    /// Output Portal paint takes precedence over the claim's own, so
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
        let theme = default_theme(Appearance::Dark);

        for (row, operands) in [(0, "**01"), (2, "||02")] {
            let mut tints = Vec::new();

            for (offset, character) in operands.chars().enumerate() {
                // The two operands of `.+` stand at columns 2-3 and 4-5; the
                // first two characters spell the rejected one.
                let x = offset + 2;
                let rejected = offset < 2;
                let position = orcvs.grid().position(x, row).expect("inside the grid");
                assert_eq!(
                    frame.at(position).source_paint(),
                    SourcePaint::Operand {
                        token: Token::Number,
                        state: if rejected {
                            OperandState::Invalid
                        } else {
                            OperandState::Valid
                        },
                    },
                    "row {row} column {x} did not bind the way its operand is written"
                );

                let painted = paint.at(position);

                assert_eq!(painted.character, character, "row {row} column {x}");
                assert_eq!(
                    painted.foreground,
                    if rejected {
                        theme.diagnostic_foreground
                    } else {
                        theme.source_number
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
    /// The Cursor's own fill wins outright on its Cell, and an Invalid operand is
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
        // The Cursor's glyph colour is cleared, so its Cell's glyph is the
        // one its operand draws.
        let theme = Theme {
            cursor_foreground: None,
            ..test_theme()
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let selected = paint.at(cursor);
        let unselected = paint.at(orcvs.grid().position(3, 0).expect("inside the grid"));

        assert_eq!(frame.cursor(), cursor);
        assert_eq!(selected.character, '*');
        assert_eq!(
            selected.foreground, theme.diagnostic_foreground,
            "the Cursor's Cell lost the Invalid operand's Diagnostic glyph"
        );
        assert_eq!(
            selected.background,
            Some(theme.selection_background),
            "the Cursor's Cell did not take the Cursor's own fill"
        );
        assert_eq!(
            unselected.foreground, theme.diagnostic_foreground,
            "the unselected half of the same rejected operand"
        );
        assert!(
            unselected.background.is_some() && unselected.background != selected.background,
            "the unselected half did not keep a tint of its own: {:?}",
            unselected.background
        );
    }

    ///
    /// End to end: a written Invalid operand's border composites
    /// `theme.diagnostic_border` over `theme.grid_border`, reaching
    /// `Paint::derive_with_theme` from a real written Source rather than a
    /// hand-built `SourcePaint` fact —
    /// `style::tests::a_diagnostic_border_with_a_non_transparent_colour_shows`
    /// is the unit-level proof this is the same rule; this is the wiring
    /// proof. Reuses `.+**01`'s fixture: `**` (columns 2–3) is the rejected
    /// Number operand, `01` (columns 4–5) the valid one.
    ///
    #[tokio::test]
    async fn a_written_invalid_operand_shows_a_non_transparent_diagnostic_border() {
        let mut orcvs = running_orcvs(8, 2);
        for (x, character) in ".+**01".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        let frame = orcvs.render_frame();
        let visible = Color32::from_rgb(200, 30, 30);
        // The Diagnostic border is given a width, and the Number border
        // none, so the Valid operand beside it shows no border of its own.
        let theme = Theme {
            diagnostic_border: visible,
            diagnostic_border_width: GridWidth::from_points(1.0).expect("within 0..=1"),
            source_number_border_width: GridWidth::from_points(0.0).expect("within 0..=1"),
            ..default_theme(Appearance::Dark)
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);

        let invalid = paint.at(orcvs.grid().position(2, 0).expect("inside the grid"));
        let valid = paint.at(orcvs.grid().position(4, 0).expect("inside the grid"));

        assert_eq!(
            invalid.border.map(|border| border.stroke),
            Some(Stroke::new(
                theme.diagnostic_border_width.points(),
                theme.grid_border.blend(visible)
            ))
        );
        assert_eq!(
            valid.border, None,
            "a Valid operand took the Diagnostic border it has no fact for"
        );
    }

    ///
    /// A visible `source.function.border` strokes both Cells of a written
    /// Function's spelling at `source.function.border.width`, over the Grid
    /// line, and leaves its operands on the plain Grid line.
    ///
    #[tokio::test]
    async fn a_visible_function_border_strokes_the_function_cells_at_its_own_width() {
        let mut orcvs = running_orcvs(8, 2);
        for (x, character) in ".+0102".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        let frame = orcvs.render_frame();
        let visible = Color32::from_rgb(30, 200, 30);
        let theme = Theme {
            source_function_border: visible,
            source_function_border_width: GridWidth::from_points(0.25).expect("within 0..=1"),
            // The Number border is cleared, so the operands show the Grid
            // line alone.
            source_number_border_width: GridWidth::from_points(0.0).expect("within 0..=1"),
            ..default_theme(Appearance::Dark)
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let at = |x| paint.at(orcvs.grid().position(x, 0).expect("inside the grid"));

        for x in [0, 1] {
            assert_eq!(
                at(x).border,
                Some(EdgeLine {
                    rank: EdgeRank::Role,
                    stroke: Stroke::new(0.25, theme.grid_border.blend(visible)),
                    seamed: false,
                }),
                "column {x}"
            );
        }
        assert_eq!(at(2).border, None, "an operand took the Function border");
    }

    ///
    /// The Cursor's own Cell inside a spanning Region keeps a role border at
    /// its role's rank, so it never outranks a neighbour's Diagnostic or
    /// Portal frame.
    ///
    #[tokio::test]
    async fn the_region_cursor_keeps_a_role_border_at_its_role_rank() {
        let mut orcvs = running_orcvs(6, 4);
        write_row(&mut orcvs, 1, ".+");
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        orcvs.select(at(3, 3));
        orcvs.extend(at(0, 1));
        let frame = orcvs.render_frame();
        let theme = Theme {
            source_function_border: Color32::from_rgb(30, 200, 30),
            ..default_theme(Appearance::Dark)
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);

        assert_eq!(
            paint.at(at(0, 1)).border.map(|line| line.rank),
            Some(EdgeRank::Role)
        );
    }

    ///
    /// A visible Bang border keeps all four edges of a `**` against a visible
    /// Function border beside it, though the Function stands right of the
    /// shared edge and would win it on a tie.
    ///
    #[tokio::test]
    async fn a_bang_border_wins_the_edge_it_shares_with_a_function() {
        let mut orcvs = running_orcvs(8, 3);
        write_row(&mut orcvs, 1, "**.+0102");
        orcvs.select(orcvs.grid().position(7, 2).expect("inside the grid"));
        let frame = orcvs.render_frame();
        let width = GridWidth::from_points(1.0).expect("within 0..=1");
        let theme = Theme {
            source_bang_border: Color32::from_rgb(0, 255, 0),
            source_bang_border_width: width,
            source_function_border: Color32::from_rgb(68, 68, 68),
            source_function_border_width: width,
            ..default_theme(Appearance::Dark)
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let bang = paint
            .at(orcvs.grid().position(1, 1).expect("inside the grid"))
            .border
            .expect("a visible Bang border");
        assert_eq!(bang.rank, EdgeRank::Bang);

        let shared = paint
            .edges()
            .columns
            .into_iter()
            .find(|run| run.line == 2 && run.span.contains(&1))
            .expect("the edge between the Bang and the Function");
        assert_eq!(shared.edge, bang);
    }

    ///
    /// A transparent Function border holds no edge, so the Function's Cells
    /// keep the Grid line at `grid.border.width`, whatever the Function
    /// border's own width. A visible one at width zero withdraws as well.
    ///
    #[tokio::test]
    async fn a_transparent_function_border_leaves_the_grid_line_and_its_width() {
        let mut orcvs = running_orcvs(8, 2);
        for (x, character) in ".+0102".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        let frame = orcvs.render_frame();
        let width = |points| GridWidth::from_points(points).expect("within 0..=1");
        let transparent = Theme {
            grid_border_width: width(0.25),
            source_function_border: Color32::TRANSPARENT,
            source_function_border_width: width(1.0),
            ..default_theme(Appearance::Dark)
        };
        let hidden = Theme {
            source_function_border: Color32::from_rgb(30, 200, 30),
            source_function_border_width: width(0.0),
            ..default_theme(Appearance::Dark)
        };
        let function = orcvs.grid().position(0, 0).expect("inside the grid");

        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &transparent);
        assert_eq!(paint.at(function).border, None);

        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &hidden);
        assert_eq!(paint.at(function).border, None);
    }

    ///
    /// A Comment's border frames every Cell the Comment claims, the blank
    /// Cells between its text included, and an empty Cell past the Grid's
    /// written Source keeps the plain Grid line.
    ///
    #[tokio::test]
    async fn a_comment_border_frames_its_blank_cells_as_well_as_its_text() {
        let mut orcvs = running_orcvs(8, 2);
        for (x, character) in [(0, '|'), (1, '|'), (2, 'a'), (4, 'b')] {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        let frame = orcvs.render_frame();
        let visible = Color32::from_rgb(30, 30, 200);
        let theme = Theme {
            source_comment_border: visible,
            ..default_theme(Appearance::Dark)
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let position = |x| orcvs.grid().position(x, 0).expect("inside the grid");

        assert_eq!(frame.at(position(3)).source_paint(), SourcePaint::Comment);
        assert_eq!(frame.at(position(3)).content(), None);
        assert_eq!(
            paint
                .at(position(2))
                .border
                .map(|border| border.stroke.color),
            Some(theme.grid_border.blend(visible))
        );
        assert_eq!(
            paint.at(position(3)).border.map(|border| border.stroke),
            Some(Stroke::new(
                theme.source_comment_border_width.points(),
                theme.grid_border.blend(visible)
            ))
        );
        let below = orcvs.grid().position(3, 1).expect("inside the grid");
        assert_eq!(frame.at(below).source_paint(), SourcePaint::Unclaimed);
        assert_eq!(paint.at(below).border, None);
    }

    ///
    /// A custom Theme document that sets the Function border decodes,
    /// resolves over its parent, and paints that border on a written
    /// Function.
    ///
    #[tokio::test]
    async fn a_theme_document_paints_the_function_border_it_sets() {
        let text = format!(
            "{}[style]\n\"source.function.border\" = \"#80808080\"\n\"source.function.border.width\" = 1\n",
            dark("Framed")
        );
        let document = crate::theme_document::decode("framed.toml", text.as_bytes())
            .expect("a valid Theme document");
        let theme = crate::theme_document::resolve(
            &[default_theme(Appearance::Dark)],
            &id("framed"),
            &document,
        )
        .expect("the document resolves over its parent");

        let mut orcvs = running_orcvs(8, 2);
        for (x, character) in ".+0102".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 0).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        let frame = orcvs.render_frame();
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let function = paint.at(orcvs.grid().position(0, 0).expect("inside the grid"));

        let framed = Color32::from_rgba_unmultiplied(0x80, 0x80, 0x80, 0x80);
        assert_eq!(
            function.border.map(|border| border.stroke),
            Some(Stroke::new(1.0, theme.grid_border.blend(framed)))
        );
    }

    ///
    /// A Function, an Invalid operand and an Output Portal answer, each with
    /// a glyph: `.+**01` on row 0, whose `**` is the rejected Number operand,
    /// and `07` left south of `.+0304` on rows 2 and 3. Answers the Orcvs and
    /// the three Positions, in that order.
    ///
    fn cursor_glyph_sampler() -> (Orcvs, [orcvs::grid::Position; 3]) {
        let mut orcvs = running_orcvs(8, 4);
        write_row(&mut orcvs, 0, ".+**01");
        write_row(&mut orcvs, 2, ".+0304");
        write_row(&mut orcvs, 3, "07");
        let at = |x, y| orcvs.grid().position(x, y).expect("inside the grid");
        let cells = [at(0, 0), at(2, 0), at(0, 3)];
        (orcvs, cells)
    }

    ///
    /// `cursor.foreground` draws the glyph on the single-Cell Cursor over a
    /// Function, an Invalid operand and an Output Portal answer alike, a
    /// transparent one hiding it. Cleared, each glyph keeps its role colour.
    /// The Cursor's fill and frame, and every other Cell, paint as they do
    /// with the glyph colour cleared.
    ///
    #[tokio::test]
    async fn cursor_foreground_draws_the_glyph_on_the_single_cell_cursor() {
        let (mut orcvs, cells) = cursor_glyph_sampler();
        let base = test_theme();
        let roles = [
            base.source_function,
            base.diagnostic_foreground,
            base.output_portal_foreground,
        ];
        let glyph = Color32::from_rgb(1, 200, 3);

        for (cursor, role) in cells.into_iter().zip(roles) {
            orcvs.select(cursor);
            let frame = orcvs.render_frame();
            assert!(frame.at(cursor).content().is_some(), "{cursor:?} is empty");
            let painted = |cursor_foreground| {
                let theme = Theme {
                    cursor_foreground,
                    ..test_theme()
                };
                Paint::derive_with_theme(FramePaint::whole(&frame), &theme)
            };
            let cleared = painted(None);
            assert!(!outline_spans(&cleared));
            assert_eq!(cleared.at(cursor).foreground, role, "{cursor:?} cleared");

            for supplied in [glyph, Color32::TRANSPARENT] {
                let paint = painted(Some(supplied));
                let (own, plain) = (paint.at(cursor), cleared.at(cursor));
                assert_eq!(own.foreground, supplied, "{cursor:?}");
                assert_eq!(own.background, plain.background, "{cursor:?}'s fill");
                assert_eq!(own.border, plain.border, "{cursor:?}'s frame");
                for (position, cell) in paint.cells() {
                    if position != cursor {
                        assert_eq!(cell, cleared.at(position), "{position:?}");
                    }
                }
            }
        }
    }

    ///
    /// A custom Theme document that sets `cursor.foreground` decodes,
    /// resolves over its parent, and paints the glyph under the Cursor in
    /// it, leaving the Cell beside it its role colour.
    ///
    #[tokio::test]
    async fn a_theme_document_paints_the_cursor_foreground_it_sets() {
        let text = format!(
            "{}[style]\n\"cursor.foreground\" = \"#102030\"\n",
            dark("Glyph")
        );
        let document = crate::theme_document::decode("glyph.toml", text.as_bytes())
            .expect("a valid Theme document");
        let theme = crate::theme_document::resolve(
            &[default_theme(Appearance::Dark)],
            &id("glyph"),
            &document,
        )
        .expect("the document resolves over its parent");

        let (mut orcvs, [function, ..]) = cursor_glyph_sampler();
        orcvs.select(function);
        let frame = orcvs.render_frame();
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let beside = orcvs.grid().position(1, 0).expect("inside the grid");

        assert_eq!(
            paint.at(function).foreground,
            Color32::from_rgb(0x10, 0x20, 0x30)
        );
        assert_eq!(paint.at(beside).foreground, theme.source_function);
    }

    ///
    /// While a Region spans more than one Cell, the glyph on the Cursor's own
    /// Cell draws in `region.cursor.foreground` when it holds a colour,
    /// transparent included; otherwise in `cursor.foreground` when that
    /// holds one; otherwise in its role colour. The Region runs from an
    /// Invalid operand back to the Function the Cursor stands on, and its
    /// other Cells, the Cursor Cell's fill and the lasso's grid line paint as
    /// they do with both glyph colours cleared.
    ///
    #[tokio::test]
    async fn the_regions_cursor_glyph_falls_back_to_the_cursor_glyph_then_the_role() {
        let (mut orcvs, [function, invalid, _]) = cursor_glyph_sampler();
        orcvs.select(invalid);
        orcvs.extend(function);
        let frame = orcvs.render_frame();
        let painted = |region_cursor_foreground, cursor_foreground| {
            let theme = Theme {
                region_cursor_background: Some(Color32::from_rgb(4, 5, 6)),
                region_cursor_foreground,
                cursor_foreground,
                ..test_theme()
            };
            Paint::derive_with_theme(FramePaint::whole(&frame), &theme)
        };
        let cleared = painted(None, None);
        assert!(outline_spans(&cleared));
        assert_eq!(frame.cursor(), function);

        let own = Color32::from_rgb(1, 200, 3);
        let cursor = Color32::from_rgb(200, 1, 3);
        for (region_cursor_foreground, cursor_foreground, expected) in [
            (Some(own), Some(cursor), own),
            (Some(own), None, own),
            (None, Some(cursor), cursor),
            (None, None, test_theme().source_function),
            (
                Some(Color32::TRANSPARENT),
                Some(cursor),
                Color32::TRANSPARENT,
            ),
        ] {
            let paint = painted(region_cursor_foreground, cursor_foreground);
            let case = format!(
                "region.cursor.foreground {region_cursor_foreground:?}, \
                 cursor.foreground {cursor_foreground:?}"
            );
            let (glyph, plain) = (paint.at(function), cleared.at(function));
            assert_eq!(glyph.foreground, expected, "{case}");
            assert_eq!(glyph.background, plain.background, "{case}: the fill");
            assert_eq!(glyph.border, plain.border, "{case}: the grid line");
            for (position, cell) in paint.cells() {
                if position != function {
                    assert_eq!(cell, cleared.at(position), "{case}: {position:?}");
                }
            }
        }
    }

    ///
    /// A custom Theme document that sets `region.cursor.foreground` decodes,
    /// resolves over its parent, and paints the glyph on the Region's Cursor
    /// Cell in it.
    ///
    #[tokio::test]
    async fn a_theme_document_paints_the_region_cursor_foreground_it_sets() {
        let text = format!(
            "{}[style]\n\"region.cursor.foreground\" = \"#102030\"\n",
            dark("Glyph")
        );
        let document = crate::theme_document::decode("glyph.toml", text.as_bytes())
            .expect("a valid Theme document");
        let theme = crate::theme_document::resolve(
            &[default_theme(Appearance::Dark)],
            &id("glyph"),
            &document,
        )
        .expect("the document resolves over its parent");

        let (mut orcvs, [function, invalid, _]) = cursor_glyph_sampler();
        orcvs.select(invalid);
        orcvs.extend(function);
        let frame = orcvs.render_frame();
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);

        assert!(outline_spans(&paint));
        assert_eq!(
            paint.at(function).foreground,
            Color32::from_rgb(0x10, 0x20, 0x30)
        );
        assert_eq!(paint.at(invalid).foreground, theme.diagnostic_foreground);
    }

    ///
    /// One Expression per Operand Token and binding state the tint
    /// distinguishes, written from Source: `.+` for a Pending Number, `.vC4`
    /// for a bound Note and `.vXY` for an Invalid Note.
    ///
    /// The three rows stand two apart. Output Portal paint takes precedence
    /// over an operand's own tint, and a root Function's Reservation lands one
    /// row south, so Expressions on adjacent rows would have row `N + 1`'s
    /// operands answering the Output Portal rule instead of their own tint.
    /// Two rows apart puts every Reservation on an empty row, and the tests
    /// below assert no Output Portal role on every Cell they read rather than
    /// trusting this paragraph.
    ///
    /// The Cursor parks at the east end of row 0, which no Expression claims
    /// and no Reservation covers, so the Cursor's own fill cannot stand in
    /// for a tint.
    ///
    fn operand_token_sampler() -> Orcvs {
        let mut orcvs = running_orcvs(8, 6);
        for (row, text) in [(0, ".+"), (2, ".vC4"), (4, ".vXY")] {
            write_row(&mut orcvs, row, text);
        }
        orcvs.select(orcvs.grid().position(7, 0).expect("inside the grid"));
        orcvs
    }

    ///
    /// The Fill tint on an Operand Cell of every Token
    /// a Source can claim, walked from written Source through the Render
    /// Frame rather than from a `Claim` built by hand: Number and Note each
    /// tint with their own colour, and the Function's own two-Cell spelling
    /// beside them tints with Function's.
    ///
    /// The glyph colour is asserted alongside, because the tint alone cannot
    /// say that the right rule produced it: a bound Note draws Note, a
    /// Pending Number draws Number, and `.vXY`'s Invalid Note draws
    /// Diagnostic while keeping the Note tint.
    ///
    /// The three tints are finally required to be three colours. Operand
    /// arms that all answered one tint — every Token reaching the same
    /// colour — would satisfy every assertion above.
    ///
    #[tokio::test]
    async fn an_operand_cell_of_every_token_a_source_can_claim_is_tinted_with_its_own_colour() {
        let orcvs = operand_token_sampler();
        let frame = orcvs.render_frame();
        // The default fills Number and Note alike, so Note is given a fill of
        // its own for the three tints to be told apart.
        let theme = Theme {
            source_note_background: Color32::from_rgb(10, 20, 30),
            ..test_theme()
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let grid = orcvs.grid();
        let mut tints = std::collections::BTreeSet::new();

        for (row, operands, token, state, tint, foreground) in [
            (
                0,
                2..6,
                Token::Number,
                OperandState::Pending,
                theme.source_number_background,
                theme.source_number,
            ),
            (
                2,
                2..4,
                Token::Note,
                OperandState::Valid,
                theme.source_note_background,
                theme.source_note,
            ),
            (
                4,
                2..4,
                Token::Note,
                OperandState::Invalid,
                theme.source_note_background,
                theme.diagnostic_foreground,
            ),
        ] {
            for x in operands {
                let position = grid.position(x, row).expect("inside the grid");
                let cell = frame.at(position);

                assert!(
                    !cell.portal_roles().contains(PortalRole::Output),
                    "row {row} column {x} is inside a Reservation, so `06`'s rule answers it"
                );
                assert_eq!(
                    cell.source_paint(),
                    SourcePaint::Operand { token, state },
                    "row {row} column {x} did not bind the way its operand is written"
                );

                let painted = paint.at(position);
                assert_eq!(
                    painted.background,
                    Some(tint),
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
                    Some(theme.source_function_background),
                    "row {row} column {x} was not tinted with the Function colour"
                );
                assert_eq!(
                    painted.foreground, theme.source_function,
                    "row {row} column {x} did not draw the Function colour"
                );
            }

            tints.insert(tint.to_array());
        }

        tints.insert(theme.source_function_background.to_array());
        assert_eq!(
            tints.len(),
            3,
            "the two Operand tints and the Function tint were not three colours: {tints:?}"
        );
    }

    ///
    /// Track paints as any other Function: `&t0103C4D4E4` and `.+0103C4D4E4`
    /// answer the same claim, tint and glyph colour on every Cell of their
    /// row.
    /// The pairs Track reads east of its operands are ordinary Source, so
    /// they paint as the Cells after Add's operands do, never as an operand.
    ///
    /// The rows stand two apart so each Reservation lands on an empty row,
    /// and the Cursor parks east of both Expressions, on a Cell neither
    /// claims.
    ///
    #[tokio::test]
    async fn track_paints_as_any_other_function() {
        let mut orcvs = running_orcvs(14, 4);
        write_row(&mut orcvs, 0, "&t0103C4D4E4");
        write_row(&mut orcvs, 2, ".+0103C4D4E4");
        orcvs.select(orcvs.grid().position(13, 0).expect("inside the grid"));
        let frame = orcvs.render_frame();
        let paint = whole(&frame);
        let grid = orcvs.grid();
        let at = |x, row| grid.position(x, row).expect("inside the grid");

        for x in 0..12 {
            let (track, add) = (frame.at(at(x, 0)), frame.at(at(x, 2)));
            let expected = match x {
                0..2 => Some(SourcePaint::Function),
                2..6 => Some(SourcePaint::Operand {
                    token: Token::Number,
                    state: OperandState::Valid,
                }),
                _ => None,
            };
            if let Some(expected) = expected {
                assert_eq!(track.source_paint(), expected, "column {x}");
            } else {
                assert!(
                    !matches!(track.source_paint(), SourcePaint::Operand { .. }),
                    "column {x} was claimed as Track's operand"
                );
            }
            assert_eq!(track.source_paint(), add.source_paint(), "column {x}");
            // Tint and glyph colour are what a claim decides; the character
            // is the spelling and the sector marks are the row's place.
            let colours = |row| {
                let painted = paint.at(at(x, row));
                (painted.background, painted.foreground)
            };
            assert_eq!(colours(0), colours(2), "column {x}");
        }
    }

    ///
    /// A Theme whose role backgrounds are all transparent paints no
    /// background anywhere — not on a Function Cell, not on an Operand Cell
    /// of any Token, and not on an Output Portal Cell either, since Output
    /// Portal paint composites the same channel. The assertion is over every
    /// Cell of the Grid rather than over the claimed ones, so a role that
    /// acquired a fill of its own would fail here.
    ///
    /// The same Source under the dark default is the control: each of
    /// the 14 Cells an Expression claims does carry a background there, so
    /// `None` answering every Cell under the transparent Theme is the Theme
    /// doing it and not the fixture having nothing to tint. `None` rather
    /// than `Some(grid_background)` is the point of the rule — a Cell with a
    /// fully transparent composite costs `background_runs` nothing to walk.
    ///
    /// The Cursor's fill is `None` on both Themes here rather than set, so
    /// the Cursor's own Cell has no fill to contribute either; its precedence
    /// over a role's own channel is
    /// `the_cursors_own_fill_wins_over_a_function_cells_tint`'s subject.
    ///
    #[tokio::test]
    async fn a_theme_with_every_role_background_transparent_paints_no_tint_on_any_cell_a_source_claims()
     {
        let orcvs = operand_token_sampler();
        let frame = orcvs.render_frame();
        let grid = orcvs.grid();
        let transparent = Theme {
            source_function_background: Color32::TRANSPARENT,
            source_number_background: Color32::TRANSPARENT,
            source_note_background: Color32::TRANSPARENT,
            output_portal_background: Color32::TRANSPARENT,
            ..default_theme(Appearance::Dark)
        };

        let control =
            Paint::derive_with_theme(FramePaint::whole(&frame), &default_theme(Appearance::Dark));
        let claimed: Vec<_> = frame
            .cells()
            .iter()
            .filter(|cell| cell.source_paint() != SourcePaint::Unclaimed)
            .map(|cell| cell.position())
            .collect();
        assert_eq!(
            claimed.len(),
            14,
            "the fixture stopped claiming what it did"
        );
        for position in &claimed {
            assert!(
                control.at(*position).background.is_some(),
                "{position:?} had nothing to tint under the dark default"
            );
        }

        let no_tint = Paint::derive_with_theme(FramePaint::whole(&frame), &transparent);
        for row in 0..grid.rows() {
            for column in 0..grid.columns() {
                let position = grid.position(column, row).expect("inside the grid");
                assert_eq!(
                    no_tint.at(position).background,
                    None,
                    "{position:?} was tinted under the transparent Theme"
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
    /// Read under the dark default with its Bang background cleared, so all
    /// three roles' own backgrounds are transparent and any tint is one
    /// borrowed from another claim. None of the three
    /// roles is inside a Reservation — a Comment reserves nothing and a
    /// standalone Bang is not a root Function — which the test asserts rather
    /// than assumes, since Output Portal paint would otherwise supply the tint
    /// that is supposed to be absent.
    ///
    #[tokio::test]
    async fn a_comment_a_bang_and_an_unclaimed_cell_take_no_tint_from_source() {
        let mut orcvs = running_orcvs(8, 4);
        write_row(&mut orcvs, 0, "||hello");
        write_row(&mut orcvs, 2, "**");
        orcvs.select(orcvs.grid().position(7, 3).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let theme = Theme {
            source_bang_background: Color32::TRANSPARENT,
            ..test_theme()
        };
        let paint = Paint::derive_with_theme(FramePaint::whole(&frame), &theme);
        let grid = orcvs.grid();

        for (row, columns, expected, foreground) in [
            (0, 0..8, SourcePaint::Comment, theme.source_comment),
            (2, 0..2, SourcePaint::Bang, theme.source_bang),
            (2, 2..8, SourcePaint::Unclaimed, theme.source_ordinary),
        ] {
            for x in columns {
                let position = grid.position(x, row).expect("inside the grid");
                let cell = frame.at(position);

                assert!(
                    !cell.portal_roles().contains(PortalRole::Output),
                    "row {row} column {x} is inside a Reservation, so `06`'s rule answers it"
                );
                assert_eq!(
                    cell.source_paint(),
                    expected,
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
    /// The Cursor's own fill wins over the Fill tint on its Cell, driven from
    /// Source on a Function Cell: `.+0102`'s `.` is the Cursor's Cell and fills
    /// with the Cursor's colour, while its twin `+` — the other Cell of the
    /// same Function spelling, unselected — keeps the Function tint. The
    /// control is what says the Cursor replaced a fill rather than that there
    /// was none.
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
        let theme = default_theme(Appearance::Dark);
        let function_tint = tinted(&theme, theme.source_function);
        let selected = paint.at(cursor);
        let unselected = paint.at(orcvs.grid().position(1, 0).expect("inside the grid"));

        assert_eq!(frame.cursor(), cursor);
        assert_ne!(theme.selection_background, function_tint);
        assert_eq!(
            selected.background,
            Some(theme.selection_background),
            "the Cursor's Cell did not take the Cursor's own fill"
        );
        assert_eq!(
            unselected.background,
            Some(function_tint),
            "the unselected half of the same Function spelling lost its tint"
        );
        assert_eq!(
            selected.foreground, theme.source_function,
            "the Cursor's Cell stopped spelling in the Function colour"
        );
    }

    ///
    /// End-to-end Output Portal paint tests, built from Source text through the
    /// Render Frame. Every scenario writes the answer directly into Source
    /// rather than running a Tick, since the highlight comes from the current
    /// revision alone: a written value south of a Function is indistinguishable
    /// from one a Tick wrote.
    ///
    mod output_portal_paint {
        use super::{
            Appearance, Color32, FramePaint, Paint, PortalRole, Theme, default_theme,
            running_orcvs, tinted, whole, write_row,
        };
        use orcvs::render_frame::RenderFrame;

        ///
        /// The dark default with an opaque Output Portal fill of its own, so a
        /// Cell's fill names whether the Output Portal painted it: the dark
        /// default frames the Output Portal without filling it.
        ///
        fn filled_portal_theme() -> Theme {
            Theme {
                output_portal_background: Color32::from_rgb(0xFF, 0x80, 0x00),
                ..default_theme(Appearance::Dark)
            }
        }

        fn painted_with(frame: &RenderFrame, theme: &Theme) -> Paint {
            Paint::derive_with_theme(FramePaint::whole(frame), theme)
        }

        ///
        /// A scalar answer south of a Function draws in the Output Portal
        /// colour on the Output Portal fill: `07` left south of `.+0304` re-
        /// parses as two one-Cell unbound Function claims, and the Output
        /// Portal fact paints over that Diagnostic-shaped claim rather than
        /// leaving it Diagnostic.
        ///
        #[tokio::test]
        async fn a_scalar_answer_paints_in_the_output_portal_colour() {
            let mut orcvs = running_orcvs(6, 3);
            write_row(&mut orcvs, 0, ".+0304");
            write_row(&mut orcvs, 1, "07");
            orcvs.select(orcvs.grid().position(0, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let theme = filled_portal_theme();
            let paint = painted_with(&frame, &theme);
            let grid = orcvs.grid();

            for x in 0..2 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(
                    frame
                        .at(position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground, theme.output_portal_foreground,
                    "column {x}"
                );
                assert_eq!(
                    painted.background,
                    Some(theme.output_portal_background),
                    "column {x}"
                );
            }
            // Past the scalar Reservation's pair: an ordinary, untinted blank
            // Cell, the same as if `.+0304` were not there.
            for x in 2..6 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(
                    !frame
                        .at(position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                assert_eq!(paint.at(position).background, None, "column {x}");
            }
        }

        ///
        /// Another scalar answer: `C4` left south of
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
            let theme = filled_portal_theme();
            let paint = painted_with(&frame, &theme);
            let grid = orcvs.grid();

            for x in 0..2 {
                let position = grid.position(x, 1).expect("inside the grid");
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground, theme.output_portal_foreground,
                    "column {x}"
                );
                assert_eq!(
                    painted.background,
                    Some(theme.output_portal_background),
                    "column {x}"
                );
            }
        }

        ///
        /// A Bang answer keeps all its own paint inside an Output Portal:
        /// `**` left south of `~*0401` (Delay) reads as a genuine bound Bang
        /// claim (`RenderFrame`'s own `a_standalone_bang_is_one_bound_bang_claim`)
        /// and paints as the second `**`, on the row below, which sits
        /// outside any Reservation.
        ///
        #[tokio::test]
        async fn a_bang_answer_keeps_its_own_paint_inside_the_output_portal() {
            let mut orcvs = running_orcvs(6, 3);
            write_row(&mut orcvs, 0, "~*0401");
            write_row(&mut orcvs, 1, "**");
            write_row(&mut orcvs, 2, "**");
            orcvs.select(orcvs.grid().position(4, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let theme = default_theme(Appearance::Dark);
            let grid = orcvs.grid();

            for x in 0..2 {
                let portal_position = grid.position(x, 1).expect("inside the grid");
                assert!(
                    frame
                        .at(portal_position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                let portal_painted = paint.at(portal_position);
                assert_eq!(portal_painted.foreground, theme.source_bang, "column {x}");
                assert_eq!(
                    portal_painted.background,
                    Some(theme.source_bang_background),
                    "column {x}"
                );

                let ordinary_position = grid.position(x, 2).expect("inside the grid");
                assert!(
                    !frame
                        .at(ordinary_position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                let ordinary_painted = paint.at(ordinary_position);
                assert_eq!(ordinary_painted.foreground, theme.source_bang, "column {x}");
                assert_eq!(
                    ordinary_painted.background,
                    Some(theme.source_bang_background),
                    "column {x}"
                );
            }
        }

        ///
        /// An empty Output Portal Cell shows the Output Portal fill and no
        /// glyph, including before the first Tick: nothing ever runs a Tick
        /// in this test, so the highlight can only have come from the
        /// current Source revision.
        ///
        #[tokio::test]
        async fn an_empty_output_portal_cell_shows_the_fill_and_no_glyph_before_any_tick() {
            let mut orcvs = running_orcvs(6, 3);
            write_row(&mut orcvs, 0, ".+0102");
            orcvs.select(orcvs.grid().position(0, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let theme = filled_portal_theme();
            let paint = painted_with(&frame, &theme);
            let grid = orcvs.grid();

            for x in 0..2 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(
                    frame
                        .at(position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                assert_eq!(frame.at(position).content(), None, "column {x}");
                let painted = paint.at(position);
                assert_eq!(painted.character, ' ', "column {x}");
                assert_eq!(
                    painted.background,
                    Some(theme.output_portal_background),
                    "column {x}"
                );
            }
            for x in 2..6 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(
                    !frame
                        .at(position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                assert_eq!(paint.at(position).background, None, "column {x}");
            }
        }

        ///
        /// A Terminal Output Function paints nothing at its Output Portal:
        /// `!>` (RawPlay) answers Play, not a Cell, so no Cell plays its
        /// Output Portal role and the row south of it reads exactly as if no root
        /// stood north of it.
        ///
        #[tokio::test]
        async fn a_terminal_output_functions_south_row_is_ordinary() {
            let mut orcvs = running_orcvs(8, 2);
            write_row(&mut orcvs, 0, "!>007F");
            orcvs.select(orcvs.grid().position(6, 0).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let theme = default_theme(Appearance::Dark);
            let grid = orcvs.grid();

            for x in 0..8 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(
                    !frame
                        .at(position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                let painted = paint.at(position);
                assert_eq!(painted.foreground, theme.source_ordinary, "column {x}");
                assert_eq!(painted.background, None, "column {x}");
                assert_eq!(painted.character, ' ', "column {x}");
            }
        }

        ///
        /// Halt paints nothing at its Output Portal: it locks the root there
        /// rather than writing, so the row south of `*!` reads as ordinary.
        ///
        #[tokio::test]
        async fn halts_south_row_is_ordinary() {
            let mut orcvs = running_orcvs(4, 3);
            write_row(&mut orcvs, 0, "*!");
            orcvs.select(orcvs.grid().position(0, 2).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let paint = whole(&frame);
            let theme = default_theme(Appearance::Dark);
            let grid = orcvs.grid();

            for x in 0..4 {
                let position = grid.position(x, 1).expect("inside the grid");
                assert!(
                    !frame
                        .at(position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                let painted = paint.at(position);
                assert_eq!(painted.foreground, theme.source_ordinary, "column {x}");
                assert_eq!(painted.background, None, "column {x}");
            }
        }

        ///
        /// A Self-Banging (Source-writing) Function paints nothing at its
        /// Output Portal: its Advance's writes, including the Cells it
        /// moves onto, are its declared Source effect rather than an answer
        /// through an Output Portal. `>>` is `SelfBangingEast`; the Cells east
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
            let theme = default_theme(Appearance::Dark);
            let grid = orcvs.grid();

            for x in 2..6 {
                let position = grid.position(x, 0).expect("inside the grid");
                assert!(
                    !frame
                        .at(position)
                        .portal_roles()
                        .contains(PortalRole::Output),
                    "column {x}"
                );
                let painted = paint.at(position);
                assert_eq!(painted.foreground, theme.source_ordinary, "column {x}");
                assert_eq!(painted.background, None, "column {x}");
            }
        }

        ///
        /// The paint precedence where an Output Portal covers another
        /// Expression's claimed Cells. `=v` (Copy South) and `.+0102` (Add)
        /// reserve the two Cell pairs south of them, where `.+0304` (Add)
        /// stands as a second, independent root. That root's own two-Cell
        /// spelling, under the Copy's Reservation, is a bound Function claim
        /// and keeps its Function paint outright — the "another root" case —
        /// while its first Number operand, under the upper Add's
        /// Reservation, takes the Output Portal colour and tint instead of
        /// its declared Number role — the "a consumer's operand" case. Both
        /// overlaps are exercised by this one Source.
        ///
        #[tokio::test]
        async fn a_bound_function_spelling_wins_but_its_operands_take_the_output_portal() {
            let mut orcvs = running_orcvs(8, 2);
            write_row(&mut orcvs, 0, "=v.+0102");
            write_row(&mut orcvs, 1, ".+0304");
            // Writing leaves the Cursor at the last Cell it wrote (row 1,
            // column 5) — one of the Cells this test asserts about. Move it
            // back onto row 0, which no assertion below reads.
            orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));

            let frame = orcvs.render_frame();
            let theme = filled_portal_theme();
            let paint = painted_with(&frame, &theme);
            let function_tinted = tinted(&theme, theme.source_function);
            let output_tinted = theme.output_portal_background;
            let grid = orcvs.grid();

            for x in 0..8 {
                assert_eq!(
                    frame
                        .at(grid.position(x, 1).expect("inside the grid"))
                        .portal_roles()
                        .contains(PortalRole::Output),
                    x < 4,
                    "column {x}: the two roots above reserve columns 0 to 3"
                );
            }

            for x in 0..2 {
                let position = grid.position(x, 1).expect("inside the grid");
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground, theme.source_function,
                    "Add's own spelling, column {x}"
                );
                assert_eq!(
                    painted.background,
                    Some(function_tinted),
                    "Add's own spelling, column {x}"
                );
            }
            // Add's own Number operand background is not `output_tinted`
            // alone: the Output Portal tint composites *over* Number's own
            // translucent role background (`role_and_portal`'s
            // `background.blend(theme.output_portal_background)`), and every
            // tinted role background is translucent rather than opaque, so
            // Number's own hue still shows through under the Portal tint here.
            // `output_tinted` alone is only the composited answer where the
            // underlying role's own background is fully transparent, which is
            // why every other `output_tinted` assertion in this module — over
            // Unclaimed Cells — is unaffected.
            let number_under_portal = theme
                .cell_background
                .blend(theme.source_number_background.blend(output_tinted));
            for x in 2..4 {
                let position = grid.position(x, 1).expect("inside the grid");
                let painted = paint.at(position);
                assert_eq!(
                    painted.foreground, theme.output_portal_foreground,
                    "Add's own Number operand, column {x}"
                );
                assert_eq!(
                    painted.background,
                    Some(number_under_portal),
                    "Add's own Number operand, column {x}"
                );
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
        let grid = Grid::with_shape(rows[0].len(), rows.len());
        assert!(
            rows.iter().all(|row| row.len() == rows[0].len()),
            "a Grid's rows are all one length"
        );

        Paint {
            grid,
            drawn: VisiblePositions::for_grid(grid, 0..grid.columns(), 0..grid.rows()),
            outline: None,
            grid_line: Some(GRID_LINE),
            sector_seam_width: 0.0,
            cells: rows
                .iter()
                .flat_map(|row| row.iter())
                .map(|&background| CellPaint {
                    background,
                    border: None,
                    foreground: default_theme(Appearance::Dark).source_ordinary,
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

    /// Whether `paint`'s selection outline is a Region larger than one Cell.
    fn outline_spans(paint: &Paint) -> bool {
        paint
            .outline
            .as_ref()
            .is_some_and(|outline| outline.columns.len() * outline.rows.len() > 1)
    }

    ///
    /// A Paint whose Cells carry the border facts in `rows`, with
    /// `GRID_LINE` between them.
    ///
    fn edge_paint(rows: &[&[Option<EdgeLine>]]) -> Paint {
        let backgrounds = rows
            .iter()
            .map(|row| vec![None; row.len()])
            .collect::<Vec<_>>();
        let mut paint = paint_of(&backgrounds.iter().map(Vec::as_slice).collect::<Vec<_>>());
        for (cell, fact) in paint.cells.iter_mut().zip(rows.iter().copied().flatten()) {
            cell.border = *fact;
        }
        paint
    }

    const OUTLINE: EdgeLine = EdgeLine {
        rank: EdgeRank::Selection,
        stroke: Stroke {
            width: 1.0,
            color: Color32::from_rgb(250, 250, 250),
        },
        seamed: false,
    };

    const SEAM: Color32 = Color32::from_rgba_premultiplied(20, 40, 30, 110);

    const GRID_LINE: EdgeLine = EdgeLine {
        rank: EdgeRank::GridLine,
        stroke: Stroke {
            width: 0.5,
            color: Color32::from_rgb(1, 2, 3),
        },
        seamed: false,
    };

    fn fact(rank: EdgeRank, colour: u8) -> Option<EdgeLine> {
        Some(EdgeLine {
            rank,
            stroke: Stroke::new(0.75, Color32::from_rgb(colour, colour, colour)),
            seamed: false,
        })
    }

    fn edge(line: usize, span: Range<usize>, edge: Option<EdgeLine>) -> EdgeRun {
        EdgeRun {
            line,
            span,
            edge: edge.expect("a line"),
        }
    }

    ///
    /// Two Portal Cells side by side share one line, which runs on into the
    /// edge each shares with its plain neighbour, and a row line coalesces
    /// while its winner holds.
    ///
    #[test]
    fn neighbours_with_equal_facts_share_one_edge() {
        let portal = fact(EdgeRank::OutputPortal, 200);
        let paint = edge_paint(&[&[portal, portal, None]]);
        let edges = paint.edges();

        assert_eq!(
            edges.columns,
            vec![
                edge(0, 0..1, portal),
                edge(1, 0..1, portal),
                edge(2, 0..1, portal),
                edge(3, 0..1, Some(GRID_LINE)),
            ]
        );
        assert_eq!(
            edges.rows,
            vec![
                edge(0, 0..2, portal),
                edge(0, 2..3, Some(GRID_LINE)),
                edge(1, 0..2, portal),
                edge(1, 2..3, Some(GRID_LINE)),
            ]
        );
    }

    ///
    /// Each fact wins against the one ranked below it, whichever side of the
    /// edge either stands on.
    ///
    #[test]
    fn the_higher_ranked_fact_wins_an_edge_from_either_side() {
        let ranks = [
            EdgeRank::Role,
            EdgeRank::Bang,
            EdgeRank::Diagnostic,
            EdgeRank::Locked,
            EdgeRank::InputPortal,
            EdgeRank::OutputPortal,
        ];
        let mut below = None;
        for (index, rank) in ranks.into_iter().enumerate() {
            let above = fact(rank, 10 * index as u8 + 10);
            for row in [[below, above], [above, below]] {
                let paint = edge_paint(&[&row]);
                assert_eq!(
                    paint.edges().columns[1],
                    edge(1, 0..1, above),
                    "{rank:?} against {below:?}"
                );
                let paint = edge_paint(&[&row[..1], &row[1..]]);
                assert_eq!(
                    paint.edges().rows[1],
                    edge(1, 0..1, above),
                    "{rank:?} above or below {below:?}"
                );
            }
            below = above;
        }
    }

    ///
    /// The single-Cell Cursor's outline owns all four of its Cell's edges,
    /// over the Output Portal on either side of it, and its neighbours'
    /// shared edges stay theirs.
    ///
    #[test]
    fn the_cursor_outline_owns_its_cells_four_edges() {
        let portal = fact(EdgeRank::OutputPortal, 200);
        let mut paint = edge_paint(&[&[portal, portal, portal]]);
        paint.outline = Some(Outline {
            line: OUTLINE,
            columns: 1..2,
            rows: 0..1,
        });
        let outline = Some(OUTLINE);
        let edges = paint.edges();

        assert_eq!(
            edges.columns,
            vec![
                edge(0, 0..1, portal),
                edge(1, 0..1, outline),
                edge(2, 0..1, outline),
                edge(3, 0..1, portal),
            ]
        );
        assert_eq!(
            edges.rows,
            vec![
                edge(0, 0..1, portal),
                edge(0, 1..2, outline),
                edge(0, 2..3, portal),
                edge(1, 0..1, portal),
                edge(1, 1..2, outline),
                edge(1, 2..3, portal),
            ]
        );
    }

    ///
    /// A spanning Region's outline owns its perimeter and nothing inside it.
    /// Inside, the Cursor's own Cell outranks the Output Portal beside it.
    ///
    #[test]
    fn a_spanning_regions_outline_owns_its_perimeter_and_its_cursor_cell_ranks_next() {
        let portal = fact(EdgeRank::OutputPortal, 200);
        let cursor = fact(EdgeRank::RegionCursor, 100);
        let mut paint = edge_paint(&[&[None, None, None, None], &[None, cursor, portal, None]]);
        paint.outline = Some(Outline {
            line: OUTLINE,
            columns: 0..3,
            rows: 0..2,
        });
        let (outline, grid) = (Some(OUTLINE), Some(GRID_LINE));
        let edges = paint.edges();

        assert_eq!(
            edges.columns,
            vec![
                edge(0, 0..2, outline),
                edge(1, 0..1, grid),
                edge(1, 1..2, cursor),
                edge(2, 0..1, grid),
                edge(2, 1..2, cursor),
                edge(3, 0..2, outline),
                edge(4, 0..2, grid),
            ]
        );
        assert_eq!(
            edges.rows,
            vec![
                edge(0, 0..3, outline),
                edge(0, 3..4, grid),
                edge(1, 0..1, grid),
                edge(1, 1..2, cursor),
                edge(1, 2..3, portal),
                edge(1, 3..4, grid),
                edge(2, 0..3, outline),
                edge(2, 3..4, grid),
            ]
        );
    }

    ///
    /// A Sector Seam lies over the line that won its edge, composited once
    /// over it, at the wider of the two widths and the line's rank. It
    /// stands alone where no line does.
    ///
    #[test]
    fn a_seam_lies_over_the_line_that_won_its_edge() {
        let portal = fact(EdgeRank::OutputPortal, 200);
        let mut paint = edge_paint(&[&[None, portal, None]]);
        paint.sector_seam_width = 0.75;
        for cell in &mut paint.cells {
            cell.sector_left = Some(SEAM);
        }
        let seamed = |line: EdgeLine, width: f32| EdgeLine {
            rank: line.rank,
            stroke: Stroke::new(width, line.stroke.color.blend(SEAM)),
            seamed: true,
        };
        let portal_line = portal.expect("a fact");

        assert_eq!(
            paint.edges().columns,
            vec![
                edge(0, 0..1, Some(seamed(GRID_LINE, 0.75))),
                edge(1, 0..1, Some(seamed(portal_line, 0.75))),
                edge(2, 0..1, Some(seamed(portal_line, 0.75))),
                edge(3, 0..1, Some(GRID_LINE)),
            ],
            "the far edge has no Cell of its own to carry a seam"
        );

        paint.grid_line = None;
        assert_eq!(
            paint.edges().columns[0],
            edge(
                0,
                0..1,
                Some(EdgeLine {
                    rank: EdgeRank::GridLine,
                    stroke: Stroke::new(0.75, SEAM),
                    seamed: true,
                })
            )
        );
    }

    ///
    /// The selection outline's edges carry no seam, and a seam stands on the
    /// Cursor's Cell's edges once a zero width withdraws the outline.
    ///
    #[test]
    fn the_selection_outline_carries_no_seam() {
        let mut paint = edge_paint(&[&[None, None]]);
        paint.sector_seam_width = 0.75;
        paint.cells[1].sector_left = Some(SEAM);
        paint.outline = Some(Outline {
            line: OUTLINE,
            columns: 1..2,
            rows: 0..1,
        });
        assert_eq!(paint.edges().columns[1], edge(1, 0..1, Some(OUTLINE)));

        paint.outline = None;
        assert_eq!(
            paint.edges().columns[1].edge.stroke.color,
            GRID_LINE.stroke.color.blend(SEAM)
        );
    }

    ///
    /// Two facts of one rank can differ in colour, as a Portal composited
    /// over a Diagnostic does: the Cell below or right of the edge draws it.
    ///
    #[test]
    fn of_two_equal_ranks_the_cell_below_or_right_draws_the_edge() {
        let (first, second) = (
            fact(EdgeRank::OutputPortal, 40),
            fact(EdgeRank::OutputPortal, 80),
        );
        for (one, other) in [(first, second), (second, first)] {
            assert_eq!(
                edge_paint(&[&[one, other]]).edges().columns[1],
                edge(1, 0..1, other),
                "left {one:?}, right {other:?}"
            );
            assert_eq!(
                edge_paint(&[&[one], &[other]]).edges().rows[1],
                edge(1, 0..1, other),
                "above {one:?}, below {other:?}"
            );
        }
    }

    ///
    /// A Theme whose Grid line is zero wide draws no line where no fact
    /// stands, and leaves a fact's edges standing.
    ///
    #[test]
    fn a_zero_width_grid_line_leaves_only_the_facts() {
        let portal = fact(EdgeRank::InputPortal, 90);
        let mut paint = edge_paint(&[&[None, portal, None, None]]);
        paint.grid_line = None;

        assert_eq!(
            paint.edges().columns,
            vec![edge(1, 0..1, portal), edge(2, 0..1, portal)]
        );
    }

    ///
    /// A zero width draws no line, whatever the rank or colour: the Grid line,
    /// the selection outline, a Diagnostic and each Portal border alike.
    ///
    #[test]
    fn a_zero_width_draws_no_line() {
        let zero = crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1");
        for rank in [
            EdgeRank::GridLine,
            EdgeRank::Selection,
            EdgeRank::Diagnostic,
            EdgeRank::Locked,
            EdgeRank::InputPortal,
            EdgeRank::OutputPortal,
        ] {
            assert_eq!(
                EdgeLine::drawn(rank, zero, Color32::from_rgb(200, 30, 30)),
                None,
                "{rank:?}"
            );
        }
        let theme = Theme {
            grid_border_width: zero,
            ..default_theme(Appearance::Dark)
        };
        assert_eq!(EdgeLine::grid(&theme), None);
    }

    ///
    /// A transparent line of nonzero width is drawn, and claims its edge over
    /// the Grid line as any other fact does.
    ///
    #[test]
    fn a_transparent_line_of_nonzero_width_still_claims_its_edge() {
        let width = crate::theme::GridWidth::from_points(0.75).expect("0.75 is within 0..=1");
        let transparent = EdgeLine::drawn(EdgeRank::InputPortal, width, Color32::TRANSPARENT);
        assert_eq!(
            transparent,
            Some(EdgeLine {
                rank: EdgeRank::InputPortal,
                stroke: Stroke::new(0.75, Color32::TRANSPARENT),
                seamed: false,
            })
        );

        let paint = edge_paint(&[&[None, transparent, None]]);
        assert_eq!(
            paint.edges().columns,
            vec![
                edge(0, 0..1, Some(GRID_LINE)),
                edge(1, 0..1, transparent),
                edge(2, 0..1, transparent),
                edge(3, 0..1, Some(GRID_LINE)),
            ]
        );
    }

    ///
    /// The Fill tint, walked end to end from written
    /// Source through `Paint::derive` rather than through `CellColours::colours`
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
        let theme = default_theme(Appearance::Dark);
        // The real bound claims this fixture already carries: `.+`'s own
        // Function claim (columns 0-1) and Multiply's first Number operand
        // (columns 4-5), rather than a claim built by hand — both are bound,
        // so `written` plays no part in either tint.
        let function_cell = frame.at(orcvs.grid().position(0, 0).expect("inside the grid"));
        let number_cell = frame.at(orcvs.grid().position(4, 0).expect("inside the grid"));
        let function_tint = expected_visuals(function_cell, &theme).background;
        let number_tint = expected_visuals(number_cell, &theme).background;
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
        let first = default_theme(Appearance::Dark).selection_background;
        let second = default_theme(Appearance::Dark).grid_background;
        let paint = paint_of(&[&[Some(first), Some(first), Some(second), Some(second)]]);

        assert_eq!(
            paint.background_runs(),
            vec![run(first, 0, 0..2), run(second, 0, 2..4)]
        );
    }

    #[test]
    fn a_run_ends_where_the_next_cell_wants_no_background() {
        let colour = default_theme(Appearance::Dark).selection_background;
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
        let colour = default_theme(Appearance::Dark).selection_background;
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
        let colour = default_theme(Appearance::Dark).selection_background;
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
    /// This is the Cell-iteration half of the claim that the cost of a Render
    /// Frame follows the viewport rather than the Source. Here the
    /// `CellPaint`s are the count, with no `egui::Context` and no window.
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

        assert_eq!(graded, 0, "a drawn Cell painted a fill");
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
    /// instead of left to a comment.
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
    /// A Paint answers the Cursor's outline edges where it covers them, and
    /// none where it does not.
    ///
    /// Their absence is a statement about the viewport rather than about the
    /// Render Frame: the same Frame answers them to a Paint whose range
    /// reaches the Cursor. A derivation that searched its own Cells and
    /// insisted on finding the Cursor would panic on every panned console
    /// whose Cursor is off screen.
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

        let outlined = |paint: Paint| {
            let edges = paint.edges();
            edges
                .rows
                .iter()
                .chain(&edges.columns)
                .filter(|run| run.edge.rank == EdgeRank::Selection)
                .count()
        };
        assert_eq!(outlined(whole(&frame)), 4);
        assert_eq!(
            outlined(Paint::derive(FramePaint::new(&frame, reaching))),
            4
        );
        assert_eq!(outlined(Paint::derive(FramePaint::new(&frame, past))), 0);
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
        assert_eq!(paint.edges(), Default::default());
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
        let other = Grid::with_shape(8, 8);
        let drawn = VisiblePositions::for_grid(other, 0..8, 0..8);

        let _ = FramePaint::new(&frame, drawn);
    }
}
