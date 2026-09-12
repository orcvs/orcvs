//!
//! The per-Cell decision of how one Render Frame is drawn: background, border,
//! foreground, sector seams and the character shown.
//!
//! # No geometry
//!
//! Nothing here is a `Rect`, and nothing here reads a viewport's geometry.
//! Where a Cell *is* on screen is `GridViewport`'s arithmetic and is asserted
//! there; a `Rect` in this layer would make every assertion about a colour
//! acquire a viewport before it could be written. The seam strengths a Render
//! Frame states become colours here, because `sector_line` is a pure reading of
//! the palette, while the stroke *widths* the same seams are drawn with scale
//! with the Cell side and so belong to the step that has one.
//!
//! A [`VisiblePositions`] does cross into this layer, and it is that rule kept
//! rather than an exception to it. It is two `Range<usize>` over the Grid's own
//! column and row numbers, and the clip `Rect` they were resolved from is
//! consumed in `grid_viewport` and never arrives here. Naming an index range
//! costs an assertion no viewport, which is the whole of what the rule is for.
//! [`BackgroundRun`] already answered columns rather than a rectangle for
//! exactly this reason.
//!
//! # Only the Positions the console draws
//!
//! A Paint covers the Positions `GridViewport::visible_positions` answers and
//! no others, so a console showing a tenth of a Grid pays for a tenth of it:
//! neither the per-Cell call to `cell_visuals` nor the `Vec` holding its
//! answers follows the Source's size. Everything the derivation reads is local
//! to its own Cell — a Cursor outside the drawn range still blooms correctly on
//! the Cells inside it — which is what makes the cull a saving rather than a
//! change of answer.
//!
//! # Why the Cells are flat
//!
//! A [`Paint`] holds one `Vec<CellPaint>` in row-major order over the drawn
//! Positions, and [`Paint::at`] indexes it by subtracting the range's own
//! corner. `Grid::index` is deliberately not that arithmetic: it addresses a
//! Cell of the whole Grid, and this `Vec` holds a sub-rectangle of one. This is
//! deliberately unlike `RenderFrame`, which nests a `Vec` per row: a Render
//! Frame's only consumer walks it in row order to paint it, and the nesting
//! exists to serve exactly that. A Paint is asked about one Cell — what colour
//! is the Cell at this Position — so the shape that serves it is the one the
//! Grid already addresses Cells by.
//!

use std::ops::Range;

use egui::Color32;

use orcvs::{
    glyph::{Glyph, GlyphString},
    grid::{Grid, Position},
    render_frame::{RenderCell, RenderFrame},
};

use crate::{
    grid_viewport::VisiblePositions,
    style::{PALETTE, cell_visuals, sector_line},
};

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
    /// The Positions this Paint covers: what the console draws, and one Cell
    /// more in every direction the Grid has one.
    drawn: VisiblePositions,
    cursor: Option<Position>,
    cells: Vec<CellPaint>,
}

impl Paint {
    ///
    /// Reads a Render Frame and answers what each of the Cells at `drawn` is
    /// drawn as.
    ///
    /// `cell_visuals` is called once per drawn Cell and is unchanged: this
    /// decides what to do with its answer, not what the answer is.
    ///
    /// `drawn` is the console's decision, not this layer's. It comes from
    /// `GridViewport::visible_positions` already clamped to the Grid, which is
    /// why the two slices below are indexed rather than reached through `get`:
    /// a range that outran this Render Frame would mean a Frame that is not
    /// the rectangle its own Grid says it is, and answering an empty slice
    /// there would turn that into rows that silently go unpainted.
    ///
    pub(crate) fn derive(frame: &RenderFrame, drawn: &VisiblePositions) -> Self {
        let grid = frame.grid();
        // The Cursor is the Position the Render Frame was derived for. A Paint
        // covers a viewport, so `None` here means that Position is outside the
        // drawn range — not that the Frame selected nothing.
        let cursor = frame.cursor();
        let cursor = (drawn.columns.contains(&cursor.x()) && drawn.rows.contains(&cursor.y()))
            .then_some(cursor);
        // What each Cell says, read once for the nine blank spellings and never
        // per Cell. It needs no `egui::Context`: what a Cell says is a reading
        // of `GlyphString`, and only drawing it reaches the font atlas.
        let characters = CellCharacters::new();
        let columns = drawn.columns.clone();
        // Sized up front. A `FlatMap` states no length, so collecting into a
        // `Vec` would grow it by doubling across every row of a Render Frame
        // where the drawn count is already known exactly.
        let mut cells = Vec::with_capacity(drawn.count());
        cells.extend(
            frame.rows()[drawn.rows.clone()]
                .iter()
                .flat_map(|row| row[columns.clone()].iter())
                .map(|cell| {
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
                }),
        );

        Self {
            grid,
            drawn: drawn.clone(),
            cursor,
            cells,
        }
    }

    ///
    /// What the Cell at `position` is drawn as.
    ///
    /// `position` has to be one of the drawn Positions. One this Paint does
    /// not cover is refused rather than indexed: the offset is a subtraction
    /// from the drawn range's own corner, so a Position left of or above that
    /// corner wraps, and a wrapped offset can land back inside `cells` and
    /// answer some other Cell's paint in silence.
    ///
    /// Production walks go through [`Self::cells`] or [`Self::background_runs`]
    /// so they never pay this offset; callers that already hold a Position —
    /// the colour tests among them — look up here.
    ///
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Position-indexed lookup; walks use cells()/background_runs()"
        )
    )]
    pub(crate) fn at(&self, position: Position) -> &CellPaint {
        let offset = self
            .offset(position)
            .unwrap_or_else(|| panic!("{position:?} is not one of the drawn Positions"));

        &self.cells[offset]
    }

    ///
    /// Where `position`'s paint sits in `cells`, or `None` when this Paint
    /// does not cover it.
    ///
    /// Row-major within the drawn range rather than `Grid::index`, which
    /// addresses a Cell of the whole Grid where this `Vec` holds one
    /// sub-rectangle of it.
    ///
    fn offset(&self, position: Position) -> Option<usize> {
        let column = position.x().checked_sub(self.drawn.columns.start)?;
        let row = position.y().checked_sub(self.drawn.rows.start)?;

        (column < self.drawn.columns.len() && row < self.drawn.rows.len())
            .then(|| row * self.drawn.columns.len() + column)
    }

    ///
    /// The drawn Positions in row order, each with what it is drawn as.
    ///
    /// The one way to walk a Paint. It zips the Positions against the `Vec`
    /// rather than asking [`Self::at`] for each of them, so the walk the shape
    /// step makes on every Render Frame does no offset arithmetic at all, and
    /// the row-major agreement between Positions and Cells is kept here rather
    /// than restated at each caller.
    ///
    pub(crate) fn cells(&self) -> impl Iterator<Item = (Position, &CellPaint)> {
        self.positions().zip(&self.cells)
    }

    ///
    /// How many Cells this Paint covers, which is the drawn count and never
    /// the Grid's.
    ///
    pub(crate) fn count(&self) -> usize {
        self.cells.len()
    }

    ///
    /// The drawn Positions in row order.
    ///
    fn positions(&self) -> impl Iterator<Item = Position> {
        self.drawn.rows.clone().flat_map(|row| self.row(row))
    }

    ///
    /// The drawn Positions of one row, left to right.
    ///
    /// `Grid::position` is the only way to mint one and it refuses a pair the
    /// Grid does not own, which `visible_positions` has already clamped
    /// against — so the `expect` restates that clamp rather than guarding a
    /// case that arises.
    ///
    fn row(&self, row: usize) -> impl Iterator<Item = Position> {
        let grid = self.grid;

        self.drawn.columns.clone().map(move |column| {
            grid.position(column, row)
                .expect("a drawn Position is one the visible range clamped to this Grid")
        })
    }

    ///
    /// Where the Cursor is, when this Paint covers it.
    ///
    /// One Position for the whole Paint rather than a flag on every Cell:
    /// [`RenderFrame::cursor`] answers the Position `derive` was given, so
    /// exactly one exists, and a per-Cell bool would re-open a state the layer
    /// below has closed. `None` says the Cursor is outside the viewport this
    /// Paint covers, not that the Render Frame selected nothing.
    ///
    pub(crate) fn cursor(&self) -> Option<Position> {
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
    /// "Its row" is the drawn part of a row, so a run opens at the first drawn
    /// Cell and is flushed at the last. Coalescing the whole row and clipping
    /// the answer would paint the same rectangles — `cell_rect` is a function
    /// of one column index, so both hand the shape step the same two endpoints
    /// — but it would walk Cells the console does not draw, which is the cost
    /// this covers only the drawn range to avoid.
    ///
    pub(crate) fn background_runs(&self) -> Vec<BackgroundRun> {
        let mut runs = Vec::new();
        let width = self.drawn.columns.len();
        let first_column = self.drawn.columns.start;

        // Row-major slices of `cells`, not `at(Position)`: the fold already
        // owns the drawn ranges, and re-deriving each Cell's offset through the
        // Position mint would pay the panic path on every Cell of every frame.
        for (row_offset, row) in self.drawn.rows.clone().enumerate() {
            let start = row_offset * width;
            let row_cells = &self.cells[start..start + width];
            // The run so far: its colour, and the columns it covers.
            let mut open: Option<(Color32, Range<usize>)> = None;

            for (column_offset, cell) in row_cells.iter().enumerate() {
                let column = first_column + column_offset;
                let background = cell.background;
                open = match (open, background) {
                    (Some((colour, columns)), Some(background)) if colour == background => {
                        Some((colour, columns.start..column + 1))
                    }
                    (finished, background) => {
                        if let Some((colour, columns)) = finished {
                            runs.push(BackgroundRun {
                                colour,
                                row,
                                columns,
                            });
                        }
                        background.map(|colour| (colour, column..column + 1))
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

///
/// Every [`Glyph`] a Render Frame can carry, in the order
/// [`blank_glyph_index`] gives them.
///
const BLANK_GLYPHS: [Glyph; 9] = [
    Glyph::Bang,
    Glyph::Char,
    Glyph::Comment,
    Glyph::Function,
    Glyph::Highlight,
    Glyph::Marker,
    Glyph::Note,
    Glyph::Number,
    Glyph::Space,
];

///
/// Where `glyph` sits in [`BLANK_GLYPHS`].
///
/// The match is exhaustive, so a `Glyph` added to the vocabulary fails to build
/// here rather than quietly painting the wrong character.
///
fn blank_glyph_index(glyph: Glyph) -> usize {
    match glyph {
        Glyph::Bang => 0,
        Glyph::Char => 1,
        Glyph::Comment => 2,
        Glyph::Function => 3,
        Glyph::Highlight => 4,
        Glyph::Marker => 5,
        Glyph::Note => 6,
        Glyph::Number => 7,
        Glyph::Space => 8,
    }
}

///
/// What an empty Cell of `glyph` shows.
///
/// `GlyphString` is where an empty Cell's spelling is decided, so the console
/// reads it rather than restating it — once per Render Frame for the nine
/// Glyphs, never once per Cell.
///
fn blank_character(glyph: Glyph) -> char {
    let spelling = GlyphString::new(None, glyph).to_string();
    debug_assert_eq!(
        spelling.chars().count(),
        1,
        "an empty Cell shows exactly one character"
    );

    spelling.chars().next().unwrap_or(' ')
}

///
/// What each Cell of a Render Frame shows: its own content, or the character
/// its Glyph spells when it holds none.
///
/// This is the whole of deciding what a Cell says, and it is in this layer
/// because it needs nothing this layer does not have: a `GlyphString` and a
/// `Glyph`, and no `egui::Context` at all. Only *drawing* that character needs
/// one — `GlyphTable` in `console.rs` holds the galleys and nothing else — so
/// the split is the same one the rest of this module makes, between deciding
/// what a Cell looks like and painting it.
///
/// The nine blank spellings are read once, because reading one is a
/// `GlyphString` and a `String` per call and a Grid has a thousand Cells; the
/// table is an array of nine `char`s indexed by [`blank_glyph_index`], so
/// building it is far cheaper than the per-Cell reads it saves.
///
struct CellCharacters {
    /// The character an empty Cell shows, indexed by [`blank_glyph_index`].
    blanks: [char; BLANK_GLYPHS.len()],
}

impl CellCharacters {
    /// Reads what an empty Cell of each [`Glyph`] spells.
    fn new() -> Self {
        Self {
            blanks: BLANK_GLYPHS.map(blank_character),
        }
    }

    /// The character `cell` shows.
    fn character(&self, cell: &RenderCell) -> char {
        cell.content()
            .unwrap_or_else(|| self.blanks[blank_glyph_index(cell.glyph())])
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BLANK_GLYPHS, BackgroundRun, CellCharacters, CellPaint, Glyph, Paint, blank_glyph_index,
    };
    use crate::grid_viewport::VisiblePositions;
    use crate::style::{PALETTE, cell_visuals, sector_line};
    use egui::Color32;
    use orcvs::{app::Orcvs, grid::Grid, render_frame::RenderFrame};
    use std::ops::Range;

    fn running_orcvs(cols: usize, rows: usize) -> Orcvs {
        Orcvs::new(cols, rows).expect("the test runtime")
    }

    ///
    /// A Paint over every Position of a Render Frame's Grid.
    ///
    /// This is the range a fitted console hands `derive` —
    /// `GridViewport::visible_positions` answers exactly it when the whole
    /// Grid is on screen, which `console.rs` asserts through a console pass.
    /// Stated directly here so that a test about what colour a Cell is still
    /// needs no viewport to say "all of them".
    ///
    fn whole(frame: &RenderFrame) -> Paint {
        let grid = frame.grid();

        Paint::derive(
            frame,
            &VisiblePositions {
                columns: 0..grid.columns(),
                rows: 0..grid.rows(),
            },
        )
    }

    ///
    /// Every Position answers the Cell the Grid indexes, not its neighbour.
    ///
    /// The Cursor's bloom grades the border across the Grid and the written
    /// Cells carry Glyph colours of their own, so an answer off by one Cell —
    /// in either axis — differs from the answer asked for.
    ///
    #[tokio::test]
    async fn a_cell_is_answered_at_the_position_the_grid_indexes() {
        let mut orcvs = running_orcvs(6, 4);
        for (x, character) in "#a#".chars().enumerate() {
            orcvs.select(orcvs.render_frame().rows()[2][x + 1].position());
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.render_frame().rows()[1][4].position());

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
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
    /// fills with the Source's own colour, over a Paint derived from a real
    /// Render Frame.
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
    /// elapsed time alone, with nothing public to set it. That half of the
    /// split is `console.rs`'s
    /// `the_skip_condition_matches_cell_visuals_in_both_blink_phases`, a truth
    /// table over `cell_visuals` that needs no Render Frame at all.
    ///
    #[tokio::test]
    async fn the_cell_needing_no_background_is_exactly_the_one_filled_with_the_source() {
        let mut orcvs = running_orcvs(24, 16);
        orcvs.select(orcvs.render_frame().rows()[7][9].position());

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
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
    #[tokio::test]
    async fn the_cursor_is_the_selected_position() {
        let mut orcvs = running_orcvs(9, 5);
        let selected = orcvs.render_frame().rows()[3][6].position();
        orcvs.select(selected);

        let paint = whole(&orcvs.render_frame());

        assert_eq!(paint.cursor(), Some(selected));
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
    #[tokio::test]
    async fn seams_stand_where_the_render_frame_asks_and_never_on_the_cursor() {
        let mut orcvs = running_orcvs(24, 24);
        // A sector corner at the default marker spacing of eight.
        let corner = orcvs.render_frame().rows()[8][8].position();
        orcvs.select(corner);

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
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
    #[tokio::test]
    async fn each_cell_shows_the_character_the_table_answers() {
        let mut orcvs = running_orcvs(8, 8);
        for (x, character) in ".+".chars().enumerate() {
            orcvs.select(orcvs.render_frame().rows()[2][x].position());
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.render_frame().rows()[5][5].position());

        let frame = orcvs.render_frame();
        let paint = whole(&frame);
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
    /// What an empty Cell shows is `GlyphString`'s answer, read once per Render
    /// Frame rather than restated in the console.
    ///
    #[test]
    fn a_blank_cell_shows_what_its_glyph_spells() {
        for glyph in BLANK_GLYPHS {
            assert_eq!(
                BLANK_GLYPHS[blank_glyph_index(glyph)],
                glyph,
                "the blank table is not indexed by its own order"
            );
            assert_eq!(
                super::blank_character(glyph).to_string(),
                orcvs::glyph::GlyphString::new(None, glyph).to_string()
            );
        }
        assert_eq!(super::blank_character(Glyph::Marker), '+');
        assert_eq!(super::blank_character(Glyph::Highlight), '.');
        assert_eq!(super::blank_character(Glyph::Space), ' ');
    }

    ///
    /// Which character a Cell shows is answered from the Render Frame alone.
    ///
    /// No `egui::Context` is built here, and that is the assertion: the lookup
    /// is a reading of `GlyphString`, not a reading of the font atlas, so the
    /// step that decides what a Cell says is reachable without the harness the
    /// galleys need. Every Cell of the Grid is checked against `GlyphString`'s
    /// own answer, the written Cells for their content and the rest for the
    /// spelling their Glyph gives an empty Cell.
    ///
    #[tokio::test]
    async fn a_cell_answers_its_character_with_no_context() {
        let mut orcvs = running_orcvs(8, 8);
        // An Addition, whose claim reaches past the two Cells it is spelled in
        // and leaves the operand Cells behind it empty but classified. Those
        // are the Cells that make the blank table answer something other than
        // the space.
        let written = ".+";
        for (x, character) in written.chars().enumerate() {
            let position = orcvs.render_frame().rows()[2][x].position();
            orcvs.select(position);
            orcvs.write(&character.to_string());
        }

        let characters = CellCharacters::new();
        let frame = orcvs.render_frame();
        let mut content = String::new();
        let mut blanks = std::collections::BTreeSet::new();

        for cell in frame.rows().iter().flatten() {
            let spelled = orcvs::glyph::GlyphString::new(
                cell.content().map(|content| content.to_string()),
                cell.glyph(),
            )
            .to_string();

            assert_eq!(
                characters.character(cell).to_string(),
                spelled,
                "the Cell at {:?} shows something its GlyphString does not spell",
                cell.position()
            );

            match cell.content() {
                Some(character) => content.push(character),
                None => {
                    blanks.insert(characters.character(cell));
                }
            }
        }

        assert_eq!(content, written, "the written Cells kept their content");
        // A Grid whose blank Cells all spell the space would pass the loop
        // above while telling nothing apart, so the Addition's unfilled operand
        // slots have to be in it: `h` is what an empty Cell a signature says a
        // Number belongs in shows.
        assert!(
            blanks.contains(&'h'),
            "no unfilled operand slot reached the blank table, so it went untested: {blanks:?}"
        );
        assert!(
            blanks.contains(&' '),
            "no empty Cell reached the blank table, so it went untested: {blanks:?}"
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
            drawn: VisiblePositions {
                columns: 0..grid.columns(),
                rows: 0..grid.rows(),
            },
            cursor: Some(grid.origin()),
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

    ///
    /// Consecutive Cells in a row that want the same background are one run,
    /// over a real Render Frame rather than a fixture.
    ///
    /// The runs this Grid asks for are written out rather than folded. Deriving
    /// the expectation with the same match the fold runs would check that the
    /// answer agrees with the rule without ever checking the rule: invert the
    /// guard in both places and the test still passes. These spans are read off
    /// the 8 by 8 default Grid instead, so the fold's arm structure is pinned
    /// by something outside the code under test.
    ///
    /// The Cursor rests at 0,0 and its bloom grades outwards through four
    /// bands, which is why the rows nearest it break into short runs while the
    /// far rows run whole. Row 7 carries the bloom's hashed outer edge, so its
    /// Cells alternate instead of joining up — that ragged boundary is the
    /// Render Frame's, and a run that swallowed it would be caught here.
    ///
    /// Columns and no geometry: what rectangle a run becomes is the viewport's
    /// arithmetic and is asserted in `console.rs`, where a viewport exists.
    ///
    #[tokio::test]
    async fn consecutive_cells_sharing_a_background_are_one_rectangle() {
        let orcvs = running_orcvs(8, 8);
        let frame = orcvs.render_frame();
        let paint = whole(&frame);

        let expected = vec![
            run(PALETTE.selection_fill, 0, 0..1),
            run(PALETTE.bloom_core_fill, 0, 1..2),
            run(PALETTE.bloom_inner_fill, 0, 2..3),
            run(PALETTE.bloom_mid_fill, 0, 3..4),
            run(PALETTE.bloom_outer_fill, 0, 4..7),
            run(PALETTE.bloom_inner_fill, 1, 0..1),
            run(PALETTE.bloom_core_fill, 1, 1..2),
            run(PALETTE.bloom_mid_fill, 1, 2..5),
            run(PALETTE.bloom_outer_fill, 1, 5..7),
            run(PALETTE.bloom_mid_fill, 2, 0..1),
            run(PALETTE.bloom_inner_fill, 2, 1..2),
            run(PALETTE.bloom_mid_fill, 2, 2..4),
            run(PALETTE.bloom_outer_fill, 2, 4..7),
            run(PALETTE.bloom_mid_fill, 3, 0..5),
            run(PALETTE.bloom_outer_fill, 3, 5..7),
            run(PALETTE.bloom_mid_fill, 4, 0..1),
            run(PALETTE.bloom_outer_fill, 4, 1..2),
            run(PALETTE.bloom_mid_fill, 4, 2..4),
            run(PALETTE.bloom_outer_fill, 4, 4..7),
            run(PALETTE.bloom_outer_fill, 5, 0..7),
            run(PALETTE.bloom_outer_fill, 6, 0..7),
            run(PALETTE.bloom_outer_fill, 7, 1..2),
            run(PALETTE.bloom_outer_fill, 7, 3..4),
            run(PALETTE.bloom_outer_fill, 7, 5..6),
            run(PALETTE.bloom_outer_fill, 7, 7..8),
        ];
        // The Cells those runs replace, counted off the same table.
        let filled: usize = expected.iter().map(|run| run.columns.len()).sum();

        assert!(
            expected.iter().any(|run| run.columns.len() > 1),
            "no run covered more than one Cell, so nothing was coalesced"
        );
        assert!(
            expected.len() < filled,
            "{} runs for {filled} filled Cells is no saving",
            expected.len()
        );
        assert_eq!(paint.background_runs(), expected);
    }

    #[test]
    fn a_row_wanting_no_background_contributes_no_run() {
        let paint = paint_of(&[&[None, None, None]]);

        assert_eq!(paint.background_runs(), Vec::new());
    }

    ///
    /// A Paint decides the Cells the console draws and no others, and decides
    /// each of them exactly as a Paint of the whole Grid would.
    ///
    /// This is the Cell-iteration half of the claim `source-grid-rendering/05`
    /// made — the cost of a Render Frame follows the viewport rather than the
    /// Source. `05` could only count it through a console pass, because the
    /// per-Cell decision had no surface of its own; here the `CellPaint`s are
    /// the count, with no `egui::Context` and no window.
    ///
    /// The second assertion is the half that matters for correctness. Nothing
    /// the derivation reads reaches past its own Cell, so a culled Paint has
    /// to answer what an unculled one answers. The Cursor is placed outside
    /// the drawn range and near enough for its bloom to reach inside it, which
    /// is the case that would catch a derivation whose answer depended on
    /// where the walk started.
    ///
    #[tokio::test]
    async fn a_paint_decides_the_drawn_cells_and_answers_them_unchanged() {
        let mut orcvs = running_orcvs(40, 30);
        orcvs.select(orcvs.render_frame().rows()[10][9].position());

        let frame = orcvs.render_frame();
        let drawn = VisiblePositions {
            columns: 12..20,
            rows: 8..14,
        };
        let culled = Paint::derive(&frame, &drawn);
        let every = whole(&frame);

        assert_eq!(culled.count(), drawn.count());
        assert!(
            culled.count() * 10 < every.count(),
            "{} of {} Cells is not a cull worth asserting about",
            culled.count(),
            every.count()
        );

        let walked = culled.cells().count();
        assert_eq!(walked, culled.count(), "the walk and the count disagree");

        let mut graded = 0;
        for (position, cell) in culled.cells() {
            assert!(
                drawn.columns.contains(&position.x()) && drawn.rows.contains(&position.y()),
                "{position:?} is outside the range the Paint was asked for"
            );
            assert_eq!(cell, every.at(position), "the paint at {position:?}");
            graded += usize::from(cell.background.is_some());
        }

        assert!(
            graded > 0,
            "the Cursor's bloom reached no drawn Cell, so nothing here would notice a derivation that read its neighbours"
        );
    }

    ///
    /// The runs a culled Paint answers are the whole Grid's runs clipped to
    /// the range it covers.
    ///
    /// Coalescing over the drawn columns and coalescing the whole row and
    /// clipping the answer produce the same rectangles: the shape step builds
    /// a run's rectangle from `GridViewport::cell_rect` on the run's first and
    /// last column, and `cell_rect` is a function of one column index, so both
    /// folds hand it the same two endpoints. The choice between them is cost
    /// alone.
    ///
    /// That is an equality rather than a resemblance, so it is asserted as one
    /// instead of left to a comment — which is what `3c2b640` asked for when
    /// it pinned the one-Cell margin's purpose with assertions.
    ///
    #[tokio::test]
    async fn the_runs_a_culled_paint_answers_are_the_whole_grids_clipped_to_it() {
        let mut orcvs = running_orcvs(32, 24);
        orcvs.select(orcvs.render_frame().rows()[11][15].position());

        let frame = orcvs.render_frame();
        let drawn = VisiblePositions {
            columns: 10..22,
            rows: 7..16,
        };
        let clipped = whole(&frame)
            .background_runs()
            .into_iter()
            .filter(|run| drawn.rows.contains(&run.row))
            .filter_map(|run| {
                let columns = run.columns.start.max(drawn.columns.start)
                    ..run.columns.end.min(drawn.columns.end);

                (!columns.is_empty()).then_some(BackgroundRun { columns, ..run })
            })
            .collect::<Vec<_>>();

        assert!(
            clipped.iter().any(|run| run.columns.len() > 1),
            "nothing was coalesced, so the two folds agree about nothing"
        );
        assert!(
            clipped.iter().any(|run| {
                run.columns.start == drawn.columns.start || run.columns.end == drawn.columns.end
            }),
            "no run reached an edge of the drawn range, which is the only place the two folds could part"
        );
        assert_eq!(Paint::derive(&frame, &drawn).background_runs(), clipped);
    }

    ///
    /// A Paint answers the Cursor's Position where it covers it, and `None`
    /// where it does not.
    ///
    /// `None` is a statement about the viewport rather than about the Render
    /// Frame: the same Frame answers a Cursor to a Paint whose range reaches
    /// it. A derivation that searched its own Cells and insisted on finding
    /// one would panic on every zoomed console whose Cursor is off screen.
    ///
    #[tokio::test]
    async fn the_cursor_is_answered_only_where_the_paint_covers_it() {
        let mut orcvs = running_orcvs(20, 20);
        let selected = orcvs.render_frame().rows()[4][5].position();
        orcvs.select(selected);

        let frame = orcvs.render_frame();
        let reaching = VisiblePositions {
            columns: 3..8,
            rows: 2..7,
        };
        let past = VisiblePositions {
            columns: 10..16,
            rows: 12..18,
        };

        assert_eq!(whole(&frame).cursor(), Some(selected));
        assert_eq!(Paint::derive(&frame, &reaching).cursor(), Some(selected));
        assert_eq!(Paint::derive(&frame, &past).cursor(), None);
    }

    ///
    /// A console showing none of its Grid paints nothing, and that is a state
    /// rather than an error.
    ///
    /// `GridViewport::visible_positions` answers `VisiblePositions::empty()`
    /// for a clip that misses the Grid, for a Grid with no Cell and for a
    /// degenerate viewport, so this is a range the shipped path reaches rather
    /// than one only a test can build.
    ///
    #[tokio::test]
    async fn a_paint_over_no_positions_is_empty_rather_than_impossible() {
        let orcvs = running_orcvs(8, 8);
        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame, &VisiblePositions::empty());

        assert_eq!(paint.count(), 0);
        assert_eq!(paint.cells().count(), 0);
        assert_eq!(paint.cursor(), None);
        assert_eq!(paint.background_runs(), Vec::new());
    }
}
