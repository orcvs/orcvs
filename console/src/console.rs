use egui::{EventFilter, FontId, Rect, Stroke, Vec2};

use crate::{
    app::App,
    glyph::GlyphString,
    grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT},
    opts::DEFAULT_FONT_SIZE,
    render_frame::RenderFrame,
    style::{PALETTE, cell_visuals, style},
};

const CELL_SIZE: f32 = 25.0;

/// ConsoleApp wraps the inner App
/// ConsoleApp handles the egui presentation concerns
/// App owns the underlying logic
///
pub struct Console {
    app: App,
    font_family: egui::FontFamily,
    /// The visible region of the egui Scene containing the Source.
    source_view_rect: Rect,
}

impl Console {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let style = style();
        cc.egui_ctx.set_style_of(egui::Theme::Dark, style);
        cc.egui_ctx.set_theme(egui::Theme::Dark);

        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();
        let key = "ServerMono";
        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            key.to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/ServerMono-Regular.otf")).into(),
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
        }
    }
}

fn show_source(
    ui: &mut egui::Ui,
    app: &mut App,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
) -> Rect {
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    ui.spacing_mut().button_padding = Vec2::splat(2.5);
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
                    cell.selected(),
                    cell.cursor_visible(),
                );
                let button_text = egui::RichText::new(glyph.to_string())
                    .font(FontId::new(DEFAULT_FONT_SIZE, font_family.clone()))
                    .color(visuals.foreground);
                let button = egui::Button::new(button_text)
                    .fill(visuals.background)
                    .stroke(Stroke::new(1.0, visuals.border))
                    .corner_radius(0.0)
                    .frame(true);

                if ui.add_sized(Vec2::splat(CELL_SIZE), button).clicked() {
                    app.select(cell.position());
                }
            }
        });
    }

    ui.min_rect()
}

impl eframe::App for Console {
    // Called by the framework to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }
    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

        egui::CentralPanel::default().show(root, |ui| {
            ui.painter()
                .rect_filled(ui.available_rect_before_wrap(), 0.0, PALETTE.source);
            let scene = egui::Scene::new()
                .zoom_range(0.25..=2.0)
                .drag_pan_buttons(egui::containers::DragPanButtons::MIDDLE);
            let mut source_rect = Rect::NAN;
            let Console {
                app,
                font_family,
                source_view_rect,
            } = self;
            let response = scene
                .show(ui, source_view_rect, |ui| {
                    source_rect = show_source(ui, app, &frame, font_family);
                })
                .response;

            if response.double_clicked() && source_rect.is_finite() {
                *source_view_rect = source_rect;
            }

            ctx.request_repaint();
        });
    }
}
