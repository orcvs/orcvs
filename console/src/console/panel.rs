//! The bottom Panel: the typed BPM field, the beat marker, the Tick and Run
//! Clock Readouts, and the MIDI Output destination and its status.

use std::time::Duration;

use egui::{Color32, FontId, Key};
use orcvs::opts::Bpm;
use orcvs::playback::{PlaybackObservation, PlaybackState};

use super::Console;
use crate::midi::destination_presentation;
use crate::native_midi;

/// The height the bottom Panel takes from the window, leaving the rest to the
/// Source Grid. It is the Panel's own minimum, which the Readouts do not exceed.
pub(super) const BOTTOM_PANEL_HEIGHT: f32 = 52.0;
/// Extra left inset on top of `Frame::side_top_panel`'s inner margin.
pub(super) const BOTTOM_PANEL_LEFT_PAD: i8 = 10;

/// Widget id of the Panel's typed BPM field, so a later pass can find the
/// rectangle it occupied and so focus is the same id the field is shown under.
pub(super) const BPM_FIELD_ID: &str = "bpm";

/// Widget id of the Panel's destination ComboBox, so focus is the same id the
/// ComboBox is shown under and `event_handler` can skip keys while it has it.
const DESTINATION_COMBO_ID: &str = "destination";
/// Tick copy is this many digits, zero-padded, so the field does not change width.
const TICK_DIGITS: usize = 5;
/// Flash next to BPM when the engine publishes a beat.
pub(super) const BEAT_MARKER: &str = "**";
/// Marker next to BPM while Playback is stopped.
const REST_MARKER: &str = "//";
/// How many points to pull each label toward its value, relative to one monospace cell.
const LABEL_VALUE_TIGHTEN: f32 = 2.0;
/// Monospace cells between Readout groups, so C is not as close to T's value as to its own.
const GROUP_GAP_CELLS: f32 = 3.0;
/// Slot the Output readout occupies so a shorter device name is not truncated early.
const OUTPUT_READOUT_WIDTH: f32 = 196.0;
/// Menu action that asks the engine to discover output destinations again.
pub(super) const OUTPUT_SCAN: &str = "Scan";

///
/// Run Clock copy for a wall-clock Duration: `mm:ss` through 59:59 inclusive,
/// then `h:mm:ss`. Hours are unpadded; minutes and seconds always occupy two
/// digits. Tick copy is zero-padded to [`TICK_DIGITS`].
///
pub(super) fn format_run_clock(elapsed: Duration) -> String {
    let total_secs = elapsed.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours == 0 {
        format!("{minutes:02}:{seconds:02}")
    } else {
        format!("{hours}:{minutes:02}:{seconds:02}")
    }
}

pub(super) fn format_tick(tick: u64) -> String {
    format!("{tick:0width$}", width = TICK_DIGITS)
}

pub(super) fn format_beat_marker(state: PlaybackState, on_beat: bool) -> &'static str {
    match (state, on_beat) {
        (PlaybackState::Playing, true) => BEAT_MARKER,
        (PlaybackState::Playing, false) => "",
        (PlaybackState::Stopped, _) => REST_MARKER,
    }
}

fn panel_readout_gaps(ui: &egui::Ui) -> (f32, f32) {
    let cell = monospace_width(ui, "0");
    (
        (cell - LABEL_VALUE_TIGHTEN).max(0.0),
        cell * GROUP_GAP_CELLS,
    )
}

pub(super) fn panel_monospace_id(ui: &egui::Ui) -> FontId {
    ui.style()
        .text_styles
        .get(&egui::TextStyle::Monospace)
        .cloned()
        .unwrap_or_else(|| FontId::monospace(12.0))
}

pub(super) fn monospace_width(ui: &egui::Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), panel_monospace_id(ui), Color32::WHITE)
        .size()
        .x
}

///
/// A monospace Readout that keeps a fixed slot, so a wider value does not
/// shove the widgets after it.
///
pub(super) fn reserved_monospace(ui: &mut egui::Ui, text: &str, width: f32) -> egui::Response {
    let font_id = panel_monospace_id(ui);
    let height = ui
        .spacing()
        .interact_size
        .y
        .max(ui.text_style_height(&egui::TextStyle::Monospace));
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font_id, ui.visuals().text_color());
    let pos = egui::Align2::LEFT_CENTER
        .anchor_size(rect.left_center(), galley.size())
        .min;
    ui.painter().galley(pos, galley, ui.visuals().text_color());
    response
}

fn panel_label(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Label::new(egui::RichText::new(text).text_style(egui::TextStyle::Monospace))
            .selectable(false),
    )
}

/// Inner padding of the typed BPM field. Wider than TextEdit's default
/// `Margin::symmetric(4, 2)` so three digits sit inside a roomier box.
pub(super) const BPM_FIELD_MARGIN: egui::Margin = egui::Margin::symmetric(8, 4);
/// The Panel BPM range is 1..=999.
const PANEL_BPM_MIN: usize = 1;
const PANEL_BPM_MAX: usize = 999;

fn parse_panel_bpm(text: &str) -> Option<usize> {
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    text.parse()
        .ok()
        .filter(|value| (PANEL_BPM_MIN..=PANEL_BPM_MAX).contains(value))
}

fn select_all_bpm_text(ctx: &egui::Context, id: egui::Id, text: &str) {
    let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::default(),
            egui::text::CCursor::new(text.chars().count()),
        )));
    state.store(ctx, id);
}

fn panel_field_height(ui: &egui::Ui) -> f32 {
    ui.text_style_height(&egui::TextStyle::Monospace) + BPM_FIELD_MARGIN.sum().y
}

///
/// ComboBox chrome reads [`Spacing::button_padding`](egui::style::Spacing::button_padding); match the BPM TextEdit margin.
///
fn apply_panel_field_spacing(ui: &mut egui::Ui) {
    ui.spacing_mut().button_padding = egui::vec2(BPM_FIELD_MARGIN.leftf(), BPM_FIELD_MARGIN.topf());
    ui.spacing_mut().interact_size.y = panel_field_height(ui);
}

///
/// Typed BPM only: commits on Enter or when focus leaves, not while dragging.
///
fn add_bpm_field(ui: &mut egui::Ui, bpm: &mut usize) -> (egui::Response, bool) {
    let id = egui::Id::new(BPM_FIELD_ID);
    let size = egui::vec2(
        monospace_width(ui, "000") + BPM_FIELD_MARGIN.sum().x,
        panel_field_height(ui),
    );
    let ctx = ui.ctx().clone();
    let mut text = ctx
        .data(|data| data.get_temp::<String>(id))
        .unwrap_or_else(|| bpm.to_string());
    if !ctx.memory(|memory| memory.has_focus(id)) {
        text = bpm.to_string();
    }
    let response = ui.add_sized(
        size,
        egui::TextEdit::singleline(&mut text)
            .id(id)
            .font(egui::TextStyle::Monospace)
            .margin(BPM_FIELD_MARGIN),
    );
    if response.clicked() {
        select_all_bpm_text(&ctx, id, &text);
    }
    ctx.data_mut(|data| data.insert_temp(id, text.clone()));
    let escape = ctx.input(|input| input.key_pressed(Key::Escape));
    let enter = response.has_focus() && ctx.input(|input| input.key_pressed(Key::Enter));
    let mut committed = false;
    if escape {
        text = bpm.to_string();
        ctx.data_mut(|data| data.insert_temp(id, text));
    } else if response.lost_focus() || enter {
        match parse_panel_bpm(&text) {
            Some(parsed) if parsed != *bpm => {
                *bpm = parsed;
                committed = true;
            }
            Some(_) => {}
            None => {
                text = bpm.to_string();
                ctx.data_mut(|data| data.insert_temp(id, text));
            }
        }
    }
    (response, committed)
}

fn bottom_panel_frame(style: &egui::Style) -> egui::Frame {
    let mut frame = egui::Frame::side_top_panel(style);
    frame.inner_margin.left += BOTTOM_PANEL_LEFT_PAD;
    frame
}

impl Console {
    ///
    /// Hands this frame's Playback diagnostics to the Panel's MIDI status, or,
    /// without a native backend, to the developer console.
    ///
    pub(super) fn observe_playback_diagnostics(&mut self) {
        let playback_diagnostics = self.orcvs.drain_playback_diagnostics();
        if native_midi::AVAILABLE {
            self.midi.observe_diagnostics(playback_diagnostics);
        } else {
            // Without a native backend the destination ComboBox is disabled
            // and Refresh is hidden, so a refused connect has nowhere on the
            // Panel to land; the developer console is the only channel a
            // failure has.
            crate::diagnostics::report_playback_failures(&playback_diagnostics);
        }
    }

    ///
    /// Shows the bottom Panel from `observation`, with the Run Clock at
    /// `run_clock`: the one sample of it this Render Frame paints and times its
    /// next repaint by.
    ///
    /// Static: no resize handle, no drag. BPM is a TextEdit: click to type;
    /// Enter or leaving the field commits. `**` is the beat, `//` while
    /// Playback is stopped. Tick and Run Clock are the engine's published
    /// Readouts. Destination is chosen from the ComboBox; Scan asks the engine
    /// to discover again. There is no periodic polling.
    ///
    pub(super) fn show_panel(
        &mut self,
        root: &mut egui::Ui,
        observation: &PlaybackObservation,
        run_clock: Duration,
    ) {
        egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .min_size(BOTTOM_PANEL_HEIGHT)
            .frame(bottom_panel_frame(root.style().as_ref()))
            .show(root, |ui| {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        let (label_value_gap, entry_gap) = panel_readout_gaps(ui);
                        ui.spacing_mut().item_spacing.x = 0.0;
                        panel_label(ui, "B");
                        ui.add_space(label_value_gap);
                        let mut bpm = self.orcvs.bpm().beats_per_minute();
                        // The field is shown under `BPM_FIELD_ID`, the id
                        // `Console::new` already gave `bpm_widget_id`.
                        let (_, committed) = add_bpm_field(ui, &mut bpm);
                        if committed
                            && let Some(next) = Bpm::new(bpm)
                            && next != self.orcvs.bpm()
                        {
                            self.orcvs.set_bpm(next);
                        }
                        ui.add_space(label_value_gap);
                        let beat_text = format_beat_marker(observation.state, observation.on_beat);
                        let beat_width = monospace_width(ui, BEAT_MARKER);
                        reserved_monospace(ui, beat_text, beat_width);
                        ui.add_space(entry_gap);
                        panel_label(ui, "T");
                        ui.add_space(label_value_gap);
                        let tick_text = format_tick(observation.tick.get());
                        let tick_width = monospace_width(ui, "00000");
                        reserved_monospace(ui, &tick_text, tick_width);
                        ui.add_space(entry_gap);
                        panel_label(ui, "C");
                        ui.add_space(label_value_gap);
                        let clock_text = format_run_clock(run_clock);
                        let clock_width =
                            monospace_width(ui, "00:00").max(monospace_width(ui, &clock_text));
                        reserved_monospace(ui, &clock_text, clock_width);
                        ui.add_space(entry_gap);
                        panel_label(ui, "O");
                        ui.add_space(label_value_gap);
                        self.show_destination(ui);
                    },
                );
            });
    }

    ///
    /// The MIDI Output destination ComboBox, then the MIDI status.
    ///
    /// The ComboBox lists the destinations the selection already holds,
    /// borrowed for the frame. Scan, and opening an empty list, ask for a
    /// fresh discovery once the ComboBox is done with that borrow; the list
    /// shows what that discovery found from the next frame.
    ///
    fn show_destination(&mut self, ui: &mut egui::Ui) {
        self.midi.auto_select_first_if_unselected();
        let selected_id = self.midi.selected_destination_id();
        let destinations = self.midi.destinations();
        let presentation = destination_presentation(destinations, selected_id.as_ref());
        let (scan, selected) = ui
            .add_enabled_ui(presentation.enabled, |ui| {
                apply_panel_field_spacing(ui);
                let mut selected = selected_id.clone();
                let mut scan = false;
                let combo_response = egui::ComboBox::from_id_salt(DESTINATION_COMBO_ID)
                    .selected_text(
                        egui::RichText::new(presentation.selected_text)
                            .text_style(egui::TextStyle::Monospace),
                    )
                    .width(OUTPUT_READOUT_WIDTH)
                    .icon(|_ui, _rect, _visuals, _is_open| {})
                    .show_ui(ui, |ui| {
                        if presentation.show_refresh {
                            scan |= ui.button(OUTPUT_SCAN).clicked();
                            ui.separator();
                        }
                        if destinations.is_empty() {
                            ui.add_enabled_ui(false, |ui| {
                                let _ = ui.selectable_label(
                                    true,
                                    egui::RichText::new(crate::midi::OUTPUT_NONE)
                                        .text_style(egui::TextStyle::Monospace),
                                );
                            });
                        } else {
                            for destination in destinations {
                                ui.selectable_value(
                                    &mut selected,
                                    Some(destination.id.clone()),
                                    destination.name.as_str(),
                                );
                            }
                        }
                    });
                scan |= combo_response.response.clicked()
                    && presentation.show_refresh
                    && destinations.is_empty();
                (scan, selected)
            })
            .inner;
        if scan {
            self.midi.refresh_destinations();
        }
        if selected != selected_id
            && let Some(id) = selected.as_ref()
        {
            self.midi.select_destination(id);
        }
        if let Some(status) = self.midi.status() {
            ui.colored_label(ui.visuals().error_fg_color, status);
        }
    }
}
