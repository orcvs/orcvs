//! Drawing the Source Grid: the Shapes a Render Frame's Cells are painted as,
//! in paint order, and the one click the Grid answers.

use std::ops::Range;

use egui::{
    Color32, CornerRadius, FontId, Mesh, Pos2, Rect, Sense, Shape, Stroke, emath::GuiRounding as _,
    epaint::RectShape,
};
use orcvs::{opts::DEFAULT_FONT_SIZE, render_frame::RenderFrame};

use super::glyphs::GlyphTable;
use super::source_view::PointerSelection;
use crate::cursor_effects::{CursorEffectMotion, cursor_effect_shapes};
use crate::grid_viewport::GridViewport;
use crate::grid_viewport::VisiblePositions;
use crate::paint::{EdgeRank, EdgeRun, Edges, FramePaint, Paint};
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
/// The edges of `drawn`, as one Mesh of quads that never overlap: a junction
/// where each column line crosses each row line, and a piece for each edge
/// between two junctions.
///
/// `columns` and `rows` are the boundaries of `drawn` in physical pixels, left
/// to right and top to bottom, the far edges of its last column and row
/// included, so each axis is spaced by its own Cell side.
///
/// An edge's piece lies across its boundary at its own width. A junction is as
/// wide as the widest column line meeting it and as tall as the widest row
/// line, in the colour of the highest-ranked line meeting it, so every piece
/// runs between junctions and a narrower line's band lies inside the
/// junction's.
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
    let place = |run: &EdgeRun| {
        PixelLine::new(run.edge.stroke, pixels_per_point).map(|line| (run.edge.rank, line))
    };
    // Every edge, row line by row line and column line by column line.
    let mut across = vec![None; (height + 1) * width];
    for run in &edges.rows {
        let row = run.line - drawn.rows.start;
        for column in run.span.clone() {
            across[row * width + column - drawn.columns.start] = place(run);
        }
    }
    let mut down = vec![None; (width + 1) * height];
    for run in &edges.columns {
        let column = run.line - drawn.columns.start;
        for row in run.span.clone() {
            down[column * height + row - drawn.rows.start] = place(run);
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
    // The junction at a crossing: its band across the column line and its
    // band across the row line, and its colour.
    let junction = |column: usize, row: usize| {
        let downs = [
            down_at(column, row.checked_sub(1)),
            down_at(column, Some(row)),
        ];
        let acrosses = [
            across_at(column.checked_sub(1), row),
            across_at(Some(column), row),
        ];
        let widest = |lines: &[Option<(EdgeRank, PixelLine)>]| {
            lines
                .iter()
                .flatten()
                .map(|(_, line)| line.width)
                .max()
                .unwrap_or(0)
        };
        let colour = downs
            .iter()
            .chain(&acrosses)
            .flatten()
            .max_by_key(|(rank, _)| *rank)
            .map(|(_, line)| line.colour);
        (
            band(columns[column], widest(&downs)),
            band(rows[row], widest(&acrosses)),
            colour,
        )
    };

    let mut mesh = Mesh::default();
    let mut quad = |x: Range<i32>, y: Range<i32>, colour: Color32| {
        if !x.is_empty() && !y.is_empty() {
            let point = |x: i32, y: i32| {
                Pos2::new(x as f32 / pixels_per_point, y as f32 / pixels_per_point)
            };
            mesh.add_colored_rect(
                Rect::from_min_max(point(x.start, y.start), point(x.end, y.end)),
                colour,
            );
        }
    };
    for (row, &row_line) in rows.iter().enumerate() {
        for (column, &column_line) in columns.iter().enumerate() {
            let (x, y, colour) = junction(column, row);
            if let Some(colour) = colour {
                quad(x.clone(), y.clone(), colour);
            }
            if let Some((_, line)) = across_at(Some(column), row) {
                let (next, _, _) = junction(column + 1, row);
                quad(x.end..next.start, band(row_line, line.width), line.colour);
            }
            if let Some((_, line)) = down_at(column, Some(row)) {
                let (_, next, _) = junction(column, row + 1);
                quad(
                    band(column_line, line.width),
                    y.end..next.start,
                    line.colour,
                );
            }
        }
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
/// an earlier Cell's Glyph, and the Cursor Effect's fragments follow both.
/// The edges, the selection outline among them, sit between the two,
/// after the fills, because a run widened across several Cells would cover
/// every edge inside it. That is a fact about how a painter composites rather
/// than about the Source, which is why it is decided here and not in the value
/// layer — and the ordered groups name it in the type.
///
/// # Why they are built eagerly
///
/// Shape construction is never lazy. `Painter::extend` runs the iterator it is
/// given inside `ctx.graphics_mut`, a full `Context` write lock, so a Shape
/// built lazily is a Shape built while holding it. The fields are complete
/// before `SourceShapes::into_shapes` hands the first one out.
///
pub struct SourceShapes {
    /// The living field beneath every exact Grid shape.
    pub(super) area: Vec<Shape>,
    /// The coalesced background runs, one rectangle each.
    pub(super) backgrounds: Vec<Shape>,
    /// Every Cell edge — the Grid lines, every border fact and the selection
    /// outline — as one Mesh, absent where no edge has a line.
    pub(super) edges: Vec<Shape>,
    /// One galley per Cell that shows a character other than the space.
    pub(super) glyphs: Vec<Shape>,
    /// The Cursor Effect's animated fragments, painted last, around its Cell
    /// or the lasso around a Region larger than one Cell.
    pub(super) cursor: Vec<Shape>,
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
        cursor_effect: crate::cursor_effects::CursorEffectShapes,
    ) -> Self {
        let mut shapes = Self::geometry(paint, viewport, pixels_per_point);
        shapes.area = cursor_effect.area;
        shapes.cursor = cursor_effect.fragments;
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
    pub fn geometry(paint: &Paint, viewport: &GridViewport, pixels_per_point: f32) -> Self {
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
        let mut edge_shapes = Vec::new();
        if !edges.rows.is_empty() || !edges.columns.is_empty() {
            let pixel = |point: f32| (point * pixels_per_point).round() as i32;
            let columns = (drawn.columns.start..=drawn.columns.end)
                .map(|column| pixel(viewport.cell_rect(column, drawn.rows.start).min.x))
                .collect::<Vec<_>>();
            let rows = (drawn.rows.start..=drawn.rows.end)
                .map(|row| pixel(viewport.cell_rect(drawn.columns.start, row).min.y))
                .collect::<Vec<_>>();
            edge_shapes.push(Shape::mesh(edge_mesh(
                &edges,
                drawn,
                &columns,
                &rows,
                pixels_per_point,
            )));
        }

        Self {
            area: Vec::new(),
            backgrounds,
            edges: edge_shapes,
            glyphs: Vec::new(),
            cursor: Vec::new(),
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
    /// An iterator over the owned `Vec`s rather than a sixth one: the chain
    /// costs nothing, and collecting it would spend a further allocation of
    /// some two thousand elements, and a whole re-move, per Render Frame.
    ///
    pub(super) fn into_shapes(self) -> impl Iterator<Item = Shape> {
        self.area
            .into_iter()
            .chain(self.backgrounds)
            .chain(self.edges)
            .chain(self.glyphs)
            .chain(self.cursor)
    }
}

///
/// The rectangle the Cursor Effect's frame outlines: the Cursor's Cell, or the
/// whole Region when it spans more than one Cell — the lasso.
///
pub(super) fn effect_outline(frame: &RenderFrame, viewport: &GridViewport) -> Rect {
    let region = frame.region();
    let (columns, rows) = (region.columns(), region.rows());
    Rect::from_min_max(
        viewport.cell_rect(columns.start, rows.start).min,
        viewport.cell_rect(columns.end - 1, rows.end - 1).max,
    )
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
    cursor_effect: CursorEffectMotion,
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
        // The Source's own size in points. egui's `pixels_per_point`, which
        // folds in its zoom factor, is what makes it more physical pixels, so
        // a Glyph is rasterised at the size it is drawn at and never
        // resampled.
        FontId::new(DEFAULT_FONT_SIZE, font_family.clone()),
    );

    // The only source of Positions the two steps below have. Everything they
    // reach is inside these ranges, so the cost of a Render Frame follows the
    // viewport rather than the Source: a Grid the console shows a tenth of
    // costs a tenth of the Cell iteration and a tenth of the Shapes, whatever
    // the Grid's size.
    //
    // The ranges reach one Cell past what is visible. A Cell carries its own
    // left and top seams, so the seam at a Cell's right or bottom edge belongs
    // to the Cell one past it, and a border feathers a physical pixel outside the
    // rectangle it strokes. Both land on the clip's boundary, which is what the
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
    let cursor_rect = viewport.cell_rect(frame.cursor().x(), frame.cursor().y());
    // The Cursor's own frame outlines its own Cell; the lasso around a
    // Region larger than one Cell takes the Region's own outline colour
    // instead — `.scratch/theming/schema.md`'s "the effect outline uses
    // `cursor.border` or `region.border`". `paint` already answered which
    // this Render Frame is, so this reads that rather than re-deriving it.
    // The same choice picks the animated frame's nominal width, as one
    // `Stroke` so colour and width cannot come from different answers:
    // `cursor.border.width` for the Cursor's own frame, `region.border.width`
    // for the lasso — a fixed display-point value `cursor_effect_shapes` never
    // scales with the Cell size.
    let frame_stroke = if paint.region_spans() {
        egui::Stroke::new(theme.region_border_width.points(), theme.region_border)
    } else {
        egui::Stroke::new(theme.cursor_border_width.points(), theme.cursor_border)
    };
    let cursor_effect = cursor_effect_shapes(
        cursor_rect,
        effect_outline(frame, &viewport),
        clip,
        viewport.cell_size,
        cursor_effect,
        theme.cursor_area,
        frame_stroke,
    );
    let shapes = SourceShapes::new(&paint, &viewport, &table, pixels_per_point, cursor_effect);

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
