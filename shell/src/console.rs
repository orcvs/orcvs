use egui::{Event, EventFilter, FontId, Key, Pos2, Rect, Stroke, Vec2};

use crate::grid_viewport::{GridViewport, grid_viewport};
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
use crate::midi::MidiDeviceSelection;
use crate::style::{PALETTE, cell_visuals, sector_line, style};
use orcvs::{
    app::{InputEvent, InputKey, Orcvs},
    glyph::GlyphString,
    grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT},
    opts::{Bpm, DEFAULT_FONT_SIZE},
    render_frame::RenderFrame,
};
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
use orcvs::{midi::MidiOutputAdapter, native_midi::MidirBackend};

const CELL_SIZE: f32 = 25.0;
const CELL_PADDING: f32 = 0.5;
const GRID_LINE_WIDTH: f32 = 0.5;
const SECTOR_LINE_WIDTH: f32 = 0.75;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 2.0;

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
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    midi: MidiDeviceSelection<MidirBackend>,
    font_family: egui::FontFamily,
    source_view: SourceView,
    diagnostics_open: bool,
    tempo_edit: TempoEdit,
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

        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        let orcvs = Orcvs::with_output_adapter(
            DEFAULT_COL_COUNT,
            DEFAULT_ROW_COUNT,
            MidiOutputAdapter::new(MidirBackend),
        );
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        let mut midi = MidiDeviceSelection::new(orcvs.midi_selection_handle());
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        midi.refresh_destinations();
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let orcvs = Orcvs::new(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT);
        Self {
            orcvs,
            #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
            midi,
            font_family: FontId::monospace(DEFAULT_FONT_SIZE).family,
            source_view: SourceView::default(),
            diagnostics_open: false,
            tempo_edit: TempoEdit::default(),
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

fn show_source(
    ui: &mut egui::Ui,
    orcvs: &mut Orcvs,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
) {
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    ui.spacing_mut().button_padding = Vec2::splat(CELL_PADDING);
    ui.spacing_mut().interact_size = Vec2::ZERO;
    for row in frame.rows() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for cell in row {
                let glyph = GlyphString::new(
                    cell.content().map(|content| content.to_string()),
                    cell.glyph(),
                );
                let visuals = cell_visuals(
                    cell.glyph(),
                    cell.cursor_bloom(),
                    cell.selected(),
                    cell.cursor_visible(),
                );
                let button_text = egui::RichText::new(glyph.to_string())
                    .font(FontId::new(DEFAULT_FONT_SIZE, font_family.clone()))
                    .color(visuals.foreground);
                let line_width = cell_line_width(cell.selected(), cell.cursor_visible());
                let button = egui::Button::new(button_text)
                    .fill(visuals.background)
                    .stroke(Stroke::new(line_width, visuals.border))
                    .corner_radius(0.0)
                    .frame(true);

                let response = ui.add_sized(Vec2::splat(CELL_SIZE), button);
                if !cell.selected() {
                    if let Some(strength) = cell.sector_left_strength() {
                        ui.painter().line_segment(
                            [response.rect.left_top(), response.rect.left_bottom()],
                            Stroke::new(SECTOR_LINE_WIDTH, sector_line(strength)),
                        );
                    }
                    if let Some(strength) = cell.sector_top_strength() {
                        ui.painter().line_segment(
                            [response.rect.left_top(), response.rect.right_top()],
                            Stroke::new(SECTOR_LINE_WIDTH, sector_line(strength)),
                        );
                    }
                }

                if response.clicked() {
                    orcvs.select(cell.position());
                }
            }
        });
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

fn cell_line_width(_selected: bool, _cursor_visible: bool) -> f32 {
    GRID_LINE_WIDTH
}

impl eframe::App for Console {
    // Called by the framework to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }
    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, root: &mut egui::Ui, eframe: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let playback_diagnostics = self.orcvs.observe_playback();
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        self.midi.observe_diagnostics(playback_diagnostics);
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        for diagnostic in &playback_diagnostics {
            if let Some(message) = crate::diagnostics::failure_message(diagnostic) {
                tracing::error!("Playback failure: {message}");
            }
        }
        let top_panel = egui::Panel::top("top_panel").resizable(true).min_size(32.0);

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
                #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
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
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.diagnostics_open, "Diagnostics");
                });
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
                    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
                        midi: _,
                    font_family,
                    source_view,
                    diagnostics_open: _,
                    tempo_edit: _,
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
    use egui::{Event, Key, Modifiers, Pos2, Rect, Vec2};
    use orcvs::app::{InputEvent, InputKey, Orcvs};

    use crate::grid_viewport::GridViewport;

    use super::{
        SourceView, frames_per_second, scene_zoom, show_source_scene, source_bounds,
        source_dimensions, translate_event,
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

    fn console_frame(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) -> GridViewport {
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
        output.drop_without_applying_deltas();

        presented.expect("the central panel showed the Source")
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

    #[test]
    fn glyph_button_fits_the_fixed_cell() {
        let ctx = egui::Context::default();
        let key = "MonaspaceNeon";
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            key.to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/MonaspaceNeon-Regular.otf"))
                .into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, key.to_owned());
        ctx.set_fonts(fonts);

        let mut button_size = Vec2::ZERO;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.spacing_mut().button_padding = Vec2::splat(super::CELL_PADDING);
            let text = egui::RichText::new("+")
                .font(egui::FontId::monospace(orcvs::opts::DEFAULT_FONT_SIZE));
            button_size = ui
                .add_sized(Vec2::splat(super::CELL_SIZE), egui::Button::new(text))
                .rect
                .size();
        });
        output.drop_without_applying_deltas();

        assert_eq!(button_size, Vec2::splat(super::CELL_SIZE));
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

    #[test]
    fn caret_phase_does_not_change_cell_border_geometry() {
        assert_eq!(super::cell_line_width(false, false), super::GRID_LINE_WIDTH);
        assert_eq!(super::cell_line_width(true, false), super::GRID_LINE_WIDTH);
        assert_eq!(super::cell_line_width(true, true), super::GRID_LINE_WIDTH);
    }
}
