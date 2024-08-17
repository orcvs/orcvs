use egui::{Color32, EventFilter, Rounding, Vec2};

use crate::{
    app::{App, DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT},
    glyph::{Glyph, GlyphStyle, CURSOR_VISUALS},
    style::style,
    Color,
};

/// ConsoleApp wraps the inner App
/// ConsoleApp handles the egui presentation concerns
/// App owns the underlying logic
///
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Console {
    app: App,
}

impl Console {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let style = style();
        cc.egui_ctx.set_style(style);

        Self {
            app: App::new(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT),
        }
    }
}

impl eframe::App for Console {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let top_panel = egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .min_height(32.0);

        // let _bottom_panel = egui::TopBottomPanel::bottom("bottom_panel")
        //     .resizable(false)
        //     .min_height(0.0);

        top_panel.show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            ui.spacing_mut().button_padding = Vec2::splat(2.5);

            // button background colour
            // ui.style_mut().visuals.widgets.inactive.weak_bg_fill = DEFAULT_VISUAL_BG_COLOR;
            ui.style_mut().visuals.widgets.inactive.rounding = Rounding::default();

            for y in 0..self.app.opts.cols {
                ui.horizontal(|ui| {
                    for x in 0..self.app.opts.rows {
                        let (s, g) = self.app.get_at(x, y);
                        let selected = self.app.cursor.is_at(x, y);

                        let visuals = g.style(selected);

                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = visuals.bg_fill;

                        // font
                        ui.style_mut().visuals.widgets.inactive.fg_stroke.color =
                            visuals.font_color;

                        // frame stroke
                        ui.style_mut().visuals.widgets.inactive.bg_stroke.color =
                            visuals.stroke_color;

                        ui.style_mut().visuals.widgets.inactive.bg_fill = visuals.stroke_color;

                        if selected {
                            self.app.cursor.blink();

                            if self.app.cursor.on {
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
                            egui::RichText::new(s).font(self.app.opts.font_id.clone());
                        // .background_color(bg_color);
                        // .color(text_color);
                        // .extra_letter_spacing(0.4);

                        let button = egui::Button::new(button_text)
                            // .stroke(stroke)
                            // .small()
                            .selected(selected)
                            .frame(true);

                        if ui.add(button).clicked() {
                            self.app.cursor.select_at(x, y);
                        }
                    }
                });
            }

            ctx.request_repaint();
        });
    }
}
