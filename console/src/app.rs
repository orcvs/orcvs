use egui::{Color32, Event, EventFilter, Stroke};
use tracing::info;

use crate::{source::Source, style};
pub const DEFAULT_FONT_SIZE: f32 = 33.0;
pub const DEFAULT_COL_COUNT: usize = 5;
pub const DEFAULT_ROW_COUNT: usize = 5;
// pub const DEFAULT_SCALE: f32 = 1.0;

// b00_bg

pub const COLOR_BG: Color32 = Color32::TRANSPARENT;
pub const COLOR_BG_LIGHT: Color32 = Color32::TRANSPARENT;
pub const COLOR_BG_SELECTED: Color32 = Color32::from_gray(33);
pub const COLOR_BG_COMMENT: Color32 = Color32::TRANSPARENT;
pub const COLOR_FG_DARK: Color32 = Color32::TRANSPARENT;
pub const COLOR_FG: Color32 = Color32::TRANSPARENT;
pub const COLOR_FG_LIGHT: Color32 = Color32::TRANSPARENT;
pub const COLOR_FG_BRIGHT: Color32 = Color32::TRANSPARENT;
pub const COLOR_VAR: Color32 = Color32::TRANSPARENT;
pub const COLOR_FG_: Color32 = Color32::TRANSPARENT;

pub const TEXT_HOVER: Color32 = Color32::WHITE;

// base00 - Default Background
// base01 - Lighter Background (Used for status bars, line number and folding marks)
// base02 - Selection Background
// base03 - Comments, Invisibles, Line Highlighting
// base04 - Dark Foreground (Used for status bars)
// base05 - Default Foreground, Caret, Delimiters, Operators
// base06 - Light Foreground (Not often used)
// base07 - Brightest Foreground (Not often used)
// base08 - Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted
// base09 - Integers, Boolean, Constants, XML Attributes, Markup Link Url
// base0A - Classes, Markup Bold, Search Text Background
// base0B - Strings, Inherited Class, Markup Code, Diff Inserted
// base0C - Support, Regular Expressions, Escape Characters, Markup Quotes
// base0D - Functions, Methods, Attribute IDs, Headings
// base0E - Keywords, Storage, Selector, Markup Italic, Diff Changed
// base0F - Deprecated, Opening/Closing Embedded Language Tags, e.g. <?php ?>

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct ConsoleApp {
    src: Source,
    selected: Option<(usize, usize)>,
    mode: Mode,
    cols: usize,
    rows: usize,
    opts: Opts,
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Opts {
    font_size: f32,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
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
            opts: Opts::default(),
            cols: DEFAULT_COL_COUNT,
            rows: DEFAULT_ROW_COUNT,
        }
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            font_size: DEFAULT_FONT_SIZE,
        }
    }
}

impl ConsoleApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let style = style();
        cc.egui_ctx.set_style(style);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        ConsoleApp::default()
    }

    #[inline]
    pub fn is_selected(&self, x: usize, y: usize) -> bool {
        self.selected == Some((x, y))
    }

    #[inline]
    pub fn select(&mut self, x: usize, y: usize) {
        self.selected = Some((x, y));
    }

    #[inline]
    pub fn select_next(&mut self) {
        if let Some((x, y)) = self.selected {
            let x = std::cmp::min(x + 1, self.cols - 1);
            self.selected = Some((x, y));
        }
    }

    #[inline]
    pub fn deselect(&mut self) {
        self.selected = None;
    }
}

impl eframe::App for ConsoleApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font_id = egui::FontId::monospace(self.opts.font_size);

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
                ui.label(format!("HELLO"));
                // egui::widgets::global_dark_light_mode_buttons(ui);
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
            // let color = Color32::from_gray(55);
            let stroke = Stroke::new(1.0, TEXT_HOVER);
            ui.style_mut().visuals.widgets.hovered.fg_stroke = stroke;

            ui.spacing_mut().item_spacing = egui::Vec2::splat(0.0);

            for y in 0..self.rows {
                ui.horizontal(|ui| {
                    for x in 0..self.cols {
                        let s = self.src.get_at(x, y);
                        // let s = ".";
                        // info!("x/y: {x}/{y}");
                        // info!("s: {s}");

                        // let mut background_color = Color32::LIGHT_BLUE;

                        // background_color = if self.is_selected(x, y) {
                        //     Color32::DARK_GREEN
                        // } else {
                        //     background_color
                        // };

                        let bg_color = if self.is_selected(x, y) {
                            COLOR_BG_SELECTED
                        } else {
                            COLOR_BG
                        };

                        // let text_color = Color32::LIGHT_BLUE;
                        let button_text = egui::RichText::new(s)
                            .font(font_id.clone())
                            .background_color(bg_color);
                        // .color(text_color);
                        // .extra_letter_spacing(0.4);

                        let color = Color32::from_gray(44);
                        let stroke = Stroke::new(1.0, color);
                        let button = egui::Button::new(button_text)
                            .stroke(stroke)
                            .small()
                            .frame(false);

                        if ui.add(button).clicked() {
                            self.select(x, y);
                        }
                    }
                });
            }
        });
    }
}
