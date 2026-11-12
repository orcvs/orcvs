//! The Toasts: each message the outputs report, floating in the Source's
//! top-right corner beneath the menu bar, clear of the Minimap.

use egui::text::LayoutJob;
use egui::{Align2, Color32, Rect, Sense, Stroke, TextStyle};

use super::Console;
use super::panel::panel_monospace_id;
use crate::midi::MidiDeviceSelection;
use crate::network::NetworkSetting;
use crate::playback_status::PlaybackStatus;
use crate::theme::Appearance;

/// How many [`Reporter`]s there are.
const REPORTERS: usize = 5;
/// Points between a Toast and the Source's edges.
const INSET: f32 = 16.0;
/// Points between two Toasts in the stack.
const GAP: f32 = 8.0;
/// A Toast's width in points, in a Source wide enough to hold it.
const WIDTH: f32 = 380.0;
/// The lines a message wraps to before it is cut with an ellipsis.
const MESSAGE_ROWS: usize = 3;
/// The seconds a Toast takes to fade out once its time is up.
const FADE: f64 = 0.5;

///
/// What reports a message, each with a Toast of its own. The Toasts stack in
/// the order these are declared.
///
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum Reporter {
    /// The MIDI selection.
    MidiSelection,
    /// A MIDI output failure.
    MidiOutput,
    /// A UDP or OSC destination the Output list refused.
    NetworkDestination,
    /// A failed UDP or OSC send.
    NetworkSend,
    /// An engine failure.
    Engine,
}

impl Reporter {
    /// The output a Toast names as the one its message belongs to.
    fn label(self) -> &'static str {
        match self {
            Self::MidiSelection | Self::MidiOutput => "MIDI",
            Self::NetworkDestination | Self::NetworkSend => "UDP/OSC",
            Self::Engine => "ENGINE",
        }
    }
}

/// What each [`Reporter`] has to say, if anything.
pub(super) fn standing_messages<'a>(
    midi: &'a MidiDeviceSelection,
    network: &'a NetworkSetting,
    playback_status: &'a PlaybackStatus,
) -> [(Reporter, Option<&'a str>); REPORTERS] {
    [
        (Reporter::MidiSelection, midi.status()),
        (Reporter::MidiOutput, playback_status.midi()),
        (Reporter::NetworkDestination, network.status()),
        (Reporter::NetworkSend, playback_status.network()),
        (Reporter::Engine, playback_status.engine()),
    ]
}

///
/// One [`Reporter`]'s message, from the frame it was raised until the Reporter
/// has no message or a different one.
///
struct Toast {
    reporter: Reporter,
    text: String,
    /// When it was raised, in egui's input time, so the stack drops the oldest.
    raised: f64,
    dismissed: bool,
    /// Its height in points as last shown, so the stack can fit above the
    /// Minimap before it is laid out. A new Toast's is unknown, but egui
    /// draws a new `Area` invisibly on its first frame, only measuring it.
    height: f32,
}

impl Toast {
    ///
    /// How opaque it is at `now`: whole for `seconds` after it was raised,
    /// then fading over [`FADE`], and gone once dismissed.
    ///
    fn opacity(&self, now: f64, seconds: f64) -> f32 {
        if self.dismissed {
            return 0.0;
        }
        let faded = (now - self.raised - seconds) / FADE;
        (1.0 - faded).clamp(0.0, 1.0) as f32
    }
}

/// The Toast for each [`Reporter`], in declaration order.
pub(super) struct Toasts {
    slots: [Option<Toast>; REPORTERS],
    /// How many seconds a Toast shows before it fades.
    seconds: f64,
}

impl Toasts {
    /// No Toasts yet, each to show for `seconds` once raised.
    pub(super) fn new(seconds: u8) -> Self {
        Self {
            slots: Default::default(),
            seconds: f64::from(seconds),
        }
    }

    ///
    /// Takes the messages standing at `now`. A Reporter with no message, or a
    /// different one, forgets its Toast, dismissed or not; a new message raises
    /// a new Toast.
    ///
    fn observe(&mut self, messages: [(Reporter, Option<&str>); REPORTERS], now: f64) {
        for (reporter, message) in messages {
            let slot = &mut self.slots[reporter as usize];
            match message {
                None => *slot = None,
                Some(text) if slot.as_ref().is_some_and(|toast| toast.text == text) => {}
                Some(text) => {
                    *slot = Some(Toast {
                        reporter,
                        text: text.to_owned(),
                        raised: now,
                        dismissed: false,
                        height: 0.0,
                    });
                }
            }
        }
    }

    /// Dismisses every standing Toast, as its `×` would.
    pub(super) fn dismiss_all(&mut self) {
        for toast in self.slots.iter_mut().flatten() {
            toast.dismissed = true;
        }
    }

    ///
    /// The Reporters whose Toasts show at `now`, in order, in a stack no taller
    /// than `room`: the oldest go first until the rest fit.
    ///
    fn fitting(&self, room: f32, now: f64) -> Vec<Reporter> {
        let mut shown: Vec<&Toast> = self
            .slots
            .iter()
            .flatten()
            .filter(|toast| toast.opacity(now, self.seconds) > 0.0)
            .collect();
        let height =
            |shown: &[&Toast]| shown.iter().map(|toast| toast.height + GAP).sum::<f32>() - GAP;
        while height(&shown) > room
            && let Some(oldest) =
                (0..shown.len()).min_by(|&a, &b| shown[a].raised.total_cmp(&shown[b].raised))
        {
            shown.remove(oldest);
        }
        shown.into_iter().map(|toast| toast.reporter).collect()
    }
}

impl Console {
    ///
    /// Shows a Toast for each standing message in the top-right corner of
    /// `source`, the Source's rect, down to `floor`, the Minimap's top edge or
    /// the Source's bottom. Each is its own `Area`, so it takes the pointer
    /// over its own frame and nowhere else, and its `×` senses a click but not
    /// focus, so keyboard input stays with the Source. It asks for a repaint
    /// when the next Toast is due to fade, and every frame while one fades.
    ///
    pub(super) fn show_toasts(
        &mut self,
        ctx: &egui::Context,
        source: Rect,
        floor: f32,
        appearance: Appearance,
    ) {
        let label_colour = self.themes.presented(appearance).diagnostic_foreground;
        let now = ctx.input(|input| input.time);
        let Self {
            toasts,
            midi,
            network,
            playback_status,
            ..
        } = self;
        toasts.observe(standing_messages(midi, network, playback_status), now);
        let top = source.top() + INSET;
        let width = WIDTH.min(source.width() - 2.0 * INSET);
        let mut y = top;
        let seconds = toasts.seconds;
        for reporter in toasts.fitting(floor - INSET - top, now) {
            let toast = toasts.slots[reporter as usize]
                .as_mut()
                .expect("a fitting Toast stands");
            let opacity = toast.opacity(now, seconds);
            if opacity < 1.0 {
                ctx.request_repaint();
            } else {
                let fades_in = toast.raised + seconds - now;
                ctx.request_repaint_after(std::time::Duration::from_secs_f64(fades_in.max(0.0)));
            }
            let shown = egui::Area::new(egui::Id::unique(("toast", reporter)))
                .pivot(Align2::RIGHT_TOP)
                .fixed_pos(egui::pos2(source.right() - INSET, y))
                .constrain(false)
                .show(ctx, |ui| {
                    ui.multiply_opacity(opacity);
                    show_toast(ui, width, label_colour, toast.reporter.label(), &toast.text)
                });
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
/// One Toast `width` points wide: `label` in `label_colour`, a `×` that answers whether it
/// was clicked, and `message` in the error colour, wrapped to
/// [`MESSAGE_ROWS`] lines and then cut, whole on hover.
///
fn show_toast(
    ui: &mut egui::Ui,
    width: f32,
    label_colour: Color32,
    label: &str,
    message: &str,
) -> bool {
    let error = ui.visuals().error_fg_color;
    let frame = egui::Frame::popup(ui.style()).stroke(Stroke::new(1.0, error));
    frame
        .show(ui, |ui| {
            ui.set_width(width - frame.total_margin().sum().x);
            let dismissed = ui
                .horizontal(|ui| {
                    let label = egui::RichText::new(label)
                        .font(panel_monospace_id(ui))
                        .color(label_colour);
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
    use super::Reporter::{
        self, Engine, MidiOutput, MidiSelection, NetworkDestination, NetworkSend,
    };
    use super::{REPORTERS, Toasts};

    const SECONDS: u8 = 5;

    const STANDING: [(Reporter, &str); 1] = [(Engine, "clock stopped")];

    /// Every Reporter's message: those `messages` names, and none for the rest.
    fn reporting<'a>(messages: &[(Reporter, &'a str)]) -> [(Reporter, Option<&'a str>); REPORTERS] {
        [
            MidiSelection,
            MidiOutput,
            NetworkDestination,
            NetworkSend,
            Engine,
        ]
        .map(|reporter| {
            let message = messages.iter().find(|(named, _)| *named == reporter);
            (reporter, message.map(|(_, text)| *text))
        })
    }

    fn dismiss(toasts: &mut Toasts, reporter: Reporter) {
        if let Some(toast) = toasts.slots[reporter as usize].as_mut() {
            toast.dismissed = true;
        }
    }

    #[test]
    fn a_message_raises_a_toast_that_stays_while_it_stands() {
        let mut toasts = Toasts::new(SECONDS);
        toasts.observe(reporting(&STANDING), 0.0);
        toasts.observe(reporting(&STANDING), 1.0);
        assert_eq!(toasts.fitting(f32::INFINITY, 3.0), [Engine]);
    }

    #[test]
    fn a_dismissed_toast_returns_only_after_its_reporter_clears() {
        let mut toasts = Toasts::new(SECONDS);
        toasts.observe(reporting(&STANDING), 0.0);
        dismiss(&mut toasts, Engine);
        toasts.observe(reporting(&STANDING), 1.0);
        assert!(
            toasts.fitting(f32::INFINITY, 3.0).is_empty(),
            "the same message"
        );

        toasts.observe(reporting(&[]), 2.0);
        toasts.observe(reporting(&STANDING), 3.0);
        assert_eq!(
            toasts.fitting(f32::INFINITY, 3.0),
            [Engine],
            "after a clear frame"
        );
    }

    #[test]
    fn a_different_message_raises_a_new_toast() {
        let mut toasts = Toasts::new(SECONDS);
        toasts.observe(reporting(&STANDING), 0.0);
        dismiss(&mut toasts, Engine);
        toasts.observe(reporting(&[(Engine, "cannot start")]), 1.0);
        assert_eq!(toasts.fitting(f32::INFINITY, 3.0), [Engine]);
    }

    #[test]
    fn a_toast_fades_once_its_seconds_have_passed_and_stays_faded() {
        let mut toasts = Toasts::new(SECONDS);
        toasts.observe(reporting(&STANDING), 0.0);
        assert_eq!(
            toasts.fitting(f32::INFINITY, 4.9),
            [Engine],
            "within its seconds"
        );
        assert!(toasts.fitting(f32::INFINITY, 6.0).is_empty(), "faded");
        toasts.observe(reporting(&STANDING), 7.0);
        assert!(
            toasts.fitting(f32::INFINITY, 7.0).is_empty(),
            "the same message raised the faded Toast again"
        );
    }

    #[test]
    fn a_stack_too_tall_for_its_room_drops_its_oldest() {
        let mut toasts = Toasts::new(SECONDS);
        let engine = (Engine, "engine");
        let midi = (MidiSelection, "midi");
        let udp = (NetworkDestination, "udp");
        toasts.observe(reporting(&[engine]), 0.0);
        toasts.observe(reporting(&[midi, engine]), 1.0);
        toasts.observe(reporting(&[midi, udp, engine]), 2.0);
        for toast in toasts.slots.iter_mut().flatten() {
            toast.height = 50.0;
        }
        assert_eq!(
            toasts.fitting(f32::INFINITY, 3.0),
            [MidiSelection, NetworkDestination, Engine]
        );
        assert_eq!(
            toasts.fitting(108.0, 3.0),
            [MidiSelection, NetworkDestination],
            "two Toasts and a gap fit"
        );
        assert_eq!(toasts.fitting(49.0, 3.0), Vec::<Reporter>::new());
    }
}
