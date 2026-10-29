//!
//! The bottom Panel's Readouts and destination field, as a viewer and
//! assistive technology meet them.
//!

use egui::accesskit::Role;
use egui::{Color32, TextStyle, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use orcvs::midi::MidiDestinationId;

use super::super::{Console, DEFAULT_VIEW_SIZE};
use super::running_console;
use crate::web_midi::{
    AccessStatus,
    tests::{FakeWebMidi, backend},
};

///
/// A running console whose MIDI selection lists the outputs `fake` holds and has
/// installed `first`.
///
async fn console_over(fake: &FakeWebMidi, first: &str) -> Harness<'static, Console> {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.state_mut().midi = crate::midi::MidiDeviceSelection::new(
        harness.state().orcvs.midi_selection_handle(),
        Box::new(backend(fake)),
    );
    harness.state_mut().midi.refresh_destinations();
    harness
        .state_mut()
        .midi
        .select_destination(&MidiDestinationId::new(first));
    tokio::task::yield_now().await;
    harness.run_steps(2);
    harness
}

///
/// The beat marker, the Tick and the Run Clock each carry their copy to
/// assistive technology, as the B, T, C and O labels beside them do.
///
#[tokio::test]
async fn the_tick_run_clock_and_beat_marker_readouts_are_in_the_accessibility_tree() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    assert!(
        harness.query_by_value("T").is_some(),
        "the T label is the control: it is in the tree"
    );
    let missing: Vec<&str> = ["//", "00000", "00:00"]
        .into_iter()
        .filter(|copy| harness.query_by_value(copy).is_none())
        .collect();
    assert!(
        missing.is_empty(),
        "these Readouts have no accessibility node: {missing:?}"
    );
}

///
/// The arrowless destination field is as wide as its device name and its
/// padding, with no empty icon slot after a name longer than its minimum width.
///
#[tokio::test]
async fn a_long_destination_name_leaves_no_empty_icon_slot_in_its_field() {
    const NAME: &str = "A destination whose device name runs long";
    let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", Some(NAME))]);
    let harness = console_over(&fake, "a").await;
    let readout = format!("{NAME} · 127.0.0.1");
    let field = harness.get_by_value(&readout).rect();
    let font_id = harness.ctx.global_style().text_styles[&TextStyle::Monospace].clone();
    let text_width = harness.ctx.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(readout.clone(), font_id, Color32::WHITE)
            .size()
            .x
    });
    let padding = 2.0 * f32::from(super::super::panel::BPM_FIELD_MARGIN.left);
    let slack = field.width() - text_width - padding;
    assert!(
        slack.abs() < 1.0,
        "the field is {slack}pt wider than its name and padding"
    );
}

///
/// A MIDI status takes the line beneath the destination field rather than
/// touching its frame.
///
#[tokio::test]
async fn the_midi_status_sits_beneath_the_destination_field() {
    let fake = FakeWebMidi::with(
        AccessStatus::Granted,
        &[("a", Some("First")), ("b", Some("Second"))],
    );
    fake.close_port("b");
    let mut harness = console_over(&fake, "a").await;
    harness.get_by_value("First · 127.0.0.1").click();
    harness.run_steps(2);
    harness.get_by_label("Second").click();
    harness.run_steps(2);
    let field = harness.get_by_value("First · 127.0.0.1").rect();
    let status = harness
        .get_by(|node| {
            node.role() == Role::Label && node.value().as_deref() == Some("Opening Second…")
        })
        .rect();
    let gap = status.top() - field.bottom();
    assert!(
        gap >= 0.0,
        "the status overlaps the destination field by {}pt",
        -gap
    );
}

///
/// The Output list holds one labelled section per output type, MIDI's above
/// the UDP and OSC destination's, and its closed field names both.
///
#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn the_output_list_has_a_section_for_each_output_type() {
    let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", Some("First"))]);
    let mut harness = console_over(&fake, "a").await;
    assert!(harness.query_by_label("MIDI").is_none());

    harness.get_by_value("First · 127.0.0.1").click();
    harness.run_steps(2);

    let midi = harness.get_by_label("MIDI").rect();
    let network = harness.get_by_label("UDP/OSC").rect();
    let host = field(&harness, "127.0.0.1").rect();
    let destination = harness.get_by_label("First").rect();
    assert!(
        midi.bottom() <= destination.top(),
        "MIDI heads its destinations"
    );
    assert!(
        destination.bottom() <= network.top(),
        "UDP/OSC follows MIDI"
    );
    assert!(network.bottom() <= host.top(), "UDP/OSC heads the host");
}

///
/// The UDP and OSC destination fields in the Output list take a typed value on
/// Enter, and report one that is not a destination without taking it.
///
#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn the_network_destination_fields_take_a_valid_value_and_report_an_invalid_one() {
    use orcvs::network::NetworkDestination;

    let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", Some("First"))]);
    let mut harness = console_over(&fake, "a").await;
    harness.get_by_value("First · 127.0.0.1").click();
    harness.run_steps(2);

    field(&harness, "49161").click();
    harness.run_steps(1);
    field(&harness, "49161").type_text("9000");
    harness.key_press(egui::Key::Enter);
    harness.run_steps(2);
    assert_eq!(harness.state().network.destination().udp_port, 9000);

    field(&harness, "127.0.0.1").click();
    harness.run_steps(1);
    field(&harness, "127.0.0.1").type_text("nowhere");
    harness.key_press(egui::Key::Enter);
    harness.run_steps(2);
    assert_eq!(
        harness.state().network.destination(),
        NetworkDestination {
            udp_port: 9000,
            ..NetworkDestination::default()
        }
    );
    assert!(
        harness
            .query_by(|node| node.role() == Role::Label
                && node.value().as_deref() == Some("host \"nowhere\" is not an IP address"))
            .is_some(),
        "the refused host is not reported on the Panel"
    );
    assert!(
        harness
            .query_by(|node| is_field(node, "127.0.0.1"))
            .is_some(),
        "the refused host stayed in its field"
    );
}

///
/// Escape cancels a typed port, and a click outside the Output list closes it
/// and keeps the port typed before it.
///
#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn escape_cancels_a_typed_port_and_a_click_outside_keeps_one() {
    let fake = FakeWebMidi::with(AccessStatus::Granted, &[("a", Some("First"))]);
    let mut harness = console_over(&fake, "a").await;
    harness.get_by_value("First · 127.0.0.1").click();
    harness.run_steps(2);

    field(&harness, "49162").click();
    harness.run_steps(1);
    field(&harness, "49162").type_text("7000");
    harness.key_press(egui::Key::Escape);
    harness.run_steps(2);
    assert_eq!(harness.state().network.destination().osc_port, 49162);

    if harness.query_by(|node| is_field(node, "49162")).is_none() {
        harness.get_by_value("First · 127.0.0.1").click();
        harness.run_steps(2);
    }
    field(&harness, "49162").click();
    harness.run_steps(1);
    field(&harness, "49162").type_text("7001");
    harness.run_steps(1);
    harness.get_by_value("T").click();
    harness.run_steps(3);
    assert!(
        harness.query_by(|node| is_field(node, "7001")).is_none(),
        "the Output list stayed open"
    );
    assert_eq!(harness.state().network.destination().osc_port, 7001);
}

#[cfg(not(target_arch = "wasm32"))]
fn is_field(node: &egui_kittest::kittest::AccessKitNode<'_>, value: &str) -> bool {
    node.role() == Role::TextInput && node.value().as_deref() == Some(value)
}

/// The Panel text field showing `value`.
#[cfg(not(target_arch = "wasm32"))]
fn field<'tree>(
    harness: &'tree Harness<'static, Console>,
    value: &'tree str,
) -> egui_kittest::Node<'tree> {
    harness.get_by(|node| is_field(node, value))
}
