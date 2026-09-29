#![cfg(target_arch = "wasm32")]

use console::cursor_effects::{CursorEffectAnimation, CursorEffectSettings};
use console::native_midi::NativeMidiBackend;
use console::web_startup::{MISSING_CANVAS_MESSAGE, canvas_or_report};
use gloo_timers::future::TimeoutFuture;
use lang::{MidiChannel, Note, Velocity};
use orcvs::app::Orcvs;
use orcvs::grid::Grid;
use orcvs::midi::{MidiBackend, MidiDestinationId};
use orcvs::playback::{
    InMemoryOutputAdapter, OutputAdapter, OutputAdapterError, OutputCommand, OutputOnlyAdapter,
    PlaybackEngine, PlaybackState,
};
use orcvs::source::{Source, SourceCommander, Tick};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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

    let mut source = Source::new(Grid::with_shape(10, 3));

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
fn web_app_and_cursor_effects_construct_without_panicking() {
    let app = Orcvs::with_shape(2, 1).expect("browser playback needs no Tokio runtime");
    assert_eq!(app.render_frame().grid().rows(), 1);

    let mut animation = CursorEffectAnimation::default();
    animation.advance(Duration::from_secs(1), CursorEffectSettings::default());
}

///
/// The browser's MIDI backend asks for Web MIDI access and answers discovery
/// and connect synchronously, whatever the browser does with the request: a
/// browser that has not answered says access is awaited, one without Web MIDI
/// or that refused it lists nothing, and one that granted it lists its ports.
/// None of them offers a port no browser names, and none panics. Which of the
/// three this browser is depends on its permission policy, so the test holds
/// all three rather than one.
///
#[wasm_bindgen_test]
fn web_midi_answers_discovery_and_connect_without_waiting() {
    const PENDING: &str = "waiting for the browser to grant MIDI access";
    let mut backend = NativeMidiBackend::new();
    const { assert!(console::native_midi::AVAILABLE) };

    match backend.destinations() {
        Ok(_) => {}
        Err(error) => {
            assert_eq!(error.message, PENDING);
            assert!(error.is_pending());
        }
    }
    assert!(
        backend
            .connect(&MidiDestinationId::new("no browser names this port"))
            .is_err()
    );
    // A second backend reuses the page's one request rather than asking again.
    let _ = NativeMidiBackend::new().destinations();
}

///
/// The page's wait for MIDI access before the console starts ends whether or
/// not the browser answers. A headless browser may leave the permission
/// request unanswered, and the wait must not hold the console back, so a wait
/// that ignored its timeout would hang this test.
///
#[wasm_bindgen_test]
async fn the_midi_access_wait_ends_whether_or_not_the_browser_answers() {
    console::native_midi::request_access_within(Duration::from_millis(50)).await;
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
    let source = SourceCommander::new(Grid::with_shape(10, 3));
    // Equality produces a fresh Bang in the middle row, activating Raw Play
    // immediately below it on every Tick.
    write(&source, ".=0101              !>007FC4");
    let adapter = InMemoryOutputAdapter::default();
    // Fallible and eager: the engine is a task, spawned here
    // rather than at the first `start`. The browser spawns onto the page's own
    // event loop, so what a Tokio runtime answers on the desktop is answered
    // by the page here.
    let engine = PlaybackEngine::new(source, adapter.clone())
        .expect("browser playback does not require a Tokio runtime");

    engine.start(Duration::from_millis(10)).unwrap();
    TimeoutFuture::new(20).await;

    assert_eq!(engine.state(), PlaybackState::Playing);
    assert!(adapter.command_lists().iter().any(|commands| commands
        == &[OutputCommand::NoteOn {
            channel: MidiChannel::try_from(0).unwrap(),
            velocity: Velocity::try_from(0x7F).unwrap(),
            note: Note::try_from(60).unwrap()
        }]));
    engine.stop();
}

// A browser timer would let the clock run during a stall. A synchronous wait
// deliberately holds the browser thread so the real clock must skip deadlines.
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = "
export function clock_now() { return performance.now(); }
export function stall_until(deadline) { while (performance.now() < deadline) {} }
")]
extern "C" {
    fn clock_now() -> f64;
    fn stall_until(deadline: f64);
}

///
/// An output adapter whose every submission costs a fixed slice of the browser
/// thread, so an executed Tick can be made to cost more than its own period.
///
/// The stall is synchronous on purpose: a Tick that yielded while it worked
/// would let the clock's own timer run and would prove nothing about the loop.
///
#[derive(Clone)]
struct StallingOutputAdapter {
    submissions: Arc<AtomicUsize>,
    cost_millis: f64,
}

impl OutputAdapter for StallingOutputAdapter {
    fn submit(&mut self, _commands: &[OutputCommand]) -> Result<(), OutputAdapterError> {
        self.submissions.fetch_add(1, Ordering::Relaxed);
        stall_until(clock_now() + self.cost_millis);
        Ok(())
    }

    fn safety_reset(&mut self) -> Result<(), OutputAdapterError> {
        Ok(())
    }
}

impl OutputOnlyAdapter for StallingOutputAdapter {}

#[wasm_bindgen_test(async)]
async fn web_clock_yields_to_the_event_loop_between_ticks() {
    let submissions = Arc::new(AtomicUsize::new(0));
    let engine = PlaybackEngine::new(
        SourceCommander::new(Grid::with_shape(1, 1)),
        StallingOutputAdapter {
            submissions: Arc::clone(&submissions),
            cost_millis: 115.0,
        },
    )
    .expect("browser playback does not require a Tokio runtime");
    // A Tick costs more than its period, so `observed_at` — sampled before the
    // Tick executes — leaves every deadline after the first already elapsed by
    // the time the clock reaches it. The clock must wait on those deadlines
    // anyway: that wait is its only yield back to the browser event loop, and
    // a loop that skipped it runs Tick after Tick with nothing rendering, no
    // input dispatched, and no turn in which `stop` could be pressed. The
    // period and the cost are far enough apart to be read through browser
    // jitter and close enough that several Ticks fit before `is_overrun`
    // declines one and re-anchors the grid ahead of the clock, ending the
    // burst; the whole burst lands inside this one timer.
    engine.start(Duration::from_millis(100)).unwrap();
    TimeoutFuture::new(0).await;
    engine.stop();
    let submissions = submissions.load(Ordering::Relaxed);
    assert!(
        submissions <= 2,
        "the clock spent {submissions} Ticks without releasing the browser thread"
    );
}

#[wasm_bindgen_test(async)]
async fn web_start_executes_its_first_tick_before_a_browser_timer() {
    let adapter = InMemoryOutputAdapter::default();
    let engine = PlaybackEngine::new(
        SourceCommander::new(Grid::with_shape(1, 1)),
        adapter.clone(),
    )
    .expect("browser playback does not require a Tokio runtime");
    // The period is long enough that only the first Tick can fall inside this
    // test, so the count answers where that Tick landed and nothing else. It
    // is the one-millisecond end of `Bpm` that makes the answer matter — a
    // Tick deferred behind a timer is declined there — but reproducing that
    // period here would race the clock's own timer against the test's.
    engine.start(Duration::from_secs(10)).unwrap();
    // spawn_local runs in a microtask, so the clock reaches its first deadline
    // before this timer, which was registered first, can fire. A zero-delay
    // timer in the clock would put that Tick behind this one instead.
    TimeoutFuture::new(0).await;
    engine.stop();
    assert_eq!(adapter.command_lists().len(), 1);
}

#[wasm_bindgen_test(async)]
async fn web_retune_keeps_the_deadline_grid_through_a_stall() {
    use orcvs::app::{InputEvent, InputKey};
    use orcvs::opts::Bpm;
    use orcvs::playback::PlaybackDiagnostic;

    let adapter = InMemoryOutputAdapter::default();
    let mut app = Orcvs::with_source_and_output_adapter(
        orcvs::source::Source::new(orcvs::grid::Grid::with_shape(1, 1)),
        adapter.clone(),
    )
    .expect("browser playback does not require a Tokio runtime");
    app.set_bpm(Bpm::new(1).unwrap()); // 15 seconds: only the immediate Tick runs.
    app.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
    TimeoutFuture::new(0).await;
    assert_eq!(adapter.command_lists().len(), 1);
    let started = clock_now();

    stall_until(started + 500.0);
    app.set_bpm(Bpm::new(15).unwrap()); // One second, anchored on that first Tick.
    TimeoutFuture::new(0).await; // Let the retuned clock establish its wait.
    assert_eq!(
        adapter.command_lists().len(),
        1,
        "retune does not begin a run"
    );

    stall_until(started + 3_500.0);
    TimeoutFuture::new(0).await;
    let first = app.drain_playback_diagnostics();
    assert_eq!(
        adapter.command_lists().len(),
        1,
        "a stalled Tick is declined"
    );
    let [
        PlaybackDiagnostic::Overrun {
            scheduled_at: first_due,
            ..
        },
    ] = first.as_slice()
    else {
        panic!("one stall must report one missed deadline: {first:?}");
    };
    // The clock starts about 500ms after Tick zero. Its first deadline must
    // still be at 1000ms, not 1000ms after retuning. Allow browser dispatch
    // jitter, but keep the two candidate grids 500ms apart.
    assert!(
        (Duration::from_millis(350)..Duration::from_millis(650)).contains(first_due),
        "retune moved the first deadline: {first_due:?}"
    );

    stall_until(started + 5_500.0);
    TimeoutFuture::new(0).await;
    let second = app.drain_playback_diagnostics();
    app.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
    let [
        PlaybackDiagnostic::Overrun {
            scheduled_at: second_due,
            ..
        },
    ] = second.as_slice()
    else {
        panic!("the second stall must name one deadline: {second:?}");
    };
    assert_eq!(
        *second_due - *first_due,
        Duration::from_secs(3),
        "the clock must resume at Tick zero + 4s, not replay 2s or rebase to 4.5s"
    );
    assert_eq!(
        adapter.command_lists().len(),
        1,
        "neither stall spends a Tick"
    );
}

///
/// Two Space events in one batch are a toggle and its cancellation, in the
/// browser where the engine's task provably cannot run between them.
///
/// `spawn_local` puts the engine's task on this thread, so it gets no turn
/// until the frame that is handling the batch returns to the event loop. Held
/// Space is how a user delivers such a batch: egui reports auto-repeat as
/// further key presses, and any frame longer than the repeat interval carries
/// two of them.
///
/// The third press is the control: it proves the pair above cancelled rather
/// than jamming, and that an empty list is not simply what this Source always
/// produces.
///
#[wasm_bindgen_test(async)]
async fn web_two_space_events_in_one_batch_leave_playback_stopped() {
    use orcvs::app::{InputEvent, InputKey};

    let adapter = InMemoryOutputAdapter::default();
    let mut app = Orcvs::with_source_and_output_adapter(
        orcvs::source::Source::new(orcvs::grid::Grid::with_shape(1, 1)),
        adapter.clone(),
    )
    .expect("browser playback does not require a Tokio runtime");

    app.event_handler(vec![
        InputEvent::KeyPressed(InputKey::Space),
        InputEvent::KeyPressed(InputKey::Space),
    ]);
    TimeoutFuture::new(0).await;
    assert_eq!(
        adapter.command_lists().len(),
        0,
        "a run cancelled inside its own batch delivers no Tick"
    );

    app.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
    TimeoutFuture::new(0).await;
    assert_eq!(
        adapter.command_lists().len(),
        1,
        "the next Space starts a run, whose first Tick is immediate"
    );

    app.event_handler(vec![InputEvent::KeyPressed(InputKey::Space)]);
}

#[wasm_bindgen_test(async)]
async fn web_playback_evaluates_dot_family_arithmetic() {
    let source = SourceCommander::new(Grid::with_shape(10, 2));
    write(&source, ".+0102");
    let engine = PlaybackEngine::new(source.clone(), InMemoryOutputAdapter::default())
        .expect("browser playback does not require a Tokio runtime");

    engine.start(Duration::from_millis(10)).unwrap();
    TimeoutFuture::new(20).await;

    assert_eq!(&source.snapshot()[10..12], "03");
    engine.stop();
}

#[wasm_bindgen_test(async)]
async fn web_playback_stop_ends_the_run_and_a_restart_begins_a_new_one() {
    let source = SourceCommander::new(Grid::with_shape(10, 3));
    write(&source, ".=0101              !>007FC4");
    let adapter = InMemoryOutputAdapter::default();
    let engine = PlaybackEngine::new(source, adapter.clone())
        .expect("browser playback does not require a Tokio runtime");

    engine.start(Duration::from_millis(10)).unwrap();
    TimeoutFuture::new(20).await;
    engine.stop();
    let stopped_count = adapter.command_lists().len();

    TimeoutFuture::new(20).await;
    assert_eq!(engine.state(), PlaybackState::Stopped);
    assert_eq!(adapter.command_lists().len(), stopped_count);

    engine.start(Duration::from_millis(10)).unwrap();
    TimeoutFuture::new(20).await;
    assert_eq!(engine.state(), PlaybackState::Playing);
    assert!(adapter.command_lists().len() > stopped_count);
    engine.stop();
}

///
/// The developer console, as a test can see it.
///
/// A browser build reports on `log`, which is the channel
/// `eframe::WebLogger` forwards to the developer console; a `tracing` event
/// alone reaches no subscriber on this target and is dropped. Standing in for
/// that logger is the only way a test sees what the console would show, so
/// every browser report this file asserts on comes back through here.
///
mod developer_console {
    use std::sync::Mutex;

    static REPORTED: Mutex<Vec<String>> = Mutex::new(Vec::new());

    struct CapturingLogger;

    impl log::Log for CapturingLogger {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            metadata.level() <= log::Level::Error
        }

        fn log(&self, record: &log::Record<'_>) {
            if self.enabled(record.metadata()) {
                REPORTED
                    .lock()
                    .expect("no test panics while holding the record list")
                    .push(record.args().to_string());
            }
        }

        fn flush(&self) {}
    }

    ///
    /// The error records `action` puts on `log`.
    ///
    /// `log::set_logger` takes the first logger installed and refuses the rest,
    /// which is why the record list is cleared here rather than at
    /// installation: the second test to run reuses the logger the first
    /// installed.
    ///
    pub fn records_from(action: impl FnOnce()) -> Vec<String> {
        log::set_logger(&CapturingLogger).ok();
        log::set_max_level(log::LevelFilter::Error);
        REPORTED
            .lock()
            .expect("no test panics while holding the record list")
            .clear();

        action();

        REPORTED
            .lock()
            .expect("no test panics while holding the record list")
            .clone()
    }
}

///
/// The browser end of the Playback failure report.
///
/// `Console::ui` hands the Playback diagnostics it drains to
/// `console::diagnostics::report_playback_failures` on every build without a
/// MIDI backend, and in the browser that report is the whole of what a
/// Playback failure produces: the Output control is disabled there
/// (`midi_output`), so the Panel shows no status. This holds the browser's
/// half of "a Playback failure is reported".
///
/// What it drives is the reporting path itself, with a real
/// `PlaybackDiagnostic` and the real failure decision: a diagnostic that is not
/// a failure has to stay silent, and one that is has to reach the developer
/// console. What it does not drive is `Console::ui` calling it. Console holds
/// its own `Orcvs` over its own adapter, and the browser build has no reachable
/// way to make that engine fail — `InMemoryOutputAdapter::fail_next_submission`
/// needs the adapter instance, and the zero Tick period that fails a start or a
/// retune is unreachable through `Bpm`, whose delay is at least one
/// millisecond. So the call in `ui()` is a one-line hand-off this test does not
/// cover.
///
mod playback_failure {
    use console::diagnostics::report_playback_failures;
    use orcvs::playback::PlaybackDiagnostic;
    use std::time::Duration;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::developer_console::records_from;

    #[wasm_bindgen_test]
    fn the_browser_reports_a_playback_failure() {
        let reported = records_from(|| {
            report_playback_failures(&[PlaybackDiagnostic::ClockFailure {
                message: "Playback clock terminated unexpectedly".to_owned(),
            }])
        });

        assert!(
            reported
                .iter()
                .any(|record| record == "Playback failure: Playback clock terminated unexpectedly"),
            "the browser build reported nothing a developer console would show: {reported:?}"
        );
    }

    #[wasm_bindgen_test]
    fn the_browser_reports_nothing_for_a_diagnostic_that_is_not_a_failure() {
        let reported = records_from(|| {
            report_playback_failures(&[PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_millis(750),
                observed_at: Duration::from_millis(1_600),
            }])
        });

        assert!(
            reported.is_empty(),
            "an Overrun is a skipped Tick, not a failure: {reported:?}"
        );
    }
}

///
/// One pass of the running console at its default view size over `events`,
/// answering the AccessKit tree it publishes, which is what a viewer's
/// assistive technology reads. `ctx` must have AccessKit enabled.
///
fn pass(
    ctx: &egui::Context,
    console: &mut console::console::Console,
    host: &mut eframe::Frame,
    events: Vec<egui::Event>,
) -> Vec<(egui::accesskit::NodeId, egui::accesskit::Node)> {
    use eframe::App as _;

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::from(console::console::DEFAULT_VIEW_SIZE),
        )),
        events,
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |root| console.ui(root, host));
    let tree = output
        .platform_output
        .accesskit_update
        .take()
        .expect("AccessKit is enabled, so every pass publishes its tree");
    output.drop_without_applying_deltas();
    tree.nodes
}

///
/// The browser build has no MIDI backend, and its Output control says so.
///
/// The running console is driven through `eframe::App::ui` with AccessKit on,
/// which is how a viewer's assistive technology reads it: the destination
/// ComboBox is present but disabled, and a click on it opens nothing, so no
/// Scan item is ever offered. A Playback failure on this build therefore has
/// no Panel status to land in and takes `playback_failure`'s path instead.
///
mod midi_output {
    use console::console::Console;
    use egui::accesskit::{Node, Role};
    use egui::{Event, Modifiers, PointerButton, Pos2};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::pass;

    #[wasm_bindgen_test]
    fn the_browser_output_control_is_disabled_and_offers_no_scan() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut console = Console::start(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("browser playback does not require a Tokio runtime");
        let mut host = eframe::Frame::_new_kittest();

        let nodes = pass(&ctx, &mut console, &mut host, Vec::new());
        let combo_boxes: Vec<&Node> = nodes
            .iter()
            .map(|(_, node)| node)
            .filter(|node| node.role() == Role::ComboBox)
            .collect();
        let [output] = combo_boxes[..] else {
            panic!("the console shows one Output control, not {combo_boxes:?}");
        };
        assert!(
            output.is_disabled(),
            "the browser build offers an Output control with nothing behind it"
        );

        let bounds = output.bounds().expect("the Output control is laid out");
        let centre = Pos2::new(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        let click = |pressed| {
            vec![Event::PointerButton {
                pos: centre,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }]
        };
        // A click lands on the release pass, which is where an enabled
        // ComboBox shows its list, so that pass and the one after it are the
        // two trees a Scan item could appear in.
        let _ = pass(
            &ctx,
            &mut console,
            &mut host,
            vec![Event::PointerMoved(centre)],
        );
        let _ = pass(&ctx, &mut console, &mut host, click(true));
        let released = pass(&ctx, &mut console, &mut host, click(false));
        let after = pass(&ctx, &mut console, &mut host, Vec::new());

        assert!(
            !egui::Popup::is_any_open(&ctx),
            "a click on the disabled Output control opened its list"
        );
        assert!(
            !released
                .iter()
                .chain(&after)
                .any(|(_, node)| node.label() == Some("Scan")),
            "the browser build offers Scan"
        );
    }
}

///
/// The browser end of the storage seam.
///
/// `console/src/persistence.rs` reports a discarded revision on two channels
/// because the two targets read different ones: the native binary installs a
/// `tracing` subscriber, and the browser build installs `eframe::WebLogger`,
/// which reads `log` and knows nothing of `tracing`. Every other persistence
/// test runs on the native target, where the `tracing` line alone is enough, so
/// only a test compiled for `wasm32` can hold the browser's half of "a
/// malformed stored value is discarded, and is reported".
///
#[cfg(feature = "persistence")]
mod discarded_revision {
    use console::console::Console;
    use eframe::App as _;
    use orcvs::grid::{COL_COUNT, ROW_COUNT};
    use orcvs::source::Source;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::developer_console::records_from;

    ///
    /// Storage holding bytes that are not the stored encoding at all, under the
    /// key the console reads.
    ///
    struct MalformedStorage;

    impl eframe::Storage for MalformedStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            (key == console::persistence::SOURCE_KEY).then(|| "not a stored Source".to_owned())
        }

        fn set_string(&mut self, _key: &str, _value: String) {}

        fn remove_string(&mut self, _key: &str) {}

        fn flush(&mut self) {}
    }

    ///
    /// Storage that keeps what it is given, so the revision the console saves
    /// can be read back and decoded.
    ///
    #[derive(Default)]
    struct RecordingStorage {
        stored: Option<String>,
    }

    impl eframe::Storage for RecordingStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            (key == console::persistence::SOURCE_KEY)
                .then_some(self.stored.clone())
                .flatten()
        }

        fn set_string(&mut self, key: &str, value: String) {
            if key == console::persistence::SOURCE_KEY {
                self.stored = Some(value);
            }
        }

        fn remove_string(&mut self, _key: &str) {}

        fn flush(&mut self) {}
    }

    #[wasm_bindgen_test]
    fn the_browser_reports_a_discarded_revision_and_starts_the_default_grid() {
        let storage = MalformedStorage;
        let mut cc = eframe::CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(&storage);
        let mut console = None;

        // The report is the whole of what the browser has: the console shows a
        // developer-console record or it shows nothing at all. A `tracing`
        // event alone reaches no subscriber on this target and is dropped.
        let reported = records_from(|| console = Some(Console::start(&cc)));
        let mut console = console
            .expect("the console was built inside the capture")
            .expect("browser playback does not require a Tokio runtime");

        assert!(
            reported.iter().any(|record| record.contains("discarded")),
            "the browser build reported nothing a developer console would show: {reported:?}"
        );

        // And the discard is whole: the console starts the ordinary default
        // Grid, which is the revision its next save stores.
        let mut saved = RecordingStorage::default();
        console.save(&mut saved);
        let started: Source = eframe::get_value(&saved, console::persistence::SOURCE_KEY)
            .expect("the console saved a revision");
        assert_eq!(started.grid().count(), COL_COUNT * ROW_COUNT);
        assert!(started.snapshot().bytes().all(|byte| byte == b' '));
    }
}

///
/// The shipped WASM backend is eframe `LocalStorage`: a thin wrapper over
/// `window.localStorage` with the raw key `orcvs_source`. These tests drive
/// that backend, not a custom in-memory stand-in.
///
#[cfg(feature = "persistence")]
mod product_path {
    use console::console::Console;
    use eframe::App as _;
    use orcvs::grid::{COL_COUNT, Grid, ROW_COUNT};
    use orcvs::source::{Source, Token};
    use wasm_bindgen_test::wasm_bindgen_test;

    ///
    /// eframe `LocalStorage` is this wrapper: every get, set, and remove
    /// reaches `window.localStorage` under the raw key.
    ///
    struct BrowserStorage;

    fn window_local_storage() -> web_sys::Storage {
        web_sys::window()
            .expect("a Window")
            .local_storage()
            .expect("localStorage is accessible")
            .expect("localStorage is present")
    }

    fn clear_orcvs_keys() {
        let storage = window_local_storage();
        let _ = storage.remove_item(console::persistence::SOURCE_KEY);
    }

    impl eframe::Storage for BrowserStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            window_local_storage().get_item(key).ok().flatten()
        }

        fn set_string(&mut self, key: &str, value: String) {
            let _ = window_local_storage().set_item(key, &value);
        }

        fn remove_string(&mut self, key: &str) {
            let _ = window_local_storage().remove_item(key);
        }

        fn flush(&mut self) {}
    }

    fn console_over(storage: &dyn eframe::Storage) -> Console {
        let mut cc = eframe::CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(storage);
        Console::start(&cc).expect("browser playback does not require a Tokio runtime")
    }

    fn edited_source() -> Source {
        let grid = Grid::new();
        let mut source = Source::new(grid);
        for (index, content) in ".+0102".chars().enumerate() {
            source
                .set(
                    grid.cell_index(index).expect("inside the Grid"),
                    &content.to_string(),
                )
                .expect("a Cell the Source accepts");
        }
        source
    }

    fn saved_revision(console: &mut Console) -> Source {
        let mut storage = BrowserStorage;
        console.save(&mut storage);
        eframe::get_value(&storage, console::persistence::SOURCE_KEY)
            .expect("the console saved a revision")
    }

    #[wasm_bindgen_test]
    fn a_console_save_restarts_from_the_browser_local_storage() {
        clear_orcvs_keys();

        let saved = edited_source();
        {
            let mut storage = BrowserStorage;
            eframe::set_value(&mut storage, console::persistence::SOURCE_KEY, &saved);
            let mut console = console_over(&storage);
            console.save(&mut storage);
        }

        let storage = BrowserStorage;
        let mut console = console_over(&storage);
        let restored = saved_revision(&mut console);
        assert_eq!(restored.snapshot(), saved.snapshot());
        assert_eq!(restored.grid().count(), COL_COUNT * ROW_COUNT);
        assert!(restored.grid().position(255, 255).is_some());
        assert!(restored.grid().position(256, 255).is_none());
        let add = restored
            .grid()
            .position(0, 0)
            .expect("the Add root is inside the Grid");
        assert_eq!(
            restored.language_map().token_at(add),
            Some(Token::Function),
            "a restore rebuilds the Token the console draws at the Add root"
        );

        clear_orcvs_keys();
    }

    #[wasm_bindgen_test]
    fn a_malformed_local_storage_revision_is_discarded_and_overwritten_by_the_next_save() {
        clear_orcvs_keys();
        window_local_storage()
            .set_item(console::persistence::SOURCE_KEY, "not a stored Source")
            .expect("localStorage accepts the malformed value");

        let storage = BrowserStorage;
        let mut console = console_over(&storage);
        let started = saved_revision(&mut console);
        assert_eq!(started.grid().count(), COL_COUNT * ROW_COUNT);
        assert!(started.snapshot().bytes().all(|byte| byte == b' '));

        clear_orcvs_keys();
    }
}

///
/// On the web the browser's page zoom is the console's one zoom: neither a
/// chord nor the View menu changes egui's zoom factor as well.
///
/// eframe's web runner turns egui's keyboard zoom off and sets its zoom factor
/// to 1.0 before it builds the app, and leaves the browser's default action for
/// command `+`, `=`, `-` and `0` alone, so the browser zooms the page and eframe
/// follows its device pixel ratio as the native pixels per point. Any egui zoom
/// factor multiplies that ratio, so these tests hold the console's half: it
/// leaves egui's keyboard zoom off, the chords as the web runner delivers them
/// leave egui's factor at 1.0, and the View menu offers no zoom of its own.
/// The browser's page zoom itself is not observable here: a synthetic key
/// event does not trigger it.
///
mod zoom {
    use console::console::Console;
    use egui::accesskit::{Node, NodeId, Role};
    use egui::{Event, Key, Modifiers, PointerButton, Pos2};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::pass;

    ///
    /// The console built over the options eframe's web runner sets, with
    /// AccessKit on so a pass answers its tree.
    ///
    fn web_console() -> (egui::Context, Console, eframe::Frame) {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        ctx.options_mut(|options| {
            options.zoom_with_keyboard = false;
            options.zoom_factor = 1.0;
        });
        let console = Console::start(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("browser playback does not require a Tokio runtime");
        (ctx, console, eframe::Frame::_new_kittest())
    }

    #[wasm_bindgen_test]
    fn the_zoom_chords_leave_the_zoom_to_the_browser() {
        let (ctx, mut console, mut host) = web_console();
        assert!(
            !ctx.options(|options| options.zoom_with_keyboard),
            "building the console turned egui's keyboard zoom on over the browser's"
        );

        for key in [Key::Plus, Key::Equals, Key::Minus, Key::Num0] {
            let chord = Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            };
            pass(&ctx, &mut console, &mut host, vec![chord]);
            pass(&ctx, &mut console, &mut host, Vec::new());
            assert_eq!(
                ctx.zoom_factor(),
                1.0,
                "command {key:?} zoomed egui as well as the browser's page"
            );
        }
    }

    ///
    /// The centre of the node labelled `label` in `nodes`.
    ///
    fn centre_of(nodes: &[(NodeId, Node)], label: &str) -> Pos2 {
        let (_, node) = nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("the console shows no {label:?}"));
        let bounds = node.bounds().expect("a shown node is laid out");
        Pos2::new(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        )
    }

    #[wasm_bindgen_test]
    fn the_view_menu_offers_no_zoom_over_the_browsers() {
        let (ctx, mut console, mut host) = web_console();
        let closed = pass(&ctx, &mut console, &mut host, Vec::new());
        let view = centre_of(&closed, "View");
        let click = |pressed| {
            vec![Event::PointerButton {
                pos: view,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }]
        };
        pass(
            &ctx,
            &mut console,
            &mut host,
            vec![Event::PointerMoved(view)],
        );
        pass(&ctx, &mut console, &mut host, click(true));
        pass(&ctx, &mut console, &mut host, click(false));
        let open = pass(&ctx, &mut console, &mut host, Vec::new());

        assert!(
            open.iter().any(|(_, node)| {
                node.role() == Role::CheckBox && node.label() == Some("Diagnostics")
            }),
            "the View menu did not open"
        );
        let zoom_items: Vec<&str> = open
            .iter()
            .filter_map(|(_, node)| node.label())
            .filter(|label| label.contains("Zoom"))
            .collect();
        assert!(
            zoom_items.is_empty(),
            "the web View menu offers egui zoom over the browser's page zoom: {zoom_items:?}"
        );
    }
}
