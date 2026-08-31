#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Once;

#[cfg(not(target_arch = "wasm32"))]
static INIT: Once = Once::new();

pub const DEFAULT_VIEW_SIZE: [f32; 2] = [800.0, 600.0];
pub const DEFAULT_VIEW_SIZE_MIN: [f32; 2] = [300.0, 220.0];

#[cfg(not(target_arch = "wasm32"))]
fn trace() {
    INIT.call_once(|| {
        use tracing_subscriber::FmtSubscriber;

        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG) // Set the maximum level of tracing events that should be logged.
            .with_line_number(true)
            .with_target(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> eframe::Result {
    use shell::console::Console;

    trace();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(DEFAULT_VIEW_SIZE)
            .with_min_inner_size(DEFAULT_VIEW_SIZE_MIN)
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),

        ..Default::default()
    };
    eframe::run_native(
        "Orcvs",
        native_options,
        Box::new(|cc| Ok(Box::new(Console::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use shell::console::Console;
    use shell::web_startup::canvas_or_report;
    use wasm_bindgen::JsCast;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();
    let document = web_sys::window().and_then(|window| window.document());
    let canvas = document
        .as_ref()
        .and_then(|document| document.get_element_by_id("the_canvas_id"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok());
    let loading_text = document.and_then(|document| document.get_element_by_id("loading_text"));
    let Some(canvas) = canvas_or_report(canvas, loading_text) else {
        log::error!("the_canvas_id is missing or is not an HTML canvas");
        return;
    };

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(Console::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        let loading_text = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));
        match start_result {
            Ok(_) => {
                if let Some(loading_text) = loading_text {
                    loading_text.remove();
                }
            }
            Err(e) => {
                log::error!("Failed to start eframe: {e:?}");
                if let Some(loading_text) = loading_text {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                }
                panic!("Failed to start eframe: {e:?}");
            }
        }
    });
}
