//! Source Grid viewport geometry.
//!
//! The console presents the Source at a Zoom and a Pan it owns (see
//! `docs/adr/0045-the-source-view-is-a-bounded-space.md`), never fitted to the
//! window. What lives here is the geometry that follows from a presented Grid
//! rectangle whatever put it there: a Cell's own rectangle, the Cell a point
//! falls in, and the Positions a clip rectangle shows. The geometry lives apart
//! from the rendering so it is settled by arithmetic a test can ask about
//! without a window.

use std::ops::Range;

use egui::{Pos2, Rect, Vec2, emath::GuiRounding as _, emath::TSTransform};
use orcvs::grid::{Grid, GridIdentity};

///
/// The side of one Source Cell in points, before any zoom.
///
pub(crate) const CELL_SIZE: f32 = 16.0;

///
/// The Grid viewport the console presents inside an available area.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridViewport {
    /// The side of one square Cell, in points.
    pub(crate) cell_size: f32,
    /// The presented Grid rectangle, anchored at the corner its transform put
    /// the Source's top-left at.
    pub(crate) rect: Rect,
}

impl GridViewport {
    ///
    /// The presented Cell side over the Source's own Cell side.
    ///
    /// Stroke widths multiply by this so Grid lines and sector seams stay one
    /// Source point wide at every zoom.
    ///
    pub(crate) fn cell_scale(&self) -> f32 {
        self.cell_size / CELL_SIZE
    }
}

///
/// The Positions the console draws, as a half-open column range and a
/// half-open row range.
///
/// This is what the console shows *and one Cell more* in every direction the
/// Grid has one, so it is not the set of Positions a viewer can see — the
/// extra Cell is the margin the seams need. A caller that needs only
/// what is on screen has to narrow it; a caller drawing them does not.
///
/// The ranges are already clamped to the Grid, so a caller walks those
/// column and row numbers rather than bounds-checking a Position at a time.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisiblePositions {
    /// The Grid these ranges belong to, or none when empty.
    grid: Option<GridIdentity>,
    /// The columns to draw, left to right.
    pub(crate) columns: Range<usize>,
    /// The rows to draw, top to bottom.
    pub(crate) rows: Range<usize>,
}

fn clamp_range(range: Range<usize>, extent: usize) -> Range<usize> {
    let start = range.start.min(extent);
    let end = range.end.min(extent);

    if start < end { start..end } else { 0..0 }
}

impl VisiblePositions {
    /// The Positions a Paint covers: column and row ranges clamped to `grid`.
    pub fn for_grid(grid: Grid, columns: Range<usize>, rows: Range<usize>) -> Self {
        Self {
            grid: Some(grid.identity()),
            columns: clamp_range(columns, grid.columns()),
            rows: clamp_range(rows, grid.rows()),
        }
    }

    /// The Grid this range belongs to, or `None` when [`Self::empty`].
    pub fn grid(&self) -> Option<GridIdentity> {
        self.grid
    }

    /// No Position at all: what a console showing none of the Grid draws.
    pub fn empty() -> Self {
        Self {
            grid: None,
            columns: 0..0,
            rows: 0..0,
        }
    }

    /// How many Positions the two ranges cover between them.
    pub fn count(&self) -> usize {
        self.columns.len().saturating_mul(self.rows.len())
    }
}

impl GridViewport {
    ///
    /// The rectangle the Cell at `column` and `row` occupies.
    ///
    /// A Cell's rectangle is a function of its Position and nothing else. The
    /// Cursor, the selection and the blink phase reach the paint a Cell is
    /// filled and stroked with; none of them reaches the geometry it is painted
    /// at.
    ///
    pub(crate) fn cell_rect(&self, column: usize, row: usize) -> Rect {
        Rect::from_min_size(
            self.rect.min + Vec2::new(column as f32, row as f32) * self.cell_size,
            Vec2::splat(self.cell_size),
        )
    }

    ///
    /// The Cell `point` falls in, as a column and a row, or `None` when the
    /// point is outside the presented Grid.
    ///
    /// This is the whole of what a click has to answer, and it is arithmetic
    /// rather than a search: no Cell is hit-tested. The Grid bounds the answer,
    /// because a point on the Grid's far edge divides to one past the last Cell.
    /// It is the inverse of [`Self::cell_rect`] — see [`Self::cell_index`] for
    /// what that costs in `f32` — so a click cannot resolve to a Cell other than
    /// the one drawn under it.
    ///
    pub(crate) fn cell_at(&self, point: Pos2, grid: Grid) -> Option<(usize, usize)> {
        // `Grid::new` asserts both counts are non-zero, so an empty Grid is
        // unrepresentable. Only a degenerate Cell size is refused here.
        if !(self.cell_size.is_finite() && self.cell_size > 0.0) {
            return None;
        }
        if !self.rect.contains(point) {
            return None;
        }
        Some((
            self.cell_index(point.x, self.rect.min.x, grid.columns()),
            self.cell_index(point.y, self.rect.min.y, grid.rows()),
        ))
    }

    ///
    /// The Cell nearest `point`: the one under it, or where `point` is past
    /// the presented Grid, the edge Cell it is past.
    ///
    /// What a drag asks while the pointer is anywhere at all, the console's
    /// surplus and the space past the window included, so the Cursor it moves
    /// always lands on a Cell. `None` only for a degenerate Cell size, as
    /// [`Self::cell_at`] refuses.
    ///
    pub(crate) fn nearest_cell(&self, point: Pos2, grid: Grid) -> Option<(usize, usize)> {
        if !(self.cell_size.is_finite() && self.cell_size > 0.0) || point.any_nan() {
            return None;
        }
        let point = point.clamp(self.rect.min, self.rect.max);
        Some((
            self.cell_index(point.x, self.rect.min.x, grid.columns()),
            self.cell_index(point.y, self.rect.min.y, grid.rows()),
        ))
    }

    ///
    /// Which of `extent` Cells along one axis `coordinate` falls in, counting
    /// from `start`.
    ///
    /// The division is the answer wherever the arithmetic is exact, and the
    /// clamp is for the far edge, which divides to one past the last Cell. The
    /// correction is for everywhere else: `start + n * cell_size` is the corner
    /// [`Self::cell_rect`] mints, and neither that multiply-add nor the division
    /// undoing it is exact in `f32`, so a corner can land a fraction either side
    /// of the boundary the division assumes. Correcting against `cell_rect`'s
    /// own arithmetic rather than trusting the division is what makes this the
    /// inverse it claims to be: the error is at most one Cell, so one step
    /// either way settles it.
    ///
    fn cell_index(&self, coordinate: f32, start: f32, extent: usize) -> usize {
        let index = ((coordinate - start) / self.cell_size) as usize;
        let index = index.min(extent - 1);

        if index > 0 && coordinate < start + index as f32 * self.cell_size {
            index - 1
        } else if index + 1 < extent && coordinate >= start + (index + 1) as f32 * self.cell_size {
            index + 1
        } else {
            index
        }
    }

    ///
    /// The Positions inside `clip`, expanded by one Cell in every direction
    /// and clamped to the Grid.
    ///
    /// This is [`Self::cell_at`] on the two corners of the Grid `clip` leaves
    /// visible — the same division a click resolves through, so the Cells that
    /// are drawn are the Cells a pointer could land on. A clip that misses the
    /// Grid answers empty ranges, and a Grid with no Cell to draw answers the
    /// same.
    ///
    /// # Why the range is one Cell wider than the clip
    ///
    /// The margin is deliberate over-draw handed to the clip rectangle, whose
    /// job it already is to discard it. A Cell draws the sector seams on its
    /// *own* left and top edges, so the seam at a Cell's right or bottom edge
    /// is drawn by the Cell one past it; and a feathered border spills roughly
    /// a physical pixel outside the rectangle it strokes.
    ///
    /// Both of those land on the clip's boundary rather than inside it. The
    /// seam at the last shown Cell's right edge is on screen only where the
    /// clip's own edge falls exactly on that Cell boundary, and the feathering
    /// is sub-pixel. So this is a margin for the boundary cases, not a rescue
    /// of seams that would otherwise be missing from inside the viewport:
    /// removing it changes no painted Shape strictly inside the clip in any
    /// case the suite reaches. That is why
    /// `a_zoomed_console_paints_every_sector_seam_inside_the_clip` asserts the
    /// seams the clip keeps, and why the margin itself is pinned as a range
    /// value in
    /// `the_visible_range_is_the_shown_positions_and_one_cell_more_each_way`
    /// rather than as a line a viewer can see.
    ///
    pub(crate) fn visible_positions(&self, clip: Rect, grid: Grid) -> VisiblePositions {
        let visible = self.rect.intersect(clip);
        // A clip sharing exactly one edge with the Grid shows no part of it.
        // `Rect::intersect` answers a zero-area rectangle there, and
        // `Rect::contains` is inclusive, so both of its corners are inside the
        // Grid and `cell_at` would accept them — answering a range of Cells
        // that are entirely off screen. Area is the same question `overlaps`
        // asks of a single Cell in the tests below, asked of the whole Grid.
        // This also covers the inverted rectangle `intersect` answers when the
        // two do not meet at all.
        if !visible.is_positive() {
            return VisiblePositions::empty();
        }
        // The degenerate viewport is refused by `cell_at` rather than by another
        // guard here. `Grid` makes a zero-count Grid unrepresentable.
        let (Some((first_column, first_row)), Some((last_column, last_row))) = (
            self.cell_at(visible.min, grid),
            self.cell_at(visible.max, grid),
        ) else {
            return VisiblePositions::empty();
        };

        VisiblePositions {
            grid: Some(grid.identity()),
            columns: first_column.saturating_sub(1)
                ..last_column.saturating_add(2).min(grid.columns()),
            rows: first_row.saturating_sub(1)..last_row.saturating_add(2).min(grid.rows()),
        }
    }
}

///
/// `side` points snapped down to a whole number of physical pixels at
/// `pixels_per_point`, the Cell side [`presented_grid`] draws at.
///
/// Floored rather than rounded, so a Grid of `columns` Cells never exceeds
/// the extent the unsnapped side asked for, and at least one physical pixel
/// wherever there is any Cell at all, so a Grid that is merely very small is
/// still drawn rather than floored away. A `side` or a device scale that is not
/// positive and finite answers no Cell: see [`presented_grid`] for why a
/// device scale is refused on those terms.
///
/// The console reads its Pan bounds and its Cursor follow through this same
/// function, so what it clamps is the Grid [`presented_grid`] draws.
///
pub(crate) fn snapped_cell_side(side: f32, pixels_per_point: f32) -> f32 {
    match device_scale(pixels_per_point) {
        Some(scale) if side.is_finite() && side > 0.0 => (side * scale).floor().max(1.0) / scale,
        _ => 0.0,
    }
}

///
/// `pixels_per_point` where it is a scale at all: positive and finite.
///
fn device_scale(pixels_per_point: f32) -> Option<f32> {
    (pixels_per_point.is_finite() && pixels_per_point > 0.0).then_some(pixels_per_point)
}

///
/// The Grid rectangle `to_global` presents, with Cell geometry snapped to whole
/// physical pixels.
///
/// **This is the one place the Source is scaled.** The console owns
/// `to_global` — `egui::Scene` used to own it, and a Scene applies its scale to
/// a whole layer of shapes after they are built. Here the scale reaches the
/// Grid before a single Shape exists, so a Cell's two axes still cannot part
/// company (one `scaling` serves both) and no galley is ever transformed after
/// layout.
///
/// # Why the Cell size is snapped
///
/// A Cell size derived from a continuous zoom is fractional, and a fractional
/// Cell size puts each row's edges at a different sub-pixel offset. Rows then
/// resolve a pixel taller or shorter than their neighbours and the eye reads
/// the Grid as irregularly spaced. Snapping the Cell side to a whole physical
/// pixel makes every row identical.
///
/// The snap is [`snapped_cell_side`]'s, and the snapped Grid is anchored at
/// the transform's origin — the corner `to_global` puts the Source's own
/// top-left at — rather than re-centred on the rectangle it asked for. The
/// console bounds its Pan and follows its Cursor at that same snapped side, so
/// the Grid it clamps is the Grid drawn here: a Source smaller than the console
/// starts at the console's top-left, and a Pan to the far edge leaves no gap
/// past the last Cell. Re-centring would move the Grid in by half the snap's
/// shortfall at both ends, which is exactly what the console's bounds cannot
/// see.
///
pub(crate) fn presented_grid(
    to_global: TSTransform,
    source: Rect,
    grid: Grid,
    pixels_per_point: f32,
) -> GridViewport {
    let columns = grid.columns();
    let rows = grid.rows();
    let presented = to_global * source;
    // The device scale is divided by as well as multiplied by — once for the
    // Cell size and again for the corner — so it is refused on the same terms
    // as every other input here rather than checked for finiteness alone. Zero
    // answers an infinite Cell and a NaN corner; a negative one answers a Cell
    // that `cell_rect` paints inverted and `cell_at` refuses every click on.
    // Refused, the Grid keeps its unsnapped corner and no Cell at all, which is
    // the same nothing a console with no area presents.
    let device_scale = device_scale(pixels_per_point);
    // `Grid` makes a zero-column Grid unrepresentable, so the division is safe.
    let cell_size = snapped_cell_side(presented.width() / columns as f32, pixels_per_point);
    let size = Vec2::new(columns as f32, rows as f32) * cell_size;
    let corner = presented.min;

    GridViewport {
        cell_size,
        rect: Rect::from_min_size(
            device_scale.map_or(corner, |scale| corner.round_to_pixels(scale)),
            size,
        ),
    }
}

#[cfg(test)]
mod tests {
    use egui::{Pos2, Rect, Vec2, emath::TSTransform};

    use orcvs::grid::Grid;

    use super::{CELL_SIZE, GridViewport, VisiblePositions, presented_grid};

    const GRID: usize = 32;

    ///
    /// The Source's own Cell is 16 points so every Zoom step of an eighth is a
    /// whole number of points, and a Cell is a whole number of physical pixels
    /// at 1×, 1.5× and 2× with nothing to snap.
    ///
    #[test]
    fn the_source_cell_is_sixteen_points() {
        assert_eq!(CELL_SIZE, 16.0);
    }

    #[test]
    fn every_eighth_from_a_quarter_to_double_is_a_whole_number_of_points() {
        let mut thousandths = 250_u32;
        while thousandths <= 2_000 {
            let zoom = thousandths as f32 / 1_000.0;
            if (zoom / 0.125 - (zoom / 0.125).round()).abs() < 1e-6 {
                let points = CELL_SIZE * zoom;
                assert!(
                    (points - points.round()).abs() < 1e-6,
                    "zoom {zoom} presented a Cell of {points} points"
                );
            }
            thousandths += 125;
        }
    }

    #[test]
    fn a_cell_is_a_whole_number_of_pixels_at_one_one_and_a_half_and_two_with_nothing_to_snap() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(CELL_SIZE * 8.0));
        let mut thousandths = 250_u32;
        while thousandths <= 2_000 {
            let zoom = thousandths as f32 / 1_000.0;
            for pixels_per_point in [1.0_f32, 1.5, 2.0] {
                let presented = presented_grid(
                    TSTransform::from_scaling(zoom),
                    source,
                    sized(8, 8),
                    pixels_per_point,
                );
                let asked = CELL_SIZE * zoom;
                let in_pixels = asked * pixels_per_point;

                assert!(
                    (in_pixels - in_pixels.round()).abs() < 1e-6,
                    "zoom {zoom} at {pixels_per_point} ppp is {in_pixels} pixels"
                );
                assert_eq!(
                    presented.cell_size, asked,
                    "zoom {zoom} at {pixels_per_point} ppp snapped {asked} to {}",
                    presented.cell_size
                );
            }
            thousandths += 125;
        }
    }

    fn square() -> Grid {
        Grid::new(GRID, GRID)
    }

    fn sized(columns: usize, rows: usize) -> Grid {
        Grid::new(columns, rows)
    }

    fn area(width: f32, height: f32) -> Rect {
        Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(width, height))
    }

    fn assert_close(left: f32, right: f32, what: &str) {
        assert!((left - right).abs() < 1e-3, "{what}: {left} is not {right}");
    }

    ///
    /// A square-Cell `GridViewport` centred in `available`, for the tests below
    /// that need an arbitrary non-degenerate viewport to drive `cell_rect`,
    /// `cell_at` and `visible_positions` through.
    ///
    /// Test-only: no shipped code fits a viewport to an area any more — the
    /// Source View pans and zooms instead of fitting, and `presented_grid` is
    /// the one place a viewport comes from a transform the console owns. This
    /// is that arithmetic rebuilt beside the tests that still need a viewport
    /// with no transform to hand.
    ///
    fn square_cell_viewport(available: Rect, grid: Grid) -> GridViewport {
        let columns = grid.columns() as f32;
        let rows = grid.rows() as f32;
        let cell_size = (available.width() / columns)
            .min(available.height() / rows)
            .max(0.0);

        GridViewport {
            cell_size,
            rect: Rect::from_center_size(
                available.center(),
                Vec2::new(columns * cell_size, rows * cell_size),
            ),
        }
    }

    ///
    /// A Cell's own corner answers that Cell for any Cell size, not only for
    /// the sizes whose arithmetic happens to come out exact.
    ///
    /// `cell_rect` reaches a corner by `rect.min + n * cell_size` and `cell_at`
    /// returns from it by `(point - rect.min) / cell_size`. Neither step is
    /// exact in `f32`, so a corner can land a fraction below the boundary that
    /// produced it and truncate into the Cell before it. The pair below is such
    /// a case, found by sweeping the two together: `rect.min.x` of `432.0` with
    /// a Cell size of `34.436707` puts the corner of column 19 just under its
    /// own boundary.
    ///
    /// At Zoom 1.0 from an unpanned origin `show_source` paints at the
    /// `CELL_SIZE` constant, where the division is exact for every origin a
    /// resize can produce. A Zoom step or a Pan puts `rect.min` and
    /// `cell_size` wherever the transform and the physical-pixel snap answer,
    /// which need not divide evenly — this pins the inverse for that case
    /// ahead of a viewer finding it by panning or zooming into it.
    ///
    #[test]
    fn a_cell_corner_answers_its_own_cell_whatever_the_cell_measures() {
        let viewport = GridViewport {
            cell_size: 34.436707,
            rect: Rect::from_min_size(Pos2::new(432.0, 432.0), Vec2::splat(34.436707 * 32.0)),
        };

        for row in 0..GRID {
            for column in 0..GRID {
                let corner = viewport.cell_rect(column, row).min;

                assert_eq!(
                    viewport.cell_at(corner, square()),
                    Some((column, row)),
                    "the corner of Cell ({column}, {row}) answered another Cell"
                );
            }
        }

        // The one pair above is the case that was found; the sweep is what says
        // the inverse holds rather than that this pair was patched. Cell
        // centres are swept alongside corners, because a correction that
        // reached past the boundary would move them too.
        for step in 0..200 {
            let cell_size = 0.75 + step as f32 * 0.31719;
            let viewport = GridViewport {
                cell_size,
                rect: Rect::from_min_size(
                    Pos2::new(311.0 + step as f32 * 7.13, 47.0 + step as f32 * 3.7),
                    Vec2::splat(cell_size * GRID as f32),
                ),
            };

            for row in 0..GRID {
                for column in 0..GRID {
                    let rect = viewport.cell_rect(column, row);

                    assert_eq!(
                        viewport.cell_at(rect.min, square()),
                        Some((column, row)),
                        "the corner of Cell ({column}, {row}) at a Cell size of {cell_size}"
                    );
                    assert_eq!(
                        viewport.cell_at(rect.center(), square()),
                        Some((column, row)),
                        "the centre of Cell ({column}, {row}) at a Cell size of {cell_size}"
                    );
                }
            }
        }
    }

    ///
    /// Painting and clicking go through one arithmetic, so the Cell a click
    /// answers is the Cell drawn under the pointer rather than a second
    /// derivation that could drift from it.
    ///
    #[test]
    fn a_point_in_a_cell_answers_the_cell_that_was_painted_there() {
        let viewport = square_cell_viewport(area(1200.0, 700.0), sized(10, 4));

        for row in 0..4 {
            for column in 0..10 {
                let rect = viewport.cell_rect(column, row);

                assert_eq!(
                    viewport.cell_at(rect.center(), sized(10, 4)),
                    Some((column, row))
                );
                assert_eq!(
                    viewport.cell_at(rect.min, sized(10, 4)),
                    Some((column, row))
                );
                assert_close(rect.width(), viewport.cell_size, "Cell width");
                assert_close(rect.height(), viewport.cell_size, "Cell height");
            }
        }
    }

    ///
    /// The Grid's far corner belongs to the last Cell rather than to a Cell one
    /// past it, and any point outside the presented rectangle — wherever that
    /// rectangle sits in the console — belongs to no Cell at all.
    ///
    ///
    /// A point past the presented Grid resolves to the edge Cell it is past,
    /// on each axis on its own, and a point on the Grid to the Cell under it.
    ///
    #[test]
    fn the_nearest_cell_to_a_point_past_the_grid_is_the_edge_cell_it_is_past() {
        let viewport = square_cell_viewport(area(1200.0, 700.0), square());
        let rect = viewport.rect;
        let inside = viewport.cell_rect(3, 5).center();

        assert_eq!(viewport.nearest_cell(inside, square()), Some((3, 5)));
        assert_eq!(
            viewport.nearest_cell(Pos2::new(rect.max.x + 500.0, inside.y), square()),
            Some((GRID - 1, 5))
        );
        assert_eq!(
            viewport.nearest_cell(Pos2::new(inside.x, rect.min.y - 500.0), square()),
            Some((3, 0))
        );
        assert_eq!(
            viewport.nearest_cell(rect.min - Vec2::splat(1.0), square()),
            Some((0, 0))
        );
        assert_eq!(
            viewport.nearest_cell(Pos2::new(f32::NAN, 0.0), square()),
            None
        );
    }

    #[test]
    fn the_grid_edge_resolves_to_the_last_cell_and_outside_it_to_none() {
        let available = area(1200.0, 700.0);
        let viewport = square_cell_viewport(available, square());

        assert_eq!(
            viewport.cell_at(viewport.rect.max, square()),
            Some((GRID - 1, GRID - 1))
        );
        assert_eq!(viewport.cell_at(viewport.rect.min, square()), Some((0, 0)));
        assert_eq!(
            viewport.cell_at(available.left_center(), square()),
            None,
            "a point outside the presented Grid answered a Cell"
        );
        assert_eq!(viewport.cell_at(Pos2::new(f32::NAN, 0.0), square()), None);

        // A wide `available` puts the surplus at `left_center`; a tall one
        // puts it at `center_top` instead, so the "outside the Grid" claim is
        // only proven if both orientations are asked.
        let tall = area(700.0, 1200.0);
        let viewport = square_cell_viewport(tall, square());

        assert_eq!(
            viewport.cell_at(tall.center_top(), square()),
            None,
            "a point above the presented Grid answered a Cell"
        );
        assert_eq!(
            viewport.cell_at(tall.left_center(), square()),
            Some((0, GRID / 2)),
            "a point inside the presented Grid answered no Cell"
        );
    }

    ///
    /// A point strictly inside both rectangles, rather than merely on the
    /// shared edge of two that touch. `Rect::intersects` answers true for a
    /// Cell whose far edge is the clip's near edge, and that Cell shows
    /// nothing.
    ///
    fn overlaps(cell: Rect, clip: Rect) -> bool {
        cell.intersect(clip).is_positive()
    }

    ///
    /// The visible range is every Position the clip shows, and one more in
    /// every direction the Grid has one.
    ///
    /// The margin is the acceptance criterion rather than a tolerance, and a
    /// range value is the only place it can be held. A Cell draws its own left
    /// and top seams, so the seam at a Cell's right or bottom edge belongs to
    /// the Cell one past it — but that seam is on screen only where the clip's
    /// edge falls exactly on the boundary, and the clip discards the rest of
    /// what the margin adds. See `GridViewport::visible_positions`: the margin
    /// reaches no painted Shape strictly inside the clip, so no assertion about
    /// what was painted can pin it.
    ///
    #[test]
    fn the_visible_range_is_the_shown_positions_and_one_cell_more_each_way() {
        let grid = square();
        let viewport = square_cell_viewport(area(800.0, 800.0), grid);
        assert_close(viewport.cell_size, 25.0, "Cell size");
        // Deliberately not on Cell boundaries: a clip that ends exactly on one
        // would not distinguish the margin from the rounding.
        let clip = Rect::from_min_max(
            viewport.rect.min + Vec2::splat(110.0),
            viewport.rect.min + Vec2::splat(290.0),
        );

        let visible = viewport.visible_positions(clip, grid);

        // Columns 4 through 11 are shown, so the range runs 3 through 12.
        assert_eq!(visible, VisiblePositions::for_grid(grid, 3..13, 3..13));
        assert_eq!(visible.count(), 10 * 10);

        for row in 0..GRID {
            for column in 0..GRID {
                let shown = overlaps(viewport.cell_rect(column, row), clip);
                assert!(
                    !shown || (visible.columns.contains(&column) && visible.rows.contains(&row)),
                    "the Cell at {column},{row} is shown and was not in {visible:?}"
                );
            }
        }
        // And one Cell further on every side, which is the margin: each of
        // these four is drawn and none of them is shown.
        for (column, row) in [
            (visible.columns.start, visible.rows.start),
            (visible.columns.end - 1, visible.rows.end - 1),
            (visible.columns.start, visible.rows.end - 1),
            (visible.columns.end - 1, visible.rows.start),
        ] {
            assert!(
                !overlaps(viewport.cell_rect(column, row), clip),
                "the Cell at {column},{row} is shown, so the range has no margin there"
            );
        }
    }

    ///
    /// A console showing the whole Grid iterates the whole Grid: the clamp is
    /// what stops the margin reaching past the last Position rather than the
    /// caller bounds-checking one.
    ///
    #[test]
    fn a_console_showing_the_whole_grid_ranges_over_the_whole_grid() {
        let available = area(800.0, 800.0);
        let grid = square();
        let viewport = square_cell_viewport(available, grid);

        let visible = viewport.visible_positions(available, grid);

        assert_eq!(visible, VisiblePositions::for_grid(grid, 0..GRID, 0..GRID));
        assert_eq!(visible.count(), GRID * GRID);
    }

    ///
    /// A clip that shows no part of the Grid and a console with no area both
    /// range over nothing — which draws nothing rather than drawing a Position
    /// that is not there. A Grid with no Cell is unrepresentable (`Grid::new`
    /// asserts both counts are non-zero).
    ///
    #[test]
    fn a_console_showing_no_part_of_the_grid_ranges_over_nothing() {
        let available = area(800.0, 800.0);
        let viewport = square_cell_viewport(available, square());

        for clip in [
            Rect::from_min_size(Pos2::new(2000.0, 2000.0), Vec2::splat(100.0)),
            Rect::from_min_size(Pos2::new(-2000.0, 20.0), Vec2::splat(100.0)),
            Rect::ZERO,
            Rect::NOTHING,
        ] {
            let visible = viewport.visible_positions(clip, square());

            assert_eq!(visible.count(), 0, "{clip:?} reached {visible:?}");
        }

        let no_area = GridViewport {
            cell_size: 0.0,
            rect: Rect::ZERO,
        };
        assert_eq!(no_area.visible_positions(available, square()).count(), 0);
    }

    #[test]
    fn visible_positions_minted_for_a_grid_are_owned_by_that_grid() {
        let grid = Grid::new(10, 8);
        let other = Grid::new(10, 8);
        let visible = VisiblePositions::for_grid(grid, 2..6, 1..5);

        assert!(grid.owns_identity(visible.grid().expect("minted for a Grid")));
        assert!(!other.owns_identity(visible.grid().expect("minted for a Grid")));
    }

    #[test]
    fn for_grid_clamps_out_of_bounds_ranges_to_the_grid() {
        let grid = Grid::new(10, 8);

        assert_eq!(
            VisiblePositions::for_grid(grid, 12..20, 10..20),
            VisiblePositions::for_grid(grid, 0..0, 0..0)
        );
        assert_eq!(
            VisiblePositions::for_grid(grid, 12..20, 6..14),
            VisiblePositions {
                grid: Some(grid.identity()),
                columns: 0..0,
                rows: 6..8,
            }
        );
        assert_eq!(
            VisiblePositions::for_grid(grid, 8..15, 2..6),
            VisiblePositions {
                grid: Some(grid.identity()),
                columns: 8..10,
                rows: 2..6,
            }
        );
        assert_eq!(
            VisiblePositions::for_grid(grid, 0..10, 0..8),
            VisiblePositions {
                grid: Some(grid.identity()),
                columns: 0..10,
                rows: 0..8,
            }
        );
    }

    ///
    /// A clip that only touches the Grid shows no part of it, so it ranges
    /// over nothing.
    ///
    /// `Rect::intersect` answers a zero-area rectangle where two rectangles
    /// share an edge, and `Rect::contains` is inclusive, so both corners of
    /// that rectangle are inside the Grid and `cell_at` accepts them. A shared
    /// edge shows nothing, which is what the `overlaps` helper above says of a
    /// Cell and what this says of the whole Grid.
    ///
    #[test]
    fn a_clip_that_only_touches_the_grid_ranges_over_nothing() {
        let viewport = square_cell_viewport(area(800.0, 800.0), square());
        let rect = viewport.rect;

        // Just past each edge in turn, sharing exactly that edge with the Grid.
        for clip in [
            Rect::from_min_max(
                Pos2::new(rect.max.x, rect.min.y),
                Pos2::new(rect.max.x + 100.0, rect.max.y),
            ),
            Rect::from_min_max(
                Pos2::new(rect.min.x - 100.0, rect.min.y),
                Pos2::new(rect.min.x, rect.max.y),
            ),
            Rect::from_min_max(
                Pos2::new(rect.min.x, rect.max.y),
                Pos2::new(rect.max.x, rect.max.y + 100.0),
            ),
            Rect::from_min_max(
                Pos2::new(rect.min.x, rect.min.y - 100.0),
                Pos2::new(rect.max.x, rect.min.y),
            ),
        ] {
            let visible = viewport.visible_positions(clip, square());

            assert_eq!(visible.count(), 0, "{clip:?} reached {visible:?}");
        }
    }

    #[test]
    fn a_viewport_with_no_area_answers_no_cell() {
        let viewport = GridViewport {
            cell_size: 0.0,
            rect: Rect::ZERO,
        };

        assert_eq!(viewport.cell_at(Pos2::ZERO, square()), None);
    }

    ///
    /// Every Cell is the same whole number of physical pixels across, at every
    /// zoom and at every device scale. A fractional Cell side would pixel-snap
    /// differently row by row and read as irregular spacing.
    ///
    #[test]
    fn every_cell_is_a_whole_number_of_physical_pixels() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0));
        for pixels_per_point in [1.0_f32, 1.5, 2.0] {
            for scaling in [0.25_f32, 0.31, 0.7, 1.0, 1.37, 2.0] {
                let presented = presented_grid(
                    TSTransform::new(Vec2::new(11.3, 7.9), scaling),
                    source,
                    sized(8, 8),
                    pixels_per_point,
                );
                let in_pixels = presented.cell_size * pixels_per_point;

                assert!(
                    (in_pixels - in_pixels.round()).abs() < 1e-3,
                    "a Cell was {in_pixels} pixels across at {scaling}x, {pixels_per_point} ppp"
                );
                assert!(
                    in_pixels >= 1.0,
                    "a Cell was floored away at {scaling}x, {pixels_per_point} ppp"
                );
                // The whole Grid is a whole number of Cells, so the last Cell's
                // far edge is where the Grid's is.
                assert_close(
                    presented.cell_rect(7, 7).max.x,
                    presented.rect.max.x,
                    "the Grid's far edge",
                );
            }
        }
    }

    ///
    /// A pan moves the presented Grid by exactly what it moved the transform
    /// by, whatever the zoom. The scale multiplies the Source's coordinates,
    /// never a translation already expressed in presented points.
    ///
    #[test]
    fn a_translation_moves_the_presented_grid_by_itself_at_any_scale() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0));
        let moved = Vec2::new(40.0, 24.0);
        for scaling in [0.5_f32, 1.0, 2.0] {
            let before =
                presented_grid(TSTransform::from_scaling(scaling), source, sized(8, 8), 1.0);
            let after = presented_grid(TSTransform::new(moved, scaling), source, sized(8, 8), 1.0);

            assert_close(after.rect.min.x - before.rect.min.x, moved.x, "panned x");
            assert_close(after.rect.min.y - before.rect.min.y, moved.y, "panned y");
            assert_close(after.cell_size, before.cell_size, "a pan changed the zoom");
        }
    }

    ///
    /// A `to_global` with no scale of its own presents no Cell rather than a
    /// NaN one. `TSTransform::inverse` divides by the scaling, so a zero there
    /// would resolve every pointer position to NaN; the console clamps Zoom
    /// before a transform like this ever reaches `presented_grid`, but the
    /// geometry refuses it on its own terms rather than trusting the caller.
    ///
    #[test]
    fn a_transform_with_no_scale_presents_no_cell_rather_than_a_nan_one() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0));
        let degenerate = TSTransform::from_scaling(0.0);

        assert!(!degenerate.is_valid());

        let presented = presented_grid(degenerate, source, square(), 1.0);

        assert_eq!(presented.cell_size, 0.0);
        assert_eq!(presented.cell_at(Pos2::ZERO, square()), None);
    }

    ///
    /// A device scale that is not a scale presents no Grid rather than an
    /// infinite or an inverted one.
    ///
    /// The snap divides by `pixels_per_point` after flooring to at least one
    /// physical pixel, so a zero scale answers an infinite Cell and a NaN
    /// rectangle, and a negative one answers a Cell that `cell_rect` paints
    /// inverted while `cell_at` refuses every click. Every other degenerate
    /// input to this function is already refused; this is the same refusal
    /// stated over the one input that was only checked for finiteness.
    ///
    #[test]
    fn a_device_scale_that_is_not_a_scale_presents_no_grid() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0));
        for pixels_per_point in [0.0_f32, -2.0, f32::NAN, f32::INFINITY] {
            let presented =
                presented_grid(TSTransform::IDENTITY, source, sized(8, 8), pixels_per_point);

            assert_eq!(
                presented.cell_size, 0.0,
                "a device scale of {pixels_per_point} presented a Cell"
            );
            assert!(
                presented.rect.min.x.is_finite() && presented.rect.min.y.is_finite(),
                "a device scale of {pixels_per_point} put the Grid at {:?}",
                presented.rect.min
            );
            assert_eq!(presented.cell_at(Pos2::ZERO, sized(8, 8)), None);
        }
    }
}
