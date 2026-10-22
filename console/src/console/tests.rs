use egui::{
    Color32, Event, Key, Modifiers, MouseWheelUnit, Pos2, Rect, Shape, TouchPhase, Vec2,
    emath::GuiRounding as _,
};
use orcvs::app::{Arrow, InputEvent, InputKey, Orcvs};
use orcvs::render_frame::RenderFrame;

use crate::grid_viewport::{CELL_SIZE, GridViewport, presented_grid, snapped_cell};
use crate::paint::{EdgeLine, EdgeRank, EdgeRun, Edges, FramePaint, Paint};
use crate::theme::{Theme, okabe_ito, orcvs_light};
use crate::theme_registry::ThemeRegistry;
use orcvs::grid::{COL_COUNT, Grid, Position, ROW_COUNT};

use super::diagnostics_panel::{
    TIMING_READOUT_INTERVAL_SECONDS, Timing, TimingReadouts, format_cpu_budget_delta,
    frames_per_second, visible_source_region,
};
use super::glyphs::{ALPHABET_FIRST, ALPHABET_LAST, GlyphTable};
use super::input::translate_event;
use super::menu_bar::TOP_PANEL_HEIGHT;
use super::panel::{BOTTOM_PANEL_HEIGHT, BOTTOM_PANEL_LEFT_PAD, BPM_FIELD_MARGIN};
use super::shapes::SourceShapes;
use super::source_view::{
    SOURCE_MARGIN_CELLS, SourceView, clamp_pan, is_presentable, show_source_scene,
    source_panel_frame,
};
use super::{Console, DEFAULT_VIEW_SIZE};

/// The Source View's margin at a device scale of one.
const MARGIN: Vec2 = Vec2::new(
    SOURCE_MARGIN_CELLS * CELL_SIZE.x,
    SOURCE_MARGIN_CELLS * CELL_SIZE.y,
);

/// Device scales the Source's own Cell is presented at: whole ones, and
/// fractional ones whose snap draws the Cell under its own points. egui folds its
/// zoom factor into `pixels_per_point`, so these are what a zoom reaches.
const DEVICE_SCALES: [f32; 5] = [0.3, 1.0, 1.1, 1.5, 2.0];

///
/// Asserts that `ctx` styles each appearance's chrome from the given
/// Theme, naming the `situation` in the failure.
///
pub(super) fn assert_chrome(ctx: &egui::Context, dark: &Theme, light: &Theme, situation: &str) {
    for (slot, theme) in [(egui::Theme::Dark, dark), (egui::Theme::Light, light)] {
        assert_eq!(
            ctx.style_of(slot).visuals,
            crate::style::style(theme).visuals,
            "{situation}: the {slot:?} appearance does not present {}'s chrome",
            theme.identity
        );
    }
}

fn key_event(key: Key, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn modified_key_event(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn command_key_event(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::COMMAND,
    }
}

#[test]
fn toolkit_events_translate_only_the_input_orcvs_handles() {
    let cases = [
        (Key::ArrowDown, InputKey::ArrowDown),
        (Key::ArrowLeft, InputKey::ArrowLeft),
        (Key::ArrowRight, InputKey::ArrowRight),
        (Key::ArrowUp, InputKey::ArrowUp),
        (Key::Backspace, InputKey::Backspace),
        (Key::Delete, InputKey::Delete),
        (Key::Space, InputKey::Space),
        (Key::Tab, InputKey::Tab),
    ];
    for (egui_key, orcvs_key) in cases {
        assert_eq!(
            translate_event(key_event(egui_key, true)),
            Some(InputEvent::KeyPressed(orcvs_key))
        );
    }

    // Shift Tab is its own InputEvent rather than the bare Tab above, as
    // Shift with an arrow is: the Cursor steps forward by Sector on one
    // and back on the other, and only the modifier tells them apart.
    assert_eq!(
        translate_event(modified_key_event(Key::Tab, Modifiers::SHIFT)),
        Some(InputEvent::PreviousSector)
    );

    // Only a bare Tab and a bare Shift Tab are the Source's: Ctrl,
    // Command, or Alt held with Tab is some other shortcut (or nothing)
    // and must not also step the Cursor.
    for modifiers in [
        Modifiers::COMMAND,
        Modifiers::CTRL,
        Modifiers::ALT,
        Modifiers::COMMAND | Modifiers::SHIFT,
        Modifiers::ALT | Modifiers::SHIFT,
    ] {
        assert_eq!(
            translate_event(modified_key_event(Key::Tab, modifiers)),
            None,
            "{modifiers:?} held with Tab translated to Source input"
        );
    }

    assert_eq!(
        translate_event(Event::Text("x".to_owned())),
        Some(InputEvent::Text("x".to_owned()))
    );
    assert_eq!(translate_event(key_event(Key::Enter, true)), None);
    assert_eq!(translate_event(key_event(Key::ArrowDown, false)), None);

    // Shift with an arrow extends the Region; command `A` spans the Grid;
    // Escape collapses the Region onto the Cursor.
    let shifted = [
        (Key::ArrowDown, Arrow::Down),
        (Key::ArrowLeft, Arrow::Left),
        (Key::ArrowRight, Arrow::Right),
        (Key::ArrowUp, Arrow::Up),
    ];
    for (egui_key, arrow) in shifted {
        assert_eq!(
            translate_event(modified_key_event(egui_key, Modifiers::SHIFT)),
            Some(InputEvent::Extend(arrow))
        );
    }
    assert_eq!(
        translate_event(command_key_event(Key::A)),
        Some(InputEvent::SelectAll)
    );
    // Command Enter arms a fill; a bare Enter is still nothing.
    assert_eq!(
        translate_event(command_key_event(Key::Enter)),
        Some(InputEvent::Fill)
    );
    assert_eq!(translate_event(key_event(Key::A, true)), None);
    assert_eq!(
        translate_event(key_event(Key::Escape, true)),
        Some(InputEvent::Collapse)
    );
    // The clipboard arrives as its own events, never as the characters
    // of the chord that raised it.
    assert_eq!(translate_event(Event::Copy), Some(InputEvent::Copy));
    assert_eq!(translate_event(Event::Cut), Some(InputEvent::Cut));
    assert_eq!(
        translate_event(Event::Paste("ab".to_owned())),
        Some(InputEvent::Paste("ab".to_owned()))
    );
    for key in [Key::C, Key::X, Key::V] {
        assert_eq!(translate_event(command_key_event(key)), None);
    }

    // Bare `+`, `-`, `=` and `0` are Source characters: the toolkit
    // reports them as `Event::Text`, which `translate_event` reaches
    // regardless of what key produced it, and never as one of the Key
    // variants matched above.
    for character in ["+", "-", "=", "0"] {
        assert_eq!(
            translate_event(Event::Text(character.to_owned())),
            Some(InputEvent::Text(character.to_owned())),
            "bare {character:?} did not reach the Source as Cell input"
        );
    }
    for key in [Key::Equals, Key::Plus, Key::Minus, Key::Num0] {
        assert_eq!(
            translate_event(key_event(key, true)),
            None,
            "bare {key:?} was translated as Source input on its own"
        );
    }
}

///
/// Command Shift R rewinds, once per press, and only with exactly those
/// modifiers: a bare R is a Source character, command R alone and command
/// Alt Shift R are nothing.
///
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn command_shift_r_alone_translates_to_a_rewind() {
    assert_eq!(
        translate_event(modified_key_event(
            Key::R,
            Modifiers::COMMAND | Modifiers::SHIFT
        )),
        Some(InputEvent::Rewind)
    );
    let repeated = Event::Key {
        key: Key::R,
        physical_key: None,
        pressed: true,
        repeat: true,
        modifiers: Modifiers::COMMAND | Modifiers::SHIFT,
    };
    assert_eq!(
        translate_event(repeated),
        None,
        "a held chord rewound again"
    );
    for modifiers in [
        Modifiers::NONE,
        Modifiers::SHIFT,
        Modifiers::COMMAND,
        Modifiers::COMMAND | Modifiers::SHIFT | Modifiers::ALT,
    ] {
        assert_eq!(
            translate_event(modified_key_event(Key::R, modifiers)),
            None,
            "{modifiers:?} held with R rewound"
        );
    }
}

///
/// The frame rate is derived. The visible Source region the diagnostics
/// show is the console area read back through the origin, and the guard is
/// what keeps that origin a value they can show.
///
#[test]
fn diagnostics_derive_frame_rate_and_read_the_visible_region_from_the_origin() {
    assert_eq!(frames_per_second(0.02), Some(50.0));
    assert_eq!(frames_per_second(0.0), None);

    // Panned 40 points into the Source and letterboxed 30 points below the
    // console's top, so each axis reads back with its own sign.
    let console = Rect::from_min_size(Pos2::new(0.0, 24.0), Vec2::new(800.0, 400.0));
    let origin = Pos2::new(-40.0, 54.0);
    assert!(is_presentable(origin));
    assert_eq!(
        visible_source_region(console, origin),
        Rect::from_min_size(Pos2::new(40.0, -30.0), Vec2::new(800.0, 400.0)),
        "the visible Source region is not the console area read back through the origin"
    );

    for unpresentable in [Pos2::new(0.0, f32::NAN), Pos2::new(f32::INFINITY, 0.0)] {
        assert!(
            !is_presentable(unpresentable),
            "{unpresentable:?} reached the diagnostics"
        );
    }
}

///
/// The timing readouts show the first frame at once, hold it for the
/// interval, then show the frame that arrives once it has passed.
///
#[test]
fn timing_readouts_hold_each_frame_for_the_interval() {
    let frame = |frame_time| Timing {
        frame_time,
        cpu_usage: Some(frame_time / 2.0),
    };
    let mut readouts = TimingReadouts::default();
    assert_eq!(readouts.observe(10.0, frame(0.016)), frame(0.016));
    assert_eq!(
        readouts.observe(10.1, frame(0.010)),
        frame(0.016),
        "a readout changed within the interval"
    );
    assert_eq!(
        readouts.observe(10.0 + TIMING_READOUT_INTERVAL_SECONDS, frame(0.030)),
        frame(0.030),
        "a readout held past the interval"
    );
    assert_eq!(
        readouts.observe(10.3, frame(0.040)),
        frame(0.030),
        "the readout did not hold after changing"
    );
}

#[test]
fn cpu_budget_delta_is_signed_against_the_sixty_fps_budget() {
    assert_eq!(format_cpu_budget_delta(Some(0.02)), "+3.33 ms");
    assert_eq!(format_cpu_budget_delta(Some(0.005)), "-11.67 ms");
    assert_eq!(format_cpu_budget_delta(Some(0.0)), "-16.67 ms");
    assert_eq!(format_cpu_budget_delta(None), "—");
}

///
/// A Glyph is laid out at the size its snapped Cell derives at every device
/// scale: 13.5 points at the Source's own 12 by 18 point Cell, and less where
/// the snap draws the Cell smaller. egui's `pixels_per_point`, which folds in
/// its zoom factor, is what makes it more physical pixels.
///
#[tokio::test]
async fn a_glyph_is_laid_out_at_its_cells_own_size_at_every_device_scale() {
    for pixels_per_point in [1.0_f32, 1.1, 1.5, 2.0] {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(4, 4);
        orcvs.select(orcvs.grid().position(1, 1).expect("inside the grid"));
        orcvs.write("+");
        let mut view = SourceView::new(CELL_SIZE);

        let (viewport, painted) = console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            pixels_per_point,
        );
        let sizes: Vec<f32> = painted
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(text) => Some(text.galley.job.sections[0].format.font_id.size),
                _ => None,
            })
            .collect();

        assert!(
            !sizes.is_empty(),
            "{pixels_per_point} ppp painted no Glyph, so it asserts nothing"
        );
        assert!(
            sizes.iter().all(|size| *size == viewport.glyph_size()),
            "{pixels_per_point} ppp laid Glyphs out at {sizes:?} points"
        );
        if pixels_per_point == 1.0 {
            assert_eq!(viewport.glyph_size(), 13.5);
        }
    }
}

///
/// One Render Frame, and everything it painted in paint order.
///
/// The shapes are flattened: what an assertion is about is the order the
/// Source Grid was painted in, not how deeply a `Shape::Vec` wrapped it.
///
fn console_pass(
    ctx: &egui::Context,
    screen: Rect,
    events: Vec<Event>,
    orcvs: &mut Orcvs,
    view: &mut SourceView,
) -> (GridViewport, Vec<Shape>) {
    console_pass_at(ctx, screen, events, orcvs, view, 1.0)
}

///
/// The same pass at a stated device scale.
///
/// The scale is given as the viewport's `native_pixels_per_point` rather
/// than through `Context::set_pixels_per_point`, which sets the zoom factor
/// instead and rewrites the next pass's `screen_rect` from the previous
/// one's to avoid jitter (in `ContextImpl::begin_pass`) — so the
/// console would not be the size the caller asked for.
///
fn console_pass_at(
    ctx: &egui::Context,
    screen: Rect,
    events: Vec<Event>,
    orcvs: &mut Orcvs,
    view: &mut SourceView,
    pixels_per_point: f32,
) -> (GridViewport, Vec<Shape>) {
    console_pass_moving(
        ctx,
        screen,
        events,
        orcvs,
        view,
        pixels_per_point,
        crate::cursor_effects::CursorEffectMotion::default(),
    )
}

/// The same pass with the Cursor Effect at `motion`, for the tests of a
/// Cursor Effect a settings file turns back on.
fn console_pass_moving(
    ctx: &egui::Context,
    screen: Rect,
    events: Vec<Event>,
    orcvs: &mut Orcvs,
    view: &mut SourceView,
    pixels_per_point: f32,
    motion: crate::cursor_effects::CursorEffectMotion,
) -> (GridViewport, Vec<Shape>) {
    let frame = orcvs.render_frame();
    let mut presented = None;
    let mut input = egui::RawInput {
        screen_rect: Some(screen),
        events,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .expect("the root viewport")
        .native_pixels_per_point = Some(pixels_per_point);
    let output = ctx.run_ui(input, |root| {
        egui::CentralPanel::default()
            .frame(source_panel_frame(
                crate::theme::okabe_ito().grid_background,
            ))
            .show(root, |ui| {
                presented = Some(show_source_scene(
                    ui,
                    &frame,
                    &egui::FontFamily::Monospace,
                    view,
                    motion,
                    &crate::theme::okabe_ito(),
                ));
            });
    });
    let mut painted = Vec::new();
    for clipped in &output.shapes {
        flatten(clipped.shape.clone(), &mut painted);
    }
    output.drop_without_applying_deltas();
    let presented = presented.expect("the central panel showed the Source");
    // What `Console::ui` does with the answer, done here for the same
    // reason: the Source Grid reports the Cell and the caller that owns the
    // running Orcvs moves the Cursor to it.
    if let Some(selection) = presented.selection {
        selection.apply(orcvs);
    }

    (presented.viewport, painted)
}

fn flatten(shape: Shape, into: &mut Vec<Shape>) {
    match shape {
        Shape::Vec(shapes) => {
            for shape in shapes {
                flatten(shape, into);
            }
        }
        shape => into.push(shape),
    }
}

fn console_frame(
    ctx: &egui::Context,
    screen: Rect,
    events: Vec<Event>,
    orcvs: &mut Orcvs,
    view: &mut SourceView,
) -> GridViewport {
    console_pass(ctx, screen, events, orcvs, view).0
}

fn click_at(point: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(point),
        Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
    ]
}

fn release_at(point: Pos2) -> Vec<Event> {
    vec![Event::PointerButton {
        pos: point,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    }]
}

fn click(ctx: &egui::Context, screen: Rect, point: Pos2, orcvs: &mut Orcvs, view: &mut SourceView) {
    console_frame(ctx, screen, click_at(point), orcvs, view);
    console_frame(ctx, screen, release_at(point), orcvs, view);
}

///
/// The middle button pressed at `point`, which is the button that pans.
///
fn middle_press_at(point: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(point),
        Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Middle,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
    ]
}

///
/// The primary button pressed at `point` while Alt (Option) is held —
/// the gesture that Pans without a middle button.
///
/// `Event::ModifiersChanged` first, the way a real backend reports the
/// Option key going down: `ui.input(|i| i.modifiers)` is carried across
/// frames from that event alone (`egui`'s own `InputState::begin_pass`),
/// not from a `PointerButton` event's own `modifiers` field, so a
/// `PointerMoved`-only frame later in the same drag still reads Alt as
/// held only because of this.
///
fn alt_primary_press_at(point: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(point),
        Event::ModifiersChanged(Modifiers::ALT),
        Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::ALT,
        },
    ]
}

///
/// Alt released along with the primary button at `point`, ending an
/// Alt-drag Pan.
///
fn alt_primary_release_at(point: Pos2) -> Vec<Event> {
    vec![
        Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::ALT,
        },
        Event::ModifiersChanged(Modifiers::NONE),
    ]
}

///
/// A pinch over `point`. Zoom is from the keyboard alone, so this must
/// leave the Cell size where it was.
///
fn pinch_at(point: Pos2) -> Vec<Event> {
    vec![Event::PointerMoved(point), Event::Zoom(1.2)]
}

///
/// A two-finger or wheel Pan over `point`.
///
fn wheel_at(point: Pos2, delta: Vec2) -> Vec<Event> {
    vec![
        Event::PointerMoved(point),
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta,
            phase: TouchPhase::Move,
            modifiers: Modifiers::NONE,
        },
    ]
}

///
/// A command-wheel over `point`. Zoom is from the keyboard alone, so this
/// must leave the Cell size where it was.
///
fn command_wheel_at(point: Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(point),
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(0.0, 80.0),
            phase: TouchPhase::Move,
            modifiers: Modifiers::COMMAND,
        },
    ]
}

fn double_click(
    ctx: &egui::Context,
    screen: Rect,
    point: Pos2,
    orcvs: &mut Orcvs,
    view: &mut SourceView,
) {
    click(ctx, screen, point, orcvs, view);
    click(ctx, screen, point, orcvs, view);
}

///
/// A point in the surplus the viewport does not cover, if there is any.
///
fn surplus(screen: Rect, viewport: Rect) -> Option<Pos2> {
    if screen.width() > viewport.width() + 1.0 {
        Some(Pos2::new(
            (viewport.right() + screen.right()) / 2.0,
            viewport.center().y,
        ))
    } else if screen.height() > viewport.height() + 1.0 {
        Some(Pos2::new(
            viewport.center().x,
            (viewport.bottom() + screen.bottom()) / 2.0,
        ))
    } else {
        None
    }
}

///
/// Wheels by `delta` at `point` frame after frame until the Pan stops
/// moving, so a test asserts where a wheel Pan settles rather than how far
/// `smooth_scroll_delta` hands it out in any one frame.
///
fn wheel_until_settled(
    ctx: &egui::Context,
    screen: Rect,
    point: Pos2,
    delta: Vec2,
    orcvs: &mut Orcvs,
    view: &mut SourceView,
) {
    for _ in 0..64 {
        let before = view.pan;
        console_frame(ctx, screen, wheel_at(point, delta), orcvs, view);
        if view.pan == before {
            return;
        }
    }
    panic!(
        "a wheel Pan was still moving after 64 frames: {:?}",
        view.pan
    );
}

fn running_orcvs(cols: usize, rows: usize) -> Orcvs {
    Orcvs::with_shape(cols, rows).expect("the test runtime")
}

fn selected_cell(orcvs: &Orcvs) -> (usize, usize) {
    let cursor = orcvs.render_frame().cursor();
    (cursor.x(), cursor.y())
}

#[tokio::test]
async fn a_click_selects_the_cell_under_the_pointer_whatever_the_window_size() {
    for screen_size in [
        Vec2::new(400.0, 200.0),
        Vec2::new(200.0, 400.0),
        Vec2::new(600.0, 300.0),
    ] {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
        let mut orcvs = running_orcvs(4, 4);
        let mut view = SourceView::new(CELL_SIZE);

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(selected_cell(&orcvs), (0, 0));
        assert_eq!(
            viewport.cell_size, CELL_SIZE,
            "a {screen_size:?} console opened at a Cell size other than the Source's own"
        );

        let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
        click(&ctx, screen, target, &mut orcvs, &mut view);

        assert_eq!(
            selected_cell(&orcvs),
            (3, 1),
            "a click at {target:?} in a {screen_size:?} console"
        );
    }
}

///
/// A console too small for the Source still opens at the Source's own Cell size and shows the
/// top-left of the Grid. A click still selects the Cell under the pointer.
///
#[tokio::test]
async fn a_click_selects_the_cell_under_the_pointer_in_a_console_smaller_than_the_source() {
    for screen_size in [Vec2::new(400.0, 102.0), Vec2::new(102.0, 400.0)] {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::new(CELL_SIZE);

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            viewport.cell_size, CELL_SIZE,
            "a {screen_size:?} console opened at a Cell size other than the Source's own"
        );

        let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
        click(&ctx, screen, target, &mut orcvs, &mut view);

        assert_eq!(
            selected_cell(&orcvs),
            (3, 1),
            "a click at {target:?} in a {screen_size:?} console"
        );
    }
}

///
/// Starts a Console from `cc` over `themes` and the settings `config` holds,
/// the way eframe starts one. Every console test builds its Console here,
/// whether it drives it over its own `Context`, through a kittest `Harness`
/// or from a stored revision.
///
pub(super) fn start_console(
    cc: &eframe::CreationContext<'_>,
    themes: ThemeRegistry,
    config: crate::config::Config,
) -> Console {
    Console::new(cc, themes, config).expect("the test runtime")
}

///
/// A fresh-install Console over `ctx`: the built-in Themes, the default
/// settings and no storage. `CreationContext::_new_kittest` is eframe's own
/// headless constructor, which is how an `App` starts outside a window.
///
fn console_on(ctx: &egui::Context) -> Console {
    start_console(
        &eframe::CreationContext::_new_kittest(ctx.clone()),
        ThemeRegistry::built_in(),
        crate::config::Config::default(),
    )
}

///
/// A fresh-install Console over a new `Context`, with eframe's headless host
/// to drive it through [`app_pass`].
///
fn fresh_console() -> (egui::Context, Console, eframe::Frame) {
    let ctx = egui::Context::default();
    let console = console_on(&ctx);
    (ctx, console, eframe::Frame::_new_kittest())
}

///
/// Waits, bounded by [`ENGINE_WAIT`], until `watch` answers `ready`. Playback
/// runs on its own task, so this waits on a fact only that task can
/// make true, never on a clock.
///
pub(super) async fn engine_reaches(
    watch: &mut orcvs::playback::PlaybackObservationWatch,
    ready: impl FnMut(&orcvs::playback::PlaybackObservation) -> bool,
) -> bool {
    tokio::time::timeout(ENGINE_WAIT, watch.wait_for(ready))
        .await
        .is_ok_and(|reached| reached.is_ok())
}

/// How long [`engine_reaches`] and a closing engine are given: far longer than
/// a current-thread task needs, so running out is a failure.
pub(super) const ENGINE_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

///
/// One pass of the whole running Console, the way eframe drives it.
///
/// `eframe::Frame::_new_kittest` and `CreationContext::_new_kittest` are
/// eframe's own headless constructors, which is how an `App` runs outside a
/// window; `storage_tests` reaches for the same pair for the same reason.
///
fn app_pass(
    ctx: &egui::Context,
    screen: Rect,
    events: Vec<Event>,
    console: &mut Console,
    host: &mut eframe::Frame,
) {
    use eframe::App as _;

    let input = egui::RawInput {
        screen_rect: Some(screen),
        events,
        ..Default::default()
    };
    let output = ctx.run_ui(input, |root| console.ui(root, host));
    output.drop_without_applying_deltas();
}

///
/// One pass, answering how soon the console asked to be painted again.
///
/// The Panel paints a published Tick. A delay started from this frame is
/// a second clock; an immediate delay after a publish is the wake.
///
fn app_pass_repaint_delay(
    ctx: &egui::Context,
    screen: Rect,
    events: Vec<Event>,
    console: &mut Console,
    host: &mut eframe::Frame,
) -> std::time::Duration {
    use eframe::App as _;

    let input = egui::RawInput {
        screen_rect: Some(screen),
        events,
        ..Default::default()
    };
    let output = ctx.run_ui(input, |root| console.ui(root, host));
    let delay = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .map(|viewport| viewport.repaint_delay)
        .unwrap_or(std::time::Duration::MAX);
    output.drop_without_applying_deltas();
    delay
}

///
/// Runs passes until one asks for no immediate repaint, so a zero delay a
/// later pass answers is the Playback wake-up's and not the settling of
/// an Open or a key press.
///
fn settle_repaint(
    ctx: &egui::Context,
    screen: Rect,
    console: &mut Console,
    host: &mut eframe::Frame,
) {
    for _ in 0..16 {
        if app_pass_repaint_delay(ctx, screen, Vec::new(), console, host)
            > std::time::Duration::ZERO
        {
            return;
        }
    }
    panic!("the console still asked for an immediate repaint after 16 quiet passes");
}

///
/// Plays at 200 BPM, settles the passes the key press asks for, and waits
/// for Playback to publish another Tick. Answers the Tick before, the Tick
/// after, and how soon the next pass asked to be painted again.
///
async fn play_and_await_the_next_tick(
    ctx: &egui::Context,
    screen: Rect,
    console: &mut Console,
    host: &mut eframe::Frame,
) -> (
    orcvs::source::Tick,
    orcvs::source::Tick,
    std::time::Duration,
) {
    console
        .orcvs
        .set_bpm(orcvs::opts::Bpm::new(200).expect("200 is in range"));
    app_pass(
        ctx,
        screen,
        vec![key_event(Key::Space, true)],
        console,
        host,
    );
    let mut playback = console.orcvs.playback_observation_watch();
    assert!(
        engine_reaches(&mut playback, |observation| {
            observation.state == orcvs::playback::PlaybackState::Playing
        })
        .await,
        "Playback never began playing"
    );

    settle_repaint(ctx, screen, console, host);
    let tick = console.orcvs.playback_observation().tick;
    assert!(
        engine_reaches(&mut playback, |observation| observation.tick != tick).await,
        "Playback never published another Tick from {tick:?}"
    );
    let advanced = console.orcvs.playback_observation().tick;

    let delay = app_pass_repaint_delay(ctx, screen, Vec::new(), console, host);
    (tick, advanced, delay)
}

fn collect_shape_text(shape: &Shape, out: &mut String) {
    match shape {
        Shape::Text(text) => {
            out.push_str(text.galley.text());
            out.push(' ');
        }
        Shape::Vec(shapes) => {
            for nested in shapes {
                collect_shape_text(nested, out);
            }
        }
        _ => {}
    }
}

fn collect_shape_strokes(shape: &Shape, out: &mut Vec<(Color32, Rect)>) {
    match shape {
        Shape::Rect(rect) if rect.stroke.width > 0.0 => {
            out.push((rect.stroke.color, rect.rect));
        }
        Shape::LineSegment { stroke, points } => {
            out.push((stroke.color, Rect::from_two_pos(points[0], points[1])));
        }
        Shape::Vec(shapes) => {
            for nested in shapes {
                collect_shape_strokes(nested, out);
            }
        }
        _ => {}
    }
}

fn painted_strokes(ctx: &egui::Context) -> Vec<(Color32, Rect)> {
    let mut strokes = Vec::new();
    let layers = [
        egui::LayerId::background(),
        egui::LayerId::new(egui::Order::Background, egui::Id::NULL),
        egui::LayerId::new(egui::Order::Middle, egui::Id::NULL),
        egui::LayerId::new(egui::Order::Background, egui::Id::new("bottom_panel")),
        egui::LayerId::new(egui::Order::Middle, egui::Id::new("bottom_panel")),
        egui::LayerId::new(egui::Order::Background, egui::Id::new("top_panel")),
        egui::LayerId::new(egui::Order::Middle, egui::Id::new("top_panel")),
    ];
    ctx.graphics(|graphics| {
        for layer in layers {
            if let Some(list) = graphics.get(layer) {
                for clipped in list.all_entries() {
                    collect_shape_strokes(&clipped.shape, &mut strokes);
                }
            }
        }
    });
    strokes
}

fn collect_shape_text_spans(shape: &Shape, out: &mut Vec<(String, f32, f32)>) {
    match shape {
        Shape::Text(text) => {
            let left = text.pos.x;
            out.push((
                text.galley.text().to_owned(),
                left,
                left + text.galley.size().x,
            ));
        }
        Shape::Vec(shapes) => {
            for nested in shapes {
                collect_shape_text_spans(nested, out);
            }
        }
        _ => {}
    }
}

fn painted_text(ctx: &egui::Context) -> String {
    let mut text = String::new();
    let layers = [
        egui::LayerId::background(),
        egui::LayerId::new(egui::Order::Background, egui::Id::new("bottom_panel")),
        egui::LayerId::new(egui::Order::Background, egui::Id::new("top_panel")),
        egui::LayerId::new(egui::Order::Middle, egui::Id::new("bottom_panel")),
    ];
    ctx.graphics(|graphics| {
        for layer in layers {
            if let Some(list) = graphics.get(layer) {
                for clipped in list.all_entries() {
                    collect_shape_text(&clipped.shape, &mut text);
                }
            }
        }
    });
    text
}

///
/// Where a running Console presented its Source Grid, asked of the
/// transform the pass left behind.
///
/// The same call `show_source_scene` makes, and for the reason the
/// `presented` helper cannot serve here: that one fits the Grid to a console
/// area, and a running Console's console area is the screen less whatever
/// height the menu bar settled the top panel at. The stored transform
/// already carries that fit, so this asks it rather than re-deriving it.
///
fn console_viewport(ctx: &egui::Context, console: &Console) -> GridViewport {
    let grid = console.orcvs.render_frame().grid();

    presented_grid(
        console.source_view.origin,
        grid,
        snapped_cell(console.source_view.cell, ctx.pixels_per_point()),
        ctx.pixels_per_point(),
    )
}

///
/// Clicking a Cell of a running Console moves the Cursor onto it.
///
/// Every other click test goes through `console_pass_at`, which shows the
/// Source itself and applies the answered Position itself — so none of them
/// reaches the `orcvs.select` call `Console::ui` owns, and all of them pass
/// with that call deleted. `show_source` answers the Cell rather than
/// selecting it, which is what makes clicking a Cell a wiring question at
/// all, and this is the one test that asks it: a real `Console`, driven
/// through `eframe::App::ui` over an egui pass, with nothing between the
/// pointer and the Cursor but the console's own code.
///
/// `storage_tests` exists for the same reason one module below, and says
/// so: an interface-level test cannot see whether the Console is wired to
/// the interface at all.
///
#[tokio::test]
async fn a_click_on_a_cell_moves_the_cursor_of_a_running_console() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    // One quiet pass, so the top panel has claimed its height and the view
    // holds the fit the Grid was presented under.
    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    assert_eq!(
        selected_cell(&console.orcvs),
        (0, 0),
        "a fresh Console did not open with the Cursor in the corner"
    );

    let viewport = console_viewport(&ctx, &console);
    let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
    // Pressed on one pass and released on the next, because a click is
    // reported on the release.
    app_pass(&ctx, screen, click_at(target), &mut console, &mut host);
    app_pass(&ctx, screen, release_at(target), &mut console, &mut host);

    assert_eq!(
        selected_cell(&console.orcvs),
        (3, 1),
        "a click at {target:?} on a Grid presented at {:?}",
        viewport.rect
    );
}

///
/// A Console started with a `[cell]` setting presents its Grid, its Pan
/// bound and its Glyphs at that Cell snapped, rather than at the default one:
/// a click lands on the Cell drawn at the snapped width and height, a Pan to
/// the far edge stops two snapped Cell widths past the Grid, the Glyphs are
/// laid out at the size the snapped Cell derives, and opening another Source
/// keeps it. The configured Cell is fractional, so each check tells the
/// snapped Cell from the one the file asked for.
///
#[tokio::test]
async fn a_console_started_with_a_configured_cell_presents_the_source_at_it() {
    let configured = Vec2::new(10.5, 15.5);
    let ctx = egui::Context::default();
    let mut console = start_console(
        &eframe::CreationContext::_new_kittest(ctx.clone()),
        ThemeRegistry::built_in(),
        crate::config::Config {
            cell: configured,
            ..crate::config::Config::default()
        },
    );
    let mut host = eframe::Frame::_new_kittest();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    console.orcvs = running_orcvs(200, 100);
    console.orcvs.select(
        console
            .orcvs
            .grid()
            .position(1, 1)
            .expect("inside the grid"),
    );
    console.orcvs.write("+");

    let output = {
        use eframe::App as _;
        let input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        ctx.run_ui(input, |root| console.ui(root, &mut host))
    };
    let viewport = console_viewport(&ctx, &console);
    let cell = crate::grid_viewport::snapped_cell(configured, ctx.pixels_per_point());
    assert_ne!(cell, configured, "the configured Cell snaps to itself");
    assert_eq!(viewport.cell_size, cell);
    let shapes = output.shapes.clone();
    output.drop_without_applying_deltas();

    fn glyph_sizes(shape: &Shape, sizes: &mut Vec<f32>) {
        match shape {
            Shape::Text(text) => {
                if let Some(section) = text.galley.job.sections.first() {
                    sizes.push(section.format.font_id.size);
                }
            }
            Shape::Vec(shapes) => shapes.iter().for_each(|shape| glyph_sizes(shape, sizes)),
            _ => {}
        }
    }
    let mut glyphs = Vec::new();
    for clipped in &shapes {
        glyph_sizes(&clipped.shape, &mut glyphs);
    }
    assert!(
        glyphs.contains(&viewport.glyph_size()),
        "no Glyph was laid out at the configured Cell's {} points: {glyphs:?}",
        viewport.glyph_size()
    );
    let default = presented_grid(
        Pos2::ZERO,
        console.orcvs.grid(),
        crate::grid_viewport::snapped_cell(CELL_SIZE, ctx.pixels_per_point()),
        ctx.pixels_per_point(),
    );
    assert_ne!(
        viewport.glyph_size(),
        default.glyph_size(),
        "the Glyph is the default Cell's"
    );

    let target = viewport.rect.min + Vec2::new(3.5, 1.5) * cell;
    app_pass(&ctx, screen, click_at(target), &mut console, &mut host);
    app_pass(&ctx, screen, release_at(target), &mut console, &mut host);
    assert_eq!(selected_cell(&console.orcvs), (3, 1));

    // A quiet pass first, so the Cursor the click moved is followed before
    // the Pan is pinned, and does not pull the Pan back to it.
    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    pinned_at(&mut console.source_view, Vec2::splat(-1_000_000.0));
    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    let viewport = console_viewport(&ctx, &console);
    assert_eq!(
        screen.max.x - viewport.rect.max.x,
        SOURCE_MARGIN_CELLS * cell.x,
        "the far-edge Pan stopped somewhere other than two snapped Cells past the Grid"
    );

    console.load_function_reference();
    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    assert_eq!(
        console_viewport(&ctx, &console).cell_size,
        cell,
        "opening another Source forgot the configured Cell"
    );
}

///
/// eframe restores egui memory — `ThemePreference` included — before it
/// calls `Console::new`, on native and on web. Standing in for that
/// restore: setting the preference on a fresh `Context` before
/// `Console::new` runs. `install` must not overwrite whatever this set.
///
/// Each egui slot holds its own appearance's default built-in, so the
/// restored Light preference presents Orcvs Light rather than egui's own
/// default light style. The style comparison is `Visuals`, not
/// `Style`'s own `PartialEq`: `Style::number_formatter` compares by
/// `Arc::ptr_eq` (`NumberFormatter`'s `PartialEq`), so two
/// independently built `Style::default()`s never compare equal on that
/// field alone, whatever their visible content.
///
#[tokio::test]
async fn console_new_keeps_a_theme_preference_already_on_the_context() {
    let ctx = egui::Context::default();
    ctx.set_theme(egui::ThemePreference::Light);

    let _console = console_on(&ctx);

    assert_eq!(
        ctx.options(|options| options.theme_preference),
        egui::ThemePreference::Light,
        "Console::new overwrote the restored theme preference"
    );

    assert_chrome(
        &ctx,
        &okabe_ito(),
        &orcvs_light(),
        "Console::new under a restored Light preference",
    );
}

///
/// A fresh `Context` restores nothing, so its `ThemePreference` starts at
/// egui's own default, `System`. `Console::new` must not force `Dark` —
/// a build without the `persistence` feature has nothing else that would
/// set a preference, so `System` is what it opens with.
///
#[tokio::test]
async fn console_new_leaves_a_fresh_context_on_the_system_preference() {
    let (ctx, _console, _host) = fresh_console();

    assert_eq!(
        ctx.options(|options| options.theme_preference),
        egui::ThemePreference::System,
        "Console::new set a theme preference a fresh context never asked for"
    );
}

///
/// Before the first Playback run the Panel shows B `120 //`, T `00000`,
/// C `00:00`, O `None`. File, View and Help are the top bar's menus, in
/// that order.
///
#[tokio::test]
async fn the_bottom_panel_shows_tick_zero_and_run_clock_before_the_first_run() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    let painted = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let sink = painted.clone();
    ctx.on_end_pass(
        "capture-panel-text",
        std::sync::Arc::new(move |ctx| {
            *sink.lock().unwrap() = painted_text(ctx);
        }),
    );

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);

    let text = painted.lock().unwrap().clone();
    let bpm_at = text
        .find('B')
        .expect("the Panel is missing the B label in {text:?}");
    let beat_at = text
        .find("//")
        .expect("the Panel is missing the rest marker in {text:?}");
    let tick_at = beat_at
        + text[beat_at..]
            .find('T')
            .expect("the Panel is missing the T label in {text:?}");
    let clock_at = beat_at
        + text[beat_at..]
            .find('C')
            .expect("the Panel is missing the C label in {text:?}");
    assert!(
        !text.contains("**"),
        "the rest Panel still shows the beat marker in {text:?}"
    );
    assert!(
        bpm_at < beat_at && beat_at < tick_at && tick_at < clock_at,
        "Readout order is not B then // then T then C in {text:?}"
    );
    assert!(
        text.contains("120"),
        "the Panel is missing BPM 120 in {text:?}"
    );
    assert!(
        text.contains("00000"),
        "the Panel is missing Tick 00000 in {text:?}"
    );
    assert!(
        text.contains("00:00"),
        "the Panel is missing Run Clock 00:00 in {text:?}"
    );
    let mut previous_menu = 0;
    for menu in ["File", "View", "Help"] {
        let at = text
            .find(menu)
            .unwrap_or_else(|| panic!("the top bar is missing {menu} in {text:?}"));
        assert!(
            previous_menu <= at,
            "the top bar does not hold {menu} in order in {text:?}"
        );
        previous_menu = at;
    }
    assert!(
        !text.contains("Theme"),
        "the Theme menu is still on the top bar in {text:?}"
    );
    assert!(
        !text[..bpm_at].contains("MIDI"),
        "the MIDI menu is still on the top bar in {text:?}"
    );
    assert!(
        !text.contains("No MIDI destinations found"),
        "the MIDI menu empty copy is still on the console in {text:?}"
    );
    assert!(
        !text.contains("Tempo"),
        "the Tempo menu is still on the top bar in {text:?}"
    );

    let output_at = clock_at
        + text[clock_at..]
            .find('O')
            .expect("the Panel is missing the O label in {text:?}");
    let none_at = text
        .find(crate::midi::OUTPUT_NONE)
        .unwrap_or_else(|| panic!("the Panel is missing Output None in {text:?}"));
    assert!(
        clock_at < output_at && output_at < none_at,
        "Readout order is not C then O then None in {text:?}"
    );
    assert!(
        !text.contains(super::panel::OUTPUT_SCAN),
        "Scan belongs in the Output menu, not on the closed Panel in {text:?}"
    );

    let panel = egui::containers::panel::PanelState::load(&ctx, egui::Id::new("bottom_panel"))
        .expect("the bottom Panel was not shown");
    assert!(
        (panel.outer_rect.bottom() - screen.bottom()).abs() < 0.5,
        "the Panel is not at the bottom: {:?}",
        panel.outer_rect
    );
    assert!(
        panel.outer_rect.height() > BOTTOM_PANEL_HEIGHT,
        "the default Panel has no room for Diagnostics: {:?}",
        panel.outer_rect
    );
}

///
/// A published Tick is what the Panel paints. Waiting a Tick period from
/// this Render Frame is a second clock, so `**` and T land late or twice.
///
#[tokio::test]
async fn a_playing_console_repaints_as_soon_as_the_published_tick_advances() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    let (tick, advanced, delay) =
        play_and_await_the_next_tick(&ctx, screen, &mut console, &mut host).await;
    assert_eq!(
        delay,
        std::time::Duration::ZERO,
        "the console waited {delay:?} after Tick {tick:?} became {advanced:?}"
    );
}

///
/// An Open installs a new Orcvs, and the Panel repaints the moment that
/// Orcvs's Playback publishes a Tick: the wake-up follows the Orcvs the
/// console runs, not the one it started with.
///
#[tokio::test]
async fn an_opened_console_repaints_as_soon_as_its_new_playback_publishes() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);

    console.load_function_reference();
    let (tick, advanced, delay) =
        play_and_await_the_next_tick(&ctx, screen, &mut console, &mut host).await;
    assert_eq!(
        delay,
        std::time::Duration::ZERO,
        "after an Open the console waited {delay:?} after Tick {tick:?} became {advanced:?}"
    );
}

///
/// The wake-up over an Orcvs an Open replaced ends once that Orcvs's
/// Playback closes its observation, so each Open leaves no task behind.
/// Awaiting the task is the observation: it ends, or the test fails.
///
#[tokio::test]
async fn the_wake_up_over_a_replaced_orcvs_ends_once_its_playback_is_gone() {
    let (ctx, mut console, _host) = fresh_console();
    let wake = tokio::spawn(super::repaint::panel_wake(
        ctx,
        console.orcvs.playback_observation_watch(),
    ));

    console.load_function_reference();

    tokio::time::timeout(std::time::Duration::from_secs(10), wake)
        .await
        .expect("the wake-up outlived the Orcvs an Open replaced")
        .expect("the wake-up ran to completion");
}

///
/// A quiet Render Frame must not start a Tick period from now. That is
/// the second clock. The next paint is the next publish, or the Cursor
/// Effect, whichever is sooner.
///
#[tokio::test]
async fn a_playing_console_does_not_schedule_a_tick_period_from_this_frame() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    let bpm = orcvs::opts::Bpm::new(200).expect("200 is in range");
    console.orcvs.set_bpm(bpm);
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Space, true)],
        &mut console,
        &mut host,
    );
    let mut playback = console.orcvs.playback_observation_watch();
    assert!(
        engine_reaches(&mut playback, |observation| {
            observation.state == orcvs::playback::PlaybackState::Playing
        })
        .await,
        "Playback never began playing"
    );

    let _ = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
    let delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
    let tick = bpm.tick_period();
    assert_ne!(
        delay, tick,
        "the console still scheduled a Tick period from this frame: {delay:?}"
    );
}

///
/// At 1 BPM a Tick lasts 15 seconds. Cursor Effect off, so its sub-second
/// wakes cannot hide a missing Run Clock remainder. A quiet Playing pass
/// must still request a delay of at most one second.
///
#[tokio::test]
async fn a_playing_console_with_cursor_effect_off_wakes_within_a_second() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    let bpm = orcvs::opts::Bpm::new(1).expect("1 is in range");
    console.orcvs.set_bpm(bpm);
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Space, true)],
        &mut console,
        &mut host,
    );
    let mut playback = console.orcvs.playback_observation_watch();
    assert!(
        engine_reaches(&mut playback, |observation| {
            observation.state == orcvs::playback::PlaybackState::Playing
        })
        .await,
        "Playback never began playing"
    );

    let _ = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
    let delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
    assert!(
        delay <= std::time::Duration::from_secs(1),
        "the console waited {delay:?} on a quiet Playing pass with Cursor Effect off"
    );
}

///
/// Amount zero leaves the Cursor Effect with no deadline of its own
/// (`repaint_after` returns `None`), which must not gate the Run Clock's
/// own term out of `until_next`: a Playing console still wakes within a
/// second on a quiet pass. A regression that gated the `until_next` call
/// on `cursor_delay` being `Some`, rather than always combining both
/// terms, would leave this console waiting past a second.
///
#[tokio::test]
async fn a_playing_console_still_repaints_when_the_cursor_effect_has_no_deadline() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    let bpm = orcvs::opts::Bpm::new(1).expect("1 is in range");
    console.orcvs.set_bpm(bpm);
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Space, true)],
        &mut console,
        &mut host,
    );
    let mut playback = console.orcvs.playback_observation_watch();
    assert!(
        engine_reaches(&mut playback, |observation| {
            observation.state == orcvs::playback::PlaybackState::Playing
        })
        .await,
        "Playback never began playing"
    );

    let _ = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
    let delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
    assert!(
        delay <= std::time::Duration::from_secs(1),
        "the console waited {delay:?} on a quiet Playing pass with Cursor Effect amount zero"
    );
}

///
/// Reduced motion's effective settings, not the stored ones, govern a
/// frame: `Console::cursor_effects` — the value a save persists —
/// survives running frames unchanged, and a quiet pass with Playback
/// stopped requests no repaint at all, though the stored settings animate
/// (`a_click_still_pans_to_follow_the_cursor_under_reduced_motion_with_playback_stopped`
/// relies on the same fact once a click has settled).
///
#[tokio::test]
async fn reduced_motion_changes_only_the_effective_settings_not_the_stored_ones() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    console.reduced_motion = true;
    *console.cursor_effects.amount_mut() = 42;
    *console.cursor_effects.frequency_mut() = 37;

    // A request made during one pass still repaints the next even with
    // nothing new to answer, "to give some things time to settle"
    // (a zero-delay `Context::request_repaint`); several quiet passes let
    // that one-time grace period from opening the console lapse before
    // the assertion below reads a steady state.
    for _ in 0..4 {
        let _ = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
    }
    let delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);

    assert_eq!(
        console.cursor_effects.amount(),
        42,
        "reduced motion overwrote the stored Glitch amount"
    );
    assert_eq!(
        console.cursor_effects.frequency(),
        37,
        "reduced motion overwrote the stored Glitch frequency"
    );
    assert_eq!(
        delay,
        std::time::Duration::MAX,
        "reduced motion's effective settings still scheduled a cursor-effect repaint: {delay:?}"
    );
}

///
/// egui subtracts `predicted_dt` from every timed request. With a
/// predicted frame longer than any Run Clock remainder, an uncompensated
/// remainder saturates to an immediate Render Frame on every pass — the
/// back-to-back redraw `until_next` adds `predicted_dt` to prevent. The
/// compensated request survives the subtraction as the remainder itself.
///
#[tokio::test]
async fn a_playing_console_compensates_the_run_clock_wake_for_predicted_frame_time() {
    use eframe::App as _;

    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    console
        .orcvs
        .set_bpm(orcvs::opts::Bpm::new(1).expect("1 is in range"));
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Space, true)],
        &mut console,
        &mut host,
    );
    let mut playback = console.orcvs.playback_observation_watch();
    assert!(
        engine_reaches(&mut playback, |observation| {
            observation.state == orcvs::playback::PlaybackState::Playing
        })
        .await,
        "Playback never began playing"
    );

    // A publish or focus wake can still ask for an immediate pass while
    // settling; a compensated Run Clock request is the only timed one.
    let mut delays = Vec::new();
    for _ in 0..8 {
        let input = egui::RawInput {
            screen_rect: Some(screen),
            predicted_dt: 2.0,
            ..Default::default()
        };
        let output = ctx.run_ui(input, |root| console.ui(root, &mut host));
        let delay = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|viewport| viewport.repaint_delay)
            .unwrap_or(std::time::Duration::MAX);
        output.drop_without_applying_deltas();
        delays.push(delay);
        if delay > std::time::Duration::ZERO {
            break;
        }
    }
    let settled = delays.last().copied().expect("at least one pass");
    assert!(
        settled > std::time::Duration::ZERO && settled <= std::time::Duration::from_secs(1),
        "a quiet Playing pass never requested the Run Clock remainder: {delays:?}"
    );
}

///
/// Rest must not grow a one-second wake, and with default settings the
/// Cursor Effect is off (ADR 0079), so a stopped console with the Cursor on
/// screen asks for no timed wake at all. The first passes request an
/// immediate Render Frame (focus, destination auto-select); settle before
/// asserting.
///
#[tokio::test]
async fn a_stopped_console_with_default_settings_requests_no_timed_wake() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    let mut delay = std::time::Duration::ZERO;
    for _ in 0..8 {
        delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
        if delay == std::time::Duration::MAX {
            break;
        }
    }
    assert_eq!(
        delay,
        std::time::Duration::MAX,
        "a Stopped console still requested a timed wake after settling: {delay:?}"
    );
}

fn origin_content(orcvs: &Orcvs) -> Option<char> {
    let frame = orcvs.render_frame();
    frame.at(frame.grid().origin()).content()
}

fn bpm_field_id(console: &Console) -> egui::Id {
    console.bpm_widget_id
}

fn focus_bpm_field(
    ctx: &egui::Context,
    screen: Rect,
    console: &mut Console,
    host: &mut eframe::Frame,
) {
    let target = ctx
        .read_response(bpm_field_id(console))
        .expect("the BPM field was not shown")
        .rect
        .center();
    app_pass(ctx, screen, click_at(target), console, host);
    app_pass(ctx, screen, release_at(target), console, host);
    assert!(
        ctx.memory(|memory| memory.has_focus(bpm_field_id(console))),
        "the BPM field did not take focus"
    );
}

fn bpm_selected_chars(ctx: &egui::Context, console: &Console) -> usize {
    egui::TextEdit::load_state(ctx, bpm_field_id(console))
        .and_then(|state| state.cursor.char_range())
        .map(|range| {
            let chars = range.as_sorted_char_range();
            chars.end.0.saturating_sub(chars.start.0)
        })
        .unwrap_or(0)
}

#[tokio::test]
async fn clicking_the_bpm_field_selects_its_text() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    let shown = console.orcvs.bpm().beats_per_minute().to_string();
    focus_bpm_field(&ctx, screen, &mut console, &mut host);
    assert_eq!(
        bpm_selected_chars(&ctx, &console),
        shown.chars().count(),
        "a click left the BPM text unselected"
    );

    app_pass(
        &ctx,
        screen,
        vec![Event::Text("7".to_owned())],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Enter, true)],
        &mut console,
        &mut host,
    );
    assert_eq!(
        console.orcvs.bpm().beats_per_minute(),
        7,
        "a click did not select the BPM text for replacement"
    );
}

///
/// With the BPM field focused, Tab is egui's own focus navigation and not
/// the Source's: `Console::ui` only cancels it while the Source holds the
/// keys, so a focused control keeps Tab exactly as it always has, and the
/// Cursor does not move.
///
#[tokio::test]
async fn tab_with_the_bpm_field_focused_leaves_the_cursor_and_moves_focus_on() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    focus_bpm_field(&ctx, screen, &mut console, &mut host);

    let cursor_before = console.orcvs.render_frame().cursor();
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Tab, true)],
        &mut console,
        &mut host,
    );

    assert_eq!(
        console.orcvs.render_frame().cursor(),
        cursor_before,
        "Tab moved the Cursor while the BPM field held focus"
    );
    assert!(
        !ctx.memory(|memory| memory.has_focus(bpm_field_id(&console))),
        "Tab left focus on the BPM field rather than moving egui's focus on from it"
    );
}

#[tokio::test]
async fn the_bpm_field_accepts_digits_only() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    focus_bpm_field(&ctx, screen, &mut console, &mut host);
    app_pass(
        &ctx,
        screen,
        vec![Event::Text("8".to_owned())],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![Event::Text("a".to_owned())],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![Event::Text("4".to_owned())],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![Event::Text("0".to_owned())],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Enter, true)],
        &mut console,
        &mut host,
    );
    assert_eq!(
        console.orcvs.bpm().beats_per_minute(),
        840,
        "the BPM field took a letter"
    );
}

#[tokio::test]
async fn escape_reverts_a_valid_uncommitted_bpm() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    let start = console.orcvs.bpm().beats_per_minute();
    let grid = console.orcvs.grid();
    console
        .orcvs
        .extend(grid.position(3, 2).expect("inside the Grid"));
    let region = console.orcvs.region();

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    focus_bpm_field(&ctx, screen, &mut console, &mut host);
    app_pass(
        &ctx,
        screen,
        vec![Event::Text("60".to_owned())],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Escape, true)],
        &mut console,
        &mut host,
    );
    assert_eq!(
        console.orcvs.bpm().beats_per_minute(),
        start,
        "Escape committed the typed BPM"
    );
    assert_eq!(
        console.orcvs.region(),
        region,
        "Escape in the BPM field collapsed the Region"
    );
}

///
/// A fill armed by command Enter waits for the next event, and typing into
/// the BPM field is that event even though the field, not the Source,
/// received it. A character typed back in the Source afterwards writes one
/// Cell rather than filling the Region the chord was pressed over.
///
#[tokio::test]
async fn keys_the_bpm_field_took_disarm_a_fill_armed_before_it() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    let grid = console.orcvs.grid();
    let at = |x, y| grid.position(x, y).expect("inside the Grid");
    console.orcvs.extend(at(2, 1));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    app_pass(
        &ctx,
        screen,
        vec![command_key_event(Key::Enter)],
        &mut console,
        &mut host,
    );
    focus_bpm_field(&ctx, screen, &mut console, &mut host);
    app_pass(
        &ctx,
        screen,
        vec![Event::Text("120".to_owned())],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Enter, true)],
        &mut console,
        &mut host,
    );
    assert!(
        !ctx.memory(|memory| memory.has_focus(bpm_field_id(&console))),
        "Enter left the BPM field focused"
    );

    app_pass(
        &ctx,
        screen,
        vec![Event::Text("x".to_owned())],
        &mut console,
        &mut host,
    );
    let frame = console.orcvs.render_frame();
    assert_eq!(
        frame.at(at(0, 0)).content(),
        None,
        "a fill armed before the BPM field took the keys filled the Region"
    );
}

///
/// A copy with the Source focused hands the Region's rows to the platform
/// clipboard, and a paste writes the text the platform delivered.
///
#[tokio::test]
async fn copy_reaches_the_platform_clipboard_and_paste_writes_the_source() {
    use eframe::App as _;

    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    let grid = console.orcvs.grid();
    let at = |x, y| grid.position(x, y).expect("inside the Grid");

    app_pass(
        &ctx,
        screen,
        vec![Event::Paste("ab\ncd".to_owned())],
        &mut console,
        &mut host,
    );
    assert_eq!(origin_content(&console.orcvs), Some('a'));
    let landed = console.orcvs.region();
    assert_eq!((landed.columns(), landed.rows()), (0..2, 0..2));

    console.orcvs.select(at(1, 0));
    console.orcvs.extend(at(1, 1));
    let input = egui::RawInput {
        screen_rect: Some(screen),
        events: vec![Event::Copy],
        ..Default::default()
    };
    let output = ctx.run_ui(input, |root| console.ui(root, &mut host));
    assert!(
        output
            .platform_output
            .commands
            .contains(&egui::OutputCommand::CopyText("b\nd".to_owned())),
        "the copy never reached the platform: {:?}",
        output.platform_output.commands
    );
}

#[tokio::test]
async fn dragging_the_bpm_field_does_not_change_the_tempo() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    let start = console.orcvs.bpm().beats_per_minute();
    let origin = ctx
        .read_response(bpm_field_id(&console))
        .expect("the BPM field was not shown")
        .rect
        .center();
    app_pass(&ctx, screen, click_at(origin), &mut console, &mut host);
    app_pass(
        &ctx,
        screen,
        vec![Event::PointerMoved(origin + Vec2::new(40.0, 0.0))],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        release_at(origin + Vec2::new(40.0, 0.0)),
        &mut console,
        &mut host,
    );
    assert_eq!(
        console.orcvs.bpm().beats_per_minute(),
        start,
        "dragging the BPM field changed the tempo"
    );
}
#[tokio::test]
async fn the_bpm_field_pads_three_digits() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    let field = ctx
        .read_response(bpm_field_id(&console))
        .expect("the BPM field was not shown")
        .rect;
    let font = ctx
        .style_of(egui::Theme::Dark)
        .text_styles
        .get(&egui::TextStyle::Monospace)
        .cloned()
        .unwrap_or_else(|| egui::FontId::monospace(12.0));
    let digits = ctx.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap("000".to_owned(), font, Color32::WHITE)
            .size()
    });
    assert!(
        field.width() + 0.5 >= digits.x + BPM_FIELD_MARGIN.sum().x,
        "the BPM field is too narrow for its padding: {field:?}"
    );
    assert!(
        field.height() + 0.5 >= digits.y + BPM_FIELD_MARGIN.sum().y,
        "the BPM field is too short for its padding: {field:?}"
    );
}

///
/// While the BPM field is focused, digits stay in the field and Space does
/// not toggle Playback. Escape (and a later Grid click) return keys to
/// the console.
///
#[tokio::test]
async fn a_focused_bpm_field_owns_digits_and_space_until_escape_or_a_grid_click() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    focus_bpm_field(&ctx, screen, &mut console, &mut host);

    app_pass(
        &ctx,
        screen,
        vec![Event::Text("5".to_owned())],
        &mut console,
        &mut host,
    );
    assert_eq!(
        origin_content(&console.orcvs),
        None,
        "a digit typed into the focused BPM field wrote the Source Cell"
    );

    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Space, true)],
        &mut console,
        &mut host,
    );
    tokio::task::yield_now().await;
    assert_eq!(
        console.orcvs.playback_observation().state,
        orcvs::playback::PlaybackState::Stopped,
        "Space toggled Playback while the BPM field had focus"
    );

    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Escape, true)],
        &mut console,
        &mut host,
    );
    assert!(
        !ctx.memory(|memory| memory.has_focus(bpm_field_id(&console))),
        "Escape left the BPM field focused"
    );

    app_pass(
        &ctx,
        screen,
        vec![Event::Text("x".to_owned())],
        &mut console,
        &mut host,
    );
    assert_eq!(
        origin_content(&console.orcvs),
        Some('x'),
        "a digit after Escape did not write the Source Cell"
    );

    focus_bpm_field(&ctx, screen, &mut console, &mut host);
    let viewport = console_viewport(&ctx, &console);
    let grid_cell = viewport.rect.min + Vec2::new(1.5, 0.5) * viewport.cell_size;
    app_pass(&ctx, screen, click_at(grid_cell), &mut console, &mut host);
    app_pass(&ctx, screen, release_at(grid_cell), &mut console, &mut host);
    assert!(
        !ctx.memory(|memory| memory.has_focus(bpm_field_id(&console))),
        "a click on the Source Grid left the BPM field focused"
    );

    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Space, true)],
        &mut console,
        &mut host,
    );
    tokio::task::yield_now().await;
    assert_eq!(
        console.orcvs.playback_observation().state,
        orcvs::playback::PlaybackState::Playing,
        "Space did not toggle Playback after a Grid click returned keys"
    );
}

///
/// Out-of-range typed BPM is rejected rather than clamped into range.
///
#[tokio::test]
async fn out_of_range_bpm_input_does_not_change_the_tempo() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    let start = console.orcvs.bpm().beats_per_minute();

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    focus_bpm_field(&ctx, screen, &mut console, &mut host);
    for text in ["0", "1500"] {
        app_pass(
            &ctx,
            screen,
            vec![Event::Text(text.to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Enter, true)],
            &mut console,
            &mut host,
        );
        assert_eq!(
            console.orcvs.bpm().beats_per_minute(),
            start,
            "committing {text} changed the tempo"
        );
        focus_bpm_field(&ctx, screen, &mut console, &mut host);
    }
}

///
/// Committing a new BPM while Playback is requested goes through
/// `Orcvs::set_bpm`. Retune-while-playing is already the engine's test;
/// this only asks that the Console's commit reaches it.
///
#[tokio::test]
async fn committing_bpm_while_playback_is_requested_sets_it_on_orcvs() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Space, true)],
        &mut console,
        &mut host,
    );
    tokio::task::yield_now().await;

    focus_bpm_field(&ctx, screen, &mut console, &mut host);
    app_pass(
        &ctx,
        screen,
        vec![Event::Key {
            key: Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        }],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![Event::Text("60".to_owned())],
        &mut console,
        &mut host,
    );
    app_pass(
        &ctx,
        screen,
        vec![key_event(Key::Enter, true)],
        &mut console,
        &mut host,
    );

    assert_eq!(console.orcvs.bpm().beats_per_minute(), 60);
}

#[tokio::test]
async fn a_source_smaller_than_the_console_sits_at_the_top_left_and_holds_no_cell_in_the_surplus() {
    for screen_size in [
        Vec2::new(400.0, 240.0),
        Vec2::new(200.0, 400.0),
        Vec2::new(300.0, 300.0),
    ] {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
        let mut orcvs = running_orcvs(8, 8);
        let mut view = SourceView::new(CELL_SIZE);

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let half_cell = viewport.cell_size / 2.0;

        assert_eq!(
            viewport.cell_size, CELL_SIZE,
            "a {screen_size:?} console opened at a Cell size other than the Source's own"
        );
        assert_eq!(
            viewport.rect.min,
            screen.min + MARGIN,
            "a {screen_size:?} console did not sit the Source one margin in from its top-left"
        );

        click(
            &ctx,
            screen,
            viewport.rect.max - half_cell,
            &mut orcvs,
            &mut view,
        );
        assert_eq!(
            selected_cell(&orcvs),
            (7, 7),
            "the last Cell of a {screen_size:?} console"
        );

        if let Some(past_the_grid) = surplus(screen, viewport.rect) {
            click(&ctx, screen, past_the_grid, &mut orcvs, &mut view);
            assert_eq!(
                selected_cell(&orcvs),
                (7, 7),
                "a click past the Grid of a {screen_size:?} console"
            );
        }

        click(
            &ctx,
            screen,
            viewport.rect.min + half_cell,
            &mut orcvs,
            &mut view,
        );
        assert_eq!(
            selected_cell(&orcvs),
            (0, 0),
            "the first Cell of a {screen_size:?} console"
        );
    }
}

///
/// The console opens on the one Grid at the Source's own Cell size, on its
/// top-left corner one margin in, with the Grid running past the far edges
/// so there is room to Pan, and no Glyph is resampled to be shown.
///
#[tokio::test]
async fn the_default_window_presents_the_grid_at_its_own_scale() {
    let ctx = egui::Context::default();
    let console = Vec2::new(
        DEFAULT_VIEW_SIZE[0],
        DEFAULT_VIEW_SIZE[1] - TOP_PANEL_HEIGHT - BOTTOM_PANEL_HEIGHT,
    );
    let screen = Rect::from_min_size(Pos2::ZERO, console);
    let mut orcvs = Orcvs::new().expect("the test runtime");
    let mut view = SourceView::new(CELL_SIZE);

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(
        viewport.cell_size, CELL_SIZE,
        "the default console opened at a scale other than one"
    );
    assert_eq!(
        viewport.rect.min,
        screen.min + MARGIN,
        "the default console did not rest the Grid one margin in"
    );
    assert!(
        viewport.rect.max.x > screen.max.x && viewport.rect.max.y > screen.max.y,
        "the Grid does not run past the default console, so there is nowhere to Pan"
    );
}

///
/// The default window is 1024 by 640 points of Source and its bars, at any
/// Cell.
///
#[test]
fn the_default_window_is_a_fixed_size_in_points() {
    assert_eq!(
        DEFAULT_VIEW_SIZE,
        [1024.0, 640.0 + TOP_PANEL_HEIGHT + BOTTOM_PANEL_HEIGHT]
    );
}

///
/// With Diagnostics hidden, the window holds back exactly the height the top
/// bar and the compact bottom Panel take, so the rest reaches the Source. A whole console
/// pass lays the panels out, so a menu, a control or a Readout that makes
/// either bar taller than its minimum, or a frame that eats into the
/// Source's area, fails here.
///
#[tokio::test]
async fn the_compact_bars_take_their_height_and_the_source_the_rest() {
    let (ctx, mut console, mut host) = fresh_console();
    console.diagnostics_open = false;
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);

    let panel = |id: &str| {
        egui::containers::panel::PanelState::load(&ctx, egui::Id::new(id))
            .unwrap_or_else(|| panic!("the console showed no {id}"))
            .outer_rect
    };
    let top = panel("top_panel");
    let bottom = panel("bottom_panel");
    assert_eq!(
        top,
        Rect::from_min_size(screen.min, Vec2::new(screen.width(), TOP_PANEL_HEIGHT)),
        "the top bar is not {TOP_PANEL_HEIGHT} tall across the top of the window"
    );
    assert_eq!(
        bottom,
        Rect::from_min_max(
            Pos2::new(screen.min.x, screen.max.y - BOTTOM_PANEL_HEIGHT),
            screen.max
        ),
        "the Panel is not {BOTTOM_PANEL_HEIGHT} tall across the bottom of the window"
    );

    // The Source's area is the one widget outside the bars that senses a
    // click and a drag without taking focus; it is allocated over all the
    // room the central panel has. Inside the bars a selectable label, such
    // as a MIDI status on a machine with no MIDI service, senses the same.
    let source_areas: Vec<Rect> = ctx.viewport(|viewport| {
        viewport
            .prev_pass
            .widgets
            .layers()
            .flat_map(|(_, widgets)| widgets)
            .filter(|widget| widget.sense == (egui::Sense::CLICK | egui::Sense::DRAG))
            .map(|widget| widget.rect)
            .filter(|rect| !top.contains_rect(*rect) && !bottom.contains_rect(*rect))
            .collect()
    });
    assert_eq!(
        source_areas,
        vec![Rect::from_min_max(
            Pos2::new(screen.min.x, top.max.y),
            Pos2::new(screen.max.x, bottom.min.y)
        )],
        "the Source is not given the whole window between the bars"
    );
}

///
/// Tick and Run Clock occupy fixed slots, so a second Tick digit does not
/// shove Run Clock.
///
#[test]
fn a_second_tick_digit_does_not_move_run_clock() {
    fn clock_left(tick: &str) -> i32 {
        let ctx = egui::Context::default();
        crate::style::install(&ctx, &okabe_ito(), &orcvs_light());
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let clock_x = std::cell::Cell::new(0.0);
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |root| {
                egui::Panel::bottom("bottom_panel")
                    .resizable(false)
                    .min_size(BOTTOM_PANEL_HEIGHT)
                    .show(root, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("T");
                            super::panel::reserved_monospace(
                                ui,
                                tick,
                                super::panel::monospace_width(ui, "00000"),
                            );
                            ui.label("C");
                            clock_x.set(
                                super::panel::reserved_monospace(
                                    ui,
                                    "00:00",
                                    super::panel::monospace_width(ui, "00:00"),
                                )
                                .rect
                                .left(),
                            );
                        });
                    });
            },
        );
        output.drop_without_applying_deltas();
        clock_x.get().round() as i32
    }

    assert_eq!(
        clock_left("00009"),
        clock_left("00010"),
        "a second Tick digit shoved Run Clock"
    );
}

#[test]
fn an_off_beat_does_not_move_tick() {
    fn tick_left(marker: &str) -> i32 {
        let ctx = egui::Context::default();
        crate::style::install(&ctx, &okabe_ito(), &orcvs_light());
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let tick_x = std::cell::Cell::new(0.0);
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |root| {
                egui::Panel::bottom("bottom_panel")
                    .resizable(false)
                    .min_size(BOTTOM_PANEL_HEIGHT)
                    .show(root, |ui| {
                        ui.horizontal(|ui| {
                            super::panel::reserved_monospace(
                                ui,
                                marker,
                                super::panel::monospace_width(ui, super::panel::BEAT_MARKER),
                            );
                            ui.label("T");
                            tick_x.set(
                                super::panel::reserved_monospace(
                                    ui,
                                    "00000",
                                    super::panel::monospace_width(ui, "00000"),
                                )
                                .rect
                                .left(),
                            );
                        });
                    });
            },
        );
        output.drop_without_applying_deltas();
        tick_x.get().round() as i32
    }

    assert_eq!(
        tick_left("**"),
        tick_left(""),
        "hiding the beat marker shoved Tick"
    );
    assert_eq!(
        tick_left("**"),
        tick_left("//"),
        "the rest marker shoved Tick"
    );
}

#[test]
fn panel_readouts_use_the_monospace_style_size_not_line_height() {
    let ctx = egui::Context::default();
    crate::style::install(&ctx, &okabe_ito(), &orcvs_light());
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    let sizes = std::cell::Cell::new((0.0, 0.0));
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |root| {
            egui::Panel::bottom("bottom_panel").show(root, |ui| {
                let style_size = ui
                    .style()
                    .text_styles
                    .get(&egui::TextStyle::Monospace)
                    .map(|font| font.size)
                    .unwrap_or(0.0);
                sizes.set((style_size, super::panel::panel_monospace_id(ui).size));
            });
        },
    );
    output.drop_without_applying_deltas();
    let (style_size, used) = sizes.get();
    assert_eq!(
        used, style_size,
        "Panel numbers were drawn at line height instead of the Monospace size"
    );
}

#[tokio::test]
async fn panel_label_gaps_match_and_entry_gaps_match() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    let painted = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = painted.clone();
    ctx.on_end_pass(
        "capture-panel-spans",
        std::sync::Arc::new(move |ctx| {
            let mut spans = Vec::new();
            let layers = [
                egui::LayerId::background(),
                egui::LayerId::new(egui::Order::Background, egui::Id::new("bottom_panel")),
                egui::LayerId::new(egui::Order::Background, egui::Id::new("top_panel")),
                egui::LayerId::new(egui::Order::Middle, egui::Id::new("bottom_panel")),
            ];
            ctx.graphics(|graphics| {
                for layer in layers {
                    if let Some(list) = graphics.get(layer) {
                        for clipped in list.all_entries() {
                            collect_shape_text_spans(&clipped.shape, &mut spans);
                        }
                    }
                }
            });
            *sink.lock().unwrap() = spans;
        }),
    );
    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);

    let mut spans = painted.lock().unwrap().clone();
    spans.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    fn span<'a>(spans: &'a [(String, f32, f32)], text: &str) -> &'a (String, f32, f32) {
        spans
            .iter()
            .find(|(shown, _, _)| shown == text)
            .unwrap_or_else(|| panic!("the Panel is missing {text:?} in {spans:?}"))
    }

    let b = span(&spans, "B");
    let rest = span(&spans, "//");
    let t = span(&spans, "T");
    let tick = span(&spans, "00000");
    let c = span(&spans, "C");
    let clock = span(&spans, "00:00");

    let t_to_v = tick.1 - t.2;
    let c_to_v = clock.1 - c.2;
    let beat_to_t = t.1 - rest.2;
    let tick_to_c = c.1 - tick.2;

    assert!(
        (t_to_v - c_to_v).abs() < 0.5,
        "readout pairs differ: T {t_to_v} C {c_to_v}"
    );
    assert!(
        (beat_to_t - tick_to_c).abs() < 0.5,
        "entry gaps differ: //-to-T {beat_to_t} Tick-to-C {tick_to_c}"
    );
    assert!(
        beat_to_t > t_to_v + 0.5,
        "groups are as tight as a label and its value: group {beat_to_t} pair {t_to_v}"
    );

    let expected_left = egui::Frame::side_top_panel(&crate::style::style(&okabe_ito()))
        .inner_margin
        .leftf()
        + f32::from(BOTTOM_PANEL_LEFT_PAD);
    assert!(
        (b.1 - expected_left).abs() < 1.0,
        "B starts at {} rather than {expected_left} in {spans:?}",
        b.1
    );
}

#[tokio::test]
async fn the_bottom_panel_separator_is_the_grid_line() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    let painted = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = painted.clone();
    ctx.on_end_pass(
        "capture-panel-separator",
        std::sync::Arc::new(move |ctx| {
            *sink.lock().unwrap() = painted_strokes(ctx);
        }),
    );
    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);

    let strokes = painted.lock().unwrap().clone();
    assert!(
        strokes.iter().any(|(color, rect)| {
            *color
                == crate::style::style(&okabe_ito())
                    .visuals
                    .window_stroke
                    .color
                && rect.height() <= 2.0
                && rect.width() > screen.width() * 0.5
        }),
        "the Panel separator was not the grid-line stroke in {strokes:?}"
    );
}

#[tokio::test]
async fn the_bpm_field_uses_the_selection_stroke_while_focused() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    let painted = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = painted.clone();
    ctx.on_end_pass(
        "capture-bpm-strokes",
        std::sync::Arc::new(move |ctx| {
            *sink.lock().unwrap() = painted_strokes(ctx);
        }),
    );

    app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
    let theme = okabe_ito();
    let field = ctx
        .read_response(bpm_field_id(&console))
        .expect("the BPM field was not shown")
        .rect;
    let rest = painted.lock().unwrap().clone();
    assert!(
        rest.iter().all(|(color, rect)| {
            !field.intersects(*rect)
                || (*color != theme.selection_border && *color != theme.selection_border_rest)
        }),
        "an unfocused BPM field still carried a selection border in {rest:?}"
    );

    focus_bpm_field(&ctx, screen, &mut console, &mut host);
    let focused_field = ctx
        .read_response(bpm_field_id(&console))
        .expect("the BPM field was not shown")
        .rect;
    let focused = painted.lock().unwrap().clone();
    assert!(
        focused.iter().any(|(color, rect)| {
            *color == theme.selection_border && focused_field.intersects(*rect)
        }),
        "a focused BPM field lost the selection stroke in {focused:?}"
    );
}

///
/// The viewport `show_source_scene` presents a Grid of this shape at, in a
/// console of this size, before any gesture has moved the view.
///
/// The same origin that function builds with no Pan, one margin in from the
/// console's top-left, so nothing about the geometry is restated here:
/// the presented Cell is `presented_grid`'s, asserted in
/// `grid_viewport.rs`.
///
fn presented(screen: Rect, columns: usize, rows: usize, pixels_per_point: f32) -> GridViewport {
    let cell = snapped_cell(CELL_SIZE, pixels_per_point);
    presented_grid(
        screen.min + SOURCE_MARGIN_CELLS * cell,
        Grid::with_shape(columns, rows),
        cell,
        pixels_per_point,
    )
}

///
/// A Paint of everything `viewport` shows of `frame` inside `clip`.
///
/// The range is `GridViewport::visible_positions`' own answer, which is
/// what `show_source` hands `Paint::derive`. A test that fits the whole
/// Grid on screen therefore gets a Paint of the whole Grid without having
/// to say so, and one that pans gets exactly what the console would draw.
///
fn painted(frame: &RenderFrame, viewport: GridViewport, clip: Rect) -> Paint {
    let grid = frame.grid();

    Paint::derive(FramePaint::new(
        frame,
        viewport.visible_positions(clip, grid),
    ))
}

/// [`painted`], but resolved against `theme` rather than the Okabe–Ito
/// built-in — for a test that asks about a Cell's own `border` or
/// `border_width`, both of which `Paint::derive_with_theme` bakes in
/// (`crate::style::ordinary_border`'s fact-priority pick), unlike
/// `SourceShapes::geometry`'s `sector_seam_width`, which stays a
/// frame-level Theme read a caller can vary independently of the Paint.
fn painted_themed(frame: &RenderFrame, viewport: GridViewport, clip: Rect, theme: &Theme) -> Paint {
    let grid = frame.grid();

    Paint::derive_with_theme(
        FramePaint::new(frame, viewport.visible_positions(clip, grid)),
        theme,
    )
}

///
/// The rectangles and strokes a Paint is drawn as at `viewport` — no
/// galleys, no font atlas, no `egui::Context`.
///
/// What is asserted through this is what the geometry step itself adds:
/// edges and background runs. Colour
/// *decisions* stay in `paint.rs`; Glyph placement needs
/// [`source_shapes`].
///
fn source_geometry(paint: &Paint, viewport: GridViewport, pixels_per_point: f32) -> SourceShapes {
    SourceShapes::geometry(paint, &viewport, pixels_per_point)
}

///
/// What a Paint is drawn as at `viewport`, including Glyphs, without a
/// console pass.
///
/// An `egui::Context` is built here for one reason: a galley needs a font
/// atlas, and a Glyph is a galley. Tests that assert nothing about a
/// galley use [`source_geometry`] instead.
///
fn source_shapes(paint: &Paint, viewport: GridViewport, pixels_per_point: f32) -> SourceShapes {
    let ctx = egui::Context::default();
    let mut shapes = None;
    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let table = GlyphTable::lay_out(
            ui.ctx(),
            egui::FontId::new(viewport.glyph_size(), egui::FontFamily::Monospace),
        );
        shapes = Some(SourceShapes::new(
            paint,
            &viewport,
            &table,
            pixels_per_point,
            crate::cursor_effects::CursorEffectShapes::default(),
        ));
    });
    output.drop_without_applying_deltas();

    shapes.expect("the pass drew the Source")
}

///
/// [`source_shapes`] with a Cursor Effect built by the caller, for a test
/// that asks what the effect's fragments add to the Paint's edges.
///
fn source_shapes_with_effect(
    paint: &Paint,
    viewport: GridViewport,
    pixels_per_point: f32,
    effect: crate::cursor_effects::CursorEffectShapes,
) -> SourceShapes {
    let ctx = egui::Context::default();
    let mut shapes = None;
    // `run_ui` takes an `FnMut`; the one pass it runs takes the effect.
    let mut effect = Some(effect);
    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let table = GlyphTable::lay_out(
            ui.ctx(),
            egui::FontId::new(viewport.glyph_size(), egui::FontFamily::Monospace),
        );
        shapes = Some(SourceShapes::new(
            paint,
            &viewport,
            &table,
            pixels_per_point,
            effect.take().expect("run_ui ran one pass"),
        ));
    });
    output.drop_without_applying_deltas();

    shapes.expect("the pass drew the Source")
}

/// The rectangle a `Shape::Rect` covers.
fn rect_of(shape: &Shape) -> Rect {
    match shape {
        Shape::Rect(rect) => rect.rect,
        other => panic!("{other:?} is not a rectangle"),
    }
}

fn close(left: Rect, right: Rect) -> bool {
    (left.min - right.min).length() < 1e-3 && (left.max - right.max).length() < 1e-3
}

///
/// The Cursor reaches what a Cell is painted *with* and never where it is
/// painted.
///
/// The property is structural — `GridViewport::cell_rect` takes a Position
/// and nothing else — so it is asserted here against the geometry that
/// actually reached the Shapes: the Grid lines lie on the edges of the
/// rectangles the Cells' Positions give them, the Cursor's included, and
/// every background lies on its Cell.
///
#[tokio::test]
async fn the_cursor_reaches_the_paint_of_a_cell_and_never_its_geometry() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let orcvs = running_orcvs(8, 8);
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 8, 8, 1.0);
    let paint = painted(&frame, viewport, screen);
    let shapes = source_geometry(&paint, viewport, 1.0);

    let (columns, rows) = boundaries(viewport, &paint, 1.0);
    assert_drawn_once_around(
        &line_quads(shapes.edges.as_slice(), 1.0),
        &columns,
        &rows,
        1,
        okabe_ito().grid_border.gamma_multiply(0.5),
        &ruled_apart(viewport, &frame, &paint, &okabe_ito(), 1.0),
    );
    for background in &shapes.backgrounds {
        assert!(
            (rect_of(background).height() - viewport.cell_size.y).abs() < 1e-3,
            "a background was {} tall against a Cell of {}",
            rect_of(background).height(),
            viewport.cell_size
        );
    }
}

///
/// While the Cursor Effect erodes the frame, its fragments are the outline:
/// the Cell edges under them show the Grid line rather than a solid outline,
/// so the erosion's gaps stay open. At amount zero there are no fragments and
/// the edges draw the still outline.
///
#[tokio::test]
async fn an_eroded_frame_leaves_the_grid_line_beneath_its_gaps() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(5, 5);
    orcvs.select(orcvs.grid().position(2, 2).expect("inside the grid"));
    let frame = orcvs.render_frame();
    let theme = okabe_ito();

    for pixels_per_point in [1.0, 1.5, 2.0] {
        let viewport = presented(screen, 5, 5, pixels_per_point);
        let paint = painted_themed(&frame, viewport, screen, &theme);
        let (columns, rows) = boundaries(viewport, &paint, pixels_per_point);
        let cursor = viewport.cell_rect(2, 2);
        let left_side = (columns[2], (rows[2] + rows[3]) / 2);

        for (amount, drawn) in [
            (0, theme.cursor_border),
            (64, expected_line(&theme, pixels_per_point).1),
        ] {
            let mut settings = crate::cursor_effects::CursorEffectSettings::default();
            *settings.amount_mut() = amount;
            let sample = crate::cursor_effects::CursorEffectAnimation::default()
                .advance(std::time::Duration::ZERO, settings);
            let effect = crate::cursor_effects::cursor_effect_shapes(
                cursor,
                cursor,
                screen,
                viewport.cell_size,
                crate::cursor_effects::CursorEffectMotion { sample, settings },
                theme.cursor_area,
                egui::Stroke::new(theme.cursor_border_width.points(), theme.cursor_border),
            );
            let fragments = !effect.fragments.is_empty();
            assert_eq!(
                fragments,
                amount > 0,
                "amount {amount} at {pixels_per_point}x"
            );
            let shapes = source_shapes_with_effect(&paint, viewport, pixels_per_point, effect);
            let pixels = edge_pixels(&line_quads(shapes.edges.as_slice(), pixels_per_point));

            assert_eq!(
                pixels.get(&left_side),
                Some(&drawn),
                "the Cursor's left side at amount {amount}, {pixels_per_point}x"
            );
        }
    }
}

///
/// The single-Cell Cursor's outline is one box, centred on its Cell's four
/// edges in `cursor.border` at `cursor.border.width`, the same width on
/// every side and at every corner, with no Grid line or neighbour's border
/// beside it — here over an Output Portal Cell, beside a Function, at 1×,
/// 1.5× and 2×.
///
#[tokio::test]
async fn the_cursor_outline_is_one_box_the_same_on_all_four_sides() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(5, 5);
    for (x, character) in ".+".chars().enumerate() {
        orcvs.select(orcvs.grid().position(x, 1).expect("inside the grid"));
        orcvs.write(&character.to_string());
    }
    // `.+`'s Output Portal Cell, below `+`.
    orcvs.select(orcvs.grid().position(1, 2).expect("inside the grid"));
    let frame = orcvs.render_frame();
    let theme = Theme {
        output_portal_border: Color32::from_rgb(200, 0, 0),
        ..okabe_ito()
    };

    for (pixels_per_point, width) in [(1.0, 1), (1.5, 2), (2.0, 2)] {
        let viewport = presented(screen, 5, 5, pixels_per_point);
        let paint = painted_themed(&frame, viewport, screen, &theme);
        let shapes = source_geometry(&paint, viewport, pixels_per_point);
        let pixels = edge_pixels(&line_quads(shapes.edges.as_slice(), pixels_per_point));
        let (columns, rows) = boundaries(viewport, &paint, pixels_per_point);
        let band = |boundary: i32| (boundary - width / 2)..(boundary - width / 2 + width);
        let middle = |from: i32, to: i32| (from + to) / 2;
        let (x, y) = (middle(columns[1], columns[2]), middle(rows[2], rows[3]));

        // Across each side at its middle: the outline's band, and nothing
        // either side of it.
        for side in [columns[1], columns[2]] {
            for at in band(side) {
                assert_eq!(
                    pixels.get(&(at, y)),
                    Some(&theme.cursor_border),
                    "{pixels_per_point}× column {at}"
                );
            }
            assert_eq!(
                pixels.get(&(band(side).start - 1, y)),
                None,
                "{pixels_per_point}× beside {side}"
            );
            assert_eq!(
                pixels.get(&(band(side).end, y)),
                None,
                "{pixels_per_point}× beside {side}"
            );
        }
        for side in [rows[2], rows[3]] {
            for at in band(side) {
                assert_eq!(
                    pixels.get(&(x, at)),
                    Some(&theme.cursor_border),
                    "{pixels_per_point}× row {at}"
                );
            }
            assert_eq!(
                pixels.get(&(x, band(side).start - 1)),
                None,
                "{pixels_per_point}× beside {side}"
            );
            assert_eq!(
                pixels.get(&(x, band(side).end)),
                None,
                "{pixels_per_point}× beside {side}"
            );
        }
        // Every corner is the outline's, whole.
        for corner_x in [columns[1], columns[2]] {
            for corner_y in [rows[2], rows[3]] {
                for at_x in band(corner_x) {
                    for at_y in band(corner_y) {
                        assert_eq!(
                            pixels.get(&(at_x, at_y)),
                            Some(&theme.cursor_border),
                            "{pixels_per_point}× corner {corner_x},{corner_y}"
                        );
                    }
                }
            }
        }
    }
}

///
/// An outline narrower than the Grid lines keeps its own width at its corners:
/// its sides meet in an L of their own bands, and the wider Grid lines fill
/// the rest of the junction outside the outlined Cell, at 1×, 1.5× and 2×.
///
#[tokio::test]
async fn an_outline_narrower_than_the_grid_lines_turns_its_corners_at_its_own_width() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(5, 5);
    orcvs.select(orcvs.grid().position(2, 2).expect("inside the grid"));
    let frame = orcvs.render_frame();
    let theme = Theme {
        grid_border_width: crate::theme::GridWidth::from_points(1.0).expect("1.0 is within 0..=1"),
        cursor_border_width: crate::theme::GridWidth::from_points(0.5)
            .expect("0.5 is within 0..=1"),
        cursor_border: Color32::from_rgb(200, 0, 0),
        ..okabe_ito()
    };

    for pixels_per_point in [1.0, 1.5, 2.0] {
        let viewport = presented(screen, 5, 5, pixels_per_point);
        let paint = painted_themed(&frame, viewport, screen, &theme);
        let shapes = source_geometry(&paint, viewport, pixels_per_point);
        let pixels = edge_pixels(&line_quads(shapes.edges.as_slice(), pixels_per_point));
        let (columns, rows) = boundaries(viewport, &paint, pixels_per_point);
        let line = |width: crate::theme::GridWidth, colour| {
            super::shapes::PixelLine::new(
                egui::Stroke::new(width.points(), colour),
                pixels_per_point,
            )
            .expect("a visible line")
        };
        let outline = line(theme.cursor_border_width, theme.cursor_border);
        let grid = line(theme.grid_border_width, theme.grid_border);
        let band =
            |boundary: i32, width: i32| (boundary - width / 2)..(boundary - width / 2 + width);

        for (corner_x, inward_x) in [(columns[2], 1), (columns[3], -1)] {
            for (corner_y, inward_y) in [(rows[2], 1), (rows[3], -1)] {
                for x in band(corner_x, grid.width) {
                    for y in band(corner_y, grid.width) {
                        // Each side runs into the corner from inside the
                        // outline, so its own pixels form an L.
                        let (own_x, own_y) =
                            (band(corner_x, outline.width), band(corner_y, outline.width));
                        let inside = |at: i32, own: &std::ops::Range<i32>, inward: i32| {
                            if inward > 0 {
                                at >= own.start
                            } else {
                                at < own.end
                            }
                        };
                        let own = (own_x.contains(&x) && inside(y, &own_y, inward_y))
                            || (own_y.contains(&y) && inside(x, &own_x, inward_x));
                        // A Grid line stops at the outline, so no Grid pixel
                        // lies inside the outlined Cell.
                        let within = |at: i32, corner: i32, inward: i32| {
                            if inward > 0 {
                                at >= corner
                            } else {
                                at < corner
                            }
                        };
                        let enclosed =
                            within(x, corner_x, inward_x) && within(y, corner_y, inward_y);
                        let expected = if own {
                            Some(&outline.colour)
                        } else if enclosed {
                            None
                        } else {
                            Some(&grid.colour)
                        };
                        assert_eq!(
                            pixels.get(&(x, y)),
                            expected,
                            "{pixels_per_point}× corner {corner_x},{corner_y} at {x},{y}"
                        );
                    }
                }
                // Just inside the corner, along each side, only the outline.
                let (x, y) = (
                    band(corner_x, outline.width).start,
                    band(corner_y, outline.width).start,
                );
                let (beside_x, beside_y) = (x + inward_x * grid.width, y + inward_y * grid.width);
                assert_eq!(pixels.get(&(beside_x, y)), Some(&outline.colour));
                assert_eq!(pixels.get(&(x, beside_y)), Some(&outline.colour));
            }
        }
    }
}

///
/// A Region larger than one Cell is one outline around its perimeter in
/// `region.border`, and the Cursor's own Cell inside it shows the Grid line
/// as every other unclaimed Cell does.
///
#[tokio::test]
async fn a_region_hides_the_cursors_cell_border() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(8, 8);
    let at = |x, y| orcvs.grid().position(x, y).expect("inside the grid");
    let (anchor, cursor) = (at(1, 1), at(4, 3));
    orcvs.select(anchor);
    orcvs.extend(cursor);
    let frame = orcvs.render_frame();
    let theme = Theme {
        region_border: Color32::from_rgb(200, 0, 0),
        ..okabe_ito()
    };
    let viewport = presented(screen, 8, 8, 1.0);
    let paint = painted_themed(&frame, viewport, screen, &theme);
    let shapes = source_geometry(&paint, viewport, 1.0);
    let pixels = edge_pixels(&line_quads(shapes.edges.as_slice(), 1.0));
    let (columns, rows) = boundaries(viewport, &paint, 1.0);
    let middle = |from: i32, to: i32| (from + to) / 2;
    let grid = theme.grid_border.gamma_multiply(0.5);

    // The perimeter, columns 1 to 4 and rows 1 to 3.
    let y = middle(rows[2], rows[3]);
    assert_eq!(pixels.get(&(columns[1], y)), Some(&theme.region_border));
    assert_eq!(pixels.get(&(columns[5], y)), Some(&theme.region_border));
    let x = middle(columns[2], columns[3]);
    assert_eq!(pixels.get(&(x, rows[1])), Some(&theme.region_border));
    assert_eq!(pixels.get(&(x, rows[4])), Some(&theme.region_border));
    // The Cursor's Cell at 4,3: its left and top edges lie inside the Region.
    let (x, y) = (middle(columns[4], columns[5]), middle(rows[3], rows[4]));
    assert_eq!(pixels.get(&(columns[4], y)), Some(&grid));
    assert_eq!(pixels.get(&(x, rows[3])), Some(&grid));
    assert!(shapes.cursor.is_empty(), "a Cell stroked the Cursor");
    assert!(!shapes.backgrounds.is_empty(), "the Region was not filled");
}

///
/// The lasso outlines the whole Region rather than the Cursor's Cell.
///
#[tokio::test]
async fn the_effect_outline_is_the_region_when_it_spans_and_the_cursor_otherwise() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(8, 8);
    let grid = orcvs.grid();
    let at = |x, y| grid.position(x, y).expect("inside the grid");
    let viewport = presented(screen, 8, 8, 1.0);

    orcvs.select(at(4, 3));
    assert!(close(
        super::shapes::effect_outline(&orcvs.render_frame(), &viewport),
        viewport.cell_rect(4, 3)
    ));

    orcvs.select(at(4, 3));
    orcvs.extend(at(1, 1));
    assert!(close(
        super::shapes::effect_outline(&orcvs.render_frame(), &viewport),
        Rect::from_min_max(viewport.cell_rect(1, 1).min, viewport.cell_rect(4, 3).max)
    ));
}

///
/// A Cell's background never paints over a Glyph, whichever Cell that Glyph
/// belongs to, and the Cursor's outline is an edge, between the two.
///
/// The groups are what makes that expressible. Every fill is in
/// `backgrounds`, every Glyph in `glyphs` and every edge in `edges`, so
/// chaining the groups orders the *kinds* however the Cells interleave: a
/// later Cell in the row order cannot erase an earlier Cell's Glyph, and no
/// neighbour's fill can reach an edge. That the shape groups
/// then arrive at the painter in that order is asserted by
/// `the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them`.
///
#[tokio::test]
async fn every_background_is_painted_before_every_glyph_and_no_cell_strokes_the_cursor() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(8, 8);
    // A Glyph in the first Cell of the Grid, so every other Cell's
    // background is built after it and would paint over it if the Shapes
    // were emitted Cell by Cell.
    orcvs.write("1");
    orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 8, 8, 1.0);
    let paint = painted(&frame, viewport, screen);
    let shapes = source_shapes(&paint, viewport, 1.0);

    assert!(
        !shapes.backgrounds.is_empty(),
        "the Grid painted no Cell background"
    );
    for background in &shapes.backgrounds {
        let Shape::Rect(rect) = background else {
            panic!("a background was {background:?}")
        };
        assert!(
            rect.fill.a() > 0 && rect.stroke.width == 0.0,
            "a background carried a stroke: {rect:?}"
        );
    }
    assert!(!shapes.glyphs.is_empty(), "the Grid painted no Glyph");
    for glyph in &shapes.glyphs {
        assert!(
            matches!(glyph, Shape::Text(_)),
            "a Glyph was {glyph:?} rather than text"
        );
    }
    assert!(
        shapes.cursor.is_empty(),
        "a Cell stroked the Cursor rather than its edges"
    );
    assert_eq!(
        shapes.backgrounds.len(),
        1,
        "only the hidden-caret selection should fill a Cell"
    );
}

///
/// The shape groups reach the painter end to end, in the order
/// `SourceShapes::into_shapes` chains them: backgrounds, the edges, Glyphs,
/// the Cursor Effect's fragments.
///
/// Read off the flat shape list of a whole console pass, which is the one
/// place paint order is observable and which also covers the three calls
/// inside `show_source` that nothing else exercises — derive a Paint, draw
/// it, extend the painter once. This is not a concession: `egui_kittest`'s
/// own regression tests assert against `output().shapes` the same way.
///
/// The panel's own fill is in that list too — `source_panel_frame` paints
/// it before the Grid's first Shape — and it is a plain fill just as a
/// background run is. It is not subtracted, because every claim below is
/// that the *last* member of one group precedes the *first* member of the
/// next, and a fill preceding every Grid Shape moves neither bound.
///
/// The edges are the one Mesh; what this asserts is that they precede
/// every Glyph and that the Cursor Effect's fragments are painted after
/// everything else. That the edges follow the backgrounds is
/// `the_grid_lines_lie_over_the_backgrounds`'.
///
#[tokio::test]
async fn the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(16, 16);
    let mut view = SourceView::new(CELL_SIZE);
    // A written Cell, so a Glyph is painted at all.
    orcvs.write("1");
    orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));
    // A settings file can still turn the Cursor Effect on.
    let mut settings = crate::cursor_effects::CursorEffectSettings::default();
    *settings.amount_mut() = 60;
    let motion = crate::cursor_effects::CursorEffectMotion {
        sample: crate::cursor_effects::CursorEffectAnimation::default()
            .advance(std::time::Duration::ZERO, settings),
        settings,
    };

    let (_viewport, shapes) =
        console_pass_moving(&ctx, screen, Vec::new(), &mut orcvs, &mut view, 1.0, motion);

    let at = |kind: fn(&Shape) -> bool| -> Vec<usize> {
        shapes
            .iter()
            .enumerate()
            .filter_map(|(index, shape)| kind(shape).then_some(index))
            .collect()
    };
    fn is_fill(shape: &Shape) -> bool {
        matches!(shape, Shape::Rect(rect) if rect.fill.a() > 0 && rect.stroke.width == 0.0)
    }
    let fills = at(is_fill);
    let lines = at(|shape| matches!(shape, Shape::Mesh(_)));
    let glyphs = at(|shape| matches!(shape, Shape::Text(_)));
    let fragments = at(|shape| matches!(shape, Shape::LineSegment { .. }));

    assert!(!fills.is_empty(), "the pass painted no fill");
    assert_eq!(
        lines.len(),
        1,
        "the pass painted {} line Meshes",
        lines.len()
    );
    assert!(!glyphs.is_empty(), "the pass painted no Glyph");
    assert!(!fragments.is_empty(), "the pass painted no Cursor fragment");

    assert!(
        lines[0] < glyphs[0],
        "the Grid lines at {} painted over the Glyph at {}",
        lines[0],
        glyphs[0]
    );
    assert!(
        *glyphs.last().expect("a Glyph") < *fragments.last().expect("a fragment"),
        "the fragmented Cursor frame was not painted last"
    );
}

///
/// The Source's Cells hold printable ASCII, so that is what the table
/// covers. The space is the one printable character left out, because a
/// Cell showing one paints no Glyph.
///
#[test]
fn the_glyph_table_covers_the_printable_ascii_a_cell_can_hold() {
    let ctx = egui::Context::default();
    let mut table = None;
    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
        table = Some(GlyphTable::lay_out(ui.ctx(), egui::FontId::monospace(13.5)));
    });
    output.drop_without_applying_deltas();
    let table = table.expect("the pass laid the alphabet out");

    for byte in ALPHABET_FIRST..=ALPHABET_LAST {
        assert!(
            table.galley(char::from(byte)).is_some(),
            "the table does not cover {:?}",
            char::from(byte)
        );
    }
    // Outside the alphabet on both sides, and beyond ASCII altogether.
    // Every one of these falls back to a single uncached layout for that
    // Cell alone.
    for character in [' ', '\n', '\u{7f}', 'é', '✦'] {
        assert!(
            table.galley(character).is_none(),
            "the table claims to cover {character:?}"
        );
    }
}

///
/// A character the table does not cover is still laid out, and is laid out
/// as itself.
///
/// `GlyphTable::glyph` answers a galley for every character, because the
/// step that draws a Cell has to put *something* in the ordered sequence
/// of Shapes rather than skip the Cell and paint it out of turn. The two
/// ways it answers are asserted against each other here: inside the
/// alphabet it hands back the table's own galley — the same `Arc`, not an
/// equal one, which is the whole point of laying the alphabet out once —
/// and outside it lays that one character out fresh.
///
/// A Source Cell holds printable ASCII by construction, so the fallback is
/// for a Source that found a way to hold something else. That is exactly
/// why it is worth pinning: nothing else reaches it, so a fallback that
/// silently answered the wrong Glyph would paint the wrong character with
/// no other test noticing.
///
#[test]
fn a_character_outside_the_alphabet_is_laid_out_as_itself() {
    let ctx = egui::Context::default();
    let mut table = None;
    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
        table = Some(GlyphTable::lay_out(ui.ctx(), egui::FontId::monospace(13.5)));
    });
    output.drop_without_applying_deltas();
    let table = table.expect("the pass laid the alphabet out");

    for character in [' ', '\n', '\u{7f}', 'é', '✦'] {
        let galley = table.glyph(character);

        assert!(
            table.galley(character).is_none(),
            "{character:?} is in the alphabet, so this asserts nothing about the fallback"
        );
        assert_eq!(
            galley.text(),
            character.to_string(),
            "the fallback laid out something other than {character:?}"
        );
    }

    for byte in [ALPHABET_FIRST, b'A', ALPHABET_LAST] {
        let character = char::from(byte);
        let cached = table.galley(character).expect("inside the alphabet");

        assert!(
            std::sync::Arc::ptr_eq(&table.glyph(character), cached),
            "{character:?} was laid out again instead of read from the table"
        );
    }
}

///
/// The quads a Grid lines Mesh is built from, each as the physical pixels it
/// covers and its colour, after checking that it lies on whole pixels.
///
/// `Mesh::add_colored_rect` adds four vertices per quad, its minimum corner
/// first and its maximum corner last.
///
fn line_quads(lines: &[Shape], pixels_per_point: f32) -> Vec<([i32; 4], Color32)> {
    let mut quads = Vec::new();
    for shape in lines {
        let Shape::Mesh(mesh) = shape else {
            panic!("{shape:?} is not a Mesh");
        };
        for corners in mesh.vertices.as_chunks::<4>().0 {
            let pixel = |point: f32| {
                let physical = point * pixels_per_point;
                assert!(
                    (physical - physical.round()).abs() < 1e-3,
                    "a Grid line piece edge sits at {physical} physical pixels"
                );
                physical.round() as i32
            };
            let (min, max) = (corners[0].pos, corners[3].pos);
            assert!(
                corners
                    .iter()
                    .all(|corner| corner.color == corners[0].color),
                "a Grid line piece is not one colour"
            );
            quads.push((
                [pixel(min.x), pixel(min.y), pixel(max.x), pixel(max.y)],
                corners[0].color,
            ));
        }
    }
    quads
}

///
/// Asserts that `quads` draw each line of a lattice once: every boundary in
/// `columns` and every boundary in `rows` covered across the lattice's whole
/// extent by `width` pixels, no pixel covered twice, and one piece per
/// crossing and one per edge between crossings.
///
fn assert_drawn_once(
    quads: &[([i32; 4], Color32)],
    columns: &[i32],
    rows: &[i32],
    width: i32,
    colour: Color32,
) {
    assert_drawn_once_around(quads, columns, rows, width, colour, &[]);
}

///
/// [`assert_drawn_once`] for every pixel outside the boxes in `apart`, which
/// lines other than the Grid's cover — the selection outline, which
/// `the_cursor_outline_is_one_box_the_same_on_all_four_sides` asserts, and
/// the Sector Seams, which `a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for`
/// does. Inside them, no pixel is covered twice either.
///
fn assert_drawn_once_around(
    quads: &[([i32; 4], Color32)],
    columns: &[i32],
    rows: &[i32],
    width: i32,
    colour: Color32,
    apart: &[[i32; 4]],
) {
    let outlined = |x: i32, y: i32| {
        apart.iter().any(|[left, top, right, bottom]| {
            (*left..*right).contains(&x) && (*top..*bottom).contains(&y)
        })
    };
    let band = |boundary: i32| (boundary - width / 2)..(boundary - width / 2 + width);
    let across = band(columns[0]).start..band(*columns.last().expect("a boundary")).end;
    let down = band(rows[0]).start..band(*rows.last().expect("a boundary")).end;

    let mut covered = std::collections::HashMap::<(i32, i32), u32>::new();
    for &([left, top, right, bottom], quad_colour) in quads {
        assert!(
            quad_colour == colour || outlined(left, top),
            "a piece at {left},{top} took another colour"
        );
        for x in left..right {
            for y in top..bottom {
                *covered.entry((x, y)).or_default() += 1;
            }
        }
    }
    for y in down.clone() {
        for x in across.clone() {
            let on_line = columns.iter().any(|&column| band(column).contains(&x))
                || rows.iter().any(|&row| band(row).contains(&y));
            let times = covered.remove(&(x, y)).unwrap_or(0);
            if outlined(x, y) {
                assert!(times <= 1, "pixel {x},{y} was covered {times} times");
                continue;
            }
            assert_eq!(
                times,
                u32::from(on_line),
                "pixel {x},{y} was covered {times} times"
            );
        }
    }
    // The outline may reach past the lattice's outermost Grid lines.
    covered.retain(|&(x, y), times| {
        assert!(*times <= 1, "pixel {x},{y} was covered {times} times");
        !outlined(x, y)
    });
    assert!(
        covered.is_empty(),
        "pieces covered pixels outside the lattice: {:?}",
        covered.keys().take(4).collect::<Vec<_>>()
    );

    let (crossings, edges) = (
        columns.len() * rows.len(),
        (columns.len() - 1) * rows.len() + columns.len() * (rows.len() - 1),
    );
    assert_eq!(
        quads.len(),
        crossings + edges,
        "{} pieces for {crossings} crossings and {edges} edges",
        quads.len()
    );
}

///
/// The boundaries of the Cells `paint` covers in physical pixels, the far
/// edges of its last column and row included: the Grid's snapped corner, then
/// one whole-pixel Cell width or height per Cell. Counted from the Grid's corner rather
/// than read from `GridViewport::cell_rect`, so a lattice that rounds a
/// boundary differently, or misses the far one, disagrees with it.
///
fn boundaries(
    viewport: GridViewport,
    paint: &Paint,
    pixels_per_point: f32,
) -> (Vec<i32>, Vec<i32>) {
    let drawn = paint.drawn();
    let cell = viewport.cell_size * pixels_per_point;
    assert_eq!(cell, cell.round(), "a presented Cell is whole pixels");
    let corner = viewport.rect.min * pixels_per_point;
    let counted = |corner: f32, side: f32, cells: std::ops::Range<usize>| {
        (cells.start..cells.end + 1)
            .map(|cell| corner.round() as i32 + cell as i32 * side as i32)
            .collect()
    };
    (
        counted(corner.x, cell.x, drawn.columns.clone()),
        counted(corner.y, cell.y, drawn.rows.clone()),
    )
}

///
/// The pixels the single-Cell Cursor's outline at `cursor` covers, with its
/// junctions, at `pixels_per_point`: its four lines' bands at the wider of
/// `cursor.border.width` and `grid.border.width`.
///
fn outline_box(
    viewport: GridViewport,
    cursor: Position,
    theme: &Theme,
    pixels_per_point: f32,
) -> [i32; 4] {
    let pixels = |points: f32| {
        let physical = points * pixels_per_point;
        if physical < 1.0 {
            1
        } else {
            physical.round() as i32
        }
    };
    let width =
        pixels(theme.cursor_border_width.points()).max(pixels(theme.grid_border_width.points()));
    let pixel = |point: f32| (point * pixels_per_point).round() as i32;
    let cell = viewport.cell_rect(cursor.x(), cursor.y());
    let [left, top, right, bottom] = [cell.min.x, cell.min.y, cell.max.x, cell.max.y].map(pixel);
    [
        left - width / 2,
        top - width / 2,
        right - width / 2 + width,
        bottom - width / 2 + width,
    ]
}

///
/// The boxes [`assert_drawn_once_around`] leaves to other tests in a lattice
/// of `paint`'s Cells: the Cursor's outline, and every column line and row
/// line a Sector Seam can lie on, across the whole lattice.
///
fn ruled_apart(
    viewport: GridViewport,
    frame: &RenderFrame,
    paint: &Paint,
    theme: &Theme,
    pixels_per_point: f32,
) -> Vec<[i32; 4]> {
    let (columns, rows) = boundaries(viewport, paint, pixels_per_point);
    let (width, _) = expected_seam(theme, theme.sector_seam, pixels_per_point);
    let band = |boundary: i32| (boundary - width / 2)..(boundary - width / 2 + width);
    let (left, right) = (columns[0] - width, columns[columns.len() - 1] + width);
    let (top, bottom) = (rows[0] - width, rows[rows.len() - 1] + width);
    let spacing = frame.sector_seam_spacing().cells();
    let drawn = paint.drawn();
    let seam_line = |line: usize| line > 0 && line.is_multiple_of(spacing);

    let mut apart = vec![outline_box(
        viewport,
        frame.cursor(),
        theme,
        pixels_per_point,
    )];
    for (column, &boundary) in drawn
        .columns
        .clone()
        .chain([drawn.columns.end])
        .zip(&columns)
    {
        if seam_line(column) {
            apart.push([band(boundary).start, top, band(boundary).end, bottom]);
        }
    }
    for (row, &boundary) in drawn.rows.clone().chain([drawn.rows.end]).zip(&rows) {
        if seam_line(row) {
            apart.push([left, band(boundary).start, right, band(boundary).end]);
        }
    }
    apart
}

///
/// The pixels across and the colour a line `width` points wide in `colour`
/// covers at `pixels_per_point`: whole pixels from one up, and one pixel
/// faded by its width below that.
///
fn pixel_line(width: f32, colour: Color32, pixels_per_point: f32) -> (i32, Color32) {
    let physical = width * pixels_per_point;
    if physical < 1.0 {
        (1, colour.gamma_multiply(physical))
    } else {
        (physical.round() as i32, colour)
    }
}

///
/// What a `grid.border.width` Grid line covers at `pixels_per_point`.
///
fn expected_line(theme: &Theme, pixels_per_point: f32) -> (i32, Color32) {
    pixel_line(
        theme.grid_border_width.points(),
        theme.grid_border,
        pixels_per_point,
    )
}

///
/// What a Sector Seam in `seam` covers where it lies over the Grid line:
/// the wider of the two widths, in the seam composited over the line.
///
fn expected_seam(theme: &Theme, seam: Color32, pixels_per_point: f32) -> (i32, Color32) {
    pixel_line(
        theme
            .grid_border_width
            .points()
            .max(theme.sector_seam_width.points()),
        theme.grid_border.blend(seam),
        pixels_per_point,
    )
}

///
/// An empty Grid draws every Grid line once, on whole physical pixels, with
/// one junction where each pair of lines crosses, at 1×, 2× and a fractional
/// scale, and at a width that rounds to one pixel and one that rounds to two.
///
/// Every Cell is unclaimed, so none carries a border of its own: the lines
/// are the Grid's, but for the Cursor's outline.
///
#[tokio::test]
async fn an_empty_grid_draws_each_grid_line_once() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let orcvs = running_orcvs(5, 3);
    let frame = orcvs.render_frame();

    // The 12 by 18 point Cell's width and height in pixels at each scale.
    for (width_points, pixels_per_point, width_pixels, (width, height)) in [
        (1.0, 1.0, 1, (12, 18)),
        (0.5, 2.0, 1, (24, 36)),
        (1.0, 2.0, 2, (24, 36)),
        (1.0, 1.5, 2, (18, 27)),
        (0.75, 2.0, 2, (24, 36)),
    ] {
        let theme = Theme {
            grid_border_width: crate::theme::GridWidth::from_points(width_points)
                .expect("within 0..=1"),
            ..okabe_ito()
        };
        let viewport = presented(screen, 5, 3, pixels_per_point);
        let paint = painted_themed(&frame, viewport, screen, &theme);
        let shapes = source_geometry(&paint, viewport, pixels_per_point);
        let (columns, rows) = boundaries(viewport, &paint, pixels_per_point);
        assert_eq!((columns.len(), rows.len()), (6, 4));
        for (lines, side) in [(&columns, width), (&rows, height)] {
            for spacing in lines.windows(2) {
                assert_eq!(spacing[1] - spacing[0], side, "at {pixels_per_point}x");
            }
        }
        // The lines lie on the boundaries the background runs are snapped
        // to: every run's edges are lattice boundaries.
        for run in &shapes.backgrounds {
            let Shape::Rect(run) = run else {
                panic!("a background run is a rectangle, not {run:?}")
            };
            let edge = |point: f32| (point * pixels_per_point).round() as i32;
            assert!(
                columns.contains(&edge(run.rect.min.x)) && columns.contains(&edge(run.rect.max.x)),
                "a run's sides {:?} lie off the column lines {columns:?} at {pixels_per_point}x",
                run.rect
            );
            assert!(
                rows.contains(&edge(run.rect.min.y)) && rows.contains(&edge(run.rect.max.y)),
                "a run's top and bottom {:?} lie off the row lines {rows:?} at {pixels_per_point}x",
                run.rect
            );
        }

        assert_drawn_once_around(
            &line_quads(shapes.edges.as_slice(), pixels_per_point),
            &columns,
            &rows,
            width_pixels,
            theme.grid_border,
            &ruled_apart(viewport, &frame, &paint, &theme, pixels_per_point),
        );
    }
}

///
/// Cells whose width and height differ are spaced by each axis's own side,
/// and their lines are still drawn once, junctions included.
///
#[test]
fn unequal_cell_sides_draw_each_grid_line_once() {
    let line = EdgeLine {
        rank: EdgeRank::GridLine,
        stroke: egui::Stroke::new(1.0, Color32::from_rgb(10, 20, 30)),
        seamed: false,
    };
    let drawn =
        crate::grid_viewport::VisiblePositions::for_grid(Grid::with_shape(3, 2), 0..3, 0..2);
    let edges = Edges {
        rows: (0..=2)
            .map(|row| EdgeRun {
                line: row,
                span: 0..3,
                edge: line,
            })
            .collect(),
        columns: (0..=3)
            .map(|column| EdgeRun {
                line: column,
                span: 0..2,
                edge: line,
            })
            .collect(),
    };
    let columns = [10, 22, 34, 46];
    let rows = [5, 23, 41];

    let quads = line_quads(
        &[Shape::mesh(super::shapes::edge_mesh(
            &edges, &drawn, &columns, &rows, 2.0,
        ))],
        2.0,
    );

    assert_drawn_once(&quads, &columns, &rows, 2, line.stroke.color);
}

///
/// A seam runs on through every crossing with a plain line of its own rank,
/// even where both are opaque and so equally strong.
///
#[test]
fn a_seam_runs_through_an_opaque_grid_line_it_crosses() {
    let plain = EdgeLine {
        rank: EdgeRank::GridLine,
        stroke: egui::Stroke::new(1.0, Color32::from_rgb(10, 20, 30)),
        seamed: false,
    };
    let seam = EdgeLine {
        stroke: egui::Stroke::new(1.0, Color32::from_rgb(200, 100, 50)),
        seamed: true,
        ..plain
    };
    let drawn =
        crate::grid_viewport::VisiblePositions::for_grid(Grid::with_shape(2, 2), 0..2, 0..2);
    let edges = Edges {
        rows: (0..=2)
            .map(|row| EdgeRun {
                line: row,
                span: 0..2,
                edge: plain,
            })
            .collect(),
        columns: (0..=2)
            .map(|column| EdgeRun {
                line: column,
                span: 0..2,
                edge: if column == 1 { seam } else { plain },
            })
            .collect(),
    };
    let columns = [0, 10, 20];
    let rows = [0, 10, 20];

    let pixels = edge_pixels(&line_quads(
        &[Shape::mesh(super::shapes::edge_mesh(
            &edges, &drawn, &columns, &rows, 1.0,
        ))],
        1.0,
    ));

    for y in rows[0]..=rows[2] {
        assert_eq!(
            pixels.get(&(columns[1], y)),
            Some(&seam.stroke.color),
            "the seam at {y}"
        );
    }
}

///
/// Where a narrower line outranks a wider one at a crossing, it keeps its own
/// width there and the wider line fills the rest of the junction outside the
/// bordered Cell: a 1 pixel border round a Cell turns its corners at 1 pixel
/// through 2 pixel Grid lines, and no Grid pixel lands inside it.
///
#[test]
fn a_narrow_border_keeps_its_width_through_a_wider_grid_lines_junction() {
    let grid = EdgeLine {
        rank: EdgeRank::GridLine,
        stroke: egui::Stroke::new(1.0, Color32::from_rgb(10, 20, 30)),
        seamed: false,
    };
    let border = EdgeLine {
        rank: EdgeRank::Diagnostic,
        stroke: egui::Stroke::new(0.5, Color32::from_rgb(200, 100, 50)),
        seamed: false,
    };
    // Cell (1, 1) of three by three carries the border on every side.
    let drawn =
        crate::grid_viewport::VisiblePositions::for_grid(Grid::with_shape(3, 3), 0..3, 0..3);
    let line = |boundary: usize, cell: usize| {
        if (boundary == 1 || boundary == 2) && cell == 1 {
            border
        } else {
            grid
        }
    };
    let runs = |boundary: usize| {
        (0..3)
            .map(|cell| EdgeRun {
                line: boundary,
                span: cell..cell + 1,
                edge: line(boundary, cell),
            })
            .collect::<Vec<_>>()
    };
    let edges = Edges {
        rows: (0..=3).flat_map(runs).collect(),
        columns: (0..=3).flat_map(runs).collect(),
    };
    let lines = [10, 30, 50, 70];

    let pixels = edge_pixels(&line_quads(
        &[Shape::mesh(super::shapes::edge_mesh(
            &edges, &drawn, &lines, &lines, 2.0,
        ))],
        2.0,
    ));

    // The border's 1 pixel bands, and the 2 pixel Grid line bands beside
    // them, which stop where the border's sides run and at the bordered Cell.
    let (near, far) = (lines[1], lines[2]);
    let grid_band = [near - 1, near, far - 1, far];
    let along_side = |pixel: i32| (near + 1..far - 1).contains(&pixel);
    for x in near - 1..=far {
        for y in near - 1..=far {
            let on_border = (near..=far).contains(&x)
                && (near..=far).contains(&y)
                && (x == near || x == far || y == near || y == far);
            let enclosed = (near..far).contains(&x) && (near..far).contains(&y);
            let on_grid = ((grid_band.contains(&x) && !along_side(y))
                || (grid_band.contains(&y) && !along_side(x)))
                && !enclosed;
            let expected = if on_border {
                Some(border.stroke.color)
            } else if on_grid {
                Some(grid.stroke.color)
            } else {
                None
            };
            assert_eq!(pixels.get(&(x, y)).copied(), expected, "pixel {x},{y}");
        }
    }
}

///
/// A line under one physical pixel covers one pixel, faded by its width, as
/// epaint's own thin strokes fade rather than thin. At 1× Okabe–Ito's
/// half-point Grid line is one pixel at half its colour.
///
#[tokio::test]
async fn a_grid_line_under_one_pixel_covers_one_pixel_faded_by_its_width() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let orcvs = running_orcvs(4, 4);
    let frame = orcvs.render_frame();
    let theme = okabe_ito();
    assert_eq!(theme.grid_border_width.points(), 0.5);

    for pixels_per_point in [1.0, 1.5] {
        let viewport = presented(screen, 4, 4, pixels_per_point);
        let paint = painted_themed(&frame, viewport, screen, &theme);
        let shapes = source_geometry(&paint, viewport, pixels_per_point);
        let (columns, rows) = boundaries(viewport, &paint, pixels_per_point);

        assert_drawn_once_around(
            &line_quads(shapes.edges.as_slice(), pixels_per_point),
            &columns,
            &rows,
            1,
            theme.grid_border.gamma_multiply(0.5 * pixels_per_point),
            &ruled_apart(viewport, &frame, &paint, &theme, pixels_per_point),
        );
    }
}

///
/// The colour each physical pixel the edges cover is painted in, after
/// checking that no two pieces cover the same pixel.
///
fn edge_pixels(quads: &[([i32; 4], Color32)]) -> std::collections::HashMap<(i32, i32), Color32> {
    let mut pixels = std::collections::HashMap::new();
    for &([left, top, right, bottom], colour) in quads {
        for x in left..right {
            for y in top..bottom {
                assert!(
                    pixels.insert((x, y), colour).is_none(),
                    "two pieces cover pixel {x},{y}"
                );
            }
        }
    }
    pixels
}

///
/// Two neighbouring Diagnostic Cells share one line, as strong as their
/// outline: a translucent border is composited once on every edge it wins and
/// at every crossing, and the wider border takes the junctions it meets.
///
#[tokio::test]
async fn neighbouring_diagnostic_cells_share_one_line_composited_once() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(8, 3);
    // `**` at columns 2 and 3 is the rejected Number operand of `.+`.
    for (x, character) in ".+**01".chars().enumerate() {
        orcvs.select(orcvs.grid().position(x, 1).expect("inside the grid"));
        orcvs.write(&character.to_string());
    }
    orcvs.select(orcvs.grid().position(7, 0).expect("inside the grid"));
    let frame = orcvs.render_frame();
    let theme = Theme {
        diagnostic_border: Color32::from_rgba_unmultiplied(255, 255, 255, 96),
        diagnostic_border_width: crate::theme::GridWidth::from_points(1.0).expect("within 0..=1"),
        ..okabe_ito()
    };
    let viewport = presented(screen, 8, 3, 2.0);
    let paint = painted_themed(&frame, viewport, screen, &theme);
    let shapes = source_geometry(&paint, viewport, 2.0);
    let (columns, rows) = boundaries(viewport, &paint, 2.0);
    let pixels = edge_pixels(&line_quads(shapes.edges.as_slice(), 2.0));

    let diagnostic = theme.grid_border.blend(theme.diagnostic_border);
    let grid = theme.grid_border;
    let middle = |from: i32, to: i32| (from + to) / 2;
    // The two Cells' left, shared and right edges, two pixels across their
    // column lines, halfway down row 1.
    let y = middle(rows[1], rows[2]);
    for column in [2, 3, 4] {
        for x in [columns[column] - 1, columns[column]] {
            assert_eq!(
                pixels.get(&(x, y)),
                Some(&diagnostic),
                "column line {column} at {x}"
            );
        }
        assert_eq!(
            pixels.get(&(columns[column] + 1, y)),
            None,
            "column line {column} is two pixels"
        );
    }
    // Their top edges, and the Grid line beside them.
    for column in [2, 3] {
        let x = middle(columns[column], columns[column + 1]);
        assert_eq!(pixels.get(&(x, rows[1] - 1)), Some(&diagnostic));
        assert_eq!(pixels.get(&(x, rows[1])), Some(&diagnostic));
    }
    let x = middle(columns[5], columns[6]);
    assert_eq!(pixels.get(&(x, rows[1])), Some(&grid));
    assert_eq!(
        pixels.get(&(x, rows[1] - 1)),
        None,
        "a Grid line is one pixel"
    );
    // Every crossing the Diagnostic lines meet is theirs, two by two, but
    // the one below their left edge, which `.+`'s Output Portal Cell below
    // it also meets and outranks.
    for (column, row) in [(2, 1), (3, 1), (4, 1), (3, 2), (4, 2)] {
        for (dx, dy) in [(-1, -1), (-1, 0), (0, -1), (0, 0)] {
            assert_eq!(
                pixels.get(&(columns[column] + dx, rows[row] + dy)),
                Some(&diagnostic),
                "the crossing of column line {column} and row line {row}"
            );
        }
    }
}

///
/// A one pixel junction is one of the pieces: epaint's rectangle path would
/// round a one by one pixel rectangle away and leave a hole at the crossing.
///
#[tokio::test]
async fn a_one_pixel_crossing_is_drawn() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let orcvs = running_orcvs(2, 2);
    let frame = orcvs.render_frame();
    // The Cursor's outline as thin as the Grid line, so its corners are one
    // pixel crossings too.
    let theme = Theme {
        cursor_border_width: okabe_ito().grid_border_width,
        ..okabe_ito()
    };
    let viewport = presented(screen, 2, 2, 2.0);
    let paint = painted_themed(&frame, viewport, screen, &theme);
    let shapes = source_geometry(&paint, viewport, 2.0);
    let (columns, rows) = boundaries(viewport, &paint, 2.0);

    let quads = line_quads(shapes.edges.as_slice(), 2.0);
    for &x in &columns {
        for &y in &rows {
            assert!(
                quads.iter().any(|(quad, _)| *quad == [x, y, x + 1, y + 1]),
                "no one pixel piece where the lines cross at {x},{y}"
            );
        }
    }
}

///
/// The Grid lines are painted straight after the backgrounds, so a fill
/// never covers a line.
///
#[tokio::test]
async fn the_grid_lines_lie_over_the_backgrounds() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(4, 4);
    orcvs.select(orcvs.grid().position(1, 1).expect("inside the grid"));
    orcvs.extend(orcvs.grid().position(2, 2).expect("inside the grid"));
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 4, 4, 1.0);
    let shapes = source_geometry(&painted(&frame, viewport, screen), viewport, 1.0);
    let backgrounds = shapes.backgrounds.len();
    assert!(backgrounds > 0, "the Region was not filled");

    let painted = shapes.into_shapes().collect::<Vec<_>>();
    assert!(
        matches!(painted[backgrounds], Shape::Mesh(_)),
        "{:?} followed the backgrounds",
        painted[backgrounds]
    );
}

///
/// A Glyph is painted for every Cell that shows a character, at the centre
/// of that Cell, in that Cell's own foreground — and for no Cell that shows
/// the space.
///
/// What a Cell shows and what colour it is are the Paint's answers, made
/// with no `egui::Context` and asserted against `cell.content()` and
/// `cell_visuals` in `paint.rs`. What this pins is the step between: a
/// galley per Cell that has something to say, positioned on that Cell and
/// handed that Cell's colour rather than its neighbour's.
///
/// The Grid carries an Addition, whose claim reaches past the two Cells it
/// is spelled in and leaves classified but empty operand Cells behind it.
/// No placeholder letter stands in those Cells, so the Cells showing
/// something are exactly the written ones — the unfilled operand Cells
/// are tinted (`style::fill_tint_colour`) but spell nothing, which is what
/// this test's exact count of two asserts rather than only a lower bound.
///
#[tokio::test]
async fn a_glyph_is_painted_for_every_cell_that_shows_one_and_no_other() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(8, 8);
    for (x, character) in ".+".chars().enumerate() {
        orcvs.select(orcvs.grid().position(x, 2).expect("inside the grid"));
        orcvs.write(&character.to_string());
    }
    orcvs.select(orcvs.grid().position(5, 5).expect("inside the grid"));

    let frame = orcvs.render_frame();
    let viewport = presented(screen, 8, 8, 1.0);
    let paint = painted(&frame, viewport, screen);
    let shapes = source_shapes(&paint, viewport, 1.0);

    let shown: Vec<_> = paint
        .cells()
        .filter(|(_, cell)| cell.character != ' ')
        .map(|(position, _)| position)
        .collect();

    assert_eq!(
        shown.len(),
        2,
        "the two written Cells are the only Cells that should show a character, not {}",
        shown.len()
    );
    assert_eq!(
        shapes.glyphs.len(),
        shown.len(),
        "painted {} Glyphs for {} Cells showing a character",
        shapes.glyphs.len(),
        shown.len()
    );
    for (shape, position) in shapes.glyphs.iter().zip(&shown) {
        let Shape::Text(text) = shape else {
            panic!("the Glyph at {position:?} was {shape:?}")
        };
        let rect = viewport.cell_rect(position.x(), position.y());

        assert_eq!(
            text.fallback_color,
            paint.at(*position).foreground,
            "the Glyph at {position:?}"
        );
        assert!(
            (text.pos - (rect.center() - text.galley.size() / 2.0)).length() < 1e-3,
            "the Glyph at {position:?} was painted at {:?}, off the Cell at {rect:?}",
            text.pos
        );
    }
}

///
/// The background the Grid declines to paint is the one the panel paints.
///
/// `show_source` omits a Cell's rectangle wherever `cell_visuals` asks for
/// the resolved Theme's `grid_background`, and what stands in its place
/// is the `CentralPanel` frame. The two values are stated in different
/// places, so nothing but this holds them together: give the panel any
/// other fill and every ordinary Cell — outside the Cursor effect, most of
/// an empty Grid — renders on a ground the Theme never chose for it.
///
/// The whole console is checked rather than the constant alone, because it
/// is the painted result that has to sit on the right colour.
///
#[test]
fn the_omitted_background_is_the_colour_the_panel_is_filled_with() {
    let theme = crate::theme::okabe_ito();
    assert_eq!(
        source_panel_frame(theme.grid_background).fill,
        theme.grid_background,
        "show_source omits a Cell's background wherever cell_visuals asks \
         for the resolved Theme's grid_background, so the panel standing \
         in for it must be filled with exactly that colour"
    );
}

///
/// `source_panel_frame` composites a transparent or partial-alpha
/// `grid_background` exactly as given — never forcing it opaque, and
/// never touching any other `egui::Frame` property — so a Theme's Grid
/// layer reveals the window backdrop `clear_color_is_the_resolved_
/// themes_opaque_window_background` wires underneath it, the same way an
/// opaque `grid_background` composites to itself.
///
#[test]
fn the_grid_panel_frame_composites_a_transparent_or_partial_alpha_background_unchanged() {
    for background in [
        Color32::TRANSPARENT,
        Color32::from_rgba_unmultiplied(10, 20, 30, 128),
        Color32::from_rgba_unmultiplied(10, 20, 30, 255),
    ] {
        assert_eq!(
            source_panel_frame(background).fill,
            background,
            "source_panel_frame changed the Grid background {background:?} it was given"
        );
    }

    // No property but `fill` is a function of `background`: two frames
    // built at different alpha, with their fills equalised, must be
    // identical.
    let opaque = source_panel_frame(Color32::from_rgba_unmultiplied(10, 20, 30, 255));
    let mut transparent = source_panel_frame(Color32::TRANSPARENT);
    transparent.fill = opaque.fill;
    assert_eq!(
        transparent, opaque,
        "source_panel_frame changed a property besides fill across two alpha levels"
    );
}

///
/// The viewport the `presented` helper builds is the viewport a whole
/// console pass presents.
///
/// Every Shape assertion here is made at a viewport built from
/// `grid_viewport` and `presented_grid` directly rather than read back from
/// a pass, and this is what stops that standing in from drifting from what
/// `show_source_scene` does. At a fractional device scale as well as at
/// one, which is where `presented_grid` floors the Cell side to whole
/// physical pixels and the two could part company.
///
#[tokio::test]
async fn the_presented_viewport_is_the_one_a_console_pass_presents() {
    for pixels_per_point in [1.0_f32, 1.5] {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(8, 8);
        let mut view = SourceView::new(CELL_SIZE);

        let (viewport, _) = console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            pixels_per_point,
        );

        assert_eq!(
            ctx.pixels_per_point(),
            pixels_per_point,
            "the pass did not run at the device scale it was asked for"
        );
        assert_eq!(viewport, presented(screen, 8, 8, pixels_per_point));
    }
}

///
/// A coalesced run covers exactly the Cells it replaces.
///
/// Which Cells coalesce is `Paint::background_runs`' answer and is pinned
/// there against a written-out table of spans; where the rectangle lands is
/// `a_background_run_is_the_rectangle_its_columns_span`'s, pinned against
/// written-out coordinates. What this adds is the part only a real
/// presented viewport has: the run starts at the corner its first Cell was
/// given by `GridViewport::cell_rect` — the column-to-x function every
/// other Shape goes through — rather than at a sum of Cell sides
/// accumulated across the row, it spans the Cells it replaced and no
/// others, and those Cells would have tiled at exactly the pixels it
/// covers, every interior edge landing on the same physical pixel from
/// either side.
///
/// Run at a fractional device scale as well as at one, because at one the
/// Cell side is a whole point and there is nothing for a snap to move.
///
#[tokio::test]
async fn a_background_run_covers_exactly_the_cells_it_replaces() {
    for pixels_per_point in [1.0_f32, 1.5] {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let orcvs = running_orcvs(8, 8);
        let frame = orcvs.render_frame();
        let viewport = presented(screen, 8, 8, pixels_per_point);
        let paint = painted(&frame, viewport, screen);
        let shapes = source_geometry(&paint, viewport, pixels_per_point);
        let runs = paint.background_runs();

        assert_eq!(
            shapes.backgrounds.len(),
            runs.len(),
            "painted {} rectangles for {} runs",
            shapes.backgrounds.len(),
            runs.len()
        );

        for (shape, run) in shapes.backgrounds.iter().zip(&runs) {
            let Shape::Rect(painted) = shape else {
                panic!("a background was {shape:?}")
            };

            assert_eq!(painted.fill, run.colour, "a run was filled wrongly");
            // The near corner is the one the run's first Cell was given,
            // and the far corner is a whole number of Cells away from it.
            // Stated as a span rather than as the far Cell's own corner,
            // because that is the expression `SourceShapes::new` evaluates
            // and a test that re-ran it would pass any simultaneous edit to
            // both. `a_background_run_is_the_rectangle_its_columns_span`
            // pins the same mapping against written-out coordinates.
            assert_eq!(
                painted.rect.min,
                viewport
                    .cell_rect(run.columns.start, run.row)
                    .min
                    .round_to_pixels(pixels_per_point),
                "the run over columns {:?} of row {} starts elsewhere",
                run.columns,
                run.row
            );
            // Half a Cell, because the snap rounds the two corners
            // independently and so can move a side by up to half a physical
            // pixel each — while a run one Cell too wide or too narrow is
            // out by a whole Cell side.
            let spanned = run.columns.len() as f32 * viewport.cell_size.x;
            assert!(
                (painted.rect.width() - spanned).abs() < viewport.cell_size.x / 2.0,
                "the run over columns {:?} of row {} is {} wide, not the {spanned} its Cells span",
                run.columns,
                run.row,
                painted.rect.width()
            );
            assert!(
                (painted.rect.height() - viewport.cell_size.y).abs() < viewport.cell_size.y / 2.0,
                "the run over columns {:?} of row {} is {} tall, not one Cell",
                run.columns,
                run.row,
                painted.rect.height()
            );
            for column in run.columns.start..run.columns.end - 1 {
                assert_eq!(
                    viewport
                        .cell_rect(column, run.row)
                        .max
                        .x
                        .round_to_pixels(pixels_per_point),
                    viewport
                        .cell_rect(column + 1, run.row)
                        .min
                        .x
                        .round_to_pixels(pixels_per_point),
                    "the Cells either side of column {column} in row {} do not tile",
                    run.row
                );
            }
        }
    }
}

///
/// A coalesced run becomes the rectangle its columns span, at coordinates
/// written out here rather than re-derived.
///
/// The viewport is stated instead of presented — a 16 point Cell with the
/// Grid's corner at the origin — so every expected rectangle below is a
/// literal. That is the point: an assertion that re-ran
/// `SourceShapes::new`'s own `Rect::from_min_max(cell_rect(start).min,
/// cell_rect(end - 1).max)` would catch a one-sided edit and pass a
/// simultaneous one. These numbers move for neither.
///
/// At one device pixel per point every edge of that Grid is already on a
/// pixel, so the snap moves nothing and the literals are the coordinates
/// the paint lands on. What a *fractional* scale does to them is
/// `a_background_run_covers_exactly_the_cells_it_replaces`'.
///
/// Four runs of the 8 by 8 default Grid, chosen so no one mistake passes
/// all four: the Cursor's own Cell at the origin, a run of three ending
/// short of the Grid's right edge, a run spanning a whole row but its last
/// Cell, and the Grid's last Cell alone. Which Cells those are is
/// `Paint::background_runs`' answer and is pinned in `paint.rs`.
///
#[tokio::test]
async fn a_background_run_is_the_rectangle_its_columns_span() {
    let viewport = GridViewport {
        cell_size: Vec2::splat(16.0),
        rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(128.0)),
    };
    let orcvs = running_orcvs(8, 8);
    let frame = orcvs.render_frame();
    let base = okabe_ito();
    let theme = Theme {
        cursor_background: Some(base.selection_background),
        ..base
    };
    let paint = Paint::derive_with_theme(
        FramePaint::new(
            &frame,
            viewport.visible_positions(viewport.rect, frame.grid()),
        ),
        &theme,
    );
    let shapes = source_geometry(&paint, viewport, 1.0);
    let runs = paint.background_runs();

    assert_eq!(
        shapes.backgrounds.len(),
        runs.len(),
        "painted {} rectangles for {} runs",
        shapes.backgrounds.len(),
        runs.len()
    );

    let (row, columns, expected) = (
        0,
        0..1,
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(16.0, 16.0)),
    );
    let index = runs
        .iter()
        .position(|run| run.row == row && run.columns == columns)
        .unwrap_or_else(|| {
            panic!("this Grid asks for no run over columns {columns:?} of row {row}")
        });
    assert_eq!(
        rect_of(&shapes.backgrounds[index]),
        expected,
        "the run over columns {columns:?} of row {row}"
    );
}

///
/// A whole console pass strokes the Grid at the resolved Theme's own
/// fixed display-point widths — unchanged by the device scale it presented the
/// Source at — and snaps its background runs to the device scale it ran
/// on.
///
/// The device scale is `show_source`'s own arithmetic — the `Ui`'s — and
/// is handed to `SourceShapes::new` and reaches the Shapes nowhere else.
/// The Grid/Sector Seam *widths* are fixed display points, read from
/// `theme` and never multiplied by the presented Cell side over the
/// Source's own or by `pixels_per_point`: at a device scale of 1.1 — egui's
/// zoom factor of 1.1 on a display of one — the snap draws the Cell under 16
/// points, so this is the test that catches a width scaled by either, which
/// at a device scale of one are indistinguishable from the constant. Every other
/// Shape assertion here builds a
/// `SourceShapes` through `source_geometry` or `source_shapes`, which are
/// given a device scale the test chose, so all of them still hold with
/// that argument replaced by a constant one at the call site. What would
/// ship then is a Grid whose lines and sector seams stay the Theme's own
/// width at every device scale, and runs snapped to whole points on a
/// screen whose pixels are not whole points.
///
/// A 400 point console shows the whole 20 Cell Grid with its margin, so
/// the Grid lines cover every Cell.
///
#[tokio::test]
async fn a_console_pass_strokes_at_its_own_theme_width_and_snaps_runs_to_the_device_scale() {
    const DEVICE_SCALE: f32 = 1.1;
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(400.0));
    let mut orcvs = running_orcvs(20, 20);
    let mut view = SourceView::new(CELL_SIZE);
    let theme = okabe_ito();

    let (viewport, shapes) = console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        DEVICE_SCALE,
    );

    assert!(
        viewport.cell_size.x < CELL_SIZE.x && viewport.cell_size.y < CELL_SIZE.y,
        "the pass drew the Source's own {} point Cell, so a width scaled by \
         the Cell would be indistinguishable from the constant",
        CELL_SIZE.y
    );

    // No Cell strokes a border of its own, and the Grid lines carry the
    // Theme's fixed width at this scale, not one scaled by the Cell.
    assert!(
        !shapes
            .iter()
            .any(|shape| matches!(shape, Shape::Rect(painted) if painted.stroke.width > 0.0)),
        "an unclaimed Cell stroked a border of its own"
    );
    let lines = shapes
        .iter()
        .filter(|shape| matches!(shape, Shape::Mesh(_)))
        .cloned()
        .collect::<Vec<_>>();
    let frame = orcvs.render_frame();
    let paint = Paint::derive_with_theme(
        FramePaint::new(&frame, viewport.visible_positions(screen, frame.grid())),
        &theme,
    );
    let (columns, rows) = boundaries(viewport, &paint, DEVICE_SCALE);
    let (width, colour) = expected_line(&theme, DEVICE_SCALE);
    assert_drawn_once_around(
        &line_quads(&lines, DEVICE_SCALE),
        &columns,
        &rows,
        width,
        colour,
        &ruled_apart(viewport, &frame, &paint, &theme, DEVICE_SCALE),
    );

    // And the sector seams lie on the Grid's edges.
    assert!(
        line_quads(&lines, DEVICE_SCALE)
            .iter()
            .any(|(_, piece)| *piece != colour && *piece != theme.cursor_border),
        "the pass drew no sector seam"
    );

    // The default theme leaves the cursor-cell colour unset, so the
    // selected Cell uses the panel's source background and contributes no
    // background run to snap. Explicit colours are covered by the Paint
    // seam tests.
    let runs = paint.background_runs();
    assert!(
        runs.is_empty(),
        "the default cursor colour should be transparent"
    );
}

///
/// A sector seam is drawn on the Cell edge the Paint asks for, composited
/// once over the Grid line it lies on and as wide as the wider of the two,
/// on whole pixels, at every device scale in [`DEVICE_SCALES`]. The
/// Cursor's outline carries none.
///
/// Which Cells carry a seam, and at what strength, are the Render Frame's
/// and the derive's answers and are asserted in `paint.rs` with no Context
/// at all.
///
/// The Grid is 16 Cells square because the default Sector Seam spacing is
/// eight: an 8x8 Grid has no column or row that is a non-zero multiple of
/// it, so every seam strength in one is `None` and this would assert
/// nothing.
///
#[tokio::test]
async fn a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 600.0));
    let mut orcvs = running_orcvs(16, 16);
    // The Cursor goes on a Cell that would otherwise carry both seams.
    let cursor = orcvs.grid().position(8, 8).expect("inside the grid");
    orcvs.select(cursor);
    let frame = orcvs.render_frame();
    let theme = okabe_ito();

    for pixels_per_point in DEVICE_SCALES {
        let viewport = presented(screen, 16, 16, pixels_per_point);
        let paint = painted(&frame, viewport, screen);
        let shapes = source_geometry(&paint, viewport, pixels_per_point);
        let pixels = edge_pixels(&line_quads(shapes.edges.as_slice(), pixels_per_point));
        let (columns, rows) = boundaries(viewport, &paint, pixels_per_point);
        let middle = |from: i32, to: i32| (from + to) / 2;
        let mut seams = 0;

        for (position, cell) in paint.cells() {
            let (x, y) = (position.x(), position.y());
            for (seam, vertical) in [(cell.sector_left, true), (cell.sector_top, false)] {
                let Some(seam) = seam else { continue };
                let (width, colour) = if position == cursor {
                    pixel_line(
                        theme.cursor_border_width.points(),
                        theme.cursor_border,
                        pixels_per_point,
                    )
                } else {
                    seams += 1;
                    expected_seam(&theme, seam, pixels_per_point)
                };
                let boundary = if vertical { columns[x] } else { rows[y] };
                let along = if vertical {
                    middle(rows[y], rows[y + 1])
                } else {
                    middle(columns[x], columns[x + 1])
                };
                let at = |across: i32| {
                    let pixel = if vertical {
                        (across, along)
                    } else {
                        (along, across)
                    };
                    pixels.get(&pixel).copied()
                };
                let start = boundary - width / 2;
                for across in start..start + width {
                    assert_eq!(
                        at(across),
                        Some(colour),
                        "{pixels_per_point}× the edge {position:?} at {across}"
                    );
                }
                assert_eq!(
                    at(start - 1),
                    None,
                    "{pixels_per_point}× beside {position:?}"
                );
                assert_eq!(
                    at(start + width),
                    None,
                    "{pixels_per_point}× beside {position:?}"
                );
            }
        }
        assert!(
            seams > 0,
            "the Paint asked for no sector seam, so nothing was asserted"
        );

        // The seam on column line 8 runs on through its crossing with a plain
        // row line, as strong as the stronger of its edges either side.
        let seam_at = |row| {
            let position = orcvs.grid().position(8, row).expect("inside the grid");
            paint
                .at(position)
                .sector_left
                .map(|seam| expected_seam(&theme, seam, pixels_per_point))
        };
        let (row, (width, colour)) = (1..8)
            .find_map(|row| {
                let (above, below) = (seam_at(row - 1)?, seam_at(row)?);
                Some((
                    row,
                    if above.1.a() > below.1.a() {
                        above
                    } else {
                        below
                    },
                ))
            })
            .expect("a seam either side of a row line");
        let start = columns[8] - width / 2;
        for x in start..start + width {
            assert_eq!(
                pixels.get(&(x, rows[row])),
                Some(&colour),
                "{pixels_per_point}× the seam's crossing with row line {row} at {x}"
            );
        }
    }
}

///
/// Width zero hides the Grid lines, the Cursor's outline and the Sector
/// Seam outright — no zero-width `Shape` left for the painter to drop — at
/// every device scale in [`DEVICE_SCALES`].
///
#[tokio::test]
async fn zero_grid_and_sector_widths_hide_every_border_and_seam_at_every_device_scale() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(600.0));
    let mut orcvs = running_orcvs(16, 16);
    // A sector corner, so the fixture would otherwise draw both a grid
    // line and a seam on plenty of Cells.
    orcvs.select(orcvs.grid().position(8, 8).expect("inside the grid"));
    let frame = orcvs.render_frame();
    // Both line widths zeroed: `grid_border_width` for every edge and
    // `cursor_border_width` for the outline of the Cursor selected above.
    let theme = Theme {
        grid_border_width: crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1"),
        cursor_border_width: crate::theme::GridWidth::from_points(0.0)
            .expect("0.0 is within 0..=1"),
        sector_seam_width: crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1"),
        ..okabe_ito()
    };

    for pixels_per_point in DEVICE_SCALES {
        let viewport = presented(screen, 16, 16, pixels_per_point);
        let paint = painted_themed(&frame, viewport, screen, &theme);
        let shapes = source_geometry(&paint, viewport, pixels_per_point);

        assert!(
            shapes.edges.is_none(),
            "{pixels_per_point} ppp: a Grid line, the Cursor's outline or a seam survived width 0"
        );
    }
}

///
/// `grid.border.width` and `cursor.border.width` are two independent Theme
/// fields, not one constant read twice: the Grid lines take
/// `grid.border.width`, and only the Cursor's outline takes
/// `cursor.border.width`, each faded by its own width below one pixel.
///
#[tokio::test]
async fn the_grid_and_cursor_border_widths_vary_independently() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(320.0));
    // Eight Cells square, so no Sector Seam lies on the lattice.
    let mut orcvs = running_orcvs(8, 8);
    let cursor = orcvs.grid().position(4, 4).expect("inside the grid");
    orcvs.select(cursor);
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 8, 8, 1.0);
    let theme = Theme {
        grid_border_width: crate::theme::GridWidth::from_points(0.2).expect("0.2 is within 0..=1"),
        cursor_border_width: crate::theme::GridWidth::from_points(0.9)
            .expect("0.9 is within 0..=1"),
        ..okabe_ito()
    };
    let paint = painted_themed(&frame, viewport, screen, &theme);
    let shapes = source_geometry(&paint, viewport, 1.0);
    let quads = line_quads(shapes.edges.as_slice(), 1.0);

    let (columns, rows) = boundaries(viewport, &paint, 1.0);
    let outline = outline_box(viewport, cursor, &theme, 1.0);
    assert_drawn_once_around(
        &quads,
        &columns,
        &rows,
        1,
        theme.grid_border.gamma_multiply(0.2),
        &[outline],
    );
    let [left, top, ..] = outline;
    assert_eq!(
        edge_pixels(&quads).get(&(left, top + 4)),
        Some(&theme.cursor_border.gamma_multiply(0.9)),
        "the Cursor's outline is not faded by its own width"
    );
}

///
/// Each line width hides only its own line at zero: a zeroed
/// `cursor.border.width` withdraws the Cursor's outline, and its edges show
/// the Grid line, and a zeroed `grid.border.width` silences the Grid lines
/// while the Cursor keeps its outline. Neither zero reaches the other, which
/// `zero_grid_and_sector_widths_hide_every_border_and_seam_at_every_device_scale`
/// does not show on its own since it zeroes both together.
///
#[tokio::test]
async fn zero_width_suppresses_only_its_own_border_stroke() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(320.0));
    // Eight Cells square, so no Sector Seam lies on the lattice.
    let mut orcvs = running_orcvs(8, 8);
    let cursor = orcvs.grid().position(4, 4).expect("inside the grid");
    orcvs.select(cursor);
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 8, 8, 1.0);

    let zero_cursor = Theme {
        cursor_border_width: crate::theme::GridWidth::from_points(0.0)
            .expect("0.0 is within 0..=1"),
        ..okabe_ito()
    };
    let paint = painted_themed(&frame, viewport, screen, &zero_cursor);
    let shapes = source_geometry(&paint, viewport, 1.0);
    let (columns, rows) = boundaries(viewport, &paint, 1.0);
    let (width, colour) = expected_line(&zero_cursor, 1.0);
    assert_drawn_once(
        &line_quads(shapes.edges.as_slice(), 1.0),
        &columns,
        &rows,
        width,
        colour,
    );

    let zero_grid = Theme {
        grid_border_width: crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1"),
        ..okabe_ito()
    };
    let paint = painted_themed(&frame, viewport, screen, &zero_grid);
    let shapes = source_geometry(&paint, viewport, 1.0);
    let quads = line_quads(shapes.edges.as_slice(), 1.0);
    assert_eq!(
        quads.len(),
        8,
        "zero grid.border.width left more than the Cursor's four edges and corners"
    );
    assert!(
        quads
            .iter()
            .all(|(_, colour)| *colour == zero_grid.cursor_border),
        "zero grid.border.width left a Grid line standing"
    );
}

///
/// A zeroed `cursor.border.width` hides the single-Cell Cursor's outline
/// outright: its edges fall to the Grid line, and the Cursor Effect builds
/// no fragment.
///
#[tokio::test]
async fn zero_cursor_border_width_hides_the_cursors_frame() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(320.0));
    let mut orcvs = running_orcvs(20, 20);
    let cursor = orcvs.grid().position(4, 4).expect("inside the grid");
    orcvs.select(cursor);
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 20, 20, 1.0);
    let theme = Theme {
        cursor_border_width: crate::theme::GridWidth::from_points(0.0)
            .expect("0.0 is within 0..=1"),
        ..okabe_ito()
    };
    let paint = painted_themed(&frame, viewport, screen, &theme);
    let cursor_rect = viewport.cell_rect(4, 4);
    let effect = crate::cursor_effects::cursor_effect_shapes(
        cursor_rect,
        cursor_rect,
        screen,
        viewport.cell_size,
        crate::cursor_effects::CursorEffectMotion::default(),
        theme.cursor_area,
        egui::Stroke::new(theme.cursor_border_width.points(), theme.cursor_border),
    );
    let shapes = source_shapes_with_effect(&paint, viewport, 1.0, effect);

    assert!(
        shapes.cursor.is_empty(),
        "zero cursor.border.width left {} Cursor fragments standing",
        shapes.cursor.len()
    );
    assert!(
        line_quads(shapes.edges.as_slice(), 1.0)
            .iter()
            .all(|(_, colour)| *colour != theme.cursor_border),
        "zero cursor.border.width left the Cursor's outline standing"
    );
}

///
/// The Grid lines stay the resolved Theme's own fixed display-point width at
/// every device scale in [`DEVICE_SCALES`] — never multiplied by the snapped
/// Cell side. A Grid line is that width in physical pixels; the Sector Seams
/// are `a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for`'s.
///
#[tokio::test]
async fn grid_and_sector_widths_stay_fixed_display_points_across_device_scales() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(600.0));
    let mut orcvs = running_orcvs(16, 16);
    orcvs.select(orcvs.grid().position(8, 8).expect("inside the grid"));
    let frame = orcvs.render_frame();
    let theme = okabe_ito();

    for pixels_per_point in DEVICE_SCALES {
        let viewport = presented(screen, 16, 16, pixels_per_point);
        let paint = painted(&frame, viewport, screen);
        let shapes = source_geometry(&paint, viewport, pixels_per_point);

        let (columns, rows) = boundaries(viewport, &paint, pixels_per_point);
        let (width, colour) = expected_line(&theme, pixels_per_point);
        assert_drawn_once_around(
            &line_quads(shapes.edges.as_slice(), pixels_per_point),
            &columns,
            &rows,
            width,
            colour,
            &ruled_apart(viewport, &frame, &paint, &theme, pixels_per_point),
        );
    }
}

///
/// The window's own clear colour is the resolved Theme's opaque
/// `window.background` — `.scratch/theming/schema.md`'s Chrome mapping
/// table, "Application backdrop | `window.background`, opaque" — rather
/// than `eframe::App::clear_color`'s own translucent default, which would
/// otherwise show through wherever a partly transparent panel, Grid or
/// Cell layer reveals the console surface beneath it.
///
/// Each appearance's backdrop is its own Theme's: eframe passes the
/// visuals of the appearance it presents, and the console's installed
/// styles say which appearance they belong to.
///
#[tokio::test]
async fn clear_color_is_the_resolved_themes_opaque_window_background() {
    let (ctx, console, _host) = fresh_console();

    for (slot, theme) in [
        (egui::Theme::Dark, okabe_ito()),
        (egui::Theme::Light, orcvs_light()),
    ] {
        assert_eq!(
            theme.window_background.a(),
            255,
            "the built-in window backdrop must be opaque"
        );
        assert_eq!(
            eframe::App::clear_color(&console, &ctx.style_of(slot).visuals),
            theme.window_background.to_normalized_gamma_f32(),
            "the {slot:?} clear colour must be {}'s own window_background",
            theme.identity
        );
    }
}

///
/// A middle drag that starts on a Cell pans the Source.
///
/// This is what `Sense::CLICK` on the Grid rectangle buys. Within one layer
/// a later-registered child wins the click tie and would win the drag tie
/// too if it sensed drag, and the Grid is registered after the pan
/// response. Sensing clicks alone is what leaves the drag to the pan
/// rectangle, and the drag has to be started *over the Grid* to assert it.
///
#[tokio::test]
async fn a_middle_drag_that_starts_on_a_cell_still_pans_the_source() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let over_a_cell = viewport.cell_rect(4, 4).center();
    assert!(
        viewport.rect.contains(over_a_cell),
        "the drag did not start over the Grid"
    );
    let before = view.pan;

    console_frame(
        &ctx,
        screen,
        middle_press_at(over_a_cell),
        &mut orcvs,
        &mut view,
    );
    console_frame(
        &ctx,
        screen,
        vec![Event::PointerMoved(over_a_cell + Vec2::new(-40.0, -25.0))],
        &mut orcvs,
        &mut view,
    );

    assert_ne!(
        view.pan, before,
        "a middle drag over a Cell did not pan the Source"
    );
}

///
/// Alt (Option) held with a primary drag Pans by exactly what the
/// pointer moved, at a device scale that is not one — the same claim
/// `a_middle_drag_pans_by_the_pointer_and_a_later_click_selects_the_cell_under_it`
/// makes for the middle button, and the same reason a device scale of two
/// is chosen: a multiply by the scale would move the Source by the scale
/// times the pointer.
///
#[tokio::test]
async fn alt_held_with_a_primary_drag_pans_by_exactly_what_the_pointer_moved() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    // egui folds its zoom factor into this same `pixels_per_point`.
    let frame_at = |events: Vec<Event>, orcvs: &mut Orcvs, view: &mut SourceView| {
        console_pass_at(&ctx, screen, events, orcvs, view, 2.0).0
    };

    frame_at(Vec::new(), &mut orcvs, &mut view);
    let anchor = view.origin;

    let from = screen.min + Vec2::splat(40.0);
    let moved = Vec2::new(-24.0, -40.0);
    frame_at(alt_primary_press_at(from), &mut orcvs, &mut view);
    frame_at(
        vec![Event::PointerMoved(from + moved)],
        &mut orcvs,
        &mut view,
    );

    assert_eq!(
        view.origin - anchor,
        moved,
        "the Source panned by {:?} for a pointer that moved {moved:?}",
        view.origin - anchor
    );

    frame_at(alt_primary_release_at(from + moved), &mut orcvs, &mut view);
    assert_eq!(
        selected_cell(&orcvs),
        (0, 0),
        "an Alt-drag moved the Cursor"
    );
}

///
/// A primary drag without Alt does not Pan — it is either a click, which
/// `a_click_selects_the_cell_under_the_pointer_whatever_the_window_size`
/// already covers, or a drag that selects a Region, which
/// `a_primary_drag_spans_a_region_that_release_keeps` covers.
///
#[tokio::test]
async fn a_primary_drag_without_alt_does_not_pan() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let over_a_cell = viewport.cell_rect(4, 4).center();
    let before = view.pan;

    console_frame(&ctx, screen, click_at(over_a_cell), &mut orcvs, &mut view);
    console_frame(
        &ctx,
        screen,
        vec![Event::PointerMoved(over_a_cell + Vec2::new(-40.0, -25.0))],
        &mut orcvs,
        &mut view,
    );

    assert_eq!(
        view.pan, before,
        "a primary drag without Alt panned the Source"
    );
}

fn region_of(orcvs: &Orcvs) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
    let region = orcvs.region();
    (region.columns(), region.rows())
}

///
/// A primary press sets the anchor on the pressed Cell, the drag moves the
/// Cursor to the Cell under the pointer, and release keeps the Region.
///
#[tokio::test]
async fn a_primary_drag_spans_a_region_that_release_keeps() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let (from, to) = (
        viewport.cell_rect(6, 4).center(),
        viewport.cell_rect(2, 2).center(),
    );
    console_frame(&ctx, screen, click_at(from), &mut orcvs, &mut view);
    console_frame(
        &ctx,
        screen,
        vec![Event::PointerMoved(to)],
        &mut orcvs,
        &mut view,
    );
    assert_eq!(region_of(&orcvs), (2..7, 2..5));
    assert_eq!(selected_cell(&orcvs), (2, 2), "the Cursor is the live end");

    console_frame(&ctx, screen, release_at(to), &mut orcvs, &mut view);
    assert_eq!(
        region_of(&orcvs),
        (2..7, 2..5),
        "release dropped the Region"
    );
}

///
/// A primary click collapses the Region onto the clicked Cell, and Shift
/// with a click extends it from the anchor it already has.
///
#[tokio::test]
async fn a_click_collapses_the_region_and_a_shift_click_extends_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    let grid = orcvs.grid();
    orcvs.select(grid.position(1, 1).expect("inside the Grid"));
    orcvs.extend(grid.position(9, 9).expect("inside the Grid"));

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    click(
        &ctx,
        screen,
        viewport.cell_rect(3, 2).center(),
        &mut orcvs,
        &mut view,
    );
    assert_eq!(region_of(&orcvs), (3..4, 2..3));

    let shift_at = viewport.cell_rect(7, 5).center();
    let shifted = |pressed| Event::PointerButton {
        pos: shift_at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::SHIFT,
    };
    console_frame(
        &ctx,
        screen,
        vec![
            Event::PointerMoved(shift_at),
            Event::ModifiersChanged(Modifiers::SHIFT),
            shifted(true),
        ],
        &mut orcvs,
        &mut view,
    );
    // Shift is let go after the button, as a viewer lets it go: the
    // click resolves on release and reads Shift then.
    console_frame(&ctx, screen, vec![shifted(false)], &mut orcvs, &mut view);
    console_frame(
        &ctx,
        screen,
        vec![Event::ModifiersChanged(Modifiers::NONE)],
        &mut orcvs,
        &mut view,
    );
    assert_eq!(region_of(&orcvs), (3..8, 2..6));
    assert_eq!(selected_cell(&orcvs), (7, 5));
}

///
/// A drag past the console's edge moves the Cursor past it and scrolls
/// after it: faster the further the pointer is outside, never more than
/// one Cell a frame, and never past the Grid.
///
#[tokio::test]
async fn a_drag_past_the_edge_scrolls_faster_the_further_out_and_stops_at_the_grid() {
    // The Pan step each of `frames` frames takes with the pointer held
    // `outside` points past the console's right edge.
    fn steps(outside: f32, frames: usize) -> (Vec<f32>, Orcvs, SourceView, Rect) {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::new(CELL_SIZE);
        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let from = viewport.cell_rect(20, 5).center();
        console_frame(&ctx, screen, click_at(from), &mut orcvs, &mut view);
        console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(Pos2::new(
                screen.max.x + outside,
                from.y,
            ))],
            &mut orcvs,
            &mut view,
        );
        let mut steps = Vec::new();
        for _ in 0..frames {
            let before = view.pan.x;
            console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            steps.push(before - view.pan.x);
        }
        (steps, orcvs, view, screen)
    }

    let (near, ..) = steps(8.0, 3);
    let (far, orcvs, view, screen) = steps(200.0, 40);
    let width = CELL_SIZE.x;

    assert!(near[0] > 0.0, "a drag just past the edge did not scroll");
    assert!(
        far[0] > near[0],
        "a drag further out scrolled no faster: {far:?} against {near:?}"
    );
    assert!(
        far.iter().chain(&near).all(|step| *step <= width),
        "a frame scrolled more than one Cell: {far:?}"
    );
    // Never past the Grid: a Cursor on the last Column brings the margin
    // past it into view, and the console's right edge meets the margin's.
    assert_eq!(view.pan.x, screen.width() - 32.0 * width - 2.0 * MARGIN.x);
    assert_eq!(selected_cell(&orcvs), (31, 5));
}

///
/// A Cell wider than tall or taller than wide is measured on its own axis
/// everywhere the Source View measures one: the margin is two Cell widths
/// across and two heights down, a Cursor follow shows the whole Cell, and a
/// drag held past an edge scrolls at most one width across or one height
/// down a frame.
///
#[tokio::test]
async fn an_unequal_cell_is_measured_across_by_its_width_and_down_by_its_height() {
    let cell = Vec2::new(12.0, 18.0);
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(64, 64);
    let mut view = SourceView::new(CELL_SIZE);
    view.cell = cell;

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(viewport.cell_size, cell);
    assert_eq!(viewport.rect.min, screen.min + SOURCE_MARGIN_CELLS * cell);

    // A Cursor move to the far corner Pans the least distance that shows its
    // whole Cell and the margin past it.
    let far = orcvs.grid().position(63, 63).expect("inside the grid");
    orcvs.select(far);
    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        viewport.cell_rect(63, 63).max + SOURCE_MARGIN_CELLS * cell,
        screen.max,
        "the follow did not bring the far Cell and its margin to the console's corner"
    );

    // Back to the top-left, then a drag held far past the bottom-right.
    let near = orcvs.grid().position(0, 0).expect("inside the grid");
    orcvs.select(near);
    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let from = viewport.cell_rect(4, 4).center();
    console_frame(&ctx, screen, click_at(from), &mut orcvs, &mut view);
    console_frame(
        &ctx,
        screen,
        vec![Event::PointerMoved(screen.max + Vec2::splat(400.0))],
        &mut orcvs,
        &mut view,
    );
    for _ in 0..4 {
        let before = view.pan;
        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let step = before - view.pan;
        assert!(
            step.x > 0.0 && step.y > 0.0,
            "the drag did not scroll both ways: {step:?}"
        );
        assert!(
            step.x <= cell.x && step.y <= cell.y,
            "a frame scrolled {step:?}, more than one {cell:?} Cell"
        );
    }
}

///
/// Releasing a drag held past the edge does not jump the Source View to
/// the Cursor: the frame the button comes up still scrolls at most a Cell.
///
#[tokio::test]
async fn releasing_a_drag_past_the_edge_scrolls_no_more_than_a_cell() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(64, 32);
    let mut view = SourceView::new(CELL_SIZE);
    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let from = viewport.cell_rect(20, 5).center();
    let far = Pos2::new(screen.max.x + 400.0, from.y);
    console_frame(&ctx, screen, click_at(from), &mut orcvs, &mut view);
    // Far enough out that the Cursor is many Cells past the edge.
    console_frame(
        &ctx,
        screen,
        vec![Event::PointerMoved(far)],
        &mut orcvs,
        &mut view,
    );

    for events in [release_at(far), Vec::new(), Vec::new()] {
        let before = view.pan.x;
        console_frame(&ctx, screen, events, &mut orcvs, &mut view);
        assert!(
            before - view.pan.x <= CELL_SIZE.x,
            "a frame around the release scrolled {}",
            before - view.pan.x
        );
    }
}

///
/// Alt with a primary drag, and a middle drag, Pan and leave the Region as
/// it was.
///
#[tokio::test]
async fn an_alt_drag_and_a_middle_drag_leave_the_region() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    let grid = orcvs.grid();
    orcvs.select(grid.position(1, 1).expect("inside the Grid"));
    orcvs.extend(grid.position(4, 3).expect("inside the Grid"));
    let region = orcvs.region();

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let from = viewport.cell_rect(8, 8).center();
    let to = from + Vec2::new(-60.0, -40.0);
    for (press, release) in [
        (alt_primary_press_at(from), alt_primary_release_at(to)),
        (
            middle_press_at(from),
            vec![Event::PointerButton {
                pos: to,
                button: egui::PointerButton::Middle,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        ),
    ] {
        let before = view.pan;
        console_frame(&ctx, screen, press, &mut orcvs, &mut view);
        console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(to)],
            &mut orcvs,
            &mut view,
        );
        console_frame(&ctx, screen, release, &mut orcvs, &mut view);
        assert_ne!(view.pan, before, "the drag did not Pan");
        assert_eq!(orcvs.region(), region, "the drag moved the Region");
    }
}

///
/// The view a viewer reaches by panning, set directly.
///
/// The gestures that write the Pan are asserted elsewhere. This is a test
/// building its own input below the shipped entry point rather than a seam
/// cut into one: nothing in `show_source` or `show_source_scene` exists for
/// it.
///
fn pinned_at(view: &mut SourceView, pan: Vec2) {
    view.pan = pan;
}

///
/// The console draws the visible Position range and nothing else, so the
/// cost of a Render Frame follows the viewport rather than the Source.
///
/// Two claims at two layers, and both are here because only a console pass
/// carries the wiring between them. That a zoom in reaches fewer Positions and
/// emits fewer Shapes is asserted through the pass itself. That the edges
/// cover the drawn Positions and no more is asserted against the
/// `SourceShapes` edges group, rather than recovered from the flat list.
/// Both are compared with
/// `GridViewport::visible_positions`' real answer rather than with a
/// counter the console keeps for a test's benefit.
///
/// The shape total is a bound rather than an equality. Consecutive
/// backgrounds coalesce into one rectangle, so a row's shapes are not
/// one per Cell and the two totals are not proportional. What the bound
/// says is that cost follows the viewport, not the Source. No frame time
/// is asserted — epaint already discards off-screen
/// shapes at tessellation (`coarse_tessellation_culling`), so what this
/// saves is Cell iteration and Shape construction. The Cell iteration is
/// counted in `paint.rs`, which needs no `Context` to count it.
///
#[tokio::test]
async fn the_draw_loop_paints_the_visible_range_rather_than_the_whole_source() {
    let ctx = egui::Context::default();
    // A 128 by 80 Grid at the Source's own Cell size, with its margin on
    // every side, is exactly this console, so the first pass at egui's zoom
    // factor of 1.0 has every Cell on screen. Smaller than the one Grid so the whole of
    // it fits a console this test can afford to paint.
    const COLUMNS: usize = 128;
    const ROWS: usize = 80;
    let screen = Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(COLUMNS as f32, ROWS as f32) * CELL_SIZE + 2.0 * MARGIN,
    );
    let mut orcvs = running_orcvs(COLUMNS, ROWS);
    let mut view = SourceView::new(CELL_SIZE);

    let (whole, every_shape) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        whole.cell_size, CELL_SIZE,
        "the console did not open at the Source's own Cell size"
    );
    assert!(
        screen.contains_rect(whole.rect),
        "the Grid at {:?} does not lie wholly inside the console {screen:?}",
        whole.rect
    );
    let all_positions = whole.visible_positions(screen, orcvs.grid());
    assert_eq!(
        all_positions.count(),
        COLUMNS * ROWS,
        "the console did not show the whole Grid at egui's zoom factor of 1.0"
    );

    // egui's zoom in halves the console in points, which is the screen its
    // integration reports once the zoom factor has changed. The frame that
    // changes it follows the Cursor, so the Pan is pinned after it.
    ctx.set_zoom_factor(2.0);
    let zoomed_in = Rect::from_min_size(Pos2::ZERO, screen.size() / 2.0);
    console_pass(&ctx, zoomed_in, Vec::new(), &mut orcvs, &mut view);
    pinned_at(&mut view, Vec2::new(-500.0, -300.0));
    let (zoomed, fewer_shapes) = console_pass(&ctx, zoomed_in, Vec::new(), &mut orcvs, &mut view);
    let some_positions = zoomed.visible_positions(zoomed_in, orcvs.grid());

    assert_eq!(ctx.zoom_factor(), 2.0, "the pass did not run zoomed in");
    assert_eq!(
        zoomed.cell_size, CELL_SIZE,
        "the zoom changed the Cell's points"
    );
    assert!(
        some_positions.columns.start > 0 && some_positions.rows.start > 0,
        "the zoom left the Grid's first Position on screen, so nothing was culled on the near side"
    );
    assert!(
        some_positions.count() * 2 < all_positions.count(),
        "the zoom showed {} of {} Positions, which is not a fraction worth asserting about",
        some_positions.count(),
        all_positions.count()
    );
    // The edges are one Mesh at any range, so what culling saves shows in
    // its vertices rather than in the count of Shapes.
    let vertices = |shapes: &[Shape]| -> usize {
        shapes
            .iter()
            .map(|shape| match shape {
                Shape::Mesh(mesh) => mesh.vertices.len(),
                _ => 0,
            })
            .sum()
    };
    assert!(
        vertices(&fewer_shapes) * 2 < vertices(&every_shape),
        "{} edge vertices for {} Positions against {} for {}",
        vertices(&fewer_shapes),
        some_positions.count(),
        vertices(&every_shape),
        all_positions.count()
    );

    // The Grid lines cover the drawn Positions and no more, at both
    // viewports.
    let frame = orcvs.render_frame();
    for (viewport, clip, positions, pixels_per_point) in [
        (whole, screen, all_positions, 1.0),
        (zoomed, zoomed_in, some_positions, 2.0),
    ] {
        let paint = painted(&frame, viewport, clip);
        assert_eq!(paint.count(), positions.count());
        let shapes = source_geometry(&paint, viewport, pixels_per_point);
        let (columns, rows) = boundaries(viewport, &paint, pixels_per_point);
        let (width, colour) = expected_line(&okabe_ito(), pixels_per_point);
        assert_drawn_once_around(
            &line_quads(shapes.edges.as_slice(), pixels_per_point),
            &columns,
            &rows,
            width,
            colour,
            &ruled_apart(viewport, &frame, &paint, &okabe_ito(), pixels_per_point),
        );
    }
}

///
/// A panned console fills every Cell its Paint asks to fill, and no other,
/// with rectangles built from runs that begin and end mid-row.
///
/// Runs are row-local state, opened and flushed inside one row. Under
/// culling a row no longer starts at column zero or ends at the last
/// column, so a run opens and is flushed at columns the Source's own row
/// does not begin or end at. That the fold gets that right is
/// `Paint::background_runs`' question and is asserted in `paint.rs`, where
/// it needs no viewport. What is asserted here is the half that does need
/// one: that the rectangle the shape step builds from a run whose columns
/// start mid-row still covers exactly the Cells that run replaces.
///
/// The console is small and panned into the middle of the Grid so both
/// sides of the Source are culled. Cursor effects are geometry beneath the Grid and
/// therefore must not reintroduce Cell background runs in this view.
///
#[tokio::test]
async fn a_culled_row_leaves_cursor_effects_out_of_cell_fills() {
    let ctx = egui::Context::default();
    // Narrow enough that every drawn row starts and ends inside the Grid.
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
    let mut orcvs = running_orcvs(COL_COUNT, ROW_COUNT);
    let mut view = SourceView::new(CELL_SIZE);
    // Below the window the console shows, keeping the Cursor near the
    // visible range while its area remains separate geometry.
    orcvs.select(orcvs.grid().position(18, 20).expect("inside the grid"));

    // The Grid's near corner at (-700, -500), so the console shows a window
    // in the middle of it rather than a corner. The Pan places the margin,
    // so it is the margin further.
    pinned_at(&mut view, Vec2::new(-700.0, -500.0) - MARGIN);
    let (viewport, _) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let visible = viewport.visible_positions(screen, orcvs.grid());

    assert!(
        visible.columns.start > 0 && visible.columns.end < COL_COUNT,
        "the Pan culled nothing on one side, so no run begins or ends mid-row"
    );

    let frame = orcvs.render_frame();
    let paint = painted(&frame, viewport, screen);
    let shapes = source_shapes(&paint, viewport, 1.0);
    // The fill covering a point, and how many rectangles claim it. A Cell
    // covered twice is a run painted over a run, which the count catches
    // where a lookup of the first match would not.
    let covering = |point: Pos2| {
        shapes
            .backgrounds
            .iter()
            .filter_map(|shape| match shape {
                Shape::Rect(rect) if rect.rect.contains(point) => Some(rect.fill),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    let mut filled = 0;
    let mut unfilled = 0;
    let mut opened_at_first_drawn = 0;
    let mut flushed_at_last_drawn = 0;
    for (position, cell) in paint.cells() {
        let rect = viewport.cell_rect(position.x(), position.y());

        match cell.background {
            Some(colour) => {
                assert_eq!(
                    covering(rect.center()),
                    vec![colour],
                    "the Cell {position:?} the Paint fills was covered wrongly"
                );
                filled += 1;
                opened_at_first_drawn += usize::from(position.x() == visible.columns.start);
                flushed_at_last_drawn += usize::from(position.x() == visible.columns.end - 1);
            }
            None => {
                assert!(
                    covering(rect.center()).is_empty(),
                    "the Cell {position:?} the Paint leaves to the Source fill was covered"
                );
                unfilled += 1;
            }
        }
    }

    assert_eq!(filled, 0, "Cursor geometry filled a Cell background");
    assert!(unfilled > 0, "the viewport drew no Cells");
    assert_eq!(opened_at_first_drawn, 0);
    assert_eq!(flushed_at_last_drawn, 0);
}

///
/// Every sector seam the Render Frame asks for inside a culled viewport is
/// painted, so culling cannot drop a seam a viewer can see.
///
/// This covers seams at the edges of the visible range, asserted from the
/// Render Frame at a Pan that culls on all four sides, at a device scale of
/// one and at a fractional one. `sector_seams_are_painted_where_the_render_frame_asks_and_never_on_the_cursor`
/// runs on a default `SourceView` — the fit, with every Position drawn — so
/// it says nothing about a range-limited row.
///
/// Only seams lying *strictly* inside the clip are asserted. A segment on
/// the clip's own edge is what the clip is entitled to discard, and it is
/// the only thing the one-Cell margin reaches: a margin Cell's left edge is
/// the right boundary of what is shown, never inside it. So this test pins
/// the claim above, and the margin is the separate, deliberate over-draw its
/// own comment describes.
///
/// Which seams land strictly inside the clip is a property of the Pan. The
/// Sector Seam spacing is 8, so the Pan is chosen in Cells to put a seam
/// column immediately inside the first drawn column and another on the last,
/// and a seam row immediately inside the first drawn row and another on the
/// last: a
/// cull that is short by a Cell on any of those sides drops a seam this
/// asserts. At an arbitrary Pan the nearest seam can sit six columns from
/// the edge and a column-side error goes unseen.
///
#[tokio::test]
async fn a_panned_console_paints_every_sector_seam_inside_the_clip() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
    // The Grid's corner 39.4 Cells up and left once the margin is added:
    // column and row 39 are the first shown, so seam column and row 40 sit
    // just inside the clip, and seam column 72 and row 56 are the last shown.
    let translation = -39.4 * CELL_SIZE - MARGIN;

    let mut asserted = 0;
    for pixels_per_point in [1.0, 1.5] {
        let ctx = egui::Context::default();
        let mut orcvs = running_orcvs(COL_COUNT, ROW_COUNT);
        let mut view = SourceView::new(CELL_SIZE);

        let frame = orcvs.render_frame();
        pinned_at(&mut view, translation);
        let (viewport, shapes) = console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            pixels_per_point,
        );
        let visible = viewport.visible_positions(screen, orcvs.grid());
        let meshes = shapes
            .iter()
            .filter(|shape| matches!(shape, Shape::Mesh(_)))
            .cloned()
            .collect::<Vec<_>>();
        let pixels = edge_pixels(&line_quads(&meshes, pixels_per_point));
        let theme = okabe_ito();
        let pixel = |point: f32| (point * pixels_per_point).round() as i32;

        assert!(
            visible.columns.start > 0
                && visible.columns.end < COL_COUNT
                && visible.rows.start > 0
                && visible.rows.end < ROW_COUNT,
            "{pixels_per_point} ppp culled nothing on one side, so no seam is near a culled edge: {visible:?}"
        );

        for cell in frame.cells() {
            let position = cell.position();
            let rect = viewport.cell_rect(position.x(), position.y());
            let spacing = frame.sector_seam_spacing().cells();
            let selected = position == frame.cursor();

            for (strength, ends) in [
                (
                    crate::marks::sector_left_strength(position, spacing),
                    [rect.left_top(), rect.left_bottom()],
                ),
                (
                    crate::marks::sector_top_strength(position, spacing),
                    [rect.left_top(), rect.right_top()],
                ),
            ] {
                if strength.is_none() || selected {
                    continue;
                }
                let span = Rect::from_two_pos(ends[0], ends[1]);
                let inside = span.min.x > screen.min.x
                    && span.max.x < screen.max.x
                    && span.min.y > screen.min.y
                    && span.max.y < screen.max.y;
                if !inside {
                    continue;
                }

                // The first pixel across the edge, halfway along it.
                let (width, colour) = expected_seam(
                    &theme,
                    crate::style::sector_line(strength.expect("a seam"), theme.sector_seam),
                    pixels_per_point,
                );
                let middle = |from: f32, to: f32| (pixel(from) + pixel(to)) / 2;
                let at = if ends[0].x == ends[1].x {
                    (pixel(ends[0].x) - width / 2, middle(ends[0].y, ends[1].y))
                } else {
                    (middle(ends[0].x, ends[1].x), pixel(ends[0].y) - width / 2)
                };
                assert_eq!(
                    pixels.get(&at),
                    Some(&colour),
                    "the seam at {position:?} is inside the clip at \
                     {pixels_per_point} ppp and was not painted"
                );
                asserted += 1;
            }
        }
    }

    assert!(
        asserted > 0,
        "no seam lies strictly inside the clip, so this asserted nothing"
    );
}

const WIDE: Vec2 = Vec2::new(400.0, 200.0);
const TALL: Vec2 = Vec2::new(200.0, 400.0);

///
/// A Source smaller than the console has nowhere to Pan; one larger than
/// the console reaches its own edges and no further.
///
#[test]
fn clamp_pan_pins_a_smaller_source_at_the_origin_and_a_larger_one_to_its_edges() {
    assert_eq!(
        clamp_pan(
            Vec2::new(-10.0, 5.0),
            Vec2::new(400.0, 200.0),
            Vec2::new(128.0, 128.0)
        ),
        Vec2::ZERO,
        "a smaller Source left the top-left"
    );
    assert_eq!(
        clamp_pan(
            Vec2::new(-200.0, -50.0),
            Vec2::new(400.0, 200.0),
            Vec2::new(512.0, 512.0)
        ),
        Vec2::new(-112.0, -50.0)
    );
    assert_eq!(
        clamp_pan(
            Vec2::new(20.0, -400.0),
            Vec2::new(400.0, 200.0),
            Vec2::new(512.0, 512.0)
        ),
        Vec2::new(0.0, -312.0)
    );
}

///
/// The console opens at the Source's own Cell size whatever the window size, so a resize
/// shows more or less of the Source and never a different Cell size.
///
#[tokio::test]
async fn a_resize_keeps_the_cell_size_and_shows_more_or_less_of_the_source() {
    let ctx = egui::Context::default();
    let wide = Rect::from_min_size(Pos2::ZERO, WIDE);
    let tall = Rect::from_min_size(Pos2::ZERO, TALL);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    let before = console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(before.cell_size, CELL_SIZE);
    let after = console_frame(&ctx, tall, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(after.cell_size, CELL_SIZE, "a resize changed the Cell size");
    assert_eq!(
        after.rect.min,
        tall.min + MARGIN,
        "a resize moved the Source off its margin in from the top-left"
    );
    click(
        &ctx,
        tall,
        after.rect.min + Vec2::new(3.5, 1.5) * after.cell_size,
        &mut orcvs,
        &mut view,
    );
    assert_eq!(selected_cell(&orcvs), (3, 1));
}

///
/// A Pan that would open a gap past an edge after a resize settles back
/// inside the Grid.
///
/// The Pan has already taken the Cursor, at (0, 0), out of view, so the
/// resize does not follow it: what settles the gap is `clamp_pan`.
///
#[tokio::test]
async fn a_resize_that_would_open_a_gap_settles_the_source_view_back_inside() {
    let ctx = egui::Context::default();
    let small = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let large = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    // 32 Cells and a margin of two either side is 36 Cells, so the far edge
    // of a console is 36 Cells less its size in.
    let far_edge = |console: f32| Vec2::splat(console) - 36.0 * CELL_SIZE;
    pinned_at(&mut view, far_edge(200.0));
    console_frame(&ctx, small, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, far_edge(200.0));

    console_frame(&ctx, large, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        far_edge(400.0),
        "the resize left a gap past the Grid: {:?}",
        view.pan
    );
}

///
/// Pinch and command-wheel do not zoom: egui's zoom factor stays at 1.0 and
/// the Cell at the Source's own size.
///
#[tokio::test]
async fn pinch_and_command_wheel_do_not_zoom() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let over = viewport.rect.min + viewport.cell_size;
    for (gesture, events) in [
        ("a pinch", pinch_at(over)),
        ("a command-wheel", command_wheel_at(over)),
    ] {
        console_frame(&ctx, screen, events, &mut orcvs, &mut view);
        let after = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(ctx.zoom_factor(), 1.0, "{gesture} changed egui's zoom");
        assert_eq!(
            after.cell_size, CELL_SIZE,
            "{gesture} changed the Cell size"
        );
    }
}

///
/// A fresh `SourceView`'s first frame does not Pan to the Cursor, however
/// far a fixture puts it from an unpanned top-left origin. The Cursor
/// follow needs a previous Cursor to compare against, and there is none
/// yet on the very first frame — the same reason a Source reload would
/// not surprise a viewer either.
///
#[tokio::test]
async fn a_fresh_source_view_does_not_pan_to_the_cursor_on_its_first_frame() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    // Column 30 at the Source's own Cell size is far outside a 200 point console.
    orcvs.select(orcvs.grid().position(30, 30).expect("inside the grid"));

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(
        view.pan,
        Vec2::ZERO,
        "the first frame panned to a Cursor it had no previous position for"
    );
}

///
/// A Cursor move that would leave the Cursor outside the Source View Pans
/// the least distance that shows the whole Cursor Cell, still bounded by
/// the Grid's edges.
///
#[tokio::test]
async fn a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

    // Column and row 30 end 33 Cells along once the two Cells of margin are
    // counted, entirely past a 200 point console on both axes.
    orcvs.select(orcvs.grid().position(30, 30).expect("inside the grid"));
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(
        view.pan,
        Vec2::splat(200.0) - 33.0 * CELL_SIZE,
        "the Cursor move did not Pan the least distance that shows it: {:?}",
        view.pan
    );
}

///
/// A Cursor move onto the Grid's last or first Column and Row brings the
/// margin past that edge into view with it, so the Grid's edge shows as
/// an edge rather than flush against the console's.
///
#[tokio::test]
async fn a_cursor_follow_to_an_edge_cell_shows_the_margin_past_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

    // 32 Cells and a margin of two either side is 36 Cells, so the far
    // margin's edge meets a 200 point console 36 Cells less 200 points in.
    orcvs.select(orcvs.grid().position(31, 31).expect("inside the grid"));
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::splat(200.0) - 36.0 * CELL_SIZE,
        "the follow to the last Column and Row left the far margin off screen: {:?}",
        view.pan
    );

    orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::ZERO,
        "the follow back to the first Column and Row left the near margin off screen: {:?}",
        view.pan
    );
}

///
/// A zoom that would open a gap past an edge settles the Source View back
/// inside the margin, through the same `clamp_pan` a Pan uses.
///
/// egui's zoom changes the console's size in points and never the Source's,
/// so zooming out widens the console around a Pan pinned to the far edge.
/// The Cursor sits on the far corner the pinned Pan already shows, so the
/// Cursor follow has nothing to do: what settles the gap is `clamp_pan`.
///
#[tokio::test]
async fn a_zoom_that_would_open_a_gap_settles_the_source_view_back_inside() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    orcvs.select(orcvs.grid().position(31, 31).expect("inside the grid"));

    // 32 Cells and a margin of two either side is 36 Cells, so the far edge
    // of a console is 36 Cells less its size in.
    let far_edge = |console: f32| Vec2::splat(console) - 36.0 * CELL_SIZE;
    pinned_at(&mut view, far_edge(200.0));
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        far_edge(200.0),
        "the fixture did not open already pinned to the far edge"
    );

    // At half zoom the same window is 400 points across, the size egui's
    // integration reports once the zoom factor has changed.
    ctx.set_zoom_factor(0.5);
    let zoomed_out = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let viewport = console_frame(&ctx, zoomed_out, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(ctx.zoom_factor(), 0.5, "the pass did not run zoomed out");
    assert_eq!(
        viewport.cell_size, CELL_SIZE,
        "the zoom changed the Cell's points"
    );
    assert_eq!(
        view.pan,
        far_edge(400.0),
        "the zoom left a gap past the Grid: {:?}",
        view.pan
    );
}

///
/// A zoom that would leave the Cursor outside the Source View Pans the
/// least distance that shows it, the same as a Cursor move does.
///
#[tokio::test]
async fn a_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    // Column 13 and row 8 end 16 and 11 Cells along, margin counted:
    // inside a 200 point console and past a 160 point one.
    orcvs.select(orcvs.grid().position(13, 8).expect("inside the grid"));

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::ZERO,
        "the fixture's first frame already Panned"
    );

    // At 1.25 the same window is 160 points across.
    ctx.set_zoom_factor(1.25);
    let zoomed_in = Rect::from_min_size(Pos2::ZERO, Vec2::new(160.0, 160.0));
    console_frame(&ctx, zoomed_in, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(ctx.zoom_factor(), 1.25, "the pass did not run zoomed in");
    // The Cell snaps to 12 by 17.6 points, so the Cursor's Cell ends 16 and
    // 11 of those along, and the least Pan puts that end on the console's.
    let cell = snapped_cell(CELL_SIZE, 1.25);
    assert_eq!(
        view.pan,
        Vec2::splat(160.0) - Vec2::new(16.0, 11.0) * cell,
        "the zoom did not Pan the least distance that shows the Cursor: {:?}",
        view.pan
    );

    // A frame at the zoom it already had is not a zoom, so a Pan away from
    // the Cursor stays where it was put.
    view.pan = Vec2::ZERO;
    console_frame(&ctx, zoomed_in, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO, "an unchanged zoom chased the Cursor");
}

///
/// A browser's page zoom is followed as egui's own zoom is.
///
/// On the web the browser zooms the page and eframe reports it as a larger
/// device pixel ratio, the viewport's `native_pixels_per_point`, with egui's
/// zoom factor left at 1.0. The console shrinks in points all the same, so a
/// Cursor near its far edge would drop out of view without the follow.
///
/// eframe's device-pixel-ratio listener rescales the canvas so its size in
/// points holds, and its resize observer later restores the canvas's physical
/// size, so the new scale and the smaller console can reach egui on separate
/// frames. The passes below arrive in that order.
///
#[tokio::test]
async fn a_browser_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    // Column 13 and row 8 end 16 and 11 Cells along, margin counted:
    // inside a 200 point console and past a 160 point one.
    orcvs.select(orcvs.grid().position(13, 8).expect("inside the grid"));

    console_pass_at(&ctx, screen, Vec::new(), &mut orcvs, &mut view, 1.0);
    assert_eq!(
        view.pan,
        Vec2::ZERO,
        "the fixture's first frame already Panned"
    );

    // A page zoom of 1.25: first the new scale over the same 200 points, then
    // the canvas's physical pixels as they were, 160 points across.
    console_pass_at(&ctx, screen, Vec::new(), &mut orcvs, &mut view, 1.25);
    assert_eq!(view.pan, Vec2::ZERO, "the new scale alone Panned");
    let zoomed_in = Rect::from_min_size(Pos2::ZERO, Vec2::new(160.0, 160.0));
    console_pass_at(&ctx, zoomed_in, Vec::new(), &mut orcvs, &mut view, 1.25);

    assert_eq!(
        ctx.zoom_factor(),
        1.0,
        "the pass changed egui's zoom factor"
    );
    assert_eq!(
        ctx.pixels_per_point(),
        1.25,
        "the pass ran at another scale"
    );
    let cell = snapped_cell(CELL_SIZE, 1.25);
    assert_eq!(
        view.pan,
        Vec2::splat(160.0) - Vec2::new(16.0, 11.0) * cell,
        "the browser zoom did not Pan the least distance that shows the Cursor: {:?}",
        view.pan
    );
}

///
/// A new scale that moves the snapped Cell side, with the console's size in
/// points unchanged, is followed as a resize is: a Cursor the view showed in
/// full at the old side stays in full view at the new one.
///
#[tokio::test]
async fn a_new_cell_side_that_would_leave_the_cursor_outside_the_view_pans_to_show_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    orcvs.select(orcvs.grid().position(9, 9).expect("inside the grid"));

    console_pass_at(&ctx, screen, Vec::new(), &mut orcvs, &mut view, 1.0);
    // The Cursor's Cell, 11 Cells along unpanned, margin counted, flush
    // against the top-left.
    let flush = -11.0 * CELL_SIZE;
    pinned_at(&mut view, flush);
    console_pass_at(&ctx, screen, Vec::new(), &mut orcvs, &mut view, 1.0);
    assert_eq!(view.pan, flush, "the fixture's Pan did not hold");

    // At 1.1 the Cell snaps to 13 by 19 pixels, under 12 by 18 points,
    // which alone would move the Cursor's Cell past the top-left edge.
    let (viewport, _) = console_pass_at(&ctx, screen, Vec::new(), &mut orcvs, &mut view, 1.1);
    assert!(
        viewport.cell_size.x < CELL_SIZE.x && viewport.cell_size.y < CELL_SIZE.y,
        "the Cell side did not change"
    );
    // The Cursor's Cell starts 11 of the new Cells along, and the least Pan
    // puts that start on the console's top-left.
    assert_eq!(
        view.pan,
        -11.0 * viewport.cell_size,
        "the new Cell side did not Pan the least distance that shows the Cursor: {:?}",
        view.pan
    );
}

///
/// A window resize that would leave the Cursor outside the Source View Pans
/// to show it, as a zoom does: both change the console's size in points.
///
#[tokio::test]
async fn a_resize_that_would_leave_the_cursor_outside_the_view_pans_to_show_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    // Column 13 and row 8 end 16 and 11 Cells along, margin counted:
    // inside a 200 point console and past a 160 point one.
    orcvs.select(orcvs.grid().position(13, 8).expect("inside the grid"));

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::ZERO,
        "the fixture's first frame already Panned"
    );

    let shrunk = Rect::from_min_size(Pos2::ZERO, Vec2::new(160.0, 160.0));
    console_frame(&ctx, shrunk, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::splat(160.0) - Vec2::new(16.0, 11.0) * CELL_SIZE,
        "the resize did not Pan the least distance that shows the Cursor: {:?}",
        view.pan
    );

    // A frame at the size it already had is not a resize, so a Pan away from
    // the Cursor stays where it was put.
    view.pan = Vec2::ZERO;
    console_frame(&ctx, shrunk, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO, "an unchanged size chased the Cursor");
}

///
/// A resize leaves a Pan the viewer chose away from the Cursor where it was:
/// only a Cursor the view already showed is followed.
///
#[tokio::test]
async fn a_resize_after_a_pan_away_from_the_cursor_keeps_the_pan() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    // Column 9 and row 5 start 11 and 7 Cells along, margin counted: 132 and
    // 126 points, inside the 200 point console before the Pan.
    orcvs.select(orcvs.grid().position(9, 5).expect("inside the grid"));

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    // A Pan that puts the Cursor off the console's top-left, inside the
    // Grid's bounds at every size below.
    let away = Vec2::new(-160.0, -160.0);
    pinned_at(&mut view, away);
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, away, "the fixture's Pan did not hold");

    // A shrink and a growth: either one, followed, would Pan the Cursor
    // back to the console's top-left.
    for size in [Vec2::new(160.0, 160.0), Vec2::new(240.0, 240.0)] {
        let resized = Rect::from_min_size(Pos2::ZERO, size);
        console_frame(&ctx, resized, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.pan, away,
            "a resize to {size:?} pulled the Pan back to the Cursor: {:?}",
            view.pan
        );
    }
}

///
/// A Pan with no Cursor move and no zoom is not pulled back to the
/// Cursor, even while the Cursor sits outside the Source View.
///
#[tokio::test]
async fn a_pan_with_no_cursor_move_and_no_zoom_is_not_pulled_back_to_the_cursor() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    orcvs.select(orcvs.grid().position(30, 30).expect("inside the grid"));

    // First frame: no previous Cursor to compare against, so no follow.
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO);

    // A wheel Pan all the way to the far edge, with the Cursor still
    // unmoved at (30, 30) and no zoom, repeated until the Pan settles
    // however far `smooth_scroll_delta` hands out a frame. Left to the
    // follow this would land 33 Cells less 200 points in instead — see
    // `a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it`
    // — so landing on the margin's far edge, 36 Cells less 200 points in,
    // is what proves a Pan alone is not chasing the Cursor.
    let over = screen.min + Vec2::splat(50.0);
    wheel_until_settled(
        &ctx,
        screen,
        over,
        Vec2::new(-1_000.0, -1_000.0),
        &mut orcvs,
        &mut view,
    );

    assert_eq!(
        view.pan,
        Vec2::splat(200.0) - 36.0 * CELL_SIZE,
        "a Pan alone was pulled back toward the Cursor: {:?}",
        view.pan
    );
}

///
/// The follow Pan is itself naive — it only asks whether the Cursor's
/// Cell already shows inside the console — so an already out-of-bounds
/// Pan it leaves untouched still has to settle back inside the Grid
/// through `clamp_pan`, the same as an ordinary Pan or a zoom does.
///
#[tokio::test]
async fn the_follow_pan_is_still_bounded_by_the_grids_edges() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

    // A Pan past the Grid's near edge, which nothing but `clamp_pan` can
    // answer: at the Source's own Cell size the Cursor's new Cell (5, 0) sits at 80..96,
    // already inside a 200 point console once this Pan is applied, so the
    // follow itself has nothing to add.
    pinned_at(&mut view, Vec2::new(50.0, 50.0));
    orcvs.select(orcvs.grid().position(5, 0).expect("inside the grid"));
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(
        view.pan,
        Vec2::ZERO,
        "clamp_pan did not settle the follow's own Pan back inside the Grid: {:?}",
        view.pan
    );
}

/// A device scale at which the Source's own Cell is not a whole number of
/// physical pixels — egui's zoom factor of 1.1 on a display of one: 16
/// points at 1.1 is 17.6 pixels, which `presented_grid` floors to 17, a Cell
/// of about 15.45 points rather than the Source's 16.
const FRACTIONAL_PPP: f32 = 1.1;

/// Half a physical pixel at [`FRACTIONAL_PPP`], the most the corner's own
/// pixel rounding can move an edge.
const HALF_A_PIXEL: f32 = 0.5 / FRACTIONAL_PPP;

///
/// A Source smaller than the console starts at the console's top-left even
/// where the snap has shrunk its Cells, rather than re-centred inside the
/// unsnapped extent and drawn in from the corner.
///
#[tokio::test]
async fn a_snapped_grid_smaller_than_the_console_starts_at_its_top_left() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(8, 8);
    let mut view = SourceView::new(CELL_SIZE);
    pinned_at(&mut view, Vec2::ZERO);

    let (viewport, _) = console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        FRACTIONAL_PPP,
    );

    assert!(
        viewport.cell_size.x < CELL_SIZE.x && viewport.cell_size.y < CELL_SIZE.y,
        "the fixture snapped nothing, so it asserts nothing: {}",
        viewport.cell_size
    );
    assert_eq!(
        viewport.rect.min,
        screen.min + SOURCE_MARGIN_CELLS * viewport.cell_size,
        "a snapped Grid smaller than the console left its margin in from the top-left"
    );
}

///
/// A Pan to the far edge of a Source larger than the console leaves no
/// gap past the Grid where the snap has shrunk its Cells: the bound is
/// the extent the Cells are drawn at, not the one the Source asks for.
///
#[tokio::test]
async fn a_far_edge_pan_leaves_no_gap_past_a_snapped_grid() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(64, 64);
    let mut view = SourceView::new(CELL_SIZE);
    pinned_at(&mut view, Vec2::splat(-1_000_000.0));

    let (viewport, _) = console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        FRACTIONAL_PPP,
    );

    assert!(
        viewport.cell_size.x < CELL_SIZE.x && viewport.cell_size.y < CELL_SIZE.y,
        "the fixture snapped nothing, so it asserts nothing: {}",
        viewport.cell_size
    );
    let gap = screen.max - viewport.rect.max - SOURCE_MARGIN_CELLS * viewport.cell_size;
    assert!(
        gap.x.abs() <= HALF_A_PIXEL && gap.y.abs() <= HALF_A_PIXEL,
        "the far-edge Pan left {gap:?} past the margin between the Grid and the console's edge"
    );
}

///
/// A Cursor move past the console's far edge Pans the whole Cursor Cell,
/// as drawn, into view — measured at the snapped Cell side the Cells are
/// painted at rather than the Source's unsnapped side.
///
#[tokio::test]
async fn a_cursor_follow_shows_the_whole_snapped_cursor_cell() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(64, 64);
    let mut view = SourceView::new(CELL_SIZE);
    pinned_at(&mut view, Vec2::ZERO);

    console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        FRACTIONAL_PPP,
    );
    // Column and row 26 end 27 Cells and a margin in, past a 400 point
    // console at either Cell side.
    orcvs.select(orcvs.grid().position(26, 26).expect("inside the grid"));
    let (viewport, _) = console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        FRACTIONAL_PPP,
    );

    let cell = viewport.cell_rect(26, 26);
    assert!(
        cell.min.x >= screen.min.x - HALF_A_PIXEL
            && cell.min.y >= screen.min.y - HALF_A_PIXEL
            && cell.max.x <= screen.max.x + HALF_A_PIXEL
            && cell.max.y <= screen.max.y + HALF_A_PIXEL,
        "the followed Cursor Cell {cell:?} is not wholly inside {screen:?}"
    );
}

///
/// A double click inside the Grid selects the Cell under it and leaves the
/// view where it was.
///
#[tokio::test]
async fn a_double_click_selects_a_cell_and_does_not_reset_the_view() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    pinned_at(&mut view, Vec2::new(-24.0, -48.0));

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let before = view.origin;
    let target = viewport.cell_rect(5, 5).center();
    double_click(&ctx, screen, target, &mut orcvs, &mut view);

    assert_eq!(
        selected_cell(&orcvs),
        (5, 5),
        "the double click did not reach the Cell under it"
    );
    assert_eq!(
        view.origin, before,
        "the double click reset a view that no longer has a fit to return to"
    );
    assert_eq!(view.pan, Vec2::new(-24.0, -48.0));
}

///
/// A middle-button drag pans the Source by exactly what the pointer moved,
/// at a device scale that is not one, and a later click still selects the
/// Cell under the pointer.
///
/// `Response::drag_delta` is in points and divides by a layer transform's
/// scaling *only when the layer has one*
/// (`Context::layer_transform_from_global`). The console sets none, so a
/// multiply by the scale would move the Source by the scale times the
/// pointer. A device scale of two — what egui's zoom factor of two gives on
/// a display of one — is what makes that bug visible.
///
#[tokio::test]
async fn a_middle_drag_pans_by_the_pointer_and_a_later_click_selects_the_cell_under_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    // egui folds its zoom factor into this same `pixels_per_point`.
    let frame_at = |events: Vec<Event>, orcvs: &mut Orcvs, view: &mut SourceView| {
        console_pass_at(&ctx, screen, events, orcvs, view, 2.0).0
    };

    let before = frame_at(Vec::new(), &mut orcvs, &mut view);
    let anchor = view.origin;

    // Press, then move further than `max_click_dist` so the gesture
    // resolves as a drag rather than a click. Drag left and up: the Pan
    // is top-left-anchored, so a drag the other way is clamped at zero.
    let from = screen.min + Vec2::splat(40.0);
    let moved = Vec2::new(-24.0, -40.0);
    frame_at(middle_press_at(from), &mut orcvs, &mut view);
    let after = frame_at(
        vec![Event::PointerMoved(from + moved)],
        &mut orcvs,
        &mut view,
    );

    assert_eq!(
        view.origin - anchor,
        moved,
        "the Source panned by {:?} for a pointer that moved {moved:?}",
        view.origin - anchor
    );
    assert_eq!(
        after.rect.min - before.rect.min,
        moved,
        "the presented Grid moved by {:?}",
        after.rect.min - before.rect.min
    );
    assert_eq!(
        after.cell_size, before.cell_size,
        "the pan changed the Cell size"
    );

    frame_at(
        vec![Event::PointerButton {
            pos: from + moved,
            button: egui::PointerButton::Middle,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
        &mut orcvs,
        &mut view,
    );
    let target = after.rect.min + Vec2::new(3.5, 1.5) * after.cell_size;
    click(&ctx, screen, target, &mut orcvs, &mut view);

    assert_eq!(selected_cell(&orcvs), (3, 1), "a click at {target:?}");
}

///
/// Wheel and two-finger scroll Pan a Source larger than the console as far
/// as its edges and no further. A Source that already fits does not Pan.
///
#[tokio::test]
async fn wheel_pans_a_larger_source_to_its_edges_and_a_smaller_one_nowhere() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let over = screen.min + Vec2::splat(16.0);

    let mut large = running_orcvs(32, 32);
    let mut view = SourceView::new(CELL_SIZE);
    console_frame(&ctx, screen, Vec::new(), &mut large, &mut view);
    wheel_until_settled(
        &ctx,
        screen,
        over,
        Vec2::new(-1_000.0, -1_000.0),
        &mut large,
        &mut view,
    );
    assert_eq!(
        view.pan,
        Vec2::new(
            WIDE.x - CELL_SIZE.x * 32.0 - 2.0 * MARGIN.x,
            WIDE.y - CELL_SIZE.y * 32.0 - 2.0 * MARGIN.y
        ),
        "the wheel left the Source short of its edge: {:?}",
        view.pan
    );

    let mut small = running_orcvs(8, 6);
    let mut small_view = SourceView::new(CELL_SIZE);
    console_frame(&ctx, screen, Vec::new(), &mut small, &mut small_view);
    console_frame(
        &ctx,
        screen,
        wheel_at(over, Vec2::new(-80.0, -40.0)),
        &mut small,
        &mut small_view,
    );
    assert_eq!(
        small_view.pan,
        Vec2::ZERO,
        "a Source smaller than the console panned"
    );
}

///
/// No layer the Source Grid is painted into carries a transform, so no
/// `TextShape` in it reaches `Arc::make_mut`. `GraphicLayers::drain`
/// applies a layer's transform to every shape in it at end of pass, and for
/// a `TextShape` that is `TextShape::transform`, which reaches the galley
/// through `Arc::make_mut` and each of its rows through `Arc::make_mut`
/// again. epaint's `GalleyCache` holds an `Arc` to every galley it hands
/// out, so the refcount is never one and the clone is never elided. Given a
/// transform the clone is unconditional, so the *absence* of the transform
/// is the whole proof — there is nothing else to observe, and a profile
/// would not show it anyway, because the clone happens inside `end_pass`
/// rather than in `tessellate_shapes`.
///
/// The Source is presented one margin in from the console's corner, so a
/// transform that presented it would not be the identity: on purpose,
/// because `Context::set_transform_layer` *removes* the entry for an
/// identity transform, which would let a Scene pass this.
///
#[tokio::test]
async fn no_layer_carrying_the_source_grid_is_transformed() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(256.0));
    let mut orcvs = running_orcvs(8, 8);
    orcvs.write("1");
    let mut view = SourceView::new(CELL_SIZE);
    let frame = orcvs.render_frame();
    let mut grid_layer = None;

    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |root| {
            egui::CentralPanel::default()
                .frame(source_panel_frame(
                    crate::theme::okabe_ito().grid_background,
                ))
                .show(root, |ui| {
                    grid_layer = Some(ui.layer_id());
                    show_source_scene(
                        ui,
                        &frame,
                        &egui::FontFamily::Monospace,
                        &mut view,
                        crate::cursor_effects::CursorEffectMotion::default(),
                        &crate::theme::okabe_ito(),
                    );
                });
        },
    );
    let mut painted = Vec::new();
    for clipped in &output.shapes {
        flatten(clipped.shape.clone(), &mut painted);
    }
    output.drop_without_applying_deltas();

    assert_ne!(
        view.origin,
        Pos2::ZERO,
        "the console presented the Source at the identity"
    );
    assert!(
        painted.iter().any(|shape| matches!(shape, Shape::Text(_))),
        "the pass painted no Glyph, so it proves nothing about galleys"
    );

    let grid_layer = grid_layer.expect("the central panel showed the Source");
    assert_eq!(
        ctx.layer_transform_to_global(grid_layer),
        None,
        "the Source Grid's own layer carries a transform"
    );
    // Not just that layer: no layer at all. An egui `Scene` paints into a
    // sublayer of its own, so checking only the layer the console paints
    // into would miss its transform.
    assert!(
        ctx.memory(|memory| memory.to_global.is_empty()),
        "a layer transform survived: {:?}",
        ctx.memory(|memory| memory.to_global.clone())
    );
}

///
/// The Panel names a destination the browser opened "Opening …" until Playback
/// publishes it installed, so the publish must bring the frame that clears the
/// message. Here Playback takes the install only after the console has stopped
/// asking for frames, the order a slow engine turn produces.
///
#[tokio::test]
async fn the_panel_repaints_once_playback_publishes_an_installed_destination() {
    use crate::web_midi::{
        AccessStatus,
        tests::{FakeWebMidi, backend},
    };

    let fake = FakeWebMidi::with(AccessStatus::Granted, &[("one", Some("Studio Synth"))]);
    fake.close_port("one");
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
    console.midi = crate::midi::MidiDeviceSelection::new(
        console.orcvs.midi_selection_handle(),
        Box::new(backend(&fake)),
    );
    console.midi.refresh_destinations();
    app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
    fake.answer_open("one", Ok(()));
    for _ in 0..4 {
        if app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host)
            > std::time::Duration::ZERO
        {
            break;
        }
    }
    assert_eq!(console.midi.status(), Some("Opening Studio Synth…"));
    let handle = console.orcvs.midi_selection_handle();
    tokio::time::timeout(ENGINE_WAIT, async {
        while handle.selected_destination_id().ok().flatten().is_none() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Playback never installed the opened destination");

    assert!(
        ctx.has_requested_repaint(),
        "Playback published the installed destination and no frame was asked for, so the \
         Panel still shows the status it painted"
    );
}
