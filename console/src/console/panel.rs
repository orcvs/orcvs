//! The bottom Panel: the Play and Rewind Cells, the typed BPM field, the beat
//! marker, the Tick and Run Clock Readouts, the Output destinations — MIDI, and
//! in its list UDP and OSC — and their status, and optional Diagnostics beneath
//! them.

use std::time::Duration;

use egui::{Color32, FontId, Key, Vec2};
use orcvs::opts::Bpm;
use orcvs::playback::{PlaybackObservation, PlaybackState};

use super::Console;
use super::diagnostics_panel::summary_height;
use crate::console_midi;
use crate::grid_viewport::{glyph_size, snapped_cell};
use crate::midi::{output_readout, selected_destination_name};
use crate::theme::{Appearance, Theme};

/// The height the bottom Panel takes from the window, leaving the rest to the
/// Source Grid. It is the Panel's own minimum, which the Readouts do not exceed.
pub(super) const BOTTOM_PANEL_HEIGHT: f32 = 52.0;

///
/// The height Diagnostics leave the Source: one Row of `cell`s, at the
/// snapped height the Source draws them.
///
fn source_reserve(ui: &egui::Ui, cell: Vec2) -> f32 {
    snapped_cell(cell, ui.ctx().pixels_per_point()).y
}

/// Extra left inset on top of `Frame::side_top_panel`'s inner margin.
pub(super) const BOTTOM_PANEL_LEFT_PAD: i8 = 10;

/// Widget id of the Panel's typed BPM field, so a later pass can find the
/// rectangle it occupied and so focus is the same id the field is shown under.
pub(super) const BPM_FIELD_ID: &str = "bpm";

/// Ids of the UDP and OSC destination fields in the Output list, and of the
/// grid that lines them up.
#[cfg(not(target_arch = "wasm32"))]
const NETWORK_GRID_ID: &str = "network";
#[cfg(not(target_arch = "wasm32"))]
const NETWORK_FIELD_IDS: [&str; 3] = ["network_host", "network_udp_port", "network_osc_port"];
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
/// The Output list's section headings. The web build cannot send, so it has
/// no UDP/OSC section.
pub(super) const OUTPUT_MIDI: &str = "MIDI";
#[cfg(not(target_arch = "wasm32"))]
pub(super) const OUTPUT_NETWORK: &str = "UDP/OSC";

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

///
/// The gap between the Panel's Readout groups, `GROUP_GAP_CELLS` monospace
/// cells, which the top bar's menus share.
///
pub(super) fn readout_group_gap(ui: &egui::Ui) -> f32 {
    panel_readout_gaps(ui).1
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
    // A painted galley has no AccessKit node of its own; name the slot as
    // the Label it reads as.
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), text));
    response
}

///
/// One Source Cell that acts when clicked, holding `glyph` in the Theme's
/// `button` role, or its `button.pressed` role while `pressed`. It starts on
/// the next of the window's Cell columns, so the Panel continues the Grid's
/// lattice.
///
fn cell_button(
    ui: &mut egui::Ui,
    cell: Vec2,
    font_family: &egui::FontFamily,
    theme: &Theme,
    glyph: &str,
    name: &str,
    pressed: bool,
) -> egui::Response {
    let left = ui.cursor().min.x;
    ui.add_space((left / cell.x).ceil() * cell.x - left);
    let (rect, response) = ui.allocate_exact_size(cell, egui::Sense::click());
    let (foreground, background, border) = if pressed {
        (
            theme.button_pressed_foreground,
            theme.button_pressed_background,
            theme.button_pressed_border,
        )
    } else {
        (
            theme.button_foreground,
            theme.button_background,
            theme.button_border,
        )
    };
    let painter = ui.painter();
    painter.rect(
        rect,
        0.0,
        background,
        egui::Stroke::new(theme.button_border_width.points(), border),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        FontId::new(glyph_size(cell), font_family.clone()),
        foreground,
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, name));
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

fn select_all_text(ctx: &egui::Context, id: egui::Id, text: &str) {
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
/// A typed field `width_text` wide that commits on Enter or when focus leaves,
/// not while typing. `commit` is handed the text when it differs from
/// `current` and answers whether it took it; text it did not take, and Escape,
/// put `current` back.
///
fn add_committed_field(
    ui: &mut egui::Ui,
    id: egui::Id,
    width_text: &str,
    current: &str,
    commit: impl FnOnce(&str) -> bool,
) -> egui::Response {
    let size = egui::vec2(
        monospace_width(ui, width_text) + BPM_FIELD_MARGIN.sum().x,
        panel_field_height(ui),
    );
    let ctx = ui.ctx().clone();
    let mut text = ctx
        .data(|data| data.get_temp::<String>(id))
        .unwrap_or_else(|| current.to_owned());
    if !ctx.memory(|memory| memory.has_focus(id)) {
        text = current.to_owned();
    }
    let response = ui.add_sized(
        size,
        egui::TextEdit::singleline(&mut text)
            .id(id)
            .font(egui::TextStyle::Monospace)
            .margin(BPM_FIELD_MARGIN),
    );
    if response.clicked() {
        select_all_text(&ctx, id, &text);
    }
    ctx.data_mut(|data| data.insert_temp(id, text.clone()));
    let escape = ctx.input(|input| input.key_pressed(Key::Escape));
    let enter = response.has_focus() && ctx.input(|input| input.key_pressed(Key::Enter));
    let refused = (response.lost_focus() || enter) && text != current && !commit(&text);
    if escape || refused {
        ctx.data_mut(|data| data.insert_temp(id, current.to_owned()));
    }
    response
}

///
/// Typed BPM only: commits on Enter or when focus leaves, not while dragging.
///
fn add_bpm_field(ui: &mut egui::Ui, bpm: &mut usize) -> (egui::Response, bool) {
    let mut committed = false;
    let response = add_committed_field(
        ui,
        egui::Id::new(BPM_FIELD_ID),
        "000",
        &bpm.to_string(),
        |text| match parse_panel_bpm(text) {
            Some(parsed) => {
                committed = parsed != *bpm;
                *bpm = parsed;
                true
            }
            None => false,
        },
    );
    (response, committed)
}

pub(super) fn bottom_panel_frame(style: &egui::Style) -> egui::Frame {
    let mut frame = egui::Frame::side_top_panel(style);
    frame.inner_margin.left += BOTTOM_PANEL_LEFT_PAD;
    frame
}

/// The edge of a bar that faces the Source.
#[derive(Clone, Copy)]
pub(super) enum BarEdge {
    Top,
    Bottom,
}

///
/// Shows `bar` in `frame` with `separator` along its `edge`, in place of
/// egui's own separator line. As egui places its line, this one lies just
/// outside the frame's outline, its inner edge touching the outline, in room
/// added to the frame's outer margin, so the bar's contents stay clear of it
/// and the bar keeps its size.
///
pub(super) fn show_bar<R>(
    root: &mut egui::Ui,
    bar: egui::Panel,
    mut frame: egui::Frame,
    edge: BarEdge,
    separator: egui::Stroke,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    // A `ChromeWidth` is at most 2 points, so the room fits an `i8`; rounded
    // up, so a fractional width is never painted over the contents.
    let room = separator.width.ceil() as i8;
    match edge {
        BarEdge::Top => frame.outer_margin.top += room,
        BarEdge::Bottom => frame.outer_margin.bottom += room,
    }
    let shown = bar
        .show_separator_line(false)
        .frame(frame)
        .show(root, add_contents);
    // The response covers the outer margin; the outline lies that margin
    // inside it, and the line's inner edge touches the outline.
    let rect = shown.response.rect;
    let half = separator.width / 2.0;
    let y = match edge {
        BarEdge::Top => rect.top() + f32::from(frame.outer_margin.top) - half,
        BarEdge::Bottom => rect.bottom() - f32::from(frame.outer_margin.bottom) + half,
    };
    root.painter().hline(rect.x_range(), y, separator);
    shown.inner
}

impl Console {
    /// Hands this frame's Playback diagnostics to the Playback status.
    pub(super) fn observe_playback_diagnostics(&mut self) {
        let diagnostics = self.orcvs.drain_playback_diagnostics();
        self.playback_status
            .observe(diagnostics, &self.orcvs.playback_observation());
    }

    ///
    /// Shows the bottom Panel from `observation`, with the Run Clock at
    /// `run_clock`: the one sample of it this Render Frame paints and times its
    /// next repaint by.
    ///
    /// Static: no resize handle, no drag. P and R do what Space and Command
    /// Shift R do. BPM is a TextEdit: click to type;
    /// Enter or leaving the field commits. `**` is the beat, `//` while
    /// Playback is stopped. Tick and Run Clock are the engine's published
    /// Readouts. Destination is chosen from the ComboBox; Scan asks the engine
    /// to discover again. There is no periodic polling.
    /// Returns the reserved Diagnostics area to fill after Source layout.
    /// Diagnostics take no more height than leaves the Source
    /// `source_reserve`; the readouts scroll within what they get.
    ///
    pub(super) fn show_panel(
        &mut self,
        root: &mut egui::Ui,
        observation: &PlaybackObservation,
        run_clock: Duration,
        appearance: Appearance,
    ) -> Option<egui::Ui> {
        let frame = bottom_panel_frame(root.style().as_ref());
        let separation = 2.0 * root.spacing().item_spacing.y;
        let diagnostics_room = root.available_height()
            - BOTTOM_PANEL_HEIGHT
            - separation
            - source_reserve(root, self.source_view.cell);
        let diagnostics_height = self
            .diagnostics_open
            .then(|| summary_height(root).min(diagnostics_room.max(0.0)));
        let extra_height = diagnostics_height.map_or(0.0, |height| height + separation);
        let height = BOTTOM_PANEL_HEIGHT + extra_height;
        let separator = crate::style::bar_separator(self.themes.presented(appearance));
        let bottom_panel = egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .exact_size(height);
        show_bar(root, bottom_panel, frame, BarEdge::Top, separator, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(
                    ui.available_width(),
                    (ui.available_height() - extra_height).max(0.0),
                ),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let (label_value_gap, entry_gap) = panel_readout_gaps(ui);
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let cell = snapped_cell(self.source_view.cell, ui.ctx().pixels_per_point());
                    let theme = self.themes.presented(appearance);
                    let playing = observation.state == PlaybackState::Playing;
                    let font = &self.font_family;
                    if cell_button(ui, cell, font, theme, "P", "Play", playing).clicked() {
                        self.orcvs.toggle_playback();
                    }
                    ui.add_space(cell.x);
                    if cell_button(ui, cell, font, theme, "R", "Rewind", false).clicked() {
                        self.orcvs.rewind();
                    }
                    ui.add_space(entry_gap);
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
            diagnostics_height.map(|height| {
                ui.add(egui::Separator::default().horizontal().spacing(0.0));
                // Reserve the readouts before the Source takes its space;
                // fill them once that Source's current geometry is known.
                let (_, rect) = ui.allocate_space(egui::vec2(ui.available_width(), height));
                let mut diagnostics =
                    ui.new_child(egui::UiBuilder::new().id_salt("diagnostics").max_rect(rect));
                diagnostics.set_clip_rect(diagnostics.clip_rect().intersect(rect));
                diagnostics
            })
        })
    }

    ///
    /// The Output ComboBox: one section per output type this build can use,
    /// MIDI's Scan and destinations, then the UDP and OSC destination.
    ///
    /// The MIDI section lists the destinations the selection already holds,
    /// borrowed for the frame. Scan, and opening the list while it is empty,
    /// ask for a fresh discovery once the ComboBox is done with that borrow;
    /// the list shows what that discovery found from the next frame.
    ///
    fn show_destination(&mut self, ui: &mut egui::Ui) {
        let selected_id = self.midi.selected_destination_id();
        let destinations = self.midi.destinations();
        let midi = selected_destination_name(destinations, selected_id.as_ref());
        #[cfg(not(target_arch = "wasm32"))]
        let host = Some(self.network.text(crate::network::NetworkField::Host));
        #[cfg(target_arch = "wasm32")]
        let host: Option<String> = None;
        let readout = output_readout(midi, host.as_deref());
        let mut selected = None;
        let mut scan = false;
        #[cfg_attr(
            target_arch = "wasm32",
            expect(
                unused_mut,
                reason = "the web build cannot send, so its list holds no network fields to change"
            )
        )]
        let mut network_changed = false;
        let opened = ui
            .scope(|ui| {
                apply_panel_field_spacing(ui);
                // The field draws no arrow, so it reserves no icon slot. The popup
                // builds its own Ui from the context's style, so the list keeps
                // egui's icon spacing.
                ui.spacing_mut().icon_width = 0.0;
                ui.spacing_mut().icon_spacing = 0.0;
                egui::ComboBox::from_id_salt(DESTINATION_COMBO_ID)
                    .selected_text(
                        egui::RichText::new(readout).text_style(egui::TextStyle::Monospace),
                    )
                    .width(OUTPUT_READOUT_WIDTH)
                    .icon(|_ui, _rect, _visuals, _is_open| {})
                    // The UDP and OSC fields in the list are typed into, so a click
                    // closes it only outside them, or on Scan or a destination, which
                    // close it themselves.
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .show_ui(ui, |ui| {
                        if console_midi::AVAILABLE {
                            panel_label(ui, OUTPUT_MIDI);
                            if ui.button(OUTPUT_SCAN).clicked() {
                                scan = true;
                                ui.close();
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
                                    if ui
                                        .push_id(destination.id.as_str(), |ui| {
                                            ui.selectable_label(
                                                selected_id.as_ref() == Some(&destination.id),
                                                destination.name.as_str(),
                                            )
                                            .clicked()
                                        })
                                        .inner
                                    {
                                        selected = Some(destination.id.clone());
                                        ui.close();
                                    }
                                }
                            }
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if console_midi::AVAILABLE {
                                ui.separator();
                            }
                            panel_label(ui, OUTPUT_NETWORK);
                            network_changed = network_fields(ui, &mut self.network);
                        }
                    })
                    .response
                    .clicked()
            })
            .inner;
        scan |= opened && console_midi::AVAILABLE && destinations.is_empty();
        if network_changed {
            self.orcvs
                .set_network_destination(self.network.destination());
            self.playback_status.network_destination_changed();
        }
        if scan {
            self.midi.refresh_destinations();
        }
        if let Some(id) = selected.as_ref() {
            self.midi.select_destination(id);
        }
        self.midi.observe_frame();
        if self.midi.awaiting_install() {
            ui.ctx().request_repaint();
        }
        for status in [
            self.midi.status(),
            self.playback_status.midi(),
            self.network.status(),
            self.playback_status.network(),
            self.playback_status.engine(),
        ]
        .into_iter()
        .flatten()
        {
            ui.add_space(readout_group_gap(ui));
            ui.colored_label(ui.visuals().error_fg_color, status);
        }
    }
}

///
/// The UDP and OSC destination as the Output list's UDP/OSC section shows it:
/// a field for the host and one for each port, each taking what
/// is typed into it on Enter or when focus leaves. Answers whether the
/// destination changed.
///
#[cfg(not(target_arch = "wasm32"))]
fn network_fields(ui: &mut egui::Ui, network: &mut crate::network::NetworkSetting) -> bool {
    use crate::network::NetworkField;

    let fields = [
        ("Host", NetworkField::Host, "000.000.000.000"),
        ("UDP", NetworkField::UdpPort, "00000"),
        ("OSC", NetworkField::OscPort, "00000"),
    ];
    let mut changed = false;
    egui::Grid::new(NETWORK_GRID_ID).show(ui, |ui| {
        for ((label, field, width_text), id) in fields.into_iter().zip(NETWORK_FIELD_IDS) {
            panel_label(ui, label);
            let current = network.text(field);
            add_committed_field(ui, egui::Id::new(id), width_text, &current, |text| {
                let committed = network.commit(field, text);
                changed |= committed;
                committed
            });
            ui.end_row();
        }
    });
    changed
}
