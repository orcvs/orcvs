//!
//! The splash the native console opens through: its own window, borderless
//! and splash-sized, holding the logo, the name and the version until the hold
//! ends or the viewer presses a key or clicks. The window then grows into the
//! console and the logo fades over the running Grid.
//!

use egui::{Align2, FontId, Pos2, Rect, Vec2, ViewportCommand, pos2, vec2};

use super::{Console, DEFAULT_VIEW_SIZE, DEFAULT_VIEW_SIZE_MIN, logo};
use crate::theme::{Appearance, Theme};
use crate::window_geometry::WindowGeometry;

/// The splash window's size, which `main` opens the window at.
pub const SPLASH_WINDOW: [f32; 2] = [420.0, 460.0];

/// How long the splash holds before it grows into the console by itself.
pub(crate) const HOLD_SECONDS: f64 = 1.5;

/// How long the logo takes to fade over the console.
pub(crate) const FADE_SECONDS: f64 = 0.35;

pub(super) struct Splash {
    logo: egui::TextureHandle,
    /// The geometry the window grows to: the one the console was closed at,
    /// or `None` for the default size, centred.
    console_window: Option<WindowGeometry>,
    /// egui's input time at the splash's first frame.
    opened_at: Option<f64>,
    /// egui's input time the hold ended at, which the fade runs from.
    ended_at: Option<f64>,
    /// Where on the screen the logo's centre was when the hold ended, so the
    /// fade keeps it there while the window gains its title bar and grows.
    /// `None` where the platform reports no window position.
    logo_on_screen: Option<Pos2>,
    /// Whether a key that ended the hold is still down, so keyboard input is
    /// withheld from the console.
    withholding_keys: bool,
}

impl Splash {
    pub(super) fn new(ctx: &egui::Context, console_window: Option<WindowGeometry>) -> Self {
        Self {
            logo: logo::texture(ctx),
            console_window,
            opened_at: None,
            ended_at: None,
            logo_on_screen: None,
            withholding_keys: false,
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
    /// the console's geometry, or to the default size about the splash's
    /// centre.
    ///
    fn end(&mut self, ctx: &egui::Context, now: f64) {
        self.ended_at = Some(now);
        self.logo_on_screen = ctx
            .input(|input| input.viewport().inner_rect)
            .map(|window| window.min + self.logo_centre(ctx).to_vec2());
        let min_size = Vec2::from(DEFAULT_VIEW_SIZE_MIN);
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(min_size));
        let (monitor_size, centre) = ctx.input(|input| {
            let viewport = input.viewport();
            (
                viewport.monitor_size,
                viewport.outer_rect.map(|rect| rect.center()),
            )
        });
        let geometry = self
            .console_window
            .unwrap_or_else(|| WindowGeometry::centred_on(centre, Vec2::from(DEFAULT_VIEW_SIZE)));
        for command in geometry.commands(min_size, monitor_size) {
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
    /// The logo's centre in the window: above the window's centre through the
    /// hold, then wherever on the screen it was when the hold ended.
    ///
    pub(super) fn logo_centre(&self, ctx: &egui::Context) -> Pos2 {
        let window = ctx.input(|input| input.viewport().inner_rect);
        match (self.logo_on_screen, window) {
            (Some(on_screen), Some(window)) => on_screen - window.min.to_vec2(),
            _ => ctx.content_rect().center() - vec2(0.0, 28.0),
        }
    }

    ///
    /// Paints `theme`'s backdrop, the logo, the name and the version over the
    /// window, on a layer above the console, at the opacity `now` gives.
    ///
    fn show(&self, ctx: &egui::Context, theme: &Theme, appearance: Appearance, now: f64) {
        let opacity = self.opacity(now);
        let rect = ctx.content_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("splash"),
        ));
        painter.rect_filled(rect, 0.0, theme.window_background.gamma_multiply(opacity));

        let logo_rect = Rect::from_center_size(self.logo_centre(ctx), Vec2::splat(logo::SIDE));
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        painter.image(
            self.logo.id(),
            logo_rect,
            uv,
            logo::ink(appearance).gamma_multiply(opacity),
        );
        painter.text(
            pos2(logo_rect.center().x, logo_rect.bottom() + 18.0),
            Align2::CENTER_TOP,
            logo::NAME,
            FontId::monospace(18.0),
            theme.text.gamma_multiply(opacity),
        );
        painter.text(
            pos2(logo_rect.center().x, logo_rect.bottom() + 46.0),
            Align2::CENTER_TOP,
            logo::VERSION,
            FontId::monospace(12.0),
            theme.text_muted.gamma_multiply(opacity),
        );
        ctx.request_repaint();
    }
}

impl Console {
    ///
    /// The console opening through the splash, which grows the window to
    /// `console_window` when it ends, or to the default size when that is
    /// `None`.
    ///
    pub(crate) fn through_splash(mut self, console_window: Option<WindowGeometry>) -> Self {
        self.splash = Some(Splash::new(&self.ctx, console_window));
        self
    }

    ///
    /// Shows the splash while it holds, and answers whether it held this
    /// frame, in which case the console shows nothing.
    ///
    /// The key press or click that ends the hold ends with this frame, so it
    /// never reaches the console, and keyboard input is withheld from the
    /// console until every key held then is released, so a held key's repeats
    /// and late text do not reach the Source either. A close request ends the
    /// hold and lets the console run this frame, so the console asks about
    /// unsaved changes in the decorated window.
    ///
    pub(super) fn hold_splash(&mut self, ctx: &egui::Context, appearance: Appearance) -> bool {
        let Some(splash) = self.splash.as_mut() else {
            return false;
        };
        if !splash.holding() {
            if splash.withholding_keys {
                splash.withholding_keys = ctx.input(|input| !input.keys_down.is_empty());
                ctx.input_mut(|input| {
                    input.events.retain(|event| {
                        !matches!(
                            event,
                            egui::Event::Key { .. } | egui::Event::Text(_) | egui::Event::Ime(_)
                        )
                    });
                });
            }
            return false;
        }
        let (now, key_pressed, clicked, closing) = ctx.input(|input| {
            let key_pressed = input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Key { pressed: true, .. }));
            let clicked = input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::PointerButton { pressed: true, .. }));
            (
                input.time,
                key_pressed,
                clicked,
                input.viewport().close_requested(),
            )
        });
        let opened_at = *splash.opened_at.get_or_insert(now);
        if closing || key_pressed || clicked || now - opened_at >= HOLD_SECONDS {
            splash.end(ctx, now);
            splash.withholding_keys = key_pressed;
            ctx.request_repaint();
            return (key_pressed || clicked) && !closing;
        }
        splash.show(ctx, self.themes.presented(appearance), appearance, now);
        true
    }

    ///
    /// Fades the logo over the console once the hold has ended, and drops the
    /// splash once the fade is done and no key held from the hold remains.
    ///
    pub(super) fn fade_splash(&mut self, ctx: &egui::Context, appearance: Appearance) {
        let Some(splash) = self.splash.as_ref() else {
            return;
        };
        let now = ctx.input(|input| input.time);
        if splash.opacity(now) > 0.0 {
            splash.show(ctx, self.themes.presented(appearance), appearance, now);
        } else if !splash.withholding_keys {
            self.splash = None;
        }
    }
}
