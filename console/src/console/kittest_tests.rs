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

use egui::{CursorIcon, Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use egui_kittest::{
    Harness,
    kittest::{NodeT as _, Queryable as _},
};
use orcvs::grid::{COL_COUNT, ROW_COUNT};
use orcvs::playback::PlaybackState;

use super::{Console, DEFAULT_VIEW_SIZE, MAX_ZOOM, MIN_ZOOM, SOURCE_MARGIN_CELLS, source_bounds};
use crate::grid_viewport::{CELL_SIZE, GridViewport, presented_grid};
use crate::theme::{Appearance, okabe_ito, orcvs_light};
use crate::theme_registry::ThemeRegistry;
use crate::theme_registry::tests_support::{id, my_dark, my_light, with_my_themes};

///
/// A running `Console` at `size`, built the way eframe builds it.
///
/// `build_eframe` hands the closure eframe's own headless `CreationContext` —
/// the same `_new_kittest` constructor `console::tests` and `storage_tests`
/// reach for — so this is `Console::new` with nothing stubbed but the Theme
/// registry, which holds the built-ins alone rather than whatever the
/// machine's own `~/.orcvs/themes` holds. It opens with no storage, which is
/// the fresh-install start.
///
fn running_console(size: Vec2) -> Harness<'static, Console> {
    running_console_with(size, ThemeRegistry::built_in())
}

///
/// [`running_console`] over the Theme registry a test built.
///
fn running_console_with(size: Vec2, themes: ThemeRegistry) -> Harness<'static, Console> {
    Harness::builder()
        .with_size(size)
        .with_pixels_per_point(1.0)
        .build_eframe(move |cc| Console::new(cc, themes).expect("the test runtime"))
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
/// ADR 0053: Themes choose colours, so no menu offers a "Reset to theme
/// defaults" button — there are no per-Theme overrides to reset.
///
#[tokio::test]
async fn no_menu_offers_a_reset_to_theme_defaults_button() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    for menu in ["File", "View", "Settings"] {
        harness.get_by_label(menu).click();
        harness.step();
        harness.run_steps(1);
        assert_eq!(
            harness
                .query_all_by_label("Reset to theme defaults")
                .count(),
            0,
            "the {menu} menu offered a Reset to theme defaults button"
        );
        harness.key_press(egui::Key::Escape);
        harness.step();
        harness.run_steps(1);
    }
}

///
/// Glitch amount and Glitch frequency are offered by the Settings menu, and
/// nowhere else: neither is in the tree before Settings is opened.
///
#[tokio::test]
async fn glitch_controls_are_offered_only_by_the_settings_menu() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);

    assert_eq!(
        harness.query_all_by_label("Glitch amount").count(),
        0,
        "Glitch amount was in the tree before any menu was opened"
    );
    assert_eq!(
        harness.query_all_by_label("Glitch frequency").count(),
        0,
        "Glitch frequency was in the tree before any menu was opened"
    );

    harness.get_by_label("Settings").click();
    harness.step();
    harness.run_steps(1);
    // Both the Slider and the `DragValue` it wraps carry the same
    // accessible label, so `query_all_by_label` (as the reset-button test
    // above already uses) rather than `query_by_label`, which requires
    // exactly one match.
    assert!(
        harness.query_all_by_label("Glitch amount").count() > 0,
        "the Settings menu did not offer Glitch amount"
    );
    assert!(
        harness.query_all_by_label("Glitch frequency").count() > 0,
        "the Settings menu did not offer Glitch frequency"
    );
}

// === Theme switching (`.scratch/theming/issues/04`) ===

///
/// Every filled rectangle the last frame painted, on every layer.
///
fn painted_fills(harness: &Harness<'_, Console>) -> Vec<egui::Color32> {
    fn collect(shape: &egui::Shape, fills: &mut Vec<egui::Color32>) {
        match shape {
            egui::Shape::Rect(rect) => fills.push(rect.fill),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, fills);
                }
            }
            _ => {}
        }
    }

    let mut fills = Vec::new();
    for clipped in &harness.output().shapes {
        collect(&clipped.shape, &mut fills);
    }
    fills
}

///
/// That the last frame painted `expected` throughout, and none of `others`:
/// the Source Grid's ground is `expected`'s `grid.background` and the
/// chrome's panels are its `panel.background`. One frame's shapes carry both
/// halves, so a frame that painted the Source from one Theme and the chrome
/// from another fails here.
///
fn assert_frame_paints(
    harness: &Harness<'_, Console>,
    expected: &crate::theme::Theme,
    others: &[crate::theme::Theme],
    context: &str,
) {
    let fills = painted_fills(harness);
    assert!(
        fills.contains(&expected.grid_background),
        "{context}: the Source Grid was not painted on {}'s grid.background",
        expected.identity
    );
    assert!(
        fills.contains(&expected.panel_background),
        "{context}: the chrome was not painted on {}'s panel.background",
        expected.identity
    );
    for other in others {
        for (key, colour) in [
            ("grid.background", other.grid_background),
            ("panel.background", other.panel_background),
        ] {
            assert!(
                !fills.contains(&colour),
                "{context}: the frame also painted {}'s {key}",
                other.identity
            );
        }
    }
}

///
/// [`assert_frame_paints`], and that nothing is waiting to change it: egui's
/// active style is `style(expected)` and the window backdrop eframe clears
/// to is `expected`'s `window.background`.
///
fn assert_frame_presents(
    harness: &Harness<'_, Console>,
    expected: &crate::theme::Theme,
    others: &[crate::theme::Theme],
    context: &str,
) {
    assert_frame_paints(harness, expected, others, context);
    let style = harness.ctx.global_style();
    let mut expected_visuals = crate::style::style(expected).visuals;
    // The harness turns caret blinking off so its frames are deterministic;
    // that one field is the harness's, not the Theme's.
    expected_visuals.text_cursor.blink = style.visuals.text_cursor.blink;
    assert_eq!(
        style.visuals, expected_visuals,
        "{context}: egui's active style is not {}'s",
        expected.identity
    );
    assert_eq!(
        eframe::App::clear_color(harness.state(), &style.visuals),
        expected.window_background.to_normalized_gamma_f32(),
        "{context}: the window backdrop is not {}'s",
        expected.identity
    );
}

///
/// A running console over `themes` whose OS appearance is `system`, with every frame's
/// input saying so the way the platform integration reports it, and whose
/// mode follows the OS.
///
/// The harness holds `ThemePreference::Dark` by default; this returns it to
/// `System`, egui's own default and what a fresh install starts from.
///
fn console_under_os_appearance(
    system: egui::Theme,
    themes: ThemeRegistry,
) -> Harness<'static, Console> {
    let mut harness = running_console_with(Vec2::from(DEFAULT_VIEW_SIZE), themes);
    harness.ctx.set_theme(egui::ThemePreference::System);
    harness.input_mut().system_theme = Some(system);
    harness.run_steps(2);
    harness
}

///
/// Opens the View menu and clicks the control labelled `label` in it, then
/// runs the frame the click lands in and the frame that presents it.
///
fn choose_in_view_menu(harness: &mut Harness<'_, Console>, label: &str) {
    harness.get_by_label("View").click();
    harness.step();
    harness.run_steps(1);
    harness.get_by_label(label).click();
    harness.step();
    harness.run_steps(1);
}

///
/// Whether the View menu's radio button labelled `label` is shown selected.
///
fn radio_selected(harness: &Harness<'_, Console>, label: &str) -> bool {
    harness.get_by_label(label).accesskit_node().toggled() == Some(egui::accesskit::Toggled::True)
}

///
/// `.scratch/theming/issues/04`: the View menu holds the mode — follow the
/// OS, Dark, Light — beside a dark Theme picker and a light Theme picker,
/// and each picker lists only Themes of its appearance — a loaded Theme only
/// in its parent's (`.scratch/theming/issues/07`). `my-dark` and `my-light`
/// are loaded so each picker has two to list.
///
/// Where a Theme is listed is read from the menu's own layout: each picker
/// is a heading followed by its Themes, so a Theme belongs to the picker
/// whose heading is the nearest one above it.
///
#[tokio::test]
async fn the_view_menu_offers_the_mode_beside_a_theme_picker_per_appearance() {
    let mut harness = running_console_with(Vec2::from(DEFAULT_VIEW_SIZE), with_my_themes());
    harness.run_steps(2);

    for label in ["Follow the OS", "Dark Theme", "Okabe–Ito", "Orcvs Light"] {
        assert!(
            harness.query_by_label(label).is_none(),
            "{label:?} was in the tree before the View menu was opened"
        );
    }

    harness.get_by_label("View").click();
    harness.step();
    harness.run_steps(1);

    for label in ["Follow the OS", "Dark", "Light"] {
        assert!(
            harness.query_by_label(label).is_some(),
            "the View menu does not offer the {label:?} mode"
        );
    }
    let top = |label: &str| harness.get_by_label(label).rect().min.y;
    let dark_heading = top("Dark Theme");
    let light_heading = top("Light Theme");
    assert!(dark_heading < light_heading);
    for dark in ["Okabe–Ito", "My Dark"] {
        let at = top(dark);
        assert!(
            dark_heading < at && at < light_heading,
            "{dark:?} is not listed in the dark Theme picker"
        );
    }
    for light in ["Orcvs Light", "My Light"] {
        assert!(
            top(light) > light_heading,
            "{light:?} is not listed in the light Theme picker"
        );
    }
    for listed in ["Okabe–Ito", "My Dark", "Orcvs Light", "My Light"] {
        assert_eq!(
            harness.query_all_by_label(listed).count(),
            1,
            "{listed:?} is listed by more than its own appearance's picker"
        );
    }
}

///
/// The mode control, under an OS in dark appearance: holding Light presents
/// Orcvs Light whatever the OS says, and Follow the OS returns to the OS's
/// appearance. The frame a choice is made in is still wholly the old Theme;
/// the next is wholly the new one. The mode is egui's `ThemePreference`,
/// which eframe persists with egui memory.
///
#[tokio::test]
async fn the_mode_switches_source_and_chrome_together() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, ThemeRegistry::built_in());
    assert_frame_presents(
        &harness,
        &okabe_ito(),
        &[orcvs_light()],
        "following a dark OS",
    );

    choose_in_view_menu(&mut harness, "Light");
    assert_eq!(
        harness.ctx.options(|options| options.theme_preference),
        egui::ThemePreference::Light
    );
    harness.run_steps(1);
    assert_frame_presents(&harness, &orcvs_light(), &[okabe_ito()], "holding Light");

    choose_in_view_menu(&mut harness, "Follow the OS");
    assert_eq!(
        harness.ctx.options(|options| options.theme_preference),
        egui::ThemePreference::System
    );
    harness.run_steps(1);
    assert_frame_presents(
        &harness,
        &okabe_ito(),
        &[orcvs_light()],
        "following a dark OS again",
    );
}

///
/// The frame a mode is chosen in is presented in the appearance it began
/// in — chrome and Source both — and the change reaches the next frame.
///
/// The Diagnostics window is open because it is chrome egui styles when it
/// is shown, after the menu bar: a change applied the moment it was clicked
/// would style it from the new Theme within the old frame.
///
#[tokio::test]
async fn the_frame_a_mode_is_chosen_in_keeps_one_theme() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, ThemeRegistry::built_in());
    harness.state_mut().diagnostics_open = true;
    harness.run_steps(1);
    harness.get_by_label("View").click();
    harness.step();
    harness.run_steps(1);

    harness.get_by_label("Light").click();
    harness.step();
    assert_eq!(
        harness.ctx.options(|options| options.theme_preference),
        egui::ThemePreference::Light,
        "the click did not reach the mode"
    );
    assert_frame_paints(
        &harness,
        &okabe_ito(),
        &[orcvs_light()],
        "the frame Light was chosen in",
    );

    harness.run_steps(1);
    assert_frame_presents(&harness, &orcvs_light(), &[okabe_ito()], "the frame after");
}

///
/// The backdrop the web clears to agrees with the frame it sits under.
/// eframe's web runner asks `clear_color` after the frame
/// (`eframe-0.36.2/src/web/app_runner.rs`, `paint` after `logic`), so on the
/// frame a mode is chosen in — still wholly the old Theme — the backdrop is
/// the old Theme's too, and the next frame's is the new one's. A translucent
/// loaded Theme would otherwise show the other Theme's backdrop through it.
///
#[tokio::test]
async fn the_web_backdrop_is_the_theme_its_frame_was_painted_from() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, ThemeRegistry::built_in());
    harness.get_by_label("View").click();
    harness.step();
    harness.run_steps(1);

    harness.get_by_label("Light").click();
    harness.step();
    assert_frame_paints(
        &harness,
        &okabe_ito(),
        &[orcvs_light()],
        "the frame Light was chosen in",
    );
    assert_eq!(
        harness.state().web_clear_color(),
        okabe_ito().window_background.to_normalized_gamma_f32(),
        "the web backdrop under the frame Light was chosen in is not Okabe–Ito's"
    );

    harness.run_steps(1);
    assert_frame_paints(&harness, &orcvs_light(), &[okabe_ito()], "the frame after");
    assert_eq!(
        harness.state().web_clear_color(),
        orcvs_light().window_background.to_normalized_gamma_f32(),
        "the web backdrop under the frame after is not Orcvs Light's"
    );
}

///
/// An operating-system appearance change, arriving as
/// `RawInput::system_theme` the way egui's integrations report it, switches
/// Source and chrome together while the mode follows the OS — and changes
/// nothing while the mode holds one appearance.
///
#[tokio::test]
async fn an_os_appearance_change_switches_source_and_chrome_together() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, ThemeRegistry::built_in());
    assert_frame_presents(&harness, &okabe_ito(), &[orcvs_light()], "a dark OS");

    harness.input_mut().system_theme = Some(egui::Theme::Light);
    harness.run_steps(1);
    assert_frame_presents(&harness, &orcvs_light(), &[okabe_ito()], "a light OS");

    harness.input_mut().system_theme = Some(egui::Theme::Dark);
    harness.run_steps(1);
    assert_frame_presents(&harness, &okabe_ito(), &[orcvs_light()], "a dark OS again");

    choose_in_view_menu(&mut harness, "Dark");
    harness.input_mut().system_theme = Some(egui::Theme::Light);
    harness.run_steps(2);
    assert_frame_presents(
        &harness,
        &okabe_ito(),
        &[orcvs_light()],
        "holding Dark under a light OS",
    );
}

///
/// A dark Theme picker change restyles the chrome and repaints the Source
/// from the picked Theme on the next frame, records the selection for the
/// next save, and leaves the light appearance's Theme alone.
///
#[tokio::test]
async fn picking_a_dark_theme_restyles_source_and_chrome() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, with_my_themes());
    harness.run_steps(1);
    assert_frame_presents(&harness, &okabe_ito(), &[my_dark()], "before picking");

    choose_in_view_menu(&mut harness, "My Dark");
    harness.run_steps(1);
    assert_frame_presents(
        &harness,
        &my_dark(),
        &[okabe_ito(), orcvs_light(), my_light()],
        "after picking My Dark",
    );
    let themes = &harness.state().themes;
    assert_eq!(
        themes.selection().identity(Appearance::Dark).as_str(),
        "my-dark"
    );
    assert_eq!(
        themes.selection().identity(Appearance::Light).as_str(),
        "orcvs-light"
    );
    assert_eq!(
        harness.ctx.style_of(egui::Theme::Light).visuals,
        crate::style::style(&orcvs_light()).visuals,
        "picking a dark Theme restyled the light appearance"
    );

    harness.input_mut().system_theme = Some(egui::Theme::Light);
    harness.run_steps(1);
    assert_frame_presents(
        &harness,
        &orcvs_light(),
        &[my_dark(), okabe_ito()],
        "a light OS after picking My Dark",
    );
}

///
/// The light Theme picker, under a light OS: picking a light Theme restyles
/// the chrome and repaints the Source from it on the next frame, records the
/// selection for the next save, and leaves the dark appearance's Theme alone.
///
#[tokio::test]
async fn picking_a_light_theme_restyles_source_and_chrome() {
    let mut harness = console_under_os_appearance(egui::Theme::Light, with_my_themes());
    harness.run_steps(1);
    assert_frame_presents(&harness, &orcvs_light(), &[my_light()], "before picking");

    choose_in_view_menu(&mut harness, "My Light");
    harness.run_steps(1);
    assert_frame_presents(
        &harness,
        &my_light(),
        &[orcvs_light(), okabe_ito(), my_dark()],
        "after picking My Light",
    );
    let themes = &harness.state().themes;
    assert_eq!(
        themes.selection().identity(Appearance::Light).as_str(),
        "my-light"
    );
    assert_eq!(
        themes.selection().identity(Appearance::Dark).as_str(),
        "okabe-ito"
    );
    assert_eq!(
        harness.ctx.style_of(egui::Theme::Dark).visuals,
        crate::style::style(&okabe_ito()).visuals,
        "picking a light Theme restyled the dark appearance"
    );

    harness.input_mut().system_theme = Some(egui::Theme::Dark);
    harness.run_steps(1);
    assert_frame_presents(
        &harness,
        &okabe_ito(),
        &[my_light(), orcvs_light()],
        "a dark OS after picking My Light",
    );
}

///
/// The frame a Theme is picked in is presented wholly in the Theme it began
/// in — chrome and Source both — and the picked Theme reaches the next
/// frame. The Diagnostics window is open for the reason
/// `the_frame_a_mode_is_chosen_in_keeps_one_theme` gives: it is chrome egui
/// styles after the menu bar, so a pick applied on click would style it from
/// the new Theme within the old frame.
///
#[tokio::test]
async fn the_frame_a_theme_is_picked_in_keeps_one_theme() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, with_my_themes());
    harness.state_mut().diagnostics_open = true;
    harness.run_steps(1);
    harness.get_by_label("View").click();
    harness.step();
    harness.run_steps(1);

    harness.get_by_label("My Dark").click();
    harness.step();
    assert_eq!(
        harness
            .state()
            .themes
            .selection()
            .identity(Appearance::Dark)
            .as_str(),
        "my-dark",
        "the click did not reach the dark Theme selection"
    );
    assert_frame_paints(
        &harness,
        &okabe_ito(),
        &[my_dark()],
        "the frame My Dark was picked in",
    );

    harness.run_steps(1);
    assert_frame_presents(&harness, &my_dark(), &[okabe_ito()], "the frame after");
}

///
/// A restored dark selection no available Theme answers to presents
/// Okabe–Ito, and the dark picker shows Okabe–Ito as selected, because that
/// is what is presented. Picking it is then the viewer's own choice, and
/// replaces the kept selection — `.scratch/theming/issues/07`'s rule is that
/// the *fallback* never rewrites a selection, and this is not the fallback.
///
#[tokio::test]
async fn picking_the_fallback_theme_replaces_a_kept_selection() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, ThemeRegistry::built_in());
    // Selecting an identity nothing answers to keeps it, as a restored one
    // is kept, and presents the fallback.
    harness
        .state_mut()
        .themes
        .select(Appearance::Dark, &id("my-dark"));
    harness.state().themes.install(&harness.ctx);
    harness.run_steps(1);
    assert_frame_presents(&harness, &okabe_ito(), &[orcvs_light()], "the fallback");

    harness.get_by_label("View").click();
    harness.step();
    harness.run_steps(1);
    assert!(
        radio_selected(&harness, "Okabe–Ito"),
        "the dark picker did not show the presented fallback as selected"
    );
    assert_eq!(
        harness
            .state()
            .themes
            .selection()
            .identity(Appearance::Dark)
            .as_str(),
        "my-dark",
        "opening the menu rewrote the kept selection"
    );

    harness.get_by_label("Okabe–Ito").click();
    harness.step();
    harness.run_steps(1);
    assert_eq!(
        harness
            .state()
            .themes
            .selection()
            .identity(Appearance::Dark)
            .as_str(),
        "okabe-ito",
        "picking the presented Theme did not become the selection"
    );
    assert_frame_presents(&harness, &okabe_ito(), &[orcvs_light()], "after picking");
}

///
/// A Theme file that failed to load is told to the viewer in the top bar, and
/// the notice lists the file's problem until the viewer dismisses it.
///
/// The registry is discovered from a directory this test builds, never from
/// the machine's own `~/.orcvs/themes`: only `Console::start` reads that.
///
#[tokio::test]
async fn a_theme_file_that_failed_to_load_is_shown_until_dismissed() {
    use crate::theme_registry::tests_support::TempDir;

    let dir = TempDir::new();
    dir.write("broken.toml", "format = \"orcvs-theme\"\nversion = 2\n");
    let mut harness = running_console_with(
        Vec2::from(DEFAULT_VIEW_SIZE),
        ThemeRegistry::discover(dir.path()),
    );
    harness.run_steps(2);

    harness.get_by_label("Theme notices (1)").click();
    harness.step();
    harness.run_steps(1);
    assert!(
        harness
            .query_all_by_label_contains("broken.toml")
            .next()
            .is_some(),
        "the notice did not name the refused file"
    );

    harness.get_by_label("Dismiss").click();
    harness.step();
    harness.run_steps(2);
    assert_eq!(harness.state().themes.notice_count(), 0);
    assert!(
        harness.query_by_label("Theme notices (1)").is_none(),
        "the dismissed notice is still in the top bar"
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
/// What every Cell of the running Source holds, in Grid order.
///
fn cells(console: &Console) -> Vec<Option<char>> {
    let frame = console.orcvs.render_frame();
    frame.cells().iter().map(|cell| cell.content()).collect()
}

///
/// Opens the File menu and clicks the item labelled `label` in it, then runs
/// the frame the click lands in and the frames that present it.
///
fn choose_in_file_menu(harness: &mut Harness<'_, Console>, label: &str) {
    harness.get_by_label("File").click();
    harness.step();
    harness.run_steps(1);
    harness.get_by_label(label).click();
    harness.step();
    harness.run_steps(2);
}

///
/// An Open replaces the environment and nothing else. The settings a viewer
/// chose — the Theme, which carries the Source colours (ADR 0053), Cursor
/// effects, and whether Diagnostics is showing — are preferences, and stand
/// across it (`.scratch/file-new/spec.md`).
///
/// Each is moved off its default first, so an Open that reset it would be
/// seen rather than read back as the default it already held.
///
#[tokio::test]
async fn an_open_leaves_every_setting_standing() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, with_my_themes());
    {
        let console = harness.state_mut();
        console.themes.select(Appearance::Dark, &id("my-dark"));
        console.themes.select(Appearance::Light, &id("my-light"));
        *console.cursor_effects.amount_mut() = 7;
        *console.cursor_effects.frequency_mut() = 9;
        console.diagnostics_open = true;
    }
    harness.state().themes.install(&harness.ctx);
    harness.run_steps(1);
    let cursor_effects = harness.state().cursor_effects;

    // The Diagnostics window opens over the menu bar's left end, so a pointer
    // click at the File button lands on the window. AccessKit's own click is
    // the one that reaches the button beneath it.
    harness.get_by_label("File").click_accesskit();
    harness.step();
    harness.run_steps(1);
    harness.get_by_label("Load Function reference").click();
    harness.step();
    harness.run_steps(2);

    let console = harness.state();
    assert_eq!(
        cursor(console),
        (0, 0),
        "the Open did not happen, so it proves nothing about what it left"
    );
    assert_eq!(
        (
            console
                .themes
                .selection()
                .identity(Appearance::Dark)
                .as_str(),
            console
                .themes
                .selection()
                .identity(Appearance::Light)
                .as_str(),
        ),
        ("my-dark", "my-light"),
        "an Open changed the Theme selection"
    );
    assert_eq!(
        console.cursor_effects, cursor_effects,
        "an Open changed the Cursor effects"
    );
    assert!(console.diagnostics_open, "an Open closed Diagnostics");
    assert_frame_presents(&harness, &my_dark(), &[okabe_ito()], "after the Open");
}

///
/// An Open whose Orcvs cannot start leaves the console on the Source it
/// already had. A running Orcvs needs a runtime to spawn its Playback Engine
/// on (ADR 0041); the console is built inside one here and the Open is asked
/// for outside it, which is the one way a build of the Orcvs fails.
///
#[test]
fn an_open_that_cannot_start_leaves_the_running_source_standing() {
    let runtime = tokio::runtime::Runtime::new().expect("a Tokio runtime");
    let entered = runtime.enter();
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    harness.event(Event::Text("x".to_owned()));
    harness.step();
    harness.run_steps(1);
    let before = cells(harness.state());
    let cursor_before = cursor(harness.state());
    drop(entered);

    choose_in_file_menu(&mut harness, "Load Function reference");

    assert_eq!(
        cells(harness.state()),
        before,
        "a failed Open replaced the Source"
    );
    assert_eq!(
        cursor(harness.state()),
        cursor_before,
        "a failed Open moved the Cursor"
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

    // A Grid with nowhere to Pan: at `MIN_ZOOM` this one and its margins are
    // smaller than the default window's console on both axes.
    harness.state_mut().orcvs = orcvs::app::Orcvs::with_shape(64, 40).expect("the test runtime");
    harness.state_mut().source_view = super::SourceView::default();
    harness.step();
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
/// to the two the console named. The Settings menu's "Glitch amount" Slider
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
async fn a_focused_settings_menu_value_box_keeps_region_and_clipboard_commands_from_the_source() {
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

    harness.get_by_label("Settings").click();
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
        "Tab never gave a Settings menu value box keyboard focus"
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

    harness.get_by_label("Settings").click();
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

    harness.get_by_label("Settings").click();
    harness.step();
    harness.run_steps(1);
    assert!(
        egui::Popup::is_any_open(&harness.ctx) && !harness.ctx.egui_wants_keyboard_input(),
        "the Settings menu did not open with nothing focused"
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

    harness.get_by_label("Settings").click();
    harness.step();
    harness.run_steps(1);
    assert!(
        egui::Popup::is_any_open(&harness.ctx),
        "the Settings menu did not open"
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
/// coming. At the default Sector Seam spacing of 8, column 8 and column 16
/// are the first two Sector starts past the origin.
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

///
/// Waits, bounded, until `watch` answers `ready`, handing the runtime the
/// turns a Playback Engine's task needs to act on what it was asked. Playback
/// runs on its own task (ADR 0041), so this is the one wait the module
/// allows: on a fact only that task can make true, never on a clock.
///
async fn engine_reaches(
    watch: &mut orcvs::playback::PlaybackObservationWatch,
    ready: impl FnMut(&orcvs::playback::PlaybackObservation) -> bool,
) -> bool {
    tokio::time::timeout(ENGINE_WAIT, watch.wait_for(ready))
        .await
        .is_ok_and(|reached| reached.is_ok())
}

/// How long [`engine_reaches`] and a closing engine are given: far longer than
/// a task on a current-thread runtime needs, so reaching it is a failure and
/// not a slow machine.
const ENGINE_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

///
/// `File → New` replaces the environment with an empty Source on the 256 by
/// 256 Grid. Everything New resets is moved off its rest first — a written Cell,
/// the Cursor, the Zoom, Bpm, and a Tick that is playing — and every setting
/// New leaves standing is moved off its default, so the test can tell a
/// reset from a value that never changed.
///
#[tokio::test]
async fn file_new_opens_an_empty_source_on_the_256_by_256_grid() {
    let mut harness = console_under_os_appearance(egui::Theme::Dark, with_my_themes());
    let default_bpm = harness.state().orcvs.bpm();
    {
        let console = harness.state_mut();
        console.themes.select(Appearance::Dark, &id("my-dark"));
        *console.cursor_effects.amount_mut() = 7;
        console
            .orcvs
            .set_bpm(orcvs::opts::Bpm::new(200).expect("200 is in range"));
    }
    harness.state().themes.install(&harness.ctx);
    let cursor_effects = harness.state().cursor_effects;

    harness.event(Event::Text("x".to_owned()));
    harness.key_press(Key::ArrowDown);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    harness.key_press(Key::Space);
    harness.step();
    harness.run_steps(1);
    let mut replaced = harness.state().orcvs.playback_observation_watch();
    assert!(
        engine_reaches(&mut replaced, |observed| observed.state
            == PlaybackState::Playing)
        .await,
        "Space did not start the Playback New is to stop"
    );
    assert_ne!(cursor(harness.state()), (0, 0), "the Cursor never moved");
    assert_ne!(
        harness.state().source_view.zoom,
        1.0,
        "the Zoom never moved"
    );
    assert!(
        cells(harness.state()).iter().any(Option::is_some),
        "nothing was written"
    );

    choose_new_and_discard(&mut harness);

    let console = harness.state();
    let grid = console.orcvs.render_frame().grid();
    assert_eq!(
        (grid.columns(), grid.rows()),
        (COL_COUNT, ROW_COUNT),
        "New did not open the 256 by 256 Grid"
    );
    assert!(
        cells(console).iter().all(Option::is_none),
        "New left a Cell written"
    );
    assert_eq!(
        cursor(console),
        (0, 0),
        "New left the Cursor off the origin"
    );
    assert_eq!(
        (console.source_view.zoom, console.source_view.pan),
        (1.0, Vec2::ZERO),
        "New left the Source View off its rest"
    );
    assert_eq!(
        console.orcvs.playback_observation().state,
        PlaybackState::Stopped,
        "New left Playback running"
    );
    assert_eq!(console.orcvs.bpm(), default_bpm, "New kept the old Bpm");
    assert_eq!(
        console
            .themes
            .selection()
            .identity(Appearance::Dark)
            .as_str(),
        "my-dark",
        "New changed the Theme selection"
    );
    assert_eq!(
        console.cursor_effects, cursor_effects,
        "New changed the Cursor effects"
    );
    assert_frame_presents(&harness, &my_dark(), &[okabe_ito()], "after New");
    // The replaced engine's task owns the only sender of its observation,
    // so the watch closing is that engine having ended.
    assert!(
        tokio::time::timeout(ENGINE_WAIT, async {
            while replaced.changed().await.is_ok() {}
        })
        .await
        .is_ok(),
        "the replaced Playback Engine outlived New"
    );
}

///
/// A console restored onto a stored Source holding written content reaches an
/// empty Source through `File → New`, and the empty Source is what the next
/// ordinary save stores — so a restart opens it rather than the Source stored
/// before New. New itself stores nothing: until that save, the
/// storage still holds the revision it was restored from.
///
#[cfg(feature = "persistence")]
#[tokio::test]
async fn file_new_is_what_the_next_save_stores_and_a_restart_opens() {
    use crate::persistence::{InMemoryStorage, SOURCE_KEY, edited_source, store};

    let mut stored = InMemoryStorage::default();
    store(&mut stored, &edited_source());
    let stored_revision = eframe::Storage::get_string(&stored, SOURCE_KEY);

    let mut harness = Harness::builder()
        .with_size(Vec2::from(DEFAULT_VIEW_SIZE))
        .with_pixels_per_point(1.0)
        .build_eframe(|cc| {
            cc.storage = Some(&stored);
            Console::new(cc, ThemeRegistry::built_in()).expect("the test runtime")
        });
    harness.run_steps(2);
    assert!(
        cells(harness.state()).iter().any(Option::is_some),
        "the console did not restore the stored, written Source"
    );

    choose_new_and_discard(&mut harness);

    assert_eq!(
        eframe::Storage::get_string(&stored, SOURCE_KEY),
        stored_revision,
        "New wrote storage itself rather than leaving it to the ordinary save"
    );
    let mut saved = InMemoryStorage::default();
    eframe::App::save(harness.state_mut(), &mut saved);
    drop(harness);

    let restarted = Harness::builder()
        .with_size(Vec2::from(DEFAULT_VIEW_SIZE))
        .with_pixels_per_point(1.0)
        .build_eframe(|cc| {
            cc.storage = Some(&saved);
            Console::new(cc, ThemeRegistry::built_in()).expect("the test runtime")
        });
    let grid = restarted.state().orcvs.render_frame().grid();
    assert_eq!(
        (grid.columns(), grid.rows()),
        (COL_COUNT, ROW_COUNT),
        "a restart after New did not open the 256 by 256 Grid"
    );
    assert!(
        cells(restarted.state()).iter().all(Option::is_none),
        "a restart after New opened a written Source"
    );
}

///
/// A console whose Source holds one written Cell, at the origin, with the
/// Cursor moved on past it and the Source View zoomed: an environment New
/// would visibly discard.
///
fn console_with_written_content() -> Harness<'static, Console> {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    harness.event(Event::Text("x".to_owned()));
    harness.key_press(Key::ArrowDown);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    harness.step();
    harness.run_steps(1);
    assert!(
        cells(harness.state()).iter().any(Option::is_some),
        "nothing was written"
    );
    harness
}

///
/// Whether the console is asking before it discards the Source: the
/// confirmation's own control is in the tree.
///
fn asking(harness: &Harness<'_, Console>) -> bool {
    harness.query_by_label(DISCARD).is_some()
}

/// The confirmation's control that goes ahead and discards the Source.
const DISCARD: &str = "Discard";

///
/// Chooses `File → New` on a Source holding written content and confirms the
/// question it asks, as a viewer who means to discard it does.
///
fn choose_new_and_discard(harness: &mut Harness<'_, Console>) {
    choose_in_file_menu(harness, "New");
    assert!(asking(harness), "New discarded written content unasked");
    harness.get_by_label(DISCARD).click();
    harness.step();
    harness.run_steps(2);
}

///
/// New on a Source holding written content asks first, and changes nothing
/// while it asks; confirming then opens an empty Source exactly as
/// New does unasked.
///
#[tokio::test]
async fn file_new_on_written_content_asks_and_confirming_opens_an_empty_source() {
    let mut harness = console_with_written_content();
    let written = cells(harness.state());
    let cursor_before = cursor(harness.state());

    choose_in_file_menu(&mut harness, "New");

    assert!(asking(&harness), "New discarded written content unasked");
    assert_eq!(cells(harness.state()), written, "asking changed the Source");
    assert_eq!(
        cursor(harness.state()),
        cursor_before,
        "asking moved the Cursor"
    );

    harness.get_by_label(DISCARD).click();
    harness.step();
    harness.run_steps(2);

    assert!(!asking(&harness), "confirming left the question showing");
    let console = harness.state();
    let grid = console.orcvs.render_frame().grid();
    assert_eq!(
        (grid.columns(), grid.rows()),
        (COL_COUNT, ROW_COUNT),
        "confirming did not open the 256 by 256 Grid"
    );
    assert!(
        cells(console).iter().all(Option::is_none),
        "confirming left a Cell written"
    );
    assert_eq!(
        cursor(console),
        (0, 0),
        "confirming left the Cursor off the origin"
    );
    assert_eq!(
        (console.source_view.zoom, console.source_view.pan),
        (1.0, Vec2::ZERO),
        "confirming left the Source View off its rest"
    );
}

///
/// Cancelling leaves the environment exactly as it was: the Source, its Grid,
/// the Cursor, the Source View, and Playback — still playing, on the engine
/// it was playing on.
///
#[tokio::test]
async fn file_new_cancelled_leaves_the_environment_as_it_was() {
    let mut harness = console_with_written_content();
    harness.key_press(Key::Space);
    harness.step();
    harness.run_steps(1);
    let mut engine = harness.state().orcvs.playback_observation_watch();
    assert!(
        engine_reaches(&mut engine, |observed| observed.state
            == PlaybackState::Playing)
        .await,
        "Space did not start Playback"
    );
    let written = cells(harness.state());
    let cursor_before = cursor(harness.state());
    let zoom = harness.state().source_view.zoom;

    choose_in_file_menu(&mut harness, "New");
    assert!(asking(&harness), "New discarded written content unasked");
    harness.get_by_label("Cancel").click();
    harness.step();
    harness.run_steps(2);

    assert!(!asking(&harness), "cancelling left the question showing");
    let console = harness.state();
    assert_eq!(cells(console), written, "cancelling changed the Source");
    assert_eq!(
        cursor(console),
        cursor_before,
        "cancelling moved the Cursor"
    );
    assert_eq!(console.source_view.zoom, zoom, "cancelling moved the Zoom");
    assert_eq!(
        console.orcvs.playback_observation().state,
        PlaybackState::Playing,
        "cancelling stopped Playback"
    );
    assert!(
        engine.has_changed().is_ok(),
        "cancelling replaced the Playback Engine"
    );
}

///
/// New on a Source with nothing written asks nothing and opens straight away.
/// The Cursor and the Zoom are moved first — neither writes the Source — so
/// the Open is seen to have happened.
///
#[tokio::test]
async fn file_new_on_an_empty_source_opens_without_asking() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    harness.key_press(Key::ArrowDown);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    harness.step();
    harness.run_steps(1);
    assert_ne!(cursor(harness.state()), (0, 0), "the Cursor never moved");

    choose_in_file_menu(&mut harness, "New");

    assert!(!asking(&harness), "New asked before discarding nothing");
    assert_eq!(cursor(harness.state()), (0, 0), "New did not open");
    assert_eq!(harness.state().source_view.zoom, 1.0, "New did not open");
}

///
/// The question holds the keys, as an open popup does: Escape cancels rather
/// than confirms, and a character, an arrow or a Zoom chord pressed while it
/// is asking reaches neither the Grid nor the Source View behind it.
///
#[tokio::test]
async fn escape_cancels_the_question_and_keys_never_reach_the_source_behind_it() {
    let mut harness = console_with_written_content();
    let written = cells(harness.state());
    let cursor_before = cursor(harness.state());
    let zoom = harness.state().source_view.zoom;

    choose_in_file_menu(&mut harness, "New");
    assert!(asking(&harness), "New discarded written content unasked");

    harness.event(Event::Text("y".to_owned()));
    harness.key_press(Key::ArrowRight);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Minus);
    harness.step();
    harness.run_steps(1);
    assert!(asking(&harness), "a key closed the question");
    assert_eq!(
        harness.state().source_view.zoom,
        zoom,
        "a Zoom chord pressed while asking zoomed the Source View"
    );
    assert_eq!(
        cells(harness.state()),
        written,
        "a character typed while asking wrote the Source"
    );
    assert_eq!(
        cursor(harness.state()),
        cursor_before,
        "an arrow pressed while asking moved the Cursor"
    );

    harness.key_press(Key::Escape);
    harness.step();
    harness.run_steps(2);

    assert!(!asking(&harness), "Escape left the question showing");
    assert_eq!(cells(harness.state()), written, "Escape confirmed New");
    assert_eq!(
        cursor(harness.state()),
        cursor_before,
        "Escape moved the Cursor"
    );
}

///
/// The question is reachable by keyboard alone: it opens with the safe
/// answer focused, Tab reaches the other, and Enter answers whichever holds
/// focus.
///
#[tokio::test]
async fn the_question_is_answered_from_the_keyboard() {
    let mut harness = console_with_written_content();

    choose_in_file_menu(&mut harness, "New");
    assert!(
        harness.get_by_label("Cancel").is_focused(),
        "the question did not open with Cancel focused"
    );

    harness.key_press(Key::Tab);
    harness.step();
    harness.run_steps(1);
    assert!(
        harness.get_by_label(DISCARD).is_focused(),
        "Tab did not reach {DISCARD}"
    );
    harness.key_press(Key::Enter);
    harness.step();
    harness.run_steps(2);

    assert!(!asking(&harness), "Enter left the question showing");
    assert!(
        cells(harness.state()).iter().all(Option::is_none),
        "Enter on {DISCARD} did not open New"
    );
}
