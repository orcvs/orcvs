#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod console;
pub mod coord;
pub mod cursor;
pub mod glyph;
pub mod source;

pub mod style;

use egui::Color32;

pub struct Color(Color32);

impl Color {
    const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(Color32::from_rgba_premultiplied(r, g, b, 255))
    }

    const fn with_alpha(self, a: u8) -> Self {
        Self(Color32::from_rgba_premultiplied(
            self.0.r(),
            self.0.g(),
            self.0.b(),
            a,
        ))
    }

    const fn build(self) -> Color32 {
        self.0
    }
}

#[cfg(test)]
mod test {

    use std::sync::Once;

    #[allow(dead_code)]
    static INIT: Once = Once::new();

    #[allow(dead_code)]
    pub fn trace() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_env_filter("debug")
                // .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .pretty()
                .init();
        });
    }
}
