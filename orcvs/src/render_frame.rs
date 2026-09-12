use crate::{
    glyph::Glyph,
    grid::{Grid, Position},
    opts::{HighlightSpacing, MarkerSpacing},
    source::SourceRevision,
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
}

#[derive(Clone, Debug)]
pub struct RenderFrame {
    grid: Grid,
    cursor: Position,
    cursor_visible: bool,
    marker_spacing: MarkerSpacing,
    highlight_dot_spacing: HighlightSpacing,
    cells: Vec<RenderCell>,
}

impl RenderFrame {
    pub(crate) fn derive(
        source: SourceRevision,
        selected: Position,
        cursor_visible: bool,
        config: RenderFrameConfig,
    ) -> Self {
        let grid = source.grid();
        grid.assert_owns(selected);
        let cells = grid
            .positions_by_row()
            .flatten()
            .map(|position| RenderCell {
                position,
                content: source.content_at(position),
                glyph: source
                    .language_map()
                    .glyph_at(position)
                    .unwrap_or(Glyph::Space),
            })
            .collect();
        Self {
            grid,
            cursor: selected,
            cursor_visible,
            marker_spacing: config.marker_spacing,
            highlight_dot_spacing: config.highlight_dot_spacing,
            cells,
        }
    }

    ///
    /// The Grid this Render Frame was derived from.
    ///
    /// Carried rather than recovered. The derivation already holds it to assert
    /// the selected Position belongs to it, and `Grid` is `Copy`, so keeping
    /// the fact costs nothing.
    ///
    pub fn grid(&self) -> Grid {
        self.grid
    }

    ///
    /// The Cursor: the Position the derivation was given and asserted
    /// the Grid owns.
    ///
    /// Carried rather than recovered. The derivation already holds the Position, and
    /// `Position` is `Copy`, so keeping it costs nothing. It spares every
    /// consumer scanning the Cells for the one whose `selected` flag is set —
    /// a search whose answer the type of `&[RenderCell]` cannot state.
    ///
    pub fn cursor(&self) -> Position {
        self.cursor
    }

    ///
    /// Whether the Cursor is on the visible half of its blink for this Frame.
    ///
    /// Carried from derivation rather than recovered from a Cell
    /// flag. The blink is a fact about the Frame, not about any one Cell.
    ///
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    ///
    /// The sector-seam period, in Cells, this Frame was derived with.
    ///
    /// Presentation configuration handed across so the console can draw the
    /// seams; the period itself still lives on `Opts`.
    ///
    pub fn marker_spacing(&self) -> MarkerSpacing {
        self.marker_spacing
    }

    ///
    /// The Cursor Bloom radius, in Cells, this Frame was derived with.
    ///
    /// Presentation configuration handed across so the console can grade the
    /// bloom; the radius itself still lives on `Opts`.
    ///
    pub fn highlight_dot_spacing(&self) -> HighlightSpacing {
        self.highlight_dot_spacing
    }

    ///
    /// The Cell at `position`.
    ///
    /// Indexed through [`Grid::index`] in row-major order over the flat Cells.
    ///
    pub fn at(&self, position: Position) -> &RenderCell {
        self.grid.assert_owns(position);
        &self.cells[self.grid.index(position).get()]
    }

    ///
    /// Every Cell of the Grid in row-major order.
    ///
    pub fn cells(&self) -> &[RenderCell] {
        &self.cells
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
        frame.at(position)
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

        assert_eq!(frame.grid().rows(), 2);
        assert_eq!(frame.grid().columns(), 2);
        assert_eq!(frame.cursor(), selected);
        assert!(frame.cursor_visible());
        assert_eq!(
            frame.at(grid.position(0, 0).unwrap()).position(),
            grid.position(0, 0).unwrap()
        );
        assert_eq!(frame.at(grid.position(0, 0).unwrap()).glyph(), Glyph::Space);
        assert_eq!(frame.at(grid.position(1, 0).unwrap()).content(), Some('x'));
        // A character standing where a Function goes is classified there,
        // whether or not the table holds its spelling.
        assert_eq!(
            frame.at(grid.position(1, 0).unwrap()).glyph(),
            Glyph::Function
        );
        assert_eq!(frame.at(grid.position(0, 1).unwrap()).glyph(), Glyph::Space);
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

        assert_eq!(frame.at(grid.position(0, 0).unwrap()).glyph(), Glyph::Bang);
        assert_eq!(frame.at(grid.position(1, 0).unwrap()).glyph(), Glyph::Bang);
        // The third `*` is not half a Bang. It opens an Expression of its own
        // whose spelling `*x` the Function table does not hold, and the `x`
        // opens the one after that — each classified where a Function goes,
        // because that is where each of them stands.
        assert_eq!(
            frame.at(grid.position(2, 0).unwrap()).glyph(),
            Glyph::Function
        );
        assert_eq!(
            frame.at(grid.position(3, 0).unwrap()).glyph(),
            Glyph::Function
        );
    }

    #[test]
    fn a_self_banging_function_is_painted_as_a_function() {
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

        // `>>` painted as an ordinary character while it was its own Atom
        // variant, which mapped to `Glyph::Char`. It is a row of the Function
        // table now, so it is painted where every other Function is. The change
        // is visible and it is a correction: these two Cells spell a Function.
        assert_eq!(
            frame.at(grid.position(0, 0).unwrap()).glyph(),
            Glyph::Function
        );
        assert_eq!(
            frame.at(grid.position(1, 0).unwrap()).glyph(),
            Glyph::Function
        );
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
        // A lone character is the first Cell of a spelling the Function table
        // does not hold, which is a classification like any other. What this
        // test is about is that it survives sector presentation at all.
        assert_eq!(cell_at(&frame, grid.origin()).glyph(), Glyph::Function);
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

    #[test]
    fn render_frame_answers_the_presentation_spacings_it_was_derived_with() {
        let grid = Grid::new(2, 2);
        let source = SourceCommander::new(grid);
        let marker_spacing = MarkerSpacing::new(3).unwrap();
        let highlight_dot_spacing = HighlightSpacing::new(5).unwrap();

        let frame = RenderFrame::derive(
            source.read_revision(),
            grid.origin(),
            false,
            RenderFrameConfig {
                marker_spacing,
                highlight_dot_spacing,
            },
        );

        assert_eq!(frame.marker_spacing(), marker_spacing);
        assert_eq!(frame.highlight_dot_spacing(), highlight_dot_spacing);
    }
}
