use egui::Color32;

pub const DEFAULT_FONT_SIZE: f32 = 25.0;
pub const DEFAULT_COL_COUNT: usize = 10;
pub const DEFAULT_ROW_COUNT: usize = 10;
// pub const DEFAULT_SCALE: f32 = 1.0;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    grid: Grid,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let grid = Grid::default();
        Self { grid }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Grid {
    items: [[Glyph; DEFAULT_COL_COUNT]; DEFAULT_ROW_COUNT],
}

impl Default for Grid {
    fn default() -> Self {
        // create multidimensional array of Glyphs using core::array::from_fn(|i| i)

        let mut items = [[Glyph::default(); DEFAULT_COL_COUNT]; DEFAULT_ROW_COUNT];

        for y in 0..=DEFAULT_ROW_COUNT - 1 {
            for x in 0..=DEFAULT_COL_COUNT - 1 {
                let glyph = Glyph {
                    char: '.',
                    x,
                    y,
                    state: GlyphState::Default,
                };

                items[y][x] = glyph;
            }
        }

        Self { items }
    }
}

impl Grid {
    fn get_glyph_at(&self, x: usize, y: usize) -> &Glyph {
        &self.items[y][x]
    }

    fn set_glyph_at(&mut self, x: usize, y: usize, glyph: Glyph) {
        self.items[y][x] = glyph;
    }
}

#[derive(Copy, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Glyph {
    char: char,
    x: usize,
    y: usize,
    state: GlyphState,
}

#[derive(Copy, Clone, Default, serde::Deserialize, serde::Serialize)]
pub enum GlyphState {
    #[default]
    Default,
    Selected,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

// let mut button_states: Vec<Vec<bool>> = vec![vec![false; DEFAULT_COL_COUNT]; DEFAULT_ROW_COUNT];

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        let top_panel = egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .min_height(32.0);

        let bottom_panel = egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .min_height(0.0);

        let central_panel = egui::CentralPanel::default().show(ctx, |ui| {});

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

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        let font_id = egui::FontId::monospace(DEFAULT_FONT_SIZE);

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("edit");
            ui.spacing_mut().item_spacing = egui::Vec2::splat(2.0);

            for y in 0..=DEFAULT_ROW_COUNT - 1 {
                ui.horizontal(|ui| {
                    for x in 0..=DEFAULT_COL_COUNT - 1 {
                        let glyph = self.grid.get_glyph_at(x, y);

                        let mut button_text =
                            egui::RichText::new(glyph.char.to_string()).font(font_id.clone());
                        let button = egui::Button::new(button_text.clone()).frame(false);

                        if ui.add(button).clicked() {
                            button_text = button_text.color(egui::Color32::RED);
                            // Change text color to red
                        }

                        ui.add(egui::Label::new(button_text));

                        // let button = egui::Button::new(
                        //     egui::RichText::new(".".to_string())
                        //         .font(font_id.clone()),
                        // )
                        // .frame(false);

                        // let button = egui::Button::new(
                        //     egui::RichText::new(".".to_string())
                        //         .font(font_id.clone()),
                        // )
                        // .frame(false);

                        // if ui.add(button).clicked() {
                        //     print!("click");
                        //     ui.style_mut()
                        //         .visuals
                        //         .widgets
                        //         .inactive
                        //         .bg_fill = egui::Color32::RED;
                        //     // button.fill(Color32::from_rgb(255, 0, 0));
                        //     // Set button background color to red
                        // }
                    }
                });
            }
        });
    }
}
