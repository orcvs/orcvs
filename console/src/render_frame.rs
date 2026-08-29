use crate::{
    glyph::Glyph,
    grid::Position,
    opts::{HighlightSpacing, MarkerSpacing},
    source::SourceRevisionCells,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderFrameConfig {
    pub marker_spacing: MarkerSpacing,
    pub highlight_dot_spacing: HighlightSpacing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CursorBloom {
    Core,
    Inner,
    Mid,
    Outer,
}

#[derive(Clone, Debug)]
pub struct RenderCell {
    position: Position,
    content: Option<char>,
    glyph: Glyph,
    cursor_bloom: Option<CursorBloom>,
    sector_left_strength: Option<u8>,
    sector_top_strength: Option<u8>,
    selected: bool,
    cursor_visible: bool,
}

impl RenderCell {
    pub fn position(&self) -> Position {
        self.position
    }

    pub fn content(&self) -> Option<char> {
        self.content
    }

    pub fn glyph(&self) -> Glyph {
        self.glyph
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub(crate) fn cursor_bloom(&self) -> Option<CursorBloom> {
        self.cursor_bloom
    }

    pub(crate) fn sector_left_strength(&self) -> Option<u8> {
        self.sector_left_strength
    }

    pub(crate) fn sector_top_strength(&self) -> Option<u8> {
        self.sector_top_strength
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }
}

#[derive(Clone, Debug)]
pub struct RenderFrame {
    rows: Vec<Vec<RenderCell>>,
}

impl RenderFrame {
    pub(crate) fn derive(
        source: SourceRevisionCells,
        selected: Position,
        cursor_visible: bool,
        config: RenderFrameConfig,
    ) -> Self {
        source.grid.assert_owns(selected);
        let rows = source
            .grid
            .rows()
            .map(|row| {
                row.map(|position| {
                    let cell = &source.cells[source.grid.index(position)];
                    let is_selected = position == selected;
                    let marker_spacing = config.marker_spacing.cells();
                    RenderCell {
                        position,
                        content: cell.content,
                        glyph: cell.glyph.unwrap_or(Glyph::Space),
                        cursor_bloom: cursor_bloom(position, selected, config),
                        sector_left_strength: (position.x() > 0
                            && position.x().is_multiple_of(marker_spacing))
                        .then(|| sector_seam_strength(position.y(), marker_spacing, position))
                        .flatten(),
                        sector_top_strength: (position.y() > 0
                            && position.y().is_multiple_of(marker_spacing))
                        .then(|| sector_seam_strength(position.x(), marker_spacing, position))
                        .flatten(),
                        selected: is_selected,
                        cursor_visible: is_selected && cursor_visible,
                    }
                })
                .collect()
            })
            .collect();
        Self { rows }
    }

    pub fn rows(&self) -> &[Vec<RenderCell>] {
        &self.rows
    }
}

fn cursor_bloom(
    position: Position,
    selected: Position,
    config: RenderFrameConfig,
) -> Option<CursorBloom> {
    let dx = position.x().abs_diff(selected.x());
    let dy = position.y().abs_diff(selected.y());
    // A Cartesian distance produces a Cell-aligned focus matrix instead of a
    // radial pool of light.
    let distance = dx.max(dy);
    let radius = config.highlight_dot_spacing.cells();
    classify_cursor_bloom(
        distance.saturating_add(signal_breakup(position, distance, radius)),
        radius,
    )
}

fn signal_breakup(position: Position, distance: usize, radius: usize) -> usize {
    let hash = cell_hash(position);
    let broken = if distance == radius {
        !hash.is_multiple_of(3)
    } else {
        hash.is_multiple_of(2)
    };
    usize::from(broken)
}

fn cell_hash(position: Position) -> usize {
    let mut hash = position.x().wrapping_mul(0x9E37_79B1) ^ position.y().wrapping_mul(0x85EB_CA77);
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
    let index = distance_from_corner
        .saturating_mul(SECTOR_SEAM_STRENGTHS.len())
        .checked_div(half_spacing)
        .unwrap_or_default()
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
    use std::sync::{Arc, Barrier};

    use crate::{
        glyph::Glyph,
        grid::Grid,
        opts::{HighlightSpacing, MarkerSpacing},
        render_frame::{RenderFrame, RenderFrameConfig},
        source::SourceCommander,
    };

    fn cell_at(frame: &RenderFrame, position: crate::grid::Position) -> &super::RenderCell {
        frame
            .rows()
            .iter()
            .flatten()
            .find(|cell| cell.position() == position)
            .expect("Render Frame contains every Grid Position")
    }

    #[test]
    fn render_frame_is_a_complete_row_structured_visual_snapshot() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        source.set(1, "x").unwrap();
        let selected = grid.position(1, 0).unwrap();

        let frame = RenderFrame::derive(
            source.read_revision_cells(),
            selected,
            true,
            RenderFrameConfig {
                marker_spacing: MarkerSpacing::new(2).unwrap(),
                highlight_dot_spacing: HighlightSpacing::new(1).unwrap(),
            },
        );

        assert_eq!(frame.rows().len(), 2);
        assert_eq!(frame.rows()[0].len(), 2);
        assert_eq!(frame.rows()[1].len(), 2);
        assert_eq!(frame.rows()[0][0].position(), grid.position(0, 0).unwrap());
        assert_eq!(frame.rows()[0][0].glyph(), Glyph::Space);
        assert_eq!(frame.rows()[0][1].content(), Some('x'));
        assert_eq!(frame.rows()[0][1].glyph(), Glyph::Char);
        assert!(frame.rows()[0][1].selected());
        assert!(frame.rows()[0][1].cursor_visible());
        assert_eq!(frame.rows()[0][0].sector_left_strength(), None);
        assert_eq!(frame.rows()[0][0].sector_top_strength(), None);
        assert_eq!(frame.rows()[0][1].sector_left_strength(), None);
        assert_eq!(frame.rows()[1][0].glyph(), Glyph::Space);
        assert!(!frame.rows()[1][0].selected());
        assert!(!frame.rows()[1][0].cursor_visible());
    }

    #[test]
    fn occupied_glyphs_win_over_sector_presentation() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        source.set(0, "x").unwrap();

        let frame = RenderFrame::derive(
            source.read_revision_cells(),
            grid.origin(),
            false,
            RenderFrameConfig {
                marker_spacing: MarkerSpacing::new(1).unwrap(),
                highlight_dot_spacing: HighlightSpacing::new(1).unwrap(),
            },
        );

        assert_eq!(cell_at(&frame, grid.origin()).content(), Some('x'));
        assert_eq!(cell_at(&frame, grid.origin()).glyph(), Glyph::Char);
    }

    #[test]
    fn cursor_field_is_local_and_centred_on_the_cursor() {
        let grid = Grid::new(24, 16);
        let source = SourceCommander::new(grid);
        let selected = grid.position(9, 7).unwrap();

        let frame = RenderFrame::derive(
            source.read_revision_cells(),
            selected,
            false,
            RenderFrameConfig {
                marker_spacing: MarkerSpacing::new(8).unwrap(),
                highlight_dot_spacing: HighlightSpacing::new(7).unwrap(),
            },
        );

        assert_eq!(
            cell_at(&frame, selected).cursor_bloom(),
            Some(super::CursorBloom::Core)
        );
        assert_eq!(
            cell_at(&frame, grid.position(12, 10).unwrap()).cursor_bloom(),
            Some(super::CursorBloom::Mid)
        );
        assert_eq!(
            cell_at(&frame, grid.position(15, 13).unwrap()).cursor_bloom(),
            Some(super::CursorBloom::Outer)
        );
        assert_eq!(
            cell_at(&frame, grid.position(17, 7).unwrap()).cursor_bloom(),
            None
        );
        assert_eq!(
            cell_at(&frame, grid.position(12, 10).unwrap()).glyph(),
            Glyph::Space
        );
    }

    #[test]
    fn signal_breakup_chips_each_fibonacci_band_edge_outward() {
        use super::{CursorBloom, classify_cursor_bloom};

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

        assert_eq!(super::sector_seam_strength(0, 8, corner), Some(100));
        assert_eq!(
            super::sector_seam_strength(7, 8, before_corner),
            Some(100),
            "the segment approaching a corner must match the segment leaving it"
        );
        assert!(
            super::sector_seam_strength(4, 8, middle).unwrap_or_default() < 100,
            "sector seams fade midway between corners"
        );
    }

    #[test]
    fn only_the_faint_space_between_sector_corners_has_deterministic_gaps() {
        let grid = Grid::new(256, 8);
        let positions = grid.rows().flatten().collect::<Vec<_>>();

        assert!(
            positions
                .iter()
                .all(|&position| super::sector_seam_strength(0, 8, position).is_some())
        );
        assert!(
            positions
                .iter()
                .all(|&position| super::sector_seam_strength(7, 8, position).is_some())
        );

        let gaps = positions
            .iter()
            .filter(|&&position| super::sector_seam_strength(3, 8, position).is_none())
            .count();
        assert!((384..=640).contains(&gaps), "tail gaps: {gaps}");

        let position = grid.position(31, 7).unwrap();
        assert_eq!(
            super::sector_seam_strength(3, 8, position),
            super::sector_seam_strength(3, 8, position)
        );
    }

    #[test]
    fn boundary_breakup_approximates_half_inner_and_two_thirds_outer() {
        let grid = Grid::new(1024, 1);
        let inner_breaks = grid
            .rows()
            .flatten()
            .filter(|&position| super::signal_breakup(position, 4, 7) == 1)
            .count();
        let outer_breaks = grid
            .rows()
            .flatten()
            .filter(|&position| super::signal_breakup(position, 7, 7) == 1)
            .count();

        assert!((460..=564).contains(&inner_breaks), "inner: {inner_breaks}");
        assert!((614..=738).contains(&outer_breaks), "outer: {outer_breaks}");
    }

    #[test]
    fn cell_noise_is_stable_while_cursor_movement_changes_the_boundary() {
        let grid = Grid::new(32, 1);
        let edge = grid
            .rows()
            .flatten()
            .skip(7)
            .find(|&position| super::signal_breakup(position, 7, 7) == 1)
            .expect("the deterministic pattern includes a broken outer Cell");
        let config = RenderFrameConfig {
            marker_spacing: MarkerSpacing::new(8).unwrap(),
            highlight_dot_spacing: HighlightSpacing::new(7).unwrap(),
        };
        let before = grid.position(edge.x() - 7, 0).unwrap();
        let after = grid.position(edge.x() - 6, 0).unwrap();

        assert_eq!(super::signal_breakup(edge, 7, 7), 1);
        assert_eq!(super::signal_breakup(edge, 7, 7), 1);
        assert_eq!(super::cursor_bloom(edge, before, config), None);
        assert_eq!(
            super::cursor_bloom(edge, after, config),
            Some(super::CursorBloom::Outer)
        );
    }

    #[test]
    fn concurrent_ticks_cannot_mix_source_revisions_within_a_render_frame() {
        let grid = Grid::new(8, 2);
        let source = SourceCommander::new(grid);
        for (idx, content) in "++010E".chars().enumerate() {
            source.set(idx, &content.to_string()).unwrap();
        }
        source.execute();

        let start = Arc::new(Barrier::new(2));
        let writer_source = source.clone();
        let writer_start = start.clone();
        let writer = std::thread::spawn(move || {
            writer_start.wait();
            for operand in ['F', 'E'].into_iter().cycle().take(2_000) {
                writer_source.set(5, &operand.to_string()).unwrap();
                writer_source.execute();
            }
        });

        start.wait();
        for _ in 0..2_000 {
            let frame = RenderFrame::derive(
                source.read_revision_cells(),
                grid.origin(),
                false,
                RenderFrameConfig {
                    marker_spacing: MarkerSpacing::new(8).unwrap(),
                    highlight_dot_spacing: HighlightSpacing::new(2).unwrap(),
                },
            );
            let result = (
                cell_at(&frame, grid.position(0, 1).unwrap()).content(),
                cell_at(&frame, grid.position(1, 1).unwrap()).content(),
            );
            assert!(
                result == (Some('0'), Some('F')) || result == (Some('1'), Some('0')),
                "one Render Frame mixed two Source revisions: {result:?}"
            );
        }
        writer.join().unwrap();
    }
}
