#![warn(clippy::all)]

pub mod console;
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
mod midi;
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
