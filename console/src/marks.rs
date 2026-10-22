//!
//! Where Sector Seams stand — presentation the console decides from a Render
//! Frame's spacing answer. A seam is a solid line along the whole of every
//! sector edge, drawn in the Theme's `sector.seam` the way a Cell grid line is
//! drawn in `grid.border`.
//!

use orcvs::grid::Position;

/// Whether a seam runs along this Cell's left edge: every column a whole
/// multiple of `spacing`, except the Grid's own left edge.
pub(crate) fn sector_left(position: Position, spacing: usize) -> bool {
    position.x() > 0 && position.x().is_multiple_of(spacing)
}

/// Whether a seam runs along this Cell's top edge: every row a whole
/// multiple of `spacing`, except the Grid's own top edge.
pub(crate) fn sector_top(position: Position, spacing: usize) -> bool {
    position.y() > 0 && position.y().is_multiple_of(spacing)
}

#[cfg(test)]
mod tests {
    use orcvs::grid::Grid;

    use super::{sector_left, sector_top};

    #[test]
    fn a_sector_seam_runs_unbroken_along_every_sector_edge() {
        let grid = Grid::with_shape(256, 24);
        for position in grid.positions_by_row().flatten() {
            assert_eq!(
                sector_left(position, 8),
                position.x() % 8 == 0 && position.x() > 0
            );
            assert_eq!(
                sector_top(position, 8),
                position.y() % 8 == 0 && position.y() > 0
            );
        }
    }

    #[tokio::test]
    async fn sector_edges_use_one_whole_cell_spacing_without_marker_glyphs() {
        let grid = Grid::with_shape(7, 3);
        let at = |x, y| grid.position(x, y).expect("inside the Grid");

        assert_eq!(
            (0..7).map(|x| sector_left(at(x, 0), 2)).collect::<Vec<_>>(),
            vec![false, false, true, false, true, false, true]
        );
        assert!(!sector_left(at(0, 0), 1));
        assert!((1..7).all(|x| sector_left(at(x, 0), 1)));

        // Token half: empty Cells stay unclaimed. Custom seam periods are not
        // settable from outside `orcvs`, so the spacing=2/1 pattern above is
        // pure spacing arithmetic; emptiness is checked on a default Frame.
        let orcvs = orcvs::app::Orcvs::with_shape(7, 3).expect("the test runtime");
        let frame = orcvs.render_frame();
        assert!((0..7).all(|x| {
            frame
                .at(frame.grid().position(x, 0).unwrap())
                .source_paint()
                == orcvs::source::SourcePaint::Unclaimed
        }));
    }
}
