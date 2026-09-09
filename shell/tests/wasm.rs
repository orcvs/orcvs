#![cfg(target_arch = "wasm32")]

use gloo_timers::future::TimeoutFuture;
use lang::{MidiChannel, Note, Velocity};
use orcvs::app::Orcvs;
use orcvs::grid::Grid;
use orcvs::playback::{InMemoryOutputAdapter, OutputCommand, PlaybackEngine, PlaybackState};
use orcvs::source::{Source, SourceCommander, Tick};
use shell::web_startup::{MISSING_CANVAS_MESSAGE, canvas_or_report};
use std::time::Duration;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn write(source: &SourceCommander, content: &str) {
    let grid = source.grid();
    for (index, content) in content.chars().enumerate() {
        let cell = grid.cell_index(index).expect("inside the Grid");
        source
            .set(cell, &content.to_string())
            .expect("valid Source cell");
    }
}

///
/// `write` for the Source a Tick can be run against.
///
/// The Tick half of this file's memory regression needs a `Source`, not a
/// `SourceCommander`: `SourceCommander::execute` is `pub(crate)`, and the only
/// public way to spend a Tick from outside the crate is `PlaybackEngine::start`,
/// which is driven by a browser timer. A page count taken across a timed run
/// would depend on how many Ticks happened to fire, which is not a shape an
/// assertion can hold. `Source` is the object both the Commander and the engine
/// drive, so writing and Ticking it directly is the same work, synchronously.
///
fn write_source(source: &mut Source, content: &str) {
    let grid = source.grid();
    for (index, content) in content.chars().enumerate() {
        let cell = grid.cell_index(index).expect("inside the Grid");
        source
            .set(cell, &content.to_string())
            .expect("valid Source cell");
    }
}

///
/// The current size of wasm linear memory, in 64 KiB pages.
///
/// Linear memory never shrinks under `wasm32-unknown-unknown`: there is no
/// counterpart to `memory.grow`, so a freed allocation returns to the
/// allocator's free list rather than to the host. That monotonicity is what
/// makes this a sound leak signal rather than a weak one. The count cannot
/// drift downwards between two samples and cannot be lowered by a collection
/// that happened to run, so two equal samples prove no net growth over the span
/// between them, and any growth after warm-up is real growth.
///
fn pages() -> usize {
    core::arch::wasm32::memory_size(0)
}

///
/// A long run of Source writes and Ticks leaves linear memory at the size it
/// reached during warm-up.
///
/// This is the only memory signal the web target gets: neither Miri nor any
/// sanitizer reaches wasm. It lives in this file, which the merge tier already
/// runs under headless Firefox, so it needs no task and no workflow of its own.
///
#[wasm_bindgen_test]
fn web_linear_memory_settles_after_warm_up() {
    // Small enough that thousands of Ticks run in a headless browser in
    // seconds, and busy enough to be a real Tick: Equality writes a fresh Bang
    // into the middle row every Tick and the Bang fires Raw Play below it, so
    // each Tick plans, interprets, commits a Cell write, and rebuilds the
    // affected row's Language Map. Rewriting the whole program each iteration
    // drives the editing path over the same Source the Ticks read, which is the
    // shape a Console being typed into while playing has.
    const PROGRAM: &str = ".=0101              !>007FC4";
    // Warm-up length. For a workload that repeats identically, the allocator's
    // high-water mark is reached once the busiest single iteration has run
    // once: the wasm allocator's arena, the parser's and interpreter's
    // scratch buffers, and every collection's capacity all stop growing there,
    // which is a handful of iterations, not hundreds. 512 is two orders of
    // magnitude past that. It is deliberate slack rather than a number tuned to
    // a measurement, so a future path that needs a few more iterations to reach
    // its steady size does not turn this test red for the wrong reason.
    const WARM_UP_TICKS: u64 = 512;
    // The measured span is four times the warm-up, so it does strictly more
    // work than the span the first sample was taken after: a per-iteration leak
    // large enough to have grown memory at all during warm-up grows at least
    // four times as much here and cannot hide inside the page the warm-up
    // already paid for. At 28 Cell writes per Tick this span is 2,048 Ticks and
    // 57,344 Source writes, which puts the assertion's resolution at about 32
    // leaked bytes per Tick — that much is 64 KiB, one whole page.
    const MEASURED_TICKS: u64 = WARM_UP_TICKS * 4;

    let mut source = Source::new(Grid::new(10, 3));

    for tick in 0..WARM_UP_TICKS {
        write_source(&mut source, PROGRAM);
        source.execute(Tick::new(tick));
    }
    let settled = pages();

    for tick in WARM_UP_TICKS..WARM_UP_TICKS + MEASURED_TICKS {
        write_source(&mut source, PROGRAM);
        source.execute(Tick::new(tick));
    }

    assert_eq!(
        pages(),
        settled,
        "linear memory grew after warm-up, and it cannot shrink, so this is retained growth"
    );
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
    let source = SourceCommander::new(Grid::new(10, 3));
    // Equality produces a fresh Bang in the middle row, activating Raw Play
    // immediately below it on every Tick.
    write(&source, ".=0101              !>007FC4");
    let adapter = InMemoryOutputAdapter::default();
    let engine = PlaybackEngine::new(source, adapter.clone());

    engine
        .start(Duration::from_millis(10))
        .expect("browser playback does not require a Tokio runtime");
    TimeoutFuture::new(20).await;

    assert_eq!(engine.observe().state, PlaybackState::Playing);
    assert!(adapter.command_lists().iter().any(|commands| commands
        == &[OutputCommand::NoteOn {
            channel: MidiChannel::try_from(0).unwrap(),
            velocity: Velocity::try_from(0x7F).unwrap(),
            note: Note::try_from(60).unwrap()
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
    let source = SourceCommander::new(Grid::new(10, 3));
    write(&source, ".=0101              !>007FC4");
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
