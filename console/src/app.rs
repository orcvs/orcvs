use egui::{Color32, Event, EventFilter};
use log::{error, info};

use lang::{eval, parse};
pub const DEFAULT_FONT_SIZE: f32 = 23.0;
pub const DEFAULT_COL_COUNT: usize = 10;
pub const DEFAULT_ROW_COUNT: usize = 10;
// pub const DEFAULT_SCALE: f32 = 1.0;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ConsoleApp {
    grid: Grid,
    src: String,
    selected: Option<(usize, usize)>,
}

impl Default for ConsoleApp {
    fn default() -> Self {
        let grid = Grid::default();
        let mut src = String::new();

        for y in 0..=DEFAULT_ROW_COUNT - 1 {
            for x in 0..=DEFAULT_COL_COUNT - 1 {
                src.push('.');
            }
            // src.push("\n");
        }

        Self {
            grid,
            src,
            selected: None,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Grid {
    items: [[Option<char>; DEFAULT_COL_COUNT]; DEFAULT_ROW_COUNT],
}

impl Default for Grid {
    fn default() -> Self {
        let mut items = [[None; DEFAULT_COL_COUNT]; DEFAULT_ROW_COUNT];
        Self { items }
    }
}

impl Grid {
    #[inline(always)]
    fn get_char_at(&self, x: usize, y: usize) -> &Option<char> {
        &self.items[y][x]
    }

    fn set_char_at(&mut self, x: usize, y: usize, c: char) {
        self.items[y][x] = Some(c);
    }
}

// #[derive(Copy, Clone, Default, serde::Deserialize, serde::Serialize)]
// #[serde(default)]
// pub struct Glyph {
//     char: char,
//     x: usize,
//     y: usize,
//     state: GlyphState,
// }

// #[derive(Copy, Clone, Default, serde::Deserialize, serde::Serialize)]
// pub enum GlyphState {
//     #[default]
//     Default,
//     Selected,
// }

impl ConsoleApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    pub fn is_selected(&self, x: usize, y: usize) -> bool {
        self.selected == Some((x, y))
    }

    pub fn select(&mut self, x: usize, y: usize) {
        self.selected = Some((x, y));
    }

    pub fn deselect(&mut self) {
        self.selected = None;
    }
}

// let mut button_states: Vec<Vec<bool>> = vec![vec![false; DEFAULT_COL_COUNT]; DEFAULT_ROW_COUNT];

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

        let event_filter = EventFilter::default();
        let events = ctx.input(|i| i.filtered_events(&event_filter));

        // error!("{}", self.src);
        // let s = "hello".to_string();
        // let ch = s.chars().nth(index).unwrap();

        // Expresions
        // Start coord, length
        // On key event
        //  - if no char to right, new expression of len 1
        //  - if char to right, increment len by 1
        //
        // On delete event
        //

        // [x, y]

        for event in &events {
            match event {
                Event::Text(text_to_insert) => {
                    info!("text_to_insert: {}", text_to_insert);

                    // This is all probably a very bad idea
                    // I am treating the string as byte array and mutating it
                    if let Some((x, y)) = &self.selected {
                        let c = text_to_insert.as_bytes();
                        let idx = y * DEFAULT_COL_COUNT + x;
                        unsafe {
                            let bytes = self.src.as_bytes_mut();
                            bytes[idx] = c[0];
                        }
                        info!("self.src: {}", self.src);
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

            for y in 0..=DEFAULT_ROW_COUNT - 1 {
                let mut start: Option<usize> = None;

                ui.horizontal(|ui| {
                    for x in 0..=DEFAULT_COL_COUNT - 1 {
                        let idx = y * DEFAULT_COL_COUNT + x;

                        // info!("s: {}", idx);
                        let s = unsafe { self.src.get_unchecked(idx..=idx) };
                        // info!("s: {}", s);

                        match s {
                            "." => {
                                // end of expression
                                if let Some(start_idx) = start {
                                    let mut exp = unsafe { self.src.get_unchecked(start_idx..idx) };
                                    info!("exp: {}", exp);

                                    let result = parse(&mut exp);
                                    // let result = eval(exp);
                                    info!("result: {:?}", result);
                                    start = None; // reset expression
                                };
                            }
                            _ => {
                                if start.is_none() {
                                    start = Some(idx);
                                };
                            }
                        };

                        let background_color = if self.is_selected(x, y) {
                            Color32::DARK_GREEN
                        } else {
                            Color32::TRANSPARENT
                        };

                        let button_text = egui::RichText::new(s)
                            .font(font_id.clone())
                            .extra_letter_spacing(0.4)
                            .background_color(background_color);

                        let button = egui::Button::new(button_text).small().frame(false);

                        if ui.add(button).clicked() {
                            let _ = &self.select(x, y);
                        }
                    }
                });
            }
        });
    }
}
