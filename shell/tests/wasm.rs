#![cfg(target_arch = "wasm32")]

use gloo_timers::future::TimeoutFuture;
use lang::PlayCommand;
use orcvs::app::Orcvs;
use orcvs::grid::Grid;
use orcvs::playback::{InMemoryOutputAdapter, PlaybackEngine, PlaybackState};
use orcvs::source::SourceCommander;
use shell::web_startup::{MISSING_CANVAS_MESSAGE, canvas_or_report};
use std::time::Duration;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn write(source: &SourceCommander, content: &str) {
    for (index, cell) in content.chars().enumerate() {
        source
            .set(index, &cell.to_string())
            .expect("valid Source cell");
    }
}

#[wasm_bindgen_test]
fn web_app_constructs_and_advances_the_cursor_without_panicking() {
    let mut app = Orcvs::new(2, 1);

    app.advance_cursor_blink();

    assert_eq!(app.render_frame().rows().len(), 1);
}

#[wasm_bindgen_test]
fn missing_canvas_reports_an_in_page_startup_error_without_panicking() {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .expect("browser test has a document");
    let loading_text = document
        .create_element("div")
        .expect("browser can create a loading element");

    assert!(canvas_or_report(None, Some(loading_text.clone())).is_none());
    assert_eq!(loading_text.inner_html(), MISSING_CANVAS_MESSAGE);
}

#[wasm_bindgen_test(async)]
async fn web_playback_dispatches_raw_play_through_the_terminal_output_spelling() {
    let source = SourceCommander::new(Grid::new(10, 2));
    // The Bang one row below the root anchor activates the Raw Play; without
    // it the terminal root emits nothing.
    write(&source, "!>007FC4  **");
    let adapter = InMemoryOutputAdapter::default();
    let engine = PlaybackEngine::new(source, adapter.clone());

    engine
        .start(Duration::from_millis(10))
        .expect("browser playback does not require a Tokio runtime");
    TimeoutFuture::new(20).await;

    assert_eq!(engine.observe().state, PlaybackState::Playing);
    assert!(adapter.command_lists().iter().any(|commands| commands
        == &[PlayCommand::Raw {
            channel: 0,
            velocity: 0x7F,
            note: 60,
        }]));
    engine.stop();
}

#[wasm_bindgen_test(async)]
async fn web_playback_evaluates_dot_family_arithmetic() {
    let source = SourceCommander::new(Grid::new(10, 2));
    write(&source, ".+0102");
    let engine = PlaybackEngine::new(source.clone(), InMemoryOutputAdapter::default());

    engine.start(Duration::from_millis(10)).unwrap();
    TimeoutFuture::new(20).await;

    assert_eq!(&source.snapshot()[10..12], "03");
    engine.stop();
}

#[wasm_bindgen_test(async)]
async fn web_playback_stop_cancels_ticks_and_restart_uses_a_new_generation() {
    let source = SourceCommander::new(Grid::new(10, 2));
    write(&source, "!>007FC4  **");
    let adapter = InMemoryOutputAdapter::default();
    let engine = PlaybackEngine::new(source, adapter.clone());

    engine.start(Duration::from_millis(10)).unwrap();
    TimeoutFuture::new(20).await;
    engine.stop();
    let stopped_count = adapter.command_lists().len();

    TimeoutFuture::new(20).await;
    assert_eq!(engine.observe().state, PlaybackState::Stopped);
    assert_eq!(adapter.command_lists().len(), stopped_count);

    engine.start(Duration::from_millis(10)).unwrap();
    TimeoutFuture::new(20).await;
    assert_eq!(engine.observe().state, PlaybackState::Playing);
    assert!(adapter.command_lists().len() > stopped_count);
    engine.stop();
}
