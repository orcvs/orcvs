//! Source Grid viewport geometry.
//!
//! The console spends the area the window leaves it on the largest Grid
//! viewport whose Cells are square, and centres that viewport so the surplus on
//! the longer axis falls away as letterboxing on both sides. The geometry lives
//! here, apart from the rendering, so the wide, tall and square cases are
//! settled by arithmetic a test can ask about without a window.

use std::ops::Range;

use egui::{Pos2, Rect, Vec2, emath::GuiRounding as _, emath::TSTransform};

///
/// The Grid viewport the console presents inside an available area.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GridViewport {
    /// The side of one square Cell, in points.
    pub(crate) cell_size: f32,
    /// The presented Grid rectangle, centred in the available area.
    pub(crate) rect: Rect,
}

///
/// The Positions the console draws, as a half-open column range and a
/// half-open row range.
///
/// This is what the console shows *and one Cell more* in every direction the
/// Grid has one, so it is not the set of Positions a viewer can see — see
/// [`GridViewport::visible_positions`] for why the margin is there. A caller
/// that needs only what is on screen has to narrow it; a caller drawing them
/// does not.
///
/// The ranges are already clamped to the Grid, so a caller slices a Render
/// Frame's rows with them rather than bounds-checking a Position at a time.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VisiblePositions {
    /// The columns to draw, left to right.
    pub(crate) columns: Range<usize>,
    /// The rows to draw, top to bottom.
    pub(crate) rows: Range<usize>,
}

impl VisiblePositions {
    /// No Position at all: what a console showing none of the Grid draws.
    pub(crate) fn empty() -> Self {
        Self {
            columns: 0..0,
            rows: 0..0,
        }
    }

    /// How many Positions the two ranges cover between them.
    pub(crate) fn count(&self) -> usize {
        self.columns.len().saturating_mul(self.rows.len())
    }
}

impl GridViewport {
    ///
    /// The scale from the Source's own coordinates to presented points.
    ///
    /// `source` is the Grid measured in the Cell size the Source is laid out
    /// with, so this is the one factor both axes are presented under. A console
    /// with no area to spend answers zero.
    ///
    pub(crate) fn scale(&self, source: Rect) -> f32 {
        let scale = self.rect.width() / source.width();
        if scale.is_normal() { scale } else { 0.0 }
    }

    ///
    /// The transform that presents this viewport: the scale and translation
    /// that carry the Source's own coordinates onto it.
    ///
    /// This is the fit rebuilt rather than borrowed. `egui::Scene` computes the
    /// same thing in a private `fit_to_rect_in_scene`, and the numbers it needs
    /// are the ones this viewport already holds: one scale for both axes, and
    /// the translation that puts the Source's corner on the viewport's.
    ///
    /// A console with no area answers a scale of zero, which is no fit to
    /// reach; the caller guards that rather than this returning a transform it
    /// cannot invert.
    ///
    pub(crate) fn fit_transform(&self, source: Rect) -> TSTransform {
        let scale = self.scale(source);

        TSTransform::new(
            self.rect.min.to_vec2() - scale * source.min.to_vec2(),
            scale,
        )
    }

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
    /// rather than a search: no Cell is hit-tested. `columns` and `rows` bound
    /// the answer, because a point on the Grid's far edge divides to one past
    /// the last Cell. It is the inverse of [`Self::cell_rect`] — see
    /// [`Self::cell_index`] for what that costs in `f32` — so a click cannot
    /// resolve to a Cell other than the one drawn under it.
    ///
    pub(crate) fn cell_at(
        &self,
        point: Pos2,
        columns: usize,
        rows: usize,
    ) -> Option<(usize, usize)> {
        if columns == 0 || rows == 0 || !(self.cell_size.is_finite() && self.cell_size > 0.0) {
            return None;
        }
        if !self.rect.contains(point) {
            return None;
        }
        Some((
            self.cell_index(point.x, self.rect.min.x, columns),
            self.cell_index(point.y, self.rect.min.y, rows),
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
    pub(crate) fn visible_positions(
        &self,
        clip: Rect,
        columns: usize,
        rows: usize,
    ) -> VisiblePositions {
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
        // The degenerate viewport and the empty Grid are refused by `cell_at`
        // rather than by two more guards here.
        let (Some((first_column, first_row)), Some((last_column, last_row))) = (
            self.cell_at(visible.min, columns, rows),
            self.cell_at(visible.max, columns, rows),
        ) else {
            return VisiblePositions::empty();
        };

        VisiblePositions {
            columns: first_column.saturating_sub(1)..last_column.saturating_add(2).min(columns),
            rows: first_row.saturating_sub(1)..last_row.saturating_add(2).min(rows),
        }
    }
}

///
/// The largest viewport with square Cells that fits `available`, centred so the
/// surplus becomes letterboxing.
///
pub(crate) fn grid_viewport(available: Rect, columns: usize, rows: usize) -> GridViewport {
    let columns = columns as f32;
    let rows = rows as f32;
    // One Cell size for both axes is what keeps a Cell square: neither axis can
    // be stretched without the other, whatever shape the console is.
    let cell_size = (available.width() / columns)
        .min(available.height() / rows)
        .max(0.0);
    let cell_size = if cell_size.is_finite() {
        cell_size
    } else {
        0.0
    };

    GridViewport {
        cell_size,
        rect: Rect::from_center_size(
            available.center(),
            Vec2::new(columns * cell_size, rows * cell_size),
        ),
    }
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
/// The snap floors rather than rounds, and the Grid is re-centred on the
/// rectangle the transform asked for afterwards. Rounding up would let a Grid
/// of `columns` Cells exceed the area the fit gave it by half a pixel per
/// column — twenty points at the default forty — and the surplus would be
/// clipped rather than letterboxed. Flooring spends the same error as
/// letterboxing, which is what the surplus already is.
///
pub(crate) fn presented_grid(
    to_global: TSTransform,
    source: Rect,
    columns: usize,
    rows: usize,
    pixels_per_point: f32,
) -> GridViewport {
    let presented = to_global * source;
    // The device scale is divided by as well as multiplied by — once for the
    // Cell size and again for the corner — so it is refused on the same terms
    // as every other input here rather than checked for finiteness alone. Zero
    // answers an infinite Cell and a NaN corner; a negative one answers a Cell
    // that `cell_rect` paints inverted and `cell_at` refuses every click on.
    // Refused, the Grid keeps its unsnapped corner and no Cell at all, which is
    // the same nothing a console with no area presents.
    let device_scale =
        (pixels_per_point.is_finite() && pixels_per_point > 0.0).then_some(pixels_per_point);
    let raw = presented.width() / columns.max(1) as f32;
    // At least one physical pixel wherever there is any Cell at all, so a Grid
    // that is merely very small is still drawn rather than floored away.
    let cell_size = match device_scale {
        Some(scale) if raw.is_finite() && raw > 0.0 => (raw * scale).floor().max(1.0) / scale,
        _ => 0.0,
    };
    let size = Vec2::new(columns as f32, rows as f32) * cell_size;
    let corner = presented.center() - size / 2.0;

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

    use super::{GridViewport, VisiblePositions, grid_viewport, presented_grid};

    const GRID: usize = 32;

    fn area(width: f32, height: f32) -> Rect {
        Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(width, height))
    }

    fn cell_sides(viewport: GridViewport, columns: usize, rows: usize) -> (f32, f32) {
        (
            viewport.rect.width() / columns as f32,
            viewport.rect.height() / rows as f32,
        )
    }

    fn assert_close(left: f32, right: f32, what: &str) {
        assert!((left - right).abs() < 1e-3, "{what}: {left} is not {right}");
    }

    #[test]
    fn a_cell_is_square_in_wide_tall_and_square_consoles() {
        for available in [
            area(1200.0, 700.0),
            area(700.0, 1200.0),
            area(800.0, 800.0),
            area(301.0, 217.0),
        ] {
            let viewport = grid_viewport(available, GRID, GRID);
            let (width, height) = cell_sides(viewport, GRID, GRID);

            assert_close(width, height, "Cell axes");
            assert_close(width, viewport.cell_size, "answered Cell size");
        }
    }

    #[test]
    fn a_cell_is_square_even_when_the_grid_is_not() {
        let viewport = grid_viewport(area(1200.0, 700.0), 10, 4);
        let (width, height) = cell_sides(viewport, 10, 4);

        assert_close(width, height, "Cell axes");
        assert_close(viewport.cell_size, 120.0, "Cell size");
    }

    #[test]
    fn a_square_grid_is_presented_square_and_centred_in_the_surplus() {
        let available = area(1200.0, 700.0);
        let viewport = grid_viewport(available, GRID, GRID);

        assert_close(
            viewport.rect.width(),
            viewport.rect.height(),
            "viewport axes",
        );
        assert_close(viewport.rect.height(), 700.0, "filled axis");
        assert_close(
            viewport.rect.left() - available.left(),
            available.right() - viewport.rect.right(),
            "horizontal letterboxing",
        );
        assert_close(viewport.rect.center().y, available.center().y, "centre");
    }

    #[test]
    fn a_tall_console_letterboxes_above_and_below() {
        let available = area(700.0, 1200.0);
        let viewport = grid_viewport(available, GRID, GRID);

        assert_close(viewport.rect.width(), 700.0, "filled axis");
        assert_close(
            viewport.rect.top() - available.top(),
            available.bottom() - viewport.rect.bottom(),
            "vertical letterboxing",
        );
        assert_close(viewport.rect.center().x, available.center().x, "centre");
    }

    #[test]
    fn resizing_never_stretches_one_cell_axis_past_the_other() {
        for width in [120.0_f32, 301.0, 640.0, 1201.0, 2560.0] {
            for height in [90.0_f32, 217.0, 480.0, 1199.0, 1440.0] {
                let available = area(width, height);
                let viewport = grid_viewport(available, GRID, GRID);
                let (cell_width, cell_height) = cell_sides(viewport, GRID, GRID);

                assert_close(cell_width, cell_height, "Cell axes");
                assert!(
                    viewport.rect.width() <= available.width() + 1e-3
                        && viewport.rect.height() <= available.height() + 1e-3,
                    "the viewport {viewport:?} left the console area {available:?}"
                );
                // The viewport is the largest that fits, so it fills the
                // shorter axis exactly.
                let filled = (viewport.rect.width() - available.width()).abs() < 1e-3
                    || (viewport.rect.height() - available.height()).abs() < 1e-3;
                assert!(filled, "the viewport {viewport:?} wasted both axes");
            }
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
    /// Today's console never reaches it — `show_source` paints at the
    /// `CELL_SIZE` constant, where the division is exact for every origin — so
    /// this pins the inverse before `source-grid-rendering/03` moves the
    /// transform into the console and starts painting at a fitted Cell size.
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
                    viewport.cell_at(corner, GRID, GRID),
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
                        viewport.cell_at(rect.min, GRID, GRID),
                        Some((column, row)),
                        "the corner of Cell ({column}, {row}) at a Cell size of {cell_size}"
                    );
                    assert_eq!(
                        viewport.cell_at(rect.center(), GRID, GRID),
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
        let viewport = grid_viewport(area(1200.0, 700.0), 10, 4);

        for row in 0..4 {
            for column in 0..10 {
                let rect = viewport.cell_rect(column, row);

                assert_eq!(viewport.cell_at(rect.center(), 10, 4), Some((column, row)));
                assert_eq!(viewport.cell_at(rect.min, 10, 4), Some((column, row)));
                assert_close(rect.width(), viewport.cell_size, "Cell width");
                assert_close(rect.height(), viewport.cell_size, "Cell height");
            }
        }
    }

    ///
    /// The Grid's far corner belongs to the last Cell rather than to a Cell one
    /// past it, and everything outside the Grid belongs to no Cell at all —
    /// which is what leaves the letterboxing to the Scene.
    ///
    #[test]
    fn the_grid_edge_and_the_letterboxing_resolve_to_no_cell_past_the_last() {
        let available = area(1200.0, 700.0);
        let viewport = grid_viewport(available, GRID, GRID);

        assert_eq!(
            viewport.cell_at(viewport.rect.max, GRID, GRID),
            Some((GRID - 1, GRID - 1))
        );
        assert_eq!(
            viewport.cell_at(viewport.rect.min, GRID, GRID),
            Some((0, 0))
        );
        assert_eq!(
            viewport.cell_at(available.left_center(), GRID, GRID),
            None,
            "a point in the letterboxing answered a Cell"
        );
        assert_eq!(viewport.cell_at(Pos2::new(f32::NAN, 0.0), GRID, GRID), None);

        // The wide console above letterboxes left and right, so `left_center`
        // is the surplus and `top_center` is inside the Grid. A tall console
        // swaps the two, and the assertion is only about letterboxing if both
        // orientations are asked.
        let tall = area(700.0, 1200.0);
        let viewport = grid_viewport(tall, GRID, GRID);

        assert_eq!(
            viewport.cell_at(tall.center_top(), GRID, GRID),
            None,
            "a point in the letterboxing above a tall Grid answered a Cell"
        );
        assert_eq!(
            viewport.cell_at(tall.left_center(), GRID, GRID),
            Some((0, GRID / 2)),
            "a tall console letterboxes above and below, not left and right"
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
        let viewport = grid_viewport(area(800.0, 800.0), GRID, GRID);
        assert_close(viewport.cell_size, 25.0, "Cell size");
        // Deliberately not on Cell boundaries: a clip that ends exactly on one
        // would not distinguish the margin from the rounding.
        let clip = Rect::from_min_max(
            viewport.rect.min + Vec2::splat(110.0),
            viewport.rect.min + Vec2::splat(290.0),
        );

        let visible = viewport.visible_positions(clip, GRID, GRID);

        // Columns 4 through 11 are shown, so the range runs 3 through 12.
        assert_eq!(
            visible,
            VisiblePositions {
                columns: 3..13,
                rows: 3..13,
            }
        );
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
        let viewport = grid_viewport(available, GRID, GRID);

        let visible = viewport.visible_positions(available, GRID, GRID);

        assert_eq!(
            visible,
            VisiblePositions {
                columns: 0..GRID,
                rows: 0..GRID,
            }
        );
        assert_eq!(visible.count(), GRID * GRID);
    }

    ///
    /// A clip that shows no part of the Grid, a Grid with no Cell, and a
    /// console with no area all range over nothing — which draws nothing rather
    /// than drawing a Position that is not there.
    ///
    #[test]
    fn a_console_showing_no_part_of_the_grid_ranges_over_nothing() {
        let available = area(800.0, 800.0);
        let viewport = grid_viewport(available, GRID, GRID);

        for clip in [
            Rect::from_min_size(Pos2::new(2000.0, 2000.0), Vec2::splat(100.0)),
            Rect::from_min_size(Pos2::new(-2000.0, 20.0), Vec2::splat(100.0)),
            Rect::ZERO,
            Rect::NOTHING,
        ] {
            let visible = viewport.visible_positions(clip, GRID, GRID);

            assert_eq!(visible.count(), 0, "{clip:?} reached {visible:?}");
        }

        assert_eq!(viewport.visible_positions(available, 0, 0).count(), 0);
        assert_eq!(
            grid_viewport(Rect::ZERO, GRID, GRID)
                .visible_positions(available, GRID, GRID)
                .count(),
            0
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
        let viewport = grid_viewport(area(800.0, 800.0), GRID, GRID);
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
            let visible = viewport.visible_positions(clip, GRID, GRID);

            assert_eq!(visible.count(), 0, "{clip:?} reached {visible:?}");
        }
    }

    #[test]
    fn a_viewport_with_no_area_answers_no_cell() {
        let viewport = grid_viewport(Rect::ZERO, GRID, GRID);

        assert_eq!(viewport.cell_at(Pos2::ZERO, GRID, GRID), None);
        assert_eq!(
            grid_viewport(area(100.0, 100.0), GRID, GRID).cell_at(Pos2::new(10.0, 20.0), 0, 0),
            None
        );
    }

    #[test]
    fn a_console_with_no_area_presents_no_viewport() {
        let viewport = grid_viewport(Rect::ZERO, GRID, GRID);

        assert_eq!(viewport.cell_size, 0.0);
        assert_eq!(viewport.rect.size(), Vec2::ZERO);
        assert_eq!(
            viewport.scale(Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0))),
            0.0
        );
    }

    ///
    /// The fit the console owns puts the Source exactly where the fitted
    /// viewport says it goes — which is what lets `egui::Scene`, and the
    /// private `fit_to_rect_in_scene` inside it, be retired rather than
    /// reached for.
    ///
    #[test]
    fn the_fit_transform_presents_exactly_the_fitted_viewport() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(GRID as f32 * 25.0));
        for available in [area(1200.0, 700.0), area(700.0, 1200.0), area(800.0, 800.0)] {
            let viewport = grid_viewport(available, GRID, GRID);
            let presented = viewport.fit_transform(source) * source;

            assert_close(presented.left(), viewport.rect.left(), "presented left");
            assert_close(presented.top(), viewport.rect.top(), "presented top");
            assert_close(presented.width(), viewport.rect.width(), "presented width");
            assert_close(
                presented.height(),
                viewport.rect.height(),
                "presented height",
            );
        }
    }

    ///
    /// The Grid the fit presents is the fitted viewport, Cell for Cell, so
    /// nothing about owning the transform moves the square-Cell fit or the
    /// letterboxing it produces.
    ///
    #[test]
    fn the_fit_transform_presents_the_viewport_it_was_built_from() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(GRID as f32 * 25.0));
        for available in [area(1200.0, 700.0), area(700.0, 1200.0), area(800.0, 800.0)] {
            let viewport = grid_viewport(available, GRID, GRID);
            let presented = presented_grid(viewport.fit_transform(source), source, GRID, GRID, 1.0);

            // Whole physical pixels, so the presented Cell is at most a pixel
            // short of the fit and the Grid is at most `GRID` pixels narrower.
            assert!(
                viewport.cell_size - presented.cell_size < 1.0
                    && presented.cell_size <= viewport.cell_size,
                "{} is not the snapped {}",
                presented.cell_size,
                viewport.cell_size
            );
            assert_close(
                presented.rect.center().x,
                viewport.rect.center().x,
                "presented centre x",
            );
            assert_close(
                presented.rect.center().y,
                viewport.rect.center().y,
                "presented centre y",
            );
            assert!(
                presented.rect.width() <= available.width() + 1e-3
                    && presented.rect.height() <= available.height() + 1e-3,
                "the presented Grid {:?} left the console area {available:?}",
                presented.rect
            );
        }
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
                    8,
                    8,
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
            let before = presented_grid(TSTransform::from_scaling(scaling), source, 8, 8, 1.0);
            let after = presented_grid(TSTransform::new(moved, scaling), source, 8, 8, 1.0);

            assert_close(after.rect.min.x - before.rect.min.x, moved.x, "panned x");
            assert_close(after.rect.min.y - before.rect.min.y, moved.y, "panned y");
            assert_close(after.cell_size, before.cell_size, "a pan changed the zoom");
        }
    }

    ///
    /// A console with no area answers a transform that cannot be inverted, and
    /// a Grid with no Cell to click. The console replaces the transform before
    /// it reaches `inverse()`; this pins the half that belongs to the geometry.
    ///
    #[test]
    fn a_degenerate_fit_presents_no_cell_rather_than_a_nan_one() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0));
        let viewport = grid_viewport(Rect::ZERO, GRID, GRID);

        assert_eq!(viewport.fit_transform(source).scaling, 0.0);
        assert!(!viewport.fit_transform(source).is_valid());

        let presented = presented_grid(viewport.fit_transform(source), source, GRID, GRID, 1.0);

        assert_eq!(presented.cell_size, 0.0);
        assert_eq!(presented.cell_at(Pos2::ZERO, GRID, GRID), None);
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
            let presented = presented_grid(TSTransform::IDENTITY, source, 8, 8, pixels_per_point);

            assert_eq!(
                presented.cell_size, 0.0,
                "a device scale of {pixels_per_point} presented a Cell"
            );
            assert!(
                presented.rect.min.x.is_finite() && presented.rect.min.y.is_finite(),
                "a device scale of {pixels_per_point} put the Grid at {:?}",
                presented.rect.min
            );
            assert_eq!(presented.cell_at(Pos2::ZERO, 8, 8), None);
        }
    }
}
