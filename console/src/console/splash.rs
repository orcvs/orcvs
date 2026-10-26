//!
//! The splash the native console opens through: its own window, borderless
//! and splash-sized, holding the logo, the name and the version until the hold
//! ends or the viewer presses a key or clicks. The window then grows into the
//! console and the logo fades over the running Grid (ADR 0083).
//!

use egui::{Align2, Color32, FontId, Rect, Vec2, ViewportCommand, pos2, vec2};

use super::{Console, DEFAULT_VIEW_SIZE, DEFAULT_VIEW_SIZE_MIN};
use crate::theme::Appearance;
use crate::window_geometry::WindowGeometry;

/// The splash window's size, which `main` opens the window at.
pub const SPLASH_WINDOW: [f32; 2] = [420.0, 460.0];

/// How long the splash holds before it grows into the console by itself.
pub(crate) const HOLD_SECONDS: f64 = 1.5;

/// How long the logo takes to fade over the console.
pub(crate) const FADE_SECONDS: f64 = 0.35;

/// The logo's side at most, in points.
const LOGO_SIDE: f32 = 300.0;

pub(super) struct Splash {
    logo: egui::TextureHandle,
    /// The geometry the window grows to: the one the console was closed at,
    /// or `None` for the default size, centred.
    console_window: Option<WindowGeometry>,
    /// egui's input time at the splash's first frame.
    opened_at: Option<f64>,
    /// egui's input time the hold ended at, which the fade runs from.
    ended_at: Option<f64>,
}

impl Splash {
    pub(super) fn new(ctx: &egui::Context, console_window: Option<WindowGeometry>) -> Self {
        // The window icon's own decoder, so the logo adds no dependency.
        let logo =
            eframe::icon_data::from_png_bytes(include_bytes!("../../assets/logo-splash.png"))
                .expect("the bundled logo is a PNG");
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [logo.width as usize, logo.height as usize],
            &logo.rgba,
        );
        Self {
            logo: ctx.load_texture("logo-splash", image, egui::TextureOptions::LINEAR),
            console_window,
            opened_at: None,
            ended_at: None,
        }
    }

    /// Whether the splash is still holding, rather than fading over the console.
    pub(super) fn holding(&self) -> bool {
        self.ended_at.is_none()
    }

    ///
    /// The geometry a save stores while the splash is showing: the one the
    /// console grows to, never the splash window's.
    ///
    #[cfg(feature = "persistence")]
    pub(super) fn console_window(&self) -> Option<WindowGeometry> {
        self.console_window
    }

    ///
    /// Ends the hold at `now`: the window gains its decorations and grows to
    /// the console's geometry.
    ///
    fn end(&mut self, ctx: &egui::Context, now: f64) {
        self.ended_at = Some(now);
        let min_size = Vec2::from(DEFAULT_VIEW_SIZE_MIN);
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(min_size));
        let monitor_size = ctx.input(|input| input.viewport().monitor_size);
        let commands = match self.console_window {
            Some(geometry) => geometry.commands(min_size, monitor_size),
            None => {
                let size = Vec2::from(DEFAULT_VIEW_SIZE);
                let mut commands = vec![ViewportCommand::InnerSize(size)];
                if let Some(monitor_size) = monitor_size {
                    let corner = ((monitor_size - size) * 0.5).max(Vec2::ZERO);
                    commands.push(ViewportCommand::OuterPosition(corner.to_pos2()));
                }
                commands
            }
        };
        for command in commands {
            ctx.send_viewport_cmd(command);
        }
    }

    /// The logo's opacity at `now`: whole through the hold, then fading.
    fn opacity(&self, now: f64) -> f32 {
        match self.ended_at {
            None => 1.0,
            Some(ended_at) => (1.0 - (now - ended_at) / FADE_SECONDS).clamp(0.0, 1.0) as f32,
        }
    }

    ///
    /// Paints the backdrop at `backdrop`'s opacity and the logo, the name and
    /// the version over `rect`, on a layer above the console.
    ///
    fn paint(&self, ctx: &egui::Context, rect: Rect, backdrop: Color32, now: f64) {
        let opacity = self.opacity(now);
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("splash"),
        ));
        painter.rect_filled(rect, 0.0, backdrop.gamma_multiply(opacity));

        let side = LOGO_SIDE.min(rect.width() * 0.7).min(rect.height() * 0.6);
        let logo = Rect::from_center_size(rect.center() - vec2(0.0, 28.0), Vec2::splat(side));
        // The logo is white on transparent: untinted it is the artwork on a
        // dark backdrop, and tinted black it reads on a light one.
        let ink = if ctx.theme() == egui::Theme::Dark {
            Color32::WHITE
        } else {
            Color32::BLACK
        };
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        painter.image(self.logo.id(), logo, uv, ink.gamma_multiply(opacity));

        let visuals = &ctx.style_of(ctx.theme()).visuals;
        painter.text(
            pos2(rect.center().x, logo.bottom() + 18.0),
            Align2::CENTER_TOP,
            "O R C V S",
            FontId::monospace(18.0),
            visuals.text_color().gamma_multiply(opacity),
        );
        painter.text(
            pos2(rect.center().x, logo.bottom() + 46.0),
            Align2::CENTER_TOP,
            concat!("v", env!("CARGO_PKG_VERSION")),
            FontId::monospace(12.0),
            visuals.weak_text_color().gamma_multiply(opacity),
        );
    }
}

impl Console {
    ///
    /// Opens the console through the splash, which grows the window to
    /// `console_window` when it ends, or to the default size when that is
    /// `None`.
    ///
    pub(crate) fn open_through_splash(&mut self, console_window: Option<WindowGeometry>) {
        self.splash = Some(Splash::new(&self.ctx, console_window));
    }

    ///
    /// Shows the splash while it holds, and answers whether it held this
    /// frame, in which case the console shows nothing.
    ///
    /// The key press or click that ends the hold ends with this frame, so it
    /// never reaches the console. A close request ends the hold and lets the
    /// console run this frame, so the console asks about unsaved changes in
    /// the decorated window.
    ///
    pub(super) fn hold_splash(&mut self, ctx: &egui::Context, appearance: Appearance) -> bool {
        let Some(splash) = self.splash.as_mut() else {
            return false;
        };
        if !splash.holding() {
            return false;
        }
        let (now, dismissed, closing) = ctx.input(|input| {
            let dismissed = input.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key { pressed: true, .. }
                        | egui::Event::PointerButton { pressed: true, .. }
                )
            });
            (input.time, dismissed, input.viewport().close_requested())
        });
        let opened_at = *splash.opened_at.get_or_insert(now);
        if closing || dismissed || now - opened_at >= HOLD_SECONDS {
            splash.end(ctx, now);
            ctx.request_repaint();
            return dismissed && !closing;
        }
        let backdrop = self.themes.presented(appearance).window_background;
        splash.paint(ctx, ctx.content_rect(), backdrop, now);
        ctx.request_repaint();
        true
    }

    ///
    /// Fades the logo over the console once the hold has ended, and drops the
    /// splash when the fade is done.
    ///
    pub(super) fn fade_splash(&mut self, ctx: &egui::Context, appearance: Appearance) {
        let Some(splash) = self.splash.as_ref() else {
            return;
        };
        let now = ctx.input(|input| input.time);
        if splash.opacity(now) <= 0.0 {
            self.splash = None;
            return;
        }
        let backdrop = self.themes.presented(appearance).window_background;
        splash.paint(ctx, ctx.content_rect(), backdrop, now);
        ctx.request_repaint();
    }
}
