//!
//! The Orcvs logo, name and version that the splash and the About box show.
//!

use egui::Color32;

use crate::theme::Appearance;

/// The logo's side, in points, which fits the splash window.
pub(super) const SIDE: f32 = 270.0;

/// The name under the logo.
pub(super) const NAME: &str = "O R C V S";

/// The version under the name.
pub(super) const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

///
/// Loads the bundled logo as a texture: white, with its brightness as alpha.
///
pub(super) fn texture(ctx: &egui::Context) -> egui::TextureHandle {
    let logo = image::load_from_memory(include_bytes!("../../assets/logo-splash.png"))
        .expect("the bundled logo is a PNG")
        .into_rgba8();
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [logo.width() as usize, logo.height() as usize],
        logo.as_raw(),
    );
    ctx.load_texture("logo", image, egui::TextureOptions::LINEAR)
}

///
/// The tint the logo is drawn in: untinted it is the artwork on a dark
/// backdrop, and tinted black it reads on a light one.
///
pub(super) fn ink(appearance: Appearance) -> Color32 {
    match appearance {
        Appearance::Dark => Color32::WHITE,
        Appearance::Light => Color32::BLACK,
    }
}
