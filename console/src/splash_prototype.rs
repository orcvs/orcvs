//! PROTOTYPE (splash-screen): three native splash variants around the real
//! console, chosen by `ORCVS_SPLASH=A|B|C|off` (default `A`), held for
//! `ORCVS_SPLASH_MS` milliseconds (default 1500). Throwaway; lives on the
//! `prototype/splash-screen` branch only.
//!
//! - A: in-window. The console window opens on the logo over the Theme's
//!   backdrop, then the logo fades to reveal the Grid already running.
//! - B: splash window. The app opens as a small borderless square showing the
//!   logo and the name, then the same window grows into the console.
//! - C: welcome card. The Grid is drawn at once beneath a scrim with the logo,
//!   the name and the version, held until the first key or click.

use std::time::{Duration, Instant};

use console::console::{Console, DEFAULT_VIEW_SIZE, DEFAULT_VIEW_SIZE_MIN};
use egui::{Color32, ViewportCommand, vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variant {
    A,
    B,
    C,
    Off,
}

impl Variant {
    pub fn from_env() -> Self {
        match std::env::var("ORCVS_SPLASH").as_deref() {
            Ok("B" | "b") => Self::B,
            Ok("C" | "c") => Self::C,
            Ok("off" | "OFF" | "0") => Self::Off,
            _ => Self::A,
        }
    }
}

pub fn hold() -> Duration {
    std::env::var("ORCVS_SPLASH_MS")
        .ok()
        .and_then(|ms| ms.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(1500))
}

/// B's splash window: a square that holds the logo and the name.
pub const SPLASH_WINDOW: [f32; 2] = [420.0, 460.0];
const FADE: Duration = Duration::from_millis(350);

pub struct Splash {
    console: Console,
    variant: Variant,
    hold: Duration,
    opened: Instant,
    logo: egui::TextureHandle,
    /// When the splash began to leave: A and C fade from here, B resizes here.
    leaving: Option<Instant>,
    done: bool,
}

impl Splash {
    pub fn new(cc: &eframe::CreationContext<'_>, console: Console, variant: Variant) -> Self {
        // eframe's own PNG decoder, the one the window icon already uses: no new dependency.
        let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/logo-splash.png"))
            .expect("logo-splash.png decodes");
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [icon.width as usize, icon.height as usize],
            &icon.rgba,
        );
        let logo = cc
            .egui_ctx
            .load_texture("logo-splash", image, egui::TextureOptions::LINEAR);
        Self {
            console,
            variant,
            hold: hold(),
            opened: Instant::now(),
            logo,
            leaving: None,
            done: variant == Variant::Off,
        }
    }

    fn any_input(ctx: &egui::Context) -> bool {
        ctx.input(|i| {
            i.events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key { pressed: true, .. }
                        | egui::Event::PointerButton { pressed: true, .. }
                )
            })
        })
    }

    /// The logo's tint: white reproduces the artwork on a dark backdrop, black
    /// inverts it for a light one.
    fn ink(ctx: &egui::Context) -> Color32 {
        if ctx.theme() == egui::Theme::Dark {
            Color32::WHITE
        } else {
            Color32::BLACK
        }
    }

    fn backdrop(&self, ctx: &egui::Context) -> Color32 {
        let [r, g, b, a] =
            eframe::App::clear_color(&self.console, &ctx.style_of(ctx.theme()).visuals);
        egui::Rgba::from_rgba_unmultiplied(r, g, b, a).into()
    }

    /// 1 while held, falling to 0 over the fade.
    fn opacity(&self) -> f32 {
        match self.leaving {
            None => 1.0,
            Some(at) => 1.0 - (at.elapsed().as_secs_f32() / FADE.as_secs_f32()).min(1.0),
        }
    }

    fn overlay(&self, ctx: &egui::Context, scrim: bool, logo_side: f32, caption: bool) {
        let opacity = self.opacity();
        let screen = ctx.content_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("splash_prototype"),
        ));
        let backdrop = self.backdrop(ctx);
        let fill = if scrim {
            backdrop.gamma_multiply(0.82 * opacity)
        } else {
            backdrop.gamma_multiply(opacity)
        };
        painter.rect_filled(screen, 0.0, fill);
        let side = logo_side
            .min(screen.width() * 0.6)
            .min(screen.height() * 0.6);
        let centre = screen.center() - vec2(0.0, if caption { 28.0 } else { 0.0 });
        let rect = egui::Rect::from_center_size(centre, vec2(side, side));
        painter.image(
            self.logo.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Self::ink(ctx).gamma_multiply(opacity),
        );
        if caption {
            let text = ctx
                .style_of(ctx.theme())
                .visuals
                .text_color()
                .gamma_multiply(opacity);
            let muted = ctx
                .style_of(ctx.theme())
                .visuals
                .weak_text_color()
                .gamma_multiply(opacity);
            painter.text(
                egui::pos2(screen.center().x, rect.bottom() + 18.0),
                egui::Align2::CENTER_TOP,
                "O R C V S",
                egui::FontId::monospace(18.0),
                text,
            );
            painter.text(
                egui::pos2(screen.center().x, rect.bottom() + 46.0),
                egui::Align2::CENTER_TOP,
                concat!("v", env!("CARGO_PKG_VERSION")),
                egui::FontId::monospace(12.0),
                muted,
            );
        }
    }
}

impl eframe::App for Splash {
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.console.save(storage);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        self.console.clear_color(visuals)
    }

    fn ui(&mut self, root: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        if self.done {
            self.console.ui(root, frame);
            return;
        }
        let input = Self::any_input(&ctx);
        match self.variant {
            Variant::A => {
                self.console.ui(root, frame);
                if self.leaving.is_none() && (self.opened.elapsed() >= self.hold || input) {
                    self.leaving = Some(Instant::now());
                }
                self.overlay(&ctx, false, 256.0, false);
            }
            Variant::B => {
                if self.leaving.is_none() && (self.opened.elapsed() >= self.hold || input) {
                    self.leaving = Some(Instant::now());
                    let size = vec2(DEFAULT_VIEW_SIZE[0], DEFAULT_VIEW_SIZE[1]);
                    ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
                    ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
                    ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(vec2(
                        DEFAULT_VIEW_SIZE_MIN[0],
                        DEFAULT_VIEW_SIZE_MIN[1],
                    )));
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
                    if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                        let at = ((monitor - size) * 0.5).max(egui::Vec2::ZERO);
                        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(at.to_pos2()));
                    }
                }
                if self.leaving.is_some() {
                    self.console.ui(root, frame);
                    self.overlay(&ctx, false, 256.0, true);
                } else {
                    let rect = ctx.content_rect();
                    root.painter().rect_filled(rect, 0.0, self.backdrop(&ctx));
                    self.overlay(&ctx, false, 300.0, true);
                }
            }
            Variant::C => {
                self.console.ui(root, frame);
                if self.leaving.is_none() && input {
                    self.leaving = Some(Instant::now());
                }
                self.overlay(&ctx, true, 220.0, true);
            }
            Variant::Off => unreachable!("Off starts done"),
        }
        if self.leaving.is_some_and(|at| at.elapsed() >= FADE) {
            self.done = true;
        }
        ctx.request_repaint();
    }
}
