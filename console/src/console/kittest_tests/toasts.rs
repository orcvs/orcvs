//!
//! The Toasts each standing message floats as, and the Panel that keeps its
//! height whatever they say.
//!

use egui::Vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use orcvs::midi::{MidiConnectionRequest, MidiDestination, MidiDestinationId};
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
    egui::containers::panel::PanelState::load(
        &harness.ctx,
        super::super::tests::panel_id("top_panel"),
    )
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
/// `×` removes the Toast, and it does not return while its Reporter has the
/// same message. Once the Reporter clears, the next message raises a new one.
///
#[tokio::test]
async fn a_dismissed_toast_stays_gone_until_its_reporter_clears() {
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
        "the message after the Reporter cleared raised no Toast"
    );
}

///
/// A MIDI service that lists one destination, `a`, whose connection refuses
/// every message, and refuses to connect any other.
///
struct RefusingMidi;

/// The connection [`RefusingMidi`] opens, which no message leaves.
struct RefusingConnection;

impl orcvs::midi::MidiConnection for RefusingConnection {
    fn send(&mut self, _message: &[u8]) -> Result<(), orcvs::midi::MidiError> {
        Err(orcvs::midi::MidiError::new("the synth refused"))
    }
}

impl orcvs::midi::MidiBackend for RefusingMidi {
    fn destinations(&mut self) -> Result<Vec<MidiDestination>, orcvs::midi::MidiError> {
        Ok(vec![MidiDestination::new("a", "Synth")])
    }

    fn connect(
        &mut self,
        destination_id: &MidiDestinationId,
    ) -> Result<MidiConnectionRequest, orcvs::midi::MidiError> {
        if destination_id.as_str() == "a" {
            Ok(MidiConnectionRequest::Ready(Box::new(RefusingConnection)))
        } else {
            Err(orcvs::midi::MidiError::new("that destination is gone"))
        }
    }
}

///
/// Writes a clock that bangs a UDP send and a MIDI note, and starts the run.
///
fn play_a_udp_send_and_a_note(harness: &mut Harness<'_, Console>) {
    crate::info::write_rows(
        &harness.state().orcvs,
        &[".=0101", "  !uhi!", "", ".=0101", "  !>007FC4"],
    );
    harness.state_mut().orcvs.toggle_playback();
}

///
/// Raises a message from every Reporter while the run plays: a MIDI choice
/// that cannot connect, a MIDI output that refuses each note, a refused UDP
/// port, a UDP send to port 0 that the operating system refuses, and an
/// engine failure.
///
async fn raise_every_message(harness: &mut Harness<'_, Console>) {
    let handle = harness.state().orcvs.midi_selection_handle();
    harness.state_mut().midi =
        crate::midi::MidiDeviceSelection::new(handle, Box::new(RefusingMidi));
    harness.state_mut().midi.refresh_destinations();
    harness
        .state_mut()
        .midi
        .select_destination(&MidiDestinationId::new("a"));
    tokio::task::yield_now().await;
    harness.run_steps(2);
    harness
        .state_mut()
        .midi
        .select_destination(&MidiDestinationId::new("gone"));
    let refused = harness
        .state_mut()
        .network
        .commit(crate::network::NetworkField::UdpPort, "not a port");
    assert!(!refused, "the Output list took a port that is not one");
    // Port 0 is no destination the Output list takes; the test sets it on
    // Orcvs so the operating system refuses the send.
    harness
        .state_mut()
        .orcvs
        .set_network_destination(orcvs::network::NetworkDestination {
            udp_port: 0,
            ..orcvs::network::NetworkDestination::default()
        });
    play_a_udp_send_and_a_note(harness);
    for _ in 0..80 {
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        harness.run_steps(1);
        let console = harness.state();
        if console.playback_status.midi().is_some() && console.playback_status.network().is_some() {
            break;
        }
    }
    // The run's own observation, so its start does not clear the failure.
    let observation = harness.state().orcvs.playback_observation();
    harness.state_mut().playback_status.observe(
        vec![PlaybackDiagnostic::ClockFailure {
            message: "clock stopped".to_owned(),
        }],
        &observation,
    );
}

///
/// The Panel is the same height with no message and with a message from every
/// Reporter, with or without the Diagnostics readouts: the Play Cell and the
/// readouts stay put.
///
#[tokio::test]
async fn the_panel_keeps_its_height_whatever_the_messages() {
    for diagnostics_open in [false, true] {
        let mut quiet = quiet_console(diagnostics_open);
        play_a_udp_send_and_a_note(&mut quiet);
        let mut reporting = quiet_console(diagnostics_open);
        raise_every_message(&mut reporting).await;
        for harness in [&mut quiet, &mut reporting] {
            harness.run_steps(3);
        }

        let console = reporting.state();
        let standing = super::super::toasts::standing_messages(
            &console.midi,
            &console.network,
            &console.playback_status,
        );
        let silent: Vec<_> = standing
            .iter()
            .filter(|(_, message)| message.is_none())
            .map(|(reporter, _)| reporter)
            .collect();
        assert!(
            silent.is_empty(),
            "these Reporters raised no message: {silent:?}"
        );

        let panel = |harness: &Harness<'_, Console>| {
            let play = harness.get_by_label("Play").rect();
            let fps = harness.query_by_label("FPS").map(|node| node.rect());
            let bottom = egui::containers::panel::PanelState::load(
                &harness.ctx,
                super::super::tests::panel_id("bottom_panel"),
            )
            .expect("the Panel is present")
            .outer_rect;
            (play, fps, bottom)
        };
        assert_eq!(
            panel(&quiet),
            panel(&reporting),
            "a message moved the Panel, Diagnostics open: {diagnostics_open}"
        );
    }
}

/// The colour the Panel's **O** is painted in on the last frame.
fn output_label_colour(harness: &Harness<'_, Console>) -> egui::Color32 {
    fn find(shape: &egui::Shape) -> Option<egui::Color32> {
        match shape {
            egui::Shape::Text(text) if text.galley.text() == "O" => {
                let colour = text.galley.job.sections[0].format.color;
                Some(if colour == egui::Color32::PLACEHOLDER {
                    text.fallback_color
                } else {
                    colour
                })
            }
            egui::Shape::Vec(shapes) => shapes.iter().find_map(find),
            _ => None,
        }
    }
    harness
        .output()
        .shapes
        .iter()
        .find_map(|clipped| find(&clipped.shape))
        .expect("the Panel paints O")
}

///
/// A Toast fades once its seconds have passed, and does not return for the
/// same message; the Panel's **O** stays in the error colour while the message
/// stands, and returns to its own colour once it clears.
///
#[tokio::test]
async fn a_toast_fades_after_its_seconds_and_o_keeps_the_failure() {
    let mut harness = quiet_console(false);
    let error = harness.ctx.global_style().visuals.error_fg_color;
    harness.run_steps(2);
    assert_ne!(output_label_colour(&harness), error, "nothing has failed");

    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);
    assert!(harness.query_by_label("clock stopped").is_some());
    assert_eq!(
        output_label_colour(&harness),
        error,
        "while the Toast shows"
    );

    // The harness steps a quarter of a second; the default is five seconds.
    harness.run_steps(24);
    assert!(
        harness.query_by_label("clock stopped").is_none(),
        "the Toast is still showing after six seconds"
    );
    assert_eq!(
        output_label_colour(&harness),
        error,
        "after the Toast faded"
    );

    clear_the_engine(&mut harness);
    harness.run_steps(2);
    assert_ne!(
        output_label_colour(&harness),
        error,
        "once the message cleared"
    );
}

///
/// `[toasts] seconds` sets how long a Toast shows: one second here, so it is
/// still showing after three quarters of one and gone well after.
///
#[tokio::test]
async fn a_toast_lasts_the_configured_seconds() {
    let config = crate::config::Config {
        toast_seconds: 1,
        ..crate::config::Config::default()
    };
    let mut harness = super::console_harness(
        Vec2::from(DEFAULT_VIEW_SIZE),
        None,
        crate::theme_registry::ThemeRegistry::built_in(),
        config,
    );
    quiet_midi(harness.state_mut());
    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);
    assert!(
        harness.query_by_label("clock stopped").is_some(),
        "within its second"
    );

    harness.run_steps(6);
    assert!(
        harness.query_by_label("clock stopped").is_none(),
        "still showing after two seconds"
    );
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

///
/// A Toast's label is painted in the presented Theme's diagnostic red, the
/// colour invalid Cells take, so the output that failed stands out under
/// every Theme.
///
#[tokio::test]
async fn a_toasts_label_is_painted_in_the_diagnostic_red() {
    fn colour_of(shape: &egui::Shape, text: &str) -> Option<egui::Color32> {
        match shape {
            egui::Shape::Text(shape) if shape.galley.text() == text => {
                let colour = shape.galley.job.sections[0].format.color;
                Some(if colour == egui::Color32::PLACEHOLDER {
                    shape.fallback_color
                } else {
                    colour
                })
            }
            egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| colour_of(shape, text)),
            _ => None,
        }
    }
    let mut harness = quiet_console(false);
    fail_the_engine(&mut harness, "clock stopped");
    harness.run_steps(3);
    let appearance = crate::theme::Appearance::from(harness.ctx.theme());
    let red = harness
        .state()
        .themes
        .presented(appearance)
        .diagnostic_foreground;
    let painted = harness
        .output()
        .shapes
        .iter()
        .find_map(|clipped| colour_of(&clipped.shape, "ENGINE"))
        .expect("the Toast paints its label");
    assert_eq!(painted, red);
}
