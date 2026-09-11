//!
//! The per-Cell decision of how one Render Frame is drawn: background, border,
//! foreground, sector seams and the character shown.
//!
//! # No geometry
//!
//! Nothing here is a `Rect`, and nothing here reads a viewport. Where a Cell
//! *is* on screen is `GridViewport`'s arithmetic and is asserted there; a
//! `Rect` in this layer would make every assertion about a colour acquire a
//! viewport before it could be written. The seam strengths a Render Frame
//! states become colours here, because `sector_line` is a pure reading of the
//! palette, while the stroke *widths* the same seams are drawn with scale with
//! the Cell side and so belong to the step that has one.
//!
//! # Why the Cells are flat
//!
//! A [`Paint`] holds one `Vec<CellPaint>` in row-major order and the [`Grid`]
//! that shapes it, and [`Paint::at`] indexes through [`Grid::index`]. This is
//! deliberately unlike `RenderFrame`, which nests a `Vec` per row: a Render
//! Frame's only consumer walks it in row order to paint it, and the nesting
//! exists to serve exactly that. A Paint is asked about one Cell — what colour
//! is the Cell at this Position — so the shape that serves it is the one the
//! Grid already addresses Cells by.
//!

// Complete and not yet wired: `show_source` still paints from its own loop and
// is rewired onto this module by the next ticket of the effort, which takes the
// allow with it. Landing the value layer and the rewire in one change would
// mean neither could be reviewed on its own.
#![allow(dead_code)]

use std::ops::Range;

use egui::Color32;

use orcvs::{
    grid::{Grid, Position},
    render_frame::RenderFrame,
};

use crate::console::CellCharacters;
use crate::style::{PALETTE, cell_visuals, sector_line};

///
/// What one Cell of a Render Frame is drawn as.
///
/// Flat, and deliberately not a `CellVisuals` alongside a filtered background.
/// A Cell carries no background exactly where `cell_visuals` asks for the
/// Source's own colour, and keeping the unfiltered answer beside the filtered
/// one would make that invariant a property of this struct — where nothing
/// exercises it — instead of a property of the derivation, which is where its
/// subtlety lives.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CellPaint {
    pub background: Option<Color32>,
    pub border: Color32,
    pub foreground: Color32,
    pub sector_left: Option<Color32>,
    pub sector_top: Option<Color32>,
    pub character: char,
}

///
/// One run of consecutive Cells of a single row that share one background
/// colour: the colour, the row, and the columns it covers.
///
/// Columns rather than a rectangle. Turning a run into the rectangle that
/// fills it is the viewport's arithmetic, and putting a `Rect` here would make
/// every assertion about coalescing acquire a viewport first.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BackgroundRun {
    pub colour: Color32,
    pub row: usize,
    pub columns: Range<usize>,
}

///
/// How one Render Frame is drawn, Cell by Cell.
///
/// Derived from a Render Frame and nothing else — no running Orcvs, no
/// `egui::Context`. That is what makes a question about what a Cell looks like
/// answerable without the harness a font atlas needs.
///
#[derive(Clone, Debug)]
pub(crate) struct Paint {
    grid: Grid,
    cursor: Position,
    cells: Vec<CellPaint>,
}

impl Paint {
    ///
    /// Reads a Render Frame and answers what each of its Cells is drawn as.
    ///
    /// `cell_visuals` is called once per Cell and is unchanged: this decides
    /// what to do with its answer, not what the answer is.
    ///
    pub(crate) fn derive(frame: &RenderFrame) -> Self {
        let grid = frame.grid();
        // Recovered in the one pass rather than searched for afterwards. The
        // Render Frame marks the Cell it was derived for and does not answer
        // the Position separately, so this is where it is read back.
        let mut cursor = None;
        // What each Cell says, read once for the nine blank spellings and never
        // per Cell. It needs no `egui::Context`: what a Cell says is a reading
        // of `GlyphString`, and only drawing it reaches the font atlas.
        let characters = CellCharacters::new();
        let cells = frame
            .rows()
            .iter()
            .flatten()
            .map(|cell| {
                if cell.selected() {
                    cursor = Some(cell.position());
                }
                let visuals = cell_visuals(
                    cell.glyph(),
                    cell.cursor_bloom(),
                    cell.selected(),
                    cell.cursor_visible(),
                );

                CellPaint {
                    // A Cell is filled only where `cell_visuals` asks for
                    // something other than the Source fill, because the panel
                    // behind the Grid is already filled with exactly that
                    // colour. The skip is the derivation's, not the struct's:
                    // a `CellPaint` carrying both the filtered background and
                    // the `CellVisuals` it came from would make this invariant
                    // a property of the value, where nothing exercises it.
                    //
                    // The colours are compared rather than the conditions
                    // behind them, so this cannot drift from `cell_visuals`.
                    // The condition it works out to is
                    // `cursor_visible || (!selected && bloom.is_none())`, which
                    // reads wrong and is right: `cell_visuals` tests
                    // `cursor_visible` *before* the bloom arm, so the Cursor's
                    // own Cell takes the Source fill even though `cursor_bloom`
                    // answers `Some(Core)` for it, and the blink alternates a
                    // background and none.
                    background: (visuals.background != PALETTE.source)
                        .then_some(visuals.background),
                    border: visuals.border,
                    foreground: visuals.foreground,
                    // A sector seam is suppressed on the Cursor's Cell, so the
                    // Cursor is never crossed by one. It is decided here rather
                    // than left to the step that draws it: in the loop this
                    // replaced the rule was structural — the selected Cell took
                    // a branch the seams were not in — and a rule that survives
                    // only as a branch shape is a rule the next reader has to
                    // rediscover.
                    //
                    // `sector_line` is pure, so the strength the Render Frame
                    // states becomes a colour here. The stroke widths are
                    // geometry and stay out of this layer.
                    sector_left: (!cell.selected())
                        .then(|| cell.sector_left_strength().map(sector_line))
                        .flatten(),
                    sector_top: (!cell.selected())
                        .then(|| cell.sector_top_strength().map(sector_line))
                        .flatten(),
                    character: characters.character(cell),
                }
            })
            .collect();

        Self {
            grid,
            // `RenderFrame::derive` takes one selected Position and asserts its
            // Grid owns it, so every Render Frame marks exactly one Cell.
            cursor: cursor.expect("a Render Frame selects one of its Cells"),
            cells,
        }
    }

    /// What the Cell at `position` is drawn as.
    pub(crate) fn at(&self, position: Position) -> &CellPaint {
        &self.cells[self.grid.index(position).get()]
    }

    ///
    /// Where the Cursor is.
    ///
    /// One Position for the whole Paint rather than a flag on every Cell:
    /// `RenderFrame::derive` takes one selected Position and asserts the Grid
    /// owns it, so exactly one exists, and a per-Cell bool would re-open a
    /// state the layer below has closed.
    ///
    pub(crate) fn cursor(&self) -> Position {
        self.cursor
    }

    ///
    /// The consecutive Cells of each row that share one background, coalesced.
    ///
    /// Derived on demand rather than stored: the per-Cell backgrounds are the
    /// one truth, and a stored copy of their runs is a second one to keep
    /// agreeing with them.
    ///
    /// A run ends at a Cell wanting a different colour, at a Cell wanting no
    /// background at all, and at the end of its row — Cells are consecutive
    /// within a row, and the row below starts somewhere else entirely.
    ///
    pub(crate) fn background_runs(&self) -> Vec<BackgroundRun> {
        let mut runs = Vec::new();

        for (row, positions) in self.grid.positions_by_row().enumerate() {
            // The run so far: its colour, and the columns it covers.
            let mut open: Option<(Color32, Range<usize>)> = None;

            for position in positions {
                let background = self.at(position).background;
                open = match (open, background) {
                    (Some((colour, columns)), Some(background)) if colour == background => {
                        Some((colour, columns.start..position.x() + 1))
                    }
                    (finished, background) => {
                        if let Some((colour, columns)) = finished {
                            runs.push(BackgroundRun {
                                colour,
                                row,
                                columns,
                            });
                        }
                        background.map(|colour| (colour, position.x()..position.x() + 1))
                    }
                };
            }

            if let Some((colour, columns)) = open {
                runs.push(BackgroundRun {
                    colour,
                    row,
                    columns,
                });
            }
        }

        runs
    }
}

#[cfg(test)]
mod tests {
    use super::{BackgroundRun, CellPaint, Paint};
    use crate::console::CellCharacters;
    use crate::style::{PALETTE, cell_visuals, sector_line};
    use egui::Color32;
    use orcvs::{app::Orcvs, grid::Grid};
    use std::ops::Range;

    ///
    /// Every Position answers the Cell the Grid indexes, not its neighbour.
    ///
    /// The Cursor's bloom grades the border across the Grid and the written
    /// Cells carry Glyph colours of their own, so an answer off by one Cell —
    /// in either axis — differs from the answer asked for.
    ///
    #[test]
    fn a_cell_is_answered_at_the_position_the_grid_indexes() {
        let mut orcvs = Orcvs::new(6, 4);
        for (x, character) in "#a#".chars().enumerate() {
            orcvs.select(orcvs.render_frame().rows()[2][x + 1].position());
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.render_frame().rows()[1][4].position());

        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let mut borders = std::collections::BTreeSet::new();
        let mut foregrounds = std::collections::BTreeSet::new();

        for cell in frame.rows().iter().flatten() {
            let visuals = cell_visuals(
                cell.glyph(),
                cell.cursor_bloom(),
                cell.selected(),
                cell.cursor_visible(),
            );
            let painted = paint.at(cell.position());

            assert_eq!(
                painted.border,
                visuals.border,
                "the border at {:?}",
                cell.position()
            );
            assert_eq!(
                painted.foreground,
                visuals.foreground,
                "the foreground at {:?}",
                cell.position()
            );
            borders.insert(painted.border.to_array());
            foregrounds.insert(painted.foreground.to_array());
        }

        // A Grid painted one colour throughout would pass the loop above with
        // every answer off by any number of Cells.
        assert!(borders.len() > 1, "every border was the same colour");
        assert!(
            foregrounds.len() > 1,
            "every foreground was the same colour"
        );
    }

    ///
    /// The Cell carrying no background is exactly the Cell `cell_visuals`
    /// fills with the Source's own colour.
    ///
    /// The comparison calls `cell_visuals` rather than restating a colour,
    /// because this is the one assertion tying the skip to the function it
    /// must not drift from. The condition the skip works out to is
    /// `cursor_visible || (!selected && bloom.is_none())`, which reads wrong
    /// and is right: `cell_visuals` tests `cursor_visible` *before* its bloom
    /// arm, so the Cursor's own Cell takes the Source fill even though
    /// `cursor_bloom` answers `Some(Core)` for it, and the blink alternates a
    /// background and none. Restating any of that here would let the derive
    /// and `cell_visuals` drift apart while both still passed.
    ///
    /// The Grid is wide enough to carry the Cursor's whole bloom and Cells
    /// beyond it, so the skipped Cells and the filled ones are both present;
    /// a Grid where every Cell wanted the same thing would pass while telling
    /// nothing apart.
    ///
    /// The visible half of the blink is the one Render Frame this cannot
    /// reach: a running Orcvs starts with the Cursor off and turns it on by
    /// elapsed time alone. That arm of the condition is asserted over
    /// `cell_visuals` itself, which needs no Render Frame at all.
    ///
    #[test]
    fn the_cell_needing_no_background_is_exactly_the_one_filled_with_the_source() {
        let mut orcvs = Orcvs::new(24, 16);
        orcvs.select(orcvs.render_frame().rows()[7][9].position());

        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let mut skipped = 0;
        let mut filled = 0;

        for cell in frame.rows().iter().flatten() {
            let visuals = cell_visuals(
                cell.glyph(),
                cell.cursor_bloom(),
                cell.selected(),
                cell.cursor_visible(),
            );
            let background = paint.at(cell.position()).background;

            assert_eq!(
                background.is_none(),
                visuals.background == PALETTE.source,
                "the background at {:?}, which cell_visuals fills with {:?}",
                cell.position(),
                visuals.background
            );
            match background {
                Some(colour) => {
                    assert_eq!(colour, visuals.background);
                    filled += 1;
                }
                None => skipped += 1,
            }
        }

        assert!(skipped > 0, "no Cell was left to the Source fill");
        assert!(filled > 0, "no Cell asked for a background of its own");
    }

    ///
    /// The Cursor is one Position the Paint answers, not a flag to be found by
    /// searching the Cells.
    ///
    #[test]
    fn the_cursor_is_the_selected_position() {
        let mut orcvs = Orcvs::new(9, 5);
        let selected = orcvs.render_frame().rows()[3][6].position();
        orcvs.select(selected);

        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);

        assert_eq!(paint.cursor(), selected);
        assert_eq!(
            frame
                .rows()
                .iter()
                .flatten()
                .filter(|cell| cell.selected())
                .map(|cell| cell.position())
                .collect::<Vec<_>>(),
            vec![selected],
            "the Render Frame selects exactly the Cell the Paint calls the Cursor"
        );
    }

    ///
    /// Sector seams stand where the Render Frame asks for them, and nowhere on
    /// the Cursor's own Cell.
    ///
    /// The suppression is the derive's, so no later step learns the rule. The
    /// Cursor is put on a Cell the Render Frame does ask a seam of — the
    /// corner of a sector, where the seam is at full strength — because a
    /// Cursor parked where no seam was wanted would prove nothing.
    ///
    /// Everywhere else the colour is `sector_line`'s answer for the strength
    /// the Render Frame stated. `sector_line` is pure, so the derive resolves
    /// it here; the stroke widths are geometry and are not in this layer.
    ///
    #[test]
    fn seams_stand_where_the_render_frame_asks_and_never_on_the_cursor() {
        let mut orcvs = Orcvs::new(24, 24);
        // A sector corner at the default marker spacing of eight.
        let corner = orcvs.render_frame().rows()[8][8].position();
        orcvs.select(corner);

        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let mut seams = 0;

        for cell in frame.rows().iter().flatten() {
            let painted = paint.at(cell.position());

            if cell.selected() {
                assert!(
                    cell.sector_left_strength().is_some() && cell.sector_top_strength().is_some(),
                    "the Cursor must sit where seams are wanted for their absence to mean anything"
                );
                assert_eq!(painted.sector_left, None);
                assert_eq!(painted.sector_top, None);
                continue;
            }

            assert_eq!(
                painted.sector_left,
                cell.sector_left_strength().map(sector_line),
                "the left seam at {:?}",
                cell.position()
            );
            assert_eq!(
                painted.sector_top,
                cell.sector_top_strength().map(sector_line),
                "the top seam at {:?}",
                cell.position()
            );
            seams += usize::from(painted.sector_left.is_some());
            seams += usize::from(painted.sector_top.is_some());
        }

        assert!(seams > 0, "no Cell but the Cursor's was asked for a seam");
    }

    ///
    /// Each Cell shows what the character table answers for it: its own
    /// content, or the character its Glyph spells when it holds none.
    ///
    /// The Grid carries an Addition, whose claim reaches past the two Cells it
    /// is spelled in and leaves classified but empty operand Cells behind it,
    /// so the blank spellings the table answers are not all the space.
    ///
    #[test]
    fn each_cell_shows_the_character_the_table_answers() {
        let mut orcvs = Orcvs::new(8, 8);
        for (x, character) in ".+".chars().enumerate() {
            orcvs.select(orcvs.render_frame().rows()[2][x].position());
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.render_frame().rows()[5][5].position());

        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let characters = CellCharacters::new();
        let mut spellings = std::collections::BTreeSet::new();

        for cell in frame.rows().iter().flatten() {
            let shown = paint.at(cell.position()).character;

            assert_eq!(
                shown,
                characters.character(cell),
                "the character at {:?}",
                cell.position()
            );
            spellings.insert(shown);
        }

        assert!(
            spellings.len() > 2,
            "every Cell spelled the same thing, so the table answered nothing: {spellings:?}"
        );
    }

    ///
    /// A Paint over `rows` of backgrounds and nothing else that matters.
    ///
    /// `background_runs` folds over the per-Cell backgrounds alone, so the
    /// rest of each Cell is filler: stating it from a Render Frame would make
    /// every case below an exercise in arranging a Cursor bloom to land where
    /// the case needs it.
    ///
    fn paint_of(rows: &[&[Option<Color32>]]) -> Paint {
        let grid = Grid::new(rows[0].len(), rows.len());
        assert!(
            rows.iter().all(|row| row.len() == rows[0].len()),
            "a Grid's rows are all one length"
        );

        Paint {
            grid,
            cursor: grid.origin(),
            cells: rows
                .iter()
                .flat_map(|row| row.iter())
                .map(|&background| CellPaint {
                    background,
                    border: PALETTE.grid_line,
                    foreground: PALETTE.ordinary,
                    sector_left: None,
                    sector_top: None,
                    character: ' ',
                })
                .collect(),
        }
    }

    fn run(colour: Color32, row: usize, columns: Range<usize>) -> BackgroundRun {
        BackgroundRun {
            colour,
            row,
            columns,
        }
    }

    #[test]
    fn a_run_ends_where_the_next_cell_wants_a_different_colour() {
        let first = PALETTE.bloom_core_fill;
        let second = PALETTE.bloom_mid_fill;
        let paint = paint_of(&[&[Some(first), Some(first), Some(second), Some(second)]]);

        assert_eq!(
            paint.background_runs(),
            vec![run(first, 0, 0..2), run(second, 0, 2..4)]
        );
    }

    #[test]
    fn a_run_ends_where_the_next_cell_wants_no_background() {
        let colour = PALETTE.bloom_core_fill;
        let paint = paint_of(&[&[Some(colour), Some(colour), None, Some(colour)]]);

        assert_eq!(
            paint.background_runs(),
            vec![run(colour, 0, 0..2), run(colour, 0, 3..4)],
            "the gap parts two runs of one colour rather than joining them"
        );
    }

    ///
    /// A run ends at the end of its row, and the identical run in the row
    /// below is a run of its own.
    ///
    #[test]
    fn a_run_ends_at_the_end_of_its_row() {
        let colour = PALETTE.bloom_core_fill;
        let paint = paint_of(&[&[Some(colour), Some(colour)], &[Some(colour), Some(colour)]]);

        assert_eq!(
            paint.background_runs(),
            vec![run(colour, 0, 0..2), run(colour, 1, 0..2)]
        );
    }

    ///
    /// One Cell wanting a background between two that want none is a run, not
    /// a Cell the fold forgets to flush.
    ///
    #[test]
    fn one_cell_wanting_a_background_alone_is_a_run_of_one() {
        let colour = PALETTE.selection_fill;
        let paint = paint_of(&[&[None, Some(colour), None]]);

        assert_eq!(paint.background_runs(), vec![run(colour, 0, 1..2)]);
    }

    #[test]
    fn a_row_wanting_no_background_contributes_no_run() {
        let paint = paint_of(&[&[None, None, None]]);

        assert_eq!(paint.background_runs(), Vec::new());
    }
}
