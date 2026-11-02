//!
//! The Toasts each standing message floats as, and the Panel that keeps its
//! height whatever they say (ADR 0100).
//!

use egui::Vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use orcvs::playback::{PlaybackDiagnostic, PlaybackObservation};

use super::super::tests::quiet_midi;
use super::super::{Console, DEFAULT_VIEW_SIZE};
use super::running_console;

/// The window edge a Toast keeps its distance from, in points.
const INSET: f32 = 16.0;

///
/// A running console whose MIDI reports nothing, with Diagnostics as
/// `diagnostics_open` says.
///
fn quiet_console(diagnostics_open: bool) -> Harness<'static, Console> {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    quiet_midi(harness.state_mut());
    harness.state_mut().diagnostics_open = diagnostics_open;
    harness
}

/// Reports an engine failure that says `message`.
fn fail_the_engine(harness: &mut Harness<'_, Console>, message: &str) {
    harness.state_mut().playback_status.observe(
        vec![PlaybackDiagnostic::ClockFailure {
            message: message.to_owned(),
        }],
        &PlaybackObservation::default(),
    );
}

/// Clears the engine failure, as the next run beginning does.
fn clear_the_engine(harness: &mut Harness<'_, Console>) {
    harness.state_mut().playback_status.observe(
        Vec::new(),
        &PlaybackObservation {
            state: orcvs::playback::PlaybackState::Playing,
            run_started_at: Some(std::time::Instant::now()),
            ..PlaybackObservation::default()
        },
    );
}

/// The bottom of the menu bar, from the panel egui laid out.
fn menu_bar_bottom(harness: &Harness<'_, Console>) -> f32 {
    egui::containers::panel::PanelState::load(&harness.ctx, egui::Id::new("top_panel"))
        .expect("the menu bar is present")
        .outer_rect
        .bottom()
}

///
/// An engine failure floats as a Toast in the window's top-right corner,
/// beneath the menu bar: its output's label and its message.
///
#[tokio::test]
async fn a_failure_floats_as_a_toast_in_the_top_right_corner() {
    let mut harness = quiet_console(false);
    harness.run_steps(2);
    assert!(harness.query_by_label("ENGINE").is_none());

    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);

    let label = harness.get_by_label("ENGINE").rect();
    let message = harness.get_by_label("clock stopped").rect();
    let dismiss = harness.get_by_label("×").rect();
    let window = Vec2::from(DEFAULT_VIEW_SIZE);
    assert!(
        label.top() >= menu_bar_bottom(&harness) + INSET,
        "the Toast's label is at {} beneath a menu bar ending at {}",
        label.top(),
        menu_bar_bottom(&harness)
    );
    assert!(
        message.top() >= label.bottom(),
        "the message is beneath its label"
    );
    assert!(
        dismiss.right() <= window.x - INSET && dismiss.right() > window.x - 2.0 * INSET - 20.0,
        "the dismiss button ends at {} in a {}pt window",
        dismiss.right(),
        window.x
    );
    assert!(
        label.left() >= window.x - INSET - 380.0,
        "the Toast starts at {}, wider than 380 points",
        label.left()
    );
}

///
/// A message too long for three lines is cut short with an ellipsis, so the
/// Toast stays 380 points wide and three lines tall.
///
#[tokio::test]
async fn a_long_message_is_cut_at_three_lines() {
    let message = "the clock stopped ".repeat(40);
    let mut harness = quiet_console(false);
    fail_the_engine(&mut harness, &message);
    harness.run_steps(3);

    let shown = harness
        .get_by(|node| {
            node.role() == egui::accesskit::Role::Label
                && node
                    .value()
                    .is_some_and(|value| value.starts_with("the clock stopped"))
        })
        .rect();
    let line = harness.ctx.global_style().text_styles[&egui::TextStyle::Body].size;
    assert!(
        shown.height() <= 3.0 * line * 1.5,
        "the message is {}pt tall in {line}pt lines",
        shown.height()
    );
    assert!(
        shown.right() <= Vec2::from(DEFAULT_VIEW_SIZE).x - INSET,
        "the message runs to {}",
        shown.right()
    );
}

///
/// `×` removes the Toast, and it does not return while its source reports the
/// same message. Once the source clears, the next message raises a new one.
///
#[tokio::test]
async fn a_dismissed_toast_stays_gone_until_its_source_clears() {
    let mut harness = quiet_console(false);
    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);

    harness.get_by_label("×").click();
    harness.run_steps(3);
    assert!(
        harness.query_by_label("clock stopped").is_none(),
        "dismissed"
    );

    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);
    assert!(
        harness.query_by_label("clock stopped").is_none(),
        "the same message raised the dismissed Toast again"
    );

    clear_the_engine(&mut harness);
    harness.run_steps(3);
    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);
    assert!(
        harness.query_by_label("clock stopped").is_some(),
        "the message after the source cleared raised no Toast"
    );
}

///
/// The Panel is the same height with no message and with messages from two
/// sources, with or without the Diagnostics readouts: the Play Cell and the
/// readouts stay put.
///
#[tokio::test]
async fn the_panel_keeps_its_height_whatever_the_messages() {
    for diagnostics_open in [false, true] {
        let panel_with = |failure: Option<&str>| {
            let mut harness = quiet_console(diagnostics_open);
            if let Some(message) = failure {
                fail_the_engine(&mut harness, message);
                let refused = harness
                    .state_mut()
                    .network
                    .commit(crate::network::NetworkField::UdpPort, "not a port");
                assert!(!refused, "the Output list took a port that is not one");
            }
            harness.run_steps(3);
            let play = harness.get_by_label("Play").rect();
            let fps = harness.query_by_label("FPS").map(|node| node.rect());
            (play, fps)
        };
        assert_eq!(
            panel_with(None),
            panel_with(Some("clock stopped")),
            "a message moved the Panel, Diagnostics open: {diagnostics_open}"
        );
    }
}

///
/// In a window too narrow for a 380-point Toast, the Toast narrows to keep its
/// inset from both edges, so its label and message stay on screen.
///
#[tokio::test]
async fn a_toast_narrows_to_fit_a_narrow_window() {
    let window = Vec2::new(360.0, 640.0);
    let mut harness = running_console(window);
    quiet_midi(harness.state_mut());
    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);

    let label = harness.get_by_label("ENGINE").rect();
    let dismiss = harness.get_by_label("×").rect();
    assert!(
        label.left() >= INSET,
        "the Toast's label starts at {} in a {}pt window",
        label.left(),
        window.x
    );
    assert!(
        dismiss.right() <= window.x - INSET,
        "the dismiss button ends at {} in a {}pt window",
        dismiss.right(),
        window.x
    );
}

///
/// Escape backs out of the innermost thing first: with a Region selected it
/// collapses the Region and leaves the Toast, and the next Escape, with
/// nothing else to undo, dismisses it.
///
#[tokio::test]
async fn escape_dismisses_the_toasts_once_it_has_nothing_else_to_undo() {
    let mut harness = quiet_console(false);
    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::ArrowRight);
    harness.run_steps(1);
    assert!(!harness.state().orcvs.region().is_one_cell(), "a Region");

    harness.key_press(egui::Key::Escape);
    harness.run_steps(2);
    assert!(harness.state().orcvs.region().is_one_cell(), "collapsed");
    assert!(
        harness.query_by_label("clock stopped").is_some(),
        "the Escape that collapsed the Region dismissed the Toast"
    );

    harness.key_press(egui::Key::Escape);
    harness.run_steps(2);
    assert!(
        harness.query_by_label("clock stopped").is_none(),
        "an Escape with nothing else to undo left the Toast"
    );
}
