//! Drawing the Source Grid: the Shapes a Render Frame's Cells are painted as,
//! in paint order, and the one click the Grid answers.

use std::ops::Range;

use egui::{
    Color32, CornerRadius, FontId, Mesh, Pos2, Rect, Sense, Shape, Stroke, emath::GuiRounding as _,
    epaint::RectShape,
};
use orcvs::render_frame::RenderFrame;

use super::glyphs::GlyphTable;
use super::source_view::PointerSelection;
use crate::grid_viewport::{GridViewport, VisiblePositions};
use crate::paint::{EdgeLine, EdgeRank, Edges, FramePaint, Paint};
use crate::theme::Theme;

///
/// One run of consecutive Cells that share a background colour, as the single
/// rectangle that fills them all.
///
/// # Why the rectangle is snapped
///
/// Not to prevent a seam. epaint snaps the rectangle whether or not this does:
/// `RectShape::filled` leaves `round_to_pixels: None`,
/// `TessellationOptions::round_rects_to_pixels` defaults to true, and
/// `Tessellator::tessellate_rect` then applies `Rect::round_to_pixels` — the
/// same `emath` function this calls — for every `StrokeKind` but `Middle`. A
/// run left unsnapped here would reach the screen as the same pixels.
///
/// It is snapped so the rectangle is one a test can predict. The snap is the
/// last thing that moves an edge, so doing it here puts the Shape a Render
/// Frame carries at the coordinates the paint lands on, and an assertion can
/// state them exactly rather than within a pixel. `Rect::round_to_pixels`
/// rounds the two corners independently, so a run's far edge lands on
/// the physical pixel the next Cell's near edge would have and neighbouring
/// runs still tile — which is what makes that predicted rectangle the same
/// paint as the Cells it replaces.
///
fn background_run(covered: Rect, fill: Color32, pixels_per_point: f32) -> Shape {
    Shape::Rect(RectShape::filled(
        covered.round_to_pixels(pixels_per_point),
        CornerRadius::ZERO,
        fill,
    ))
}

///
/// A line placed on whole physical pixels: how many it covers across its
/// boundary, and the colour it covers them in.
///
/// A line thinner than one physical pixel covers one at its colour scaled by
/// its width, as epaint's own thin-stroke path fades a line rather than
/// thinning it: a fine Theme still shows a Grid at 1×, and nothing blurs
/// across two pixels.
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PixelLine {
    pub(super) width: i32,
    pub(super) colour: Color32,
}

impl PixelLine {
    ///
    /// `stroke` at `pixels_per_point`, or `None` where it covers no pixel.
    ///
    pub(super) fn new(stroke: Stroke, pixels_per_point: f32) -> Option<Self> {
        let physical = stroke.width * pixels_per_point;
        if physical >= 1.0 {
            Some(Self {
                width: physical.round() as i32,
                colour: stroke.color,
            })
        } else if physical > 0.0 {
            Some(Self {
                width: 1,
                colour: stroke.color.gamma_multiply(physical),
            })
        } else {
            None
        }
    }
}

///
/// The pixels a line `width` pixels wide covers across the boundary at pixel
/// `boundary`: a 1 pixel line lies right of or below it, a 2 pixel line
/// straddles it. A narrower line's band lies inside a wider one's on the same
/// boundary, and a width of zero covers nothing.
///
fn band(boundary: i32, width: i32) -> Range<i32> {
    let start = boundary - width / 2;
    start..start + width
}

///
/// An edge's line on whole physical pixels, with the rank and seam that decide
/// which line a junction it meets shows.
///
#[derive(Clone, Copy)]
struct Line {
    rank: EdgeRank,
    seamed: bool,
    width: i32,
    colour: Color32,
}

impl Line {
    fn new(edge: EdgeLine, pixels_per_point: f32) -> Option<Self> {
        let PixelLine { width, colour } = PixelLine::new(edge.stroke, pixels_per_point)?;
        Some(Self {
            rank: edge.rank,
            seamed: edge.seamed,
            width,
            colour,
        })
    }

    ///
    /// Which of two lines meeting a junction gives it its colour: the higher
    /// rank, and of one rank a seamed line, so a seam runs on through a plain
    /// Grid line it crosses, and then the more opaque.
    ///
    fn strength(&self) -> (EdgeRank, bool, u8) {
        (self.rank, self.seamed, self.colour.a())
    }
}

/// The widest of `lines`, or zero where there are none.
fn widest(lines: impl IntoIterator<Item = Line>) -> i32 {
    lines.into_iter().map(|line| line.width).max().unwrap_or(0)
}

///
/// The quad covering physical pixels `x` by `y` in `colour`, two triangles
/// over four vertices, or nothing where it covers no pixel.
///
fn quad(mesh: &mut Mesh, x: Range<i32>, y: Range<i32>, colour: Color32, pixels_per_point: f32) {
    if !x.is_empty() && !y.is_empty() {
        let point =
            |x: i32, y: i32| Pos2::new(x as f32 / pixels_per_point, y as f32 / pixels_per_point);
        mesh.add_colored_rect(
            Rect::from_min_max(point(x.start, y.start), point(x.end, y.end)),
            colour,
        );
    }
}

///
/// `whole` and both ends of each of `arms`, at most four, in order.
///
fn cuts<'a>(whole: &Range<i32>, arms: impl Iterator<Item = &'a Range<i32>>) -> [i32; 10] {
    let mut cuts = [whole.start; 10];
    cuts[1] = whole.end;
    for (arm, cut) in arms.zip(cuts[2..].chunks_mut(2)) {
        cut.copy_from_slice(&[arm.start, arm.end]);
    }
    cuts.sort_unstable();
    cuts
}

/// The pieces between consecutive `cuts`.
fn pieces(cuts: &[i32; 10]) -> impl Iterator<Item = Range<i32>> + '_ {
    cuts.windows(2)
        .filter(|cut| cut[0] < cut[1])
        .map(|cut| cut[0]..cut[1])
}

/// The pixels along a band of a junction's rows that its lines cover, and that
/// its top lines do.
type Covered = (Option<Range<i32>>, Option<Range<i32>>);

///
/// Where a column line crosses a row line: the lines meeting it — above,
/// below, left and right — and the pixels it covers, as wide as the widest
/// line across the column line and as tall as the widest across the row line.
///
struct Junction {
    lines: [Option<Line>; 4],
    x: Range<i32>,
    y: Range<i32>,
}

impl Junction {
    fn new(lines: [Option<Line>; 4], column: i32, row: i32) -> Self {
        let [above, below, left, right] = lines;
        Self {
            lines,
            x: band(column, widest(above.into_iter().chain(below))),
            y: band(row, widest(left.into_iter().chain(right))),
        }
    }

    ///
    /// Draws the junction on column line `column` and row line `row`: one quad
    /// where every line meeting it is of one rank, and otherwise a quad for
    /// each piece of it some line reaches, in the colour of the highest line
    /// reaching it.
    ///
    fn draw(&self, mesh: &mut Mesh, column: i32, row: i32, pixels_per_point: f32) {
        let (x, y) = (&self.x, &self.y);
        let lines = self.lines.iter().flatten();
        let Some(top) = lines.clone().max_by_key(|line| line.strength()) else {
            return;
        };
        let Some(beneath) = lines
            .filter(|line| line.rank != top.rank)
            .max_by_key(|line| line.strength())
        else {
            return quad(mesh, x.clone(), y.clone(), top.colour, pixels_per_point);
        };
        // Where the top lines cross: their widest bands on each axis, and the
        // whole junction along an axis none of them is on.
        let [above, below, left, right] = self.lines;
        let on_top = |line: Option<Line>| line.filter(|line| line.rank == top.rank);
        let own = |whole: &Range<i32>, boundary: i32, one: Option<Line>, other: Option<Line>| {
            match widest(on_top(one).into_iter().chain(on_top(other))) {
                0 => whole.clone(),
                width => band(boundary, width),
            }
        };
        let own_x = own(x, column, above, below);
        let own_y = own(y, row, left, right);
        // Each line reaches from its own side of the junction to the far side
        // of the top lines' crossing, so a lower line never fills a corner the
        // top lines turn away from.
        let arms = [
            above.map(|line| (line, band(column, line.width), y.start..own_y.end)),
            below.map(|line| (line, band(column, line.width), own_y.start..y.end)),
            left.map(|line| (line, x.start..own_x.end, band(row, line.width))),
            right.map(|line| (line, own_x.start..x.end, band(row, line.width))),
        ];
        let arms = arms.iter().flatten();
        // What a band of rows covers. Every arm covers the pixel right of the
        // column line, so each is one run.
        let runs = |rows: &Range<i32>| -> Covered {
            let run = |top_only: bool| {
                arms.clone()
                    .filter(|(_, _, y)| y.start <= rows.start && rows.end <= y.end)
                    .filter(|(line, _, _)| !top_only || line.rank == top.rank)
                    .map(|(_, x, _)| x.clone())
                    .reduce(|run, x| run.start.min(x.start)..run.end.max(x.end))
            };
            (run(false), run(true))
        };
        let mut draw = |rows: Range<i32>, (all, own): Covered| {
            let Some(all) = all else { return };
            let mut fill = |x, colour| quad(mesh, x, rows.clone(), colour, pixels_per_point);
            match own {
                Some(own) => {
                    fill(all.start..own.start, beneath.colour);
                    fill(own.clone(), top.colour);
                    fill(own.end..all.end, beneath.colour);
                }
                None => fill(all, beneath.colour),
            }
        };
        // A band of rows for each change in the arms they cross, and
        // consecutive bands that cover the same runs as one.
        let mut open: Option<(Range<i32>, Covered)> = None;
        for rows in pieces(&cuts(y, arms.clone().map(|(_, _, y)| y))) {
            let covered = runs(&rows);
            match &mut open {
                Some((band, shown)) if *shown == covered => band.end = rows.end,
                _ => {
                    if let Some((band, shown)) = open.replace((rows, covered)) {
                        draw(band, shown);
                    }
                }
            }
        }
        if let Some((band, shown)) = open {
            draw(band, shown);
        }
    }
}

///
/// The edges of `drawn`, as one Mesh of quads that never overlap: a junction
/// where each column line crosses each row line, and a piece for each edge
/// between two junctions.
///
/// `columns` and `rows` are the boundaries of `drawn` in physical pixels, left
/// to right and top to bottom, the far edges of its last column and row
/// included, so each axis is spaced by its own Cell side.
///
/// An edge's piece lies across its boundary at its own width, and runs
/// between the junctions at its ends. A narrower line's band lies inside the
/// junction's. The highest-ranked lines meeting a junction keep their own
/// widths there, and the highest-ranked of the others fills only the pixels
/// its own lines reach, so a narrow border turns its corner at its own width
/// through wider Grid lines and no Grid pixel lands inside the border. Of
/// lines of one rank, a seamed one gives the colour, and then the most opaque.
///
/// Quads rather than rectangles or segments: epaint draws a filled rectangle
/// no more than two feathering widths across as a line segment, rounds a one
/// pixel junction away, and shrinks a segment's ends by a quarter pixel, so
/// either leaves holes or overlaps at the crossings. A quad on whole pixels has
/// no edge to anti-alias.
///
pub(super) fn edge_mesh(
    edges: &Edges,
    drawn: &VisiblePositions,
    columns: &[i32],
    rows: &[i32],
    pixels_per_point: f32,
) -> Mesh {
    let (width, height) = (drawn.columns.len(), drawn.rows.len());
    // Every edge, row line by row line and column line by column line.
    let mut across = vec![None; (height + 1) * width];
    for run in &edges.rows {
        let row = run.line - drawn.rows.start;
        for column in run.span.clone() {
            across[row * width + column - drawn.columns.start] =
                Line::new(run.edge, pixels_per_point);
        }
    }
    let mut down = vec![None; (width + 1) * height];
    for run in &edges.columns {
        let column = run.line - drawn.columns.start;
        for row in run.span.clone() {
            down[column * height + row - drawn.rows.start] = Line::new(run.edge, pixels_per_point);
        }
    }
    let across_at = |column: Option<usize>, row: usize| {
        column
            .filter(|column| *column < width)
            .and_then(|column| across[row * width + column])
    };
    let down_at = |column: usize, row: Option<usize>| {
        row.filter(|row| *row < height)
            .and_then(|row| down[column * height + row])
    };

    // The junctions along row line `row`, each built once: a crossing's own
    // pieces and the ends of the edges that meet it are both read from them.
    let junctions = |row: usize| {
        columns
            .iter()
            .enumerate()
            .map(|(column, &column_line)| {
                let lines = [
                    down_at(column, row.checked_sub(1)),
                    down_at(column, Some(row)),
                    across_at(column.checked_sub(1), row),
                    across_at(Some(column), row),
                ];
                Junction::new(lines, column_line, rows[row])
            })
            .collect::<Vec<_>>()
    };

    // Reserved up front, since the Mesh is rebuilt every frame and outgrowing
    // the reserve doubles and copies it: a quad per edge, and two per junction,
    // since one where lines of two ranks meet takes more than one.
    let quads = 2 * columns.len() * rows.len()
        + across.iter().flatten().count()
        + down.iter().flatten().count();
    let mut mesh = Mesh::default();
    mesh.reserve_triangles(2 * quads);
    mesh.reserve_vertices(4 * quads);
    let mut this = junctions(0);
    for (row, &row_line) in rows.iter().enumerate() {
        let next = if row + 1 < rows.len() {
            junctions(row + 1)
        } else {
            Vec::new()
        };
        for (column, (junction, &column_line)) in this.iter().zip(columns).enumerate() {
            junction.draw(&mut mesh, column_line, row_line, pixels_per_point);
            let [_, below, _, right] = junction.lines;
            if let Some(line) = right {
                let (x, y) = (
                    junction.x.end..this[column + 1].x.start,
                    band(row_line, line.width),
                );
                quad(&mut mesh, x, y, line.colour, pixels_per_point);
            }
            if let Some(line) = below {
                let (x, y) = (
                    band(column_line, line.width),
                    junction.y.end..next[column].y.start,
                );
                quad(&mut mesh, x, y, line.colour, pixels_per_point);
            }
        }
        this = next;
    }

    mesh
}

///
/// A [`Paint`] as the Shapes that draw it, grouped by the order they are
/// painted in.
///
/// # Why the groups are fields
///
/// The order that matters is across kinds and not across Cells: every
/// background precedes every Glyph, so a later Cell's fill can never paint over
/// an earlier Cell's Glyph. The edges, the selection outline among them, sit
/// between the two, after the fills, because a run widened across several
/// Cells would cover every edge inside it. That is a fact about how a painter
/// composites rather than about the Source, which is why it is decided here and
/// not in the value layer — and the ordered groups name it in the type.
///
/// # Why they are built eagerly
///
/// Shape construction is never lazy. `Painter::extend` runs the iterator it is
/// given inside `ctx.graphics_mut`, a full `Context` write lock, so a Shape
/// built lazily is a Shape built while holding it. The fields are complete
/// before [`SourceShapes::into_shapes`] hands the first one out.
///
pub struct SourceShapes {
    /// The coalesced background runs, one rectangle each.
    pub(super) backgrounds: Vec<Shape>,
    /// Every Cell edge — the Grid lines, every border fact and the selection
    /// outline — as one Mesh, absent where no edge has a line.
    pub(super) edges: Option<Shape>,
    /// One galley per Cell that shows a character other than the space.
    pub(super) glyphs: Vec<Shape>,
}

impl SourceShapes {
    ///
    /// Draws a Paint at `viewport`: the geometry the value layer carries none
    /// of, applied to the colours and characters it carries all of.
    ///
    /// Stroke widths are fixed display points at every device scale and egui
    /// zoom factor, never multiplied by the Cell size: each edge's width is
    /// already resolved onto [`Paint::edges`]. `pixels_per_point` is the
    /// device scale the background runs are snapped to (see
    /// [`background_run`]) and the edges are placed at (see [`edge_mesh`]).
    ///
    /// Edges and the background runs need no font atlas. Glyph placement does — see [`Self::place_glyphs`] —
    /// so this composes the two steps rather than folding the atlas into the
    /// geometry pass.
    ///
    pub(super) fn new(
        paint: &Paint,
        viewport: &GridViewport,
        table: &GlyphTable,
        pixels_per_point: f32,
    ) -> Self {
        let mut shapes = Self::place(paint, viewport, pixels_per_point);
        shapes.place_glyphs(paint, viewport, table);
        shapes
    }

    ///
    /// Backgrounds and edges — every Shape whose construction is arithmetic
    /// on a `Rect` and a `Color32`.
    ///
    /// Takes no [`GlyphTable`] and no `egui::Context`. The `glyphs` field is
    /// empty until [`Self::place_glyphs`] fills it; both steps finish before
    /// [`Self::into_shapes`] hands the first Shape out.
    ///
    #[cfg(any(test, feature = "test-source-shapes"))]
    pub fn geometry(paint: &Paint, viewport: &GridViewport, pixels_per_point: f32) -> Self {
        Self::place(paint, viewport, pixels_per_point)
    }

    fn place(paint: &Paint, viewport: &GridViewport, pixels_per_point: f32) -> Self {
        // Backgrounds are sparse selection state, so that group starts empty.
        // Glyphs are reserved in [`Self::place_glyphs`].
        let mut backgrounds = Vec::new();

        for run in paint.background_runs() {
            // Built from the Cells' own rectangles — `GridViewport::cell_rect`
            // is the column-to-x function every other Shape here goes through —
            // so a coalesced edge is the edge the Cells it replaces would have
            // been given rather than a sum of Cell sides accumulated across the
            // row. A run covers at least one column, so `end - 1` is a column
            // of the run.
            //
            // Asserted rather than assumed: the non-emptiness is
            // `Paint::background_runs`' — it seeds a run as
            // `position.x()..position.x() + 1` and only ever extends the end —
            // and nothing in `BackgroundRun` holds the fold to it. An empty
            // range would wrap `end - 1` to `usize::MAX` in release and hand
            // `cell_rect` a column off the far side of the Grid, which paints a
            // wild rectangle rather than panicking.
            debug_assert!(
                !run.columns.is_empty(),
                "a background run covers at least one column, not {:?}",
                run.columns
            );
            let covered = Rect::from_min_max(
                viewport.cell_rect(run.columns.start, run.row).min,
                viewport.cell_rect(run.columns.end - 1, run.row).max,
            );
            backgrounds.push(background_run(covered, run.colour, pixels_per_point));
        }

        // Every boundary of the drawn range, the far edge of its last column
        // and row included, in physical pixels. `cell_rect` is a function of
        // one index, so a boundary past the last drawn Cell is the near edge
        // the next Cell would have.
        let drawn = paint.drawn();
        let edges = paint.edges();
        let edge_shapes = (!edges.rows.is_empty() || !edges.columns.is_empty()).then(|| {
            let pixel = |point: f32| (point * pixels_per_point).round() as i32;
            let columns = (drawn.columns.start..=drawn.columns.end)
                .map(|column| pixel(viewport.cell_rect(column, drawn.rows.start).min.x))
                .collect::<Vec<_>>();
            let rows = (drawn.rows.start..=drawn.rows.end)
                .map(|row| pixel(viewport.cell_rect(drawn.columns.start, row).min.y))
                .collect::<Vec<_>>();
            Shape::mesh(edge_mesh(&edges, drawn, &columns, &rows, pixels_per_point))
        });

        Self {
            backgrounds,
            edges: edge_shapes,
            glyphs: Vec::new(),
        }
    }

    ///
    /// Places one galley per Cell that shows a character other than the space.
    ///
    /// Needs a [`GlyphTable`] — a galley's size exists only after layout — and
    /// fills `glyphs` eagerly, for the reason [`SourceShapes`] gives.
    ///
    fn place_glyphs(&mut self, paint: &Paint, viewport: &GridViewport, table: &GlyphTable) {
        self.glyphs = Vec::with_capacity(paint.count());

        for (position, cell) in paint.cells() {
            if cell.character == ' ' {
                continue;
            }

            let rect = viewport.cell_rect(position.x(), position.y());
            let galley = table.glyph(cell.character);
            // Centred in a Cell whose own corner is an exact multiple of the
            // Cell size.
            self.glyphs.push(Shape::galley(
                rect.center() - galley.size() / 2.0,
                galley,
                cell.foreground,
            ));
        }
    }

    ///
    /// The shape groups end to end, in paint order.
    ///
    /// An iterator over the owned groups rather than a fourth `Vec`: the chain
    /// costs nothing, and collecting it would spend a further allocation of
    /// some two thousand elements, and a whole re-move, per Render Frame.
    ///
    pub(super) fn into_shapes(self) -> impl Iterator<Item = Shape> {
        self.backgrounds
            .into_iter()
            .chain(self.edges)
            .chain(self.glyphs)
    }
}

///
/// Draws the Source Grid and answers the one question a click asks of it:
/// which Cell, and whether Shift extends the Region to it.
///
/// The whole Grid is one allocated rectangle and no Cell is a widget, so the
/// cost of a Render Frame is shapes rather than interaction rects and widget
/// ids. Only the Positions the viewport covers are painted —
/// `GridViewport::visible_positions` is this loop's one source of them — so
/// that cost follows the viewport rather than the Source.
///
/// A background is painted only where selection state differs from the Source
/// fill the panel already provides. Consecutive Cells in a row that want the
/// same background share one rectangle.
///
/// The click is answered rather than acted on. Selecting a Cell is the Source's
/// business and `Console::show_source_panel` owns the running Orcvs it is asked
/// of; handing the [`PointerSelection`] back is what leaves this function with
/// nothing but a Render Frame and a place to draw it.
///
pub(super) fn show_source(
    ui: &mut egui::Ui,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    viewport: GridViewport,
    clip: Rect,
    theme: &Theme,
) -> Option<PointerSelection> {
    // The shape the Render Frame was derived from, named apart from the
    // `GridViewport` the Cells are painted at.
    let source_grid = frame.grid();
    // One rectangle for the whole Grid, sensing clicks and nothing else.
    //
    // Within a layer a later-registered child wins the click tie, and would win
    // the drag too if it sensed drag. The pan rectangle `show_source_scene`
    // allocates is registered before this one, so sensing clicks alone takes
    // the clicks and leaves the middle-drag and the Alt-held primary-drag Pan
    // to it. `Sense::CLICK` rather than `Sense::click()`, which is
    // `CLICK | FOCUSABLE` and would put the Grid in the tab order where a
    // thousand Buttons never were.
    //
    // The rectangle is the Grid, not the console area. Surplus console past the
    // Grid's edges is not a Cell, so a click there selects nothing.
    // Clipped to the console, because `Ui::interact` bounds a widget by the
    // `Ui`'s clip rect rather than by the console area, and a Grid larger than
    // the console reaches past it on every side. No container sets that clip
    // rect, so the Grid states its own bound.
    let response = ui.interact(
        viewport.rect.intersect(clip),
        ui.id().with("source_grid"),
        Sense::CLICK,
    );
    // The Grid is clipped to the console area, so a panned Grid cannot paint
    // over the chrome around it. The Source Grid is all this layer holds, so
    // painting it directly orders it without a sublayer.
    let painter = ui.painter().with_clip_rect(clip);
    // The device scale the background runs are snapped to; see
    // [`background_run`].
    let pixels_per_point = ui.pixels_per_point();
    let table = GlyphTable::lay_out(
        ui.ctx(),
        // The Cell's own Glyph size in points. egui's `pixels_per_point`,
        // which folds in its zoom factor, is what makes it more physical
        // pixels, so a Glyph is rasterised at the size it is drawn at and
        // never resampled.
        FontId::new(viewport.glyph_size(), font_family.clone()),
    );

    // The only source of Positions the two steps below have. Everything they
    // reach is inside these ranges, so the cost of a Render Frame follows the
    // viewport rather than the Source: a Grid the console shows a tenth of
    // costs a tenth of the Cell iteration and a tenth of the Shapes, whatever
    // the Grid's size.
    //
    // The ranges reach one Cell past what is visible. An edge shows the line
    // of the higher-ranked fact either Cell beside it carries, so the edge at
    // the last visible Cell's right or bottom side needs the Cell one past it.
    // That Cell's own pieces land on the clip's boundary, which is what the
    // clip rectangle above discards. See `GridViewport::visible_positions` for
    // why that makes the margin a boundary case rather than a seam that would
    // otherwise go missing.
    //
    // The bounds are the Render Frame's own Grid's, rather than a shape
    // recovered out of its rows.
    let visible = viewport.visible_positions(clip, source_grid);

    // What the console decided to draw, then what draws it. The decision is a
    // value derived from the Render Frame and the range above, so what colour a
    // Cell is can be asked without a `Context`, a window or a running Orcvs.
    let paint = Paint::derive_with_theme(FramePaint::new(frame, visible), theme);
    let shapes = SourceShapes::new(&paint, &viewport, &table, pixels_per_point);

    // One `Painter::extend`, never a `Painter::add` per Shape. `add` reaches
    // `Context::graphics_mut`, which is a full `Context` write lock, so a
    // per-Cell loop would take one lock per Shape.
    painter.extend(shapes.into_shapes());

    // The click resolves by division through the viewport the Cells were
    // painted at. With no layer transform, `interact_pointer_pos` is in global
    // points, which is the space the presented Grid is in.
    // Shift with a click extends the Region from its anchor; a click on its
    // own collapses it.
    if response.clicked()
        && let Some(pointer) = response.interact_pointer_pos()
        && let Some((column, row)) = viewport.cell_at(pointer, source_grid)
    {
        let extend = ui.input(|i| i.modifiers.shift);
        source_grid.position(column, row).map(|position| {
            if extend {
                PointerSelection::Extend(position)
            } else {
                PointerSelection::Select(position)
            }
        })
    } else {
        None
    }
}
