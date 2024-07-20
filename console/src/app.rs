use std::time::{Duration, Instant};

use egui::{Color32, Event, EventFilter, FontId, Key, Rounding, Vec2};
use tracing::{error, info};

use crate::{
    source::{is_terminator, Glyph, Source, TERMINATOR},
    style::style,
    Color, Coord,
};

pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_GRID_SIZE: f32 = 8.0;
const DEFAULT_GRID_SELECTED_DOT_SPACING: usize = 2;

const DEFAULT_VISUAL_BG_COLOR: Color32 = Color32::TRANSPARENT;
const DEFAULT_VISUAL_SELECTED_BG_COLOR: Color32 = Color::rgb(0, 92, 128).build();
const DEFAULT_VISUAL_SELECTED_STROKE_COLOR: Color32 = Color::rgb(192, 222, 255).build();

const VISUAL_SELECTED_STROKE_COLOR_BLINK: Color32 = Color::rgb(66, 66, 66).build();

const CURSOR_DELAY: u64 = 800;

const TERMINATOR_SELECT: &str = ".";
const TERMINATOR_MARKER: &str = "+";

// const GRID_MARKER_SPACE: usize = 2;

/// ConsoleApp wraps the inner App
/// ConsoleApp handles the egui presentation concerns
/// App owns the underlying logic
///

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct ConsoleApp<const X: usize, const Y: usize> {
    app: App<X, Y>,
}

impl<const X: usize, const Y: usize> Default for ConsoleApp<X, Y> {
    fn default() -> Self {
        Self {
            app: App::default(),
        }
    }
}

impl<const X: usize, const Y: usize> ConsoleApp<X, Y> {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let style = style();
        cc.egui_ctx.set_style(style);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        ConsoleApp::<X, Y>::default()
    }

    #[inline(always)]
    pub fn select_at(&mut self, x: usize, y: usize) {
        self.app.select_at(x, y);
    }
}

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct App<const X: usize, const Y: usize> {
    src: Source,
    cursor: Coord<X, Y>,
    blinked_at: Instant,
    blink: bool,
    opts: Opts,
}
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Opts {
    font_id: FontId,
    grid_selected_dot_spacing: usize,
    grid_size: f32,
    mode: Mode,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum Mode {
    Insert,
    Command,
}

impl<const X: usize, const Y: usize> Default for App<X, Y> {
    fn default() -> Self {
        let src = Source::new(X, Y);
        Self {
            src,
            cursor: Coord::<X, Y>::from(0, 0),
            opts: Opts::default(),
            blinked_at: Instant::now(),
            blink: false,
        }
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            font_id: egui::FontId::monospace(DEFAULT_FONT_SIZE),
            grid_selected_dot_spacing: DEFAULT_GRID_SELECTED_DOT_SPACING,
            grid_size: DEFAULT_GRID_SIZE,
            mode: Mode::Insert,
        }
    }
}

impl<const X: usize, const Y: usize> eframe::App for ConsoleApp<X, Y> {
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

        self.app.event_handler(events);

        egui::CentralPanel::default().show(ctx, |ui| {
            // let color = Color32::from_gray(55);

            // let stroke = Stroke::new(1.0, TEXT_HOVER);
            // ui.style_mut().visuals.widgets.hovered.fg_stroke = stroke;

            ui.spacing_mut().item_spacing = Vec2::splat(0.0);
            ui.spacing_mut().button_padding = Vec2::splat(2.5);

            // button background colour
            ui.style_mut().visuals.widgets.inactive.weak_bg_fill = DEFAULT_VISUAL_BG_COLOR;
            ui.style_mut().visuals.widgets.inactive.rounding = Rounding::default();

            for y in 0..Y {
                ui.horizontal(|ui| {
                    for x in 0..X {
                        // Need
                        // s - char at x, y
                        //   -> button text

                        // g - glyph
                        //   -> bg and fill

                        let mut s = self.app.src.get_at(x, y);
                        let mut g = self.app.src.get_glyph_at(x, y);

                        if is_terminator(&s) {
                            if matches!(g, Glyph::Terminator(_)) {
                                g = self.app.terminator(x, y);
                            }
                            s = g.into()
                        }

                        // info!("{s} - {g:?}");// info!("{s} - {g:?}");

                        // if is_terminator(&s) {
                        //     let mut t = TERMINATOR;

                        //     // Highlight
                        //     if self.selected.in_grid(x, y, self.opts.grid_size) {
                        //         if x % 2 == 0 && y % 2 == 0 {
                        //             t = TERMINATOR_SELECT;
                        //         }
                        //     }

                        //     // Grid markers
                        //     if x as f32 % self.opts.grid_size == 0.0
                        //         && y as f32 % self.opts.grid_size == 0.0
                        //     {
                        //         t = TERMINATOR_MARKER;
                        //     }

                        //     self.src.set_terminator_at(x, y, t);
                        // } else {
                        //     if let Some(g) = self.src.get_token_at(x, y) {
                        //         info!("{s} - {g:?}")
                        //         //     let exp = exp.inner.first().unwrap();
                        //     }
                        // }

                        // info!("!!!elapsed {:?}", self.blinked_at);

                        let selected = self.app.is_cursor(x, y);

                        let visuals = g.style(selected);

                        ui.style_mut().visuals.selection.bg_fill = visuals.bg_fill;
                        ui.style_mut().visuals.selection.stroke.color = visuals.stroke_color;

                        if selected {
                            // error!("elapsed {:?}", self.blinked_at.elapsed());

                            if self.app.blinked_at.elapsed() >= Duration::from_millis(CURSOR_DELAY)
                            {
                                self.app.blinked_at = Instant::now();
                                self.app.blink = !self.app.blink;
                            }

                            if self.app.blink {
                                ui.style_mut().visuals.selection.bg_fill = Color32::TRANSPARENT;
                                ui.style_mut().visuals.selection.stroke.color =
                                    VISUAL_SELECTED_STROKE_COLOR_BLINK;
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
                            self.select_at(x, y);
                        }
                    }
                });
            }

            ctx.request_repaint();
        });
    }
}

impl<const X: usize, const Y: usize> App<X, Y> {
    #[inline]
    pub fn is_cursor(&self, x: usize, y: usize) -> bool {
        self.cursor == Coord::from(x, y)
    }

    #[inline]
    pub fn select_at(&mut self, x: usize, y: usize) {
        self.cursor(Coord::from(x, y));
    }

    #[inline]
    pub fn cursor_up(&mut self) {
        self.cursor(self.cursor.up());
    }

    #[inline]
    pub fn cursor_down(&mut self) {
        self.cursor(self.cursor.down());
    }

    #[inline]
    pub fn cursor_left(&mut self) {
        self.cursor(self.cursor.left());
    }

    #[inline]
    pub fn cursor_right(&mut self) {
        self.cursor(self.cursor.right());
    }

    #[inline]
    pub fn select_next(&mut self) {
        self.cursor_right()
    }

    #[inline]
    pub fn cursor(&mut self, selected: Coord<X, Y>) {
        self.cursor = selected;
        self.blink = false;
        self.blinked_at = Instant::now();
    }

    #[inline]
    pub fn delete(&mut self) {
        self.src.unset_at(self.cursor.x, self.cursor.y);
        self.cursor_left();
    }

    fn terminator(&self, x: usize, y: usize) -> Glyph {
        // Highlight
        if self.cursor.in_grid(x, y, self.opts.grid_size) {
            if x % self.opts.grid_selected_dot_spacing == 0
                && y % self.opts.grid_selected_dot_spacing == 0
            {
                return Glyph::highlight();
            }
        }

        // Grid markers
        if x as f32 % self.opts.grid_size == 0.0 && y as f32 % self.opts.grid_size == 0.0 {
            return Glyph::marker();
        }

        Glyph::default()
    }

    fn set_at(&mut self, x: usize, y: usize, text_to_insert: &str) {
        self.src.set_at(x, y, text_to_insert);
    }

    fn render(&self, x: usize, y: usize) -> (String, Glyph) {
        // s - char at x, y
        //   -> button text

        // g - glyph
        //   -> bg and fill
        let mut s = self.src.get_at(x, y);
        let mut g = self.src.get_glyph_at(x, y);

        // if is_terminator(&s) {
        //     if matches!(g, Glyph::Terminator(_)) {
        //         g = self.terminator(x, y);
        //     }
        //     s = g.into()
        // }

        (s, g)
    }

    /// Handles event and returns boolean indicating if repating is required
    ///
    fn event_handler(&mut self, events: Vec<Event>) -> bool {
        let mut repaint = false;
        for event in &events {
            match event {
                Event::Key {
                    key: Key::ArrowDown,
                    pressed: true,
                    ..
                } => self.cursor_down(),
                Event::Key {
                    key: Key::ArrowLeft,
                    pressed: true,
                    ..
                } => self.cursor_left(),
                Event::Key {
                    key: Key::ArrowRight,
                    pressed: true,
                    ..
                } => self.cursor_right(),
                Event::Key {
                    key: Key::ArrowUp,
                    pressed: true,
                    ..
                } => self.cursor_up(),
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

                    self.src
                        .set_at(self.cursor.x, self.cursor.y, text_to_insert);

                    // if self.opts.mode == Mode::Insert {
                    self.select_next();
                    repaint = true;
                    // }
                }

                _ => {
                    // info!("Pressed");
                }
            }
        }
        repaint
    }
}

struct GlyphStyle {
    bg_fill: Color32,
    stroke_color: Color32,
}

impl Glyph {
    fn style(&self, selected: bool) -> GlyphStyle {
        let v = GlyphStyle {
            bg_fill: DEFAULT_VISUAL_SELECTED_BG_COLOR,
            stroke_color: DEFAULT_VISUAL_SELECTED_STROKE_COLOR,
        };

        match (self, selected) {
            (Glyph::Function, true) => v,
            // (Glyph::Function, true) => GlyphVisuals {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::Function, false) => GlyphStyle {
                bg_fill: Color::rgb(97, 0, 255).build(),
                stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            },
            // (Glyph::Number, true) => GlyphVisuals {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::Number, true) => v,
            (Glyph::Number, false) => GlyphStyle {
                bg_fill: Color::rgb(97, 0, 255).build(),
                stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            },
            // (Glyph::Note, true) => GlyphVisuals {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::Note, true) => v,
            (Glyph::Note, false) => GlyphStyle {
                bg_fill: Color::rgb(97, 0, 255).build(),
                stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            },
            // (Glyph::String, true) => GlyphVisuals {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::String, true) => v,
            (Glyph::String, false) => GlyphStyle {
                bg_fill: Color::rgb(97, 0, 255).build(),
                stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            },
            (Glyph::Terminator(_), true) => v,
            // (Glyph::Terminator(_), true) => GlyphVisuals {
            //     bg_fill: Color::rgb(97, 0, 255).build(),
            //     stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            // },
            (Glyph::Terminator(_), false) => GlyphStyle {
                bg_fill: Color::rgb(97, 0, 255).build(),
                stroke_color: Color::rgb(97, 0, 255).with_alpha(128).build(),
            },
        }
    }
}

#[cfg(test)]
mod test {

    use tracing::info;

    use crate::{source::Glyph, test::trace};

    use super::App;

    #[test]
    fn test_highlight() {
        trace();

        let mut app = App::<10, 10>::default();

        app.set_at(0, 0, "i");
        app.set_at(1, 0, "d");

        // info!(":{:?}", app.src.inner);

        // let (s, g) = app.render(1, 0);
        // info!("{}:{:?}", s, g);

        let (s, g) = app.render(2, 0);
        info!("{}:{:?}", s, g);
        assert_eq!(&s, "s");
        // assert_eq!(g, Glyph::String);
    }
}
