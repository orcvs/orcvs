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
//! (`egui_kittest-0.36.1/src/app_kind.rs:36-44`), so what runs here is the
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
//! `presented_grid` maps the owned transform onto a `GridViewport`, and
//! `GridViewport::cell_at` inverts it — so every pointer coordinate below is
//! *derived from the live transform at the moment of the click* rather than
//! written down. That is what makes the resize and zoom cases mean anything: a
//! hardcoded coordinate would either keep passing after the mapping broke or
//! start failing for reasons that have nothing to do with it.
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

use egui::{Event, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::{Harness, kittest::Queryable as _};

use super::{Console, DEFAULT_VIEW_SIZE, source_bounds};
use crate::grid_viewport::{GridViewport, presented_grid};

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
/// The pointer-to-Cell mapping after the transform has moved, which is the one
/// thing a fixed coordinate cannot test.
///
/// Three transforms, in order: the fit the default window opens on, the fit a
/// resize re-derives, and a zoom the viewer pinned. After each, the click
/// target is read back out of the transform the console is presenting under,
/// and the Cell it selects has to be the Cell that coordinate was painted from.
///
/// The selection is also asserted across the resize itself. The Cursor belongs
/// to the Source and the transform belongs to the console, so a resize that
/// moved it would mean a presentation change had reached the Source.
///
#[tokio::test]
async fn a_resized_and_zoomed_console_still_selects_the_cell_under_the_pointer() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    let fitted = cell_centre(&harness, 3, 1);
    click_at(&mut harness, fitted);
    assert_eq!(
        cursor(harness.state()),
        (3, 1),
        "a click at {fitted:?} on the fitted Grid"
    );

    // Smaller and a different shape, so the re-fit changes both the scale and
    // the letterboxing. The view has not been pinned, so the console re-fits
    // rather than cropping.
    harness.set_size(Vec2::new(640.0, 480.0));
    harness.run_steps(2);

    assert_eq!(
        cursor(harness.state()),
        (3, 1),
        "resizing the console moved the Cursor"
    );

    // Same Cell as `fitted`. Comparing (7, 5) after a resize against (3, 1)
    // before it is true under one transform, so it cannot prove the re-fit.
    let after_resize = cell_centre(&harness, 3, 1);
    assert_ne!(
        after_resize, fitted,
        "the resize left Cell (3, 1) exactly where it was, so this proves nothing"
    );
    let resized = cell_centre(&harness, 7, 5);
    click_at(&mut harness, resized);
    assert_eq!(
        cursor(harness.state()),
        (7, 5),
        "a click at {resized:?} on the re-fitted Grid"
    );

    // A pinch over the Grid, which pins the view: `register_pan_and_zoom` moves
    // the owned transform and `show_source_scene` records that the viewer
    // adjusted it. Every later coordinate has to come back through the moved
    // transform.
    let scale_before_zoom = harness.state().source_view.to_global.scaling;
    let over = cell_centre(&harness, 7, 5);
    harness.event(Event::PointerMoved(over));
    harness.event(Event::Zoom(1.5));
    harness.step();
    harness.run_steps(1);

    let scaling = harness.state().source_view.to_global.scaling;
    assert_ne!(
        scaling, scale_before_zoom,
        "the zoom left the Grid at the same scale"
    );
    assert!(
        harness.state().source_view.adjusted,
        "the zoom did not pin the view"
    );

    let zoomed = cell_centre(&harness, 9, 6);
    click_at(&mut harness, zoomed);
    assert_eq!(
        cursor(harness.state()),
        (9, 6),
        "a click at {zoomed:?} on a Grid presented at {scaling}x"
    );
}
