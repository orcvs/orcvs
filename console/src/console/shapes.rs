//! Drawing the Source Grid: the Shapes a Render Frame's Cells are painted as,
//! in paint order, and the one click the Grid answers.

use egui::{
    Color32, CornerRadius, FontId, Rect, Sense, Shape, Stroke, StrokeKind, emath::GuiRounding as _,
    epaint::RectShape,
};
use orcvs::{opts::DEFAULT_FONT_SIZE, render_frame::RenderFrame};

use super::glyphs::{GlyphTable, glyph_scale};
use super::source_view::PointerSelection;
use crate::cursor_effects::{CursorEffectSample, CursorEffectSettings, cursor_effect_shapes};
use crate::grid_viewport::GridViewport;
use crate::paint::{FramePaint, Paint};
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
/// `tessellate_rect` then applies `Rect::round_to_pixels` — the same `emath`
/// function this calls — at `epaint-0.36.2/src/tessellator.rs:1829-1862`. A run
/// left unsnapped here would reach the screen as the same pixels.
///
/// It is snapped so the rectangle is one a test can predict. The snap is the
/// last thing that moves an edge, so doing it here puts the Shape a Render
/// Frame carries at the coordinates the paint lands on, and an assertion can
/// state them exactly rather than within a pixel. `Rect::round_to_pixels`
/// rounds the two corners independently
/// (`emath-0.36.2/src/gui_rounding.rs:155-186`), so a run's far edge lands on
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
/// A [`Paint`] as the Shapes that draw it, grouped by the order they are
/// painted in.
///
/// # Why the groups are fields
///
/// The order that matters is across kinds and not across Cells: every
/// background precedes every Glyph, so a later Cell's fill can never paint over
/// an earlier Cell's Glyph, and the Cursor follows both, so no neighbouring
/// Cell's fill or seam can paint over it. The borders sit between the two,
/// after the fills, because a run widened across several Cells covers the
/// borders of every Cell but its last. That is a fact about how a painter
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
pub(super) struct SourceShapes {
    /// The living field beneath every exact Grid shape.
    pub(super) area: Vec<Shape>,
    /// The coalesced background runs, one rectangle each.
    pub(super) backgrounds: Vec<Shape>,
    /// Every Cell's own border but the Cursor's, and the Cursor's too while a
    /// Region spans more than one Cell.
    pub(super) borders: Vec<Shape>,
    /// One galley per Cell that shows a character other than the space.
    pub(super) glyphs: Vec<Shape>,
    /// The sector seams, left edge then top edge, Cell by Cell.
    pub(super) seams: Vec<Shape>,
    /// The Cursor's own stroke, painted last: the Cursor Effect's frame — its
    /// Cell's, or the lasso around a Region larger than one Cell, and empty
    /// when a zero width hid it — or the selected Cell's border when no
    /// effect frame was built.
    pub(super) cursor: Vec<Shape>,
}

impl SourceShapes {
    ///
    /// Draws a Paint at `viewport`: the geometry the value layer carries none
    /// of, applied to the colours and characters it carries all of.
    ///
    /// Stroke widths are fixed display points that stay the same visible
    /// thickness at every Grid zoom (`.scratch/theming/issues/06` slice C),
    /// unlike the [`GridViewport::cell_scale`]-multiplied constants this
    /// replaced: `sector.seam.width` from the resolved `theme`, and each
    /// Cell's own border width already resolved onto it as `cell.
    /// border_width` (`grid.border.width`, `cell.selection.border.width`, or
    /// either composited with Diagnostic/Output Portal, by fact priority).
    /// `pixels_per_point` is the device scale the background runs are
    /// snapped to; see [`background_run`].
    ///
    /// Borders, the Cursor's border, the sector seams and the background runs
    /// need no font atlas. Glyph placement does — see [`Self::place_glyphs`] —
    /// so this composes the two steps rather than folding the atlas into the
    /// geometry pass.
    ///
    pub(super) fn new(
        paint: &Paint,
        viewport: &GridViewport,
        table: &GlyphTable,
        pixels_per_point: f32,
        cursor_effect: crate::cursor_effects::CursorEffectShapes,
        theme: &Theme,
    ) -> Self {
        let mut shapes = Self::geometry(paint, viewport, pixels_per_point, theme);
        shapes.area = cursor_effect.area;
        if let Some(frame) = cursor_effect.frame {
            shapes.cursor = frame;
        }
        shapes.place_glyphs(paint, viewport, table);
        shapes
    }

    ///
    /// Backgrounds, borders, the Cursor's border and the sector seams — every
    /// Shape whose construction is arithmetic on a `Rect` and a `Color32`.
    ///
    /// Takes no [`GlyphTable`] and no `egui::Context`. The `glyphs` field is
    /// empty until [`Self::place_glyphs`] fills it; both steps finish before
    /// [`Self::into_shapes`] hands the first Shape out.
    ///
    /// `theme.sector_seam_width` is read once, here, rather than once per
    /// Cell inside the loop below — `.scratch/theming/issues/06`'s "resolve
    /// widths once per frame" — and a width of exactly zero skips building
    /// that Shape outright rather than emitting a zero-width one for the
    /// painter to drop, per the same issue's "Width 0 hides the stroke." Each
    /// Cell's own border width has no one frame-level constant to read here:
    /// `cell.border_width` already carries the fact-priority pick
    /// `crate::style::cell_visuals_with_cursor_colour` and
    /// `crate::style::ordinary_border` resolved for it — `grid.border.width`
    /// for an ordinary Cell composited with Diagnostic/Output Portal,
    /// `cell.selection.border.width` for a single-Cell Cursor — so this step
    /// reads that answer per Cell rather than choosing among Theme fields
    /// itself.
    ///
    pub(super) fn geometry(
        paint: &Paint,
        viewport: &GridViewport,
        pixels_per_point: f32,
        theme: &Theme,
    ) -> Self {
        // Border widths do not read `theme` here: `cell.border_width` is
        // already the resolved, fact-priority-picked answer — see the loop
        // below. Sector Seam width has no per-Cell fact to vary by, so it
        // stays a plain frame-level read.
        let sector_seam_width = theme.sector_seam_width.points();
        // A border is the rule on every Cell the Paint covers, so it is sized
        // up front — to those Cells rather than to the Grid: a densely written
        // Source that regrew the group would pay the reallocation on every
        // Render Frame, and a zoomed console reserves what it draws instead of
        // what the Source holds. Backgrounds are sparse selection state, so
        // that group starts empty. Glyphs are reserved in
        // [`Self::place_glyphs`].
        let mut backgrounds = Vec::new();
        let mut borders = Vec::with_capacity(paint.count());
        let mut seams = Vec::new();
        let mut cursor = Vec::new();

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

        for (position, cell) in paint.cells() {
            let rect = viewport.cell_rect(position.x(), position.y());
            // The Cell's own border, stroke and no fill: a widened run paints
            // over the borders of every Cell inside it, so the fill and the
            // border cannot be one shape.
            //
            // `cell.border_width` is already the resolved answer —
            // `crate::style::cell_visuals_with_cursor_colour` and
            // `crate::style::ordinary_border`'s fact-priority pick, threaded
            // through `CellPaint` — so this step is purely mechanical: a
            // single-Cell Cursor's own `cell.selection.border.width`, or the
            // ordinary Grid border's width, itself composited with
            // Diagnostic/Output Portal by the same priority
            // (`.scratch/theming/schema.md`'s Source composition steps 4 and
            // 5). Each width independently hides its own stroke at zero,
            // since it is this Cell's *only* width, decided once above.
            if cell.border_width > 0.0 {
                let border = Shape::Rect(RectShape::stroke(
                    rect,
                    CornerRadius::ZERO,
                    Stroke::new(cell.border_width, cell.border),
                    StrokeKind::Inside,
                ));

                // The selected Cell's border is the Cursor, and the Cursor is
                // painted last. A Cursor the viewport does not reach is no
                // Cell of this Paint, so the comparison never matches and the
                // group stays empty.
                if paint.cursor() == Some(position) && !paint.region_spans() {
                    cursor.push(border);
                } else {
                    borders.push(border);
                }
            }

            // A seam is absent on the Cursor's Cell, while it is framed on its
            // own, because the derive suppressed it there, so this step never
            // learns that rule.
            if sector_seam_width > 0.0 {
                for (colour, ends) in [
                    (cell.sector_left, [rect.left_top(), rect.left_bottom()]),
                    (cell.sector_top, [rect.left_top(), rect.right_top()]),
                ] {
                    if let Some(colour) = colour {
                        seams.push(Shape::line_segment(
                            ends,
                            Stroke::new(sector_seam_width, colour),
                        ));
                    }
                }
            }
        }

        Self {
            area: Vec::new(),
            backgrounds,
            borders,
            glyphs: Vec::new(),
            seams,
            cursor,
        }
    }

    ///
    /// Places one galley per Cell that shows a character other than the space.
    ///
    /// Needs a [`GlyphTable`] — a galley's size exists only after layout — and
    /// fills `glyphs` eagerly so [`Self::into_shapes`] never builds a Shape
    /// while holding the `Context` write lock `Painter::extend` takes.
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
            .chain(self.borders)
            .chain(self.glyphs)
            .chain(self.seams)
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
/// business and `Console::ui` owns the running Orcvs it is asked of; handing the
/// [`PointerSelection`] back is what leaves this function with nothing but a
/// Render Frame and a place to draw it.
///
/// Eight parameters, one over clippy's default: `theme` is the eighth,
/// added by `syntax-highlighting/01` as `source_paint` and repurposed by
/// `.scratch/theming/issues/06`. Each of the eight is an independent,
/// already-tested value threaded straight through from `show_source_scene`'s
/// own parameters of the same names — geometry, a Render Frame, and the two
/// presentation values `Console::ui` owns — so grouping any of them into a
/// struct would add an indirection this function's one caller does not need,
/// for a threshold rather than a real complexity this function has grown.
///
#[allow(clippy::too_many_arguments)]
pub(super) fn show_source(
    ui: &mut egui::Ui,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    viewport: GridViewport,
    clip: Rect,
    cursor_effect_sample: CursorEffectSample,
    cursor_effect_settings: CursorEffectSettings,
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
    // `Ui`'s clip rect rather than by the console area, and a zoomed-in Grid
    // reaches past the console on every side. No container sets that clip
    // rect, so the Grid states its own bound.
    let response = ui.interact(
        viewport.rect.intersect(clip),
        ui.id().with("source_grid"),
        Sense::CLICK,
    );
    // The Grid is clipped to the console area, so a zoomed Grid cannot paint
    // over the chrome around it. The Source Grid is all this layer holds, so
    // painting it directly orders it without a sublayer.
    let painter = ui.painter().with_clip_rect(clip);
    // The scale is already in the Cell size, and the strokes need it too, so
    // the Grid lines and sector seams take it here
    // rather than staying one Source point wide at every zoom.
    // The device scale the background runs are snapped to; see
    // [`background_run`].
    let pixels_per_point = ui.pixels_per_point();
    let table = GlyphTable::lay_out(
        ui.ctx(),
        // Laid out at the size it is drawn at rather than resampled from a
        // rasterisation at the Source's own size, which is the second thing the
        // layer transform cost. The scale is quantised so a steady zoom hits
        // the galley cache and a sweep across the zoom range stays inside the
        // atlas; see `GLYPH_SCALE_STEP`.
        FontId::new(
            DEFAULT_FONT_SIZE * glyph_scale(viewport.cell_scale()),
            font_family.clone(),
        ),
    );

    // The only source of Positions the two steps below have. Everything they
    // reach is inside these ranges, so the cost of a Render Frame follows the
    // viewport rather than the Source: a Grid the console shows a tenth of
    // costs a tenth of the Cell iteration and a tenth of the Shapes, whatever
    // the Grid's size.
    //
    // The ranges reach one Cell past what is visible. A Cell draws its own left
    // and top seams, so the seam at a Cell's right or bottom edge belongs to
    // the Cell one past it, and a border feathers a physical pixel outside the
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
    // scales with Grid zoom (`.scratch/theming/issues/06` slice C).
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
        cursor_effect_sample,
        cursor_effect_settings,
        theme.cursor_area,
        frame_stroke,
    );
    let shapes = SourceShapes::new(
        &paint,
        &viewport,
        &table,
        pixels_per_point,
        cursor_effect,
        theme,
    );

    // One `Painter::extend`, never a `Painter::add` per Shape. `add` reaches
    // `Context::graphics_mut`, which is a full `Context` write lock, so a
    // per-Cell loop would take more locks than the Button field it replaces and
    // turn this change into a regression.
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
