use egui::{Color32, Event, EventFilter};

use crate::source::Source;
pub const DEFAULT_FONT_SIZE: f32 = 23.0;
pub const DEFAULT_COL_COUNT: usize = 4;
pub const DEFAULT_ROW_COUNT: usize = 1;
// pub const DEFAULT_SCALE: f32 = 1.0;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ConsoleApp {
    src: Source,
    selected: Option<(usize, usize)>,
    mode: Mode,
    cols: usize,
    rows: usize,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
pub enum Mode {
    Insert,
    Command,
}

impl Default for ConsoleApp {
    fn default() -> Self {
        let src = Source::new(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT);
        Self {
            src,
            selected: None,
            mode: Mode::Insert,
            cols: DEFAULT_COL_COUNT,
            rows: DEFAULT_ROW_COUNT,
        }
    }
}

impl ConsoleApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        ConsoleApp::default()
    }

    #[inline(always)]
    #[must_use]
    pub fn is_selected(&self, x: usize, y: usize) -> bool {
        self.selected == Some((x, y))
    }

    #[inline(always)]
    pub fn select(&mut self, x: usize, y: usize) {
        self.selected = Some((x, y));
    }

    #[inline(always)]
    pub fn select_next(&mut self) {
        if let Some((x, y)) = self.selected {
            let x = std::cmp::min(x + 1, self.cols - 1);
            self.selected = Some((x, y));
        }
    }

    #[inline(always)]
    pub fn deselect(&mut self) {
        self.selected = None;
    }
}

#[derive(Debug, Clone, Copy)]
enum Glyph {
    Any,
    Function,
    // Note,
    Number,
    // String,
}

const FUNCTION_ID: [Glyph; 4] = [
    Glyph::Function,
    Glyph::Function,
    Glyph::Number,
    Glyph::Number,
];

const FUNCTION_PLAY: [Glyph; 5] = [
    Glyph::Function,
    Glyph::Function,
    Glyph::Number,
    Glyph::Number,
    Glyph::Number,
];

/// ID Number
///
/// Play Number 1, Number, Note
///
///
/// src IDAA
///     |
///     idx
///
///     IDAA
///       |
///       pos

impl eframe::App for ConsoleApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        let font_id = egui::FontId::monospace(DEFAULT_FONT_SIZE);

        let top_panel = egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .min_height(32.0);

        let _bottom_panel = egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .min_height(0.0);

        let _central_panel = egui::CentralPanel::default().show(ctx, |_ui| {});

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

        let event_filter = EventFilter::default();
        let events = ctx.input(|i| i.filtered_events(&event_filter));

        for event in &events {
            match event {
                Event::Text(text_to_insert) => {
                    // info!("text_to_insert: {}", text_to_insert);

                    // This is all probably a very bad idea
                    // I am treating the string as byte array and mutating it
                    if let Some((x, y)) = &self.selected {
                        self.src.set_at(*x, *y, text_to_insert);

                        // let c = text_to_insert.as_bytes();
                        // let idx = y * self.cols + x;
                        // unsafe {
                        //     let bytes = self.src.as_bytes_mut();
                        //     bytes[idx] = c[0];
                        // }
                        // info!("self.src: {}", self.src);
                        if self.mode == Mode::Insert {
                            self.select_next();
                        }
                    }
                }

                // egui::Event::Key {
                //     key,
                //     physical_key,
                //     pressed: true,
                //     modifiers,
                //     repeat,
                // } => {
                //     info!("Pressed key: {}", key);
                // }
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::splat(0.0);

            for y in 0..self.rows {
                ui.horizontal(|ui| {
                    for x in 0..self.cols {
                        // let idx = y * self.cols + x;

                        let s = self.src.get_at(x, y);

                        let mut background_color = Color32::TRANSPARENT;

                        background_color = if self.is_selected(x, y) {
                            Color32::DARK_GREEN
                        } else {
                            background_color
                        };

                        let button_text = egui::RichText::new(s)
                            .font(font_id.clone())
                            .extra_letter_spacing(0.4)
                            .background_color(background_color);

                        let button = egui::Button::new(button_text).small().frame(false);

                        if ui.add(button).clicked() {
                            self.select(x, y);
                        }
                    }
                });
            }
        });
    }
}
