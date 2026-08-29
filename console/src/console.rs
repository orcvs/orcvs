use std::time::Duration;

use egui::{EventFilter, FontId, Pos2, Rect, Stroke, Vec2};

use crate::{
    app::App,
    glyph::GlyphString,
    grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT},
    opts::{DEFAULT_CURSOR_DELAY, DEFAULT_FONT_SIZE},
    render_frame::RenderFrame,
    style::{PALETTE, cell_visuals, sector_line, style},
};

const CELL_SIZE: f32 = 25.0;
const CELL_PADDING: f32 = 0.5;
const GRID_LINE_WIDTH: f32 = 0.5;
const SECTOR_LINE_WIDTH: f32 = 0.75;
const INITIAL_ZOOM: f32 = 1.0;
const INITIAL_SOURCE_X_OFFSET: f32 = 15.0;

fn top_right_source_view(
    source: Rect,
    viewport_size: Vec2,
    zoom: f32,
    source_x_offset: f32,
) -> Rect {
    let view_size = viewport_size / zoom;
    Rect::from_min_size(
        egui::pos2(source.right() - view_size.x - source_x_offset, source.top()),
        view_size,
    )
}

fn source_bounds(frame: &RenderFrame) -> Rect {
    let rows = frame.rows();
    let col_count = rows
        .first()
        .expect("a Render Frame contains at least one row")
        .len();
    debug_assert!(
        rows.iter().all(|row| row.len() == col_count),
        "a Render Frame has the Grid's fixed rectangular shape"
    );

    Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(col_count as f32, rows.len() as f32) * CELL_SIZE,
    )
}

/// ConsoleApp wraps the inner App
/// ConsoleApp handles the egui presentation concerns
/// App owns the underlying logic
///
pub struct Console {
    app: App,
    font_family: egui::FontFamily,
    /// The visible region of the egui Scene containing the Source.
    source_view_rect: Rect,
    diagnostics_open: bool,
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

        let mut app = App::new(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT);
        app.refresh_midi_destinations();
        Self {
            app,
            font_family: FontId::monospace(DEFAULT_FONT_SIZE).family,
            source_view_rect: Rect::ZERO,
            diagnostics_open: false,
        }
    }
}

fn frames_per_second(frame_time: f32) -> Option<f32> {
    frame_time.is_normal().then(|| frame_time.recip())
}

fn scene_zoom(viewport_size: Vec2, source_view_rect: Rect) -> Option<f32> {
    (source_view_rect.is_positive() && viewport_size.x > 0.0 && viewport_size.y > 0.0).then(|| {
        (viewport_size.x / source_view_rect.width())
            .min(viewport_size.y / source_view_rect.height())
    })
}

fn next_repaint_after(cursor_delay_ms: u64) -> Duration {
    Duration::from_millis(cursor_delay_ms)
}

fn show_diagnostics(
    ctx: &egui::Context,
    open: &mut bool,
    frame: &eframe::Frame,
    source_view_rect: Rect,
    viewport_size: Vec2,
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

                    ui.label("Source zoom");
                    ui.monospace(
                        scene_zoom(viewport_size, source_view_rect)
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
    app: &mut App,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
) -> Rect {
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
                    cell.content(),
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
                    app.select(cell.position());
                }
            }
        });
    }

    ui.min_rect()
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
        self.app.observe_playback();
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
                        self.app.refresh_midi_destinations();
                    }
                    let selected = self.app.selected_midi_destination_id();
                    for destination in self.app.midi_destinations().to_vec() {
                        let is_selected = selected.as_ref() == Some(&destination.id);
                        if ui.selectable_label(is_selected, destination.name).clicked() {
                            self.app.select_midi_destination(&destination.id);
                        }
                    }
                    if self.app.midi_destinations().is_empty() {
                        ui.label("No MIDI destinations found");
                    }
                    if let Some(status) = self.app.midi_status() {
                        ui.separator();
                        ui.colored_label(ui.visuals().error_fg_color, status);
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.diagnostics_open, "Diagnostics");
                });
                // ui.label(format!("HELLO"));
                // egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        let event_filter = EventFilter {
            tab: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            escape: true,
        };

        let events = ctx.input(|i| i.filtered_events(&event_filter));
        self.app.event_handler(events);
        self.app.advance_cursor_blink();
        let frame = self.app.render_frame();

        let mut viewport_size = Vec2::ZERO;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(PALETTE.source))
            .show(root, |ui| {
                viewport_size = ui.available_size_before_wrap();
                let scene = egui::Scene::new()
                    .zoom_range(0.25..=2.0)
                    .drag_pan_buttons(egui::containers::DragPanButtons::MIDDLE);
                let mut source_rect = Rect::NAN;
                let Console {
                    app,
                    font_family,
                    source_view_rect,
                    diagnostics_open: _,
                } = self;
                if *source_view_rect == Rect::ZERO {
                    *source_view_rect = top_right_source_view(
                        source_bounds(&frame),
                        viewport_size,
                        INITIAL_ZOOM,
                        INITIAL_SOURCE_X_OFFSET,
                    );
                }
                let response = scene
                    .show(ui, source_view_rect, |ui| {
                        source_rect = show_source(ui, app, &frame, font_family);
                    })
                    .response;

                if response.double_clicked() && source_rect.is_finite() {
                    *source_view_rect = source_rect;
                }

                ctx.request_repaint_after(next_repaint_after(DEFAULT_CURSOR_DELAY));
            });

        if self.diagnostics_open {
            show_diagnostics(
                &ctx,
                &mut self.diagnostics_open,
                eframe,
                self.source_view_rect,
                viewport_size,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::{Pos2, Rect, Vec2};
    use std::time::Duration;

    use crate::app::App;

    use super::{frames_per_second, scene_zoom, source_bounds, top_right_source_view};

    #[test]
    fn idle_repaint_policy_waits_for_the_cursor_blink_deadline() {
        assert_eq!(super::next_repaint_after(500), Duration::from_millis(500));
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

    #[test]
    fn initial_source_view_is_one_to_one_and_offset_right() {
        let source = Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(800.0, 800.0));
        let view = top_right_source_view(source, Vec2::new(600.0, 400.0), 1.0, 15.0);

        assert_eq!(view.size(), Vec2::new(600.0, 400.0));
        assert_eq!(view.top(), source.top());
        assert_eq!(source.right() - view.right(), 15.0);
    }

    #[test]
    fn source_bounds_are_available_before_the_first_scene_render() {
        let app = App::new(32, 16);
        let bounds = source_bounds(&app.render_frame());

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
                .font(egui::FontId::monospace(crate::opts::DEFAULT_FONT_SIZE));
            button_size = ui
                .add_sized(Vec2::splat(super::CELL_SIZE), egui::Button::new(text))
                .rect
                .size();
        });
        output.drop_without_applying_deltas();

        assert_eq!(button_size, Vec2::splat(super::CELL_SIZE));
    }

    #[test]
    fn caret_phase_does_not_change_cell_border_geometry() {
        assert_eq!(super::cell_line_width(false, false), super::GRID_LINE_WIDTH);
        assert_eq!(super::cell_line_width(true, false), super::GRID_LINE_WIDTH);
        assert_eq!(super::cell_line_width(true, true), super::GRID_LINE_WIDTH);
    }
}
