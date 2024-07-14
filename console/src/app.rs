use std::time::{Duration, Instant};

use egui::{Color32, Event, EventFilter, Id, Key, Stroke, Vec2};
use tracing::{error, info};

use crate::{
    source::{is_terminator, Source, TERMINATOR},
    style::style,
    Color, Coord,
};

pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_GRID_SIZE: f32 = 8.0;

const DEFAULT_VISUAL_SELECTED_BG_COLOR: Color32 = Color::rgb(0, 92, 128).build();
const DEFAULT_VISUAL_SELECTED_STROKE_COLOR: Color32 = Color::rgb(192, 222, 255).build();

const VISUAL_SELECTED_STROKE_COLOR_BLINK: Color32 = Color::rgb(66, 66, 66).build();

const CURSOR_DELAY: u64 = 800;

const TERMINATOR_SELECT: &str = ".";
const TERMINATOR_MARKER: &str = "+";

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
pub struct ConsoleApp<const X: usize, const Y: usize> {
    src: Source,
    selected: Coord<X, Y>,
    blinked_at: Instant,
    blink_on: bool,
    opts: Opts,
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Opts {
    font_size: f32,
    grid_size: f32,
    mode: Mode,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum Mode {
    Insert,
    Command,
}

impl<const X: usize, const Y: usize> Default for ConsoleApp<X, Y> {
    fn default() -> Self {
        let src = Source::new(X, Y);
        Self {
            src,
            selected: Coord::<X, Y>::from(0, 0),
            opts: Opts::default(),
            blinked_at: Instant::now(),
            blink_on: true,
        }
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            font_size: DEFAULT_FONT_SIZE,
            grid_size: DEFAULT_GRID_SIZE,
            mode: Mode::Insert,
        }
    }
}

impl<const X: usize, const Y: usize> ConsoleApp<X, Y> {
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
        ConsoleApp::<X, Y>::default()
    }

    #[inline]
    pub fn is_selected(&self, x: usize, y: usize) -> bool {
        self.selected == Coord::from(x, y)
    }

    #[inline]
    pub fn select_at(&mut self, x: usize, y: usize) {
        self.select(Coord::from(x, y));
    }

    #[inline]
    pub fn select_up(&mut self) {
        self.select(self.selected.up());
    }

    #[inline]
    pub fn select_down(&mut self) {
        self.select(self.selected.down());
    }

    #[inline]
    pub fn select_left(&mut self) {
        self.select(self.selected.left());
    }

    #[inline]
    pub fn select_right(&mut self) {
        self.select(self.selected.right());
    }

    #[inline]
    pub fn select_next(&mut self) {
        self.select_right()
    }

    #[inline]
    pub fn select(&mut self, selected: Coord<X, Y>) {
        self.selected = selected;
        self.blink_on = true;
        self.blinked_at = Instant::now();
    }

    // this.isCursor = (x, y) => {
    //     return x === this.cursor.x && y === this.cursor.y
    //   }

    //   this.isMarker = (x, y) => {
    //     return x % this.grid.w === 0 && y % this.grid.h === 0
    //   }

    //   this.isNear = (x, y) => {
    //     return x > (parseInt(this.cursor.x / this.grid.w) * this.grid.w) - 1 && x <= ((1 + parseInt(this.cursor.x / this.grid.w)) * this.grid.w) && y > (parseInt(this.cursor.y / this.grid.h) * this.grid.h) - 1 && y <= ((1 + parseInt(this.cursor.y / this.grid.h)) * this.grid.h)
    //   }

    //   this.isLocals = (x, y) => {
    //     return this.isNear(x, y) === true && (x % (this.grid.w / 4) === 0 && y % (this.grid.h / 4) === 0) === true
    //   }

    //   this.isInvisible = (x, y) => {
    //     return this.orca.glyphAt(x, y) === '.' && !this.isMarker(x, y) && !this.cursor.selected(x, y) && !this.isLocals(x, y) && !this.ports[this.orca.indexAt(x, y)] && !this.orca.lockAt(x, y)
    //   }

    #[inline]
    pub fn delete(&mut self) {
        // If we are at last position delete the selected character
        // if self.selected.x == X - 1 {

        // }
        self.src.unset_at(self.selected.x, self.selected.y);

        self.select_left();
        self.src.unset_at(self.selected.x, self.selected.y);
    }
}

impl<const X: usize, const Y: usize> eframe::App for ConsoleApp<X, Y> {
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

        let event_filter = EventFilter {
            tab: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            escape: true,
        };

        let events = ctx.input(|i| i.filtered_events(&event_filter));

        for event in &events {
            match event {
                Event::Key {
                    key: Key::ArrowDown,
                    pressed: true,
                    ..
                } => self.select_down(),
                Event::Key {
                    key: Key::ArrowLeft,
                    pressed: true,
                    ..
                } => self.select_left(),
                Event::Key {
                    key: Key::ArrowRight,
                    pressed: true,
                    ..
                } => self.select_right(),
                Event::Key {
                    key: Key::ArrowUp,
                    pressed: true,
                    ..
                } => self.select_up(),
                Event::Key {
                    key: Key::Backspace,
                    pressed: true,
                    ..
                } => self.delete(),
                Event::Key {
                    key: Key::Delete,
                    pressed: true,
                    ..
                } => self.delete(),

                Event::Text(text_to_insert) => {
                    // info!("text_to_insert: {}", text_to_insert);

                    // This is all probably a very bad idea
                    // I am treating the string as byte array and mutating it
                    {
                        self.src
                            .set_at(self.selected.x, self.selected.y, text_to_insert);

                        // if self.opts.mode == Mode::Insert {
                        self.select_next();
                        // }
                    }
                }

                _ => {
                    // info!("Pressed");
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // let color = Color32::from_gray(55);

            // let stroke = Stroke::new(1.0, TEXT_HOVER);
            // ui.style_mut().visuals.widgets.hovered.fg_stroke = stroke;

            ui.spacing_mut().item_spacing = Vec2::splat(0.0);

            // let time = ui.input(|i| i.time);
            // let stable_time = ui.input(|i| i.stable_dt);
            // info!("time {time}");
            // info!("stable_time {stable_time}");

            for y in 0..Y {
                ui.horizontal(|ui| {
                    for x in 0..X {
                        let s = self.src.get_at(x, y);

                        if is_terminator(&s) {
                            let mut t = TERMINATOR;

                            // Highlight
                            if self.selected.in_grid(x, y, self.opts.grid_size) {
                                if x % 2 == 0 && y % 2 == 0 {
                                    t = TERMINATOR_SELECT;
                                }
                            }

                            // Grid markers
                            if x as f32 % self.opts.grid_size == 0.0
                                && y as f32 % self.opts.grid_size == 0.0
                            {
                                t = TERMINATOR_MARKER;
                            }

                            self.src.set_terminator_at(x, y, t);
                        } else {
                            if let Some(exp) = self.src.get_exp_at(x, y) {
                                let exp = exp.inner.first().unwrap();
                            }
                        }

                        ui.spacing_mut().item_spacing = Vec2::splat(0.0);
                        ui.spacing_mut().button_padding = Vec2::splat(0.0);

                        // let mut bg_color = Color32::TRANSPARENT;

                        // let bg_color = if self.is_selected(x, y) {
                        //     COLOR_BG_SELECTED
                        // } else {
                        //     COLOR_BG
                        // };

                        // info!("!!!elapsed {:?}", self.blinked_at);

                        let selected = self.is_selected(x, y);

                        if selected {
                            // error!("elapsed {:?}", self.blinked_at.elapsed());
                            // error!("blink_on {:?}", self.blink_on);

                            if self.blinked_at.elapsed() >= Duration::from_millis(CURSOR_DELAY) {
                                self.blinked_at = Instant::now();
                                self.blink_on = !self.blink_on;
                            }

                            if self.blink_on {
                                ui.style_mut().visuals.selection.bg_fill =
                                    DEFAULT_VISUAL_SELECTED_BG_COLOR;
                                ui.style_mut().visuals.selection.stroke.color =
                                    DEFAULT_VISUAL_SELECTED_STROKE_COLOR;
                            } else {
                                ui.style_mut().visuals.selection.bg_fill = Color32::TRANSPARENT;
                                ui.style_mut().visuals.selection.stroke.color =
                                    VISUAL_SELECTED_STROKE_COLOR_BLINK;
                            }
                        }

                        // let text_color = Color32::LIGHT_BLUE;
                        let button_text = egui::RichText::new(s).font(font_id.clone());
                        // .background_color(bg_color);
                        // .color(text_color);
                        // .extra_letter_spacing(0.4);

                        let color = Color32::from_gray(44);
                        // let stroke = Stroke::new(1.0, color);
                        let button = egui::Button::new(button_text)
                            // .stroke(stroke)
                            .small()
                            .selected(selected)
                            .frame(false);

                        if ui.add(button).clicked() {
                            self.select_at(x, y);
                        }
                    }
                });
            }

            ctx.request_repaint();
        });
    }
}
