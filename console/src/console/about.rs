//!
//! Help → About: a modal over the console holding the logo, the name and the
//! version.
//!

use egui::{FontId, RichText, Vec2, load::SizedTexture};

use super::{Console, logo};
use crate::theme::Appearance;

impl Console {
    /// Opens the About box, loading the logo it shows.
    pub(super) fn open_about(&mut self, ctx: &egui::Context) {
        self.about_logo = Some(logo::texture(ctx));
    }

    ///
    /// Shows the About box while it is open. Escape and a click outside close
    /// it, dropping the logo's texture.
    ///
    pub(super) fn show_about(&mut self, ctx: &egui::Context, appearance: Appearance) {
        let Some(texture) = &self.about_logo else {
            return;
        };
        let theme = self.themes.presented(appearance);
        let modal = egui::Modal::new(egui::Id::new("about")).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Image::new(SizedTexture::new(texture.id(), Vec2::splat(logo::SIDE)))
                        .tint(logo::ink(appearance)),
                );
                ui.label(
                    RichText::new(logo::NAME)
                        .font(FontId::monospace(18.0))
                        .color(theme.text),
                );
                ui.label(
                    RichText::new(logo::VERSION)
                        .font(FontId::monospace(12.0))
                        .color(theme.text_muted),
                );
            });
        });
        if modal.should_close() {
            self.about_logo = None;
        }
    }
}
