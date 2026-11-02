//! The Toasts: each message the outputs report, floating in the Source's
//! top-right corner beneath the menu bar, clear of the Minimap.

use egui::text::LayoutJob;
use egui::{Align2, Rect, Sense, Stroke, TextStyle};

use super::Console;
use super::panel::panel_monospace_id;
use crate::midi::MidiDeviceSelection;
use crate::network::NetworkSetting;
use crate::playback_status::PlaybackStatus;

/// How many sources report a message, each with a Toast of its own.
const SOURCES: usize = 5;
/// Points between a Toast and the Source's edges.
const INSET: f32 = 16.0;
/// Points between two Toasts in the stack.
const GAP: f32 = 8.0;
/// A Toast's width in points.
const WIDTH: f32 = 380.0;
/// The lines a message wraps to before it is cut with an ellipsis.
const MESSAGE_ROWS: usize = 3;

///
/// Each source's message, labelled with the output it belongs to, in the order
/// the Toasts stack: the MIDI selection, a MIDI output failure, a UDP or OSC
/// destination the Output list refused, a failed UDP or OSC send, and an
/// engine failure.
///
pub(super) fn statuses<'a>(
    midi: &'a MidiDeviceSelection,
    network: &'a NetworkSetting,
    playback_status: &'a PlaybackStatus,
) -> [(&'static str, Option<&'a str>); SOURCES] {
    [
        ("MIDI", midi.status()),
        ("MIDI", playback_status.midi()),
        ("UDP/OSC", network.status()),
        ("UDP/OSC", playback_status.network()),
        ("ENGINE", playback_status.engine()),
    ]
}

///
/// One source's message, from the frame it was raised until the source reports
/// no message or a different one.
///
struct Toast {
    label: &'static str,
    text: String,
    /// When it was raised, in egui's input time, so the stack drops the oldest.
    raised: f64,
    dismissed: bool,
    /// Its height in points as last shown, so the stack can fit above the
    /// Minimap before it is laid out. A new Toast's is unknown, but egui
    /// draws a new `Area` invisibly on its first frame, only measuring it.
    height: f32,
}

/// The Toast for each message source, in [`statuses`] order.
#[derive(Default)]
pub(super) struct Toasts {
    slots: [Option<Toast>; SOURCES],
}

impl Toasts {
    ///
    /// Takes the messages standing at `now`. A source that reports no message,
    /// or a different one, forgets its Toast, dismissed or not; a new message
    /// raises a new Toast.
    ///
    fn observe(&mut self, statuses: [(&'static str, Option<&str>); SOURCES], now: f64) {
        for (slot, (label, status)) in self.slots.iter_mut().zip(statuses) {
            match status {
                None => *slot = None,
                Some(text) if slot.as_ref().is_some_and(|toast| toast.text == text) => {}
                Some(text) => {
                    *slot = Some(Toast {
                        label,
                        text: text.to_owned(),
                        raised: now,
                        dismissed: false,
                        height: 0.0,
                    });
                }
            }
        }
    }

    ///
    /// The sources whose Toasts show, in order, in a stack no taller than
    /// `room`: the oldest go first until the rest fit.
    ///
    fn fitting(&self, room: f32) -> Vec<usize> {
        let mut shown: Vec<(usize, &Toast)> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(slot, toast)| Some((slot, toast.as_ref()?)))
            .filter(|(_, toast)| !toast.dismissed)
            .collect();
        let height = |shown: &[(usize, &Toast)]| {
            shown
                .iter()
                .map(|(_, toast)| toast.height + GAP)
                .sum::<f32>()
                - GAP
        };
        while height(&shown) > room
            && let Some(oldest) =
                (0..shown.len()).min_by(|&a, &b| shown[a].1.raised.total_cmp(&shown[b].1.raised))
        {
            shown.remove(oldest);
        }
        shown.into_iter().map(|(slot, _)| slot).collect()
    }
}

impl Console {
    ///
    /// Shows a Toast for each standing message in the top-right corner of
    /// `source`, the Source's rect, down to `floor`, the Minimap's top edge or
    /// the Source's bottom. Each is its own `Area`, so it takes the pointer
    /// over its own frame and nowhere else, and its `×` senses a click but not
    /// focus, so keyboard input stays with the Source.
    ///
    pub(super) fn show_toasts(&mut self, ctx: &egui::Context, source: Rect, floor: f32) {
        let now = ctx.input(|input| input.time);
        let Self {
            toasts,
            midi,
            network,
            playback_status,
            ..
        } = self;
        toasts.observe(statuses(midi, network, playback_status), now);
        let top = source.top() + INSET;
        let mut y = top;
        for slot in toasts.fitting(floor - INSET - top) {
            let toast = toasts.slots[slot].as_mut().expect("a fitting Toast stands");
            let shown = egui::Area::new(egui::Id::new(("toast", slot)))
                .pivot(Align2::RIGHT_TOP)
                .fixed_pos(egui::pos2(source.right() - INSET, y))
                .constrain(false)
                .show(ctx, |ui| show_toast(ui, toast.label, &toast.text));
            toast.dismissed |= shown.inner;
            if toast.height != shown.response.rect.height() {
                toast.height = shown.response.rect.height();
                ctx.request_repaint();
            }
            y += toast.height + GAP;
        }
    }
}

///
/// One Toast: `label` in the weak text colour, a `×` that answers whether it
/// was clicked, and `message` in the error colour, wrapped to
/// [`MESSAGE_ROWS`] lines and then cut, whole on hover.
///
fn show_toast(ui: &mut egui::Ui, label: &str, message: &str) -> bool {
    let error = ui.visuals().error_fg_color;
    let weak = ui.visuals().weak_text_color();
    let frame = egui::Frame::popup(ui.style()).stroke(Stroke::new(1.0, error));
    frame
        .show(ui, |ui| {
            ui.set_width(WIDTH - frame.total_margin().sum().x);
            let dismissed = ui
                .horizontal(|ui| {
                    let label = egui::RichText::new(label)
                        .font(panel_monospace_id(ui))
                        .color(weak);
                    ui.add(egui::Label::new(label).selectable(false));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::Button::new("×").frame(false).sense(Sense::CLICK))
                            .clicked()
                    })
                    .inner
                })
                .inner;
            let font = TextStyle::Body.resolve(ui.style());
            let mut job = LayoutJob::simple(message.to_owned(), font, error, ui.available_width());
            job.wrap.max_rows = MESSAGE_ROWS;
            ui.add(egui::Label::new(job).wrap().selectable(false));
            dismissed
        })
        .inner
}

#[cfg(test)]
mod tests {
    use super::Toasts;

    const STANDING: [(&str, Option<&str>); 5] = [
        ("MIDI", None),
        ("MIDI", None),
        ("UDP/OSC", None),
        ("UDP/OSC", None),
        ("ENGINE", Some("clock stopped")),
    ];
    const CLEAR: [(&str, Option<&str>); 5] = [("", None); 5];

    /// The five sources reporting `messages`.
    fn reporting(messages: [Option<&'static str>; 5]) -> [(&'static str, Option<&'static str>); 5] {
        messages.map(|message| ("", message))
    }

    fn dismiss(toasts: &mut Toasts, slot: usize) {
        if let Some(toast) = toasts.slots[slot].as_mut() {
            toast.dismissed = true;
        }
    }

    #[test]
    fn a_message_raises_a_toast_that_stays_while_it_stands() {
        let mut toasts = Toasts::default();
        toasts.observe(STANDING, 0.0);
        toasts.observe(STANDING, 1.0);
        assert_eq!(toasts.fitting(f32::INFINITY), [4]);
    }

    #[test]
    fn a_dismissed_toast_returns_only_after_its_source_clears() {
        let mut toasts = Toasts::default();
        toasts.observe(STANDING, 0.0);
        dismiss(&mut toasts, 4);
        toasts.observe(STANDING, 1.0);
        assert!(toasts.fitting(f32::INFINITY).is_empty(), "the same message");

        toasts.observe(CLEAR, 2.0);
        toasts.observe(STANDING, 3.0);
        assert_eq!(toasts.fitting(f32::INFINITY), [4], "after a clear frame");
    }

    #[test]
    fn a_different_message_raises_a_new_toast() {
        let mut toasts = Toasts::default();
        toasts.observe(STANDING, 0.0);
        dismiss(&mut toasts, 4);
        toasts.observe(
            reporting([None, None, None, None, Some("cannot start")]),
            1.0,
        );
        assert_eq!(toasts.fitting(f32::INFINITY), [4]);
    }

    #[test]
    fn a_stack_too_tall_for_its_room_drops_its_oldest() {
        let mut toasts = Toasts::default();
        toasts.observe(reporting([None, None, None, None, Some("engine")]), 0.0);
        toasts.observe(
            reporting([Some("midi"), None, None, None, Some("engine")]),
            1.0,
        );
        toasts.observe(
            reporting([Some("midi"), None, Some("udp"), None, Some("engine")]),
            2.0,
        );
        for toast in toasts.slots.iter_mut().flatten() {
            toast.height = 50.0;
        }
        assert_eq!(toasts.fitting(f32::INFINITY), [0, 2, 4]);
        assert_eq!(toasts.fitting(108.0), [0, 2], "two Toasts and a gap fit");
        assert_eq!(toasts.fitting(49.0), Vec::<usize>::new());
    }
}
