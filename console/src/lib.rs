#![warn(clippy::all)]

pub mod app;
pub mod console;
pub mod cursor;
pub mod glyph;
pub mod grid;
pub mod midi;
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
pub mod native_midi;
pub mod opts;
pub mod playback;
pub mod render_frame;
pub mod source;
pub mod style;
#[cfg(target_arch = "wasm32")]
pub mod web_startup;

#[cfg(test)]
mod test {

    use std::sync::Once;

    use tracing::debug;

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

    #[test]
    fn test_something() {
        trace();

        debug!("etc");
    }
}
