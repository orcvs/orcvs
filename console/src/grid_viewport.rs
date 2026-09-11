//! Source Grid viewport geometry.
//!
//! The console spends the area the window leaves it on the largest Grid
//! viewport whose Cells are square, and centres that viewport so the surplus on
//! the longer axis falls away as letterboxing on both sides. The geometry lives
//! here, apart from the rendering, so the wide, tall and square cases are
//! settled by arithmetic a test can ask about without a window.

use egui::{Pos2, Rect, Vec2};

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
    /// The region of the Source an [`egui::Scene`] must show to present this
    /// viewport.
    ///
    /// A Scene fits the region it is given into the available area under one
    /// scale factor for both axes and centres it there, so the region that
    /// presents this viewport is the available area measured in Source
    /// coordinates, centred on the Source. The surplus the letterboxing covers
    /// is part of the region: it is Source coordinate space that holds no Cell.
    ///
    pub(crate) fn scene_view(&self, source: Rect, available: Rect) -> Rect {
        let scale = self.scale(source);
        if scale <= 0.0 {
            return source;
        }
        Rect::from_center_size(source.center(), available.size() / scale)
    }

    ///
    /// The rectangle the Cell at `column` and `row` occupies.
    ///
    /// A Cell's rectangle is a function of its Position and nothing else. The
    /// Cursor, the selection and the caret phase reach the paint a Cell is
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
    /// This is the whole of what a click has to answer, and it is a division
    /// rather than a search: no Cell is hit-tested. `columns` and `rows` bound
    /// the answer, because a point on the Grid's far edge divides to one past
    /// the last Cell. It is the inverse of [`Self::cell_rect`], so a click
    /// cannot resolve to a Cell other than the one drawn under it.
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
        let offset = point - self.rect.min;
        // Both offsets are non-negative inside the Grid, so truncation is the
        // floor, and the clamp is for the far edge alone.
        let column = (offset.x / self.cell_size) as usize;
        let row = (offset.y / self.cell_size) as usize;

        Some((column.min(columns - 1), row.min(rows - 1)))
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

#[cfg(test)]
mod tests {
    use egui::{Pos2, Rect, Vec2, emath::TSTransform};

    use super::{GridViewport, grid_viewport};

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
    /// Mirrors the fit `egui::Scene` applies to the region it is shown: one
    /// scale for both axes, centred on the region.
    ///
    fn scene_fit(region: Rect, available: Rect) -> TSTransform {
        let scale = (available.size() / region.size()).min_elem();
        TSTransform::from_translation(
            available.center().to_vec2() - scale * region.center().to_vec2(),
        ) * TSTransform::from_scaling(scale)
    }

    #[test]
    fn the_scene_view_presents_exactly_the_fitted_viewport() {
        let source = Rect::from_min_size(Pos2::ZERO, Vec2::splat(GRID as f32 * 25.0));
        for available in [area(1200.0, 700.0), area(700.0, 1200.0), area(800.0, 800.0)] {
            let viewport = grid_viewport(available, GRID, GRID);
            let presented = scene_fit(viewport.scene_view(source, available), available) * source;

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
}
