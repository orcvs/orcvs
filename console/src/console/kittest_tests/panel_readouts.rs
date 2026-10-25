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
    let field = harness.get_by_value(NAME).rect();
    let font_id = harness.ctx.global_style().text_styles[&TextStyle::Monospace].clone();
    let text_width = harness.ctx.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(NAME.to_owned(), font_id, Color32::WHITE)
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
/// A MIDI status stands apart from the destination field it follows, rather
/// than touching its frame.
///
#[tokio::test]
async fn the_midi_status_does_not_touch_the_destination_field() {
    let fake = FakeWebMidi::with(
        AccessStatus::Granted,
        &[("a", Some("First")), ("b", Some("Second"))],
    );
    fake.close_port("b");
    let mut harness = console_over(&fake, "a").await;
    harness.get_by_value("First").click();
    harness.run_steps(2);
    harness.get_by_label("Second").click();
    harness.run_steps(2);
    let field = harness.get_by_value("First").rect();
    let status = harness
        .get_by(|node| {
            node.role() == Role::Label && node.value().as_deref() == Some("Opening Second…")
        })
        .rect();
    let gap = status.left() - field.right();
    assert!(
        gap >= 1.0,
        "the status sits {gap}pt from the destination field"
    );
}
