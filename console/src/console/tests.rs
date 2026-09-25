use egui::{
    Color32, Event, Key, Modifiers, MouseWheelUnit, Pos2, Rect, Shape, TouchPhase, Vec2,
    emath::GuiRounding as _, emath::TSTransform,
};
use orcvs::app::{Arrow, InputEvent, InputKey, Orcvs};
use orcvs::render_frame::RenderFrame;

use crate::grid_viewport::{CELL_SIZE, GridViewport, presented_grid};
use crate::paint::{FramePaint, Paint};
use crate::theme::{Theme, okabe_ito, orcvs_light};
use crate::theme_registry::ThemeRegistry;
use orcvs::grid::{COL_COUNT, Grid, ROW_COUNT};

use super::diagnostics_window::frames_per_second;
use super::glyphs::{ALPHABET_FIRST, ALPHABET_LAST, GLYPH_SCALE_STEP, GlyphTable, glyph_scale};
use super::input::{ZoomCommand, translate_event, zoom_command};
use super::menu_bar::TOP_PANEL_HEIGHT;
use super::panel::{BOTTOM_PANEL_HEIGHT, BOTTOM_PANEL_LEFT_PAD, BPM_FIELD_MARGIN};
use super::shapes::SourceShapes;
use super::source_view::{
    MAX_ZOOM, MIN_ZOOM, SOURCE_MARGIN_CELLS, SourceView, clamp_pan, is_presentable,
    show_source_scene, source_bounds, source_panel_frame, stepped_zoom,
};
use super::{Console, DEFAULT_FONT_SIZE, DEFAULT_VIEW_SIZE};

/// The Source View's margin at Zoom 1.0 and a device scale of one.
const MARGIN: f32 = SOURCE_MARGIN_CELLS * CELL_SIZE;

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

///
/// A command Zoom chord for `key`, as `pinch_at` and `command_wheel_at`
/// stage a pointer gesture: the one event a real `=`/`+`/`-`/`0` press
/// under a held command modifier delivers, with no accompanying
/// `Event::Text` — `egui-winit` and eframe's web backend both withhold it
/// while a command modifier is held.
///
fn command_zoom_at(key: Key) -> Vec<Event> {
    vec![command_key_event(key)]
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
/// The command chord [`zoom_command`] answers, and the bare key it never
/// answers for: `=`, `+`, `-` and `0` ask for a Zoom only with
/// [`egui::Modifiers::command`] held, and an unrelated command chord asks
/// for nothing.
///
#[test]
fn only_a_command_chord_of_the_four_keys_asks_for_a_zoom() {
    let cases = [
        (Key::Equals, ZoomCommand::In),
        (Key::Plus, ZoomCommand::In),
        (Key::Minus, ZoomCommand::Out),
        (Key::Num0, ZoomCommand::Reset),
    ];
    for (key, command) in cases {
        assert_eq!(
            zoom_command(&command_key_event(key)),
            Some(command),
            "command {key:?} did not ask for a Zoom"
        );
        assert_eq!(
            zoom_command(&key_event(key, true)),
            None,
            "bare {key:?} asked for a Zoom"
        );
        assert_eq!(
            zoom_command(&Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            }),
            None,
            "a released command {key:?} asked for a Zoom"
        );
    }

    assert_eq!(
        zoom_command(&command_key_event(Key::C)),
        None,
        "an unrelated command chord asked for a Zoom"
    );
}

///
/// A Zoom step is exact: `In` and `Out` move by one [`GLYPH_SCALE_STEP`]
/// from the nearest multiple of it, `Reset` always lands on 1.0, and every
/// step stops at [`MIN_ZOOM`] or [`MAX_ZOOM`] rather than passing it.
///
#[test]
fn a_zoom_step_moves_by_one_step_and_stops_at_the_range() {
    assert_eq!(stepped_zoom(1.0, ZoomCommand::In), 1.125);
    assert_eq!(stepped_zoom(1.0, ZoomCommand::Out), 0.875);
    assert_eq!(stepped_zoom(1.375, ZoomCommand::Reset), 1.0);
    assert_eq!(stepped_zoom(MIN_ZOOM, ZoomCommand::Reset), 1.0);

    assert_eq!(stepped_zoom(MAX_ZOOM, ZoomCommand::In), MAX_ZOOM);
    assert_eq!(stepped_zoom(MIN_ZOOM, ZoomCommand::Out), MIN_ZOOM);

    // Every step a keyboard Zoom can reach is a whole number of eighths,
    // and `glyph_scale` — the atlas budget `GLYPH_SCALE_STEP` states —
    // has to floor every one of them to itself rather than to the step
    // below.
    let mut zoom = MIN_ZOOM;
    let mut steps = 0;
    while zoom < MAX_ZOOM {
        let stepped = stepped_zoom(zoom, ZoomCommand::In);
        assert!(
            stepped > zoom,
            "In did not move the Zoom forward from {zoom}"
        );
        assert_eq!(
            glyph_scale(stepped),
            stepped,
            "the Glyph was not laid out at the Cell size of the {stepped} step"
        );
        zoom = stepped;
        steps += 1;
    }
    assert_eq!(
        steps, 14,
        "the keyboard range holds fifteen steps, not {steps} moves between them"
    );
}

///
/// The frame rate is still derived; the Source zoom no longer is. Under an
/// owned transform the zoom the diagnostics show *is* `scaling`, so what
/// used to be a ratio recovered from a Scene-space region collapsed to a
/// field read and `scene_zoom` went with it. What is left to assert is that
/// the field the diagnostics read is never a value they cannot show: the
/// guard is what makes the read safe.
///
#[test]
fn diagnostics_derive_frame_rate_and_read_the_source_zoom_from_the_owned_transform() {
    assert_eq!(frames_per_second(0.02), Some(50.0));
    assert_eq!(frames_per_second(0.0), None);

    let zoomed = TSTransform::new(Vec2::new(11.0, 7.0), 2.0);
    assert!(is_presentable(zoomed));
    assert_eq!(zoomed.scaling, 2.0);
    // The visible Source region the diagnostics show is the console area
    // read back through the transform, which is what the Scene-space
    // rectangle used to hold directly.
    assert_eq!(
        zoomed.inverse() * Rect::from_min_size(Pos2::new(11.0, 7.0), Vec2::new(800.0, 400.0)),
        Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))
    );

    for unpresentable in [
        TSTransform::from_scaling(0.0),
        TSTransform::from_scaling(-1.0),
        TSTransform::from_scaling(f32::NAN),
        TSTransform::from_translation(Vec2::new(0.0, f32::NAN)),
    ] {
        assert!(
            !is_presentable(unpresentable),
            "{unpresentable:?} reached the diagnostics"
        );
    }
}

///
/// At Zoom 1.0 the Glyph is 11.5 points inside the Source's 16 point Cell.
///
#[test]
fn the_glyph_at_zoom_one_is_eleven_point_five_points() {
    assert_eq!(DEFAULT_FONT_SIZE * glyph_scale(1.0), 11.5);
}

///
/// The step the glyph scale is quantised to, and the budget it implies.
/// Both zoom limits and the Source's own scale land on a step, so the
/// default window lays its Glyphs out at exactly the Source's font size.
///
#[test]
fn the_glyph_scale_is_quantised_to_a_stated_step() {
    assert_eq!(glyph_scale(1.0), 1.0);
    assert_eq!(glyph_scale(MIN_ZOOM), MIN_ZOOM);
    assert_eq!(glyph_scale(MAX_ZOOM), MAX_ZOOM);
    assert_eq!(glyph_scale(1.01), 1.0, "a nudge re-laid the whole alphabet");
    // Downwards, so the Glyph keeps its share of the Cell: a zoom part way
    // into a step is laid out at the step it is past, not the one it is
    // approaching.
    assert_eq!(glyph_scale(1.1), 1.0);
    assert_eq!(glyph_scale(1.13), 1.125);
    // Never zero, never negative, whatever reaches it.
    for degenerate in [0.0, -1.0, f32::NAN, f32::INFINITY, 1e-9] {
        assert!(
            glyph_scale(degenerate) >= GLYPH_SCALE_STEP,
            "{degenerate} laid out at a font size of {}",
            glyph_scale(degenerate)
        );
    }

    // Fifteen distinct sizes over the whole zoom range is the atlas budget
    // `GLYPH_SCALE_STEP` states. Swept in exact thousandths rather than by
    // accumulating one: the top of the range is reached by the
    // `zoom_range` clamp exactly, and a sum that drifts past it would drop
    // the step it lands on.
    let mut sizes: Vec<f32> = Vec::new();
    for thousandth in (MIN_ZOOM * 1_000.0) as u32..=(MAX_ZOOM * 1_000.0) as u32 {
        let scale = glyph_scale(thousandth as f32 / 1_000.0);
        if !sizes.iter().any(|held| (held - scale).abs() < 1e-6) {
            sizes.push(scale);
        }
    }
    assert_eq!(
        sizes.len(),
        15,
        "the zoom range holds {} sizes",
        sizes.len()
    );
}

///
/// A Glyph is never laid out at a larger fraction of its Cell than the fit
/// gave it.
///
/// The quantisation step is an absolute one, so rounding to the nearest
/// step is disproportionate at a small scale: a console fitting at 0.2
/// rounds up to 0.25 and lays an 11.5 point Glyph out at 2.875 points inside a
/// 3.2 point Cell, where the same Glyph at the Source's own scale takes 11.5 of
/// 16. Under the retired Scene the layer scaled the Glyph exactly, so this
/// is the proportion the effort's strict-parity rule is about. Quantising
/// downwards keeps it and costs at most one step of sharpness.
///
/// The floor at [`GLYPH_SCALE_STEP`] is the one deliberate exception, and
/// the case above it is what this pins.
///
#[test]
fn a_glyph_is_never_laid_out_larger_than_the_scale_it_is_drawn_at() {
    // A sweep at half the step, so it lands both on steps and between them.
    let mut scaling = GLYPH_SCALE_STEP;
    while scaling <= MAX_ZOOM {
        assert!(
            glyph_scale(scaling) <= scaling,
            "a Glyph at {scaling} was laid out at {}",
            glyph_scale(scaling)
        );
        scaling += GLYPH_SCALE_STEP / 2.0;
    }
    // The fit below the zoom floor is where the rounding was worst.
    assert_eq!(glyph_scale(0.2), 0.125);
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
/// one's to avoid jitter (`egui-0.36.2/src/context.rs:437-447`) — so the
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
                    crate::cursor_effects::CursorEffectMotion::default(),
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
        let mut view = SourceView::default();

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
/// A console too small for the Source still opens at Zoom 1.0 and shows the
/// top-left of the Grid. A click still selects the Cell under the pointer.
///
#[tokio::test]
async fn a_click_selects_the_cell_under_the_pointer_in_a_console_smaller_than_the_source() {
    for screen_size in [Vec2::new(400.0, 102.0), Vec2::new(102.0, 400.0)] {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

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
/// runs on its own task (ADR 0041), so this waits on a fact only that task can
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
/// The same three calls `show_source_scene` makes, and for the reason the
/// `presented` helper cannot serve here: that one fits the Grid to a console
/// area, and a running Console's console area is the screen less whatever
/// height the menu bar settled the top panel at. The stored transform
/// already carries that fit, so this asks it rather than re-deriving it.
///
fn console_viewport(ctx: &egui::Context, console: &Console) -> GridViewport {
    let grid = console.orcvs.render_frame().grid();

    presented_grid(
        console.source_view.to_global,
        source_bounds(grid),
        grid,
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
/// eframe restores egui memory — `ThemePreference` included — before it
/// calls `Console::new`, on native and on web. Standing in for that
/// restore: setting the preference on a fresh `Context` before
/// `Console::new` runs, the way `.scratch/theming/issues/02-…` describes
/// the defect. The removed `set_theme(Dark)` call used to overwrite
/// whatever this set; `install` must not.
///
/// Each egui slot holds its own appearance's default built-in, so the
/// restored Light preference presents Orcvs Light rather than egui's own
/// default light style. The style comparison is `Visuals`, not
/// `Style`'s own `PartialEq`: `Style::number_formatter` compares by
/// `Arc::ptr_eq` (`egui-0.36.2/src/style.rs:57-60`), so two
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
/// egui's own default, `System`. `Console::new` must not force `Dark`
/// the way the removed `set_theme(Dark)` call did — a build without the
/// `persistence` feature has nothing else that would set a preference,
/// so `System` is what it opens with.
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
        (panel.outer_rect.height() - BOTTOM_PANEL_HEIGHT).abs() < 0.5,
        "the Panel is not {BOTTOM_PANEL_HEIGHT} tall: {:?}",
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
    // Reduced motion keeps the Cursor Effect's own wakes out of the delay.
    console.reduced_motion = true;
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
/// At 1 BPM a Tick lasts 15 seconds. Cursor Effect off, so its 45–190 ms
/// wakes cannot hide a missing Run Clock remainder. A quiet Playing pass
/// must still request a delay of at most one second.
///
#[tokio::test]
async fn a_playing_console_with_cursor_effect_off_wakes_within_a_second() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    console.reduced_motion = true;
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

    *console.cursor_effects.amount_mut() = 0;
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
/// stopped requests no repaint at all, the one combination in which the
/// console's own scheduling asks for nothing
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
    // (`egui-0.36.2/src/context.rs:128-137`); several quiet passes let
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

    console.reduced_motion = true;
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
/// Rest must not grow a one-second wake. Cursor Effect is off so its
/// cadence cannot be mistaken for Run Clock honesty. The first passes
/// request an immediate Render Frame (focus, destination auto-select);
/// settle before asserting.
///
#[tokio::test]
async fn a_stopped_console_with_cursor_effect_off_requests_no_timed_wake() {
    let (ctx, mut console, mut host) = fresh_console();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));

    console.reduced_motion = true;
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
        Vec2::new(400.0, 200.0),
        Vec2::new(200.0, 400.0),
        Vec2::new(300.0, 300.0),
    ] {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
        let mut orcvs = running_orcvs(8, 8);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let half_cell = Vec2::splat(viewport.cell_size / 2.0);

        assert_eq!(
            viewport.cell_size, CELL_SIZE,
            "a {screen_size:?} console opened at a Cell size other than the Source's own"
        );
        assert_eq!(
            viewport.rect.min,
            screen.min + Vec2::splat(MARGIN),
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
    let mut view = SourceView::default();

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(
        viewport.cell_size, CELL_SIZE,
        "the default console opened at a scale other than one"
    );
    assert_eq!(
        view.zoom, 1.0,
        "the default console opened at a Zoom other than 1.0"
    );
    assert_eq!(
        viewport.rect.min,
        screen.min + Vec2::splat(MARGIN),
        "the default console did not rest the Grid one margin in"
    );
    assert!(
        viewport.rect.max.x > screen.max.x && viewport.rect.max.y > screen.max.y,
        "the Grid does not run past the default console, so there is nowhere to Pan"
    );
}

///
/// The default window size holds back exactly the height the top bar and
/// the bottom Panel take, so the rest reaches the Source. A whole console
/// pass lays the panels out, so a menu, a control or a Readout that makes
/// either bar taller than its minimum, or a frame that eats into the
/// Source's area, fails here.
///
#[tokio::test]
async fn the_bars_take_the_height_the_default_window_holds_back_and_the_source_the_rest() {
    let (ctx, mut console, mut host) = fresh_console();
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

#[tokio::test]
async fn source_bounds_are_available_before_the_first_render() {
    let orcvs = running_orcvs(32, 16);
    let source_grid = orcvs.render_frame().grid();
    let bounds = source_bounds(source_grid);

    assert_eq!(
        bounds,
        Rect::from_min_size(Pos2::ZERO, Vec2::new(CELL_SIZE * 32.0, CELL_SIZE * 16.0))
    );
}

///
/// The viewport `show_source_scene` presents a Grid of this shape at, in a
/// console of this size, before any gesture has moved the view.
///
/// The same transform that function builds at Zoom 1.0 with the Source at
/// the console's top-left, so nothing about the geometry is restated here:
/// the presented Cell side is `presented_grid`'s, asserted in
/// `grid_viewport.rs`.
///
fn presented(screen: Rect, columns: usize, rows: usize, pixels_per_point: f32) -> GridViewport {
    let grid = Grid::with_shape(columns, rows);
    let source = source_bounds(grid);

    let side = crate::grid_viewport::snapped_cell_side(CELL_SIZE, pixels_per_point);
    presented_grid(
        TSTransform::new(
            screen.min.to_vec2() + Vec2::splat(SOURCE_MARGIN_CELLS * side),
            1.0,
        ),
        source,
        grid,
        pixels_per_point,
    )
}

///
/// A Paint of everything `viewport` shows of `frame` inside `clip`.
///
/// The range is `GridViewport::visible_positions`' own answer, which is
/// what `show_source` hands `Paint::derive`. A test that fits the whole
/// Grid on screen therefore gets a Paint of the whole Grid without having
/// to say so, and one that zooms gets exactly what the console would draw.
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
/// borders, the Cursor's stroke, background runs and sector seams. Colour
/// *decisions* stay in `paint.rs`; Glyph placement needs
/// [`source_shapes`].
///
fn source_geometry(paint: &Paint, viewport: GridViewport, pixels_per_point: f32) -> SourceShapes {
    source_geometry_themed(paint, viewport, pixels_per_point, &okabe_ito())
}

/// [`source_geometry`], but at a `theme` the caller states rather than
/// the built-in default — for a test that asks about the resolved
/// Theme's own Grid/Sector Seam widths instead of `okabe_ito`'s.
fn source_geometry_themed(
    paint: &Paint,
    viewport: GridViewport,
    pixels_per_point: f32,
    theme: &Theme,
) -> SourceShapes {
    SourceShapes::geometry(paint, &viewport, pixels_per_point, theme)
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
    let theme = okabe_ito();
    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let table = GlyphTable::lay_out(
            ui.ctx(),
            egui::FontId::new(
                DEFAULT_FONT_SIZE * glyph_scale(viewport.cell_scale()),
                egui::FontFamily::Monospace,
            ),
        );
        shapes = Some(SourceShapes::new(
            paint,
            &viewport,
            &table,
            pixels_per_point,
            crate::cursor_effects::CursorEffectShapes::default(),
            &theme,
        ));
    });
    output.drop_without_applying_deltas();

    shapes.expect("the pass drew the Source")
}

///
/// [`source_shapes`] with a Cursor Effect built by the caller and a
/// resolved `theme`, for a test that asks how the effect's frame and the
/// Paint's own Cursor stroke compose.
///
fn source_shapes_with_effect(
    paint: &Paint,
    viewport: GridViewport,
    pixels_per_point: f32,
    effect: crate::cursor_effects::CursorEffectShapes,
    theme: &Theme,
) -> SourceShapes {
    let ctx = egui::Context::default();
    let mut shapes = None;
    // `run_ui` takes an `FnMut`; the one pass it runs takes the effect.
    let mut effect = Some(effect);
    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let table = GlyphTable::lay_out(
            ui.ctx(),
            egui::FontId::new(
                DEFAULT_FONT_SIZE * glyph_scale(viewport.cell_scale()),
                egui::FontFamily::Monospace,
            ),
        );
        shapes = Some(SourceShapes::new(
            paint,
            &viewport,
            &table,
            pixels_per_point,
            effect.take().expect("run_ui ran one pass"),
            theme,
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
/// The Cursor Effect reaches what a Cell is painted *with* and never
/// where it is painted.
///
/// This is the property the retired `cell_line_width` test held over a
/// shipped function that took the blink phase and ignored it. Under the
/// painter the property is structural —
/// `GridViewport::cell_rect` takes a Position and nothing else — so it is
/// asserted here against the geometry that actually reached the Shapes.
///
/// The Cursor's visibility is owned by `orcvs`; a console test does not
/// need a wall-clock seam to assert geometry. What is asserted is the
/// whole of what the presentation could move — every Cell,
/// the Cursor's included, occupies exactly the rectangle its Position gives
/// it, and the Cursor's own stroke is drawn on that same rectangle rather
/// than beside it or around it.
///
/// The Cell's rectangle is the one it is *stroked* at: a Cell is filled
/// only where its background differs from the Source, and the Cells that
/// are filled share their rectangles with their neighbours.
///
#[tokio::test]
async fn the_cursor_reaches_the_paint_of_a_cell_and_never_its_geometry() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let orcvs = running_orcvs(8, 8);
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 8, 8, 1.0);
    let paint = painted(&frame, viewport, screen);
    let shapes = source_geometry(&paint, viewport, 1.0);

    assert_eq!(paint.cursor(), Some(frame.cursor()));
    // Every Cell but the Cursor's is stroked with its own border, in row
    // order.
    let mut expected = Vec::new();
    for row in 0..8 {
        for column in 0..8 {
            if (column, row) != (0, 0) {
                expected.push(viewport.cell_rect(column, row));
            }
        }
    }
    assert_eq!(
        shapes.borders.len(),
        expected.len(),
        "stroked {} Cell borders",
        shapes.borders.len()
    );
    for (index, rect) in expected.iter().enumerate() {
        assert!(
            close(rect_of(&shapes.borders[index]), *rect),
            "Cell {index} was stroked at {:?} rather than {rect:?}",
            rect_of(&shapes.borders[index])
        );
    }
    // And the Cursor's Cell is stroked once, by the Cursor, on that same
    // rectangle.
    assert_eq!(shapes.cursor.len(), 1, "the Cursor is one stroke");
    assert!(
        close(rect_of(&shapes.cursor[0]), viewport.cell_rect(0, 0)),
        "the Cursor was stroked at {:?} rather than {:?}",
        rect_of(&shapes.cursor[0]),
        viewport.cell_rect(0, 0)
    );
    // Every background lies on the Cell geometry too, rather than beside
    // it: a run starts and ends on a Cell edge.
    for background in &shapes.backgrounds {
        assert!(
            (rect_of(background).height() - viewport.cell_size).abs() < 1e-3,
            "a background was {} tall against a Cell of {}",
            rect_of(background).height(),
            viewport.cell_size
        );
    }
}

///
/// A Region larger than one Cell hides the Cursor's own Cell border: the
/// lasso around the Region is the Cursor's presentation then, so the
/// Cursor's Cell is stroked as every other Cell is.
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
    let viewport = presented(screen, 8, 8, 1.0);
    let shapes = source_geometry(&painted(&frame, viewport, screen), viewport, 1.0);

    assert!(
        shapes.cursor.is_empty(),
        "the Cursor's Cell kept its border"
    );
    assert_eq!(shapes.borders.len(), 64, "every Cell keeps its grid line");
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
/// belongs to, and the Cursor is painted over both.
///
/// The groups are what makes that expressible. Every fill is in
/// `backgrounds`, every Glyph in `glyphs` and the Cursor alone in `cursor`,
/// so chaining the groups orders the *kinds* however the Cells interleave:
/// a later Cell in the row order cannot erase an earlier Cell's Glyph, and
/// no neighbour's fill or seam can reach the Cursor. That the shape groups
/// then arrive at the painter in that order is asserted by
/// `the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them`.
///
#[tokio::test]
async fn every_background_is_painted_before_every_glyph_and_the_cursor_after_both() {
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
    assert_eq!(shapes.cursor.len(), 1, "the Cursor is one stroke");
    for stroked in shapes.borders.iter().chain(&shapes.cursor) {
        let Shape::Rect(rect) = stroked else {
            panic!("a border was {stroked:?}")
        };
        assert!(
            rect.fill.a() == 0 && rect.stroke.width > 0.0,
            "a border carried a fill: {rect:?}"
        );
    }
    assert_eq!(
        shapes.backgrounds.len(),
        1,
        "only the hidden-caret selection should fill a Cell"
    );
}

///
/// The shape groups reach the painter end to end, in the order
/// `SourceShapes::into_shapes` chains them: backgrounds, borders, Glyphs,
/// seams, the Cursor.
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
/// A Cell border and the Cursor are both stroked rectangles and nothing
/// about the Shape tells them apart; what parts them is that the Cursor is
/// painted after everything else, which is what this asserts.
///
/// The Grid is 16 Cells square because the default Sector Seam spacing is
/// eight: an 8x8 Grid asks for no sector seam at all, and an empty group
/// would let the claims either side of it hold vacuously.
///
#[tokio::test]
async fn the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(16, 16);
    let mut view = SourceView::default();
    // A written Cell, so a Glyph is painted at all.
    orcvs.write("1");
    orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));

    let (_viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

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
    let strokes = at(|shape| matches!(shape, Shape::Rect(rect) if rect.stroke.width > 0.0));
    let glyphs = at(|shape| matches!(shape, Shape::Text(_)));
    let seams = at(|shape| matches!(shape, Shape::LineSegment { .. }));

    assert!(!fills.is_empty(), "the pass painted no fill");
    assert!(strokes.len() > 1, "the pass painted no Cell border");
    assert!(!glyphs.is_empty(), "the pass painted no Glyph");
    assert!(!seams.is_empty(), "the pass painted no sector seam");

    let borders = &strokes;

    assert!(
        *borders.last().expect("a border") < glyphs[0],
        "a border at {:?} painted over the Glyph at {}",
        borders.last(),
        glyphs[0]
    );
    assert!(
        *glyphs.last().expect("a Glyph") < *seams.last().expect("a frame fragment"),
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
        table = Some(GlyphTable::lay_out(
            ui.ctx(),
            egui::FontId::monospace(orcvs::opts::DEFAULT_FONT_SIZE),
        ));
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
        table = Some(GlyphTable::lay_out(
            ui.ctx(),
            egui::FontId::monospace(orcvs::opts::DEFAULT_FONT_SIZE),
        ));
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
/// Every Cell is stroked with its own border, and the Cursor Effect
/// changes that colour, never the width its own Cell is stroked at.
///
/// "That width" is no longer one constant to compare every Cell against:
/// `.scratch/theming/schema.md`'s Source composition step 5 gives a
/// single-Cell Cursor its own `cell.selection.border.width`, independent
/// of the ordinary `grid.border.width` every other Cell — including the
/// Cursor's Cell inside a multi-Cell Region — is stroked at. Okabe–Ito
/// happens to default both to 0.5 points, which is what lets this test
/// still compare every stroke, cursor included, against one
/// `grid_border_width` local;
/// `the_grid_and_selection_border_widths_vary_independently` and
/// `zero_width_suppresses_only_its_own_border_stroke` below are what
/// actually prove the widths are two independent Theme fields rather
/// than one shared constant.
///
/// The colours come from the Paint rather than from `cell_visuals`: which
/// colour a Cell's border *is* is decided in the value layer and asserted
/// there against `cell_visuals` itself. What is asserted here is that the
/// shape step gives each Cell the border the Paint gave that Cell, and
/// not its neighbour's.
///
/// The Grid is wider than the Cursor effect, so the borders
/// are not all one colour and a step that handed every Cell the same
/// stroke would be caught.
///
#[tokio::test]
async fn a_cell_border_is_one_grid_line_wide_whatever_the_cell_is_doing() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(320.0, 320.0));
    let orcvs = running_orcvs(20, 20);
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 20, 20, 1.0);
    let paint = painted(&frame, viewport, screen);
    let shapes = source_geometry(&paint, viewport, 1.0);
    // Fixed at the resolved Theme's `grid.border.width`, in display
    // points — `.scratch/theming/issues/06` slice C — rather than scaled
    // by the owned transform the way the Scene's layer transform used to.
    // Okabe–Ito's `cell.selection.border.width` is also 0.5, so this
    // single local still covers the Cursor's own Cell; see the doc above.
    let grid_border_width = okabe_ito().grid_border_width.points();
    let mut colours = std::collections::BTreeSet::new();

    for (position, cell) in paint.cells() {
        let rect = viewport.cell_rect(position.x(), position.y());
        let stroked = shapes
            .borders
            .iter()
            .chain(&shapes.cursor)
            .find_map(|shape| match shape {
                Shape::Rect(painted) if close(painted.rect, rect) => Some(painted.stroke),
                _ => None,
            })
            .unwrap_or_else(|| panic!("Cell {position:?} was never stroked"));

        assert!(
            (stroked.width - grid_border_width).abs() < 1e-3,
            "the border at {position:?} was {} points wide",
            stroked.width
        );
        assert_eq!(stroked.color, cell.border, "the border at {position:?}");
        colours.insert(stroked.color.to_array());
    }

    assert!(
        colours.len() > 1,
        "every Cell was stroked the same colour, so nothing was told apart"
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
/// `syntax-highlighting/03` retired the blank spelling table that used to
/// stand a placeholder letter in those Cells, so the Cells showing
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
/// `.scratch/theming/issues/06`: "Test transparent and partial-alpha Grid
/// compositing without changing other background properties."
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
        let mut view = SourceView::default();

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
            let spanned = run.columns.len() as f32 * viewport.cell_size;
            assert!(
                (painted.rect.width() - spanned).abs() < viewport.cell_size / 2.0,
                "the run over columns {:?} of row {} is {} wide, not the {spanned} its Cells span",
                run.columns,
                run.row,
                painted.rect.width()
            );
            assert!(
                (painted.rect.height() - viewport.cell_size).abs() < viewport.cell_size / 2.0,
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
/// literal. That is the point: the assertion this replaced re-ran
/// `SourceShapes::new`'s own `Rect::from_min_max(cell_rect(start).min,
/// cell_rect(end - 1).max)`, which catches a one-sided edit and passes a
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
        cell_size: 16.0,
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
/// fixed display-point widths — unchanged by the zoom it presented the
/// Source at — and snaps its background runs to the device scale it ran
/// on.
///
/// The device scale is `show_source`'s own arithmetic — the `Ui`'s — and
/// is handed to `SourceShapes::new` and reaches the Shapes nowhere else.
/// The Grid/Sector Seam *widths* are `.scratch/theming/issues/06` slice
/// C's fixed points, read from `theme` and never multiplied by the
/// presented Cell side over the Source's own: at Zoom 0.5 this is the
/// test that would have caught the old `GRID_LINE_WIDTH * scale`/
/// `SECTOR_LINE_WIDTH * scale` behaviour reappearing, since at Zoom 1 the
/// two are indistinguishable. Every other Shape assertion here builds a
/// `SourceShapes` through `source_geometry` or `source_shapes`, which are
/// given a device scale the test chose, so all of them still hold with
/// that argument replaced by a constant one at the call site. What would
/// ship then is a Grid whose lines and sector seams stay the Theme's own
/// width at every zoom, and runs snapped to whole points on a screen
/// whose pixels are not whole points.
///
/// The geometry is chosen so neither the zoom nor the device scale can be
/// mistaken for the other. A 161 point console over a 20 Cell Grid at
/// Zoom 0.5, and at a device scale of 1.5 the presented Grid's corner is
/// floored a physical pixel in — two thirds of a point — so every run
/// edge is snapped somewhere a snap to whole points would not put it.
///
#[tokio::test]
async fn a_console_pass_strokes_at_its_own_theme_width_and_snaps_runs_to_the_device_scale() {
    const DEVICE_SCALE: f32 = 1.5;
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(161.0));
    let mut orcvs = running_orcvs(20, 20);
    let mut view = SourceView::default();
    pinned_at(&mut view, Vec2::ZERO, 0.5);
    let theme = okabe_ito();

    let (viewport, shapes) = console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        DEVICE_SCALE,
    );
    let scale = viewport.cell_scale();

    assert!(
        (scale - 0.5).abs() < 1e-6,
        "the pass fitted the Source at {scale}, and a zoom of one would be \
         indistinguishable from the constant"
    );

    // Every Cell is stroked once — the Cursor's by the Cursor — and every
    // one of those strokes carries the Theme's fixed width, not the zoom.
    let mut stroked = 0;
    for shape in &shapes {
        if let Shape::Rect(painted) = shape
            && painted.stroke.width > 0.0
        {
            assert!(
                (painted.stroke.width - theme.grid_border_width.points()).abs() < 1e-6,
                "a Cell border was stroked {} points wide against the Theme's fixed {}",
                painted.stroke.width,
                theme.grid_border_width.points()
            );
            stroked += 1;
        }
    }
    assert_eq!(stroked, 399, "the Cursor Cell uses the effect frame");

    // And so does every sector seam, which takes its own fixed width.
    let mut seams = 0;
    for shape in &shapes {
        if let Shape::LineSegment { stroke, .. } = shape
            && (stroke.width - theme.sector_seam_width.points()).abs() < 1e-6
        {
            seams += 1;
        }
    }
    assert!(seams > 0, "the pass drew no sector seam");

    // The default theme leaves the cursor-cell colour unset, so the
    // selected Cell uses the panel's source background and contributes no
    // background run to snap. Explicit colours are covered by the Paint
    // seam tests.
    let frame = orcvs.render_frame();
    let paint = Paint::derive_with_theme(
        FramePaint::new(&frame, viewport.visible_positions(screen, frame.grid())),
        &okabe_ito(),
    );
    let runs = paint.background_runs();
    assert!(
        runs.is_empty(),
        "the default cursor colour should be transparent"
    );
}

///
/// A sector seam is drawn on the Cell edge the Paint asks for, in the
/// colour it asks for, one sector line wide.
///
/// Which Cells carry a seam, at what strength, and that the Cursor's own
/// Cell carries none while it is framed on its own, are the Render
/// Frame's and the derive's answers and
/// are asserted in `paint.rs` with no Context at all. The seam's *width*
/// is the resolved Theme's own fixed display-point value
/// (`.scratch/theming/issues/06` slice C), so it is geometry and belongs
/// here.
///
/// The Grid is 16 Cells square because the default Sector Seam spacing is
/// eight: an 8x8 Grid has no column or row that is a non-zero multiple of
/// it, so every seam strength in one is `None` and this would assert
/// nothing.
///
#[tokio::test]
async fn a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(16, 16);
    // The Cursor goes on a Cell that would otherwise carry both seams, so
    // the suppression the derive applies is visible as an absence here too.
    orcvs.select(orcvs.grid().position(8, 8).expect("inside the grid"));

    let frame = orcvs.render_frame();
    let viewport = presented(screen, 16, 16, 1.0);
    let paint = painted(&frame, viewport, screen);
    let shapes = source_geometry(&paint, viewport, 1.0);
    let sector_seam_width = okabe_ito().sector_seam_width.points();

    let mut expected = Vec::new();
    for (position, cell) in paint.cells() {
        let rect = viewport.cell_rect(position.x(), position.y());

        for (colour, ends) in [
            (cell.sector_left, [rect.left_top(), rect.left_bottom()]),
            (cell.sector_top, [rect.left_top(), rect.right_top()]),
        ] {
            if let Some(colour) = colour {
                expected.push((position, colour, ends));
            }
        }
    }

    assert!(
        !expected.is_empty(),
        "the Paint asked for no sector seam, so nothing was asserted"
    );
    assert_eq!(
        shapes.seams.len(),
        expected.len(),
        "drew {} seams against {} the Paint asked for",
        shapes.seams.len(),
        expected.len()
    );
    for (shape, (position, colour, ends)) in shapes.seams.iter().zip(&expected) {
        let Shape::LineSegment { points, stroke } = shape else {
            panic!("the seam at {position:?} was {shape:?}")
        };

        assert!(
            (points[0] - ends[0]).length() < 1e-3 && (points[1] - ends[1]).length() < 1e-3,
            "the seam at {position:?} ran {points:?} rather than {ends:?}"
        );
        assert_eq!(stroke.color, *colour, "the seam at {position:?}");
        assert!(
            (stroke.width - sector_seam_width).abs() < 1e-3,
            "the seam at {position:?} was {} points wide",
            stroke.width
        );
    }
}

///
/// Width zero hides the Cell grid line and the Sector Seam outright — no
/// zero-width `Shape` left for the painter to drop — at several Grid
/// zoom levels spanning `MIN_ZOOM` to `MAX_ZOOM`.
/// `.scratch/theming/issues/06`: "Width 0 hides the stroke: emit no
/// shape rather than a zero-width one."
///
#[tokio::test]
async fn zero_grid_and_sector_widths_hide_every_border_and_seam_at_every_zoom() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(600.0));
    let mut orcvs = running_orcvs(16, 16);
    // A sector corner, so the fixture would otherwise draw both a grid
    // line and a seam on plenty of Cells.
    orcvs.select(orcvs.grid().position(8, 8).expect("inside the grid"));
    let frame = orcvs.render_frame();
    // Both border widths zeroed: `grid_border_width` for every ordinary
    // Cell and `cell_selection_border_width` for the single-Cell Cursor
    // selected below, which are independent Theme fields since
    // `.scratch/theming/schema.md`'s Source composition step 5 gives the
    // Cursor its own width.
    let theme = Theme {
        grid_border_width: crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1"),
        cell_selection_border_width: crate::theme::GridWidth::from_points(0.0)
            .expect("0.0 is within 0..=1"),
        sector_seam_width: crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1"),
        ..okabe_ito()
    };

    for zoom in [MIN_ZOOM, 1.0, MAX_ZOOM] {
        let cell_size = CELL_SIZE * zoom;
        let viewport = GridViewport {
            cell_size,
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(cell_size * 16.0)),
        };
        let paint = painted_themed(&frame, viewport, screen, &theme);
        let shapes = source_geometry_themed(&paint, viewport, 1.0, &theme);

        assert!(
            shapes.borders.is_empty(),
            "zoom {zoom}: a grid border stroke survived width 0"
        );
        assert!(
            shapes.cursor.is_empty(),
            "zoom {zoom}: the Cursor's own border stroke survived width 0"
        );
        assert!(
            shapes.seams.is_empty(),
            "zoom {zoom}: a sector seam stroke survived width 0"
        );
    }
}

///
/// `grid.border.width` and `cell.selection.border.width` are two
/// independent Theme fields, not one constant read twice — Okabe–Ito
/// happens to default both to 0.5, which is what every other width test
/// in this module reads through one local. Here they are given different
/// values: every ordinary Cell's border takes `grid.border.width`, and
/// only the single-Cell Cursor's own border takes
/// `cell.selection.border.width`, per `.scratch/theming/schema.md`'s
/// Source composition step 5.
///
#[tokio::test]
async fn the_grid_and_selection_border_widths_vary_independently() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(320.0));
    let mut orcvs = running_orcvs(20, 20);
    let cursor = orcvs.grid().position(4, 4).expect("inside the grid");
    orcvs.select(cursor);
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 20, 20, 1.0);
    let theme = Theme {
        grid_border_width: crate::theme::GridWidth::from_points(0.2).expect("0.2 is within 0..=1"),
        cell_selection_border_width: crate::theme::GridWidth::from_points(0.9)
            .expect("0.9 is within 0..=1"),
        ..okabe_ito()
    };
    // Both `paint` and `shapes` resolve against the same `theme`: the
    // border width is baked into `CellPaint` at `Paint::derive_with_theme`,
    // not read again by `SourceShapes::geometry`.
    let paint = painted_themed(&frame, viewport, screen, &theme);
    let shapes = source_geometry_themed(&paint, viewport, 1.0, &theme);

    assert!(
        !shapes.borders.is_empty(),
        "the fixture drew no ordinary Cell border"
    );
    for shape in &shapes.borders {
        let Shape::Rect(stroked) = shape else {
            panic!("a border was {shape:?}")
        };
        assert!(
            (stroked.stroke.width - 0.2).abs() < 1e-4,
            "an ordinary border was {} points wide, not grid.border.width",
            stroked.stroke.width
        );
    }

    assert_eq!(
        shapes.cursor.len(),
        1,
        "the single-Cell Cursor was not framed once"
    );
    let Shape::Rect(cursor_stroke) = &shapes.cursor[0] else {
        panic!("the Cursor's border was {:?}", shapes.cursor[0])
    };
    assert!(
        (cursor_stroke.stroke.width - 0.9).abs() < 1e-4,
        "the Cursor's own border was {} points wide, not cell.selection.border.width",
        cursor_stroke.stroke.width
    );
}

///
/// Each border width hides only its own stroke at zero: a zeroed
/// `cell.selection.border.width` silences the single-Cell Cursor's own
/// border while every ordinary Cell keeps its nonzero `grid.border.width`
/// stroke, and a zeroed `grid.border.width` silences every ordinary Cell
/// while the Cursor keeps its own nonzero stroke. Neither zero reaches
/// the other Cell's border, which
/// `zero_grid_and_sector_widths_hide_every_border_and_seam_at_every_zoom`
/// does not show on its own since it zeroes both together.
///
#[tokio::test]
async fn zero_width_suppresses_only_its_own_border_stroke() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(320.0));
    let mut orcvs = running_orcvs(20, 20);
    let cursor = orcvs.grid().position(4, 4).expect("inside the grid");
    orcvs.select(cursor);
    let frame = orcvs.render_frame();
    let viewport = presented(screen, 20, 20, 1.0);

    let zero_selection = Theme {
        cell_selection_border_width: crate::theme::GridWidth::from_points(0.0)
            .expect("0.0 is within 0..=1"),
        ..okabe_ito()
    };
    let paint = painted_themed(&frame, viewport, screen, &zero_selection);
    let shapes = source_geometry_themed(&paint, viewport, 1.0, &zero_selection);
    assert!(
        shapes.cursor.is_empty(),
        "zero cell.selection.border.width left the Cursor's own border standing"
    );
    assert!(
        !shapes.borders.is_empty(),
        "zero cell.selection.border.width also silenced the ordinary Grid border"
    );

    let zero_grid = Theme {
        grid_border_width: crate::theme::GridWidth::from_points(0.0).expect("0.0 is within 0..=1"),
        ..okabe_ito()
    };
    let paint = painted_themed(&frame, viewport, screen, &zero_grid);
    let shapes = source_geometry_themed(&paint, viewport, 1.0, &zero_grid);
    assert!(
        shapes.borders.is_empty(),
        "zero grid.border.width left an ordinary Cell border standing"
    );
    assert!(
        !shapes.cursor.is_empty(),
        "zero grid.border.width also silenced the Cursor's own border"
    );
}

///
/// A zeroed `cursor.border.width` hides the single-Cell Cursor's frame
/// outright: the Cursor Effect built its frame and built no stroke, which
/// `SourceShapes::new` must not read as "no effect frame" and back-fill
/// with the selected Cell's own `cell.selection.border.width` stroke.
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
    let shapes = source_shapes_with_effect(&paint, viewport, 1.0, effect, &theme);

    assert!(
        shapes.cursor.is_empty(),
        "zero cursor.border.width left {} Cursor strokes standing",
        shapes.cursor.len()
    );
}

///
/// The Cell grid line and the Sector Seam stay the resolved Theme's own
/// fixed display-point widths at every Grid zoom from `MIN_ZOOM` to
/// `MAX_ZOOM` — never multiplied by `GridViewport::cell_scale`, the
/// zoom-scaled behaviour `.scratch/theming/issues/06` slice C replaces.
/// The single selected Cell's own stroke is chained in against
/// `grid_border_width` too: Okabe–Ito's `cell.selection.border.width` is
/// also 0.5, the same coincidence `a_cell_border_is_one_grid_line_wide_
/// whatever_the_cell_is_doing` notes, so this loop still covers it
/// without a second theme field to track across every zoom.
///
#[tokio::test]
async fn grid_and_sector_widths_stay_fixed_display_points_across_zoom_levels() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(600.0));
    let mut orcvs = running_orcvs(16, 16);
    orcvs.select(orcvs.grid().position(8, 8).expect("inside the grid"));
    let frame = orcvs.render_frame();
    let theme = okabe_ito();

    for zoom in [MIN_ZOOM, 0.5, 1.0, 1.5, MAX_ZOOM] {
        let cell_size = CELL_SIZE * zoom;
        let viewport = GridViewport {
            cell_size,
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(cell_size * 16.0)),
        };
        let paint = painted(&frame, viewport, screen);
        let shapes = source_geometry(&paint, viewport, 1.0);

        assert!(
            !shapes.borders.is_empty(),
            "zoom {zoom}: the fixture drew no Cell border"
        );
        for shape in shapes.borders.iter().chain(&shapes.cursor) {
            let Shape::Rect(stroked) = shape else {
                panic!("a border was {shape:?}")
            };
            assert!(
                (stroked.stroke.width - theme.grid_border_width.points()).abs() < 1e-4,
                "zoom {zoom}: a border was {} points wide against the fixed {}",
                stroked.stroke.width,
                theme.grid_border_width.points()
            );
        }

        assert!(
            !shapes.seams.is_empty(),
            "zoom {zoom}: the fixture drew no sector seam"
        );
        for shape in &shapes.seams {
            let Shape::LineSegment { stroke, .. } = shape else {
                panic!("a seam was {shape:?}")
            };
            assert!(
                (stroke.width - theme.sector_seam_width.points()).abs() < 1e-4,
                "zoom {zoom}: a seam was {} points wide against the fixed {}",
                stroke.width,
                theme.sector_seam_width.points()
            );
        }
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
    let mut view = SourceView::default();

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
/// pointer moved, at a scale that is not one — the same claim
/// `a_middle_drag_pans_by_the_pointer_and_a_later_click_selects_the_cell_under_it`
/// makes for the middle button, and the same reason Zoom 2.0 is chosen:
/// a leftover multiply by the Zoom would move the Source by the Zoom
/// times the pointer.
///
#[tokio::test]
async fn alt_held_with_a_primary_drag_pans_by_exactly_what_the_pointer_moved() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();
    pinned_at(&mut view, Vec2::ZERO, 2.0);

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.to_global.scaling, 2.0);
    let anchor = view.to_global.translation;

    let from = screen.min + Vec2::splat(40.0);
    let moved = Vec2::new(-40.0, -24.0);
    console_frame(
        &ctx,
        screen,
        alt_primary_press_at(from),
        &mut orcvs,
        &mut view,
    );
    console_frame(
        &ctx,
        screen,
        vec![Event::PointerMoved(from + moved)],
        &mut orcvs,
        &mut view,
    );

    assert_eq!(
        view.to_global.translation - anchor,
        moved,
        "the Source panned by {:?} for a pointer that moved {moved:?}",
        view.to_global.translation - anchor
    );

    console_frame(
        &ctx,
        screen,
        alt_primary_release_at(from + moved),
        &mut orcvs,
        &mut view,
    );
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
    let mut view = SourceView::default();

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
    let mut view = SourceView::default();

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
    let mut view = SourceView::default();
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
        let mut view = SourceView::default();
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
    let side = CELL_SIZE;

    assert!(near[0] > 0.0, "a drag just past the edge did not scroll");
    assert!(
        far[0] > near[0],
        "a drag further out scrolled no faster: {far:?} against {near:?}"
    );
    assert!(
        far.iter().chain(&near).all(|step| *step <= side),
        "a frame scrolled more than one Cell: {far:?}"
    );
    // Never past the Grid: a Cursor on the last Column brings the margin
    // past it into view, and the console's right edge meets the margin's
    // (ADR 0047).
    assert_eq!(view.pan.x, screen.width() - 32.0 * side - 2.0 * MARGIN);
    assert_eq!(selected_cell(&orcvs), (31, 5));
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
    let mut view = SourceView::default();
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
            before - view.pan.x <= CELL_SIZE,
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
    let mut view = SourceView::default();
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
/// The view a viewer reaches by zooming and panning, set directly.
///
/// The gestures that write these fields are asserted elsewhere. This is a
/// test building its own input below the shipped entry point rather than a
/// seam cut into one: nothing in `show_source` or `show_source_scene`
/// exists for it.
///
fn pinned_at(view: &mut SourceView, pan: Vec2, zoom: f32) {
    view.zoom = zoom;
    view.pan = pan;
}

///
/// The console draws the visible Position range and nothing else, so the
/// cost of a Render Frame follows the viewport rather than the Source.
///
/// Two claims at two layers, and both are here because only a console pass
/// carries the wiring between them. That a zoom reaches fewer Positions and
/// emits fewer Shapes is asserted through the pass itself. That every drawn
/// Position leaves exactly one Cell-sized *stroked* rectangle — its own
/// border, or the Cursor's stroke on the selected Cell — is asserted
/// against the `SourceShapes` groups, counted rather than recovered from
/// the flat list by a fill and a stroke width. Both are compared with
/// `GridViewport::visible_positions`' real answer rather than with a
/// counter the console keeps for a test's benefit.
///
/// The shape total is a bound rather than an equality. Issue 04 coalesces
/// consecutive backgrounds into one rectangle, so a row's shapes are not
/// one per Cell and the two totals are not proportional. What the bound
/// says is the claim this change makes: cost follows the viewport, not the
/// Source. No frame time is asserted — epaint already discards off-screen
/// shapes at tessellation (`coarse_tessellation_culling`), so what this
/// saves is Cell iteration and Shape construction. The Cell iteration is
/// counted in `paint.rs`, which needs no `Context` to count it.
///
#[tokio::test]
async fn the_draw_loop_paints_the_visible_range_rather_than_the_whole_source() {
    let ctx = egui::Context::default();
    // A 128 by 80 Grid at the Source's own Cell size, with its margin on
    // every side, is exactly this console, so the first pass at Zoom 1.0
    // has every Cell on screen. Smaller than the one Grid so the whole of
    // it fits a console this test can afford to paint.
    const COLUMNS: usize = 128;
    const ROWS: usize = 80;
    let screen = Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(COLUMNS as f32 * CELL_SIZE, ROWS as f32 * CELL_SIZE) + Vec2::splat(2.0 * MARGIN),
    );
    let mut orcvs = running_orcvs(COLUMNS, ROWS);
    let mut view = SourceView::default();

    let (whole, every_shape) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        whole.cell_size, CELL_SIZE,
        "the console did not open at Zoom 1.0"
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
        "the console did not show the whole Grid at Zoom 1.0"
    );

    pinned_at(&mut view, Vec2::new(-500.0, -300.0), MAX_ZOOM);
    let (zoomed, fewer_shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let some_positions = zoomed.visible_positions(screen, orcvs.grid());

    assert_eq!(
        zoomed.cell_size,
        CELL_SIZE * MAX_ZOOM,
        "the console did not zoom to {MAX_ZOOM}"
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
    assert!(
        fewer_shapes.len() * 2 < every_shape.len(),
        "{} shapes for {} Positions against {} shapes for {}",
        fewer_shapes.len(),
        some_positions.count(),
        every_shape.len(),
        all_positions.count()
    );

    // One stroked Cell rectangle per drawn Position — the Cell's own
    // border, or the Cursor's stroke on the selected Cell — at both
    // viewports. Counted off the groups the shape step builds rather than
    // recovered from the flat list by their fill and stroke width.
    let frame = orcvs.render_frame();
    for (viewport, positions) in [(whole, all_positions), (zoomed, some_positions)] {
        let shapes = source_shapes(&painted(&frame, viewport, screen), viewport, 1.0);

        assert_eq!(
            shapes.borders.len() + shapes.cursor.len(),
            positions.count(),
            "at a Cell size of {} the shape step drew {} Cells for {} drawn Positions",
            viewport.cell_size,
            shapes.borders.len() + shapes.cursor.len(),
            positions.count()
        );
    }
}

///
/// A zoomed console fills every Cell its Paint asks to fill, and no other,
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
/// The console is small and the zoom is at the limit so both sides of the
/// Source are culled. Cursor effects are geometry beneath the Grid and
/// therefore must not reintroduce Cell background runs in this view.
///
#[tokio::test]
async fn a_zoomed_row_leaves_cursor_effects_out_of_cell_fills() {
    let ctx = egui::Context::default();
    // Narrow enough that every drawn row starts and ends inside the Grid.
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
    let mut orcvs = running_orcvs(COL_COUNT, ROW_COUNT);
    let mut view = SourceView::default();
    // Below the window the console shows, keeping the Cursor near the
    // visible range while its area remains separate geometry.
    orcvs.select(orcvs.grid().position(18, 20).expect("inside the grid"));

    // The Grid's near corner at (-700, -500), so the console shows a window
    // in the middle of it rather than a corner. The Pan places the margin,
    // which is two Cells of 32 points at `MAX_ZOOM`, so it is 64 further.
    pinned_at(&mut view, Vec2::new(-764.0, -564.0), MAX_ZOOM);
    let (viewport, _) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let visible = viewport.visible_positions(screen, orcvs.grid());

    assert!(
        visible.columns.start > 0 && visible.columns.end < COL_COUNT,
        "the zoom culled nothing on one side, so no run begins or ends mid-row"
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
/// This is the acceptance criterion about seams at the edges of the visible
/// range, asserted from the Render Frame at a zoom that culls on all four
/// sides. `sector_seams_are_painted_where_the_render_frame_asks_and_never_on_the_cursor`
/// runs on a default `SourceView` — the fit, with every Position drawn — so
/// it says nothing about a range-limited row.
///
/// Only seams lying *strictly* inside the clip are asserted. A segment on
/// the clip's own edge is what the clip is entitled to discard, and it is
/// the only thing the one-Cell margin reaches: a margin Cell's left edge is
/// the right boundary of what is shown, never inside it. So this test pins
/// the criterion, and the margin is the separate, deliberate over-draw its
/// own comment describes.
///
/// Two pans rather than one, because which seams land strictly inside the
/// clip is a property of the pan. The Sector Seam spacing is 8 and a Cell is 32
/// points at this zoom, so one pan is chosen to put a seam column
/// immediately inside the first drawn column and the other to put one on
/// the last: between them a cull that is short by a Cell on any of the four
/// sides drops a seam this asserts. At a single pan the nearest seam can
/// sit six columns from the edge and a column-side error goes unseen.
///
#[tokio::test]
async fn a_zoomed_console_paints_every_sector_seam_inside_the_clip() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));

    let mut asserted = 0;
    for translation in [Vec2::new(-486.0, -333.0), Vec2::new(-525.0, -333.0)] {
        let mut orcvs = running_orcvs(COL_COUNT, ROW_COUNT);
        let mut view = SourceView::default();

        let frame = orcvs.render_frame();
        pinned_at(&mut view, translation, MAX_ZOOM);
        let (viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let visible = viewport.visible_positions(screen, orcvs.grid());
        let painted: Vec<_> = shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::LineSegment { points, .. } => Some(*points),
                _ => None,
            })
            .collect();

        assert!(
            visible.columns.start > 0
                && visible.columns.end < COL_COUNT
                && visible.rows.start > 0
                && visible.rows.end < ROW_COUNT,
            "the pan {translation:?} culled nothing on one side, so no seam is near a culled edge: {visible:?}"
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

                assert!(
                    painted.iter().any(|points| {
                        (points[0] - ends[0]).length() < 1e-3
                            && (points[1] - ends[1]).length() < 1e-3
                    }),
                    "the seam at {position:?} is inside the clip at pan \
                     {translation:?} and was not painted"
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
/// The console opens at Zoom 1.0 whatever the window size, so a resize
/// shows more or less of the Source and never a different Cell size.
///
#[tokio::test]
async fn a_resize_keeps_the_cell_size_and_shows_more_or_less_of_the_source() {
    let ctx = egui::Context::default();
    let wide = Rect::from_min_size(Pos2::ZERO, WIDE);
    let tall = Rect::from_min_size(Pos2::ZERO, TALL);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();

    let before = console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.zoom, 1.0);
    assert_eq!(before.cell_size, CELL_SIZE);
    let after = console_frame(&ctx, tall, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(view.zoom, 1.0, "a resize changed the Zoom");
    assert_eq!(after.cell_size, CELL_SIZE, "a resize changed the Cell size");
    assert_eq!(
        after.rect.min,
        tall.min + Vec2::splat(MARGIN),
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
#[tokio::test]
async fn a_resize_that_would_open_a_gap_settles_the_source_view_back_inside() {
    let ctx = egui::Context::default();
    let small = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let large = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();

    pinned_at(&mut view, Vec2::new(-200.0, -200.0), 1.0);
    console_frame(&ctx, small, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::new(-200.0, -200.0));

    // 512 points of Grid and a margin either side is 576, so the far edge
    // of a 400 point console is -176.
    console_frame(&ctx, large, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::new(-176.0, -176.0),
        "the resize left a gap past the Grid: {:?}",
        view.pan
    );
}

///
/// Pinch and command-wheel no longer Zoom. Zoom is a change of Cell size
/// from the keyboard alone.
///
#[tokio::test]
async fn pinch_and_command_wheel_do_not_zoom() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let over = viewport.rect.min + Vec2::splat(viewport.cell_size);
    console_frame(&ctx, screen, pinch_at(over), &mut orcvs, &mut view);
    assert_eq!(view.zoom, 1.0, "a pinch changed the Zoom");
    assert_eq!(
        view.to_global.scaling, 1.0,
        "a pinch changed the presented scale"
    );

    console_frame(&ctx, screen, command_wheel_at(over), &mut orcvs, &mut view);
    assert_eq!(view.zoom, 1.0, "a command-wheel changed the Zoom");
    assert_eq!(
        view.to_global.scaling, 1.0,
        "a command-wheel changed the presented scale"
    );
}

///
/// Command `=` and command `+` step the Zoom in; command `-` steps it
/// out; command `0` returns it to 1.0 whatever step it was on.
///
#[tokio::test]
async fn command_chords_step_the_zoom_and_command_zero_resets_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.zoom, 1.0);

    console_frame(
        &ctx,
        screen,
        command_zoom_at(Key::Equals),
        &mut orcvs,
        &mut view,
    );
    assert_eq!(view.zoom, 1.125, "command Equals did not step the Zoom in");

    console_frame(
        &ctx,
        screen,
        command_zoom_at(Key::Plus),
        &mut orcvs,
        &mut view,
    );
    assert_eq!(view.zoom, 1.25, "command Plus did not step the Zoom in");

    console_frame(
        &ctx,
        screen,
        command_zoom_at(Key::Minus),
        &mut orcvs,
        &mut view,
    );
    console_frame(
        &ctx,
        screen,
        command_zoom_at(Key::Minus),
        &mut orcvs,
        &mut view,
    );
    assert_eq!(
        view.zoom, 1.0,
        "two command Minus did not undo two steps in"
    );

    console_frame(
        &ctx,
        screen,
        command_zoom_at(Key::Minus),
        &mut orcvs,
        &mut view,
    );
    assert_eq!(view.zoom, 0.875, "command Minus did not step the Zoom out");

    console_frame(
        &ctx,
        screen,
        command_zoom_at(Key::Num0),
        &mut orcvs,
        &mut view,
    );
    assert_eq!(view.zoom, 1.0, "command Num0 did not reset the Zoom to 1.0");
}

///
/// The Zoom stops exactly at [`MIN_ZOOM`] and [`MAX_ZOOM`] rather than
/// passing them, however many times the chord repeats.
///
#[tokio::test]
async fn command_zoom_stops_at_the_range_limits() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();

    for _ in 0..16 {
        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Equals),
            &mut orcvs,
            &mut view,
        );
    }
    assert_eq!(
        view.zoom, MAX_ZOOM,
        "command Equals passed the Zoom's ceiling"
    );

    for _ in 0..32 {
        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Minus),
            &mut orcvs,
            &mut view,
        );
    }
    assert_eq!(view.zoom, MIN_ZOOM, "command Minus passed the Zoom's floor");
}

///
/// A command Zoom that would open a gap past an edge settles the Source
/// View back inside the Grid, through the same `clamp_pan` a Pan uses.
///
/// The Cursor is moved to the Cell the pinned Pan already shows, at the
/// far corner, so issue 05's follow has nothing to do here: what settles
/// the gap below is `clamp_pan` alone, which is this test's own claim.
///
#[tokio::test]
async fn a_command_zoom_that_would_open_a_gap_settles_the_source_view_back_inside() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();
    orcvs.select(orcvs.grid().position(31, 31).expect("inside the grid"));

    // 32 Cells of 16 points is 512 points; at `MAX_ZOOM` that is 1024, with
    // a 64 point margin either side, and panning fully to the far edge of
    // a 200 point console takes -952.
    pinned_at(&mut view, Vec2::new(-952.0, -952.0), MAX_ZOOM);
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::new(-952.0, -952.0),
        "the fixture did not open already pinned to the far edge"
    );

    console_frame(
        &ctx,
        screen,
        command_zoom_at(Key::Minus),
        &mut orcvs,
        &mut view,
    );
    assert_eq!(view.zoom, 1.875, "command Minus did not step the Zoom out");
    // At 1.875 the source is 960 points and its margins 60 each, so the
    // far edge of a 200 point console is -880: the old -952 Pan now opens
    // a 72 point gap past it.
    assert_eq!(
        view.pan,
        Vec2::new(-880.0, -880.0),
        "the Zoom left a gap past the Grid: {:?}",
        view.pan
    );
}

///
/// A fresh `SourceView`'s first frame does not Pan to the Cursor, however
/// far a fixture puts it from an unpanned top-left origin. Issue 05's
/// follow needs a previous Cursor to compare against, and there is none
/// yet on the very first frame — the same reason a Source reload would
/// not surprise a viewer either.
///
#[tokio::test]
async fn a_fresh_source_view_does_not_pan_to_the_cursor_on_its_first_frame() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();
    // Column 30 at Zoom 1.0 is far outside a 200 point console.
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
    let mut view = SourceView::default();

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

    // Column and row 30 at Zoom 1.0 sit at 512..528 once the margin is
    // counted, entirely past a 200 point console on both axes.
    orcvs.select(orcvs.grid().position(30, 30).expect("inside the grid"));
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(
        view.pan,
        Vec2::new(-328.0, -328.0),
        "the Cursor move did not Pan the least distance that shows it: {:?}",
        view.pan
    );
}

///
/// A Cursor move onto the Grid's last or first Column and Row brings the
/// margin past that edge into view with it, so the Grid's edge shows as
/// an edge rather than flush against the console's (ADR 0047).
///
#[tokio::test]
async fn a_cursor_follow_to_an_edge_cell_shows_the_margin_past_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

    // 32 Cells of 16 points and a 32 point margin either side is 576
    // points, so the far margin's edge meets a 200 point console at -376.
    orcvs.select(orcvs.grid().position(31, 31).expect("inside the grid"));
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::new(-376.0, -376.0),
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
/// A Zoom that would leave the Cursor outside the Source View Pans the
/// least distance that shows it, the same as a Cursor move does.
///
#[tokio::test]
async fn a_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();
    // Already past a 200 point console at Zoom 1.0 (240..256), but the
    // first frame below only records it: see
    // `a_fresh_source_view_does_not_pan_to_the_cursor_on_its_first_frame`.
    orcvs.select(orcvs.grid().position(15, 15).expect("inside the grid"));

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(
        view.pan,
        Vec2::ZERO,
        "the fixture's first frame already Panned"
    );

    console_frame(
        &ctx,
        screen,
        command_zoom_at(Key::Equals),
        &mut orcvs,
        &mut view,
    );

    assert_eq!(view.zoom, 1.125, "command Equals did not step the Zoom in");
    assert_eq!(
        view.pan,
        Vec2::new(-124.0, -124.0),
        "the Zoom did not Pan the least distance that shows the Cursor: {:?}",
        view.pan
    );
}

///
/// A Pan with no Cursor move and no Zoom is not pulled back to the
/// Cursor, even while the Cursor sits outside the Source View.
///
#[tokio::test]
async fn a_pan_with_no_cursor_move_and_no_zoom_is_not_pulled_back_to_the_cursor() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();
    orcvs.select(orcvs.grid().position(30, 30).expect("inside the grid"));

    // First frame: no previous Cursor to compare against, so no follow.
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO);

    // A wheel Pan all the way to the far edge, with the Cursor still
    // unmoved at (30, 30) and no Zoom, repeated until the Pan settles
    // however far `smooth_scroll_delta` hands out a frame. Left to the
    // follow this would land at
    // (-328, -328) instead — see
    // `a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it`
    // — so landing on the margin's far edge at (-376, -376) is what
    // proves a Pan alone is not chasing the Cursor.
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
        Vec2::new(-376.0, -376.0),
        "a Pan alone was pulled back toward the Cursor: {:?}",
        view.pan
    );
}

///
/// The follow Pan is itself naive — it only asks whether the Cursor's
/// Cell already shows inside the console — so an already out-of-bounds
/// Pan it leaves untouched still has to settle back inside the Grid
/// through `clamp_pan`, the same as an ordinary Pan or Zoom does.
///
#[tokio::test]
async fn the_follow_pan_is_still_bounded_by_the_grids_edges() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();

    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

    // A Pan past the Grid's near edge, which nothing but `clamp_pan` can
    // answer: at Zoom 1.0 the Cursor's new Cell (5, 0) sits at 80..96,
    // already inside a 200 point console once this Pan is applied, so the
    // follow itself has nothing to add.
    pinned_at(&mut view, Vec2::new(50.0, 50.0), 1.0);
    orcvs.select(orcvs.grid().position(5, 0).expect("inside the grid"));
    console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

    assert_eq!(
        view.pan,
        Vec2::ZERO,
        "clamp_pan did not settle the follow's own Pan back inside the Grid: {:?}",
        view.pan
    );
}

/// A device scale at which a Zoom step's Cell is not a whole number of
/// physical pixels, and the Zoom step that shows it: 18 points at 1.25 is
/// 22.5 pixels, which `presented_grid` floors to 22 — a Cell of 17.6
/// points rather than the 18 a Zoom of 1.125 asks for.
const FRACTIONAL_PPP: f32 = 1.25;
const FRACTIONAL_ZOOM: f32 = 1.125;

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
    let mut view = SourceView::default();
    pinned_at(&mut view, Vec2::ZERO, FRACTIONAL_ZOOM);

    let (viewport, _) = console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        FRACTIONAL_PPP,
    );

    assert!(
        viewport.cell_size < CELL_SIZE * FRACTIONAL_ZOOM,
        "the fixture snapped nothing, so it asserts nothing: {}",
        viewport.cell_size
    );
    assert_eq!(
        viewport.rect.min,
        screen.min + Vec2::splat(SOURCE_MARGIN_CELLS * viewport.cell_size),
        "a snapped Grid smaller than the console left its margin in from the top-left"
    );
}

///
/// A Pan to the far edge of a Source larger than the console leaves no
/// gap past the Grid where the snap has shrunk its Cells: the bound is
/// the extent the Cells are drawn at, not the one the Zoom asked for.
///
#[tokio::test]
async fn a_far_edge_pan_leaves_no_gap_past_a_snapped_grid() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(64, 64);
    let mut view = SourceView::default();
    pinned_at(&mut view, Vec2::splat(-1_000_000.0), FRACTIONAL_ZOOM);

    let (viewport, _) = console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        FRACTIONAL_PPP,
    );

    assert!(
        viewport.cell_size < CELL_SIZE * FRACTIONAL_ZOOM,
        "the fixture snapped nothing, so it asserts nothing: {}",
        viewport.cell_size
    );
    let gap =
        screen.max - viewport.rect.max - Vec2::splat(SOURCE_MARGIN_CELLS * viewport.cell_size);
    assert!(
        gap.x.abs() <= HALF_A_PIXEL && gap.y.abs() <= HALF_A_PIXEL,
        "the far-edge Pan left {gap:?} past the margin between the Grid and the console's edge"
    );
}

///
/// A Cursor move past the console's far edge Pans the whole Cursor Cell,
/// as drawn, into view — measured at the snapped Cell side the Cells are
/// painted at rather than the unsnapped side the Zoom asked for.
///
#[tokio::test]
async fn a_cursor_follow_shows_the_whole_snapped_cursor_cell() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
    let mut orcvs = running_orcvs(64, 64);
    let mut view = SourceView::default();
    pinned_at(&mut view, Vec2::ZERO, FRACTIONAL_ZOOM);

    console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        FRACTIONAL_PPP,
    );
    // Column and row 22 end at 23 Cells, past a 400 point console at
    // either Cell side.
    orcvs.select(orcvs.grid().position(22, 22).expect("inside the grid"));
    let (viewport, _) = console_pass_at(
        &ctx,
        screen,
        Vec::new(),
        &mut orcvs,
        &mut view,
        FRACTIONAL_PPP,
    );

    let cell = viewport.cell_rect(22, 22);
    assert!(
        cell.min.x >= screen.min.x - HALF_A_PIXEL
            && cell.min.y >= screen.min.y - HALF_A_PIXEL
            && cell.max.x <= screen.max.x + HALF_A_PIXEL
            && cell.max.y <= screen.max.y + HALF_A_PIXEL,
        "the followed Cursor Cell {cell:?} is not wholly inside {screen:?}"
    );
}

///
/// The double click that returned to the fit is gone, along with the fit.
/// A double click inside the Grid still selects the Cell under it.
///
#[tokio::test]
async fn a_double_click_selects_a_cell_and_does_not_reset_the_view() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();
    pinned_at(&mut view, Vec2::new(-48.0, -32.0), 1.0);

    let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    let before = view.to_global;
    let target = viewport.cell_rect(5, 5).center();
    double_click(&ctx, screen, target, &mut orcvs, &mut view);

    assert_eq!(
        selected_cell(&orcvs),
        (5, 5),
        "the double click did not reach the Cell under it"
    );
    assert_eq!(
        view.to_global, before,
        "the double click reset a view that no longer has a fit to return to"
    );
    assert_eq!(view.zoom, 1.0);
    assert_eq!(view.pan, Vec2::new(-48.0, -32.0));
}

///
/// A middle-button drag pans the Source by exactly what the pointer moved,
/// at a scale that is not one, and a later click still selects the Cell
/// under the pointer.
///
/// `Response::drag_delta` divides by the layer transform's scaling *only
/// when the layer has one* (`response.rs:452-465`). With the transform
/// owned by the console there is no layer transform, so a leftover
/// multiply by the Zoom would move the Source by the Zoom times the
/// pointer. Zoom 2.0 is what makes that bug visible.
///
#[tokio::test]
async fn a_middle_drag_pans_by_the_pointer_and_a_later_click_selects_the_cell_under_it() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
    let mut orcvs = running_orcvs(32, 32);
    let mut view = SourceView::default();
    pinned_at(&mut view, Vec2::ZERO, 2.0);

    let before = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
    assert_eq!(view.to_global.scaling, 2.0);
    let anchor = view.to_global.translation;

    // Press, then move further than `max_click_dist` so the gesture
    // resolves as a drag rather than a click. Drag left and up: the Pan
    // is top-left-anchored, so a drag the other way is clamped at zero.
    let from = screen.min + Vec2::splat(40.0);
    let moved = Vec2::new(-40.0, -24.0);
    console_frame(&ctx, screen, middle_press_at(from), &mut orcvs, &mut view);
    let after = console_frame(
        &ctx,
        screen,
        vec![Event::PointerMoved(from + moved)],
        &mut orcvs,
        &mut view,
    );

    assert_eq!(
        view.to_global.translation - anchor,
        moved,
        "the Source panned by {:?} for a pointer that moved {moved:?}",
        view.to_global.translation - anchor
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

    console_frame(
        &ctx,
        screen,
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
    let mut view = SourceView::default();
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
            WIDE.x - CELL_SIZE * 32.0 - 2.0 * MARGIN,
            WIDE.y - CELL_SIZE * 32.0 - 2.0 * MARGIN
        ),
        "the wheel left the Source short of its edge: {:?}",
        view.pan
    );

    let mut small = running_orcvs(8, 8);
    let mut small_view = SourceView::default();
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
/// **The acceptance criterion the whole effort exists for.**
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
/// Zoom is pinned at two rather than left at one on purpose:
/// `Context::set_transform_layer` *removes* the entry for an identity
/// transform, so Zoom 1.0 would let a Scene pass this.
///
#[tokio::test]
async fn no_layer_carrying_the_source_grid_is_transformed() {
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(256.0));
    let mut orcvs = running_orcvs(8, 8);
    orcvs.write("1");
    let mut view = SourceView::default();
    pinned_at(&mut view, Vec2::ZERO, 2.0);
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

    assert_eq!(
        view.to_global.scaling, 2.0,
        "the console did not present at Zoom 2.0"
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
    // Not just that layer: no layer at all. The Scene painted into a
    // sublayer of its own, so checking only the layer the console paints
    // into would miss the transform that used to exist.
    assert!(
        ctx.memory(|memory| memory.to_global.is_empty()),
        "a layer transform survived: {:?}",
        ctx.memory(|memory| memory.to_global.clone())
    );
}
