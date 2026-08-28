use egui::{Align, EventFilter, FontId, Layout, Rect, Stroke, UiBuilder, Vec2};

use crate::{
    app::App,
    glyph::GlyphString,
    grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT},
    opts::DEFAULT_FONT_SIZE,
    style::{PALETTE, cell_visuals, style},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ConsoleLayout {
    pub(crate) viewport: Rect,
    pub(crate) grid: Rect,
    pub(crate) cell_size: Vec2,
}

impl ConsoleLayout {
    pub(crate) fn new(available: Rect, cols: usize, rows: usize) -> Self {
        assert!(cols > 0, "a Grid must contain at least one column");
        assert!(rows > 0, "a Grid must contain at least one row");
        let side = available.width().min(available.height()).max(0.0);
        let viewport = Rect::from_center_size(available.center(), Vec2::splat(side));
        let cell_side = (side / cols as f32).min(side / rows as f32);
        let grid = Rect::from_center_size(
            viewport.center(),
            Vec2::new(cell_side * cols as f32, cell_side * rows as f32),
        );
        Self {
            viewport,
            grid,
            cell_size: Vec2::splat(cell_side),
        }
    }
}

/// ConsoleApp wraps the inner App
/// ConsoleApp handles the egui presentation concerns
/// App owns the underlying logic
///
pub struct Console {
    app: App,
    font_family: egui::FontFamily,
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
            let available = ui.available_rect_before_wrap();
            let rows = frame.rows().len();
            let cols = frame.rows().first().map_or(0, Vec::len);
            let layout = ConsoleLayout::new(available, cols, rows);
            ui.painter()
                .rect_filled(layout.viewport, 0.0, PALETTE.source);

            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(layout.grid)
                    .layout(Layout::top_down(Align::Min)),
                |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    ui.spacing_mut().button_padding = Vec2::ZERO;
                    ui.spacing_mut().interact_size = Vec2::ZERO;
                    for row in frame.rows() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::ZERO;
                            ui.spacing_mut().button_padding = Vec2::ZERO;
                            ui.spacing_mut().interact_size = Vec2::ZERO;
                            for cell in row {
                                let glyph = GlyphString::new(
                                    cell.content().map(|content| content.to_string()),
                                    cell.glyph(),
                                );
                                let selected = cell.selected();
                                let visuals = cell_visuals(
                                    cell.glyph(),
                                    cell.content(),
                                    selected,
                                    cell.cursor_visible(),
                                );

                                let button_text = egui::RichText::new(glyph.to_string())
                                    .font(FontId::new(
                                        (layout.cell_size.y * 0.65).min(DEFAULT_FONT_SIZE),
                                        self.font_family.clone(),
                                    ))
                                    .color(visuals.foreground);

                                let button = egui::Button::new(button_text)
                                    .fill(visuals.background)
                                    .stroke(Stroke::new(1.0, visuals.border))
                                    .corner_radius(0.0)
                                    .frame(true);

                                if ui.add_sized(layout.cell_size, button).clicked() {
                                    self.app.select(cell.position());
                                }
                            }
                        });
                    }
                },
            );

            ctx.request_repaint();
        });
    }
}

#[cfg(test)]
mod tests {
    use egui::{Pos2, Rect, Vec2};

    use super::ConsoleLayout;

    #[test]
    fn source_grid_viewport_is_square_and_cells_cannot_stretch() {
        for available_size in [Vec2::new(1200.0, 600.0), Vec2::new(600.0, 1200.0)] {
            let available = Rect::from_min_size(Pos2::ZERO, available_size);
            let layout = ConsoleLayout::new(available, 32, 32);

            assert_eq!(layout.viewport.width(), layout.viewport.height());
            assert_eq!(layout.cell_size.x, layout.cell_size.y);
            assert_eq!(layout.viewport.width(), 600.0);
            assert_eq!(layout.cell_size.x, 600.0 / 32.0);
        }
    }

    #[test]
    fn surplus_rectangular_space_is_centred_as_letterboxing() {
        let wide = ConsoleLayout::new(
            Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(1000.0, 600.0)),
            32,
            32,
        );
        assert_eq!(wide.viewport.min, Pos2::new(210.0, 20.0));
        assert_eq!(wide.viewport.max, Pos2::new(810.0, 620.0));

        let tall = ConsoleLayout::new(
            Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(600.0, 1000.0)),
            32,
            32,
        );
        assert_eq!(tall.viewport.min, Pos2::new(10.0, 220.0));
        assert_eq!(tall.viewport.max, Pos2::new(610.0, 820.0));
    }

    #[test]
    fn rectangular_grid_uses_square_cells_centred_inside_square_viewport() {
        let available = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let layout = ConsoleLayout::new(available, 4, 2);

        assert_eq!(layout.viewport.size(), Vec2::splat(600.0));
        assert_eq!(layout.cell_size, Vec2::splat(150.0));
        assert_eq!(layout.grid.size(), Vec2::new(600.0, 300.0));
        assert_eq!(layout.grid.center(), layout.viewport.center());
    }
}
