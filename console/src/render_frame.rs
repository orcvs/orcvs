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

#[derive(Clone, Debug)]
pub struct RenderCell {
    position: Position,
    content: Option<char>,
    glyph: Glyph,
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
                    RenderCell {
                        position,
                        content: cell.content,
                        glyph: cell
                            .glyph
                            .unwrap_or_else(|| background_glyph(position, selected, config)),
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

fn background_glyph(position: Position, selected: Position, config: RenderFrameConfig) -> Glyph {
    let spacing = config.marker_spacing.cells();
    if position.x().is_multiple_of(spacing) && position.y().is_multiple_of(spacing) {
        return Glyph::Marker;
    }
    if in_marker_block(selected, position, config.marker_spacing)
        && position
            .x()
            .is_multiple_of(config.highlight_dot_spacing.cells())
        && position
            .y()
            .is_multiple_of(config.highlight_dot_spacing.cells())
    {
        return Glyph::Highlight;
    }
    Glyph::Space
}

fn in_marker_block(selected: Position, target: Position, spacing: MarkerSpacing) -> bool {
    let spacing = spacing.cells();
    let min_x = selected.x() / spacing * spacing;
    let end_x = (selected.x() / spacing + 1).saturating_mul(spacing);
    let min_y = selected.y() / spacing * spacing;
    let end_y = (selected.y() / spacing + 1).saturating_mul(spacing);
    target.x() >= min_x && target.x() < end_x && target.y() >= min_y && target.y() < end_y
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
        assert_eq!(frame.rows()[0][0].glyph(), Glyph::Marker);
        assert_eq!(frame.rows()[0][1].content(), Some('x'));
        assert_eq!(frame.rows()[0][1].glyph(), Glyph::Char);
        assert!(frame.rows()[0][1].selected());
        assert!(frame.rows()[0][1].cursor_visible());
        assert_eq!(frame.rows()[1][0].glyph(), Glyph::Highlight);
        assert!(!frame.rows()[1][0].selected());
        assert!(!frame.rows()[1][0].cursor_visible());
    }

    #[test]
    fn occupied_glyphs_win_over_every_background_glyph() {
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
    fn highlight_stops_at_the_exclusive_marker_block_edges() {
        let grid = Grid::new(24, 16);
        let source = SourceCommander::new(grid);
        let selected = grid.position(8, 8).unwrap();

        let frame = RenderFrame::derive(
            source.read_revision_cells(),
            selected,
            false,
            RenderFrameConfig {
                marker_spacing: MarkerSpacing::new(8).unwrap(),
                highlight_dot_spacing: HighlightSpacing::new(2).unwrap(),
            },
        );

        assert_eq!(
            cell_at(&frame, grid.position(14, 10).unwrap()).glyph(),
            Glyph::Highlight
        );
        assert_eq!(
            cell_at(&frame, grid.position(16, 10).unwrap()).glyph(),
            Glyph::Space
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
