//!
//! The splash window the native console opens through: what it holds, what
//! ends it, and the window it gives back.
//!
//! Every frame here advances egui's input time by the harness's step, so the
//! hold and the fade are stepped, never waited for.
//!

use egui::{Event, Key, Modifiers, PointerButton, Vec2, ViewportCommand, pos2};
use egui_kittest::{Harness, kittest::Queryable as _};

use super::{cells, sent};
use crate::console::splash::{FADE_SECONDS, HOLD_SECONDS};
use crate::console::tests::start_console;
use crate::console::{Console, DEFAULT_VIEW_SIZE, DEFAULT_VIEW_SIZE_MIN, SPLASH_WINDOW};
use crate::theme_registry::ThemeRegistry;
use crate::window_geometry::stored;

/// Each harness step advances egui's input time by this many seconds.
const STEP: f32 = 0.05;

///
/// A console opening through the splash as `Console::start` opens it, over
/// `storage` when a test gives one.
///
fn splash_console(storage: Option<&dyn eframe::Storage>) -> Harness<'_, Console> {
    Harness::builder()
        .with_size(Vec2::from(SPLASH_WINDOW))
        .with_pixels_per_point(1.0)
        .with_step_dt(STEP)
        .build_eframe(move |cc| {
            cc.storage = storage;
            start_console(
                cc,
                ThemeRegistry::built_in(),
                crate::config::Config::default(),
            )
            .through_splash(stored(storage))
        })
}

/// Runs frames until egui's input time has passed `seconds`.
fn run_for(harness: &mut Harness<'_, Console>, seconds: f64) {
    let steps = (seconds / f64::from(STEP)).ceil() as usize + 1;
    harness.run_steps(steps);
}

fn holding(harness: &Harness<'_, Console>) -> bool {
    harness
        .state()
        .splash
        .as_ref()
        .is_some_and(|splash| splash.holding())
}

fn fading(harness: &Harness<'_, Console>) -> bool {
    harness
        .state()
        .splash
        .as_ref()
        .is_some_and(|splash| !splash.holding())
}

/// Runs one frame with a key tapped in it: pressed and released.
fn press_a_key(harness: &mut Harness<'_, Console>) {
    harness
        .input_mut()
        .events
        .extend([true, false].map(|pressed| Event::Key {
            key: Key::Space,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        }));
    harness.step();
}

/// Whether the console's own chrome is showing.
fn console_showing(harness: &Harness<'_, Console>) -> bool {
    harness.query_by_label("View").is_some()
}

/// Whether the frame just run sent the commands that grow the splash into
/// the console.
fn grew(harness: &Harness<'_, Console>) -> bool {
    let sent = sent(harness);
    sent.contains(&ViewportCommand::Decorations(true))
        && sent.contains(&ViewportCommand::Resizable(true))
        && sent.contains(&ViewportCommand::MinInnerSize(DEFAULT_VIEW_SIZE_MIN.into()))
}

#[tokio::test]
async fn the_splash_holds_the_logo_in_place_of_the_console() {
    let mut harness = splash_console(None);
    run_for(&mut harness, HOLD_SECONDS / 2.0);
    assert!(
        !console_showing(&harness),
        "the console showed through the hold"
    );
    assert!(holding(&harness));
}

#[tokio::test]
async fn the_hold_ends_by_itself_and_grows_the_window_to_the_default_console() {
    let mut harness = splash_console(None);
    let mut grown = false;
    for _ in 0..((HOLD_SECONDS / f64::from(STEP)) as usize + 4) {
        harness.step();
        grown |= grew(&harness)
            && sent(&harness).contains(&ViewportCommand::InnerSize(DEFAULT_VIEW_SIZE.into()));
    }
    assert!(
        grown,
        "the hold ended without growing the window to the console"
    );
    assert!(console_showing(&harness));
}

#[tokio::test]
async fn a_key_press_ends_the_splash_and_reaches_nothing_else() {
    let mut harness = splash_console(None);
    harness.step();
    let before = cells(harness.state());
    // A key press reaches egui with its text in the same frame.
    harness.input_mut().events.extend([
        Event::Key {
            key: Key::X,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        },
        Event::Text("x".to_owned()),
    ]);
    harness.step();
    assert!(!holding(&harness), "the key press left the splash holding");
    run_for(&mut harness, FADE_SECONDS);
    assert_eq!(
        cells(harness.state()),
        before,
        "the dismissing key edited the Source"
    );
    assert!(console_showing(&harness));
}

#[tokio::test]
async fn a_key_held_from_the_splash_reaches_nothing_until_it_is_released() {
    let mut harness = splash_console(None);
    harness.step();
    let before = cells(harness.state());
    let x = |pressed, repeat| Event::Key {
        key: Key::X,
        physical_key: None,
        pressed,
        repeat,
        modifiers: Modifiers::NONE,
    };
    harness.input_mut().events.push(x(true, false));
    harness.step();
    for _ in 0..((FADE_SECONDS / f64::from(STEP)) as usize + 4) {
        harness
            .input_mut()
            .events
            .extend([x(true, true), Event::Text("x".to_owned())]);
        harness.step();
    }
    assert_eq!(
        cells(harness.state()),
        before,
        "the held key's repeats edited the Source"
    );
    harness.input_mut().events.push(x(false, false));
    harness.step();
    assert!(
        harness.state().splash.is_none(),
        "the splash outlived the released key"
    );
}

#[tokio::test]
async fn a_click_ends_the_splash() {
    let mut harness = splash_console(None);
    harness.step();
    harness.event(Event::PointerButton {
        pos: pos2(200.0, 200.0),
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.step();
    assert!(!holding(&harness));
}

#[tokio::test]
async fn the_logo_fades_and_then_the_splash_is_gone() {
    let mut harness = splash_console(None);
    harness.step();
    press_a_key(&mut harness);
    assert!(fading(&harness));
    run_for(&mut harness, FADE_SECONDS);
    assert!(!fading(&harness));
}

#[cfg(feature = "persistence")]
mod with_storage {
    use eframe::App as _;

    use egui::vec2;

    use super::super::{asking, request_close};
    use super::*;
    use crate::persistence::InMemoryStorage;
    use crate::window_geometry::{WindowGeometry, store};

    /// A geometry the console was closed at on an earlier run.
    fn closed_at() -> WindowGeometry {
        WindowGeometry {
            inner_size: vec2(900.0, 640.0),
            outer_position: Some(pos2(120.0, 80.0)),
            maximized: false,
            fullscreen: false,
        }
    }

    /// A storage holding only the window geometry `closed_at` gives.
    fn closed_at_storage() -> InMemoryStorage {
        let mut storage = InMemoryStorage::default();
        store(&mut storage, &closed_at());
        storage
    }

    #[tokio::test]
    async fn the_window_grows_to_the_geometry_the_console_was_closed_at() {
        let storage = closed_at_storage();
        let mut harness = splash_console(Some(&storage));
        harness.step();
        press_a_key(&mut harness);
        assert!(grew(&harness));
        let sent = sent(&harness);
        assert!(sent.contains(&ViewportCommand::InnerSize(vec2(900.0, 640.0))));
        assert!(sent.contains(&ViewportCommand::OuterPosition(pos2(120.0, 80.0))));
    }

    #[tokio::test]
    async fn a_relaunch_grows_the_splash_to_the_window_the_console_last_ran_in() {
        let mut storage = InMemoryStorage::default();
        {
            let mut running = Harness::builder()
                .with_size(vec2(900.0, 640.0))
                .with_pixels_per_point(1.0)
                .build_eframe(|cc| {
                    start_console(
                        cc,
                        ThemeRegistry::built_in(),
                        crate::config::Config::default(),
                    )
                });
            running.step();
            running.state_mut().save(&mut storage);
        }
        let mut harness = splash_console(Some(&storage));
        harness.step();
        press_a_key(&mut harness);
        assert!(sent(&harness).contains(&ViewportCommand::InnerSize(vec2(900.0, 640.0))));
    }

    #[tokio::test]
    async fn a_save_during_the_hold_stores_the_consoles_geometry() {
        let storage = closed_at_storage();
        let mut harness = splash_console(Some(&storage));
        harness.step();
        let mut saved = InMemoryStorage::default();
        harness.state_mut().save(&mut saved);
        assert_eq!(stored(Some(&saved)), Some(closed_at()));
    }

    #[tokio::test]
    async fn a_save_during_the_hold_stores_no_geometry_when_none_was_stored() {
        let mut harness = splash_console(None);
        harness.step();
        let mut storage = InMemoryStorage::default();
        harness.state_mut().save(&mut storage);
        assert_eq!(stored(Some(&storage)), None, "the splash window was stored");
    }

    #[tokio::test]
    async fn a_quit_during_the_hold_stores_the_consoles_geometry() {
        let storage = closed_at_storage();
        let mut harness = splash_console(Some(&storage));
        harness.step();
        request_close(&mut harness);
        assert!(
            !sent(&harness).contains(&ViewportCommand::CancelClose),
            "a clean console held the window open"
        );
        let mut saved = InMemoryStorage::default();
        harness.state_mut().save(&mut saved);
        assert_eq!(stored(Some(&saved)), Some(closed_at()));
    }

    #[tokio::test]
    async fn a_close_during_the_hold_ends_the_splash_and_asks_about_unsaved_changes() {
        let mut storage = InMemoryStorage::default();
        crate::persistence::save(&mut storage, &crate::persistence::edited_source());
        let mut harness = splash_console(Some(&storage));
        harness.step();

        request_close(&mut harness);
        assert!(grew(&harness), "the close left the window a splash");
        assert!(sent(&harness).contains(&ViewportCommand::CancelClose));
        run_for(&mut harness, FADE_SECONDS);
        assert!(asking(&harness), "the close asked nothing");
    }
}
