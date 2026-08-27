use egui::{Color32, CornerRadius, EventFilter, FontId, Vec2};

use crate::{
    app::App,
    glyph::{Glyph, GlyphString},
    grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT},
    opts::DEFAULT_FONT_SIZE,
    style::style,
    Color,
};

const DEFAULT_GLYPH_FONT_COLOR: Color32 = Color::rgb(164, 166, 169).build();

struct GlyphStyle {
    bg_fill: Color32,
    stroke_color: Color32,
    font_color: Color32,
}

const CURSOR_VISUALS: GlyphStyle = GlyphStyle {
    bg_fill: Color32::TRANSPARENT,
    stroke_color: Color::rgb(192, 222, 255).build(),
    font_color: DEFAULT_GLYPH_FONT_COLOR,
};

fn glyph_style(glyph: Glyph, selected: bool) -> GlyphStyle {
    let default = GlyphStyle {
        bg_fill: Color32::TRANSPARENT,
        stroke_color: Color32::TRANSPARENT,
        font_color: DEFAULT_GLYPH_FONT_COLOR,
    };
    let default_selected = GlyphStyle {
        bg_fill: Color::rgb(0, 92, 128).build(),
        stroke_color: DEFAULT_GLYPH_FONT_COLOR,
        font_color: DEFAULT_GLYPH_FONT_COLOR,
    };

    match (glyph, selected) {
        (Glyph::Function, true) => GlyphStyle {
            bg_fill: Color::rgb(200, 75, 255).build(),
            ..default_selected
        },
        (Glyph::Function, false) => GlyphStyle {
            bg_fill: Color::rgb(255, 0, 230).build(),
            stroke_color: Color::rgb(0, 0, 0).build(),
            font_color: Color::rgb(255, 255, 255).build(),
        },
        (Glyph::Number | Glyph::Note | Glyph::Char, true)
        | (Glyph::Space | Glyph::Marker | Glyph::Highlight, true) => default_selected,
        (Glyph::Note, false) => GlyphStyle {
            bg_fill: Color::rgb(25, 150, 135).build(),
            stroke_color: Color::rgb(33, 33, 33).build(),
            font_color: Color::rgb(200, 200, 200).build(),
        },
        (Glyph::Char, false) => GlyphStyle {
            bg_fill: Color::rgb(125, 225, 220).build(),
            stroke_color: Color::rgb(33, 33, 33).build(),
            font_color: Color::rgb(200, 200, 200).build(),
        },
        (Glyph::Number | Glyph::Space | Glyph::Marker | Glyph::Highlight, false) => default,
    }
}

/// ConsoleApp wraps the inner App
/// ConsoleApp handles the egui presentation concerns
/// App owns the underlying logic
///
pub struct Console {
    app: App,
    font_id: FontId,
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
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        app.refresh_midi_destinations();
        Self {
            app,
            font_id: FontId::monospace(DEFAULT_FONT_SIZE),
        }
    }
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
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            ui.spacing_mut().button_padding = Vec2::splat(2.5);

            // button background colour
            // ui.style_mut().visuals.widgets.inactive.weak_bg_fill = DEFAULT_VISUAL_BG_COLOR;
            ui.style_mut().visuals.widgets.inactive.corner_radius = CornerRadius::default();

            for row in frame.rows() {
                ui.horizontal(|ui| {
                    for cell in row {
                        let glyph = GlyphString::new(
                            cell.content().map(|content| content.to_string()),
                            cell.glyph(),
                        );
                        let selected = cell.selected();

                        let visuals = glyph_style(cell.glyph(), selected);

                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = visuals.bg_fill;

                        // font
                        ui.style_mut().visuals.widgets.inactive.fg_stroke.color =
                            visuals.font_color;

                        // frame stroke
                        ui.style_mut().visuals.widgets.inactive.bg_stroke.color =
                            visuals.stroke_color;

                        ui.style_mut().visuals.widgets.inactive.bg_fill = visuals.stroke_color;

                        if selected {
                            if cell.cursor_visible() {
                                ui.style_mut().visuals.selection.bg_fill = CURSOR_VISUALS.bg_fill;
                                ui.style_mut().visuals.selection.stroke.color =
                                    CURSOR_VISUALS.stroke_color;
                            } else {
                                ui.style_mut().visuals.selection.bg_fill = visuals.bg_fill;
                                ui.style_mut().visuals.selection.stroke.color =
                                    visuals.stroke_color;
                            }
                        }

                        let button_text =
                            egui::RichText::new(glyph.to_string()).font(self.font_id.clone());
                        // .background_color(bg_color);
                        // .color(text_color);
                        // .extra_letter_spacing(0.4);

                        let button = egui::Button::new(button_text)
                            // .stroke(stroke)
                            // .small()
                            .selected(selected)
                            .frame(true);

                        if ui.add(button).clicked() {
                            self.app.select(cell.position());
                        }
                    }
                });
            }

            ctx.request_repaint();
        });
    }
}
