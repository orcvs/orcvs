use std::sync::Arc;

use egui::{
    Color32, CornerRadius, Event, EventFilter, FontId, Key, Pos2, Rect, Sense, Shape, Stroke,
    StrokeKind, Vec2, epaint::RectShape, text::Galley,
};

use crate::grid_viewport::{GridViewport, grid_viewport};
use crate::midi::MidiDeviceSelection;
use crate::persistence::starting_source;
use crate::style::{PALETTE, cell_visuals, sector_line, style};
use orcvs::{
    app::{InputEvent, InputKey, Orcvs},
    glyph::{Glyph, GlyphString},
    grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT},
    native_midi::{self, NativeMidiBackend},
    opts::{Bpm, DEFAULT_FONT_SIZE},
    render_frame::{RenderCell, RenderFrame},
};

const CELL_SIZE: f32 = 25.0;
const GRID_LINE_WIDTH: f32 = 0.5;
const SECTOR_LINE_WIDTH: f32 = 0.75;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 2.0;

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

fn source_dimensions(frame: &RenderFrame) -> (usize, usize) {
    let rows = frame.rows();
    let col_count = rows
        .first()
        .expect("a Render Frame contains at least one row")
        .len();
    debug_assert!(
        rows.iter().all(|row| row.len() == col_count),
        "a Render Frame has the Grid's fixed rectangular shape"
    );

    (col_count, rows.len())
}

fn source_bounds(columns: usize, rows: usize) -> Rect {
    Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(columns as f32, rows as f32) * CELL_SIZE,
    )
}

///
/// The region of the Source the Scene shows, and whether the viewer has moved
/// it.
///
/// While the viewer has not panned or zoomed, the region follows the fitted
/// square viewport, so every resize re-fits rather than cropping.
///
struct SourceView {
    rect: Rect,
    adjusted: bool,
}

impl Default for SourceView {
    fn default() -> Self {
        Self {
            rect: Rect::ZERO,
            adjusted: false,
        }
    }
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

fn scene_zoom(console_area: Vec2, source_view_rect: Rect) -> Option<f32> {
    (source_view_rect.is_positive() && console_area.x > 0.0 && console_area.y > 0.0).then(|| {
        (console_area.x / source_view_rect.width()).min(console_area.y / source_view_rect.height())
    })
}

fn show_diagnostics(
    ctx: &egui::Context,
    open: &mut bool,
    frame: &eframe::Frame,
    source_view_rect: Rect,
    console_area: Vec2,
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

                    ui.label("Source zoom");
                    ui.monospace(
                        scene_zoom(console_area, source_view_rect)
                            .map(|zoom| format!("{zoom:.2}×"))
                            .unwrap_or_else(|| "—".to_owned()),
                    );
                    ui.end_row();

                    ui.label("Visible Source region");
                    ui.monospace(format!("{source_view_rect:.1?}"));
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
    /// The font the alphabet was laid out at, for the one character the table
    /// cannot cover.
    font: FontId,
    /// The alphabet, indexed by `byte - ALPHABET_FIRST`.
    characters: Vec<Arc<Galley>>,
    /// The character an empty Cell shows, indexed by [`blank_glyph_index`].
    blanks: [char; BLANK_GLYPHS.len()],
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
            font,
            characters,
            blanks: BLANK_GLYPHS.map(blank_character),
        }
    }

    /// The character `cell` shows.
    fn character(&self, cell: &RenderCell) -> char {
        cell.content()
            .unwrap_or_else(|| self.blanks[blank_glyph_index(cell.glyph())])
    }

    /// The laid-out Glyph for `character`, or `None` for a character outside
    /// the alphabet — the space included.
    fn galley(&self, character: char) -> Option<&Arc<Galley>> {
        let byte = u8::try_from(character).ok()?;
        self.characters
            .get(usize::from(byte.checked_sub(ALPHABET_FIRST)?))
    }
}

///
/// Draws the Source Grid and answers the one question a click asks of it.
///
/// The whole Grid is one allocated rectangle and every Cell is painted, so no
/// Cell is a widget and the cost of a Render Frame is shapes rather than
/// interaction rects and widget ids.
///
fn show_source(
    ui: &mut egui::Ui,
    orcvs: &mut Orcvs,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
) {
    let (columns, rows) = source_dimensions(frame);
    // One rectangle for the whole Grid, sensing clicks and nothing else.
    //
    // Within a layer a later-registered child wins the click tie, and would win
    // the drag too if it sensed drag. The Scene's pan response is registered
    // before any content, so sensing clicks alone takes the clicks and leaves
    // the middle-drag pan to the Scene. `Sense::CLICK` rather than
    // `Sense::click()`, which is `CLICK | FOCUSABLE` and would put the Grid in
    // the tab order where a thousand Buttons never were.
    //
    // The rectangle is the Grid, not the console area. The letterboxing is the
    // only territory where the Scene's own `double_clicked()` still fires, and
    // that double click is what hands a pinned view back to the fit.
    let (grid_rect, response) = ui.allocate_exact_size(
        Vec2::new(columns as f32, rows as f32) * CELL_SIZE,
        Sense::CLICK,
    );
    // The Grid as the Source lays it out. Painting and clicking go through this
    // one arithmetic, so a click cannot resolve to a Cell other than the one
    // drawn under it.
    let grid = GridViewport {
        cell_size: CELL_SIZE,
        rect: grid_rect,
    };
    let glyphs = GlyphTable::lay_out(
        ui.ctx(),
        FontId::new(DEFAULT_FONT_SIZE, font_family.clone()),
    );

    let cells = columns.saturating_mul(rows);
    let mut backgrounds = Vec::with_capacity(cells);
    let mut painted_glyphs = Vec::with_capacity(cells);
    let mut seams = Vec::new();
    let mut cursor_strokes = Vec::new();

    for row in frame.rows() {
        for cell in row {
            let position = cell.position();
            let rect = grid.cell_rect(position.x(), position.y());
            let visuals = cell_visuals(
                cell.glyph(),
                cell.cursor_bloom(),
                cell.selected(),
                cell.cursor_visible(),
            );
            let border = Stroke::new(GRID_LINE_WIDTH, visuals.border);

            backgrounds.push(Shape::Rect(RectShape::new(
                rect,
                CornerRadius::ZERO,
                visuals.background,
                // The selected Cell's border is the Cursor, and the Cursor is
                // painted last.
                if cell.selected() {
                    Stroke::NONE
                } else {
                    border
                },
                StrokeKind::Inside,
            )));

            if cell.selected() {
                cursor_strokes.push(Shape::Rect(RectShape::stroke(
                    rect,
                    CornerRadius::ZERO,
                    border,
                    StrokeKind::Inside,
                )));
            } else {
                // A sector seam is suppressed on a selected Cell, so the Cursor
                // is never crossed by one.
                if let Some(strength) = cell.sector_left_strength() {
                    seams.push(Shape::line_segment(
                        [rect.left_top(), rect.left_bottom()],
                        Stroke::new(SECTOR_LINE_WIDTH, sector_line(strength)),
                    ));
                }
                if let Some(strength) = cell.sector_top_strength() {
                    seams.push(Shape::line_segment(
                        [rect.left_top(), rect.right_top()],
                        Stroke::new(SECTOR_LINE_WIDTH, sector_line(strength)),
                    ));
                }
            }

            let character = glyphs.character(cell);
            if character != ' ' {
                let galley = match glyphs.galley(character) {
                    Some(galley) => galley.clone(),
                    // A character the alphabet does not cover. A Source Cell
                    // holds printable ASCII by construction, so this lays out
                    // at most the odd galley for a Source that found a way to
                    // hold something else — and it costs that one Cell the
                    // allocation and the whole-`Context` lock the table exists
                    // to take once. It stays a Shape in the ordered sequence
                    // rather than a `Painter::text`, which would both allocate
                    // and paint out of turn.
                    None => ui.painter().layout_no_wrap(
                        character.to_string(),
                        glyphs.font.clone(),
                        Color32::PLACEHOLDER,
                    ),
                };
                // Centred in a Cell whose own corner is an exact multiple of
                // the Cell size.
                painted_glyphs.push(Shape::galley(
                    rect.center() - galley.size() / 2.0,
                    galley,
                    visuals.foreground,
                ));
            }
        }
    }

    // One `Painter::extend`, never a `Painter::add` per Shape. `add` reaches
    // `Context::graphics_mut`, which is a full `Context` write lock, so a
    // per-Cell loop would take more locks than the Button field it replaces and
    // turn this change into a regression.
    //
    // Every background precedes every Glyph, so a later Cell's fill can never
    // paint over an earlier Cell's Glyph, and the Cursor comes after both, so no
    // neighbouring Cell's fill or seam can paint over it.
    ui.painter().extend(
        backgrounds
            .into_iter()
            .chain(painted_glyphs)
            .chain(seams)
            .chain(cursor_strokes),
    );

    // The click resolves by division through the viewport the Cells were
    // painted at. `interact_pointer_pos` is already in the Scene's own
    // coordinates, which is the space `grid_rect` is in.
    if response.clicked()
        && let Some(pointer) = response.interact_pointer_pos()
        && let Some((column, row)) = grid.cell_at(pointer, columns, rows)
        && let Some(cell) = frame.rows().get(row).and_then(|row| row.get(column))
    {
        orcvs.select(cell.position());
    }
}

///
/// Shows the Source in the largest square-Celled viewport the console area
/// holds, centred so the surplus is letterboxing, and answers the geometry it
/// was presented under.
///
/// The Scene is the one place the Source is scaled, so a Cell's two axes cannot
/// part company: a Scene scales both under a single factor. Every Cell, and so
/// every click that lands on one, goes through this geometry.
///
fn show_source_scene(
    ui: &mut egui::Ui,
    orcvs: &mut Orcvs,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    view: &mut SourceView,
) -> GridViewport {
    let available = ui.available_rect_before_wrap();
    let (columns, rows) = source_dimensions(frame);
    let source = source_bounds(columns, rows);
    let viewport = grid_viewport(available, columns, rows);

    if !view.adjusted {
        view.rect = viewport.scene_view(source, available);
    }
    // The fitted scale has to be reachable, or the Scene clamps it and the Grid
    // stops filling the console. A console smaller than the viewer's zoom
    // limits allows fits it out on either side, so both ends give. A console
    // with no area answers a scale of zero, which is no fit to reach.
    let fitted_zoom = viewport.scale(source);
    let min_zoom = if fitted_zoom > 0.0 {
        MIN_ZOOM.min(fitted_zoom)
    } else {
        MIN_ZOOM
    };
    let response = egui::Scene::new()
        .zoom_range(min_zoom..=MAX_ZOOM.max(fitted_zoom))
        .drag_pan_buttons(egui::containers::DragPanButtons::MIDDLE)
        .show(ui, &mut view.rect, |ui| {
            show_source(ui, orcvs, frame, font_family);
        })
        .response;

    // Panning or zooming moves the view off the fitted viewport and holds it
    // there; a double click hands it back. A frame that does both is a reset:
    // the double click is the later intent.
    if response.double_clicked() {
        view.adjusted = false;
    } else if response.changed() {
        view.adjusted = true;
    }

    viewport
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

        let mut console_area = Vec2::ZERO;
        let mut cell_size = 0.0;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(PALETTE.source))
            .show(root, |ui| {
                console_area = ui.available_size_before_wrap();
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
                cell_size =
                    show_source_scene(ui, orcvs, &frame, font_family, source_view).cell_size;

                ctx.request_repaint_after(self.orcvs.remaining_cursor_blink_delay());
            });

        if self.diagnostics_open {
            show_diagnostics(
                &ctx,
                &mut self.diagnostics_open,
                eframe,
                self.source_view.rect,
                console_area,
                cell_size,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::{Event, Key, Modifiers, Pos2, Rect, Shape, Vec2};
    use orcvs::app::{InputEvent, InputKey, Orcvs};
    use orcvs::glyph::Glyph;

    use crate::grid_viewport::GridViewport;
    use crate::style::{PALETTE, sector_line};
    use orcvs::grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT};

    use super::{
        ALPHABET_FIRST, ALPHABET_LAST, BLANK_GLYPHS, CELL_SIZE, DEFAULT_VIEW_SIZE, GRID_LINE_WIDTH,
        GlyphTable, SECTOR_LINE_WIDTH, SourceView, TOP_PANEL_HEIGHT, blank_glyph_index,
        frames_per_second, scene_zoom, show_source_scene, source_bounds, source_dimensions,
        translate_event,
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

    #[test]
    fn diagnostics_derive_frame_rate_and_scene_zoom_from_view_state() {
        assert_eq!(frames_per_second(0.02), Some(50.0));
        assert_eq!(frames_per_second(0.0), None);
        assert_eq!(
            scene_zoom(
                Vec2::new(800.0, 400.0),
                Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))
            ),
            Some(2.0)
        );
        assert_eq!(scene_zoom(Vec2::ZERO, Rect::ZERO), None);
    }

    ///
    /// One Render Frame, and everything it painted in paint order.
    ///
    /// The Scene nests its contents, so the shapes are flattened: what an
    /// assertion is about is the order the Source Grid was painted in, not how
    /// deeply a container wrapped it.
    ///
    fn console_pass(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) -> (GridViewport, Vec<Shape>) {
        let frame = orcvs.render_frame();
        let mut presented = None;
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                events,
                ..Default::default()
            },
            |root| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new())
                    .show(root, |ui| {
                        presented = Some(show_source_scene(
                            ui,
                            orcvs,
                            &frame,
                            &egui::FontFamily::Monospace,
                            view,
                        ));
                    });
            },
        );
        let mut painted = Vec::new();
        for clipped in &output.shapes {
            flatten(clipped.shape.clone(), &mut painted);
        }
        output.drop_without_applying_deltas();

        (
            presented.expect("the central panel showed the Source"),
            painted,
        )
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
                    .frame(egui::Frame::new().fill(PALETTE.source))
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
    fn source_bounds_are_available_before_the_first_scene_render() {
        let orcvs = Orcvs::new(32, 16);
        let (columns, rows) = source_dimensions(&orcvs.render_frame());
        let bounds = source_bounds(columns, rows);

        assert_eq!(
            bounds,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 400.0))
        );
    }

    /// The rectangles the Cells were painted at, in paint order.
    fn painted_rects(shapes: &[Shape], cell_size: f32) -> Vec<Rect> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Rect(rect) => Some(rect.rect),
                _ => None,
            })
            .filter(|rect| {
                (rect.width() - cell_size).abs() < 1e-3 && (rect.height() - cell_size).abs() < 1e-3
            })
            .collect()
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
    /// asserted here against the geometry that actually reached the Render
    /// Frame.
    ///
    /// The Cursor's own blink phase cannot be driven from a console test: it
    /// turns on a wall-clock delay held inside `orcvs`, and a seam to set it
    /// would be a test-only input cut into shipped code. What is asserted
    /// instead is the whole of what that phase could have moved — every Cell,
    /// the selected one included, occupies exactly the rectangle its Position
    /// gives it, and the Cursor's own stroke is drawn on that same rectangle
    /// rather than beside it or around it.
    ///
    #[test]
    fn the_cursor_reaches_the_paint_of_a_cell_and_never_its_geometry() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        let (viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let painted = painted_rects(&shapes, viewport.cell_size);

        // Every Cell, and then the Cursor's stroke on the one Cell that has it.
        assert_eq!(
            painted.len(),
            8 * 8 + 1,
            "painted {} Cell rects",
            painted.len()
        );
        let mut expected = Vec::new();
        for row in 0..8 {
            for column in 0..8 {
                expected.push(viewport.cell_rect(column, row));
            }
        }
        for (index, rect) in expected.iter().enumerate() {
            assert!(
                close(painted[index], *rect),
                "Cell {index} was painted at {:?} rather than {rect:?}",
                painted[index]
            );
        }
        // The selected Cell is (0, 0), and the Cursor's stroke lands on exactly
        // the rectangle that Cell already occupies.
        assert_eq!(selected_cell(&orcvs), (0, 0));
        assert!(
            close(painted[8 * 8], viewport.cell_rect(0, 0)),
            "the Cursor was painted at {:?} rather than {:?}",
            painted[8 * 8],
            viewport.cell_rect(0, 0)
        );
    }

    ///
    /// A Cell's background never paints over a Glyph, whichever Cell that Glyph
    /// belongs to. Backgrounds and Glyphs are built as two sequences and
    /// concatenated precisely so a later Cell in the row order cannot erase an
    /// earlier Cell's Glyph, and the Cursor comes after both.
    ///
    #[test]
    fn every_background_is_painted_before_every_glyph_and_the_cursor_after_both() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();
        // A Glyph in the first Cell of the Grid, so every other Cell's
        // background is built after it and would paint over it if the shapes
        // were emitted Cell by Cell.
        orcvs.write("1");
        orcvs.select(orcvs.render_frame().rows()[0][0].position());

        let (viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        let last_background = shapes
            .iter()
            .rposition(|shape| match shape {
                Shape::Rect(rect) => rect.fill.a() > 0,
                _ => false,
            })
            .expect("the Grid painted Cell backgrounds");
        let first_glyph = shapes
            .iter()
            .position(|shape| matches!(shape, Shape::Text(_)))
            .expect("the Grid painted a Glyph");
        let cursor = shapes
            .iter()
            .rposition(|shape| match shape {
                Shape::Rect(rect) => {
                    rect.fill.a() == 0 && close(rect.rect, viewport.cell_rect(0, 0))
                }
                _ => false,
            })
            .expect("the Grid painted the Cursor");

        assert!(
            last_background < first_glyph,
            "a background at {last_background} painted after the Glyph at {first_glyph}"
        );
        assert!(
            first_glyph < cursor,
            "the Cursor at {cursor} painted before the Glyph at {first_glyph}"
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
    /// The Cell border is the ordinary Grid line, and the Cursor's blink
    /// changes its colour rather than its width — which is what
    /// `cell_line_width` returned a constant for.
    ///
    #[test]
    fn a_cell_border_is_one_grid_line_wide_whatever_the_cell_is_doing() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        let (viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        // The Scene scales the stroke with everything else, so the width is
        // asserted in the Source's own points.
        let scale = viewport.cell_size / CELL_SIZE;

        for shape in &shapes {
            let Shape::Rect(rect) = shape else { continue };
            if (rect.rect.width() - viewport.cell_size).abs() >= 1e-3 || rect.stroke.width == 0.0 {
                continue;
            }
            assert!(
                (rect.stroke.width / scale - GRID_LINE_WIDTH).abs() < 1e-3,
                "a Cell border was {} points wide",
                rect.stroke.width / scale
            );
        }
    }

    ///
    /// Every case `cell_visuals` distinguishes reaches the paint unchanged:
    /// the Glyph foreground colours, the selection fill and its resting stroke,
    /// the Grid line, and the four `CursorBloom` fill and line pairs.
    ///
    /// The palette took five issues to settle and this drawing is not allowed
    /// to move it, so the assertion is against `cell_visuals` itself rather
    /// than against a second list of colours that could drift from it.
    ///
    #[test]
    fn a_painted_cell_takes_exactly_the_visuals_its_render_cell_asks_for() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();
        // Source enough to colour several Cells differently from each other.
        orcvs.select(orcvs.render_frame().rows()[3][1].position());
        for character in ["C", "4", "#", "a"] {
            orcvs.write(character);
        }

        let frame = orcvs.render_frame();
        let (viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        let backgrounds: Vec<_> = shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Rect(rect)
                    if (rect.rect.width() - viewport.cell_size).abs() < 1e-3
                        && rect.fill.a() > 0 =>
                {
                    Some(rect)
                }
                _ => None,
            })
            .collect();
        let strokes: Vec<_> = shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Rect(rect)
                    if (rect.rect.width() - viewport.cell_size).abs() < 1e-3
                        && rect.stroke.width > 0.0 =>
                {
                    Some((rect.rect, rect.stroke.color))
                }
                _ => None,
            })
            .collect();
        let glyphs: Vec<_> = shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(text) => Some(text.fallback_color),
                _ => None,
            })
            .collect();

        assert_eq!(
            backgrounds.len(),
            8 * 8,
            "the painted backgrounds do not line up with the Grid's Cells"
        );
        let mut painted_glyphs = 0;
        let mut bloomed = 0;
        for (index, cell) in frame.rows().iter().flatten().enumerate() {
            let expected = crate::style::cell_visuals(
                cell.glyph(),
                cell.cursor_bloom(),
                cell.selected(),
                cell.cursor_visible(),
            );
            let position = cell.position();
            let rect = viewport.cell_rect(position.x(), position.y());
            bloomed += usize::from(cell.cursor_bloom().is_some());

            assert_eq!(
                backgrounds[index].fill, expected.background,
                "Cell {position:?} was filled wrongly"
            );
            let stroke = strokes
                .iter()
                .find(|(painted, _)| close(*painted, rect))
                .unwrap_or_else(|| panic!("Cell {position:?} was never stroked"));
            assert_eq!(
                stroke.1, expected.border,
                "Cell {position:?} bordered wrongly"
            );

            if let Some(content) = cell.content() {
                assert_eq!(
                    glyphs[painted_glyphs], expected.foreground,
                    "the Glyph {content:?} at {position:?} was painted wrongly"
                );
                painted_glyphs += 1;
            }
        }
        assert_eq!(painted_glyphs, 4, "the written Source was not painted");
        assert_eq!(glyphs.len(), painted_glyphs, "a blank Cell painted a Glyph");
        assert!(bloomed > 0, "no Cell took a CursorBloom");
    }

    ///
    /// Sector seams reach the Render Frame with the geometry and the
    /// `sector_line` attenuation the Render Cell asks for, and a selected Cell
    /// carries none.
    ///
    /// The Grid is 16 Cells square because the default Marker spacing is 8: an
    /// 8x8 Grid has no column or row that is a non-zero multiple of it, so every
    /// `sector_left_strength` and `sector_top_strength` in one is `None` and the
    /// seams the console paints would go unexercised by a Grid that size.
    ///
    #[test]
    fn sector_seams_are_painted_where_the_render_frame_asks_and_never_on_the_cursor() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = Orcvs::new(16, 16);
        let mut view = SourceView::default();
        // The Cursor goes on a Cell that would otherwise carry both seams, so
        // the suppression is asserted against a Cell that has something to
        // suppress.
        let corner = orcvs.render_frame().rows()[8][8].position();
        orcvs.select(corner);

        let frame = orcvs.render_frame();
        let (viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let scale = viewport.cell_size / CELL_SIZE;

        let painted: Vec<_> = shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::LineSegment { points, stroke } => Some((*points, *stroke)),
                _ => None,
            })
            .collect();

        let mut expected = 0;
        for cell in frame.rows().iter().flatten() {
            let position = cell.position();
            let rect = viewport.cell_rect(position.x(), position.y());
            let seams = [
                (
                    cell.sector_left_strength(),
                    [rect.left_top(), rect.left_bottom()],
                ),
                (
                    cell.sector_top_strength(),
                    [rect.left_top(), rect.right_top()],
                ),
            ];

            for (strength, ends) in seams {
                let Some(strength) = strength else { continue };
                let found = painted.iter().find(|(points, _)| {
                    (points[0] - ends[0]).length() < 1e-3 && (points[1] - ends[1]).length() < 1e-3
                });

                if cell.selected() {
                    assert!(
                        found.is_none(),
                        "the Cursor at {position:?} was crossed by a sector seam"
                    );
                    continue;
                }
                let (_, stroke) =
                    found.unwrap_or_else(|| panic!("Cell {position:?} painted no sector seam"));
                assert_eq!(
                    stroke.color,
                    sector_line(strength),
                    "the seam at {position:?} did not take its attenuated colour"
                );
                assert!(
                    (stroke.width / scale - SECTOR_LINE_WIDTH).abs() < 1e-3,
                    "the seam at {position:?} was {} points wide",
                    stroke.width / scale
                );
                expected += 1;
            }
        }

        assert!(
            expected > 0,
            "the Grid asked for no sector seams, so nothing was asserted"
        );
        assert_eq!(
            painted.len(),
            expected,
            "the console painted sector seams the Render Frame did not ask for"
        );
    }

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
    /// A middle drag that starts on a Cell pans the Scene.
    ///
    /// This is what `Sense::CLICK` on the Grid rectangle buys. Within one layer
    /// a later-registered child wins the click tie and would win the drag tie
    /// too if it sensed drag, and the Grid is registered after the Scene's own
    /// pan response. Sensing clicks alone is what leaves the drag to the Scene,
    /// and the drag has to be started *over the Grid* to assert it: the
    /// letterboxing is territory the Grid never covered.
    ///
    #[test]
    fn a_middle_drag_that_starts_on_a_cell_still_pans_the_scene() {
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
        let fitted = view.rect;

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
            "a middle drag over a Cell did not pan the Scene"
        );
        assert_ne!(view.rect, fitted, "the pan did not move the view");
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
        let wide_region = view.rect;
        let viewport = console_frame(&ctx, tall, Vec::new(), &mut orcvs, &mut view);

        assert!(!view.adjusted, "an untouched view was pinned");
        assert_ne!(view.rect, wide_region, "the resize did not re-fit");
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
        let pinned = view.rect;
        console_frame(&ctx, tall, Vec::new(), &mut orcvs, &mut view);

        assert_eq!(view.rect, pinned, "the resize discarded the viewer's zoom");
    }

    #[test]
    fn a_double_click_unpins_the_view_and_hands_it_back_to_the_fit() {
        let ctx = egui::Context::default();
        let wide = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = Orcvs::new(8, 8);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
        let fitted = view.rect;
        let over_the_scene = letterboxing(wide, viewport.rect).expect("a wide console letterboxes");
        console_frame(&ctx, wide, zoom_at(over_the_scene), &mut orcvs, &mut view);
        assert!(view.adjusted, "zooming did not pin the view");

        double_click(&ctx, wide, over_the_scene, &mut orcvs, &mut view);
        assert!(!view.adjusted, "the double click did not unpin the view");

        console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.rect, fitted, "the view did not return to the fit");
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
