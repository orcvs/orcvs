//!
//! Cursor Bloom and Sector Seam strengths — presentation the console decides
//! from a Render Frame's Cursor and spacing answers.
//!

use orcvs::grid::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "retained only to decode historical Cell styling tests"
    )
)]
pub(crate) enum CursorBloom {
    Core,
    Inner,
    Mid,
    Outer,
}

#[cfg(test)]
pub(crate) fn cursor_bloom(
    position: Position,
    selected: Position,
    radius: usize,
) -> Option<CursorBloom> {
    let dx = position.x().abs_diff(selected.x());
    let dy = position.y().abs_diff(selected.y());
    // A Cartesian distance produces a Cell-aligned focus matrix instead of a
    // radial pool of light.
    let distance = dx.max(dy);
    classify_cursor_bloom(
        distance.saturating_add(signal_breakup(position, distance, radius)),
        radius,
    )
}

#[cfg(test)]
fn signal_breakup(position: Position, distance: usize, radius: usize) -> usize {
    let hash = cell_hash(position);
    let broken = if distance == radius {
        !hash.is_multiple_of(3)
    } else {
        hash.is_multiple_of(2)
    };
    usize::from(broken)
}

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
fn classify_cursor_bloom(distance: usize, radius: usize) -> Option<CursorBloom> {
    // Band widths follow 1:1:2:3, with cumulative radii 1:2:4:7.
    let scaled_distance = distance.saturating_mul(7);

    if scaled_distance <= radius {
        Some(CursorBloom::Core)
    } else if scaled_distance <= radius.saturating_mul(2) {
        Some(CursorBloom::Inner)
    } else if scaled_distance <= radius.saturating_mul(4) {
        Some(CursorBloom::Mid)
    } else if distance <= radius {
        Some(CursorBloom::Outer)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use orcvs::grid::Grid;

    use super::{
        CursorBloom, cell_hash, classify_cursor_bloom, cursor_bloom, sector_left_strength,
        sector_seam_strength, signal_breakup,
    };

    #[test]
    fn cell_noise_uses_the_wasm32_integer_domain_on_every_target() {
        let grid = Grid::new(8, 8);
        let position = grid.position(7, 5).unwrap();

        assert_eq!(cell_hash(position), 0xea1e_857c);
        assert_eq!(std::mem::size_of_val(&cell_hash(position)), 4);
    }

    #[tokio::test]
    async fn cursor_field_is_local_and_centred_on_the_cursor() {
        let mut orcvs = orcvs::app::Orcvs::new(24, 16).expect("the test runtime");
        let grid = orcvs.grid();
        let selected = grid.position(9, 7).unwrap();
        orcvs.select(selected);
        let frame = orcvs.render_frame();
        let radius = frame.cursor_bloom_radius().cells();

        assert_eq!(
            cursor_bloom(selected, selected, radius),
            Some(CursorBloom::Core)
        );
        assert_eq!(
            cursor_bloom(grid.position(12, 10).unwrap(), selected, radius),
            Some(CursorBloom::Mid)
        );
        assert_eq!(
            cursor_bloom(grid.position(15, 13).unwrap(), selected, radius),
            Some(CursorBloom::Outer)
        );
        assert_eq!(
            cursor_bloom(grid.position(17, 7).unwrap(), selected, radius),
            None
        );
        assert_eq!(frame.at(grid.position(12, 10).unwrap()).token(), None);
    }

    #[test]
    fn cursor_bloom_bands_have_cumulative_radii_of_one_two_four_and_seven() {
        assert_eq!(classify_cursor_bloom(1, 7), Some(CursorBloom::Core));
        assert_eq!(classify_cursor_bloom(2, 7), Some(CursorBloom::Inner));
        assert_eq!(classify_cursor_bloom(3, 7), Some(CursorBloom::Mid));
        assert_eq!(classify_cursor_bloom(5, 7), Some(CursorBloom::Outer));
        assert_eq!(classify_cursor_bloom(8, 7), None);
    }

    #[test]
    fn sector_seams_form_four_fading_arms_around_each_corner() {
        let grid = Grid::new(16, 16);
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
        let grid = Grid::new(256, 8);
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

    #[test]
    fn boundary_breakup_approximates_half_inner_and_two_thirds_outer() {
        let grid = Grid::new(1024, 1);
        let inner_breaks = grid
            .positions_by_row()
            .flatten()
            .filter(|&position| signal_breakup(position, 4, 7) == 1)
            .count();
        let outer_breaks = grid
            .positions_by_row()
            .flatten()
            .filter(|&position| signal_breakup(position, 7, 7) == 1)
            .count();

        assert!(
            (460..=564).contains(&inner_breaks),
            "expected the inner break count in 460..=564, observed {inner_breaks}"
        );
        assert!(
            (614..=738).contains(&outer_breaks),
            "expected the outer break count in 614..=738, observed {outer_breaks}"
        );
    }

    #[test]
    fn cell_noise_is_stable_while_cursor_movement_changes_the_boundary() {
        let grid = Grid::new(32, 1);
        let edge = grid
            .positions_by_row()
            .flatten()
            .skip(7)
            .find(|&position| signal_breakup(position, 7, 7) == 1)
            .expect("the deterministic pattern includes a broken outer Cell");
        let before = grid.position(edge.x() - 7, 0).unwrap();
        let after = grid.position(edge.x() - 6, 0).unwrap();

        assert_eq!(signal_breakup(edge, 7, 7), 1);
        assert_eq!(signal_breakup(edge, 7, 7), 1);
        assert_eq!(cursor_bloom(edge, before, 7), None);
        assert_eq!(cursor_bloom(edge, after, 7), Some(CursorBloom::Outer));
    }

    #[tokio::test]
    async fn sector_edges_use_one_whole_cell_spacing_without_marker_glyphs() {
        let grid = Grid::new(7, 3);
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
        let orcvs = orcvs::app::Orcvs::new(7, 3).expect("the test runtime");
        let frame = orcvs.render_frame();
        assert!((0..7).all(|x| {
            frame
                .at(frame.grid().position(x, 0).unwrap())
                .token()
                .is_none()
        }));
    }
}
