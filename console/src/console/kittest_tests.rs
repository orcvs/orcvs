//!
//! The console driven the way a viewer drives it: through the AccessKit tree
//! `egui_kittest` reads, and through the pointer and key events the toolkit
//! delivers.
//!
//! # What this module is for that the module above is not
//!
//! `console::tests` already runs `Console` over an `egui::Context` — `app_pass`
//! calls `eframe::App::ui` directly and asserts on the `Shape`s and the
//! `GridViewport` that came back. What it cannot do is *find* a control. Every
//! interaction it performs is a position the test computed, so a menu item that
//! moved, lost its label, or stopped being a widget at all is invisible to it.
//!
//! `egui_kittest` closes exactly that gap and nothing else: it enables AccessKit
//! on the `Context`, runs the same `eframe::App`, and hands back a queryable
//! tree, so `get_by_label("View")` fails when the console stops offering a
//! control by that name. The harness is `Harness::build_eframe`, which calls
//! `App::logic` and `App::ui` with no wrapper of its own
//! (`egui_kittest-0.36.2/src/app_kind.rs:36-44`), so what runs here is the
//! shipped `Console::ui` and not a second UI written for a test.
//!
//! # The two halves of the console, and why the assertions differ across them
//!
//! The chrome is ordinary egui: menu buttons, a checkbox, sliders and a
//! `DragValue`, each of which egui reports to AccessKit with a role and a label.
//! Those are queried semantically.
//!
//! The Source Grid is not. `show_source` allocates *one* `Ui::interact`
//! rectangle over the whole Grid and paints every Cell into it. `Sense::CLICK`
//! rather than `Sense::click()` keeps that rectangle out of the keyboard tab
//! order, which is the input-routing this module relies on; Cells are painted
//! rather than instantiated as widgets, and must not become widgets solely so
//! a test can query them. One `Painter::extend` rather than a `Painter::add`
//! per Cell. So there is no widget per Cell to query and there must not become
//! one: minting a thousand AccessKit nodes to please a test tool would undo
//! the change ADR 0040 and `show_source` were written to make.
//!
//! What the Source Grid offers instead is a geometry contract —
//! `presented_grid` maps the owned transform onto a `GridViewport`,
//! `GridViewport::cell_rect` hands out a Cell's rectangle and
//! `GridViewport::cell_at` inverts it — so every pointer coordinate below is
//! *derived from the live transform at the moment of the click* rather than
//! written down. That is what makes the resize and Pan cases mean anything: a
//! hardcoded coordinate would either keep passing after the mapping broke or
//! start failing for reasons that have nothing to do with it.
//!
//! Deriving it is also the limit of what those cases prove, and the limit is
//! deliberate. `presented_source` makes the same `presented_grid` call
//! `show_source_scene` makes, so what the click tests pin is the *round trip*:
//! that `cell_rect` and `cell_at` remain mutual inverses under a transform the
//! console has moved, and that a click at the coordinate `cell_rect` answers
//! reaches the Source as that Cell. An error inside `presented_grid` itself — a
//! mishandled `pixels_per_point`, a rounded corner off by a Cell — would move
//! both sides of that equality and pass here.
//!
//! Where the Grid actually lands is asserted where nothing cancels, and is not
//! restated here. `console::tests` holds it for a whole console pass:
//! `the_default_window_presents_the_default_grid_at_its_own_scale` pins the
//! presented rectangle and Cell side against written-down values,
//! `a_source_smaller_than_the_console_sits_at_the_top_left_and_holds_no_cell_in_the_surplus`
//! pins the corners of a Source with nowhere to Pan, and
//! `the_presented_viewport_is_the_one_a_console_pass_presents` holds a pass to
//! the helper at a fractional device scale as well as at one.
//! `grid_viewport::tests` holds `presented_grid`'s own whole-physical-pixel
//! snap and its Pan translation. Making the cases below independent of
//! `presented_grid` would mean writing a second copy of it in a test, which is
//! the arrangement those modules already cover better.
//!
//! # Determinism
//!
//! Frame advancement is bounded and explicit. `Harness::run` loops until no
//! immediate repaint is requested, and the console requests one whenever the
//! cursor effect is animating — `CursorEffectSettings::default` has a frequency
//! of 55, so it always is. `step` runs one frame per queued event and
//! `run_steps` runs a stated number, which is what every test here uses. No
//! test sleeps, reads the clock, or depends on Playback: the harness advances
//! `predicted_dt` itself and the Cursor moves only because an event moved it.

use egui::{Color32, CursorIcon, Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::{Harness, kittest::Queryable as _};

use super::{Console, DEFAULT_VIEW_SIZE, MAX_ZOOM, MIN_ZOOM, SOURCE_MARGIN_CELLS, source_bounds};
use crate::cursor_effects::CursorEffectSettings;
use crate::grid_viewport::{CELL_SIZE, GridViewport, presented_grid};
use crate::source_paint::SourcePaintSettings;

///
/// A running `Console` at `size`, built the way eframe builds it.
///
/// `build_eframe` hands the closure eframe's own headless `CreationContext` —
/// the same `_new_kittest` constructor `console::tests` and `storage_tests`
/// reach for — so this is `Console::new` with nothing stubbed. It opens with no
/// storage, which is the fresh-install start.
///
fn running_console(size: Vec2) -> Harness<'static, Console> {
    Harness::builder()
        .with_size(size)
        .with_pixels_per_point(1.0)
        .build_eframe(|cc| Console::new(cc).expect("the test runtime"))
}

///
/// The Cell the Cursor is on, asked of the running Orcvs the console owns.
///
fn cursor(console: &Console) -> (usize, usize) {
    let cursor = console.orcvs.render_frame().cursor();
    (cursor.x(), cursor.y())
}

///
/// What the Cell under the Cursor holds right now, or `None` when it is
/// empty.
///
fn cell_under_cursor(console: &Console) -> Option<char> {
    let frame = console.orcvs.render_frame();
    frame.at(frame.cursor()).content()
}

///
/// Where the console presented its Source Grid on the last frame.
///
/// The same three calls `show_source_scene` makes, asked of the transform the
/// pass left behind rather than re-derived from a console area — so a
/// coordinate built from this is the coordinate the Cells were actually painted
/// at, whatever the window has since done.
///
fn presented_source(harness: &Harness<'_, Console>) -> GridViewport {
    let console = harness.state();
    let grid = console.orcvs.render_frame().grid();

    presented_grid(
        console.source_view.to_global,
        source_bounds(grid),
        grid,
        harness.ctx.pixels_per_point(),
    )
}

///
/// The centre of a Cell as the console is presenting it right now.
///
fn cell_centre(harness: &Harness<'_, Console>, column: usize, row: usize) -> Pos2 {
    presented_source(harness).cell_rect(column, row).center()
}

///
/// A primary click at `point`, delivered as the three events a pointer sends.
///
/// `Harness::step` runs one frame per queued event, so the press and the
/// release land on separate frames — which is what egui reports a click on, and
/// what `console::tests::click` arranges by hand for the same reason.
///
fn click_at(harness: &mut Harness<'_, Console>, point: Pos2) {
    harness.event(Event::PointerMoved(point));
    for pressed in [true, false] {
        harness.event(Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    harness.step();
}

///
/// An ordinary control, found by the name a viewer reads rather than by a
/// position a test computed.
///
/// The View menu holds one checkbox and it owns `Console::diagnostics_open`,
/// which is presentation state: opening the window changes what the console
/// shows and nothing about the Source. Asserting on the field alone would pass
/// against a checkbox bound to it that no menu presents, so the window it opens
/// is asserted too — by one of its own labels, which is a widget only the
/// running Diagnostics window contributes to the tree.
///
#[tokio::test]
async fn the_view_menu_opens_the_diagnostics_window_a_viewer_asked_for() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    assert!(
        !harness.state().diagnostics_open,
        "a fresh console opened with the Diagnostics window already showing"
    );
    assert!(
        harness.query_by_label("Source zoom").is_none(),
        "the Diagnostics window was in the tree before anything opened it"
    );

    harness.get_by_label("View").click();
    harness.step();
    harness.run_steps(1);

    harness.get_by_label("Diagnostics").click();
    harness.step();
    harness.run_steps(2);

    assert!(
        harness.state().diagnostics_open,
        "the View menu's checkbox did not reach the console's own state"
    );
    assert!(
        harness.query_by_label("Source zoom").is_some(),
        "the console holds diagnostics_open but presented no Diagnostics window"
    );
}

///
/// `Theme → Source colours` holds its own "Reset to theme defaults", separate
/// from `Theme → Cursor effects`' — the shape `syntax-highlighting/01` mirrors
/// from Cursor effects, doubled. Both sections carry the same button text, so
/// this finds the Source colours one by its position in the tree rather than
/// by a label unique to it, and proves through the running `Console` — not
/// merely through the two settings values in isolation — that clicking it
/// touches only `source_paint`.
///
#[tokio::test]
async fn the_source_colours_reset_restores_its_defaults_and_leaves_cursor_effects_untouched() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    let mut changed_source_paint = SourcePaintSettings::default();
    *changed_source_paint.ordinary_mut() = Color32::from_rgb(1, 2, 3);
    *changed_source_paint.fill_tint_mut() = 77;
    let mut changed_cursor_effects = CursorEffectSettings::default();
    *changed_cursor_effects.cursor_colour_mut() = Color32::from_rgb(9, 8, 7);
    harness.state_mut().source_paint = changed_source_paint;
    harness.state_mut().cursor_effects = changed_cursor_effects;
    harness.run_steps(1);

    assert_ne!(harness.state().source_paint, SourcePaintSettings::default());

    harness.get_by_label("Theme").click();
    harness.step();
    harness.run_steps(1);

    let resets: Vec<_> = harness
        .get_all_by_label("Reset to theme defaults")
        .collect();
    assert_eq!(
        resets.len(),
        2,
        "expected one reset button for Cursor effects and one for Source colours"
    );
    resets[1].click();
    harness.step();
    harness.run_steps(1);

    assert_eq!(
        harness.state().source_paint,
        SourcePaintSettings::default(),
        "the Source colours reset did not restore its defaults"
    );
    assert_eq!(
        harness.state().cursor_effects,
        changed_cursor_effects,
        "the Source colours reset moved Cursor effects"
    );
}

///
/// The Source's own input path, which is keyboard rather than widget.
///
/// A key press reaches `Context::filtered_events`, survives the console's
/// `EventFilter`, is translated by `translate_event` and handed to
/// `Orcvs::event_handler`. None of that is a widget, so no AccessKit query can
/// reach it; what proves it is the Cursor arriving where the keys asked.
///
/// The Grid rectangle is `Sense::CLICK` and not `Sense::click()` — no focus, no
/// tab stop — so these arrows are handled because nothing else claimed them,
/// which is the arrangement this asserts holds.
///
#[tokio::test]
async fn arrow_keys_move_the_cursor_through_the_source_input_path() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    assert_eq!(
        cursor(harness.state()),
        (0, 0),
        "a fresh console did not open with the Cursor in the corner"
    );

    for _ in 0..3 {
        harness.key_press(egui::Key::ArrowRight);
    }
    harness.key_press(egui::Key::ArrowDown);
    harness.step();
    harness.run_steps(1);

    assert_eq!(
        cursor(harness.state()),
        (3, 1),
        "four arrow presses did not reach the Source"
    );

    // Frames a viewer did not ask for change nothing. The UI function runs
    // again on every one of them, and the Cursor is the Source's state rather
    // than the pass's, so a frame that moved it would mean the presentation had
    // started acting on the Source merely by being drawn.
    harness.run_steps(4);
    assert_eq!(
        cursor(harness.state()),
        (3, 1),
        "quiet frames moved the Cursor"
    );
}

///
/// Issue 05's own criterion, end to end: an ArrowRight run that pushes the
/// Cursor past the console Pans the Source View to bring it back, the same
/// frame the keys reach the Source (`Console::ui` reads the Render Frame
/// after `Orcvs::event_handler` runs).
///
/// This Zooms to `MAX_ZOOM` first, so Column 40 lies past the default
/// window's far edge — command Zoom is keyboard-only and leaves
/// the window and its Panels exactly as they were, so the console's own width
/// is still the default window's, `DEFAULT_VIEW_SIZE[0]`, with none of a
/// resize's uncertainty about how tall the Panels leave the console.
///
#[tokio::test]
async fn arrow_keys_that_move_the_cursor_out_of_view_pan_the_source_view_to_follow_it() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    for _ in 0..8 {
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    }
    harness.step();
    harness.run_steps(1);
    assert_eq!(
        harness.state().source_view.zoom,
        MAX_ZOOM,
        "eight command Equals did not reach MAX_ZOOM"
    );
    assert_eq!(
        harness.state().source_view.pan,
        Vec2::ZERO,
        "Zooming in on an unmoved Cursor already in view Panned regardless"
    );

    // At `MAX_ZOOM` a Cell is `CELL_SIZE * MAX_ZOOM` points, and the default
    // window is `DEFAULT_VIEW_SIZE[0]` points wide whatever the Zoom — so
    // Column 40 sits well past it.
    for _ in 0..40 {
        harness.key_press(egui::Key::ArrowRight);
    }
    harness.step();
    harness.run_steps(1);

    assert_eq!(
        cursor(harness.state()),
        (40, 0),
        "forty ArrowRight presses did not reach the Source"
    );

    // The follow shows Column 40's far edge, which sits the Source View's
    // margin of two Cells further along than the Column itself.
    let cell_at_max_zoom = CELL_SIZE * MAX_ZOOM;
    let console_width = DEFAULT_VIEW_SIZE[0];
    let pan = harness.state().source_view.pan;
    assert_eq!(
        pan,
        Vec2::new(
            console_width - (41.0 + SOURCE_MARGIN_CELLS) * cell_at_max_zoom,
            0.0
        ),
        "the Cursor move did not Pan the least distance that shows Column 40: {pan:?}"
    );

    let column_40 = presented_source(&harness).cell_rect(40, 0);
    assert!(
        column_40.min.x >= 0.0 && column_40.max.x <= console_width,
        "Column 40 is still out of view at {column_40:?} in a console {console_width} points wide"
    );
}

///
/// Keyboard Zoom end to end: a command `=`/`0` chord reaches
/// `Console::ui`'s own event routing exactly like an arrow key does, and
/// changes the Source View rather than the Source. The bare `=` the chord is
/// built from is still Source input, egui's own Cmd `=` UI zoom
/// (`Options::zoom_with_keyboard`, `Console::new` turns it off) never fires
/// for it, and a fresh console opens with `zoom_with_keyboard` already off —
/// otherwise the first chord below would move `harness.ctx.zoom_factor()`
/// too, and pass for the wrong reason.
///
#[tokio::test]
async fn command_zoom_chords_change_the_source_view_and_never_the_source() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    assert_eq!(harness.state().source_view.zoom, 1.0);
    assert_eq!(
        cell_under_cursor(harness.state()),
        None,
        "a fresh console did not open with an empty Cell under the Cursor"
    );

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    harness.step();
    harness.run_steps(1);

    assert_eq!(
        harness.state().source_view.zoom,
        1.125,
        "command Equals did not Zoom the Source View"
    );
    assert_eq!(
        harness.ctx.zoom_factor(),
        1.0,
        "egui's own UI zoom fired for the Source View's Zoom chord"
    );
    assert_eq!(
        cell_under_cursor(harness.state()),
        None,
        "a command Equals chord reached the Source"
    );

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num0);
    harness.step();
    harness.run_steps(1);
    assert_eq!(
        harness.state().source_view.zoom,
        1.0,
        "command Num0 did not reset the Zoom"
    );

    // Bare "=" is still Source input: it types into the Cell under the
    // Cursor, the same path the arrow keys above take.
    let cell = harness.state().orcvs.render_frame().cursor();
    harness.event(Event::Text("=".to_owned()));
    harness.step();
    harness.run_steps(1);

    assert_eq!(
        harness.state().orcvs.render_frame().at(cell).content(),
        Some('='),
        "a bare \"=\" did not reach the Source as Cell input"
    );
    assert_eq!(
        harness.state().source_view.zoom,
        1.0,
        "a bare \"=\" changed the Zoom"
    );
}

/// `File → Load Function reference` is reachable from the menu, and clicking
/// it is an explicit action rather than a no-op: it discards whatever the
/// running Orcvs currently holds and replaces it, Cursor included, with the
/// reference.
///
/// The Cursor is moved away from the origin first — the same move
/// `arrow_keys_move_the_cursor_through_the_source_input_path` proves — so a
/// menu item that changed nothing would leave it standing, and the Grid
/// check below would still read the blank default a fresh console opens on.
///
#[tokio::test]
async fn the_file_menu_loads_the_function_reference_on_demand() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    harness.key_press(egui::Key::ArrowRight);
    harness.key_press(egui::Key::ArrowDown);
    harness.step();
    harness.run_steps(1);
    assert_ne!(
        cursor(harness.state()),
        (0, 0),
        "the arrow presses above did not move the Cursor"
    );

    harness.get_by_label("File").click();
    harness.step();
    harness.run_steps(1);
    harness.get_by_label("Load Function reference").click();
    harness.step();
    harness.run_steps(2);

    assert_eq!(
        cursor(harness.state()),
        (0, 0),
        "loading the Function reference did not reset the Cursor to the Grid origin"
    );
    let grid = harness.state().orcvs.render_frame().grid();
    let reference_grid = crate::function_reference::function_reference().grid();
    assert_eq!(
        (grid.columns(), grid.rows()),
        (reference_grid.columns(), reference_grid.rows()),
        "loading the Function reference did not carry its own Grid"
    );
}

///
/// The pointer-to-Cell round trip after the transform has moved, which is the
/// one thing a fixed coordinate cannot test.
///
/// Two stages, in order: a resize, and a middle-drag Pan. After each, the
/// click target is read back out of the transform the console is presenting
/// under, and the Cell it selects has to be the Cell that coordinate was
/// painted from.
///
/// That is a round trip and not a geometry assertion: the target comes from
/// the same `presented_grid` call `show_source_scene` makes, so this holds
/// `cell_rect` and `cell_at` to inverting each other under a moved transform,
/// and holds a click at `cell_rect`'s answer to reaching the Source as that
/// Cell — the whole input path, from the toolkit's event through the one Grid
/// rectangle and `show_source`'s answered Position to `Console::ui`'s
/// `orcvs.select`. Where `presented_grid` puts the Grid is `console::tests`'
/// and `grid_viewport::tests`' to assert; the module documentation names which
/// tests those are.
///
/// The resize on its own moves nothing to click at: the Source View opens
/// unpanned and anchored at the console's top-left, and a Pan of zero is
/// already inside whatever clamp a smaller console asks for, so Cell (3, 1)
/// stays exactly where it was — only how much of the Grid is on screen has
/// changed. That is asserted rather than assumed, because a round trip
/// through a transform that did not move proves nothing about it. The drag is
/// what actually moves the transform, and the Pan recorded before it guards
/// that stage the same way.
///
/// The selection is also asserted across the resize itself. The Cursor belongs
/// to the Source and the transform belongs to the console, so a resize that
/// moved it would mean a presentation change had reached the Source.
///
#[tokio::test]
async fn a_resized_and_panned_console_still_selects_the_cell_under_the_pointer() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    let opened = cell_centre(&harness, 3, 1);
    click_at(&mut harness, opened);
    assert_eq!(
        cursor(harness.state()),
        (3, 1),
        "a click at {opened:?} on the console the default window opens at"
    );

    // Smaller than the Source on both axes, so the Pan stage below has
    // somewhere to go — the default window is an exact fit at Zoom 1.0 and
    // leaves no room to Pan at all.
    harness.set_size(Vec2::new(320.0, 300.0));
    harness.run_steps(2);

    assert_eq!(
        cursor(harness.state()),
        (3, 1),
        "resizing the console moved the Cursor"
    );

    let after_resize = cell_centre(&harness, 3, 1);
    assert_eq!(
        after_resize, opened,
        "an unpanned Source View moved Cell (3, 1) on a resize alone"
    );
    click_at(&mut harness, after_resize);
    assert_eq!(
        cursor(harness.state()),
        (3, 1),
        "a click at {after_resize:?} on the resized console"
    );

    // A middle-drag Pan over the Grid, the one gesture here that actually
    // moves the owned transform: `show_source_scene` folds the drag into
    // `SourceView::pan`. Every later coordinate has to come back through the
    // moved transform.
    let pan_before = harness.state().source_view.pan;
    let start = cell_centre(&harness, 6, 5);
    let dragged_to = start - Vec2::new(80.0, 60.0);
    harness.event(Event::PointerMoved(start));
    harness.event(Event::PointerButton {
        pos: start,
        button: PointerButton::Middle,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.event(Event::PointerMoved(dragged_to));
    harness.event(Event::PointerButton {
        pos: dragged_to,
        button: PointerButton::Middle,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.step();
    harness.run_steps(1);

    let pan_after = harness.state().source_view.pan;
    assert_ne!(
        pan_after, pan_before,
        "the middle drag left the Source View exactly where it was, so this proves nothing"
    );

    let panned = cell_centre(&harness, 10, 9);
    click_at(&mut harness, panned);
    assert_eq!(
        cursor(harness.state()),
        (10, 9),
        "a click at {panned:?} on a Grid panned to {pan_after:?}"
    );
}

///
/// Issue 06's own criterion, end to end: Alt (Option) held with a primary
/// drag Pans the Source View through the shipped `Console`, moves the Cursor
/// nowhere, and reaches the Source as nothing — the whole input path a
/// trackpad with no middle button takes to Pan by dragging.
///
#[tokio::test]
async fn alt_held_with_a_primary_drag_pans_and_reaches_the_source_as_nothing() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    assert_eq!(
        cursor(harness.state()),
        (0, 0),
        "a fresh console did not open with the Cursor in the corner"
    );

    // Smaller than the Source on both axes, so there is somewhere to Pan —
    // the default window is an exact fit at Zoom 1.0 and leaves no room to
    // Pan at all, the same reason
    // `a_resized_and_panned_console_still_selects_the_cell_under_the_pointer`
    // resizes before its own middle-drag Pan.
    harness.set_size(Vec2::new(320.0, 300.0));
    harness.run_steps(2);

    let pan_before = harness.state().source_view.pan;
    let start = cell_centre(&harness, 6, 5);
    let dragged_to = start - Vec2::new(80.0, 60.0);
    harness.event(Event::PointerMoved(start));
    harness.event(Event::ModifiersChanged(Modifiers::ALT));
    harness.event(Event::PointerButton {
        pos: start,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::ALT,
    });
    harness.event(Event::PointerMoved(dragged_to));
    harness.event(Event::PointerButton {
        pos: dragged_to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::ALT,
    });
    harness.event(Event::ModifiersChanged(Modifiers::default()));
    harness.step();
    harness.run_steps(1);

    let pan_after = harness.state().source_view.pan;
    assert_ne!(
        pan_after, pan_before,
        "an Alt-held primary drag left the Source View exactly where it was, so this proves nothing"
    );
    assert_eq!(
        cursor(harness.state()),
        (0, 0),
        "an Alt-held primary drag moved the Cursor"
    );
    assert_eq!(
        cell_under_cursor(harness.state()),
        None,
        "an Alt-held primary drag reached the Source"
    );
}

///
/// The pointer announces a drag Pan: a grab hand while Alt is held over the
/// console, a grabbing hand while the Alt-held primary drag or a middle-drag
/// is Panning, and the ordinary pointer once neither is (ADR 0047).
///
#[tokio::test]
async fn the_pointer_shows_a_grab_hand_for_alt_and_a_grabbing_hand_while_a_drag_pans() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    let start = cell_centre(&harness, 6, 5);
    let cursor_icon = |harness: &Harness<'_, Console>| harness.output().platform_output.cursor_icon;

    harness.event(Event::PointerMoved(start));
    harness.step();
    assert_eq!(
        cursor_icon(&harness),
        CursorIcon::Default,
        "the pointer changed with no Pan gesture on offer"
    );

    harness.event(Event::ModifiersChanged(Modifiers::ALT));
    harness.step();
    assert_eq!(
        cursor_icon(&harness),
        CursorIcon::Grab,
        "holding Alt over the console did not offer a grab"
    );

    harness.event(Event::PointerButton {
        pos: start,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::ALT,
    });
    harness.event(Event::PointerMoved(start - Vec2::new(80.0, 60.0)));
    harness.step();
    assert_eq!(
        cursor_icon(&harness),
        CursorIcon::Grabbing,
        "an Alt-held primary drag did not show a grabbing hand"
    );

    harness.event(Event::PointerButton {
        pos: start - Vec2::new(80.0, 60.0),
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::ALT,
    });
    harness.event(Event::ModifiersChanged(Modifiers::default()));
    harness.step();
    harness.event(Event::PointerButton {
        pos: start,
        button: PointerButton::Middle,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.event(Event::PointerMoved(start - Vec2::new(40.0, 30.0)));
    harness.step();
    assert_eq!(
        cursor_icon(&harness),
        CursorIcon::Grabbing,
        "a middle-drag did not show a grabbing hand"
    );

    harness.event(Event::PointerButton {
        pos: start - Vec2::new(40.0, 30.0),
        button: PointerButton::Middle,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.step();
    assert_eq!(
        cursor_icon(&harness),
        CursorIcon::Default,
        "the grabbing hand outlived the drag"
    );
}

///
/// The grab hand offers a Pan, so it does not show where Alt starts none: partway
/// through a primary drag that is selecting a Region, which stays a Region drag
/// (ADR 0046), or over a Grid that with its margins fits the console on both
/// axes and has nowhere to Pan (ADR 0047).
///
#[tokio::test]
async fn the_pointer_shows_no_grab_hand_where_alt_offers_no_pan() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    let start = cell_centre(&harness, 6, 5);
    let cursor_icon = |harness: &Harness<'_, Console>| harness.output().platform_output.cursor_icon;

    harness.event(Event::PointerMoved(start));
    harness.event(Event::PointerButton {
        pos: start,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.step();
    harness.event(Event::PointerMoved(start + Vec2::new(60.0, 40.0)));
    harness.step();
    harness.event(Event::ModifiersChanged(Modifiers::ALT));
    harness.event(Event::PointerMoved(start + Vec2::new(64.0, 44.0)));
    harness.step();
    assert_eq!(
        cursor_icon(&harness),
        CursorIcon::Default,
        "Alt pressed partway through a Region drag offered a grab"
    );
    harness.event(Event::PointerButton {
        pos: start + Vec2::new(64.0, 44.0),
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::ALT,
    });
    harness.event(Event::ModifiersChanged(Modifiers::default()));
    harness.step();

    // At `MIN_ZOOM` the default Grid and its margins are smaller than the
    // default window's console on both axes.
    for _ in 0..8 {
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Minus);
    }
    harness.step();
    harness.run_steps(1);
    assert_eq!(
        harness.state().source_view.zoom,
        MIN_ZOOM,
        "eight command Minus did not reach MIN_ZOOM"
    );
    let over = cell_centre(&harness, 2, 2);
    harness.event(Event::PointerMoved(over));
    harness.event(Event::ModifiersChanged(Modifiers::ALT));
    harness.step();
    assert_eq!(
        cursor_icon(&harness),
        CursorIcon::Default,
        "Alt over a Grid with nowhere to Pan offered a grab"
    );
    harness.event(Event::ModifiersChanged(Modifiers::default()));
    harness.step();

    harness.event(Event::PointerButton {
        pos: over,
        button: PointerButton::Middle,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.event(Event::PointerMoved(over + Vec2::new(40.0, 30.0)));
    harness.step();
    assert_eq!(
        cursor_icon(&harness),
        CursorIcon::Default,
        "a middle-drag over a Grid with nowhere to Pan showed a grabbing hand"
    );
    harness.event(Event::PointerButton {
        pos: over + Vec2::new(40.0, 30.0),
        button: PointerButton::Middle,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.step();
}

///
/// Issue 05's own criterion for a click, end to end and under the one
/// condition where nothing *Console-specific* repaints on its own: reduced
/// motion zeroes the Cursor Effect's frequency and this console never starts
/// Playback, so `Console::ui` itself asks for no further frame once the click
/// has been handled.
///
/// A click's Cursor move reaches the Source only once `Console::ui` calls
/// `orcvs.select` after `show_source_scene` returns (see that function's own
/// doc comment), so the follow needs a further frame in which
/// `show_source_scene` reads the moved Cursor back. `Harness::run` is used
/// rather than a fixed `run_steps` count precisely so that frame either runs
/// because something asked for it, or does not run at all — a fixed count
/// would paper over a missing repaint request by supplying the frame anyway.
///
/// It runs regardless: pinned egui 0.36.2's own `InputState::wants_repaint_after`
/// (`egui-0.36.2/src/input_state/mod.rs:655-678`) answers an immediate repaint
/// for any pass whose `RawInput` carries events — which the click's own
/// resolving `PointerButton` release does — and `Context::request_repaint_after`
/// answers that with *two* repaints rather than one, "to give some things
/// time to settle" and "solve some corner-cases of missing repaints on
/// frame-delayed responses" (`egui-0.36.2/src/context.rs:128-137`). That
/// second, free repaint is exactly the frame after a click needs, supplied by
/// the toolkit itself rather than by anything Console asks for — so this
/// holds even with reduced motion on and Playback stopped, the one
/// combination in which Console's own repaint scheduling asks for nothing at
/// all.
///
/// The console is resized to a width that is not a multiple of `CELL_SIZE`,
/// so Column 10 (192..208 at Zoom 1.0, after the two-Cell margin) is cut off
/// at the console's right edge (200) and a click on it needs the follow to
/// bring it fully into view.
///
#[tokio::test]
async fn a_click_still_pans_to_follow_the_cursor_under_reduced_motion_with_playback_stopped() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    harness.state_mut().reduced_motion = true;
    harness.set_size(Vec2::new(200.0, 300.0));
    harness.run_steps(2);

    assert_eq!(
        harness.state().source_view.pan,
        Vec2::ZERO,
        "the resize alone Panned before anything moved the Cursor"
    );

    let target = presented_source(&harness).cell_rect(10, 0).min + Vec2::new(3.0, 3.0);
    harness.event(Event::PointerMoved(target));
    harness.event(Event::PointerButton {
        pos: target,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.event(Event::PointerButton {
        pos: target,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.run();

    assert_eq!(
        cursor(harness.state()),
        (10, 0),
        "the click did not select Column 10"
    );

    let column_10 = presented_source(&harness).cell_rect(10, 0);
    assert!(
        column_10.max.x <= 200.0,
        "the click's Cursor move did not Pan to bring Column 10 fully into a 200 point \
         console: {column_10:?}"
    );
}

///
/// Keyboard input belongs to whichever control holds egui's focus, not only
/// to the two the console named. The Theme menu's "Glitch amount" Slider
/// carries an editable `DragValue`; with its value box focused, command A,
/// Backspace, and a paste are that box's text editing and must reach nothing
/// in the Source.
///
/// Egui states the rule itself: `RawInput::events` has "no way to know if
/// egui handles a particular event, but you can check if egui is using the
/// keyboard with `Context::egui_wants_keyboard_input`"
/// (`egui-0.36.2/src/data/input/raw_input.rs:56-60`).
///
#[tokio::test]
async fn a_focused_theme_menu_value_box_keeps_region_and_clipboard_commands_from_the_source() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    harness.event(Event::Text("x".to_owned()));
    harness.step();
    harness.run_steps(1);
    let origin_content = |console: &Console| {
        let frame = console.orcvs.render_frame();
        frame.at(frame.grid().origin()).content()
    };
    assert_eq!(
        origin_content(harness.state()),
        Some('x'),
        "the setup character never reached the Source"
    );

    harness.get_by_label("Theme").click();
    harness.step();
    harness.run_steps(1);
    // A pointer click on the value box closes the menu it sits in
    // (`PopupCloseBehavior::CloseOnClick`, `egui-0.36.2/src/containers/popup.rs:78-82`),
    // so the viewer who edits it arrives by keyboard: Tab walks egui's focus
    // order into the open menu.
    let value_box_focused = |harness: &Harness<'_, Console>| {
        harness
            .query_all_by_role(egui::accesskit::Role::SpinButton)
            .any(|node| node.is_focused())
    };
    for _ in 0..32 {
        if value_box_focused(&harness) {
            break;
        }
        harness.key_press(Key::Tab);
        harness.step();
    }
    assert!(
        value_box_focused(&harness) && harness.ctx.egui_wants_keyboard_input(),
        "Tab never gave a Theme menu value box keyboard focus"
    );

    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.key_press(Key::Backspace);
    harness.event(Event::Paste("zz".to_owned()));
    harness.step();
    harness.run_steps(1);

    assert_eq!(
        origin_content(harness.state()),
        Some('x'),
        "command A, Backspace, or a paste in a focused value box reached the Source"
    );
    let region = harness.state().orcvs.region();
    assert!(
        region.is_one_cell(),
        "command A in a focused value box selected the whole Source: {region:?}"
    );
    // The paste would land at the Cursor, which the setup character left on
    // the Cell after the origin.
    let frame = harness.state().orcvs.render_frame();
    let after_origin = frame.grid().position(1, 0).expect("inside the Grid");
    assert_eq!(
        frame.at(after_origin).content(),
        None,
        "a paste into a focused value box wrote the Source"
    );
}

///
/// While a menu is open, Tab belongs to egui's own focus navigation alone —
/// the Cursor must not also step a Sector for the same press.
///
/// A menu button's click does not itself take focus (the previous test's own
/// comment explains why the value box is reached by Tab rather than a
/// click), so an open menu has to hold the keys by being open: were it asked
/// only of focus, the Tab-focus cancellation and the event routing could
/// disagree about the one press, and it would both move focus and step the
/// Cursor.
///
#[tokio::test]
async fn tab_with_a_menu_open_moves_focus_and_not_the_cursor() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    harness.get_by_label("Theme").click();
    harness.step();
    harness.run_steps(1);

    let cursor_before = cursor(harness.state());
    harness.key_press(egui::Key::Tab);
    harness.step();
    harness.run_steps(1);

    assert_eq!(
        cursor(harness.state()),
        cursor_before,
        "Tab moved the Cursor while a menu was open"
    );
}

///
/// An open menu holds the keys as a focused control does, though opening one
/// by a click focuses nothing: a character typed while it is open does not
/// write the Source.
///
#[tokio::test]
async fn typing_with_a_menu_open_leaves_the_source_unwritten() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    harness.get_by_label("Theme").click();
    harness.step();
    harness.run_steps(1);
    assert!(
        egui::Popup::is_any_open(&harness.ctx) && !harness.ctx.egui_wants_keyboard_input(),
        "the Theme menu did not open with nothing focused"
    );

    let cursor_before = cursor(harness.state());
    harness.event(Event::Text("x".to_owned()));
    harness.step();
    harness.run_steps(1);

    // A written character would also step the Cursor on, so the Cell it was
    // on is the one to ask.
    assert_eq!(
        cursor(harness.state()),
        cursor_before,
        "a character typed while a menu was open moved the Cursor"
    );
    assert_eq!(
        cell_under_cursor(harness.state()),
        None,
        "a character typed while a menu was open wrote the Source"
    );
}

///
/// Escape with a menu open closes the menu and leaves the Region alone: the
/// open menu holds the keys, so the press is the menu's and not also the
/// Source's collapse.
///
#[tokio::test]
async fn escape_with_a_menu_open_closes_it_and_keeps_the_region() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    harness.key_press_modifiers(Modifiers::SHIFT, egui::Key::ArrowRight);
    harness.step();
    harness.run_steps(1);
    let region_before = harness.state().orcvs.region();
    assert!(!region_before.is_one_cell(), "test setup spanned no Region");

    harness.get_by_label("Theme").click();
    harness.step();
    harness.run_steps(1);
    assert!(
        egui::Popup::is_any_open(&harness.ctx),
        "the Theme menu did not open"
    );

    harness.key_press(egui::Key::Escape);
    harness.step();
    harness.run_steps(1);

    assert!(
        !egui::Popup::is_any_open(&harness.ctx),
        "Escape did not close the open menu"
    );
    assert_eq!(
        harness.state().orcvs.region(),
        region_before,
        "the Escape that closed a menu also collapsed the Region"
    );
}

///
/// Tab and Shift Tab reach the Source through the same input path the arrow
/// keys do: Tab steps the Cursor to the first Cell of the next Sector on its
/// row, Shift Tab steps back, and neither moves anything once the keys stop
/// coming. The default Grid is 64 Cells wide at the default Sector Seam
/// spacing of 8, so column 8 and column 16 are the first two Sector starts
/// past the origin.
///
#[tokio::test]
async fn tab_and_shift_tab_move_the_cursor_by_sector_through_the_source_input_path() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    assert_eq!(
        cursor(harness.state()),
        (0, 0),
        "a fresh console did not open with the Cursor in the corner"
    );

    harness.key_press(egui::Key::Tab);
    harness.step();
    harness.run_steps(1);
    assert_eq!(
        cursor(harness.state()),
        (8, 0),
        "Tab did not step the Cursor to the next Sector"
    );

    harness.key_press(egui::Key::Tab);
    harness.step();
    harness.run_steps(1);
    assert_eq!(
        cursor(harness.state()),
        (16, 0),
        "a second Tab did not step another Sector"
    );

    harness.key_press_modifiers(Modifiers::SHIFT, egui::Key::Tab);
    harness.step();
    harness.run_steps(1);
    assert_eq!(
        cursor(harness.state()),
        (8, 0),
        "Shift Tab did not step back to the Sector it came from"
    );

    // Frames a viewer did not ask for change nothing, the same guarantee
    // `arrow_keys_move_the_cursor_through_the_source_input_path` holds for
    // the arrow keys.
    harness.run_steps(4);
    assert_eq!(
        cursor(harness.state()),
        (8, 0),
        "quiet frames moved the Cursor"
    );
}

///
/// Tab belongs to the Source while it holds the keys: it collapses a
/// multi-Cell Region to the Cursor's stepped Position, focuses no widget in
/// the chrome, and leaves the very next character free to write the Source.
///
/// `tab_never_focuses_the_console_area_the_source_is_shown_in` guards the
/// console area specifically; this guards that Tab, while the Source holds
/// the keys, focuses nothing in the tree at all — not even a menu-bar
/// button, which `docs/adr/0048-the-source-takes-the-keys-no-control-holds.md`
/// names as the pre-fix behaviour.
///
#[tokio::test]
async fn tab_focuses_no_widget_collapses_the_region_and_leaves_typing_open() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    harness.key_press_modifiers(Modifiers::SHIFT, egui::Key::ArrowRight);
    harness.step();
    harness.run_steps(1);
    assert!(
        !harness.state().orcvs.region().is_one_cell(),
        "test setup spanned no Region"
    );

    harness.key_press(egui::Key::Tab);
    harness.step();
    harness.run_steps(1);

    assert!(
        harness.state().orcvs.region().is_one_cell(),
        "Tab did not collapse the Region"
    );
    assert_eq!(
        cursor(harness.state()),
        (8, 0),
        "Tab did not step the collapsed Cursor to the next Sector"
    );
    assert!(
        harness.ctx.memory(|memory| memory.focused()).is_none(),
        "Tab focused a widget while the Source held the keys"
    );

    let written_at = harness.state().orcvs.render_frame().cursor();
    harness.event(Event::Text("x".to_owned()));
    harness.step();
    harness.run_steps(1);

    assert_eq!(
        harness
            .state()
            .orcvs
            .render_frame()
            .at(written_at)
            .content(),
        Some('x'),
        "the character typed right after Tab did not reach the Source"
    );
}

///
/// Tab walks egui's focus order through the chrome and never onto the console
/// area the Source is shown in. The pan rectangle there senses clicks and
/// drags, and `Sense::click_and_drag()` is `CLICK | FOCUSABLE | DRAG`
/// (`egui-0.36.2/src/sense.rs:81-83`): focused, it would count as a control
/// holding the keyboard, and the Source would get no keys until Escape or a
/// click.
///
#[tokio::test]
async fn tab_never_focuses_the_console_area_the_source_is_shown_in() {
    let size = Vec2::from(DEFAULT_VIEW_SIZE);
    let mut harness = running_console(size);
    harness.run_steps(2);
    let console_centre = Pos2::ZERO + size / 2.0;

    for _ in 0..64 {
        harness.key_press(Key::Tab);
        harness.step();
        let focused = harness
            .ctx
            .memory(|memory| memory.focused())
            .and_then(|id| harness.ctx.read_response(id));
        if let Some(response) = focused {
            assert!(
                !response.rect.contains(console_centre),
                "Tab focused a widget covering the console area: {:?}",
                response.rect
            );
        }
    }
}
