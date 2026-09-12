use std::sync::Arc;

use egui::{
    Color32, CornerRadius, Event, EventFilter, FontId, Key, PointerButton, Pos2, Rect, Sense,
    Shape, Stroke, StrokeKind, Vec2, containers::DragPanButtons, emath::GuiRounding as _,
    emath::TSTransform, epaint::RectShape, text::Galley,
};

use crate::grid_viewport::{GridViewport, grid_viewport, presented_grid};
use crate::midi::MidiDeviceSelection;
use crate::paint::Paint;
use crate::persistence::starting_source;
use crate::style::{PALETTE, style};
use orcvs::{
    app::{InputEvent, InputKey, Orcvs},
    grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT, Position},
    native_midi::{self, NativeMidiBackend},
    opts::{Bpm, DEFAULT_FONT_SIZE},
    render_frame::RenderFrame,
};

const CELL_SIZE: f32 = 25.0;
const GRID_LINE_WIDTH: f32 = 0.5;
const SECTOR_LINE_WIDTH: f32 = 0.75;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 2.0;

///
/// The step the scale is quantised to before it reaches a [`FontId`].
///
/// # Why this is an atlas budget, not a cache-hit rate
///
/// A Glyph is laid out at the size it is drawn at, so the scale has to reach
/// the font size. A *continuous* scale would reach it as a fresh size per
/// Render Frame, and epaint rasterises a fresh glyph set per distinct size —
/// `FontImpl::glyph_info` scales by `font_size * pixels_per_point` and rounds
/// nothing (`epaint-0.36.1/src/text/font.rs:567`). `subpixel_binning` is on by
/// default (`epaint-0.36.1/src/text/mod.rs:62`) and renders each glyph at up to
/// four fractional offsets, so a zoom sweep across `N` sizes costs up to
/// `N x alphabet x 4` rasters into one atlas.
///
/// That is the budget, because the atlas is not merely wasted when it fills:
/// `Fonts::begin_pass` replaces the whole `FontsImpl` — a new atlas with empty
/// glyph caches — as soon as `atlas.fill_ratio()` passes 0.8
/// (`epaint-0.36.1/src/text/fonts.rs:734-748`), restarting glyph rasterisation
/// mid-session for every size already paid for.
///
/// At a step of an eighth, the zoom range `MIN_ZOOM..=MAX_ZOOM` holds fifteen
/// distinct scales, so a viewer who sweeps that range spends at most
/// `15 x 94 x 4 = 5,640` rasters — roughly two megapixels of a 2048-square
/// atlas at one device pixel per point, which stays inside the fill ratio.
/// Both zoom limits and the Source's own scale are exact multiples of the step,
/// so the default window and either end of the range land on it rather than
/// beside it.
///
/// Fifteen is the floor of the budget, not its ceiling, and the honest
/// statement is that the step bounds a sweep rather than eliminating it. The
/// range the console actually offers is `min_zoom..=MAX_ZOOM.max(fitted_zoom)`
/// (see `show_source_scene`), because the fitted scale has to stay reachable,
/// and a console large enough to fit the Grid above `MAX_ZOOM` widens it: a
/// 2560-point-wide window on the default Grid fits at about 2.25 and offers
/// seventeen steps, and one twice that wide fits at about 4.5 and offers
/// thirty-five — some 13,000 rasters, which would pass the fill ratio. That is
/// a sweep across the whole of a very large console's range, not a zoom a
/// viewer holds, and the cost of passing it is a re-rasterisation rather than a
/// fault.
///
/// The step costs a Glyph at most an eighth of the Source's Cell scale in size,
/// taken downwards so a Glyph is never larger than its share of the Cell — see
/// [`glyph_scale`], which states why the rounding goes that way. It is still
/// strictly sharper than what it replaces: a Scene bilinearly resamples one
/// rasterised size at *every* zoom.
///
const GLYPH_SCALE_STEP: f32 = 0.125;

/// The height the top panel takes from the window, leaving the rest to the
/// console. It is the panel's own minimum, which the menu bar does not exceed.
const TOP_PANEL_HEIGHT: f32 = 32.0;

///
/// The window size that presents the default Grid at the Source's own Cell
/// size: the Source's own points, and the chrome above the console.
///
/// A console opened at this size fits the Grid at a scale of exactly one, so
/// the Grid fills it with no letterboxing and Glyphs are drawn at the size they
/// are rasterised at. Every other window size still presents the Grid — fitted,
/// centred, and letterboxed on the longer axis — so this is where the console
/// opens, not a shape it holds the viewer to.
///
pub const DEFAULT_VIEW_SIZE: [f32; 2] = [
    DEFAULT_COL_COUNT as f32 * CELL_SIZE,
    DEFAULT_ROW_COUNT as f32 * CELL_SIZE + TOP_PANEL_HEIGHT,
];

fn translate_event(event: Event) -> Option<InputEvent> {
    match event {
        Event::Key {
            key, pressed: true, ..
        } => match key {
            Key::ArrowDown => Some(InputEvent::KeyPressed(InputKey::ArrowDown)),
            Key::ArrowLeft => Some(InputEvent::KeyPressed(InputKey::ArrowLeft)),
            Key::ArrowRight => Some(InputEvent::KeyPressed(InputKey::ArrowRight)),
            Key::ArrowUp => Some(InputEvent::KeyPressed(InputKey::ArrowUp)),
            Key::Backspace => Some(InputEvent::KeyPressed(InputKey::Backspace)),
            Key::Delete => Some(InputEvent::KeyPressed(InputKey::Delete)),
            Key::Space => Some(InputEvent::KeyPressed(InputKey::Space)),
            _ => None,
        },
        Event::Text(text) => Some(InputEvent::Text(text)),
        // The running Orcvs models only input it acts on; all other toolkit
        // events remain presentation concerns and are dropped here.
        _ => None,
    }
}

fn source_bounds(columns: usize, rows: usize) -> Rect {
    Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(columns as f32, rows as f32) * CELL_SIZE,
    )
}

///
/// The scale and translation the Source is presented under, and whether the
/// viewer has moved it.
///
/// This is what the console holds instead of handing a region to an
/// `egui::Scene`. It carries a transform rather than a Scene-space rectangle
/// because a rectangle only describes a fit: it cannot say where the Source
/// sits once a viewer has panned to somewhere the fit never chose.
///
/// While the viewer has not panned or zoomed, the transform follows the fitted
/// square viewport, so every resize re-fits rather than cropping.
///
#[derive(Default)]
struct SourceView {
    to_global: TSTransform,
    adjusted: bool,
}

///
/// Whether `to_global` can be presented and inverted.
///
/// `egui::Scene::show` used to reset a transform that had gone bad
/// (`egui-0.36.1/src/containers/scene.rs:151-152, 168-173`) and nothing
/// replaces that once the container is gone. `grid_viewport` answers a Cell
/// size of zero for a console with no area, so the fit it yields has a scaling
/// of zero, and `TSTransform::inverse` divides by the scaling — which
/// `Scene::register_pan_and_zoom` does on every frame the pointer is over the
/// console. An unguarded zero therefore resolves every pointer position to NaN.
///
/// `TSTransform::is_valid` is not enough on its own: it checks only
/// `translation.x` (`emath-0.36.1/src/ts_transform.rs:55-57`) and admits a
/// negative scaling, which would present the Source mirrored.
///
fn is_presentable(to_global: TSTransform) -> bool {
    to_global.scaling.is_finite() && to_global.scaling > 0.0 && to_global.translation.is_finite()
}

///
/// The scale a [`FontId`] is derived from, quantised to [`GLYPH_SCALE_STEP`].
///
/// Never zero or negative: a font size of zero lays nothing out, and the
/// smallest step still draws something a viewer can see is there.
///
/// # Why the step is taken downwards
///
/// The step is absolute, so rounding to the nearest one is disproportionate at
/// a small scale: a console fitting at 0.2 would round up to 0.25 and lay an
/// 18 point Glyph out at 4.5 points inside a 5 point Cell, where the same Glyph
/// at the Source's own scale takes 18 of 25. Flooring keeps a Glyph's share of
/// its Cell at or under what the fit gave it at every scale, and costs at most
/// one step of sharpness rather than a Cell's worth of proportion. Both zoom
/// limits and the Source's own scale are exact multiples of the step, so
/// flooring leaves them exactly where rounding did, and the step count the
/// atlas budget above is stated over is unchanged.
///
fn glyph_scale(scaling: f32) -> f32 {
    if !scaling.is_finite() || scaling <= 0.0 {
        return GLYPH_SCALE_STEP;
    }

    ((scaling / GLYPH_SCALE_STEP).floor() * GLYPH_SCALE_STEP).max(GLYPH_SCALE_STEP)
}

#[derive(Default)]
struct TempoEdit {
    pending: Option<Bpm>,
}

impl TempoEdit {
    fn changed(&mut self, bpm: Bpm) {
        self.pending = Some(bpm);
    }

    fn take_commit(&mut self, pointer_down: bool) -> Option<Bpm> {
        if pointer_down {
            None
        } else {
            self.pending.take()
        }
    }
}

#[cfg(test)]
mod tempo_edit_tests {
    use super::TempoEdit;
    use orcvs::opts::Bpm;

    #[test]
    fn a_dragged_tempo_commits_after_release_even_if_the_menu_closed() {
        let mut edit = TempoEdit::default();
        edit.changed(Bpm::new(120).unwrap());

        assert_eq!(edit.take_commit(true), None);
        assert_eq!(edit.take_commit(false), Bpm::new(120));
    }
}

/// Console wraps the running Orcvs with egui presentation concerns.
///
pub struct Console {
    orcvs: Orcvs,
    /// Device discovery and selection for whatever MIDI backend `orcvs` has on
    /// this target. The console never asks what target it is on: a target with
    /// no native backend answers an empty destination list here, and
    /// `native_midi::AVAILABLE` says whether the menu presenting it exists.
    midi: MidiDeviceSelection<NativeMidiBackend>,
    font_family: egui::FontFamily,
    source_view: SourceView,
    diagnostics_open: bool,
    tempo_edit: TempoEdit,
    #[cfg(feature = "persistence")]
    persistence: crate::persistence::Persistence,
}

impl Console {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let style = style();
        cc.egui_ctx.set_style_of(egui::Theme::Dark, style);
        cc.egui_ctx.set_theme(egui::Theme::Dark);

        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();
        let key = "MonaspaceNeon";
        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            key.to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/MonaspaceNeon-Regular.otf"))
                .into(),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, key.to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, key.to_owned());

        cc.egui_ctx.set_fonts(fonts);

        // The stored Source revision when storage holds one, and the ordinary
        // default Grid otherwise. Every derived view is rebuilt from it.
        let start = starting_source(cc.storage);
        let orcvs = Orcvs::with_source(start.source);
        let mut midi = MidiDeviceSelection::new(orcvs.midi_selection_handle());
        midi.refresh_destinations();
        Self {
            orcvs,
            midi,
            font_family: FontId::monospace(DEFAULT_FONT_SIZE).family,
            source_view: SourceView::default(),
            diagnostics_open: false,
            tempo_edit: TempoEdit::default(),
            #[cfg(feature = "persistence")]
            persistence: start.persistence,
        }
    }
}

fn frames_per_second(frame_time: f32) -> Option<f32> {
    frame_time.is_normal().then(|| frame_time.recip())
}

fn show_diagnostics(
    ctx: &egui::Context,
    open: &mut bool,
    frame: &eframe::Frame,
    to_global: TSTransform,
    console: Rect,
    cell_size: f32,
) {
    let frame_time = ctx.input(|input| input.stable_dt);
    egui::Window::new("Diagnostics")
        .open(open)
        .default_width(360.0)
        .show(ctx, |ui| {
            egui::Grid::new("orcvs-diagnostics-summary")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("FPS");
                    ui.monospace(
                        frames_per_second(frame_time)
                            .map(|fps| format!("{fps:.1}"))
                            .unwrap_or_else(|| "—".to_owned()),
                    );
                    ui.end_row();

                    ui.label("Frame time");
                    ui.monospace(format!("{:.2} ms", frame_time * 1_000.0));
                    ui.end_row();

                    ui.label("CPU time");
                    ui.monospace(
                        frame
                            .info()
                            .cpu_usage
                            .map(|seconds| format!("{:.2} ms", seconds * 1_000.0))
                            .unwrap_or_else(|| "—".to_owned()),
                    );
                    ui.end_row();

                    ui.label("Cell size");
                    ui.monospace(format!("{cell_size:.1} pt"));
                    ui.end_row();

                    // The console owns the transform, so the zoom is a field of
                    // it rather than a ratio derived back out of a region.
                    ui.label("Source zoom");
                    ui.monospace(format!("{:.2}×", to_global.scaling));
                    ui.end_row();

                    ui.label("Visible Source region");
                    ui.monospace(format!("{:.1?}", to_global.inverse() * console));
                    ui.end_row();

                    ui.label("Pixels per point");
                    ui.monospace(format!("{:.2}", ctx.pixels_per_point()));
                    ui.end_row();
                });

            ui.separator();
            egui::CollapsingHeader::new("egui inspection")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(420.0)
                        .show(ui, |ui| ctx.inspection_ui(ui));
                });
        });
}

///
/// The alphabet the Glyph table covers: the printable ASCII a Source Cell can
/// show, less the space.
///
/// `orcvs::source::CellContent` accepts exactly `0x20..=0x7e`, and every
/// spelling `GlyphString` prints for an empty Cell is inside that range, so no
/// Cell of a Source can ask for a character outside it. The space is the one
/// printable character deliberately left out: a Cell showing one paints no
/// Glyph at all, and a Shape and a galley clone spent showing nothing is the
/// cost this drawing exists to stop paying.
///
const ALPHABET_FIRST: u8 = b'!';
const ALPHABET_LAST: u8 = b'~';

///
/// One laid-out Glyph per character of the alphabet, for the font and size the
/// Source is painted at.
///
/// The table has one owner and one construction site — [`show_source`] — so
/// nothing lays the alphabet out at a second size and thrashes it.
///
/// # Scratch for exactly one Render Frame
///
/// Retaining this table across Render Frames is unsound, not merely wasteful,
/// and that is why it is rebuilt every frame rather than memoised.
///
/// A galley's `RowVisuals::mesh` holds *texel* coordinates into the live font
/// atlas, normalised against that atlas's size at tessellation
/// (`epaint-0.36.1/src/text/text_layout_types.rs`, `tessellator.rs`), and
/// `Fonts::begin_pass` (`epaint-0.36.1/src/text/fonts.rs`) replaces the whole
/// `FontsImpl` — a fresh atlas with empty glyph caches — whenever the text
/// options change or the atlas passes its fill ratio. A galley held across that
/// recreate indexes unrelated texels and paints a *different character*. Atlas
/// *growth* is safe, because it only extends the image height and every texel
/// keeps its coordinates; *recreation* is what corrupts, and `font_image_size()`
/// cannot tell the two apart.
///
/// epaint's own `GalleyCache` is the memo, and it is the only cache in the
/// stack that `begin_pass` invalidates alongside the atlas. An
/// `egui::cache::FrameCache` or a `ctx.data()` entry evicts on last-frame use
/// and knows nothing about fonts, so either would carry exactly that
/// corruption. Do not "optimise" this into one.
///
/// Rebuilding costs one `Context` write lock for the whole table instead of one
/// per Cell, and one `String` per character of the alphabet instead of one per
/// Cell. Those are the wins; retaining the table is needed for none of them.
///
struct GlyphTable {
    /// The `Context` the alphabet was laid out through, for the one character
    /// the table cannot cover. Cloning a `Context` clones an `Arc`, and the
    /// table lives for one Render Frame, so this holds nothing open.
    ctx: egui::Context,
    /// The font the alphabet was laid out at, for the one character the table
    /// cannot cover.
    font: FontId,
    /// The alphabet, indexed by `byte - ALPHABET_FIRST`.
    characters: Vec<Arc<Galley>>,
}

impl GlyphTable {
    ///
    /// Lays the alphabet out for this Render Frame, inside a single
    /// `ctx.fonts_mut` closure.
    ///
    /// One galley per character, never one per row: egui 0.36 shapes through
    /// harfrust with `liga` and `calt` enabled and does not apply
    /// `extra_letter_spacing` within a shaping cluster, so a row laid out as one
    /// galley would let a ligature consume two Cells and shift the rest of the
    /// row. MonaspaceNeon has ligatures and Orcvs Source is full of the pairs
    /// that trigger them.
    ///
    fn lay_out(ctx: &egui::Context, font: FontId) -> Self {
        let characters = ctx.fonts_mut(|fonts| {
            (ALPHABET_FIRST..=ALPHABET_LAST)
                .map(|byte| {
                    // Laid out with `Color32::PLACEHOLDER`, so one galley serves
                    // every Cell whatever colour that Cell's Glyph is painted
                    // in: the tessellator substitutes the fallback colour for
                    // placeholder vertices alone.
                    fonts.layout_delayed_color(
                        char::from(byte).to_string(),
                        font.clone(),
                        f32::INFINITY,
                    )
                })
                .collect()
        });

        Self {
            ctx: ctx.clone(),
            font,
            characters,
        }
    }

    /// The laid-out Glyph for `character`, or `None` for a character outside
    /// the alphabet — the space included.
    fn galley(&self, character: char) -> Option<&Arc<Galley>> {
        let byte = u8::try_from(character).ok()?;
        self.characters
            .get(usize::from(byte.checked_sub(ALPHABET_FIRST)?))
    }

    ///
    /// The laid-out Glyph for a character a Cell shows, laying out the one the
    /// alphabet does not cover.
    ///
    /// A Source Cell holds printable ASCII by construction, so the fallback
    /// lays out at most the odd galley for a Source that found a way to hold
    /// something else — and it costs that one Cell the allocation and the
    /// whole-`Context` lock the table exists to take once. It answers a galley
    /// rather than painting one, so that Cell stays a Shape in the ordered
    /// sequence rather than a `Painter::text` painted out of turn.
    ///
    fn glyph(&self, character: char) -> Arc<Galley> {
        match self.galley(character) {
            Some(galley) => galley.clone(),
            None => self.ctx.fonts_mut(|fonts| {
                fonts.layout_delayed_color(character.to_string(), self.font.clone(), f32::INFINITY)
            }),
        }
    }
}

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
/// function this calls — at `epaint-0.36.1/src/tessellator.rs:1829-1862`. A run
/// left unsnapped here would reach the screen as the same pixels.
///
/// It is snapped so the rectangle is one a test can predict. The snap is the
/// last thing that moves an edge, so doing it here puts the Shape a Render
/// Frame carries at the coordinates the paint lands on, and an assertion can
/// state them exactly rather than within a pixel. `Rect::round_to_pixels`
/// rounds the two corners independently
/// (`emath-0.36.1/src/gui_rounding.rs:155-186`), so a run's far edge lands on
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
/// not in the value layer — and naming the five groups states it where a
/// comment used to.
///
/// # Why they are built eagerly
///
/// Shape construction is never lazy. `Painter::extend` runs the iterator it is
/// given inside `ctx.graphics_mut`, a full `Context` write lock, so a Shape
/// built lazily is a Shape built while holding it. The fields are complete
/// before [`SourceShapes::into_shapes`] hands the first one out.
///
struct SourceShapes {
    /// The coalesced background runs, one rectangle each.
    backgrounds: Vec<Shape>,
    /// Every Cell's own border but the Cursor's.
    borders: Vec<Shape>,
    /// One galley per Cell that shows a character other than the space.
    glyphs: Vec<Shape>,
    /// The sector seams, left edge then top edge, Cell by Cell.
    seams: Vec<Shape>,
    /// The Cursor's own stroke, which is the selected Cell's border painted
    /// last.
    cursor: Vec<Shape>,
}

impl SourceShapes {
    ///
    /// Draws a Paint at `viewport`: the geometry the value layer carries none
    /// of, applied to the colours and characters it carries all of.
    ///
    /// `scale` is the presented Cell side over the Source's own, which the
    /// stroke widths take so the Grid lines and sector seams are one Source
    /// point wide at every zoom. `pixels_per_point` is the device scale the
    /// background runs are snapped to; see [`background_run`].
    ///
    fn new(
        paint: &Paint,
        viewport: &GridViewport,
        table: &GlyphTable,
        scale: f32,
        pixels_per_point: f32,
    ) -> Self {
        let grid = paint.grid();
        // A border is the rule and a Glyph is one on a written Grid, so both
        // are sized to the Grid up front: a densely written Source that regrew
        // either of them would pay the reallocation on every Render Frame. A
        // background is the exception — the Cursor's bloom reaches fifteen
        // Cells and the rest of the Grid asks for none — so that one starts
        // empty and grows to whatever the blink is asking for.
        let mut backgrounds = Vec::new();
        let mut borders = Vec::with_capacity(grid.count());
        let mut glyphs = Vec::with_capacity(grid.count());
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

        for position in grid.positions_by_row().flatten() {
            let cell = paint.at(position);
            let rect = viewport.cell_rect(position.x(), position.y());
            // The Cell's own border, stroke and no fill: a widened run paints
            // over the borders of every Cell inside it, so the fill and the
            // border cannot be one shape.
            let border = Shape::Rect(RectShape::stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(GRID_LINE_WIDTH * scale, cell.border),
                StrokeKind::Inside,
            ));

            // The selected Cell's border is the Cursor, and the Cursor is
            // painted last.
            if position == paint.cursor() {
                cursor.push(border);
            } else {
                borders.push(border);
            }

            // A seam is absent on the Cursor's Cell because the derive
            // suppressed it there, so this step never learns that rule.
            for (colour, ends) in [
                (cell.sector_left, [rect.left_top(), rect.left_bottom()]),
                (cell.sector_top, [rect.left_top(), rect.right_top()]),
            ] {
                if let Some(colour) = colour {
                    seams.push(Shape::line_segment(
                        ends,
                        Stroke::new(SECTOR_LINE_WIDTH * scale, colour),
                    ));
                }
            }

            if cell.character != ' ' {
                let galley = table.glyph(cell.character);
                // Centred in a Cell whose own corner is an exact multiple of
                // the Cell size.
                glyphs.push(Shape::galley(
                    rect.center() - galley.size() / 2.0,
                    galley,
                    cell.foreground,
                ));
            }
        }

        Self {
            backgrounds,
            borders,
            glyphs,
            seams,
            cursor,
        }
    }

    ///
    /// The five groups end to end, in paint order.
    ///
    /// An iterator over the owned `Vec`s rather than a sixth one: the chain
    /// costs nothing, and collecting it would spend a further allocation of
    /// some two thousand elements, and a whole re-move, per Render Frame.
    ///
    fn into_shapes(self) -> impl Iterator<Item = Shape> {
        self.backgrounds
            .into_iter()
            .chain(self.borders)
            .chain(self.glyphs)
            .chain(self.seams)
            .chain(self.cursor)
    }
}

///
/// Draws the Source Grid and answers the one question a click asks of it.
///
/// The whole Grid is one allocated rectangle and every Cell is painted, so no
/// Cell is a widget and the cost of a Render Frame is shapes rather than
/// interaction rects and widget ids.
///
/// A background is painted only where it differs from the Source fill the panel
/// is already filled with, and consecutive Cells in a row that want the same
/// background share one rectangle. The Cursor's bloom reaches fifteen Cells
/// across, so on the default Grid most Cells ask for no background at all and
/// the ones that do arrive in runs.
///
/// The click is answered rather than acted on. Selecting a Cell is the Source's
/// business and `Console::ui` owns the running Orcvs it is asked of; handing the
/// Position back is what leaves this function with nothing but a Render Frame
/// and a place to draw it.
///
fn show_source(
    ui: &mut egui::Ui,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    grid: GridViewport,
    clip: Rect,
) -> Option<Position> {
    // The shape the Render Frame was derived from, named apart from the
    // `GridViewport` the Cells are painted at.
    let source_grid = frame.grid();
    // One rectangle for the whole Grid, sensing clicks and nothing else.
    //
    // Within a layer a later-registered child wins the click tie, and would win
    // the drag too if it sensed drag. The pan rectangle `show_source_scene`
    // allocates is registered before this one, so sensing clicks alone takes
    // the clicks and leaves the middle-drag pan to it. `Sense::CLICK` rather
    // than `Sense::click()`, which is `CLICK | FOCUSABLE` and would put the
    // Grid in the tab order where a thousand Buttons never were.
    //
    // The rectangle is the Grid, not the console area. The letterboxing is the
    // only territory where the pan rectangle's own `double_clicked()` still
    // fires, and that double click is what hands a pinned view back to the fit.
    //
    // Clipped to the console, because `Ui::interact` bounds a widget by the
    // `Ui`'s clip rect rather than by the console area, and a zoomed-in Grid
    // reaches past the console on every side. `Scene::show` used to set that
    // clip rect itself (`scene.rs:209`); with the container gone the Grid
    // states its own bound.
    let response = ui.interact(
        grid.rect.intersect(clip),
        ui.id().with("source_grid"),
        Sense::CLICK,
    );
    // What `Scene::show` did with `set_clip_rect` and a sublayer, in the one
    // layer that is left: the Grid is clipped to the console area, so a zoomed
    // Grid cannot paint over the chrome around it. A sublayer is not needed
    // because the Source Grid is all this layer holds, so painting it directly
    // is the ordering `set_sublayer` used to arrange.
    let painter = ui.painter().with_clip_rect(clip);
    // The scale is already in the Cell size, and the Scene used to carry it to
    // the strokes as well, so the Grid lines and sector seams take it here
    // rather than staying one Source point wide at every zoom.
    let scale = grid.cell_size / CELL_SIZE;
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
        FontId::new(DEFAULT_FONT_SIZE * glyph_scale(scale), font_family.clone()),
    );

    // What the console decided to draw, then what draws it. The decision is a
    // value derived from the Render Frame alone, so what colour a Cell is can
    // be asked without a `Context`, a window or a running Orcvs.
    let paint = Paint::derive(frame);
    let shapes = SourceShapes::new(&paint, &grid, &table, scale, pixels_per_point);

    // One `Painter::extend`, never a `Painter::add` per Shape. `add` reaches
    // `Context::graphics_mut`, which is a full `Context` write lock, so a
    // per-Cell loop would take more locks than the Button field it replaces and
    // turn this change into a regression.
    painter.extend(shapes.into_shapes());

    // The click resolves by division through the viewport the Cells were
    // painted at. With no layer transform, `interact_pointer_pos` is in global
    // points, which is the space the presented Grid is in.
    if response.clicked()
        && let Some(pointer) = response.interact_pointer_pos()
        && let Some((column, row)) =
            grid.cell_at(pointer, source_grid.columns(), source_grid.rows())
        && let Some(cell) = frame.rows().get(row).and_then(|row| row.get(column))
    {
        Some(cell.position())
    } else {
        None
    }
}

///
/// The Source as it was presented for one Render Frame: the geometry it was
/// drawn under, and the Cell a click asked for.
///
/// "Presented" is already the repository's word for this step —
/// `grid_viewport::presented_grid` uses it.
///
struct PresentedSource {
    /// The viewport the Cells were drawn at.
    viewport: GridViewport,
    /// The Cell the viewer clicked, for the caller that owns the Source to
    /// select.
    clicked: Option<Position>,
}

///
/// Shows the Source in the largest square-Celled viewport the console area
/// holds, centred so the surplus is letterboxing, and answers the geometry it
/// was presented under along with the Cell a click asked for.
///
/// The console owns the scale and translation the Source is presented under —
/// `view.to_global` — and `grid_viewport::presented_grid` is the one place that
/// scale is applied, so a Cell's two axes still cannot part company: one
/// `scaling` serves both. Every Cell, and so every click that lands on one,
/// goes through that one arithmetic. Nothing here sets a layer transform, which
/// is the point: a transformed layer reaches every `TextShape` in it through
/// `Arc::make_mut` at end of pass, and a cached galley's refcount is never one.
/// See `docs/adr/0038-the-console-owns-the-source-grid-transform.md`.
///
/// The pan and zoom *input* handling is still `egui::Scene`'s:
/// `Scene::register_pan_and_zoom` is public, takes the `&mut TSTransform` its
/// caller owns, and touches no layer. Only its drag-pan branch has to be
/// replaced, and only because that branch corrects for a division that happens
/// nowhere but inside a transformed layer.
///
fn show_source_scene(
    ui: &mut egui::Ui,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    view: &mut SourceView,
) -> PresentedSource {
    // The shape the Render Frame was derived from, named apart from the
    // `GridViewport` this function goes on to present it at.
    let source_grid = frame.grid();
    let source = source_bounds(source_grid.columns(), source_grid.rows());
    // The whole console area, sensing clicks and drags, allocated before any
    // Cell rectangle so the Grid's own click rectangle registers after it. This
    // is also what `Scene::show` reached `force_set_min_rect` for: the space
    // the Source is presented in is claimed from the parent layout whether the
    // Grid fills it or letterboxes inside it.
    let (console, mut pan) =
        ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::click_and_drag());
    let viewport = grid_viewport(console, source_grid.columns(), source_grid.rows());
    let fitted = viewport.fit_transform(source);

    if !view.adjusted {
        view.to_global = fitted;
    }
    if !is_presentable(view.to_global) {
        // A console with no area has no fit to reach either, and the identity
        // is the one transform that is always invertible. The Grid it presents
        // is clipped away to nothing, which is what a console with no area
        // shows regardless.
        view.to_global = if is_presentable(fitted) {
            fitted
        } else {
            TSTransform::IDENTITY
        };
    }

    // The fitted scale has to be reachable, or the clamp inside
    // `register_pan_and_zoom` pulls the Grid off the console. A console smaller
    // than the viewer's zoom limits allows fits it out on either side, so both
    // ends give. A console with no area answers a scale of zero, which is no
    // fit to reach.
    let fitted_zoom = viewport.scale(source);
    let min_zoom = if fitted_zoom > 0.0 {
        MIN_ZOOM.min(fitted_zoom)
    } else {
        MIN_ZOOM
    };
    let pan_and_zoom = egui::Scene::new()
        .zoom_range(min_zoom..=MAX_ZOOM.max(fitted_zoom))
        // The helper's own drag-pan branch is dead here, and deliberately.
        // It computes `to_global.translation += to_global.scaling *
        // resp.drag_delta()` (`scene.rs:239`), and `Response::drag_delta`
        // divides by the layer transform's scaling *only when the layer has
        // one* (`response.rs:452-465`). Inside `Scene::show` the two cancel and
        // the pan is 1:1 with the pointer. With the transform owned here there
        // is no layer transform, nothing divides, and the multiply would
        // over-pan by the zoom factor — invisibly at the fitted scale of one,
        // which is exactly where a test would be looking.
        .drag_pan_buttons(DragPanButtons::empty());

    // Where the view sits before any gesture reaches it, so the pin below can
    // ask whether one moved it.
    let before_the_gesture = view.to_global;

    if pan.dragged_by(PointerButton::Middle) {
        // The pointer moved this far in presented points, and the translation
        // is in presented points, so it is added and not scaled.
        view.to_global.translation += pan.drag_delta();
        pan.mark_changed();
    }
    // Zoom at the pointer, the smooth-scroll pan and the `zoom_range` clamp are
    // kept rather than reimplemented: all three are layer-independent.
    pan_and_zoom.register_pan_and_zoom(ui, &mut pan, &mut view.to_global);

    let grid = presented_grid(
        view.to_global,
        source,
        source_grid.columns(),
        source_grid.rows(),
        ui.ctx().pixels_per_point(),
    );
    let clicked = show_source(ui, frame, font_family, grid, console);

    // Panning or zooming moves the view off the fitted viewport and holds it
    // there; a double click on the letterboxing hands it back. A frame that
    // does both is a reset: the double click is the later intent.
    //
    // Only on the letterboxing. The Grid's own click rectangle is registered
    // after this one and wins every tie inside the Grid, so a double click on
    // a Cell selects it and leaves the view pinned. That is what the field of
    // Cell Buttons did before the Grid was painted, and it means the gesture
    // is unreachable at a window the Grid fills exactly — `DEFAULT_VIEW_SIZE`
    // included, where the fit is 1.0 and there is no letterboxing to hit. A
    // viewer pinned there zooms back out rather than double clicking. Giving
    // the reset a gesture that does not depend on surplus area is a change to
    // what the console offers, not to how it draws, so it is not made here.
    //
    // The pin asks the transform whether it moved rather than asking the
    // `Response` whether it changed. `register_pan_and_zoom` calls
    // `mark_changed` whenever a zoom or scroll event arrived at all, whether or
    // not the `zoom_range` clamp left `to_global` exactly where it was
    // (`scene.rs:265-274`). A console already sitting at either end of its zoom
    // range therefore reports a change for a gesture the clamp reverted, and
    // pinning on that costs the viewer every later re-fit: the owned transform
    // is absolute, and unlike the Scene-space rectangle it replaces it does not
    // track the window across a resize.
    if pan.double_clicked() {
        view.adjusted = false;
    } else if view.to_global != before_the_gesture {
        view.adjusted = true;
    }

    PresentedSource {
        viewport: grid,
        clicked,
    }
}

///
/// The frame the Source Grid is painted on.
///
/// The fill is load-bearing rather than decorative. `show_source` omits a
/// Cell's background wherever `cell_visuals` asks for `PALETTE.source`, on the
/// grounds that this frame has already painted exactly that colour across the
/// whole console and clips every Shape to it. An ordinary Cell therefore has no
/// rectangle of its own, and on the default Grid — where the Cursor's bloom
/// reaches fifteen Cells — most Cells are ordinary.
///
/// It is a function rather than a literal at the panel so the painting tests
/// render on the same ground production does, and so
/// `the_omitted_background_is_the_colour_the_panel_is_filled_with` has one
/// value to pin instead of a comment to trust.
///
fn source_panel_frame() -> egui::Frame {
    egui::Frame::new().fill(PALETTE.source)
}

impl eframe::App for Console {
    ///
    /// Called by the framework to save state before shutdown, and at
    /// intervals while running. The Source is the one persistence root, so
    /// this stores the current revision and nothing of the Console around it.
    ///
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.persistence.save(storage, self.orcvs.source());
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, root: &mut egui::Ui, eframe: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let playback_diagnostics = self.orcvs.observe_playback();
        if native_midi::AVAILABLE {
            self.midi.observe_diagnostics(playback_diagnostics);
        } else {
            // Without a native backend there is no MIDI menu, so the status
            // line those diagnostics would reach is never presented and the
            // developer console is the only channel a failure has.
            crate::diagnostics::report_playback_failures(&playback_diagnostics);
        }
        let top_panel = egui::Panel::top("top_panel")
            .resizable(true)
            .min_size(TOP_PANEL_HEIGHT);

        // let _bottom_panel = egui::TopBottomPanel::bottom("bottom_panel")
        //     .resizable(false)
        //     .min_height(0.0);

        top_panel.show(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                // The menu presents a choice of destination, so it exists only
                // where a backend can have one. Which targets those are is
                // `orcvs`'s answer, not a condition restated here.
                if native_midi::AVAILABLE {
                    ui.menu_button("MIDI", |ui| {
                        if ui.button("Refresh destinations").clicked() {
                            self.midi.refresh_destinations();
                        }
                        let selected = self.midi.selected_destination_id();
                        for destination in self.midi.destinations().to_vec() {
                            let is_selected = selected.as_ref() == Some(&destination.id);
                            if ui.selectable_label(is_selected, destination.name).clicked() {
                                self.midi.select_destination(&destination.id);
                            }
                        }
                        if self.midi.destinations().is_empty() {
                            ui.label("No MIDI destinations found");
                        }
                        if let Some(status) = self.midi.status() {
                            ui.separator();
                            ui.colored_label(ui.visuals().error_fg_color, status);
                        }
                    });
                }
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.diagnostics_open, "Diagnostics");
                });
                // Presented in the menu bar rather than the Diagnostics window,
                // which opens on a viewer's request and reports the running
                // frame. This is a start-up answer about the Source in front of
                // them, and it stays until they dismiss it. `report` reaches a
                // developer console; this is the channel a viewer reads.
                #[cfg(feature = "persistence")]
                if self.persistence.notice_visible() {
                    ui.add_space(16.0);
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        format!(
                            "Stored Source could not be read back; it was kept under \
                             \"{}\"",
                            crate::persistence::REFUSED_KEY
                        ),
                    );
                    if ui.button("Dismiss").clicked() {
                        self.persistence.dismiss_notice();
                    }
                }
                ui.menu_button("Tempo", |ui| {
                    let mut beats_per_minute = self.orcvs.bpm().beats_per_minute();
                    let tempo_response = ui.add(
                        egui::DragValue::new(&mut beats_per_minute)
                            .range(1..=999)
                            .suffix(" BPM"),
                    );
                    if tempo_response.changed() {
                        self.tempo_edit.changed(
                            Bpm::new(beats_per_minute)
                                .expect("the tempo control has a positive range"),
                        );
                    }
                });
                // ui.label(format!("HELLO"));
                // egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        if let Some(bpm) = self
            .tempo_edit
            .take_commit(ctx.input(|input| input.pointer.primary_down()))
        {
            self.orcvs.set_bpm(bpm);
        }

        let event_filter = EventFilter {
            tab: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            escape: true,
        };

        let events = ctx.input(|i| {
            i.filtered_events(&event_filter)
                .into_iter()
                .filter_map(translate_event)
                .collect()
        });
        self.orcvs.event_handler(events);
        self.orcvs.advance_cursor_blink();
        let frame = self.orcvs.render_frame();

        let mut console_area = Rect::ZERO;
        let mut cell_size = 0.0;
        egui::CentralPanel::default()
            .frame(source_panel_frame())
            .show(root, |ui| {
                console_area = ui.available_rect_before_wrap();
                let Console {
                    orcvs,
                    midi: _,
                    font_family,
                    source_view,
                    diagnostics_open: _,
                    tempo_edit: _,
                    #[cfg(feature = "persistence")]
                        persistence: _,
                } = self;
                let presented = show_source_scene(ui, &frame, font_family, source_view);
                cell_size = presented.viewport.cell_size;
                // The Source Grid answers which Cell was clicked; moving the
                // Cursor there is the Source's own business, and this is where
                // the running Orcvs is owned.
                if let Some(position) = presented.clicked {
                    orcvs.select(position);
                }

                ctx.request_repaint_after(self.orcvs.remaining_cursor_blink_delay());
            });

        if self.diagnostics_open {
            show_diagnostics(
                &ctx,
                &mut self.diagnostics_open,
                eframe,
                self.source_view.to_global,
                console_area,
                cell_size,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::{
        Event, Key, Modifiers, Pos2, Rect, Shape, Vec2, emath::GuiRounding as _, emath::TSTransform,
    };
    use orcvs::app::{InputEvent, InputKey, Orcvs};
    use orcvs::glyph::Glyph;
    use orcvs::render_frame::CursorBloom;

    use crate::grid_viewport::{GridViewport, grid_viewport, presented_grid};
    use crate::paint::Paint;
    use crate::style::PALETTE;
    use orcvs::grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT};

    use super::{
        ALPHABET_FIRST, ALPHABET_LAST, CELL_SIZE, Console, DEFAULT_FONT_SIZE, DEFAULT_VIEW_SIZE,
        GLYPH_SCALE_STEP, GRID_LINE_WIDTH, GlyphTable, MAX_ZOOM, MIN_ZOOM, SECTOR_LINE_WIDTH,
        SourceShapes, SourceView, TOP_PANEL_HEIGHT, frames_per_second, glyph_scale, is_presentable,
        show_source_scene, source_bounds, source_panel_frame, translate_event,
    };

    fn key_event(key: Key, pressed: bool) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn toolkit_events_translate_only_the_input_orcvs_handles() {
        let cases = [
            (Key::ArrowDown, InputKey::ArrowDown),
            (Key::ArrowLeft, InputKey::ArrowLeft),
            (Key::ArrowRight, InputKey::ArrowRight),
            (Key::ArrowUp, InputKey::ArrowUp),
            (Key::Backspace, InputKey::Backspace),
            (Key::Delete, InputKey::Delete),
            (Key::Space, InputKey::Space),
        ];
        for (egui_key, orcvs_key) in cases {
            assert_eq!(
                translate_event(key_event(egui_key, true)),
                Some(InputEvent::KeyPressed(orcvs_key))
            );
        }

        assert_eq!(
            translate_event(Event::Text("x".to_owned())),
            Some(InputEvent::Text("x".to_owned()))
        );
        assert_eq!(translate_event(key_event(Key::Enter, true)), None);
        assert_eq!(translate_event(key_event(Key::ArrowDown, false)), None);
        assert_eq!(translate_event(Event::Copy), None);
    }

    ///
    /// The frame rate is still derived; the Source zoom no longer is. Under an
    /// owned transform the zoom the diagnostics show *is* `scaling`, so what
    /// used to be a ratio recovered from a Scene-space region collapsed to a
    /// field read and `scene_zoom` went with it. What is left to assert is that
    /// the field the diagnostics read is never a value they cannot show: the
    /// guard is what makes the read safe.
    ///
    #[test]
    fn diagnostics_derive_frame_rate_and_read_the_source_zoom_from_the_owned_transform() {
        assert_eq!(frames_per_second(0.02), Some(50.0));
        assert_eq!(frames_per_second(0.0), None);

        let zoomed = TSTransform::new(Vec2::new(11.0, 7.0), 2.0);
        assert!(is_presentable(zoomed));
        assert_eq!(zoomed.scaling, 2.0);
        // The visible Source region the diagnostics show is the console area
        // read back through the transform, which is what the Scene-space
        // rectangle used to hold directly.
        assert_eq!(
            zoomed.inverse() * Rect::from_min_size(Pos2::new(11.0, 7.0), Vec2::new(800.0, 400.0)),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))
        );

        for unpresentable in [
            TSTransform::from_scaling(0.0),
            TSTransform::from_scaling(-1.0),
            TSTransform::from_scaling(f32::NAN),
            TSTransform::from_translation(Vec2::new(0.0, f32::NAN)),
        ] {
            assert!(
                !is_presentable(unpresentable),
                "{unpresentable:?} reached the diagnostics"
            );
        }
    }

    ///
    /// The step the glyph scale is quantised to, and the budget it implies.
    /// Both zoom limits and the Source's own scale land on a step, so the
    /// default window lays its Glyphs out at exactly the Source's font size.
    ///
    #[test]
    fn the_glyph_scale_is_quantised_to_a_stated_step() {
        assert_eq!(glyph_scale(1.0), 1.0);
        assert_eq!(glyph_scale(MIN_ZOOM), MIN_ZOOM);
        assert_eq!(glyph_scale(MAX_ZOOM), MAX_ZOOM);
        assert_eq!(glyph_scale(1.01), 1.0, "a nudge re-laid the whole alphabet");
        // Downwards, so the Glyph keeps its share of the Cell: a zoom part way
        // into a step is laid out at the step it is past, not the one it is
        // approaching.
        assert_eq!(glyph_scale(1.1), 1.0);
        assert_eq!(glyph_scale(1.13), 1.125);
        // Never zero, never negative, whatever reaches it.
        for degenerate in [0.0, -1.0, f32::NAN, f32::INFINITY, 1e-9] {
            assert!(
                glyph_scale(degenerate) >= GLYPH_SCALE_STEP,
                "{degenerate} laid out at a font size of {}",
                glyph_scale(degenerate)
            );
        }

        // Fifteen distinct sizes over the whole zoom range is the atlas budget
        // `GLYPH_SCALE_STEP` states. Swept in exact thousandths rather than by
        // accumulating one: the top of the range is reached by the
        // `zoom_range` clamp exactly, and a sum that drifts past it would drop
        // the step it lands on.
        let mut sizes: Vec<f32> = Vec::new();
        for thousandth in (MIN_ZOOM * 1_000.0) as u32..=(MAX_ZOOM * 1_000.0) as u32 {
            let scale = glyph_scale(thousandth as f32 / 1_000.0);
            if !sizes.iter().any(|held| (held - scale).abs() < 1e-6) {
                sizes.push(scale);
            }
        }
        assert_eq!(
            sizes.len(),
            15,
            "the zoom range holds {} sizes",
            sizes.len()
        );
    }

    ///
    /// A Glyph is never laid out at a larger fraction of its Cell than the fit
    /// gave it.
    ///
    /// The quantisation step is an absolute one, so rounding to the nearest
    /// step is disproportionate at a small scale: a console fitting at 0.2
    /// rounds up to 0.25 and lays an 18 point Glyph out at 4.5 points inside a
    /// 5 point Cell, where the same Glyph at the Source's own scale takes 18 of
    /// 25. Under the retired Scene the layer scaled the Glyph exactly, so this
    /// is the proportion the effort's strict-parity rule is about. Quantising
    /// downwards keeps it and costs at most one step of sharpness.
    ///
    /// The floor at [`GLYPH_SCALE_STEP`] is the one deliberate exception, and
    /// the case above it is what this pins.
    ///
    #[test]
    fn a_glyph_is_never_laid_out_larger_than_the_scale_it_is_drawn_at() {
        // A sweep at half the step, so it lands both on steps and between them.
        let mut scaling = GLYPH_SCALE_STEP;
        while scaling <= MAX_ZOOM {
            assert!(
                glyph_scale(scaling) <= scaling,
                "a Glyph at {scaling} was laid out at {}",
                glyph_scale(scaling)
            );
            scaling += GLYPH_SCALE_STEP / 2.0;
        }
        // The fit below the zoom floor is where the rounding was worst.
        assert_eq!(glyph_scale(0.2), 0.125);
    }

    ///
    /// One Render Frame, and everything it painted in paint order.
    ///
    /// The shapes are flattened: what an assertion is about is the order the
    /// Source Grid was painted in, not how deeply a `Shape::Vec` wrapped it.
    ///
    fn console_pass(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) -> (GridViewport, Vec<Shape>) {
        console_pass_at(ctx, screen, events, orcvs, view, 1.0)
    }

    ///
    /// The same pass at a stated device scale.
    ///
    /// The scale is given as the viewport's `native_pixels_per_point` rather
    /// than through `Context::set_pixels_per_point`, which sets the zoom factor
    /// instead and rewrites the next pass's `screen_rect` from the previous
    /// one's to avoid jitter (`egui-0.36.1/src/context.rs:436-446`) — so the
    /// console would not be the size the caller asked for.
    ///
    fn console_pass_at(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
        pixels_per_point: f32,
    ) -> (GridViewport, Vec<Shape>) {
        let frame = orcvs.render_frame();
        let mut presented = None;
        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .expect("the root viewport")
            .native_pixels_per_point = Some(pixels_per_point);
        let output = ctx.run_ui(input, |root| {
            egui::CentralPanel::default()
                .frame(source_panel_frame())
                .show(root, |ui| {
                    presented = Some(show_source_scene(
                        ui,
                        &frame,
                        &egui::FontFamily::Monospace,
                        view,
                    ));
                });
        });
        let mut painted = Vec::new();
        for clipped in &output.shapes {
            flatten(clipped.shape.clone(), &mut painted);
        }
        output.drop_without_applying_deltas();
        let presented = presented.expect("the central panel showed the Source");
        // What `Console::ui` does with the answer, done here for the same
        // reason: the Source Grid reports the Cell and the caller that owns the
        // running Orcvs moves the Cursor to it.
        if let Some(position) = presented.clicked {
            orcvs.select(position);
        }

        (presented.viewport, painted)
    }

    fn flatten(shape: Shape, into: &mut Vec<Shape>) {
        match shape {
            Shape::Vec(shapes) => {
                for shape in shapes {
                    flatten(shape, into);
                }
            }
            shape => into.push(shape),
        }
    }

    fn console_frame(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) -> GridViewport {
        console_pass(ctx, screen, events, orcvs, view).0
    }

    fn click_at(point: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(point),
            Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    fn release_at(point: Pos2) -> Vec<Event> {
        vec![Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }]
    }

    fn click(
        ctx: &egui::Context,
        screen: Rect,
        point: Pos2,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) {
        console_frame(ctx, screen, click_at(point), orcvs, view);
        console_frame(ctx, screen, release_at(point), orcvs, view);
    }

    ///
    /// The middle button pressed at `point`, which is the button that pans.
    ///
    fn middle_press_at(point: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(point),
            Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Middle,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    ///
    /// A pinch zoom over `point`. Zoom is not smoothed over later frames the
    /// way a wheel scroll is, so the frames after it are quiet.
    ///
    fn zoom_at(point: Pos2) -> Vec<Event> {
        vec![Event::PointerMoved(point), Event::Zoom(1.2)]
    }

    fn double_click(
        ctx: &egui::Context,
        screen: Rect,
        point: Pos2,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) {
        click(ctx, screen, point, orcvs, view);
        click(ctx, screen, point, orcvs, view);
    }

    ///
    /// A point in the surplus the viewport does not cover, if there is any.
    ///
    fn letterboxing(screen: Rect, viewport: Rect) -> Option<Pos2> {
        if screen.width() > viewport.width() + 1.0 {
            Some(Pos2::new(
                (screen.left() + viewport.left()) / 2.0,
                screen.center().y,
            ))
        } else if screen.height() > viewport.height() + 1.0 {
            Some(Pos2::new(
                screen.center().x,
                (screen.top() + viewport.top()) / 2.0,
            ))
        } else {
            None
        }
    }

    fn selected_cell(orcvs: &Orcvs) -> (usize, usize) {
        let frame = orcvs.render_frame();
        let cell = frame
            .rows()
            .iter()
            .flatten()
            .find(|cell| cell.selected())
            .expect("the Cursor is on a Cell");

        (cell.position().x(), cell.position().y())
    }

    #[test]
    fn a_click_selects_the_cell_under_the_pointer_in_a_letterboxed_console() {
        // The last shape fits the Source at a scale above MAX_ZOOM, where a
        // Scene whose zoom range excluded the fitted scale would clamp it and
        // put the Cells somewhere else.
        for screen_size in [
            Vec2::new(400.0, 200.0),
            Vec2::new(200.0, 400.0),
            Vec2::new(600.0, 300.0),
        ] {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
            let mut orcvs = Orcvs::new(4, 4);
            let mut view = SourceView::default();

            let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            assert_eq!(selected_cell(&orcvs), (0, 0));

            let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
            click(&ctx, screen, target, &mut orcvs, &mut view);

            assert_eq!(
                selected_cell(&orcvs),
                (3, 1),
                "a click at {target:?} in a {screen_size:?} console"
            );
        }
    }

    ///
    /// The other end of the same clamp: a console too small for the Source fits
    /// it at a scale below MIN_ZOOM, where a Scene whose zoom range excluded the
    /// fitted scale would clamp it up and spill the Grid out of the console.
    ///
    #[test]
    fn a_click_selects_the_cell_under_the_pointer_in_a_console_smaller_than_the_zoom_floor() {
        // A 32 by 32 Source is 800 points wide, so these shapes fit it at 0.2:
        // below the 0.25 floor.
        for screen_size in [Vec2::new(400.0, 160.0), Vec2::new(160.0, 400.0)] {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
            let mut orcvs = Orcvs::new(32, 32);
            let mut view = SourceView::default();

            let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            assert!(
                viewport.rect.width() <= screen.width() + 1e-3
                    && viewport.rect.height() <= screen.height() + 1e-3,
                "the viewport {viewport:?} left the {screen_size:?} console"
            );

            let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
            click(&ctx, screen, target, &mut orcvs, &mut view);

            assert_eq!(
                selected_cell(&orcvs),
                (3, 1),
                "a click at {target:?} in a {screen_size:?} console"
            );
        }
    }

    ///
    /// One pass of the whole running Console, the way eframe drives it.
    ///
    /// `eframe::Frame::_new_kittest` and `CreationContext::_new_kittest` are
    /// eframe's own headless constructors, which is how an `App` runs outside a
    /// window; `storage_tests` reaches for the same pair for the same reason.
    ///
    fn app_pass(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        console: &mut Console,
        host: &mut eframe::Frame,
    ) {
        use eframe::App as _;

        let input = egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };
        let output = ctx.run_ui(input, |root| console.ui(root, host));
        output.drop_without_applying_deltas();
    }

    ///
    /// Where a running Console presented its Source Grid, asked of the
    /// transform the pass left behind.
    ///
    /// The same three calls `show_source_scene` makes, and for the reason the
    /// `presented` helper cannot serve here: that one fits the Grid to a console
    /// area, and a running Console's console area is the screen less whatever
    /// height the menu bar settled the top panel at. The stored transform
    /// already carries that fit, so this asks it rather than re-deriving it.
    ///
    fn console_viewport(ctx: &egui::Context, console: &Console) -> GridViewport {
        let grid = console.orcvs.render_frame().grid();

        presented_grid(
            console.source_view.to_global,
            source_bounds(grid.columns(), grid.rows()),
            grid.columns(),
            grid.rows(),
            ctx.pixels_per_point(),
        )
    }

    ///
    /// Clicking a Cell of a running Console moves the Cursor onto it.
    ///
    /// Every other click test goes through `console_pass_at`, which shows the
    /// Source itself and applies the answered Position itself — so none of them
    /// reaches the `orcvs.select` call `Console::ui` owns, and all of them pass
    /// with that call deleted. `show_source` answers the Cell rather than
    /// selecting it, which is what makes clicking a Cell a wiring question at
    /// all, and this is the one test that asks it: a real `Console`, driven
    /// through `eframe::App::ui` over an egui pass, with nothing between the
    /// pointer and the Cursor but the console's own code.
    ///
    /// `storage_tests` exists for the same reason one module below, and says
    /// so: an interface-level test cannot see whether the Console is wired to
    /// the interface at all.
    ///
    #[test]
    fn a_click_on_a_cell_moves_the_cursor_of_a_running_console() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
        let mut host = eframe::Frame::_new_kittest();

        // One quiet pass, so the top panel has claimed its height and the view
        // holds the fit the Grid was presented under.
        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        assert_eq!(
            selected_cell(&console.orcvs),
            (0, 0),
            "a fresh Console did not open with the Cursor in the corner"
        );

        let viewport = console_viewport(&ctx, &console);
        let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
        // Pressed on one pass and released on the next, because a click is
        // reported on the release.
        app_pass(&ctx, screen, click_at(target), &mut console, &mut host);
        app_pass(&ctx, screen, release_at(target), &mut console, &mut host);

        assert_eq!(
            selected_cell(&console.orcvs),
            (3, 1),
            "a click at {target:?} on a Grid presented at {:?}",
            viewport.rect
        );
    }

    #[test]
    fn the_grid_fills_the_centred_viewport_and_the_letterboxing_holds_no_cell() {
        for screen_size in [
            Vec2::new(400.0, 200.0),
            Vec2::new(200.0, 400.0),
            Vec2::new(300.0, 300.0),
        ] {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
            let mut orcvs = Orcvs::new(8, 8);
            let mut view = SourceView::default();

            let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            let half_cell = Vec2::splat(viewport.cell_size / 2.0);

            // The far corner Cell of the Grid sits in the far corner of the
            // viewport, so the Grid fills it rather than a part of it.
            click(
                &ctx,
                screen,
                viewport.rect.max - half_cell,
                &mut orcvs,
                &mut view,
            );
            assert_eq!(
                selected_cell(&orcvs),
                (7, 7),
                "the last Cell of a {screen_size:?} console"
            );

            // The surplus is letterboxing rather than stretched Cells, so a
            // click there selects nothing and the Cursor stays where it was.
            if let Some(surplus) = letterboxing(screen, viewport.rect) {
                click(&ctx, screen, surplus, &mut orcvs, &mut view);
                assert_eq!(
                    selected_cell(&orcvs),
                    (7, 7),
                    "a click on the letterboxing of a {screen_size:?} console"
                );
            }

            click(
                &ctx,
                screen,
                viewport.rect.min + half_cell,
                &mut orcvs,
                &mut view,
            );
            assert_eq!(
                selected_cell(&orcvs),
                (0, 0),
                "the first Cell of a {screen_size:?} console"
            );
        }
    }

    ///
    /// The console opens on the whole Grid at the Source's own Cell size, so
    /// the default window spends every point it has on Cells and none on
    /// letterboxing, and no Glyph is resampled to be shown.
    ///
    #[test]
    fn the_default_window_presents_the_default_grid_at_its_own_scale() {
        let ctx = egui::Context::default();
        let console = Vec2::new(
            DEFAULT_VIEW_SIZE[0],
            DEFAULT_VIEW_SIZE[1] - TOP_PANEL_HEIGHT,
        );
        let screen = Rect::from_min_size(Pos2::ZERO, console);
        let mut orcvs = Orcvs::new(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        assert_eq!(
            viewport.cell_size, CELL_SIZE,
            "the default console fits the Grid at a scale other than one"
        );
        assert_eq!(
            viewport.rect, screen,
            "the default console letterboxes the Grid it was sized for"
        );
    }

    ///
    /// The default window size holds back exactly the height the top panel
    /// takes, so the rest reaches the console. The menu bar is rebuilt here
    /// rather than shared, so this also asserts that no menu makes the panel
    /// taller than its minimum.
    ///
    #[test]
    fn the_top_panel_takes_the_height_the_default_window_holds_back() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Vec2::ZERO;

        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |root| {
                egui::Panel::top("top_panel")
                    .resizable(true)
                    .min_size(TOP_PANEL_HEIGHT)
                    .show(root, |ui| {
                        egui::MenuBar::new().ui(ui, |ui| {
                            ui.menu_button("File", |_ui| {});
                            ui.add_space(16.0);
                            ui.menu_button("MIDI", |_ui| {});
                            ui.menu_button("View", |_ui| {});
                            ui.menu_button("Tempo", |_ui| {});
                        });
                    });
                egui::CentralPanel::default()
                    .frame(source_panel_frame())
                    .show(root, |ui| {
                        console = ui.available_size_before_wrap();
                    });
            },
        );
        output.drop_without_applying_deltas();

        assert_eq!(
            console,
            Vec2::new(
                DEFAULT_VIEW_SIZE[0],
                DEFAULT_VIEW_SIZE[1] - TOP_PANEL_HEIGHT
            )
        );
    }

    #[test]
    fn source_bounds_are_available_before_the_first_render() {
        let orcvs = Orcvs::new(32, 16);
        let source_grid = orcvs.render_frame().grid();
        let bounds = source_bounds(source_grid.columns(), source_grid.rows());

        assert_eq!(
            bounds,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 400.0))
        );
    }

    ///
    /// The viewport `show_source_scene` presents a Grid of this shape at, in a
    /// console of this size, before any gesture has moved the view.
    ///
    /// The same three calls that function makes, so nothing about the geometry
    /// is restated here: the fit is `GridViewport::fit_transform`'s and the
    /// presented Cell side is `presented_grid`'s, both asserted in
    /// `grid_viewport.rs`.
    ///
    fn presented(screen: Rect, columns: usize, rows: usize, pixels_per_point: f32) -> GridViewport {
        let source = source_bounds(columns, rows);
        let viewport = grid_viewport(screen, columns, rows);

        presented_grid(
            viewport.fit_transform(source),
            source,
            columns,
            rows,
            pixels_per_point,
        )
    }

    ///
    /// What a Paint is drawn as at `viewport`, without a console pass.
    ///
    /// An `egui::Context` is built here for one reason: a galley needs a font
    /// atlas, and a Glyph is a galley. Nothing asserted through this reads a
    /// colour *decision* — which colour `cell_visuals` gives a Cell is
    /// `paint.rs`'s question and is answered there with no Context at all.
    /// What is asserted here is what the shape step itself adds: the geometry,
    /// the grouping and the stroke widths.
    ///
    fn source_shapes(paint: &Paint, viewport: GridViewport, pixels_per_point: f32) -> SourceShapes {
        let scale = viewport.cell_size / CELL_SIZE;
        let ctx = egui::Context::default();
        let mut shapes = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let table = GlyphTable::lay_out(
                ui.ctx(),
                egui::FontId::new(
                    DEFAULT_FONT_SIZE * glyph_scale(scale),
                    egui::FontFamily::Monospace,
                ),
            );
            shapes = Some(SourceShapes::new(
                paint,
                &viewport,
                &table,
                scale,
                pixels_per_point,
            ));
        });
        output.drop_without_applying_deltas();

        shapes.expect("the pass drew the Source")
    }

    /// The rectangle a `Shape::Rect` covers.
    fn rect_of(shape: &Shape) -> Rect {
        match shape {
            Shape::Rect(rect) => rect.rect,
            other => panic!("{other:?} is not a rectangle"),
        }
    }

    fn close(left: Rect, right: Rect) -> bool {
        (left.min - right.min).length() < 1e-3 && (left.max - right.max).length() < 1e-3
    }

    ///
    /// The Cursor's blink reaches what a Cell is painted *with* and never
    /// where it is painted.
    ///
    /// This is the property the retired `cell_line_width` test held over a
    /// shipped function that took the blink phase and ignored it. Under the
    /// painter the property is structural —
    /// `GridViewport::cell_rect` takes a Position and nothing else — so it is
    /// asserted here against the geometry that actually reached the Shapes.
    ///
    /// The Cursor's own blink phase cannot be driven from a console test: it
    /// turns on a wall-clock delay held inside `orcvs`, and a seam to set it
    /// would be a test-only input cut into shipped code. What is asserted
    /// instead is the whole of what that phase could have moved — every Cell,
    /// the Cursor's included, occupies exactly the rectangle its Position gives
    /// it, and the Cursor's own stroke is drawn on that same rectangle rather
    /// than beside it or around it.
    ///
    /// The Cell's rectangle is the one it is *stroked* at: a Cell is filled
    /// only where its background differs from the Source, and the Cells that
    /// are filled share their rectangles with their neighbours.
    ///
    #[test]
    fn the_cursor_reaches_the_paint_of_a_cell_and_never_its_geometry() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let orcvs = Orcvs::new(8, 8);
        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let viewport = presented(screen, 8, 8, 1.0);
        let shapes = source_shapes(&paint, viewport, 1.0);

        assert_eq!(paint.cursor(), frame.rows()[0][0].position());
        // Every Cell but the Cursor's is stroked with its own border, in row
        // order.
        let mut expected = Vec::new();
        for row in 0..8 {
            for column in 0..8 {
                if (column, row) != (0, 0) {
                    expected.push(viewport.cell_rect(column, row));
                }
            }
        }
        assert_eq!(
            shapes.borders.len(),
            expected.len(),
            "stroked {} Cell borders",
            shapes.borders.len()
        );
        for (index, rect) in expected.iter().enumerate() {
            assert!(
                close(rect_of(&shapes.borders[index]), *rect),
                "Cell {index} was stroked at {:?} rather than {rect:?}",
                rect_of(&shapes.borders[index])
            );
        }
        // And the Cursor's Cell is stroked once, by the Cursor, on that same
        // rectangle.
        assert_eq!(shapes.cursor.len(), 1, "the Cursor is one stroke");
        assert!(
            close(rect_of(&shapes.cursor[0]), viewport.cell_rect(0, 0)),
            "the Cursor was stroked at {:?} rather than {:?}",
            rect_of(&shapes.cursor[0]),
            viewport.cell_rect(0, 0)
        );
        // Every background lies on the Cell geometry too, rather than beside
        // it: a run starts and ends on a Cell edge.
        for background in &shapes.backgrounds {
            assert!(
                (rect_of(background).height() - viewport.cell_size).abs() < 1e-3,
                "a background was {} tall against a Cell of {}",
                rect_of(background).height(),
                viewport.cell_size
            );
        }
    }

    ///
    /// A Cell's background never paints over a Glyph, whichever Cell that Glyph
    /// belongs to, and the Cursor is painted over both.
    ///
    /// The groups are what makes that expressible. Every fill is in
    /// `backgrounds`, every Glyph in `glyphs` and the Cursor alone in `cursor`,
    /// so chaining the groups orders the *kinds* however the Cells interleave:
    /// a later Cell in the row order cannot erase an earlier Cell's Glyph, and
    /// no neighbour's fill or seam can reach the Cursor. That the five groups
    /// then arrive at the painter in that order is asserted by
    /// `the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them`.
    ///
    #[test]
    fn every_background_is_painted_before_every_glyph_and_the_cursor_after_both() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = Orcvs::new(8, 8);
        // A Glyph in the first Cell of the Grid, so every other Cell's
        // background is built after it and would paint over it if the Shapes
        // were emitted Cell by Cell.
        orcvs.write("1");
        orcvs.select(orcvs.render_frame().rows()[0][0].position());
        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let viewport = presented(screen, 8, 8, 1.0);
        let shapes = source_shapes(&paint, viewport, 1.0);

        assert!(
            !shapes.backgrounds.is_empty(),
            "the Grid painted no Cell background"
        );
        for background in &shapes.backgrounds {
            let Shape::Rect(rect) = background else {
                panic!("a background was {background:?}")
            };
            assert!(
                rect.fill.a() > 0 && rect.stroke.width == 0.0,
                "a background carried a stroke: {rect:?}"
            );
        }
        assert!(!shapes.glyphs.is_empty(), "the Grid painted no Glyph");
        for glyph in &shapes.glyphs {
            assert!(
                matches!(glyph, Shape::Text(_)),
                "a Glyph was {glyph:?} rather than text"
            );
        }
        assert_eq!(shapes.cursor.len(), 1, "the Cursor is one stroke");
        for stroked in shapes.borders.iter().chain(&shapes.cursor) {
            let Shape::Rect(rect) = stroked else {
                panic!("a border was {stroked:?}")
            };
            assert!(
                rect.fill.a() == 0 && rect.stroke.width > 0.0,
                "a border carried a fill: {rect:?}"
            );
        }
        // The widened runs are the Shapes this grouping is about: one of them
        // reaches into Cells built after it, and would paint over their Glyphs
        // if the fills were emitted Cell by Cell.
        assert!(
            shapes
                .backgrounds
                .iter()
                .any(|run| rect_of(run).width() > viewport.cell_size * 1.5),
            "no background covered more than one Cell, so the grouping proves nothing"
        );
    }

    ///
    /// The five groups reach the painter end to end, in the order
    /// `SourceShapes::into_shapes` chains them: backgrounds, borders, Glyphs,
    /// seams, the Cursor.
    ///
    /// Read off the flat shape list of a whole console pass, which is the one
    /// place paint order is observable and which also covers the three calls
    /// inside `show_source` that nothing else exercises — derive a Paint, draw
    /// it, extend the painter once. This is not a concession: `egui_kittest`'s
    /// own regression tests assert against `output().shapes` the same way.
    ///
    /// The panel's own fill is in that list too — `source_panel_frame` paints
    /// it before the Grid's first Shape — and it is a plain fill just as a
    /// background run is. It is not subtracted, because every claim below is
    /// that the *last* member of one group precedes the *first* member of the
    /// next, and a fill preceding every Grid Shape moves neither bound.
    ///
    /// A Cell border and the Cursor are both stroked rectangles and nothing
    /// about the Shape tells them apart; what parts them is that the Cursor is
    /// painted after everything else, which is what this asserts.
    ///
    /// The Grid is 16 Cells square because the default Marker spacing is
    /// eight: an 8x8 Grid asks for no sector seam at all, and an empty group
    /// would let the claims either side of it hold vacuously.
    ///
    #[test]
    fn the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = Orcvs::new(16, 16);
        let mut view = SourceView::default();
        // A written Cell, so a Glyph is painted at all.
        orcvs.write("1");
        orcvs.select(orcvs.render_frame().rows()[0][0].position());

        let (viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        let at = |kind: fn(&Shape) -> bool| -> Vec<usize> {
            shapes
                .iter()
                .enumerate()
                .filter_map(|(index, shape)| kind(shape).then_some(index))
                .collect()
        };
        fn is_fill(shape: &Shape) -> bool {
            matches!(shape, Shape::Rect(rect) if rect.fill.a() > 0 && rect.stroke.width == 0.0)
        }
        let fills = at(is_fill);
        let strokes = at(|shape| matches!(shape, Shape::Rect(rect) if rect.stroke.width > 0.0));
        let glyphs = at(|shape| matches!(shape, Shape::Text(_)));
        let seams = at(|shape| matches!(shape, Shape::LineSegment { .. }));

        assert!(!fills.is_empty(), "the pass painted no fill");
        assert!(strokes.len() > 1, "the pass painted no Cell border");
        assert!(!glyphs.is_empty(), "the pass painted no Glyph");
        assert!(!seams.is_empty(), "the pass painted no sector seam");

        let cursor = *strokes.last().expect("the pass painted the Cursor");
        assert!(
            close(rect_of(&shapes[cursor]), viewport.cell_rect(0, 0)),
            "the last stroked rectangle was not the Cursor's Cell"
        );
        let borders = &strokes[..strokes.len() - 1];

        assert!(
            *fills.last().expect("a fill") < borders[0],
            "a background at {:?} painted over the border at {}",
            fills.last(),
            borders[0]
        );
        assert!(
            *borders.last().expect("a border") < glyphs[0],
            "a border at {:?} painted over the Glyph at {}",
            borders.last(),
            glyphs[0]
        );
        assert!(
            *glyphs.last().expect("a Glyph") < seams[0],
            "a Glyph at {:?} painted over the seam at {}",
            glyphs.last(),
            seams[0]
        );
        assert!(
            *seams.last().expect("a seam") < cursor,
            "a seam at {:?} painted over the Cursor at {cursor}",
            seams.last()
        );
    }

    ///
    /// The Source's Cells hold printable ASCII, so that is what the table
    /// covers. The space is the one printable character left out, because a
    /// Cell showing one paints no Glyph.
    ///
    #[test]
    fn the_glyph_table_covers_the_printable_ascii_a_cell_can_hold() {
        let ctx = egui::Context::default();
        let mut table = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            table = Some(GlyphTable::lay_out(
                ui.ctx(),
                egui::FontId::monospace(orcvs::opts::DEFAULT_FONT_SIZE),
            ));
        });
        output.drop_without_applying_deltas();
        let table = table.expect("the pass laid the alphabet out");

        for byte in ALPHABET_FIRST..=ALPHABET_LAST {
            assert!(
                table.galley(char::from(byte)).is_some(),
                "the table does not cover {:?}",
                char::from(byte)
            );
        }
        // Outside the alphabet on both sides, and beyond ASCII altogether.
        // Every one of these falls back to a single uncached layout for that
        // Cell alone.
        for character in [' ', '\n', '\u{7f}', 'é', '✦'] {
            assert!(
                table.galley(character).is_none(),
                "the table claims to cover {character:?}"
            );
        }
    }

    ///
    /// A character the table does not cover is still laid out, and is laid out
    /// as itself.
    ///
    /// `GlyphTable::glyph` answers a galley for every character, because the
    /// step that draws a Cell has to put *something* in the ordered sequence
    /// of Shapes rather than skip the Cell and paint it out of turn. The two
    /// ways it answers are asserted against each other here: inside the
    /// alphabet it hands back the table's own galley — the same `Arc`, not an
    /// equal one, which is the whole point of laying the alphabet out once —
    /// and outside it lays that one character out fresh.
    ///
    /// A Source Cell holds printable ASCII by construction, so the fallback is
    /// for a Source that found a way to hold something else. That is exactly
    /// why it is worth pinning: nothing else reaches it, so a fallback that
    /// silently answered the wrong Glyph would paint the wrong character with
    /// no other test noticing.
    ///
    #[test]
    fn a_character_outside_the_alphabet_is_laid_out_as_itself() {
        let ctx = egui::Context::default();
        let mut table = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            table = Some(GlyphTable::lay_out(
                ui.ctx(),
                egui::FontId::monospace(orcvs::opts::DEFAULT_FONT_SIZE),
            ));
        });
        output.drop_without_applying_deltas();
        let table = table.expect("the pass laid the alphabet out");

        for character in [' ', '\n', '\u{7f}', 'é', '✦'] {
            let galley = table.glyph(character);

            assert!(
                table.galley(character).is_none(),
                "{character:?} is in the alphabet, so this asserts nothing about the fallback"
            );
            assert_eq!(
                galley.text(),
                character.to_string(),
                "the fallback laid out something other than {character:?}"
            );
        }

        for byte in [ALPHABET_FIRST, b'A', ALPHABET_LAST] {
            let character = char::from(byte);
            let cached = table.galley(character).expect("inside the alphabet");

            assert!(
                std::sync::Arc::ptr_eq(&table.glyph(character), cached),
                "{character:?} was laid out again instead of read from the table"
            );
        }
    }

    ///
    /// Every Cell is stroked with its own border, one Grid line wide, and the
    /// Cursor's blink changes that colour rather than that width — which is
    /// what `cell_line_width` returned a constant for.
    ///
    /// The colours come from the Paint rather than from `cell_visuals`: which
    /// colour a Cell's border *is* is decided in the value layer and asserted
    /// there against `cell_visuals` itself. What is asserted here is that the
    /// shape step gives each Cell the border the Paint gave that Cell, and
    /// not its neighbour's.
    ///
    /// The Grid is wider than the Cursor's fifteen-Cell bloom, so the borders
    /// are not all one colour and a step that handed every Cell the same
    /// stroke would be caught.
    ///
    #[test]
    fn a_cell_border_is_one_grid_line_wide_whatever_the_cell_is_doing() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let orcvs = Orcvs::new(20, 20);
        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let viewport = presented(screen, 20, 20, 1.0);
        let shapes = source_shapes(&paint, viewport, 1.0);
        // The owned transform scales the stroke with everything else, the way
        // the Scene's layer transform used to, so the width is asserted in the
        // Source's own points.
        let scale = viewport.cell_size / CELL_SIZE;
        let mut colours = std::collections::BTreeSet::new();

        for position in paint.grid().positions_by_row().flatten() {
            let rect = viewport.cell_rect(position.x(), position.y());
            let stroked = shapes
                .borders
                .iter()
                .chain(&shapes.cursor)
                .find_map(|shape| match shape {
                    Shape::Rect(painted) if close(painted.rect, rect) => Some(painted.stroke),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("Cell {position:?} was never stroked"));

            assert!(
                (stroked.width / scale - GRID_LINE_WIDTH).abs() < 1e-3,
                "the border at {position:?} was {} points wide",
                stroked.width / scale
            );
            assert_eq!(
                stroked.color,
                paint.at(position).border,
                "the border at {position:?}"
            );
            colours.insert(stroked.color.to_array());
        }

        assert!(
            colours.len() > 1,
            "every Cell was stroked the same colour, so nothing was told apart"
        );
    }

    ///
    /// A Glyph is painted for every Cell that shows a character, at the centre
    /// of that Cell, in that Cell's own foreground — and for no Cell that shows
    /// the space.
    ///
    /// What a Cell shows and what colour it is are the Paint's answers, made
    /// with no `egui::Context` and asserted against `GlyphString` and
    /// `cell_visuals` in `paint.rs`. What this pins is the step between: a
    /// galley per Cell that has something to say, positioned on that Cell and
    /// handed that Cell's colour rather than its neighbour's.
    ///
    /// The Grid carries an Addition, whose claim reaches past the two Cells it
    /// is spelled in and leaves classified but empty operand Cells behind it,
    /// so the Cells showing something are not only the written ones.
    ///
    #[test]
    fn a_glyph_is_painted_for_every_cell_that_shows_one_and_no_other() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = Orcvs::new(8, 8);
        for (x, character) in ".+".chars().enumerate() {
            orcvs.select(orcvs.render_frame().rows()[2][x].position());
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.render_frame().rows()[5][5].position());

        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let viewport = presented(screen, 8, 8, 1.0);
        let shapes = source_shapes(&paint, viewport, 1.0);

        let shown: Vec<_> = paint
            .grid()
            .positions_by_row()
            .flatten()
            .filter(|position| paint.at(*position).character != ' ')
            .collect();

        assert!(
            shown.len() > 2,
            "only {} Cells showed a character, so the blank spellings went untested",
            shown.len()
        );
        assert_eq!(
            shapes.glyphs.len(),
            shown.len(),
            "painted {} Glyphs for {} Cells showing a character",
            shapes.glyphs.len(),
            shown.len()
        );
        for (shape, position) in shapes.glyphs.iter().zip(&shown) {
            let Shape::Text(text) = shape else {
                panic!("the Glyph at {position:?} was {shape:?}")
            };
            let rect = viewport.cell_rect(position.x(), position.y());

            assert_eq!(
                text.fallback_color,
                paint.at(*position).foreground,
                "the Glyph at {position:?}"
            );
            assert!(
                (text.pos - (rect.center() - text.galley.size() / 2.0)).length() < 1e-3,
                "the Glyph at {position:?} was painted at {:?}, off the Cell at {rect:?}",
                text.pos
            );
        }
    }

    ///
    /// The background the Grid declines to paint is the one the panel paints.
    ///
    /// `show_source` omits a Cell's rectangle wherever `cell_visuals` asks for
    /// `PALETTE.source`, and what stands in its place is the `CentralPanel`
    /// frame. The two values are stated in different places, so nothing but
    /// this holds them together: give the panel any other fill and every
    /// ordinary Cell — outside the Cursor's fifteen-Cell bloom, most of the
    /// default Grid — renders on a ground the palette never chose for it.
    ///
    /// The whole console is checked rather than the constant alone, because it
    /// is the painted result that has to sit on the right colour.
    ///
    #[test]
    fn the_omitted_background_is_the_colour_the_panel_is_filled_with() {
        assert_eq!(
            source_panel_frame().fill,
            PALETTE.source,
            "show_source omits a Cell's background wherever cell_visuals asks \
             for PALETTE.source, so the panel standing in for it must be \
             filled with exactly that colour"
        );
    }

    ///
    /// `cell_visuals` fills a Cell with the Source's own colour — which is what
    /// `Paint::derive` skips a background for — in exactly the cases
    /// `cursor_visible || (!selected && bloom.is_none())` names, over every
    /// case of the truth table and so in **both** halves of the Cursor's blink.
    ///
    /// This half of the split is a table over `cell_visuals`. It builds no
    /// Render Frame and no Paint, and that is what lets it reach the blink's
    /// visible half at all: a running Orcvs starts with the Cursor off and
    /// turns it on by elapsed time alone, with nothing public to set it, so
    /// `paint.rs`'s
    /// `the_cell_needing_no_background_is_exactly_the_one_filled_with_the_source`
    /// — which asserts the same skip over a Paint derived from a real Render
    /// Frame — can only ever see `cursor_visible` false. Neither test is the
    /// other written twice.
    ///
    /// `Paint::derive` compares the colours rather than restating the
    /// condition, so it cannot drift from `cell_visuals`. This is where the
    /// condition is written down, because it reads wrong: `cell_visuals` tests
    /// `cursor_visible` *before* its bloom arm, so the Cursor's own Cell takes
    /// the Source fill on the visible half of the blink even though
    /// `cursor_bloom` answers `Some(Core)` for it — and the blink therefore
    /// alternates a rectangle and no rectangle. A reordering of those arms
    /// would be a palette change, and this fails when one happens.
    ///
    #[test]
    fn the_skip_condition_matches_cell_visuals_in_both_blink_phases() {
        for glyph in [Glyph::Char, Glyph::Bang, Glyph::Space, Glyph::Comment] {
            for bloom in [
                None,
                Some(CursorBloom::Core),
                Some(CursorBloom::Inner),
                Some(CursorBloom::Mid),
                Some(CursorBloom::Outer),
            ] {
                for selected in [false, true] {
                    for cursor_visible in [false, true] {
                        let visuals =
                            crate::style::cell_visuals(glyph, bloom, selected, cursor_visible);
                        let skipped = cursor_visible || (!selected && bloom.is_none());

                        assert_eq!(
                            visuals.background == PALETTE.source,
                            skipped,
                            "{glyph:?} {bloom:?} selected={selected}, cursor_visible={cursor_visible}, filled {:?}",
                            visuals.background
                        );
                    }
                }
            }
        }
    }

    ///
    /// The viewport the `presented` helper builds is the viewport a whole
    /// console pass presents.
    ///
    /// Every Shape assertion here is made at a viewport built from
    /// `grid_viewport` and `presented_grid` directly rather than read back from
    /// a pass, and this is what stops that standing in from drifting from what
    /// `show_source_scene` does. At a fractional device scale as well as at
    /// one, which is where `presented_grid` floors the Cell side to whole
    /// physical pixels and the two could part company.
    ///
    #[test]
    fn the_presented_viewport_is_the_one_a_console_pass_presents() {
        for pixels_per_point in [1.0_f32, 1.5] {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
            let mut orcvs = Orcvs::new(8, 8);
            let mut view = SourceView::default();

            let (viewport, _) = console_pass_at(
                &ctx,
                screen,
                Vec::new(),
                &mut orcvs,
                &mut view,
                pixels_per_point,
            );

            assert_eq!(
                ctx.pixels_per_point(),
                pixels_per_point,
                "the pass did not run at the device scale it was asked for"
            );
            assert_eq!(viewport, presented(screen, 8, 8, pixels_per_point));
        }
    }

    ///
    /// A coalesced run covers exactly the Cells it replaces.
    ///
    /// Which Cells coalesce is `Paint::background_runs`' answer and is pinned
    /// there against a written-out table of spans; where the rectangle lands is
    /// `a_background_run_is_the_rectangle_its_columns_span`'s, pinned against
    /// written-out coordinates. What this adds is the part only a real
    /// presented viewport has: the run starts at the corner its first Cell was
    /// given by `GridViewport::cell_rect` — the column-to-x function every
    /// other Shape goes through — rather than at a sum of Cell sides
    /// accumulated across the row, it spans the Cells it replaced and no
    /// others, and those Cells would have tiled at exactly the pixels it
    /// covers, every interior edge landing on the same physical pixel from
    /// either side.
    ///
    /// Run at a fractional device scale as well as at one, because at one the
    /// Cell side is a whole point and there is nothing for a snap to move.
    ///
    #[test]
    fn a_background_run_covers_exactly_the_cells_it_replaces() {
        for pixels_per_point in [1.0_f32, 1.5] {
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
            let orcvs = Orcvs::new(8, 8);
            let frame = orcvs.render_frame();
            let paint = Paint::derive(&frame);
            let viewport = presented(screen, 8, 8, pixels_per_point);
            let shapes = source_shapes(&paint, viewport, pixels_per_point);
            let runs = paint.background_runs();

            assert!(
                runs.iter().any(|run| run.columns.len() > 1),
                "no run covered more than one Cell, so nothing was coalesced"
            );
            assert_eq!(
                shapes.backgrounds.len(),
                runs.len(),
                "painted {} rectangles for {} runs",
                shapes.backgrounds.len(),
                runs.len()
            );

            for (shape, run) in shapes.backgrounds.iter().zip(&runs) {
                let Shape::Rect(painted) = shape else {
                    panic!("a background was {shape:?}")
                };

                assert_eq!(painted.fill, run.colour, "a run was filled wrongly");
                // The near corner is the one the run's first Cell was given,
                // and the far corner is a whole number of Cells away from it.
                // Stated as a span rather than as the far Cell's own corner,
                // because that is the expression `SourceShapes::new` evaluates
                // and a test that re-ran it would pass any simultaneous edit to
                // both. `a_background_run_is_the_rectangle_its_columns_span`
                // pins the same mapping against written-out coordinates.
                assert_eq!(
                    painted.rect.min,
                    viewport
                        .cell_rect(run.columns.start, run.row)
                        .min
                        .round_to_pixels(pixels_per_point),
                    "the run over columns {:?} of row {} starts elsewhere",
                    run.columns,
                    run.row
                );
                // Half a Cell, because the snap rounds the two corners
                // independently and so can move a side by up to half a physical
                // pixel each — while a run one Cell too wide or too narrow is
                // out by a whole Cell side.
                let spanned = run.columns.len() as f32 * viewport.cell_size;
                assert!(
                    (painted.rect.width() - spanned).abs() < viewport.cell_size / 2.0,
                    "the run over columns {:?} of row {} is {} wide, not the {spanned} its Cells span",
                    run.columns,
                    run.row,
                    painted.rect.width()
                );
                assert!(
                    (painted.rect.height() - viewport.cell_size).abs() < viewport.cell_size / 2.0,
                    "the run over columns {:?} of row {} is {} tall, not one Cell",
                    run.columns,
                    run.row,
                    painted.rect.height()
                );
                for column in run.columns.start..run.columns.end - 1 {
                    assert_eq!(
                        viewport
                            .cell_rect(column, run.row)
                            .max
                            .x
                            .round_to_pixels(pixels_per_point),
                        viewport
                            .cell_rect(column + 1, run.row)
                            .min
                            .x
                            .round_to_pixels(pixels_per_point),
                        "the Cells either side of column {column} in row {} do not tile",
                        run.row
                    );
                }
            }
        }
    }

    ///
    /// A coalesced run becomes the rectangle its columns span, at coordinates
    /// written out here rather than re-derived.
    ///
    /// The viewport is stated instead of presented — a 25 point Cell with the
    /// Grid's corner at the origin — so every expected rectangle below is a
    /// literal. That is the point: the assertion this replaced re-ran
    /// `SourceShapes::new`'s own `Rect::from_min_max(cell_rect(start).min,
    /// cell_rect(end - 1).max)`, which catches a one-sided edit and passes a
    /// simultaneous one. These numbers move for neither.
    ///
    /// At one device pixel per point every edge of that Grid is already on a
    /// pixel, so the snap moves nothing and the literals are the coordinates
    /// the paint lands on. What a *fractional* scale does to them is
    /// `a_background_run_covers_exactly_the_cells_it_replaces`'.
    ///
    /// Four runs of the 8 by 8 default Grid, chosen so no one mistake passes
    /// all four: the Cursor's own Cell at the origin, a run of three ending
    /// short of the Grid's right edge, a run spanning a whole row but its last
    /// Cell, and the Grid's last Cell alone. Which Cells those are is
    /// `Paint::background_runs`' answer and is pinned in `paint.rs`.
    ///
    #[test]
    fn a_background_run_is_the_rectangle_its_columns_span() {
        let viewport = GridViewport {
            cell_size: 25.0,
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0)),
        };
        let orcvs = Orcvs::new(8, 8);
        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let shapes = source_shapes(&paint, viewport, 1.0);
        let runs = paint.background_runs();

        assert_eq!(
            shapes.backgrounds.len(),
            runs.len(),
            "painted {} rectangles for {} runs",
            shapes.backgrounds.len(),
            runs.len()
        );

        for (row, columns, expected) in [
            (
                0,
                0..1,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(25.0, 25.0)),
            ),
            (
                0,
                4..7,
                Rect::from_min_max(Pos2::new(100.0, 0.0), Pos2::new(175.0, 25.0)),
            ),
            (
                5,
                0..7,
                Rect::from_min_max(Pos2::new(0.0, 125.0), Pos2::new(175.0, 150.0)),
            ),
            (
                7,
                7..8,
                Rect::from_min_max(Pos2::new(175.0, 175.0), Pos2::new(200.0, 200.0)),
            ),
        ] {
            let index = runs
                .iter()
                .position(|run| run.row == row && run.columns == columns)
                .unwrap_or_else(|| {
                    panic!("this Grid asks for no run over columns {columns:?} of row {row}")
                });

            assert_eq!(
                rect_of(&shapes.backgrounds[index]),
                expected,
                "the run over columns {columns:?} of row {row}"
            );
        }
    }

    ///
    /// A whole console pass strokes the Grid at the zoom it presented the
    /// Source at, and snaps its background runs to the device scale it ran on.
    ///
    /// Both are `show_source`'s own arithmetic — the zoom is the presented Cell
    /// side over the Source's own, the device scale is the `Ui`'s — and both
    /// are handed to `SourceShapes::new` and reach the Shapes nowhere else.
    /// Every other Shape assertion here builds a `SourceShapes` through the
    /// `source_shapes` helper, which is given a zoom and a device scale the
    /// test chose, so all of them still hold with either argument replaced by a
    /// constant one at the call site. What would ship then is a Grid whose
    /// lines and sector seams stay one Source point wide at every zoom instead
    /// of scaling with it, and runs snapped to whole points on a screen whose
    /// pixels are not whole points.
    ///
    /// The geometry is chosen so neither argument can be mistaken for one. A
    /// 201 point console over a 20 Cell Grid fits the Source at 0.4, and at a
    /// device scale of 1.5 the presented Grid's corner is floored two physical
    /// pixels in — two thirds of a point — so every run edge is snapped
    /// somewhere a snap to whole points would not put it.
    ///
    #[test]
    fn a_console_pass_strokes_at_its_own_zoom_and_snaps_its_runs_to_its_own_device_scale() {
        const DEVICE_SCALE: f32 = 1.5;
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(201.0));
        let mut orcvs = Orcvs::new(20, 20);
        let mut view = SourceView::default();

        let (viewport, shapes) = console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            DEVICE_SCALE,
        );
        let scale = viewport.cell_size / CELL_SIZE;

        assert!(
            (scale - 0.4).abs() < 1e-6,
            "the pass fitted the Source at {scale}, and a zoom of one would be \
             indistinguishable from the constant"
        );

        // Every Cell is stroked once — the Cursor's by the Cursor — and every
        // one of those strokes carries the zoom.
        let mut stroked = 0;
        for shape in &shapes {
            if let Shape::Rect(painted) = shape
                && painted.stroke.width > 0.0
            {
                assert!(
                    (painted.stroke.width - GRID_LINE_WIDTH * scale).abs() < 1e-6,
                    "a Cell border was stroked {} points wide against {} at this zoom",
                    painted.stroke.width,
                    GRID_LINE_WIDTH * scale
                );
                stroked += 1;
            }
        }
        assert_eq!(stroked, 400, "the pass stroked {stroked} of 400 Cells");

        // And so does every sector seam, which takes its own width.
        let mut seams = 0;
        for shape in &shapes {
            if let Shape::LineSegment { stroke, .. } = shape {
                assert!(
                    (stroke.width - SECTOR_LINE_WIDTH * scale).abs() < 1e-6,
                    "a sector seam was stroked {} points wide against {} at this zoom",
                    stroke.width,
                    SECTOR_LINE_WIDTH * scale
                );
                seams += 1;
            }
        }
        assert!(seams > 0, "the pass drew no sector seam");

        // Which Cells coalesce into a run is `Paint::background_runs`' answer
        // and is pinned there; what this asks is where the pass put the
        // rectangle that replaces them.
        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let runs = paint.background_runs();
        assert!(!runs.is_empty(), "the pass painted no background run");

        for run in &runs {
            let covered = Rect::from_min_max(
                viewport.cell_rect(run.columns.start, run.row).min,
                viewport.cell_rect(run.columns.end - 1, run.row).max,
            );
            let snapped = covered.round_to_pixels(DEVICE_SCALE);

            assert_ne!(
                snapped,
                covered.round_to_pixels(1.0),
                "the run over columns {:?} of row {} is snapped to the same \
                 rectangle at either device scale, so it tells them apart from \
                 nothing",
                run.columns,
                run.row
            );
            assert!(
                shapes.iter().any(|shape| matches!(
                    shape,
                    Shape::Rect(painted)
                        if painted.rect == snapped
                            && painted.fill == run.colour
                            && painted.stroke.width == 0.0
                )),
                "the pass filled nothing at {snapped:?} for the run over columns \
                 {:?} of row {}",
                run.columns,
                run.row
            );
        }
    }

    ///
    /// A sector seam is drawn on the Cell edge the Paint asks for, in the
    /// colour it asks for, one sector line wide.
    ///
    /// Which Cells carry a seam, at what strength, and that the Cursor's own
    /// Cell carries none, are the Render Frame's and the derive's answers and
    /// are asserted in `paint.rs` with no Context at all. The seam's *width*
    /// scales with the Cell side, so it is geometry and belongs here.
    ///
    /// The Grid is 16 Cells square because the default Marker spacing is
    /// eight: an 8x8 Grid has no column or row that is a non-zero multiple of
    /// it, so every seam strength in one is `None` and this would assert
    /// nothing.
    ///
    #[test]
    fn a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = Orcvs::new(16, 16);
        // The Cursor goes on a Cell that would otherwise carry both seams, so
        // the suppression the derive applies is visible as an absence here too.
        orcvs.select(orcvs.render_frame().rows()[8][8].position());

        let frame = orcvs.render_frame();
        let paint = Paint::derive(&frame);
        let viewport = presented(screen, 16, 16, 1.0);
        let shapes = source_shapes(&paint, viewport, 1.0);
        let scale = viewport.cell_size / CELL_SIZE;

        let mut expected = Vec::new();
        for position in paint.grid().positions_by_row().flatten() {
            let rect = viewport.cell_rect(position.x(), position.y());
            let cell = paint.at(position);

            for (colour, ends) in [
                (cell.sector_left, [rect.left_top(), rect.left_bottom()]),
                (cell.sector_top, [rect.left_top(), rect.right_top()]),
            ] {
                if let Some(colour) = colour {
                    expected.push((position, colour, ends));
                }
            }
        }

        assert!(
            !expected.is_empty(),
            "the Paint asked for no sector seam, so nothing was asserted"
        );
        assert_eq!(
            shapes.seams.len(),
            expected.len(),
            "drew {} seams against {} the Paint asked for",
            shapes.seams.len(),
            expected.len()
        );
        for (shape, (position, colour, ends)) in shapes.seams.iter().zip(&expected) {
            let Shape::LineSegment { points, stroke } = shape else {
                panic!("the seam at {position:?} was {shape:?}")
            };

            assert!(
                (points[0] - ends[0]).length() < 1e-3 && (points[1] - ends[1]).length() < 1e-3,
                "the seam at {position:?} ran {points:?} rather than {ends:?}"
            );
            assert_eq!(stroke.color, *colour, "the seam at {position:?}");
            assert!(
                (stroke.width / scale - SECTOR_LINE_WIDTH).abs() < 1e-3,
                "the seam at {position:?} was {} points wide",
                stroke.width / scale
            );
        }
    }

    ///
    /// A middle drag that starts on a Cell pans the Source.
    ///
    /// This is what `Sense::CLICK` on the Grid rectangle buys. Within one layer
    /// a later-registered child wins the click tie and would win the drag tie
    /// too if it sensed drag, and the Grid is registered after the pan
    /// response. Sensing clicks alone is what leaves the drag to the pan
    /// rectangle, and the drag has to be started *over the Grid* to assert it:
    /// the letterboxing is territory the Grid never covered.
    ///
    #[test]
    fn a_middle_drag_that_starts_on_a_cell_still_pans_the_source() {
        let ctx = egui::Context::default();
        let wide = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
        // The middle of the Grid, which is a Cell rather than letterboxing.
        let over_a_cell = viewport.cell_rect(4, 4).center();
        assert!(
            viewport.rect.contains(over_a_cell),
            "the drag did not start over the Grid"
        );
        let fitted = view.to_global;

        console_frame(
            &ctx,
            wide,
            middle_press_at(over_a_cell),
            &mut orcvs,
            &mut view,
        );
        console_frame(
            &ctx,
            wide,
            vec![Event::PointerMoved(over_a_cell + Vec2::new(40.0, 25.0))],
            &mut orcvs,
            &mut view,
        );

        assert!(
            view.adjusted,
            "a middle drag over a Cell did not pan the Source"
        );
        assert_ne!(view.to_global, fitted, "the pan did not move the view");
    }

    const WIDE: Vec2 = Vec2::new(400.0, 200.0);
    const TALL: Vec2 = Vec2::new(200.0, 400.0);

    #[test]
    fn a_resize_re_fits_the_viewport_while_the_view_is_unpinned() {
        let ctx = egui::Context::default();
        let wide = Rect::from_min_size(Pos2::ZERO, WIDE);
        let tall = Rect::from_min_size(Pos2::ZERO, TALL);
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
        let wide_fit = view.to_global;
        let viewport = console_frame(&ctx, tall, Vec::new(), &mut orcvs, &mut view);

        assert!(!view.adjusted, "an untouched view was pinned");
        assert_ne!(view.to_global, wide_fit, "the resize did not re-fit");
        // The Grid follows the re-fitted viewport rather than the old one.
        click(
            &ctx,
            tall,
            viewport.rect.max - Vec2::splat(viewport.cell_size / 2.0),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(selected_cell(&orcvs), (7, 7));
    }

    #[test]
    fn a_zoom_pins_the_view_and_a_later_resize_leaves_it_where_the_viewer_put_it() {
        let ctx = egui::Context::default();
        let wide = Rect::from_min_size(Pos2::ZERO, WIDE);
        let tall = Rect::from_min_size(Pos2::ZERO, TALL);
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
        let over_the_scene = letterboxing(wide, viewport.rect).expect("a wide console letterboxes");
        console_frame(&ctx, wide, zoom_at(over_the_scene), &mut orcvs, &mut view);

        assert!(view.adjusted, "zooming did not pin the view");
        let pinned = view.to_global;
        console_frame(&ctx, tall, Vec::new(), &mut orcvs, &mut view);

        assert_eq!(
            view.to_global, pinned,
            "the resize discarded the viewer's zoom"
        );
    }

    ///
    /// A zoom the `zoom_range` clamp reverts is not a zoom, so it leaves the
    /// view unpinned and still re-fitting.
    ///
    /// `Scene::register_pan_and_zoom` calls `mark_changed` whenever a zoom or
    /// scroll event arrived at all, whether or not the clamp left `to_global`
    /// exactly where it was (`scene.rs:265-274`), so `Response::changed` cannot
    /// say whether the view moved. Pinning on it costs the viewer every later
    /// re-fit: the owned transform is absolute, and unlike the Scene-space
    /// rectangle it replaces it does not track the window across a resize.
    ///
    #[test]
    fn a_zoom_the_clamp_reverts_leaves_the_view_unpinned_and_re_fitting() {
        let ctx = egui::Context::default();
        // An 8 by 8 Source is 200 points square, so a 400 point console fits it
        // at exactly two — which is `MAX_ZOOM`, leaving a zoom in nowhere to go.
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(400.0));
        let larger = Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0));
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.to_global.scaling, MAX_ZOOM,
            "the console did not fit at the zoom ceiling"
        );
        let fitted = view.to_global;

        console_frame(
            &ctx,
            screen,
            zoom_at(screen.center()),
            &mut orcvs,
            &mut view,
        );

        assert_eq!(view.to_global, fitted, "the clamp let the zoom through");
        assert!(!view.adjusted, "a zoom that moved nothing pinned the view");

        // And the view is still the console's to re-fit.
        console_frame(&ctx, larger, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.to_global.scaling, 4.0,
            "the resize did not re-fit the Grid the viewer never moved"
        );
    }

    #[test]
    fn a_double_click_unpins_the_view_and_hands_it_back_to_the_fit() {
        let ctx = egui::Context::default();
        let wide = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
        let fitted = view.to_global;
        let over_the_scene = letterboxing(wide, viewport.rect).expect("a wide console letterboxes");
        console_frame(&ctx, wide, zoom_at(over_the_scene), &mut orcvs, &mut view);
        assert!(view.adjusted, "zooming did not pin the view");

        double_click(&ctx, wide, over_the_scene, &mut orcvs, &mut view);
        assert!(!view.adjusted, "the double click did not unpin the view");

        console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.to_global, fitted, "the view did not return to the fit");
    }

    ///
    /// A double click inside the Grid selects the Cell and leaves the view
    /// where the viewer put it.
    ///
    /// The reset is the letterboxing's gesture alone, because the Grid's click
    /// rectangle is registered after the pan rectangle and wins every tie
    /// inside the Grid. That is the Cell Buttons' own resolution, kept
    /// deliberately, and it has a consequence worth pinning rather than
    /// leaving to the comment beside the branch: a console the Grid fills
    /// exactly has no letterboxing, so it offers no way to double click back
    /// to the fit. `DEFAULT_VIEW_SIZE` is such a console.
    ///
    #[test]
    fn a_double_click_inside_the_grid_selects_a_cell_and_holds_the_view() {
        let ctx = egui::Context::default();
        // A 8 by 8 Source is 200 points square, so this console fits it
        // exactly and letterboxes nowhere — the shape `DEFAULT_VIEW_SIZE` has.
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0));
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert!(
            letterboxing(screen, viewport.rect).is_none(),
            "the console letterboxes, so it is not the case this test is about"
        );

        console_frame(
            &ctx,
            screen,
            zoom_at(screen.center()),
            &mut orcvs,
            &mut view,
        );
        assert!(view.adjusted, "zooming did not pin the view");
        let pinned = view.to_global;

        let target = viewport.rect.min + Vec2::new(2.5, 3.5) * viewport.cell_size;
        double_click(&ctx, screen, target, &mut orcvs, &mut view);

        assert_eq!(
            selected_cell(&orcvs),
            (2, 3),
            "the double click did not reach the Cell under it"
        );
        assert!(
            view.adjusted,
            "the double click unpinned a view with no letterboxing to hit"
        );
        assert_eq!(
            view.to_global, pinned,
            "the double click moved a view it should have left alone"
        );
    }

    ///
    /// A middle-button drag pans the Source by exactly what the pointer moved,
    /// at a scale that is not one.
    ///
    /// This is the one behaviour retiring the container could break silently.
    /// `Scene::register_pan_and_zoom` pans with `to_global.translation +=
    /// to_global.scaling * resp.drag_delta()` (`scene.rs:239`), and
    /// `Response::drag_delta` divides by the layer transform's scaling *only
    /// when the layer has one* (`response.rs:452-465`). Inside `Scene::show`
    /// those cancel; with the transform owned by the console there is no layer
    /// transform, nothing divides, and the multiply would move the Source by
    /// the zoom factor times the pointer. At the default window the fitted
    /// scale is exactly one and that bug is invisible, so this console is sized
    /// to fit at two.
    ///
    #[test]
    fn a_middle_drag_pans_by_the_pointer_and_not_by_the_pointer_times_the_zoom() {
        let ctx = egui::Context::default();
        // A 8 by 8 Source is 200 points square, so a 400 point console fits it
        // at exactly two.
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(400.0));
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        let before = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.to_global.scaling, 2.0,
            "the console did not fit at two"
        );
        let anchor = view.to_global.translation;

        // Press, then move further than `max_click_dist` so the gesture
        // resolves as a drag rather than a click.
        let from = screen.center();
        let moved = Vec2::new(40.0, 24.0);
        console_frame(&ctx, screen, middle_press_at(from), &mut orcvs, &mut view);
        let after = console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(from + moved)],
            &mut orcvs,
            &mut view,
        );

        assert!(view.adjusted, "the pan did not pin the view");
        assert_eq!(
            view.to_global.translation - anchor,
            moved,
            "the Source panned by {:?} for a pointer that moved {moved:?}",
            view.to_global.translation - anchor
        );
        assert_eq!(
            after.rect.min - before.rect.min,
            moved,
            "the presented Grid moved by {:?}",
            after.rect.min - before.rect.min
        );
        assert_eq!(
            after.cell_size, before.cell_size,
            "the pan changed the Cell size"
        );

        // The click arithmetic followed the pan: painting and clicking go
        // through the one presented viewport, at a scale and an offset the fit
        // never chose.
        console_frame(
            &ctx,
            screen,
            vec![Event::PointerButton {
                pos: from + moved,
                button: egui::PointerButton::Middle,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
            &mut orcvs,
            &mut view,
        );
        let target = after.rect.min + Vec2::new(3.5, 1.5) * after.cell_size;
        click(&ctx, screen, target, &mut orcvs, &mut view);

        assert_eq!(selected_cell(&orcvs), (3, 1), "a click at {target:?}");
    }

    ///
    /// **The acceptance criterion the whole effort exists for.**
    ///
    /// No layer the Source Grid is painted into carries a transform, so no
    /// `TextShape` in it reaches `Arc::make_mut`. `GraphicLayers::drain`
    /// applies a layer's transform to every shape in it at end of pass, and for
    /// a `TextShape` that is `TextShape::transform`, which reaches the galley
    /// through `Arc::make_mut` and each of its rows through `Arc::make_mut`
    /// again. epaint's `GalleyCache` holds an `Arc` to every galley it hands
    /// out, so the refcount is never one and the clone is never elided. Given a
    /// transform the clone is unconditional, so the *absence* of the transform
    /// is the whole proof — there is nothing else to observe, and a profile
    /// would not show it anyway, because the clone happens inside `end_pass`
    /// rather than in `tessellate_shapes`.
    ///
    /// The console is sized to fit at two rather than at one on purpose:
    /// `Context::set_transform_layer` *removes* the entry for an identity
    /// transform, so a fit of one would let a Scene pass this.
    ///
    #[test]
    fn no_layer_carrying_the_source_grid_is_transformed() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(400.0));
        let mut orcvs = Orcvs::new(8, 8);
        orcvs.write("1");
        let mut view = SourceView::default();
        let frame = orcvs.render_frame();
        let mut grid_layer = None;

        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |root| {
                egui::CentralPanel::default()
                    .frame(source_panel_frame())
                    .show(root, |ui| {
                        grid_layer = Some(ui.layer_id());
                        show_source_scene(ui, &frame, &egui::FontFamily::Monospace, &mut view);
                    });
            },
        );
        let mut painted = Vec::new();
        for clipped in &output.shapes {
            flatten(clipped.shape.clone(), &mut painted);
        }
        output.drop_without_applying_deltas();

        assert_eq!(
            view.to_global.scaling, 2.0,
            "the console did not fit at two"
        );
        assert!(
            painted.iter().any(|shape| matches!(shape, Shape::Text(_))),
            "the pass painted no Glyph, so it proves nothing about galleys"
        );

        let grid_layer = grid_layer.expect("the central panel showed the Source");
        assert_eq!(
            ctx.layer_transform_to_global(grid_layer),
            None,
            "the Source Grid's own layer carries a transform"
        );
        // Not just that layer: no layer at all. The Scene painted into a
        // sublayer of its own, so checking only the layer the console paints
        // into would miss the transform that used to exist.
        assert!(
            ctx.memory(|memory| memory.to_global.is_empty()),
            "a layer transform survived: {:?}",
            ctx.memory(|memory| memory.to_global.clone())
        );
    }
}

///
/// The console's own end of the storage seam: the two lines that wire the
/// running Console to `console::persistence`. Every other persistence test drives
/// the persistence interface directly and would still pass with the
/// Console unwired, so these construct a real `Console` and call the real
/// `eframe::App::save`.
///
#[cfg(all(test, feature = "persistence"))]
mod storage_tests {
    use eframe::App as _;
    use orcvs::source::SourceCommander;

    use super::Console;
    use crate::persistence::{
        InMemoryStorage, REFUSED_KEY, SOURCE_KEY, edited_source, starting_source, store,
    };

    ///
    /// Storage holding a value no build can read back, and that value, so a
    /// test can assert on the thing that was refused.
    ///
    fn storage_holding_a_refused_value() -> (InMemoryStorage, String) {
        let mut written = InMemoryStorage::default();
        store(&mut written, &edited_source());
        let refused = eframe::Storage::get_string(&written, SOURCE_KEY)
            .expect("the save call stored the revision")
            .replace("cols:6", "cols:7");

        let mut storage = InMemoryStorage::default();
        eframe::Storage::set_string(&mut storage, SOURCE_KEY, refused.clone());
        (storage, refused)
    }

    ///
    /// A Console started the way eframe starts it, over `storage`.
    ///
    /// `_new_kittest` is eframe's own headless `CreationContext`, which is how
    /// an `App` is constructed outside a window; it opens with no storage, and
    /// the field is public precisely so a test can supply one.
    ///
    fn console_over(storage: &dyn eframe::Storage) -> Console {
        let mut cc = eframe::CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(storage);
        Console::new(&cc)
    }

    ///
    /// A refused value is Cells a viewer may still recover by hand, and the
    /// console's own save is what would otherwise destroy them: eframe calls it
    /// every thirty seconds and it writes the key the refused value sits under.
    ///
    #[test]
    fn a_refused_value_is_preserved_before_the_next_save_overwrites_it() {
        let (mut storage, refused) = storage_holding_a_refused_value();

        let mut console = console_over(&storage);
        console.save(&mut storage);

        assert_eq!(
            eframe::Storage::get_string(&storage, REFUSED_KEY).as_deref(),
            Some(refused.as_str()),
            "the refused value was not preserved"
        );
        // The save still happened: a console that stopped saving after a
        // refusal would lose the session that followed it instead.
        assert!(
            eframe::Storage::get_string(&storage, SOURCE_KEY).is_some(),
            "the console stopped saving after a refusal"
        );
    }

    ///
    /// A viewer looking at a Grid that is not theirs is told so by the running
    /// Console, and stays told after the save that moves the refused value
    /// aside. `persistence.rs` proves the obligations end independently; this
    /// proves the Console is wired to them at all, which is the one thing an
    /// interface-level test cannot see.
    ///
    #[test]
    fn a_refused_start_raises_a_console_notice_that_outlives_the_save() {
        let (mut storage, _) = storage_holding_a_refused_value();

        let mut console = console_over(&storage);
        assert!(
            console.persistence.notice_visible(),
            "a refused start told the viewer nothing"
        );

        console.save(&mut storage);

        assert!(
            console.persistence.notice_visible(),
            "the notice went with the value the save moved aside"
        );
    }

    ///
    /// A start with nothing wrong raises nothing. A notice a viewer sees on an
    /// ordinary start is a notice they learn to ignore.
    ///
    #[test]
    fn an_absent_or_restored_start_raises_no_console_notice() {
        let mut restored = InMemoryStorage::default();
        store(&mut restored, &edited_source());

        for storage in [InMemoryStorage::default(), restored] {
            assert!(!console_over(&storage).persistence.notice_visible());
        }
    }

    #[test]
    fn a_console_starts_the_revision_its_creation_storage_holds() {
        let saved = edited_source();
        let mut storage = InMemoryStorage::default();
        store(&mut storage, &saved);

        let console = console_over(&storage);

        assert_eq!(
            console.orcvs.source().snapshot(),
            saved.snapshot(),
            "the Console did not start the revision storage held"
        );
        assert_eq!(console.orcvs.source().grid().count(), 18);
    }

    #[test]
    fn the_console_save_call_stores_the_current_revision() {
        let mut restored_from = InMemoryStorage::default();
        store(&mut restored_from, &edited_source());
        let mut console = console_over(&restored_from);
        let mut storage = InMemoryStorage::default();

        console.save(&mut storage);

        // A Console that never saves leaves storage empty, and the start that
        // reads it opens the ordinary default Grid instead of this revision.
        let next_start = SourceCommander::with_source(starting_source(Some(&storage)).source);
        assert_eq!(
            next_start.snapshot(),
            edited_source().snapshot(),
            "the next start did not open the revision the Console saved"
        );
    }
}
