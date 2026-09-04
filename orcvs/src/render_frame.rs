use crate::{
    glyph::Glyph,
    grid::Position,
    opts::{HighlightSpacing, MarkerSpacing},
    source::SourceRevision,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderFrameConfig {
    pub marker_spacing: MarkerSpacing,
    pub highlight_dot_spacing: HighlightSpacing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorBloom {
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

    pub fn cursor_bloom(&self) -> Option<CursorBloom> {
        self.cursor_bloom
    }

    pub fn sector_left_strength(&self) -> Option<u8> {
        self.sector_left_strength
    }

    pub fn sector_top_strength(&self) -> Option<u8> {
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
        source: SourceRevision,
        selected: Position,
        cursor_visible: bool,
        config: RenderFrameConfig,
    ) -> Self {
        source.grid().assert_owns(selected);
        let rows = source
            .grid()
            .rows()
            .map(|row| {
                row.map(|position| {
                    let is_selected = position == selected;
                    let marker_spacing = config.marker_spacing.cells();
                    RenderCell {
                        position,
                        content: source.content_at(position),
                        glyph: source
                            .language_map()
                            .glyph_at(position)
                            .unwrap_or(Glyph::Space),
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
                .collect::<Vec<_>>()
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
        grid::{CellIndex, Grid},
        opts::{HighlightSpacing, MarkerSpacing},
        render_frame::{RenderFrame, RenderFrameConfig},
        source::{SourceCommander, Tick},
    };

    ///
    /// The index `grid` mints for `idx`. A Cell is named by an index its Grid
    /// minted, so a test states the number and the Grid answers with the Cell.
    ///
    fn cell(grid: Grid, idx: usize) -> CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    fn cell_at(frame: &RenderFrame, position: crate::grid::Position) -> &super::RenderCell {
        frame
            .rows()
            .iter()
            .flatten()
            .find(|cell| cell.position() == position)
            .expect("Render Frame contains every Grid Position")
    }

    #[test]
    fn cell_noise_uses_the_wasm32_integer_domain_on_every_target() {
        let grid = Grid::new(8, 8);
        let position = grid.position(7, 5).unwrap();

        assert_eq!(super::cell_hash(position), 0xea1e_857c);
        assert_eq!(std::mem::size_of_val(&super::cell_hash(position)), 4);
    }

    #[test]
    fn render_frame_is_a_complete_row_structured_visual_snapshot() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        source.set(cell(grid, 1), "x").unwrap();
        let selected = grid.position(1, 0).unwrap();

        let frame = RenderFrame::derive(
            source.read_revision(),
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
    fn only_complete_bang_units_receive_bang_glyphs() {
        let grid = Grid::new(4, 1);
        let source = SourceCommander::new(grid);
        for (index, content) in "***x".chars().enumerate() {
            source.set(cell(grid, index), &content.to_string()).unwrap();
        }

        let frame = RenderFrame::derive(
            source.read_revision(),
            grid.origin(),
            false,
            RenderFrameConfig {
                marker_spacing: MarkerSpacing::new(2).unwrap(),
                highlight_dot_spacing: HighlightSpacing::new(1).unwrap(),
            },
        );

        assert_eq!(frame.rows()[0][0].glyph(), Glyph::Bang);
        assert_eq!(frame.rows()[0][1].glyph(), Glyph::Bang);
        assert_eq!(frame.rows()[0][2].glyph(), Glyph::Char);
        assert_eq!(frame.rows()[0][3].glyph(), Glyph::Char);
    }

    #[test]
    fn east_activation_is_not_classified_as_a_function() {
        let grid = Grid::new(2, 1);
        let source = SourceCommander::new(grid);
        source.set(cell(grid, 0), ">").unwrap();
        source.set(cell(grid, 1), ">").unwrap();

        let frame = RenderFrame::derive(
            source.read_revision(),
            grid.origin(),
            false,
            RenderFrameConfig {
                marker_spacing: MarkerSpacing::new(2).unwrap(),
                highlight_dot_spacing: HighlightSpacing::new(1).unwrap(),
            },
        );

        assert_eq!(frame.rows()[0][0].glyph(), Glyph::Char);
        assert_eq!(frame.rows()[0][1].glyph(), Glyph::Char);
    }

    #[test]
    fn occupied_glyphs_win_over_sector_presentation() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        source.set(cell(grid, 0), "x").unwrap();

        let frame = RenderFrame::derive(
            source.read_revision(),
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
            source.read_revision(),
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
    fn cursor_bloom_bands_have_cumulative_radii_of_one_two_four_and_seven() {
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
        for (idx, content) in ".+010E".chars().enumerate() {
            source.set(cell(grid, idx), &content.to_string()).unwrap();
        }
        // Tick `0` of this Playback run, before either thread starts: it is
        // what puts a committed result in row 1 for the reader to observe.
        source.execute(Tick::ZERO);

        let start = Arc::new(Barrier::new(2));
        let writer_source = source.clone();
        let writer_start = start.clone();
        let writer = std::thread::spawn(move || {
            writer_start.wait();
            // ADR 0012 numbers each Tick after the first one on from the last,
            // so the writer carries the run forward from Tick `1` rather than
            // re-running Tick `0` two thousand times. What the reader is
            // watching for is a torn Render Frame, and a Playback run this
            // test could not otherwise describe is no basis for pinning one.
            let mut tick = Tick::ZERO.next();
            for operand in ['F', 'E'].into_iter().cycle().take(2_000) {
                writer_source
                    .set(cell(grid, 5), &operand.to_string())
                    .unwrap();
                writer_source.execute(tick);
                tick = tick.next();
            }
        });

        start.wait();
        for _ in 0..2_000 {
            let frame = RenderFrame::derive(
                source.read_revision(),
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
