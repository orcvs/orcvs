//!
//! Sector Seam strengths — presentation the console decides from a Render
//! Frame's spacing answer.
//!

use orcvs::grid::Position;

fn cell_hash(position: Position) -> u32 {
    let mut hash = (position.x() as u32).wrapping_mul(0x9E37_79B1)
        ^ (position.y() as u32).wrapping_mul(0x85EB_CA77);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0xC2B2_AE3D);
    hash ^= hash >> 13;
    hash
}

const SECTOR_SEAM_STRENGTHS: [u8; 4] = [100, 72, 34, 13];

fn sector_seam_strength(offset: usize, spacing: usize, position: Position) -> Option<u8> {
    let offset = offset % spacing;
    let distance_from_corner = offset.min(spacing - 1 - offset);
    let half_spacing = spacing.div_ceil(2);
    let index = (distance_from_corner.saturating_mul(SECTOR_SEAM_STRENGTHS.len()) / half_spacing)
        .min(SECTOR_SEAM_STRENGTHS.len() - 1);

    // Preserve a legible four-arm registration mark, then let only the
    // phosphor-faint middle between corners acquire stable gaps from the
    // absolute Cell address.
    if index >= 2 && cell_hash(position).is_multiple_of(4) {
        None
    } else {
        Some(SECTOR_SEAM_STRENGTHS[index])
    }
}

pub(crate) fn sector_left_strength(position: Position, spacing: usize) -> Option<u8> {
    (position.x() > 0 && position.x().is_multiple_of(spacing))
        .then(|| sector_seam_strength(position.y(), spacing, position))
        .flatten()
}

pub(crate) fn sector_top_strength(position: Position, spacing: usize) -> Option<u8> {
    (position.y() > 0 && position.y().is_multiple_of(spacing))
        .then(|| sector_seam_strength(position.x(), spacing, position))
        .flatten()
}

#[cfg(test)]
mod tests {
    use orcvs::grid::Grid;

    use super::{cell_hash, sector_left_strength, sector_seam_strength};

    #[test]
    fn cell_noise_uses_the_wasm32_integer_domain_on_every_target() {
        let grid = Grid::with_shape(8, 8);
        let position = grid.position(7, 5).unwrap();

        assert_eq!(cell_hash(position), 0xea1e_857c);
        assert_eq!(std::mem::size_of_val(&cell_hash(position)), 4);
    }

    #[test]
    fn sector_seams_form_four_fading_arms_around_each_corner() {
        let grid = Grid::with_shape(16, 16);
        let corner = grid.position(8, 0).unwrap();
        let before_corner = grid.position(8, 7).unwrap();
        let middle = grid.position(8, 4).unwrap();

        assert_eq!(sector_seam_strength(0, 8, corner), Some(100));
        assert_eq!(
            sector_seam_strength(7, 8, before_corner),
            Some(100),
            "the segment approaching a corner must match the segment leaving it"
        );
        assert!(
            sector_seam_strength(4, 8, middle).unwrap_or_default() < 100,
            "sector seams fade midway between corners"
        );
    }

    #[test]
    fn only_the_faint_space_between_sector_corners_has_deterministic_gaps() {
        let grid = Grid::with_shape(256, 8);
        let positions = grid.positions_by_row().flatten().collect::<Vec<_>>();

        assert!(
            positions
                .iter()
                .all(|&position| sector_seam_strength(0, 8, position).is_some())
        );
        assert!(
            positions
                .iter()
                .all(|&position| sector_seam_strength(7, 8, position).is_some())
        );

        let gaps = positions
            .iter()
            .filter(|&&position| sector_seam_strength(3, 8, position).is_none())
            .count();
        assert!((384..=640).contains(&gaps), "tail gaps: {gaps}");

        let position = grid.position(31, 7).unwrap();
        assert_eq!(
            sector_seam_strength(3, 8, position),
            sector_seam_strength(3, 8, position)
        );
    }

    #[tokio::test]
    async fn sector_edges_use_one_whole_cell_spacing_without_marker_glyphs() {
        let grid = Grid::with_shape(7, 3);
        let at = |x, y| grid.position(x, y).expect("inside the Grid");

        assert_eq!(
            (0..7)
                .map(|x| sector_left_strength(at(x, 0), 2).is_some())
                .collect::<Vec<_>>(),
            vec![false, false, true, false, true, false, true]
        );
        assert_eq!(sector_left_strength(at(0, 0), 1), None);
        assert!((1..7).all(|x| sector_left_strength(at(x, 0), 1).is_some()));

        // Token half: empty Cells stay unclaimed. Custom seam periods are not
        // settable from outside `orcvs`, so the spacing=2/1 pattern above is
        // pure strength arithmetic; emptiness is checked on a default Frame.
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
