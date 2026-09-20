use std::sync::Arc;
use std::time::Duration;

use egui::{
    Color32, CornerRadius, CursorIcon, Event, EventFilter, FontId, Key, PointerButton, Pos2, Rect,
    Sense, Shape, Stroke, StrokeKind, Vec2, emath::GuiRounding as _, emath::TSTransform,
    epaint::RectShape, text::Galley,
};

use crate::cursor_effects::{
    CursorEffectAnimation, CursorEffectSample, CursorEffectSettings, DEFAULT_CURSOR_COLOUR,
    cursor_effect_shapes, effect_bounds,
};
use crate::function_reference::function_reference;
use crate::grid_viewport::{CELL_SIZE, GridViewport, presented_grid, snapped_cell_side};
use crate::midi::{MidiDeviceSelection, destination_presentation};
use crate::native_midi::{self, NativeMidiBackend};
use crate::paint::{FramePaint, Paint};
use crate::persistence::starting_source;
use crate::readout_deadline::until_next;
use crate::source_paint::SourcePaintSettings;
use crate::style::style;
use orcvs::{
    app::{Arrow, InputEvent, InputKey, Orcvs},
    grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT, Grid, Position},
    opts::{Bpm, DEFAULT_FONT_SIZE},
    playback::{PlaybackStartError, PlaybackState},
    render_frame::RenderFrame,
};

const GRID_LINE_WIDTH: f32 = 0.5;
const SECTOR_LINE_WIDTH: f32 = 0.75;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 2.0;
///
/// How far past the console's edge, in points, a Region drag's pointer has to
/// be for each point the Source View scrolls a frame after it. A pointer four
/// Cells out scrolls one Cell a frame, which is the most a frame scrolls.
///
const EDGE_SCROLL_REACH: f32 = 4.0;

///
/// The step the scale is quantised to before it reaches a [`FontId`].
///
/// # Why this is an atlas budget, not a cache-hit rate
///
/// A Glyph is laid out at the size it is drawn at, so the scale has to reach
/// the font size. A *continuous* scale would reach it as a fresh size per
/// Render Frame, and epaint rasterises a fresh glyph set per distinct size —
/// `FontImpl::glyph_info` scales by `font_size * pixels_per_point` and rounds
/// nothing (`epaint-0.36.1/src/text/font.rs:567`). `subpixel_binning` is on by
/// default (`epaint-0.36.1/src/text/mod.rs:62`) and renders each glyph at up to
/// four fractional offsets, so a zoom sweep across `N` sizes costs up to
/// `N x alphabet x 4` rasters into one atlas.
///
/// That is the budget, because the atlas is not merely wasted when it fills:
/// `Fonts::begin_pass` replaces the whole `FontsImpl` — a new atlas with empty
/// glyph caches — as soon as `atlas.fill_ratio()` passes 0.8
/// (`epaint-0.36.1/src/text/fonts.rs:734-748`), restarting glyph rasterisation
/// mid-session for every size already paid for.
///
/// At a step of an eighth, the zoom range `MIN_ZOOM..=MAX_ZOOM` holds fifteen
/// distinct scales, so a viewer who sweeps that range spends at most
/// `15 x 94 x 4 = 5,640` rasters — roughly two megapixels of a 2048-square
/// atlas at one device pixel per point, which stays inside the fill ratio.
/// Both zoom limits and the Source's own scale are exact multiples of the step,
/// so the default window and either end of the range land on it rather than
/// beside it.
///
/// Fifteen is the whole range, not a floor a large window can widen: Zoom is
/// a stated step between [`MIN_ZOOM`] and [`MAX_ZOOM`], and no window size
/// changes the Cell size. The atlas budget ADR 0038 and ADR 0040 state is now
/// the whole range rather than its floor.
///
/// The step costs a Glyph at most an eighth of the Source's Cell scale in size,
/// taken downwards so a Glyph is never larger than its share of the Cell — see
/// [`glyph_scale`], which states why the rounding goes that way. It is still
/// strictly sharper than what it replaces: a Scene bilinearly resamples one
/// rasterised size at *every* zoom.
///
const GLYPH_SCALE_STEP: f32 = 0.125;

/// The height the top panel takes from the window, leaving the rest to the
/// console. It is the panel's own minimum, which the menu bar does not exceed.
const TOP_PANEL_HEIGHT: f32 = 32.0;

#[cfg(target_arch = "wasm32")]
fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|query| query.matches())
}

#[cfg(target_os = "macos")]
fn prefers_reduced_motion() -> bool {
    std::process::Command::new("defaults")
        .args(["read", "com.apple.universalaccess", "reduceMotion"])
        .output()
        .is_ok_and(|output| output.status.success() && output.stdout.starts_with(b"1"))
}

#[cfg(target_os = "linux")]
fn prefers_reduced_motion() -> bool {
    std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "enable-animations"])
        .output()
        .is_ok_and(|output| output.status.success() && output.stdout.starts_with(b"false"))
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "macos"),
    not(target_os = "linux")
))]
fn prefers_reduced_motion() -> bool {
    false
}

/// The height the bottom Panel takes from the window, leaving the rest to the
/// Source Grid. It is the Panel's own minimum, which the Readouts do not exceed.
const BOTTOM_PANEL_HEIGHT: f32 = 52.0;
/// Extra left inset on top of `Frame::side_top_panel`'s inner margin.
const BOTTOM_PANEL_LEFT_PAD: i8 = 10;

/// Widget id of the Panel's typed BPM field, so a later pass can find the
/// rectangle it occupied and so focus is the same id the field is shown under.
const BPM_FIELD_ID: &str = "bpm";

/// Widget id of the Panel's destination ComboBox, so focus is the same id the
/// ComboBox is shown under and `event_handler` can skip keys while it has it.
const DESTINATION_COMBO_ID: &str = "destination";
/// Tick copy is this many digits, zero-padded, so the field does not change width.
const TICK_DIGITS: usize = 5;
/// Flash next to BPM when the engine publishes a beat.
const BEAT_MARKER: &str = "**";
/// Marker next to BPM while Playback is stopped.
const REST_MARKER: &str = "//";
/// How many points to pull each label toward its value, relative to one monospace cell.
const LABEL_VALUE_TIGHTEN: f32 = 2.0;
/// Monospace cells between Readout groups, so C is not as close to T's value as to its own.
const GROUP_GAP_CELLS: f32 = 3.0;
/// Slot the Output readout occupies so a shorter device name is not truncated early.
const OUTPUT_READOUT_WIDTH: f32 = 196.0;
/// Menu action that asks the engine to discover output destinations again.
const OUTPUT_SCAN: &str = "Scan";

///
/// How many Cells the default window's console shows at Zoom 1.0, margin
/// included: half the default Grid on each axis, so a fresh console opens on
/// its top-left quarter with room to Pan (ADR 0047).
///
const DEFAULT_VIEW_COLUMNS: usize = DEFAULT_COL_COUNT / 2;
const DEFAULT_VIEW_ROWS: usize = DEFAULT_ROW_COUNT / 2;

///
/// The window size that presents `DEFAULT_VIEW_COLUMNS` by `DEFAULT_VIEW_ROWS`
/// Cells at Zoom 1.0: the Source's own points, and the chrome above and below
/// the console. Both are private, so neither is linked here.
///
/// A larger window shows more of the Grid rather than larger Cells; a smaller
/// one shows less of it.
///
pub const DEFAULT_VIEW_SIZE: [f32; 2] = [
    DEFAULT_VIEW_COLUMNS as f32 * CELL_SIZE,
    DEFAULT_VIEW_ROWS as f32 * CELL_SIZE + TOP_PANEL_HEIGHT + BOTTOM_PANEL_HEIGHT,
];

///
/// The margin, in Cells, between the Grid and the console at rest and the
/// distance a Pan can reach past each Grid edge (ADR 0047). It is counted in
/// Cells so it scales with the Zoom, and it is measured in the snapped Cell
/// side so it is always a whole number of physical pixels.
///
const SOURCE_MARGIN_CELLS: f32 = 2.0;

///
/// Run Clock copy for a wall-clock Duration: `mm:ss` through 59:59 inclusive,
/// then `h:mm:ss`. Hours are unpadded; minutes and seconds always occupy two
/// digits. Tick copy is zero-padded to [`TICK_DIGITS`].
///
fn format_run_clock(elapsed: Duration) -> String {
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

fn format_tick(tick: u64) -> String {
    format!("{tick:0width$}", width = TICK_DIGITS)
}

fn format_beat_marker(state: PlaybackState, on_beat: bool) -> &'static str {
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

fn panel_monospace_id(ui: &egui::Ui) -> FontId {
    ui.style()
        .text_styles
        .get(&egui::TextStyle::Monospace)
        .cloned()
        .unwrap_or_else(|| FontId::monospace(12.0))
}

fn monospace_width(ui: &egui::Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), panel_monospace_id(ui), Color32::WHITE)
        .size()
        .x
}

///
/// A monospace Readout that keeps a fixed slot, so a wider value does not
/// shove the widgets after it.
///
fn reserved_monospace(ui: &mut egui::Ui, text: &str, width: f32) -> egui::Response {
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
const BPM_FIELD_MARGIN: egui::Margin = egui::Margin::symmetric(8, 4);
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
/// ComboBox chrome reads [`Spacing::button_padding`]; match the BPM TextEdit margin.
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

fn keep_digits_in_text_events(events: &mut Vec<egui::Event>) {
    for event in events.iter_mut() {
        match event {
            egui::Event::Text(text) | egui::Event::Paste(text) => {
                text.retain(|c| c.is_ascii_digit());
            }
            _ => {}
        }
    }
    events.retain(|event| match event {
        egui::Event::Text(text) | egui::Event::Paste(text) => !text.is_empty(),
        _ => true,
    });
}

#[cfg(test)]
mod run_clock_tests {
    use super::{format_beat_marker, format_run_clock, format_tick};
    use std::time::Duration;

    #[test]
    fn run_clock_copy_is_mm_ss_until_an_hour_then_h_mm_ss() {
        let cases = [
            (0, "00:00"),
            (59, "00:59"),
            (60, "01:00"),
            (3599, "59:59"),
            (3600, "1:00:00"),
            (36000, "10:00:00"),
        ];
        for (seconds, copy) in cases {
            assert_eq!(
                format_run_clock(Duration::from_secs(seconds)),
                copy,
                "{seconds} seconds"
            );
        }
    }

    #[test]
    fn tick_copy_is_five_zero_padded_digits() {
        assert_eq!(format_tick(0), "00000");
        assert_eq!(format_tick(1), "00001");
        assert_eq!(format_tick(99999), "99999");
    }

    #[test]
    fn beat_marker_is_stars_when_the_engine_publishes_a_beat() {
        use orcvs::playback::PlaybackState;
        assert_eq!(format_beat_marker(PlaybackState::Playing, true), "**");
        assert_eq!(format_beat_marker(PlaybackState::Playing, false), "");
        assert_eq!(format_beat_marker(PlaybackState::Stopped, true), "//");
        assert_eq!(format_beat_marker(PlaybackState::Stopped, false), "//");
    }
}

fn translate_event(event: Event) -> Option<InputEvent> {
    match event {
        // Shift with an arrow keeps the anchor and extends the Region; a bare
        // arrow falls through to the arm below and collapses it.
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } if modifiers.shift && arrow(key).is_some() => arrow(key).map(InputEvent::Extend),
        Event::Key {
            key: Key::A,
            pressed: true,
            modifiers,
            ..
        } if modifiers.command => Some(InputEvent::SelectAll),
        Event::Key {
            key: Key::Enter,
            pressed: true,
            modifiers,
            ..
        } if modifiers.command => Some(InputEvent::Fill),
        // Tab steps the Cursor a Sector at a time; Shift Tab steps it back.
        // `Console::ui` cancels egui's own Tab focus navigation for the same
        // press whenever the Source holds the keys, so this and that
        // cancellation are two views of the one rule: Tab belongs to the
        // Source, not to focus. Only a bare Tab and a bare Shift Tab are
        // either of those — `modifiers.is_none()` and `modifiers.shift_only()`
        // are the same tests `Memory::begin_pass` itself uses to turn a Tab
        // into `FocusDirection::Next`/`Previous` (`egui-0.36.1/src/memory/mod.rs:596-597`),
        // so Ctrl, Command, or Alt held with Tab reaches neither egui's focus
        // navigation nor the Source here.
        Event::Key {
            key: Key::Tab,
            pressed: true,
            modifiers,
            ..
        } if modifiers.is_none() || modifiers.shift_only() => Some(if modifiers.shift_only() {
            InputEvent::PreviousSector
        } else {
            InputEvent::KeyPressed(InputKey::Tab)
        }),
        Event::Key {
            key, pressed: true, ..
        } => match key {
            Key::ArrowDown => Some(InputEvent::KeyPressed(InputKey::ArrowDown)),
            Key::ArrowLeft => Some(InputEvent::KeyPressed(InputKey::ArrowLeft)),
            Key::ArrowRight => Some(InputEvent::KeyPressed(InputKey::ArrowRight)),
            Key::ArrowUp => Some(InputEvent::KeyPressed(InputKey::ArrowUp)),
            Key::Backspace => Some(InputEvent::KeyPressed(InputKey::Backspace)),
            Key::Delete => Some(InputEvent::KeyPressed(InputKey::Delete)),
            Key::Space => Some(InputEvent::KeyPressed(InputKey::Space)),
            Key::Escape => Some(InputEvent::Collapse),
            _ => None,
        },
        Event::Text(text) => Some(InputEvent::Text(text)),
        Event::Copy => Some(InputEvent::Copy),
        Event::Cut => Some(InputEvent::Cut),
        Event::Paste(text) => Some(InputEvent::Paste(text)),
        // The running Orcvs models only input it acts on; all other toolkit
        // events remain presentation concerns and are dropped here.
        _ => None,
    }
}

///
/// The direction an arrow key moves the Cursor, or `None` for any other key.
///
fn arrow(key: Key) -> Option<Arrow> {
    match key {
        Key::ArrowDown => Some(Arrow::Down),
        Key::ArrowLeft => Some(Arrow::Left),
        Key::ArrowRight => Some(Arrow::Right),
        Key::ArrowUp => Some(Arrow::Up),
        _ => None,
    }
}

///
/// A keyboard Zoom command: a command chord for `=`/`+`, `-`, or `0`.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ZoomCommand {
    In,
    Out,
    Reset,
}

///
/// The Zoom command a toolkit event asks for, or none.
///
/// Only a held [`egui::Modifiers::command`] turns `=`, `+`, `-` or `0` into a
/// Zoom step. Bare, they are Source characters — [`translate_event`] reaches
/// them as [`Event::Text`], never through this — so this answers `None` for
/// an unmodified key and [`show_source_scene`] leaves the Zoom exactly where
/// it was.
///
fn zoom_command(event: &Event) -> Option<ZoomCommand> {
    match event {
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } if modifiers.command => match key {
            Key::Equals | Key::Plus => Some(ZoomCommand::In),
            Key::Minus => Some(ZoomCommand::Out),
            Key::Num0 => Some(ZoomCommand::Reset),
            _ => None,
        },
        _ => None,
    }
}

///
/// `zoom` after one keyboard Zoom command: stepped by [`GLYPH_SCALE_STEP`] and
/// clamped to [`MIN_ZOOM`]..=[`MAX_ZOOM`].
///
/// Stepped from the nearest multiple of the step rather than by adding it, so
/// a long session stays exactly on the grid [`glyph_scale`] quantises to
/// instead of drifting off it through repeated float addition. `Reset`
/// answers 1.0 outright, whatever step `zoom` was on.
///
fn stepped_zoom(zoom: f32, command: ZoomCommand) -> f32 {
    if command == ZoomCommand::Reset {
        return 1.0;
    }
    let direction = if command == ZoomCommand::In {
        1.0
    } else {
        -1.0
    };
    let steps = (zoom / GLYPH_SCALE_STEP).round() + direction;
    (steps * GLYPH_SCALE_STEP).clamp(MIN_ZOOM, MAX_ZOOM)
}

fn source_bounds(grid: Grid) -> Rect {
    Rect::from_min_size(
        Pos2::ZERO,
        Vec2::new(grid.columns() as f32, grid.rows() as f32) * CELL_SIZE,
    )
}

///
/// The Source View: a Zoom and a Pan, presented as the scale and translation
/// the Cells are drawn under.
///
/// Zoom opens at 1.0 — the Source's own Cell — and is a stated step, never a
/// property of the window. Pan is anchored at the console's top-left and is
/// bounded by the Grid: an axis the whole Source already fills has nowhere to
/// Pan, and a Pan that would open a gap past an edge settles back inside.
///
/// A Cursor move or a Zoom that would leave the Cursor's Cell outside the
/// console Pans the least distance that brings the whole Cell back into view,
/// still bounded by the Grid; a Pan on its own does not chase the Cursor.
/// `previous_cursor` is what tells a Cursor move apart from a frame that
/// merely redrew it — see `docs/adr/0045-the-source-view-is-a-bounded-space.md`.
///
/// `to_global` is derived each frame from Zoom, Pan and the console's origin
/// so `presented_grid` and the diagnostics still read one transform.
///
struct SourceView {
    zoom: f32,
    pan: Vec2,
    /// The Cursor [`show_source_scene`] last saw, so a change from one frame
    /// to the next reads as a Cursor move worth following rather than every
    /// frame answering yes. `None` before the first frame a fresh `SourceView`
    /// presents, so it does not Pan away from wherever the console opened
    /// merely because there was nothing yet to compare the Cursor against.
    previous_cursor: Option<Position>,
    /// The anchor of the primary drag selecting a Region, while one is in
    /// progress. The Cursor follow is paced to the pointer while it is.
    region_drag: Option<Position>,
    to_global: TSTransform,
}

impl Default for SourceView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            previous_cursor: None,
            region_drag: None,
            to_global: TSTransform::IDENTITY,
        }
    }
}

///
/// The Pan that keeps the Source inside the console: top-left when the Source
/// is smaller on an axis, and between the two edges when it is larger.
/// [`show_source_scene`] passes the padded Source — the Grid and its margin on
/// both sides — so the edges here are the margin's, not the Grid's.
///
fn clamp_pan(pan: Vec2, console: Vec2, source: Vec2) -> Vec2 {
    Vec2::new(
        clamp_pan_axis(pan.x, console.x, source.x),
        clamp_pan_axis(pan.y, console.y, source.y),
    )
}

fn clamp_pan_axis(pan: f32, console: f32, source: f32) -> f32 {
    let slack = console - source;
    if !slack.is_finite() || slack >= 0.0 {
        0.0
    } else {
        pan.clamp(slack, 0.0)
    }
}

///
/// The rectangle a Cursor follow brings into view, in the unpanned points of
/// the padded Source — the Grid with a margin of [`SOURCE_MARGIN_CELLS`] on
/// every side — at the snapped Cell `side` [`show_source_scene`] also bounds
/// [`clamp_pan`] by.
///
/// The Cursor's own Cell, reaching across the margin on each side where the
/// Cursor is on the Grid's first or last Column or Row, so a follow to an edge
/// shows the Grid's edge as an edge rather than flush against the console's
/// (ADR 0047).
///
fn followed_cell(cursor: Position, grid: Grid, side: f32) -> Rect {
    let margin = SOURCE_MARGIN_CELLS * side;
    let cell = Rect::from_min_size(
        Pos2::new(cursor.x() as f32, cursor.y() as f32) * side + Vec2::splat(margin),
        Vec2::splat(side),
    );
    let reach = |at: usize, count: usize| {
        let near = if at == 0 { margin } else { 0.0 };
        let far = if at + 1 == count { margin } else { 0.0 };
        (near, far)
    };
    let (left, right) = reach(cursor.x(), grid.columns());
    let (top, bottom) = reach(cursor.y(), grid.rows());
    Rect::from_min_max(
        cell.min - Vec2::new(left, top),
        cell.max + Vec2::new(right, bottom),
    )
}

///
/// The Pan that brings `cell` — already in the same units as `pan` once
/// translated by it — fully inside a console of `console_size`, moving the
/// least distance along each axis and leaving an axis alone where the Cell
/// already shows in full.
///
/// Not itself bounded by the Grid: [`clamp_pan`] runs after this wherever it
/// is called, so a Cell nearer an edge than the console is wide settles
/// against that edge rather than opening a gap past it.
///
fn follow_cursor(pan: Vec2, console_size: Vec2, cell: Rect) -> Vec2 {
    let shown = cell.translate(pan);
    Vec2::new(
        follow_axis(pan.x, shown.min.x, shown.max.x, console_size.x),
        follow_axis(pan.y, shown.min.y, shown.max.y, console_size.y),
    )
}

///
/// One axis of [`follow_cursor`]: shift `pan` by exactly the overflow past
/// whichever edge the Cell has fallen outside, or leave it be when the Cell
/// already sits between the two.
///
fn follow_axis(pan: f32, min: f32, max: f32, console: f32) -> f32 {
    if min < 0.0 {
        pan - min
    } else if max > console {
        pan - (max - console)
    } else {
        pan
    }
}

///
/// How far `pointer` is past each edge of `console`, and zero on an axis where
/// it is between the two.
///
fn overshoot(console: Rect, pointer: Pos2) -> Vec2 {
    Vec2::new(
        (console.min.x - pointer.x)
            .max(pointer.x - console.max.x)
            .max(0.0),
        (console.min.y - pointer.y)
            .max(pointer.y - console.max.y)
            .max(0.0),
    )
}

///
/// The part of a follow Pan of `wanted` that a Region drag takes this frame:
/// more the further the pointer is `past` the console's edge, never more than one
/// Cell of `side`, and none on an axis the pointer has not left.
///
fn edge_scroll(wanted: Vec2, past: Vec2, side: f32) -> Vec2 {
    let most = (past / EDGE_SCROLL_REACH).min(Vec2::splat(side));
    Vec2::new(
        wanted.x.clamp(-most.x, most.x),
        wanted.y.clamp(-most.y, most.y),
    )
}

///
/// Whether `to_global` can be presented and inverted.
///
/// `egui::Scene::show` used to reset a transform that had gone bad
/// (`egui-0.36.1/src/containers/scene.rs:151-152, 168-173`) and nothing
/// replaces that once the container is gone. `grid_viewport` answers a Cell
/// size of zero for a console with no area, so the fit it yields has a scaling
/// of zero, and `TSTransform::inverse` divides by the scaling — which
/// `Scene::register_pan_and_zoom` does on every frame the pointer is over the
/// console. An unguarded zero therefore resolves every pointer position to NaN.
///
/// `TSTransform::is_valid` is not enough on its own: it checks only
/// `translation.x` (`emath-0.36.1/src/ts_transform.rs:55-57`) and admits a
/// negative scaling, which would present the Source mirrored.
///
fn is_presentable(to_global: TSTransform) -> bool {
    to_global.scaling.is_finite() && to_global.scaling > 0.0 && to_global.translation.is_finite()
}

///
/// The scale a [`FontId`] is derived from, quantised to [`GLYPH_SCALE_STEP`].
///
/// Never zero or negative: a font size of zero lays nothing out, and the
/// smallest step still draws something a viewer can see is there.
///
/// # Why the step is taken downwards
///
/// The step is absolute, so rounding to the nearest one is disproportionate at
/// a small scale: a console fitting at 0.2 would round up to 0.25 and lay an
/// 11.5 point Glyph out at 2.875 points inside a 3.2 point Cell, where the same Glyph
/// at the Source's own scale takes 11.5 of 16. Flooring keeps a Glyph's share of
/// its Cell at or under what the fit gave it at every scale, and costs at most
/// one step of sharpness rather than a Cell's worth of proportion. Both zoom
/// limits and the Source's own scale are exact multiples of the step, so
/// flooring leaves them exactly where rounding did, and the step count the
/// atlas budget above is stated over is unchanged.
///
fn glyph_scale(scaling: f32) -> f32 {
    if !scaling.is_finite() || scaling <= 0.0 {
        return GLYPH_SCALE_STEP;
    }

    ((scaling / GLYPH_SCALE_STEP).floor() * GLYPH_SCALE_STEP).max(GLYPH_SCALE_STEP)
}

/// Console wraps the running Orcvs with egui presentation concerns.
///
pub struct Console {
    orcvs: Orcvs,
    /// Device discovery and selection for whatever MIDI backend `orcvs` has on
    /// this target. The console never asks what target it is on: a target with
    /// no native backend answers an empty destination list here, and
    /// `native_midi::AVAILABLE` says whether the ComboBox is enabled and
    /// whether Refresh is shown.
    midi: MidiDeviceSelection,
    font_family: egui::FontFamily,
    source_view: SourceView,
    diagnostics_open: bool,
    /// The BPM field's widget id, so a later pass can find the rectangle it
    /// occupied and so focus is the same id the field is shown under.
    #[cfg(test)]
    bpm_widget_id: egui::Id,
    /// Whether keyboard input belonged to a control rather than the Source
    /// when the last frame's widgets were done: any widget holding egui's
    /// keyboard focus (`Context::egui_wants_keyboard_input`, which is
    /// `Memory::focused().is_some()`, `egui-0.36.1/src/context.rs:2982-2985`),
    /// or any open popup — a menu, or the destination ComboBox's list — which
    /// a click opens without taking focus (`Popup::is_any_open`). Latched
    /// rather than asked where it is read, because
    /// `event_handler` runs before this frame's widgets are shown, and
    /// `Memory::begin_pass` has already let Escape clear the focus it was
    /// pressed to leave (`egui-0.36.1/src/memory/mod.rs:596-601`).
    keyboard_elsewhere: bool,
    cursor_effects: CursorEffectSettings,
    cursor_effect_animation: CursorEffectAnimation,
    source_paint: SourcePaintSettings,
    reduced_motion: bool,
    #[cfg(feature = "persistence")]
    persistence: crate::persistence::Persistence,
}

impl Console {
    ///
    /// The console over the running Orcvs its storage last held.
    ///
    /// Fallible because a running Orcvs is: ADR 0041 makes its Playback Engine
    /// a task, and a task needs a runtime to be spawned on. The native binary
    /// is inside `#[tokio::main]` when `eframe` calls this, and the browser
    /// spawns onto the page's event loop and needs nothing; a build that
    /// reached here with neither has no console to show, which is what handing
    /// the error to `eframe` says.
    ///
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, PlaybackStartError> {
        let style = style();
        cc.egui_ctx.set_style_of(egui::Theme::Dark, style);
        cc.egui_ctx.set_theme(egui::Theme::Dark);

        // egui's own `Context::end_pass` answers the same command `=`/`+`,
        // `-` and `0` chords by changing `zoom_factor` — the whole UI's
        // scale, not the Source View's (`egui-0.36.1/src/gui_zoom.rs`,
        // `Options::zoom_with_keyboard`, on by default). Those chords are the
        // Source View's Zoom here, so egui's own reading of them is turned
        // off rather than left to race it.
        cc.egui_ctx
            .options_mut(|options| options.zoom_with_keyboard = false);

        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();
        let key = "MonaspaceNeon";
        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            key.to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/MonaspaceNeon-Regular.otf"))
                .into(),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, key.to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, key.to_owned());

        cc.egui_ctx.set_fonts(fonts);

        // The stored Source revision when storage holds one, and the ordinary
        // default Grid otherwise. Every derived view is rebuilt from it.
        let start = starting_source(cc.storage);
        let orcvs = Orcvs::with_source(start.source)?;
        wake_panel_when_playback_publishes(cc.egui_ctx.clone(), orcvs.playback_observation_watch());
        let mut midi = MidiDeviceSelection::new(
            orcvs.midi_selection_handle(),
            Box::new(NativeMidiBackend::new()),
        );
        midi.refresh_destinations();
        Ok(Self {
            orcvs,
            midi,
            font_family: FontId::monospace(DEFAULT_FONT_SIZE).family,
            source_view: SourceView::default(),
            diagnostics_open: false,
            #[cfg(test)]
            bpm_widget_id: egui::Id::new(BPM_FIELD_ID),
            keyboard_elsewhere: false,
            cursor_effects: start.cursor_effects,
            cursor_effect_animation: CursorEffectAnimation::default(),
            source_paint: start.source_paint,
            reduced_motion: prefers_reduced_motion(),
            #[cfg(feature = "persistence")]
            persistence: start.persistence,
        })
    }

    ///
    /// Replaces the running Orcvs's Source, Grid included, with the Function
    /// reference — what the File menu offers, since a console that restores
    /// nothing opens the blank default Grid (`source-view/03`). With the `persistence`
    /// feature on, the reference then saves like any other Source on the next
    /// scheduled save.
    ///
    /// A Grid change is a whole-Orcvs replacement, so this also rebuilds MIDI
    /// device selection over the new Orcvs's handle exactly as [`Console::new`]
    /// does; a previously selected destination does not carry over. Every
    /// other console setting — Theme, Cursor effects, Diagnostics visibility —
    /// is untouched, because only the Source was asked to change.
    ///
    fn load_function_reference(&mut self) {
        match Orcvs::with_source(function_reference()) {
            Ok(orcvs) => {
                let mut midi = MidiDeviceSelection::new(
                    orcvs.midi_selection_handle(),
                    Box::new(NativeMidiBackend::new()),
                );
                midi.refresh_destinations();
                self.orcvs = orcvs;
                self.midi = midi;
                self.source_view = SourceView::default();
            }
            Err(error) => {
                crate::report::error!("failed to load the Function reference: {error}");
            }
        }
    }
}

fn frames_per_second(frame_time: f32) -> Option<f32> {
    frame_time.is_normal().then(|| frame_time.recip())
}

///
/// Paints the Panel from a published Tick. A wait started from this Render
/// Frame is a second clock; this asks for a paint when the engine publishes.
///
fn wake_panel_when_playback_publishes(
    ctx: egui::Context,
    mut observation: orcvs::playback::PlaybackObservationWatch,
) {
    let _ = observation.borrow_and_update();
    let wake = async move {
        loop {
            if observation.changed().await.is_err() {
                break;
            }
            ctx.request_repaint();
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::spawn(wake);
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(wake);
    }
}

fn show_diagnostics(
    ctx: &egui::Context,
    open: &mut bool,
    frame: &eframe::Frame,
    to_global: TSTransform,
    console: Rect,
    cell_size: f32,
) {
    let frame_time = ctx.input(|input| input.stable_dt);
    egui::Window::new("Diagnostics")
        .open(open)
        .default_width(360.0)
        .show(ctx, |ui| {
            egui::Grid::new("orcvs-diagnostics-summary")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("FPS");
                    ui.monospace(
                        frames_per_second(frame_time)
                            .map(|fps| format!("{fps:.1}"))
                            .unwrap_or_else(|| "—".to_owned()),
                    );
                    ui.end_row();

                    ui.label("Frame time");
                    ui.monospace(format!("{:.2} ms", frame_time * 1_000.0));
                    ui.end_row();

                    ui.label("CPU time");
                    ui.monospace(
                        frame
                            .info()
                            .cpu_usage
                            .map(|seconds| format!("{:.2} ms", seconds * 1_000.0))
                            .unwrap_or_else(|| "—".to_owned()),
                    );
                    ui.end_row();

                    ui.label("Cell size");
                    ui.monospace(format!("{cell_size:.1} pt"));
                    ui.end_row();

                    // The console owns the transform, so the zoom is a field of
                    // it rather than a ratio derived back out of a region.
                    ui.label("Source zoom");
                    ui.monospace(format!("{:.2}×", to_global.scaling));
                    ui.end_row();

                    ui.label("Visible Source region");
                    ui.monospace(format!("{:.1?}", to_global.inverse() * console));
                    ui.end_row();

                    ui.label("Pixels per point");
                    ui.monospace(format!("{:.2}", ctx.pixels_per_point()));
                    ui.end_row();
                });

            ui.separator();
            egui::CollapsingHeader::new("egui inspection")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(420.0)
                        .show(ui, |ui| ctx.inspection_ui(ui));
                });
        });
}

///
/// The alphabet the Glyph table covers: the printable ASCII a Source Cell can
/// show, less the space.
///
/// `orcvs::source::CellContent` accepts exactly `0x20..=0x7e`, and every
/// spelling a Token prints for an empty Cell is inside that range, so no
/// Cell of a Source can ask for a character outside it. The space is the one
/// printable character deliberately left out: a Cell showing one paints no
/// Glyph at all, and a Shape and a galley clone spent showing nothing is the
/// cost this drawing exists to stop paying.
///
const ALPHABET_FIRST: u8 = b'!';
const ALPHABET_LAST: u8 = b'~';

///
/// One laid-out Glyph per character of the alphabet, for the font and size the
/// Source is painted at.
///
/// The table has one owner and one construction site — [`show_source`] — so
/// nothing lays the alphabet out at a second size and thrashes it.
///
/// # Scratch for exactly one Render Frame
///
/// Retaining this table across Render Frames is unsound, not merely wasteful,
/// and that is why it is rebuilt every frame rather than memoised.
///
/// A galley's `RowVisuals::mesh` holds *texel* coordinates into the live font
/// atlas, normalised against that atlas's size at tessellation
/// (`epaint-0.36.1/src/text/text_layout_types.rs`, `tessellator.rs`), and
/// `Fonts::begin_pass` (`epaint-0.36.1/src/text/fonts.rs`) replaces the whole
/// `FontsImpl` — a fresh atlas with empty glyph caches — whenever the text
/// options change or the atlas passes its fill ratio. A galley held across that
/// recreate indexes unrelated texels and paints a *different character*. Atlas
/// *growth* is safe, because it only extends the image height and every texel
/// keeps its coordinates; *recreation* is what corrupts, and `font_image_size()`
/// cannot tell the two apart.
///
/// epaint's own `GalleyCache` is the memo, and it is the only cache in the
/// stack that `begin_pass` invalidates alongside the atlas. An
/// `egui::cache::FrameCache` or a `ctx.data()` entry evicts on last-frame use
/// and knows nothing about fonts, so either would carry exactly that
/// corruption. Do not "optimise" this into one.
///
/// Rebuilding costs one `Context` write lock for the whole table instead of one
/// per Cell, and one `String` per character of the alphabet instead of one per
/// Cell. Those are the wins; retaining the table is needed for none of them.
///
struct GlyphTable {
    /// The `Context` the alphabet was laid out through, for the one character
    /// the table cannot cover. Cloning a `Context` clones an `Arc`, and the
    /// table lives for one Render Frame, so this holds nothing open.
    ctx: egui::Context,
    /// The font the alphabet was laid out at, for the one character the table
    /// cannot cover.
    font: FontId,
    /// The alphabet, indexed by `byte - ALPHABET_FIRST`.
    characters: Vec<Arc<Galley>>,
}

impl GlyphTable {
    ///
    /// Lays the alphabet out for this Render Frame, inside a single
    /// `ctx.fonts_mut` closure.
    ///
    /// One galley per character, never one per row: egui 0.36 shapes through
    /// harfrust with `liga` and `calt` enabled and does not apply
    /// `extra_letter_spacing` within a shaping cluster, so a row laid out as one
    /// galley would let a ligature consume two Cells and shift the rest of the
    /// row. MonaspaceNeon has ligatures and Orcvs Source is full of the pairs
    /// that trigger them.
    ///
    fn lay_out(ctx: &egui::Context, font: FontId) -> Self {
        let characters = ctx.fonts_mut(|fonts| {
            (ALPHABET_FIRST..=ALPHABET_LAST)
                .map(|byte| {
                    // Laid out with `Color32::PLACEHOLDER`, so one galley serves
                    // every Cell whatever colour that Cell's Glyph is painted
                    // in: the tessellator substitutes the fallback colour for
                    // placeholder vertices alone.
                    fonts.layout_delayed_color(
                        char::from(byte).to_string(),
                        font.clone(),
                        f32::INFINITY,
                    )
                })
                .collect()
        });

        Self {
            ctx: ctx.clone(),
            font,
            characters,
        }
    }

    /// The laid-out Glyph for `character`, or `None` for a character outside
    /// the alphabet — the space included.
    fn galley(&self, character: char) -> Option<&Arc<Galley>> {
        let byte = u8::try_from(character).ok()?;
        self.characters
            .get(usize::from(byte.checked_sub(ALPHABET_FIRST)?))
    }

    ///
    /// The laid-out Glyph for a character a Cell shows, laying out the one the
    /// alphabet does not cover.
    ///
    /// A Source Cell holds printable ASCII by construction, so the fallback
    /// lays out at most the odd galley for a Source that found a way to hold
    /// something else — and it costs that one Cell the allocation and the
    /// whole-`Context` lock the table exists to take once. It answers a galley
    /// rather than painting one, so that Cell stays a Shape in the ordered
    /// sequence rather than a `Painter::text` painted out of turn.
    ///
    fn glyph(&self, character: char) -> Arc<Galley> {
        match self.galley(character) {
            Some(galley) => galley.clone(),
            None => self.ctx.fonts_mut(|fonts| {
                fonts.layout_delayed_color(character.to_string(), self.font.clone(), f32::INFINITY)
            }),
        }
    }
}

///
/// One run of consecutive Cells that share a background colour, as the single
/// rectangle that fills them all.
///
/// # Why the rectangle is snapped
///
/// Not to prevent a seam. epaint snaps the rectangle whether or not this does:
/// `RectShape::filled` leaves `round_to_pixels: None`,
/// `TessellationOptions::round_rects_to_pixels` defaults to true, and
/// `tessellate_rect` then applies `Rect::round_to_pixels` — the same `emath`
/// function this calls — at `epaint-0.36.1/src/tessellator.rs:1829-1862`. A run
/// left unsnapped here would reach the screen as the same pixels.
///
/// It is snapped so the rectangle is one a test can predict. The snap is the
/// last thing that moves an edge, so doing it here puts the Shape a Render
/// Frame carries at the coordinates the paint lands on, and an assertion can
/// state them exactly rather than within a pixel. `Rect::round_to_pixels`
/// rounds the two corners independently
/// (`emath-0.36.1/src/gui_rounding.rs:155-186`), so a run's far edge lands on
/// the physical pixel the next Cell's near edge would have and neighbouring
/// runs still tile — which is what makes that predicted rectangle the same
/// paint as the Cells it replaces.
///
fn background_run(covered: Rect, fill: Color32, pixels_per_point: f32) -> Shape {
    Shape::Rect(RectShape::filled(
        covered.round_to_pixels(pixels_per_point),
        CornerRadius::ZERO,
        fill,
    ))
}

///
/// A [`Paint`] as the Shapes that draw it, grouped by the order they are
/// painted in.
///
/// # Why the groups are fields
///
/// The order that matters is across kinds and not across Cells: every
/// background precedes every Glyph, so a later Cell's fill can never paint over
/// an earlier Cell's Glyph, and the Cursor follows both, so no neighbouring
/// Cell's fill or seam can paint over it. The borders sit between the two,
/// after the fills, because a run widened across several Cells covers the
/// borders of every Cell but its last. That is a fact about how a painter
/// composites rather than about the Source, which is why it is decided here and
/// not in the value layer — and naming the ordered groups states it where a
/// comment used to.
///
/// # Why they are built eagerly
///
/// Shape construction is never lazy. `Painter::extend` runs the iterator it is
/// given inside `ctx.graphics_mut`, a full `Context` write lock, so a Shape
/// built lazily is a Shape built while holding it. The fields are complete
/// before [`SourceShapes::into_shapes`] hands the first one out.
///
struct SourceShapes {
    /// The living field beneath every exact Grid shape.
    area: Vec<Shape>,
    /// The coalesced background runs, one rectangle each.
    backgrounds: Vec<Shape>,
    /// Every Cell's own border but the Cursor's, and the Cursor's too while a
    /// Region spans more than one Cell.
    borders: Vec<Shape>,
    /// One galley per Cell that shows a character other than the space.
    glyphs: Vec<Shape>,
    /// The sector seams, left edge then top edge, Cell by Cell.
    seams: Vec<Shape>,
    /// The Cursor's own stroke, painted last: the Cursor Effect's frame — its
    /// Cell's, or the lasso around a Region larger than one Cell — or the
    /// selected Cell's border when no effect frame was built.
    cursor: Vec<Shape>,
}

impl SourceShapes {
    ///
    /// Draws a Paint at `viewport`: the geometry the value layer carries none
    /// of, applied to the colours and characters it carries all of.
    ///
    /// Stroke widths take [`GridViewport::cell_scale`] so the Grid lines and
    /// sector seams are one Source point wide at every zoom.
    /// `pixels_per_point` is the device scale the background runs are snapped
    /// to; see [`background_run`].
    ///
    /// Borders, the Cursor's border, the sector seams and the background runs
    /// need no font atlas. Glyph placement does — see [`Self::place_glyphs`] —
    /// so this composes the two steps rather than folding the atlas into the
    /// geometry pass.
    ///
    fn new(
        paint: &Paint,
        viewport: &GridViewport,
        table: &GlyphTable,
        pixels_per_point: f32,
        cursor_effect: crate::cursor_effects::CursorEffectShapes,
    ) -> Self {
        let mut shapes = Self::geometry(paint, viewport, pixels_per_point);
        shapes.area = cursor_effect.area;
        if !cursor_effect.frame.is_empty() {
            shapes.cursor = cursor_effect.frame;
        }
        shapes.place_glyphs(paint, viewport, table);
        shapes
    }

    ///
    /// Backgrounds, borders, the Cursor's border and the sector seams — every
    /// Shape whose construction is arithmetic on a `Rect` and a `Color32`.
    ///
    /// Takes no [`GlyphTable`] and no `egui::Context`. The `glyphs` field is
    /// empty until [`Self::place_glyphs`] fills it; both steps finish before
    /// [`Self::into_shapes`] hands the first Shape out.
    ///
    fn geometry(paint: &Paint, viewport: &GridViewport, pixels_per_point: f32) -> Self {
        let scale = viewport.cell_scale();
        // A border is the rule on every Cell the Paint covers, so it is sized
        // up front — to those Cells rather than to the Grid: a densely written
        // Source that regrew the group would pay the reallocation on every
        // Render Frame, and a zoomed console reserves what it draws instead of
        // what the Source holds. Backgrounds are sparse selection state, so
        // that group starts empty. Glyphs are reserved in
        // [`Self::place_glyphs`].
        let mut backgrounds = Vec::new();
        let mut borders = Vec::with_capacity(paint.count());
        let mut seams = Vec::new();
        let mut cursor = Vec::new();

        for run in paint.background_runs() {
            // Built from the Cells' own rectangles — `GridViewport::cell_rect`
            // is the column-to-x function every other Shape here goes through —
            // so a coalesced edge is the edge the Cells it replaces would have
            // been given rather than a sum of Cell sides accumulated across the
            // row. A run covers at least one column, so `end - 1` is a column
            // of the run.
            //
            // Asserted rather than assumed: the non-emptiness is
            // `Paint::background_runs`' — it seeds a run as
            // `position.x()..position.x() + 1` and only ever extends the end —
            // and nothing in `BackgroundRun` holds the fold to it. An empty
            // range would wrap `end - 1` to `usize::MAX` in release and hand
            // `cell_rect` a column off the far side of the Grid, which paints a
            // wild rectangle rather than panicking.
            debug_assert!(
                !run.columns.is_empty(),
                "a background run covers at least one column, not {:?}",
                run.columns
            );
            let covered = Rect::from_min_max(
                viewport.cell_rect(run.columns.start, run.row).min,
                viewport.cell_rect(run.columns.end - 1, run.row).max,
            );
            backgrounds.push(background_run(covered, run.colour, pixels_per_point));
        }

        for (position, cell) in paint.cells() {
            let rect = viewport.cell_rect(position.x(), position.y());
            // The Cell's own border, stroke and no fill: a widened run paints
            // over the borders of every Cell inside it, so the fill and the
            // border cannot be one shape.
            let border = Shape::Rect(RectShape::stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(GRID_LINE_WIDTH * scale, cell.border),
                StrokeKind::Inside,
            ));

            // The selected Cell's border is the Cursor, and the Cursor is
            // painted last. A Cursor the viewport does not reach is no Cell of
            // this Paint, so the comparison never matches and the group stays
            // empty. While a Region spans more than one Cell the lasso around
            // it is the Cursor, and the Cursor's Cell keeps an ordinary border.
            if paint.cursor() == Some(position) && !paint.region_spans() {
                cursor.push(border);
            } else {
                borders.push(border);
            }

            // A seam is absent on the Cursor's Cell, while it is framed on its
            // own, because the derive suppressed it there, so this step never
            // learns that rule.
            for (colour, ends) in [
                (cell.sector_left, [rect.left_top(), rect.left_bottom()]),
                (cell.sector_top, [rect.left_top(), rect.right_top()]),
            ] {
                if let Some(colour) = colour {
                    seams.push(Shape::line_segment(
                        ends,
                        Stroke::new(SECTOR_LINE_WIDTH * scale, colour),
                    ));
                }
            }
        }

        Self {
            area: Vec::new(),
            backgrounds,
            borders,
            glyphs: Vec::new(),
            seams,
            cursor,
        }
    }

    ///
    /// Places one galley per Cell that shows a character other than the space.
    ///
    /// Needs a [`GlyphTable`] — a galley's size exists only after layout — and
    /// fills `glyphs` eagerly so [`Self::into_shapes`] never builds a Shape
    /// while holding the `Context` write lock `Painter::extend` takes.
    ///
    fn place_glyphs(&mut self, paint: &Paint, viewport: &GridViewport, table: &GlyphTable) {
        self.glyphs = Vec::with_capacity(paint.count());

        for (position, cell) in paint.cells() {
            if cell.character == ' ' {
                continue;
            }

            let rect = viewport.cell_rect(position.x(), position.y());
            let galley = table.glyph(cell.character);
            // Centred in a Cell whose own corner is an exact multiple of the
            // Cell size.
            self.glyphs.push(Shape::galley(
                rect.center() - galley.size() / 2.0,
                galley,
                cell.foreground,
            ));
        }
    }

    ///
    /// The shape groups end to end, in paint order.
    ///
    /// An iterator over the owned `Vec`s rather than a sixth one: the chain
    /// costs nothing, and collecting it would spend a further allocation of
    /// some two thousand elements, and a whole re-move, per Render Frame.
    ///
    fn into_shapes(self) -> impl Iterator<Item = Shape> {
        self.area
            .into_iter()
            .chain(self.backgrounds)
            .chain(self.borders)
            .chain(self.glyphs)
            .chain(self.seams)
            .chain(self.cursor)
    }
}

///
/// The rectangle the Cursor Effect's frame outlines: the Cursor's Cell, or the
/// whole Region when it spans more than one Cell — the lasso.
///
fn effect_outline(frame: &RenderFrame, viewport: &GridViewport) -> Rect {
    let region = frame.region();
    let (columns, rows) = (region.columns(), region.rows());
    Rect::from_min_max(
        viewport.cell_rect(columns.start, rows.start).min,
        viewport.cell_rect(columns.end - 1, rows.end - 1).max,
    )
}

///
/// Draws the Source Grid and answers the one question a click asks of it:
/// which Cell, and whether Shift extends the Region to it.
///
/// The whole Grid is one allocated rectangle and no Cell is a widget, so the
/// cost of a Render Frame is shapes rather than interaction rects and widget
/// ids. Only the Positions the viewport covers are painted —
/// `GridViewport::visible_positions` is this loop's one source of them — so
/// that cost follows the viewport rather than the Source.
///
/// A background is painted only where selection state differs from the Source
/// fill the panel already provides. Consecutive Cells in a row that want the
/// same background share one rectangle.
///
/// The click is answered rather than acted on. Selecting a Cell is the Source's
/// business and `Console::ui` owns the running Orcvs it is asked of; handing the
/// [`PointerSelection`] back is what leaves this function with nothing but a
/// Render Frame and a place to draw it.
///
/// Eight parameters, one over clippy's default: `source_paint` is the eighth,
/// added by `syntax-highlighting/01`. Each of the eight is an independent,
/// already-tested value threaded straight through from `show_source_scene`'s
/// own parameters of the same names — geometry, a Render Frame, and the three
/// presentation settings `Console::ui` owns — so grouping any of them into a
/// struct would add an indirection this function's one caller does not need,
/// for a threshold rather than a real complexity this function has grown.
///
#[allow(clippy::too_many_arguments)]
fn show_source(
    ui: &mut egui::Ui,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    viewport: GridViewport,
    clip: Rect,
    cursor_effect_sample: CursorEffectSample,
    cursor_effect_settings: CursorEffectSettings,
    source_paint: SourcePaintSettings,
) -> Option<PointerSelection> {
    // The shape the Render Frame was derived from, named apart from the
    // `GridViewport` the Cells are painted at.
    let source_grid = frame.grid();
    // One rectangle for the whole Grid, sensing clicks and nothing else.
    //
    // Within a layer a later-registered child wins the click tie, and would win
    // the drag too if it sensed drag. The pan rectangle `show_source_scene`
    // allocates is registered before this one, so sensing clicks alone takes
    // the clicks and leaves the middle-drag and the Alt-held primary-drag Pan
    // to it. `Sense::CLICK` rather than `Sense::click()`, which is
    // `CLICK | FOCUSABLE` and would put the Grid in the tab order where a
    // thousand Buttons never were.
    //
    // The rectangle is the Grid, not the console area. Surplus console past the
    // Grid's edges is not a Cell, so a click there selects nothing.
    // Clipped to the console, because `Ui::interact` bounds a widget by the
    // `Ui`'s clip rect rather than by the console area, and a zoomed-in Grid
    // reaches past the console on every side. `Scene::show` used to set that
    // clip rect itself (`scene.rs:209`); with the container gone the Grid
    // states its own bound.
    let response = ui.interact(
        viewport.rect.intersect(clip),
        ui.id().with("source_grid"),
        Sense::CLICK,
    );
    // What `Scene::show` did with `set_clip_rect` and a sublayer, in the one
    // layer that is left: the Grid is clipped to the console area, so a zoomed
    // Grid cannot paint over the chrome around it. A sublayer is not needed
    // because the Source Grid is all this layer holds, so painting it directly
    // is the ordering `set_sublayer` used to arrange.
    let painter = ui.painter().with_clip_rect(clip);
    // The scale is already in the Cell size, and the Scene used to carry it to
    // the strokes as well, so the Grid lines and sector seams take it here
    // rather than staying one Source point wide at every zoom.
    // The device scale the background runs are snapped to; see
    // [`background_run`].
    let pixels_per_point = ui.pixels_per_point();
    let table = GlyphTable::lay_out(
        ui.ctx(),
        // Laid out at the size it is drawn at rather than resampled from a
        // rasterisation at the Source's own size, which is the second thing the
        // layer transform cost. The scale is quantised so a steady zoom hits
        // the galley cache and a sweep across the zoom range stays inside the
        // atlas; see `GLYPH_SCALE_STEP`.
        FontId::new(
            DEFAULT_FONT_SIZE * glyph_scale(viewport.cell_scale()),
            font_family.clone(),
        ),
    );

    // The only source of Positions the two steps below have. Everything they
    // reach is inside these ranges, so the cost of a Render Frame follows the
    // viewport rather than the Source: a Grid the console shows a tenth of
    // costs a tenth of the Cell iteration and a tenth of the Shapes, whatever
    // the Grid's size.
    //
    // The ranges reach one Cell past what is visible. A Cell draws its own left
    // and top seams, so the seam at a Cell's right or bottom edge belongs to
    // the Cell one past it, and a border feathers a physical pixel outside the
    // rectangle it strokes. Both land on the clip's boundary, which is what the
    // clip rectangle above discards. See `GridViewport::visible_positions` for
    // why that makes the margin a boundary case rather than a seam that would
    // otherwise go missing.
    //
    // The bounds are the Render Frame's own Grid's, rather than a shape
    // recovered out of its rows.
    let visible = viewport.visible_positions(clip, source_grid);

    // What the console decided to draw, then what draws it. The decision is a
    // value derived from the Render Frame and the range above, so what colour a
    // Cell is can be asked without a `Context`, a window or a running Orcvs.
    let paint = Paint::derive_with_colours(
        FramePaint::new(frame, visible),
        cursor_effect_settings.cell_colour(),
        cursor_effect_settings.region_colour(),
        cursor_effect_settings.region_cursor_colour(),
        source_paint,
    );
    let cursor_rect = viewport.cell_rect(frame.cursor().x(), frame.cursor().y());
    let cursor_effect = cursor_effect_shapes(
        cursor_rect,
        effect_outline(frame, &viewport),
        clip,
        viewport.cell_size,
        cursor_effect_sample,
        cursor_effect_settings,
    );
    let shapes = SourceShapes::new(&paint, &viewport, &table, pixels_per_point, cursor_effect);

    // One `Painter::extend`, never a `Painter::add` per Shape. `add` reaches
    // `Context::graphics_mut`, which is a full `Context` write lock, so a
    // per-Cell loop would take more locks than the Button field it replaces and
    // turn this change into a regression.
    painter.extend(shapes.into_shapes());

    // The click resolves by division through the viewport the Cells were
    // painted at. With no layer transform, `interact_pointer_pos` is in global
    // points, which is the space the presented Grid is in.
    // Shift with a click extends the Region from its anchor; a click on its
    // own collapses it.
    if response.clicked()
        && let Some(pointer) = response.interact_pointer_pos()
        && let Some((column, row)) = viewport.cell_at(pointer, source_grid)
    {
        let extend = ui.input(|i| i.modifiers.shift);
        source_grid.position(column, row).map(|position| {
            if extend {
                PointerSelection::Extend(position)
            } else {
                PointerSelection::Select(position)
            }
        })
    } else {
        None
    }
}

///
/// What the pointer asked of the Region in one Render Frame.
///
#[derive(Clone, Copy, Debug, PartialEq)]
enum PointerSelection {
    /// A click: the Region collapses onto the Cell.
    Select(Position),
    /// Shift with a click: the Cursor moves to the Cell and the anchor stays.
    Extend(Position),
    /// A primary drag: the Region spans from the pressed Cell to the Cell
    /// nearest the pointer.
    Span { anchor: Position, cursor: Position },
}

impl PointerSelection {
    ///
    /// Asks `orcvs` for the Region this selection names. Where the Cursor and
    /// the anchor are is the running Orcvs's business; the pointer only
    /// answers which Cells it asked for.
    ///
    fn apply(self, orcvs: &mut Orcvs) {
        match self {
            Self::Select(position) => orcvs.select(position),
            Self::Extend(position) => orcvs.extend(position),
            Self::Span { anchor, cursor } => {
                orcvs.select(anchor);
                orcvs.extend(cursor);
            }
        }
    }
}

///
/// The Source as it was presented for one Render Frame: the geometry it was
/// drawn under, and the Cell a click asked for.
///
/// "Presented" is already the repository's word for this step —
/// `grid_viewport::presented_grid` uses it.
///
struct PresentedSource {
    /// The viewport the Cells were drawn at.
    viewport: GridViewport,
    /// The Region the viewer's click or drag asked for, for the caller that
    /// owns the Source to select.
    selection: Option<PointerSelection>,
}

///
/// Shows the Source in the console area at the Source View's Zoom and Pan, and
/// answers the geometry it was presented under along with the Region a click
/// or a drag asked for.
///
/// The console owns the scale and translation the Source is presented under —
/// `view.to_global` — and `grid_viewport::presented_grid` is the one place that
/// scale is applied, so a Cell's two axes still cannot part company: one
/// `scaling` serves both. Every Cell, and so every click that lands on one,
/// goes through that one arithmetic. Nothing here sets a layer transform.
/// See `docs/adr/0038-the-console-owns-the-source-grid-transform.md` and
/// `docs/adr/0045-the-source-view-is-a-bounded-space.md`.
///
/// Pan is by wheel or two-finger scroll, by middle-drag, and by Alt (Option)
/// held with a primary drag, bounded by the Grid's edges plus a margin of
/// [`SOURCE_MARGIN_CELLS`] (ADR 0047). Where the Grid and its margins leave
/// somewhere to Pan, the pointer shows a grab hand while Alt is held over the
/// console outside a Region drag, and a grabbing hand during a drag Pan. A primary
/// click selects a Cell — [`show_source`]'s own click-sensing rect answers it,
/// with Shift extending the Region rather than collapsing it — and a primary
/// drag without Alt selects a Region from the pressed Cell to the Cell nearest
/// the pointer; neither Pans. A drag past the console's edge scrolls after the
/// Cursor at the pointer's pace, at most one Cell a frame — see
/// `docs/adr/0046-the-primary-drag-selects-a-region.md`. Pinch and command-wheel do not
/// Zoom: Zoom is a command `=`, `+`, `-` or `0` chord from the keyboard
/// alone, stepped by [`GLYPH_SCALE_STEP`] and clamped to
/// [`MIN_ZOOM`]..=[`MAX_ZOOM`]. A Zoom that would open a gap past an edge
/// settles back inside through the same `clamp_pan` a Pan does.
///
/// A Cursor move or a Zoom that would leave the Cursor's Cell outside the
/// console Pans just far enough to bring it back, before that same
/// `clamp_pan` settles the result inside the Grid; a Pan with neither is not
/// pulled back to the Cursor. `frame` already carries a keyboard Cursor move
/// from this same Render Frame — `Console::ui` reads it after
/// `Orcvs::event_handler` runs — so that case is caught the frame it happens.
/// A click's or a drag's Cursor move reaches the Source only after this call
/// returns (`Console::ui` applies the [`PointerSelection`] next), so it is
/// followed on the frame after, not this one.
///
fn show_source_scene(
    ui: &mut egui::Ui,
    frame: &RenderFrame,
    font_family: &egui::FontFamily,
    view: &mut SourceView,
    cursor_effect_sample: CursorEffectSample,
    cursor_effect_settings: CursorEffectSettings,
    source_paint: SourcePaintSettings,
) -> PresentedSource {
    let source_grid = frame.grid();
    let source = source_bounds(source_grid);
    // `Sense::CLICK | Sense::DRAG` rather than `Sense::click_and_drag()`,
    // which adds `FOCUSABLE` (`egui-0.36.1/src/sense.rs:81-83`) and would let
    // Tab focus the console area, where a focused widget keeps every key from
    // the Source.
    let (console, mut pan) =
        ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::CLICK | Sense::DRAG);

    if !view.zoom.is_finite() || view.zoom <= 0.0 {
        view.zoom = 1.0;
    }
    view.zoom = view.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

    let zoom_before_command = view.zoom;
    if let Some(command) = ui.input(|i| i.events.iter().find_map(zoom_command)) {
        view.zoom = stepped_zoom(view.zoom, command);
    }
    let zoomed = view.zoom != zoom_before_command;

    // Middle-drag Pans outright; a primary drag Pans only with Alt (Option)
    // held, so a trackpad with no middle button still has a way to Pan by
    // dragging. Without Alt this branch is skipped: a primary gesture that
    // stayed inside the click threshold resolves to `show_source`'s own click
    // on release, and one that moved past it selects a Region below.
    //
    // A primary drag that began selecting a Region stays one if Alt is
    // pressed partway through it.
    let alt = ui.input(|i| i.modifiers.alt);
    let drag_panning = pan.dragged_by(PointerButton::Middle)
        || (pan.dragged_by(PointerButton::Primary) && view.region_drag.is_none() && alt);
    if drag_panning {
        view.pan += pan.drag_delta();
        pan.mark_changed();
    }

    if pan.contains_pointer() {
        let pan_delta = ui.input(|i| i.smooth_scroll_delta());
        if pan_delta != Vec2::ZERO {
            view.pan += pan_delta;
            pan.mark_changed();
        }
    }

    // A Cursor move is a change from the Cursor `previous_cursor` last saw,
    // not every frame the Cursor happens to be drawn — otherwise an ordinary
    // Pan with the Cursor already out of view would be pulled straight back
    // to it. `None` on a fresh `SourceView`'s first frame answers no move, so
    // the console does not Pan away from where it opened before anything has
    // moved the Cursor at all.
    let cursor = frame.cursor();
    let cursor_moved = view
        .previous_cursor
        .is_some_and(|previous| previous != cursor);
    view.previous_cursor = Some(cursor);

    // The Cell side `presented_grid` will draw at, snapped to whole physical
    // pixels, so the follow and the bounds below are measured against the
    // Grid as drawn rather than the unsnapped extent the Zoom asked for.
    let side = snapped_cell_side(CELL_SIZE * view.zoom, ui.ctx().pixels_per_point());

    // `view.pan` places the padded Source — the Grid with a margin on every
    // side — so a Pan of zero rests the Grid one margin in from the console's
    // top-left, and the clamp lets a Pan reach one margin past each far edge.
    let margin = Vec2::splat(SOURCE_MARGIN_CELLS * side);
    let cursor_at = followed_cell(cursor, source_grid, side);

    // A Region drag follows the Cursor at the pointer's pace rather than in
    // one jump, so the Source View scrolls after a pointer held past the edge
    // — the Cursor is the Cell nearest the pointer, one past the edge — and
    // keeps scrolling each frame the pointer stays there, which is why this
    // frame asks for the next.
    let pointer = ui.input(|i| i.pointer.interact_pos());
    // Still paced on the frame the button comes up — `region_drag` is cleared
    // only once this frame has answered it — so a release with the Cursor
    // many Cells past the edge does not jump the Source View to it.
    let dragging_region = view.region_drag.is_some();
    if zoomed || (cursor_moved && !dragging_region) {
        view.pan = follow_cursor(view.pan, console.size(), cursor_at);
    } else if dragging_region && let Some(pointer) = pointer {
        let past = overshoot(console, pointer);
        let wanted = follow_cursor(view.pan, console.size(), cursor_at) - view.pan;
        view.pan += edge_scroll(wanted, past, side);
        if past != Vec2::ZERO {
            ui.ctx().request_repaint();
        }
    }

    let source_size = Vec2::new(source_grid.columns() as f32, source_grid.rows() as f32) * side;
    let padded_size = source_size + 2.0 * margin;
    view.pan = clamp_pan(view.pan, console.size(), padded_size);

    // The pointer offers a Pan only where there is one: not on a Grid that
    // with its margins fits the console on both axes, and not for Alt
    // pressed partway through a Region drag, which stays one. Alt is the one
    // Pan gesture the pointer can announce before it starts: a middle-drag
    // has no hover state to show, and a wheel Pan is not a grab.
    let pannable = padded_size.x > console.width() || padded_size.y > console.height();
    if pannable && drag_panning {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    } else if pannable && alt && view.region_drag.is_none() && pan.contains_pointer() {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }

    let to_global = TSTransform::new(console.min.to_vec2() + view.pan + margin, view.zoom);
    view.to_global = if is_presentable(to_global) {
        to_global
    } else {
        TSTransform::IDENTITY
    };

    let grid = presented_grid(
        view.to_global,
        source,
        source_grid,
        ui.ctx().pixels_per_point(),
    );
    let clicked = show_source(
        ui,
        frame,
        font_family,
        grid,
        console,
        cursor_effect_sample,
        cursor_effect_settings,
        source_paint,
    );

    // A primary drag without Alt selects a Region: its anchor is the Cell the
    // press landed on, and a press off the Grid selects nothing. The drag is
    // decided once the pointer has moved past egui's click distance, so the
    // anchor is read from where the press began rather than where the
    // pointer is now.
    if pan.drag_started_by(PointerButton::Primary) && !ui.input(|i| i.modifiers.alt) {
        view.region_drag = ui
            .input(|i| i.pointer.press_origin())
            .and_then(|origin| grid.cell_at(origin, source_grid))
            .and_then(|(column, row)| source_grid.position(column, row));
    }
    let spanned = view
        .region_drag
        .filter(|_| pan.dragged_by(PointerButton::Primary))
        .zip(
            pointer
                .and_then(|pointer| grid.nearest_cell(pointer, source_grid))
                .and_then(|(column, row)| source_grid.position(column, row)),
        )
        .map(|(anchor, cursor)| PointerSelection::Span { anchor, cursor });
    // Answered on every frame of the drag, not only when the pointer moves:
    // a pointer held past the edge moves no further while the Source View
    // scrolls under it, so the Cell nearest it changes without an event.
    // Asking for the same Region again is idempotent, so a repeated frame
    // changes nothing.
    //
    // Release keeps the Region the drag last spanned, and spans no further:
    // a Cursor moved on the release frame would be followed in full on the
    // next, once the drag no longer paces it.
    if !pan.dragged_by(PointerButton::Primary) {
        view.region_drag = None;
    }

    PresentedSource {
        viewport: grid,
        selection: spanned.or(clicked),
    }
}

///
/// The frame the Source Grid is painted on.
///
/// The fill is load-bearing rather than decorative. `cell_visuals` answers
/// `None` for a Cell's background wherever the panel has already painted
/// `background`, on the grounds that this frame has already painted exactly
/// that colour across the whole console and clips every Shape to it. An ordinary
/// Cell therefore has no rectangle of its own.
///
/// `background` is the live `SourcePaintSettings::source_background`, not a
/// constant: a viewer's `Theme → Source colours` edit has to repaint this
/// panel on the very next frame for the Cell it stands in for to still agree
/// with it.
///
/// It is a function rather than a literal at the panel so the painting tests
/// render on the same ground production does, and so
/// `the_omitted_background_is_the_colour_the_panel_is_filled_with` has one
/// value to pin instead of a comment to trust.
///
fn source_panel_frame(background: Color32) -> egui::Frame {
    egui::Frame::new().fill(background)
}

fn bottom_panel_frame(style: &egui::Style) -> egui::Frame {
    let mut frame = egui::Frame::side_top_panel(style);
    frame.inner_margin.left += BOTTOM_PANEL_LEFT_PAD;
    frame
}

impl eframe::App for Console {
    ///
    /// Called by the framework to save state before shutdown, and at
    /// intervals while running. This stores the current Source revision and
    /// the persisted presentation settings.
    ///
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.persistence.save(
            storage,
            self.orcvs.source(),
            self.cursor_effects,
            self.source_paint,
        );
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, root: &mut egui::Ui, eframe: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        // Tab belongs to the Source while it holds the keys. `Memory::begin_pass`
        // already turned an unmodified Tab into `FocusDirection::Next` and a
        // Shift Tab into `FocusDirection::Previous` before this runs
        // (`egui-0.36.1/src/memory/mod.rs:596-597`), and the first focusable
        // widget shown below — a menu-bar button — would otherwise claim it
        // the moment it is shown. Cancelling here, before anything is shown,
        // is what keeps Tab off every widget rather than only the ones drawn
        // after this line. `!self.keyboard_elsewhere` is last frame's answer,
        // the same one the event routing below reads, so a focused control or
        // an open menu keeps egui's own Tab navigation and the Source gets
        // none of it — a viewer in a menu reaches its controls by Tab alone,
        // because a pointer click there closes the menu first.
        if !self.keyboard_elsewhere {
            ctx.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
        }
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
        let top_panel = egui::Panel::top("top_panel")
            .resizable(true)
            .min_size(TOP_PANEL_HEIGHT);

        top_panel.show(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if ui.button("Load Function reference").clicked() {
                        self.load_function_reference();
                    }
                    if !is_web {
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.add_space(16.0);
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.diagnostics_open, "Diagnostics");
                });
                ui.menu_button("Theme", |ui| {
                    ui.label("Cursor effects");
                    ui.horizontal(|ui| {
                        ui.label("Cursor colour");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.cursor_effects.cursor_colour_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Area colour");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.cursor_effects.area_colour_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Region colour");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.cursor_effects.region_colour_mut(),
                            egui::color_picker::Alpha::OnlyBlend,
                        );
                    });
                    let mut region_cursor_enabled =
                        self.cursor_effects.region_cursor_colour().is_some();
                    if ui
                        .checkbox(&mut region_cursor_enabled, "Cursor colour in a Region")
                        .changed()
                    {
                        self.cursor_effects.set_region_cursor_colour(
                            region_cursor_enabled
                                .then_some(crate::cursor_effects::DEFAULT_REGION_COLOUR),
                        );
                    }
                    if let Some(mut colour) = self.cursor_effects.region_cursor_colour()
                        && egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut colour,
                            egui::color_picker::Alpha::OnlyBlend,
                        )
                        .changed()
                    {
                        self.cursor_effects.set_region_cursor_colour(Some(colour));
                    }
                    let mut cell_colour_enabled = self.cursor_effects.cell_colour().is_some();
                    if ui
                        .checkbox(&mut cell_colour_enabled, "Cursor cell colour")
                        .changed()
                    {
                        self.cursor_effects.set_cell_colour(if cell_colour_enabled {
                            Some(DEFAULT_CURSOR_COLOUR)
                        } else {
                            None
                        });
                    }
                    if let Some(mut colour) = self.cursor_effects.cell_colour()
                        && egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut colour,
                            egui::color_picker::Alpha::Opaque,
                        )
                        .changed()
                    {
                        self.cursor_effects.set_cell_colour(Some(colour));
                    }
                    ui.add(
                        egui::Slider::new(self.cursor_effects.amount_mut(), 0..=100)
                            .text("Glitch amount"),
                    );
                    ui.add(
                        egui::Slider::new(self.cursor_effects.frequency_mut(), 0..=100)
                            .text("Glitch frequency"),
                    );
                    ui.separator();
                    if ui.button("Reset to theme defaults").clicked() {
                        self.cursor_effects = CursorEffectSettings::default();
                    }

                    ui.separator();
                    ui.label("Source colours");
                    ui.horizontal(|ui| {
                        ui.label("Source background");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.source_background_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Ordinary (Char, Atom)");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.ordinary_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Comment");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.comment_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Function");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.function_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Bang");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.bang_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Number");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.number_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Note");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.note_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Sequence");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.sequence_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Diagnostic");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.diagnostic_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Output Portal");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            self.source_paint.output_portal_mut(),
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                    ui.add(
                        egui::Slider::new(self.source_paint.fill_tint_mut(), 0..=100)
                            .text("Fill tint"),
                    );
                    ui.separator();
                    // Its own reset, independent of Cursor effects' above: it
                    // only ever assigns `self.source_paint`, so a Source
                    // colours reset cannot move a Cursor effect and a Cursor
                    // effects reset cannot move a Source colour.
                    if ui.button("Reset to theme defaults").clicked() {
                        self.source_paint = SourcePaintSettings::default();
                    }
                });
                // Presented in the menu bar rather than the Diagnostics window,
                // which opens on a viewer's request and reports the running
                // frame. This is a start-up answer about the Source in front of
                // them, and it stays until they dismiss it. `report` reaches a
                // developer console; this is the channel a viewer reads.
                #[cfg(feature = "persistence")]
                if self.persistence.notice_visible() {
                    ui.add_space(16.0);
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        format!(
                            "Stored Source could not be read back; it was kept under \
                             \"{}\"",
                            crate::persistence::REFUSED_KEY
                        ),
                    );
                    if ui.button("Dismiss").clicked() {
                        self.persistence.dismiss_notice();
                    }
                }
            });
        });

        let event_filter = EventFilter {
            tab: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            escape: true,
        };

        // Keys belong to whichever control held focus when last frame's
        // widgets were done. Last frame's, because this frame's events are
        // collected before the widgets are shown and would otherwise write
        // Source or toggle Playback in the same pass a control is already
        // editing. egui offers no per-event answer to whether a widget used
        // an event — `TextEdit` reads its events without consuming them
        // (`egui-0.36.1/src/widgets/text_edit/builder.rs:1081`) — and names
        // `egui_wants_keyboard_input` as the question to ask instead
        // (`egui-0.36.1/src/data/input/raw_input.rs:56-60`).
        if ctx.memory(|memory| memory.had_focus_last_frame(egui::Id::new(BPM_FIELD_ID))) {
            ctx.input_mut(|i| keep_digits_in_text_events(&mut i.events));
        }
        if !self.keyboard_elsewhere {
            // A command Zoom chord answers `show_source_scene`, not the
            // Source. `egui-winit` and eframe's web backend both withhold
            // `Event::Text` while a command modifier is held
            // (`egui-winit-0.36.1/src/lib.rs:1059-1065`,
            // `eframe-0.36.1/src/web/events.rs:155-162`), so a shipped build
            // never raises the matching bare character alongside the chord
            // that already answered it.
            let events = ctx.input(|i| {
                i.filtered_events(&event_filter)
                    .into_iter()
                    .filter_map(translate_event)
                    .collect()
            });
            // A Copy or Cut answers the text the platform clipboard is to
            // hold; `copy_text` is how egui hands it to the native and the
            // browser backend alike.
            if let Some(copied) = self.orcvs.event_handler(events).copied {
                ctx.copy_text(copied);
            }
        } else {
            // Keys a control took are still the event that follows a command
            // Enter, so they disarm its fill as one reaching the Source would.
            self.orcvs.disarm_fill();
        }
        let frame = self.orcvs.render_frame();
        let observation = self.orcvs.playback_observation();
        let sampled_run_clock = observation.run_clock();
        let moving_run_clock = (observation.state == PlaybackState::Playing
            && observation.run_started_at.is_some())
        .then_some(sampled_run_clock);
        let effect_now = Duration::from_secs_f64(ctx.input(|input| input.time).max(0.0));
        let cursor_effect_settings = self
            .cursor_effects
            .respecting_reduced_motion(self.reduced_motion);
        let cursor_effect_sample = self
            .cursor_effect_animation
            .advance(effect_now, cursor_effect_settings);
        let source_paint = self.source_paint;

        // Shown before CentralPanel so it takes height rather than overlaying
        // the Grid. Static: no resize handle, no drag. BPM is a TextEdit:
        // click to type; Enter or leaving the field commits. `**` is the beat, `//`
        // while Playback is stopped. Tick and Run Clock are the engine's
        // published Readouts. Destination is chosen from the ComboBox;
        // Scan asks the engine to discover again. There is no periodic polling.
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
                        let clock_text = format_run_clock(sampled_run_clock);
                        let clock_width =
                            monospace_width(ui, "00:00").max(monospace_width(ui, &clock_text));
                        reserved_monospace(ui, &clock_text, clock_width);
                        ui.add_space(entry_gap);
                        panel_label(ui, "O");
                        ui.add_space(label_value_gap);

                        self.midi.auto_select_first_if_unselected();
                        let destinations = self.midi.destinations().to_vec();
                        let selected_id = self.midi.selected_destination_id();
                        let presentation =
                            destination_presentation(&destinations, selected_id.as_ref());
                        ui.add_enabled_ui(presentation.enabled, |ui| {
                            apply_panel_field_spacing(ui);
                            let mut selected = selected_id.clone();
                            let combo_response = egui::ComboBox::from_id_salt(DESTINATION_COMBO_ID)
                                .selected_text(
                                    egui::RichText::new(presentation.selected_text)
                                        .text_style(egui::TextStyle::Monospace),
                                )
                                .width(OUTPUT_READOUT_WIDTH)
                                .icon(|_ui, _rect, _visuals, _is_open| {})
                                .show_ui(ui, |ui| {
                                    if presentation.show_refresh {
                                        if ui.button(OUTPUT_SCAN).clicked() {
                                            self.midi.refresh_destinations();
                                        }
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
                                        for destination in &destinations {
                                            ui.selectable_value(
                                                &mut selected,
                                                Some(destination.id.clone()),
                                                destination.name.as_str(),
                                            );
                                        }
                                    }
                                });
                            if combo_response.response.clicked()
                                && presentation.show_refresh
                                && destinations.is_empty()
                            {
                                self.midi.refresh_destinations();
                            }
                            if selected != selected_id
                                && let Some(id) = selected.as_ref()
                            {
                                self.midi.select_destination(id);
                            }
                        });
                        if let Some(status) = self.midi.status() {
                            ui.colored_label(ui.visuals().error_fg_color, status);
                        }
                    },
                );
            });

        let mut console_area = Rect::ZERO;
        let mut cell_size = 0.0;
        let cursor_delay = egui::CentralPanel::default()
            .frame(source_panel_frame(source_paint.source_background()))
            .show(root, |ui| {
                console_area = ui.available_rect_before_wrap();
                let Console {
                    orcvs,
                    midi: _,
                    font_family,
                    source_view,
                    diagnostics_open: _,
                    #[cfg(test)]
                        bpm_widget_id: _,
                    keyboard_elsewhere: _,
                    cursor_effects: _,
                    cursor_effect_animation: _,
                    source_paint: _,
                    reduced_motion: _,
                    #[cfg(feature = "persistence")]
                        persistence: _,
                } = self;
                let presented = show_source_scene(
                    ui,
                    &frame,
                    font_family,
                    source_view,
                    cursor_effect_sample,
                    cursor_effect_settings,
                    source_paint,
                );
                cell_size = presented.viewport.cell_size;
                // The Source Grid answers which Cells the pointer asked for;
                // moving the Cursor and the anchor there is the Source's own
                // business, and this is where the running Orcvs is owned.
                if let Some(selection) = presented.selection {
                    selection.apply(orcvs);
                }

                let cursor_rect = presented
                    .viewport
                    .cell_rect(frame.cursor().x(), frame.cursor().y());
                let outline = effect_outline(&frame, &presented.viewport);
                if effect_bounds(cursor_rect, presented.viewport.cell_size)
                    .union(outline.expand(presented.viewport.cell_size))
                    .intersects(console_area)
                {
                    self.cursor_effect_animation
                        .repaint_after(effect_now, cursor_effect_settings)
                } else {
                    None
                }
            })
            .inner;

        if let Some(delay) = until_next(
            moving_run_clock,
            cursor_delay,
            Duration::from_secs_f32(ctx.input(|input| input.predicted_dt).max(0.0)),
        ) {
            ctx.request_repaint_after(delay);
        }

        if self.diagnostics_open {
            show_diagnostics(
                &ctx,
                &mut self.diagnostics_open,
                eframe,
                self.source_view.to_global,
                console_area,
                cell_size,
            );
        }

        // Sampled once every widget, the Diagnostics window's included, has
        // been shown and has taken or surrendered focus, and every popup has
        // opened or closed.
        self.keyboard_elsewhere = ctx.egui_wants_keyboard_input() || egui::Popup::is_any_open(&ctx);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod kittest_tests;

#[cfg(test)]
mod tests {
    use egui::{
        Color32, Event, Key, Modifiers, MouseWheelUnit, Pos2, Rect, Shape, TouchPhase, Vec2,
        emath::GuiRounding as _, emath::TSTransform,
    };
    use orcvs::app::{Arrow, InputEvent, InputKey, Orcvs};
    use orcvs::render_frame::RenderFrame;

    use crate::grid_viewport::{CELL_SIZE, GridViewport, presented_grid};
    use crate::paint::{FramePaint, Paint};
    use crate::style::PALETTE;
    use orcvs::grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT, Grid};

    use super::{
        ALPHABET_FIRST, ALPHABET_LAST, BOTTOM_PANEL_HEIGHT, BOTTOM_PANEL_LEFT_PAD,
        BPM_FIELD_MARGIN, Console, DEFAULT_FONT_SIZE, DEFAULT_VIEW_SIZE, GLYPH_SCALE_STEP,
        GRID_LINE_WIDTH, GlyphTable, MAX_ZOOM, MIN_ZOOM, SECTOR_LINE_WIDTH, SOURCE_MARGIN_CELLS,
        SourceShapes, SourceView, TOP_PANEL_HEIGHT, ZoomCommand, clamp_pan, frames_per_second,
        glyph_scale, is_presentable, show_source_scene, source_bounds, source_panel_frame,
        stepped_zoom, translate_event, zoom_command,
    };

    /// The Source View's margin at Zoom 1.0 and a device scale of one.
    const MARGIN: f32 = SOURCE_MARGIN_CELLS * CELL_SIZE;

    fn key_event(key: Key, pressed: bool) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    fn modified_key_event(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn command_key_event(key: Key) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        }
    }

    ///
    /// A command Zoom chord for `key`, as `pinch_at` and `command_wheel_at`
    /// stage a pointer gesture: the one event a real `=`/`+`/`-`/`0` press
    /// under a held command modifier delivers, with no accompanying
    /// `Event::Text` — `egui-winit` and eframe's web backend both withhold it
    /// while a command modifier is held.
    ///
    fn command_zoom_at(key: Key) -> Vec<Event> {
        vec![command_key_event(key)]
    }

    #[test]
    fn toolkit_events_translate_only_the_input_orcvs_handles() {
        let cases = [
            (Key::ArrowDown, InputKey::ArrowDown),
            (Key::ArrowLeft, InputKey::ArrowLeft),
            (Key::ArrowRight, InputKey::ArrowRight),
            (Key::ArrowUp, InputKey::ArrowUp),
            (Key::Backspace, InputKey::Backspace),
            (Key::Delete, InputKey::Delete),
            (Key::Space, InputKey::Space),
            (Key::Tab, InputKey::Tab),
        ];
        for (egui_key, orcvs_key) in cases {
            assert_eq!(
                translate_event(key_event(egui_key, true)),
                Some(InputEvent::KeyPressed(orcvs_key))
            );
        }

        // Shift Tab is its own InputEvent rather than the bare Tab above, as
        // Shift with an arrow is: the Cursor steps forward by Sector on one
        // and back on the other, and only the modifier tells them apart.
        assert_eq!(
            translate_event(modified_key_event(Key::Tab, Modifiers::SHIFT)),
            Some(InputEvent::PreviousSector)
        );

        // Only a bare Tab and a bare Shift Tab are the Source's: Ctrl,
        // Command, or Alt held with Tab is some other shortcut (or nothing)
        // and must not also step the Cursor.
        for modifiers in [
            Modifiers::COMMAND,
            Modifiers::CTRL,
            Modifiers::ALT,
            Modifiers::COMMAND | Modifiers::SHIFT,
            Modifiers::ALT | Modifiers::SHIFT,
        ] {
            assert_eq!(
                translate_event(modified_key_event(Key::Tab, modifiers)),
                None,
                "{modifiers:?} held with Tab translated to Source input"
            );
        }

        assert_eq!(
            translate_event(Event::Text("x".to_owned())),
            Some(InputEvent::Text("x".to_owned()))
        );
        assert_eq!(translate_event(key_event(Key::Enter, true)), None);
        assert_eq!(translate_event(key_event(Key::ArrowDown, false)), None);

        // Shift with an arrow extends the Region; command `A` spans the Grid;
        // Escape collapses the Region onto the Cursor.
        let shifted = [
            (Key::ArrowDown, Arrow::Down),
            (Key::ArrowLeft, Arrow::Left),
            (Key::ArrowRight, Arrow::Right),
            (Key::ArrowUp, Arrow::Up),
        ];
        for (egui_key, arrow) in shifted {
            assert_eq!(
                translate_event(modified_key_event(egui_key, Modifiers::SHIFT)),
                Some(InputEvent::Extend(arrow))
            );
        }
        assert_eq!(
            translate_event(command_key_event(Key::A)),
            Some(InputEvent::SelectAll)
        );
        // Command Enter arms a fill; a bare Enter is still nothing.
        assert_eq!(
            translate_event(command_key_event(Key::Enter)),
            Some(InputEvent::Fill)
        );
        assert_eq!(translate_event(key_event(Key::A, true)), None);
        assert_eq!(
            translate_event(key_event(Key::Escape, true)),
            Some(InputEvent::Collapse)
        );
        // The clipboard arrives as its own events, never as the characters
        // of the chord that raised it.
        assert_eq!(translate_event(Event::Copy), Some(InputEvent::Copy));
        assert_eq!(translate_event(Event::Cut), Some(InputEvent::Cut));
        assert_eq!(
            translate_event(Event::Paste("ab".to_owned())),
            Some(InputEvent::Paste("ab".to_owned()))
        );
        for key in [Key::C, Key::X, Key::V] {
            assert_eq!(translate_event(command_key_event(key)), None);
        }

        // Bare `+`, `-`, `=` and `0` are Source characters: the toolkit
        // reports them as `Event::Text`, which `translate_event` reaches
        // regardless of what key produced it, and never as one of the Key
        // variants matched above.
        for character in ["+", "-", "=", "0"] {
            assert_eq!(
                translate_event(Event::Text(character.to_owned())),
                Some(InputEvent::Text(character.to_owned())),
                "bare {character:?} did not reach the Source as Cell input"
            );
        }
        for key in [Key::Equals, Key::Plus, Key::Minus, Key::Num0] {
            assert_eq!(
                translate_event(key_event(key, true)),
                None,
                "bare {key:?} was translated as Source input on its own"
            );
        }
    }

    ///
    /// The command chord [`zoom_command`] answers, and the bare key it never
    /// answers for: `=`, `+`, `-` and `0` ask for a Zoom only with
    /// [`egui::Modifiers::command`] held, and an unrelated command chord asks
    /// for nothing.
    ///
    #[test]
    fn only_a_command_chord_of_the_four_keys_asks_for_a_zoom() {
        let cases = [
            (Key::Equals, ZoomCommand::In),
            (Key::Plus, ZoomCommand::In),
            (Key::Minus, ZoomCommand::Out),
            (Key::Num0, ZoomCommand::Reset),
        ];
        for (key, command) in cases {
            assert_eq!(
                zoom_command(&command_key_event(key)),
                Some(command),
                "command {key:?} did not ask for a Zoom"
            );
            assert_eq!(
                zoom_command(&key_event(key, true)),
                None,
                "bare {key:?} asked for a Zoom"
            );
            assert_eq!(
                zoom_command(&Event::Key {
                    key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Modifiers::COMMAND,
                }),
                None,
                "a released command {key:?} asked for a Zoom"
            );
        }

        assert_eq!(
            zoom_command(&command_key_event(Key::C)),
            None,
            "an unrelated command chord asked for a Zoom"
        );
    }

    ///
    /// A Zoom step is exact: `In` and `Out` move by one [`GLYPH_SCALE_STEP`]
    /// from the nearest multiple of it, `Reset` always lands on 1.0, and every
    /// step stops at [`MIN_ZOOM`] or [`MAX_ZOOM`] rather than passing it.
    ///
    #[test]
    fn a_zoom_step_moves_by_one_step_and_stops_at_the_range() {
        assert_eq!(stepped_zoom(1.0, ZoomCommand::In), 1.125);
        assert_eq!(stepped_zoom(1.0, ZoomCommand::Out), 0.875);
        assert_eq!(stepped_zoom(1.375, ZoomCommand::Reset), 1.0);
        assert_eq!(stepped_zoom(MIN_ZOOM, ZoomCommand::Reset), 1.0);

        assert_eq!(stepped_zoom(MAX_ZOOM, ZoomCommand::In), MAX_ZOOM);
        assert_eq!(stepped_zoom(MIN_ZOOM, ZoomCommand::Out), MIN_ZOOM);

        // Every step a keyboard Zoom can reach is a whole number of eighths,
        // and `glyph_scale` — the atlas budget `GLYPH_SCALE_STEP` states —
        // has to floor every one of them to itself rather than to the step
        // below.
        let mut zoom = MIN_ZOOM;
        let mut steps = 0;
        while zoom < MAX_ZOOM {
            let stepped = stepped_zoom(zoom, ZoomCommand::In);
            assert!(
                stepped > zoom,
                "In did not move the Zoom forward from {zoom}"
            );
            assert_eq!(
                glyph_scale(stepped),
                stepped,
                "the Glyph was not laid out at the Cell size of the {stepped} step"
            );
            zoom = stepped;
            steps += 1;
        }
        assert_eq!(
            steps, 14,
            "the keyboard range holds fifteen steps, not {steps} moves between them"
        );
    }

    ///
    /// The frame rate is still derived; the Source zoom no longer is. Under an
    /// owned transform the zoom the diagnostics show *is* `scaling`, so what
    /// used to be a ratio recovered from a Scene-space region collapsed to a
    /// field read and `scene_zoom` went with it. What is left to assert is that
    /// the field the diagnostics read is never a value they cannot show: the
    /// guard is what makes the read safe.
    ///
    #[test]
    fn diagnostics_derive_frame_rate_and_read_the_source_zoom_from_the_owned_transform() {
        assert_eq!(frames_per_second(0.02), Some(50.0));
        assert_eq!(frames_per_second(0.0), None);

        let zoomed = TSTransform::new(Vec2::new(11.0, 7.0), 2.0);
        assert!(is_presentable(zoomed));
        assert_eq!(zoomed.scaling, 2.0);
        // The visible Source region the diagnostics show is the console area
        // read back through the transform, which is what the Scene-space
        // rectangle used to hold directly.
        assert_eq!(
            zoomed.inverse() * Rect::from_min_size(Pos2::new(11.0, 7.0), Vec2::new(800.0, 400.0)),
            Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))
        );

        for unpresentable in [
            TSTransform::from_scaling(0.0),
            TSTransform::from_scaling(-1.0),
            TSTransform::from_scaling(f32::NAN),
            TSTransform::from_translation(Vec2::new(0.0, f32::NAN)),
        ] {
            assert!(
                !is_presentable(unpresentable),
                "{unpresentable:?} reached the diagnostics"
            );
        }
    }

    ///
    /// At Zoom 1.0 the Glyph is 11.5 points inside the Source's 16 point Cell.
    ///
    #[test]
    fn the_glyph_at_zoom_one_is_eleven_point_five_points() {
        assert_eq!(DEFAULT_FONT_SIZE * glyph_scale(1.0), 11.5);
    }

    ///
    /// The step the glyph scale is quantised to, and the budget it implies.
    /// Both zoom limits and the Source's own scale land on a step, so the
    /// default window lays its Glyphs out at exactly the Source's font size.
    ///
    #[test]
    fn the_glyph_scale_is_quantised_to_a_stated_step() {
        assert_eq!(glyph_scale(1.0), 1.0);
        assert_eq!(glyph_scale(MIN_ZOOM), MIN_ZOOM);
        assert_eq!(glyph_scale(MAX_ZOOM), MAX_ZOOM);
        assert_eq!(glyph_scale(1.01), 1.0, "a nudge re-laid the whole alphabet");
        // Downwards, so the Glyph keeps its share of the Cell: a zoom part way
        // into a step is laid out at the step it is past, not the one it is
        // approaching.
        assert_eq!(glyph_scale(1.1), 1.0);
        assert_eq!(glyph_scale(1.13), 1.125);
        // Never zero, never negative, whatever reaches it.
        for degenerate in [0.0, -1.0, f32::NAN, f32::INFINITY, 1e-9] {
            assert!(
                glyph_scale(degenerate) >= GLYPH_SCALE_STEP,
                "{degenerate} laid out at a font size of {}",
                glyph_scale(degenerate)
            );
        }

        // Fifteen distinct sizes over the whole zoom range is the atlas budget
        // `GLYPH_SCALE_STEP` states. Swept in exact thousandths rather than by
        // accumulating one: the top of the range is reached by the
        // `zoom_range` clamp exactly, and a sum that drifts past it would drop
        // the step it lands on.
        let mut sizes: Vec<f32> = Vec::new();
        for thousandth in (MIN_ZOOM * 1_000.0) as u32..=(MAX_ZOOM * 1_000.0) as u32 {
            let scale = glyph_scale(thousandth as f32 / 1_000.0);
            if !sizes.iter().any(|held| (held - scale).abs() < 1e-6) {
                sizes.push(scale);
            }
        }
        assert_eq!(
            sizes.len(),
            15,
            "the zoom range holds {} sizes",
            sizes.len()
        );
    }

    ///
    /// A Glyph is never laid out at a larger fraction of its Cell than the fit
    /// gave it.
    ///
    /// The quantisation step is an absolute one, so rounding to the nearest
    /// step is disproportionate at a small scale: a console fitting at 0.2
    /// rounds up to 0.25 and lays an 11.5 point Glyph out at 2.875 points inside a
    /// 3.2 point Cell, where the same Glyph at the Source's own scale takes 11.5 of
    /// 16. Under the retired Scene the layer scaled the Glyph exactly, so this
    /// is the proportion the effort's strict-parity rule is about. Quantising
    /// downwards keeps it and costs at most one step of sharpness.
    ///
    /// The floor at [`GLYPH_SCALE_STEP`] is the one deliberate exception, and
    /// the case above it is what this pins.
    ///
    #[test]
    fn a_glyph_is_never_laid_out_larger_than_the_scale_it_is_drawn_at() {
        // A sweep at half the step, so it lands both on steps and between them.
        let mut scaling = GLYPH_SCALE_STEP;
        while scaling <= MAX_ZOOM {
            assert!(
                glyph_scale(scaling) <= scaling,
                "a Glyph at {scaling} was laid out at {}",
                glyph_scale(scaling)
            );
            scaling += GLYPH_SCALE_STEP / 2.0;
        }
        // The fit below the zoom floor is where the rounding was worst.
        assert_eq!(glyph_scale(0.2), 0.125);
    }

    ///
    /// One Render Frame, and everything it painted in paint order.
    ///
    /// The shapes are flattened: what an assertion is about is the order the
    /// Source Grid was painted in, not how deeply a `Shape::Vec` wrapped it.
    ///
    fn console_pass(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) -> (GridViewport, Vec<Shape>) {
        console_pass_at(ctx, screen, events, orcvs, view, 1.0)
    }

    ///
    /// The same pass at a stated device scale.
    ///
    /// The scale is given as the viewport's `native_pixels_per_point` rather
    /// than through `Context::set_pixels_per_point`, which sets the zoom factor
    /// instead and rewrites the next pass's `screen_rect` from the previous
    /// one's to avoid jitter (`egui-0.36.1/src/context.rs:436-446`) — so the
    /// console would not be the size the caller asked for.
    ///
    fn console_pass_at(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
        pixels_per_point: f32,
    ) -> (GridViewport, Vec<Shape>) {
        let frame = orcvs.render_frame();
        let mut presented = None;
        let mut input = egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .expect("the root viewport")
            .native_pixels_per_point = Some(pixels_per_point);
        let output = ctx.run_ui(input, |root| {
            egui::CentralPanel::default()
                .frame(source_panel_frame(
                    crate::source_paint::SourcePaintSettings::default().source_background(),
                ))
                .show(root, |ui| {
                    presented = Some(show_source_scene(
                        ui,
                        &frame,
                        &egui::FontFamily::Monospace,
                        view,
                        crate::cursor_effects::CursorEffectSample::default(),
                        crate::cursor_effects::CursorEffectSettings::default(),
                        crate::source_paint::SourcePaintSettings::default(),
                    ));
                });
        });
        let mut painted = Vec::new();
        for clipped in &output.shapes {
            flatten(clipped.shape.clone(), &mut painted);
        }
        output.drop_without_applying_deltas();
        let presented = presented.expect("the central panel showed the Source");
        // What `Console::ui` does with the answer, done here for the same
        // reason: the Source Grid reports the Cell and the caller that owns the
        // running Orcvs moves the Cursor to it.
        if let Some(selection) = presented.selection {
            selection.apply(orcvs);
        }

        (presented.viewport, painted)
    }

    fn flatten(shape: Shape, into: &mut Vec<Shape>) {
        match shape {
            Shape::Vec(shapes) => {
                for shape in shapes {
                    flatten(shape, into);
                }
            }
            shape => into.push(shape),
        }
    }

    fn console_frame(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) -> GridViewport {
        console_pass(ctx, screen, events, orcvs, view).0
    }

    fn click_at(point: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(point),
            Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    fn release_at(point: Pos2) -> Vec<Event> {
        vec![Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }]
    }

    fn click(
        ctx: &egui::Context,
        screen: Rect,
        point: Pos2,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) {
        console_frame(ctx, screen, click_at(point), orcvs, view);
        console_frame(ctx, screen, release_at(point), orcvs, view);
    }

    ///
    /// The middle button pressed at `point`, which is the button that pans.
    ///
    fn middle_press_at(point: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(point),
            Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Middle,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    ///
    /// The primary button pressed at `point` while Alt (Option) is held —
    /// the gesture that Pans without a middle button.
    ///
    /// `Event::ModifiersChanged` first, the way a real backend reports the
    /// Option key going down: `ui.input(|i| i.modifiers)` is carried across
    /// frames from that event alone (`egui`'s own `InputState::begin_pass`),
    /// not from a `PointerButton` event's own `modifiers` field, so a
    /// `PointerMoved`-only frame later in the same drag still reads Alt as
    /// held only because of this.
    ///
    fn alt_primary_press_at(point: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(point),
            Event::ModifiersChanged(Modifiers::ALT),
            Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::ALT,
            },
        ]
    }

    ///
    /// Alt released along with the primary button at `point`, ending an
    /// Alt-drag Pan.
    ///
    fn alt_primary_release_at(point: Pos2) -> Vec<Event> {
        vec![
            Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::ALT,
            },
            Event::ModifiersChanged(Modifiers::NONE),
        ]
    }

    ///
    /// A pinch over `point`. Zoom is from the keyboard alone, so this must
    /// leave the Cell size where it was.
    ///
    fn pinch_at(point: Pos2) -> Vec<Event> {
        vec![Event::PointerMoved(point), Event::Zoom(1.2)]
    }

    ///
    /// A two-finger or wheel Pan over `point`.
    ///
    fn wheel_at(point: Pos2, delta: Vec2) -> Vec<Event> {
        vec![
            Event::PointerMoved(point),
            Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta,
                phase: TouchPhase::Move,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    ///
    /// A command-wheel over `point`. Zoom is from the keyboard alone, so this
    /// must leave the Cell size where it was.
    ///
    fn command_wheel_at(point: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(point),
            Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                delta: Vec2::new(0.0, 80.0),
                phase: TouchPhase::Move,
                modifiers: Modifiers::COMMAND,
            },
        ]
    }

    fn double_click(
        ctx: &egui::Context,
        screen: Rect,
        point: Pos2,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) {
        click(ctx, screen, point, orcvs, view);
        click(ctx, screen, point, orcvs, view);
    }

    ///
    /// A point in the surplus the viewport does not cover, if there is any.
    ///
    fn surplus(screen: Rect, viewport: Rect) -> Option<Pos2> {
        if screen.width() > viewport.width() + 1.0 {
            Some(Pos2::new(
                (viewport.right() + screen.right()) / 2.0,
                viewport.center().y,
            ))
        } else if screen.height() > viewport.height() + 1.0 {
            Some(Pos2::new(
                viewport.center().x,
                (viewport.bottom() + screen.bottom()) / 2.0,
            ))
        } else {
            None
        }
    }

    ///
    /// Wheels by `delta` at `point` frame after frame until the Pan stops
    /// moving, so a test asserts where a wheel Pan settles rather than how far
    /// `smooth_scroll_delta` hands it out in any one frame.
    ///
    fn wheel_until_settled(
        ctx: &egui::Context,
        screen: Rect,
        point: Pos2,
        delta: Vec2,
        orcvs: &mut Orcvs,
        view: &mut SourceView,
    ) {
        for _ in 0..64 {
            let before = view.pan;
            console_frame(ctx, screen, wheel_at(point, delta), orcvs, view);
            if view.pan == before {
                return;
            }
        }
        panic!(
            "a wheel Pan was still moving after 64 frames: {:?}",
            view.pan
        );
    }

    fn running_orcvs(cols: usize, rows: usize) -> Orcvs {
        Orcvs::new(cols, rows).expect("the test runtime")
    }

    fn selected_cell(orcvs: &Orcvs) -> (usize, usize) {
        let cursor = orcvs.render_frame().cursor();
        (cursor.x(), cursor.y())
    }

    #[tokio::test]
    async fn a_click_selects_the_cell_under_the_pointer_whatever_the_window_size() {
        for screen_size in [
            Vec2::new(400.0, 200.0),
            Vec2::new(200.0, 400.0),
            Vec2::new(600.0, 300.0),
        ] {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
            let mut orcvs = running_orcvs(4, 4);
            let mut view = SourceView::default();

            let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            assert_eq!(selected_cell(&orcvs), (0, 0));
            assert_eq!(
                viewport.cell_size, CELL_SIZE,
                "a {screen_size:?} console opened at a Cell size other than the Source's own"
            );

            let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
            click(&ctx, screen, target, &mut orcvs, &mut view);

            assert_eq!(
                selected_cell(&orcvs),
                (3, 1),
                "a click at {target:?} in a {screen_size:?} console"
            );
        }
    }

    ///
    /// A console too small for the Source still opens at Zoom 1.0 and shows the
    /// top-left of the Grid. A click still selects the Cell under the pointer.
    ///
    #[tokio::test]
    async fn a_click_selects_the_cell_under_the_pointer_in_a_console_smaller_than_the_source() {
        for screen_size in [Vec2::new(400.0, 102.0), Vec2::new(102.0, 400.0)] {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
            let mut orcvs = running_orcvs(32, 32);
            let mut view = SourceView::default();

            let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            assert_eq!(
                viewport.cell_size, CELL_SIZE,
                "a {screen_size:?} console opened at a Cell size other than the Source's own"
            );

            let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
            click(&ctx, screen, target, &mut orcvs, &mut view);

            assert_eq!(
                selected_cell(&orcvs),
                (3, 1),
                "a click at {target:?} in a {screen_size:?} console"
            );
        }
    }

    ///
    /// One pass of the whole running Console, the way eframe drives it.
    ///
    /// `eframe::Frame::_new_kittest` and `CreationContext::_new_kittest` are
    /// eframe's own headless constructors, which is how an `App` runs outside a
    /// window; `storage_tests` reaches for the same pair for the same reason.
    ///
    fn app_pass(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        console: &mut Console,
        host: &mut eframe::Frame,
    ) {
        use eframe::App as _;

        let input = egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };
        let output = ctx.run_ui(input, |root| console.ui(root, host));
        output.drop_without_applying_deltas();
    }

    ///
    /// One pass, answering how soon the console asked to be painted again.
    ///
    /// The Panel paints a published Tick. A delay started from this frame is
    /// a second clock; an immediate delay after a publish is the wake.
    ///
    fn app_pass_repaint_delay(
        ctx: &egui::Context,
        screen: Rect,
        events: Vec<Event>,
        console: &mut Console,
        host: &mut eframe::Frame,
    ) -> std::time::Duration {
        use eframe::App as _;

        let input = egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };
        let output = ctx.run_ui(input, |root| console.ui(root, host));
        let delay = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|viewport| viewport.repaint_delay)
            .unwrap_or(std::time::Duration::MAX);
        output.drop_without_applying_deltas();
        delay
    }

    fn collect_shape_text(shape: &Shape, out: &mut String) {
        match shape {
            Shape::Text(text) => {
                out.push_str(text.galley.text());
                out.push(' ');
            }
            Shape::Vec(shapes) => {
                for nested in shapes {
                    collect_shape_text(nested, out);
                }
            }
            _ => {}
        }
    }

    fn collect_shape_strokes(shape: &Shape, out: &mut Vec<(Color32, Rect)>) {
        match shape {
            Shape::Rect(rect) if rect.stroke.width > 0.0 => {
                out.push((rect.stroke.color, rect.rect));
            }
            Shape::LineSegment { stroke, points } => {
                out.push((stroke.color, Rect::from_two_pos(points[0], points[1])));
            }
            Shape::Vec(shapes) => {
                for nested in shapes {
                    collect_shape_strokes(nested, out);
                }
            }
            _ => {}
        }
    }

    fn painted_strokes(ctx: &egui::Context) -> Vec<(Color32, Rect)> {
        let mut strokes = Vec::new();
        let layers = [
            egui::LayerId::background(),
            egui::LayerId::new(egui::Order::Background, egui::Id::NULL),
            egui::LayerId::new(egui::Order::Middle, egui::Id::NULL),
            egui::LayerId::new(egui::Order::Background, egui::Id::new("bottom_panel")),
            egui::LayerId::new(egui::Order::Middle, egui::Id::new("bottom_panel")),
            egui::LayerId::new(egui::Order::Background, egui::Id::new("top_panel")),
            egui::LayerId::new(egui::Order::Middle, egui::Id::new("top_panel")),
        ];
        ctx.graphics(|graphics| {
            for layer in layers {
                if let Some(list) = graphics.get(layer) {
                    for clipped in list.all_entries() {
                        collect_shape_strokes(&clipped.shape, &mut strokes);
                    }
                }
            }
        });
        strokes
    }

    fn collect_shape_text_spans(shape: &Shape, out: &mut Vec<(String, f32, f32)>) {
        match shape {
            Shape::Text(text) => {
                let left = text.pos.x;
                out.push((
                    text.galley.text().to_owned(),
                    left,
                    left + text.galley.size().x,
                ));
            }
            Shape::Vec(shapes) => {
                for nested in shapes {
                    collect_shape_text_spans(nested, out);
                }
            }
            _ => {}
        }
    }

    fn painted_text(ctx: &egui::Context) -> String {
        let mut text = String::new();
        let layers = [
            egui::LayerId::background(),
            egui::LayerId::new(egui::Order::Background, egui::Id::new("bottom_panel")),
            egui::LayerId::new(egui::Order::Background, egui::Id::new("top_panel")),
            egui::LayerId::new(egui::Order::Middle, egui::Id::new("bottom_panel")),
        ];
        ctx.graphics(|graphics| {
            for layer in layers {
                if let Some(list) = graphics.get(layer) {
                    for clipped in list.all_entries() {
                        collect_shape_text(&clipped.shape, &mut text);
                    }
                }
            }
        });
        text
    }

    ///
    /// Where a running Console presented its Source Grid, asked of the
    /// transform the pass left behind.
    ///
    /// The same three calls `show_source_scene` makes, and for the reason the
    /// `presented` helper cannot serve here: that one fits the Grid to a console
    /// area, and a running Console's console area is the screen less whatever
    /// height the menu bar settled the top panel at. The stored transform
    /// already carries that fit, so this asks it rather than re-deriving it.
    ///
    fn console_viewport(ctx: &egui::Context, console: &Console) -> GridViewport {
        let grid = console.orcvs.render_frame().grid();

        presented_grid(
            console.source_view.to_global,
            source_bounds(grid),
            grid,
            ctx.pixels_per_point(),
        )
    }

    ///
    /// Clicking a Cell of a running Console moves the Cursor onto it.
    ///
    /// Every other click test goes through `console_pass_at`, which shows the
    /// Source itself and applies the answered Position itself — so none of them
    /// reaches the `orcvs.select` call `Console::ui` owns, and all of them pass
    /// with that call deleted. `show_source` answers the Cell rather than
    /// selecting it, which is what makes clicking a Cell a wiring question at
    /// all, and this is the one test that asks it: a real `Console`, driven
    /// through `eframe::App::ui` over an egui pass, with nothing between the
    /// pointer and the Cursor but the console's own code.
    ///
    /// `storage_tests` exists for the same reason one module below, and says
    /// so: an interface-level test cannot see whether the Console is wired to
    /// the interface at all.
    ///
    #[tokio::test]
    async fn a_click_on_a_cell_moves_the_cursor_of_a_running_console() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        // One quiet pass, so the top panel has claimed its height and the view
        // holds the fit the Grid was presented under.
        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        assert_eq!(
            selected_cell(&console.orcvs),
            (0, 0),
            "a fresh Console did not open with the Cursor in the corner"
        );

        let viewport = console_viewport(&ctx, &console);
        let target = viewport.rect.min + Vec2::new(3.5, 1.5) * viewport.cell_size;
        // Pressed on one pass and released on the next, because a click is
        // reported on the release.
        app_pass(&ctx, screen, click_at(target), &mut console, &mut host);
        app_pass(&ctx, screen, release_at(target), &mut console, &mut host);

        assert_eq!(
            selected_cell(&console.orcvs),
            (3, 1),
            "a click at {target:?} on a Grid presented at {:?}",
            viewport.rect
        );
    }

    ///
    /// Before the first Playback run the Panel shows B `120 //`, T `00000`,
    /// C `00:00`, O `None`. File, View, and Theme remain on the top bar; the
    /// MIDI menu is gone.
    ///
    #[tokio::test]
    async fn the_bottom_panel_shows_tick_zero_and_run_clock_before_the_first_run() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        let painted = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let sink = painted.clone();
        ctx.on_end_pass(
            "capture-panel-text",
            std::sync::Arc::new(move |ctx| {
                *sink.lock().unwrap() = painted_text(ctx);
            }),
        );

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);

        let text = painted.lock().unwrap().clone();
        let bpm_at = text
            .find('B')
            .expect("the Panel is missing the B label in {text:?}");
        let beat_at = text
            .find("//")
            .expect("the Panel is missing the rest marker in {text:?}");
        let tick_at = beat_at
            + text[beat_at..]
                .find('T')
                .expect("the Panel is missing the T label in {text:?}");
        let clock_at = beat_at
            + text[beat_at..]
                .find('C')
                .expect("the Panel is missing the C label in {text:?}");
        assert!(
            !text.contains("**"),
            "the rest Panel still shows the beat marker in {text:?}"
        );
        assert!(
            bpm_at < beat_at && beat_at < tick_at && tick_at < clock_at,
            "Readout order is not B then // then T then C in {text:?}"
        );
        assert!(
            text.contains("120"),
            "the Panel is missing BPM 120 in {text:?}"
        );
        assert!(
            text.contains("00000"),
            "the Panel is missing Tick 00000 in {text:?}"
        );
        assert!(
            text.contains("00:00"),
            "the Panel is missing Run Clock 00:00 in {text:?}"
        );
        for menu in ["File", "View", "Theme"] {
            assert!(
                text.contains(menu),
                "the top bar is missing {menu} in {text:?}"
            );
        }
        assert!(
            !text[..bpm_at].contains("MIDI"),
            "the MIDI menu is still on the top bar in {text:?}"
        );
        assert!(
            !text.contains("No MIDI destinations found"),
            "the MIDI menu empty copy is still on the console in {text:?}"
        );
        assert!(
            !text.contains("Tempo"),
            "the Tempo menu is still on the top bar in {text:?}"
        );

        let output_at = clock_at
            + text[clock_at..]
                .find('O')
                .expect("the Panel is missing the O label in {text:?}");
        let none_at = text
            .find(crate::midi::OUTPUT_NONE)
            .unwrap_or_else(|| panic!("the Panel is missing Output None in {text:?}"));
        assert!(
            clock_at < output_at && output_at < none_at,
            "Readout order is not C then O then None in {text:?}"
        );
        assert!(
            !text.contains(super::OUTPUT_SCAN),
            "Scan belongs in the Output menu, not on the closed Panel in {text:?}"
        );

        let panel = egui::containers::panel::PanelState::load(&ctx, egui::Id::new("bottom_panel"))
            .expect("the bottom Panel was not shown");
        assert!(
            (panel.outer_rect.bottom() - screen.bottom()).abs() < 0.5,
            "the Panel is not at the bottom: {:?}",
            panel.outer_rect
        );
        assert!(
            (panel.outer_rect.height() - BOTTOM_PANEL_HEIGHT).abs() < 0.5,
            "the Panel is not {BOTTOM_PANEL_HEIGHT} tall: {:?}",
            panel.outer_rect
        );
    }

    ///
    /// A published Tick is what the Panel paints. Waiting a Tick period from
    /// this Render Frame is a second clock, so `**` and T land late or twice.
    ///
    #[tokio::test]
    async fn a_playing_console_repaints_as_soon_as_the_published_tick_advances() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        let bpm = orcvs::opts::Bpm::new(200).expect("200 is in range");
        console.orcvs.set_bpm(bpm);
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Space, true)],
            &mut console,
            &mut host,
        );
        for _ in 0..1_000 {
            if console.orcvs.playback_observation().state == orcvs::playback::PlaybackState::Playing
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        assert_eq!(
            console.orcvs.playback_observation().state,
            orcvs::playback::PlaybackState::Playing
        );

        let _ = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
        let tick = console.orcvs.playback_observation().tick;
        for _ in 0..2_000 {
            if console.orcvs.playback_observation().tick != tick {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        let advanced = console.orcvs.playback_observation().tick;
        assert_ne!(
            advanced, tick,
            "Playback never published another Tick from {tick:?}"
        );

        let delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
        assert_eq!(
            delay,
            std::time::Duration::ZERO,
            "the console waited {delay:?} after Tick {tick:?} became {advanced:?}"
        );
    }

    ///
    /// A quiet Render Frame must not start a Tick period from now. That is
    /// the second clock. The next paint is the next publish, or the Cursor
    /// Effect, whichever is sooner.
    ///
    #[tokio::test]
    async fn a_playing_console_does_not_schedule_a_tick_period_from_this_frame() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        let bpm = orcvs::opts::Bpm::new(200).expect("200 is in range");
        console.orcvs.set_bpm(bpm);
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Space, true)],
            &mut console,
            &mut host,
        );
        for _ in 0..1_000 {
            if console.orcvs.playback_observation().state == orcvs::playback::PlaybackState::Playing
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        assert_eq!(
            console.orcvs.playback_observation().state,
            orcvs::playback::PlaybackState::Playing
        );

        let _ = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
        let delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
        let tick = std::time::Duration::from_millis(bpm.delay_ms());
        assert_ne!(
            delay, tick,
            "the console still scheduled a Tick period from this frame: {delay:?}"
        );
    }

    ///
    /// At 1 BPM a Tick lasts 15 seconds. Cursor Effect off, so its 45–190 ms
    /// wakes cannot hide a missing Run Clock remainder. A quiet Playing pass
    /// must still request a delay of at most one second.
    ///
    #[tokio::test]
    async fn a_playing_console_with_cursor_effect_off_wakes_within_a_second() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        console.reduced_motion = true;
        let bpm = orcvs::opts::Bpm::new(1).expect("1 is in range");
        console.orcvs.set_bpm(bpm);
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Space, true)],
            &mut console,
            &mut host,
        );
        for _ in 0..1_000 {
            if console.orcvs.playback_observation().state == orcvs::playback::PlaybackState::Playing
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        assert_eq!(
            console.orcvs.playback_observation().state,
            orcvs::playback::PlaybackState::Playing
        );

        let _ = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
        let delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
        assert!(
            delay <= std::time::Duration::from_secs(1),
            "the console waited {delay:?} on a quiet Playing pass with Cursor Effect off"
        );
    }

    ///
    /// egui subtracts `predicted_dt` from every timed request. With a
    /// predicted frame longer than any Run Clock remainder, an uncompensated
    /// remainder saturates to an immediate Render Frame on every pass — the
    /// back-to-back redraw `until_next` adds `predicted_dt` to prevent. The
    /// compensated request survives the subtraction as the remainder itself.
    ///
    #[tokio::test]
    async fn a_playing_console_compensates_the_run_clock_wake_for_predicted_frame_time() {
        use eframe::App as _;

        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        console.reduced_motion = true;
        console
            .orcvs
            .set_bpm(orcvs::opts::Bpm::new(1).expect("1 is in range"));
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Space, true)],
            &mut console,
            &mut host,
        );
        for _ in 0..1_000 {
            if console.orcvs.playback_observation().state == orcvs::playback::PlaybackState::Playing
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        assert_eq!(
            console.orcvs.playback_observation().state,
            orcvs::playback::PlaybackState::Playing
        );

        // A publish or focus wake can still ask for an immediate pass while
        // settling; a compensated Run Clock request is the only timed one.
        let mut delays = Vec::new();
        for _ in 0..8 {
            let input = egui::RawInput {
                screen_rect: Some(screen),
                predicted_dt: 2.0,
                ..Default::default()
            };
            let output = ctx.run_ui(input, |root| console.ui(root, &mut host));
            let delay = output
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .map(|viewport| viewport.repaint_delay)
                .unwrap_or(std::time::Duration::MAX);
            output.drop_without_applying_deltas();
            delays.push(delay);
            if delay > std::time::Duration::ZERO {
                break;
            }
        }
        let settled = delays.last().copied().expect("at least one pass");
        assert!(
            settled > std::time::Duration::ZERO && settled <= std::time::Duration::from_secs(1),
            "a quiet Playing pass never requested the Run Clock remainder: {delays:?}"
        );
    }

    ///
    /// Rest must not grow a one-second wake. Cursor Effect is off so its
    /// cadence cannot be mistaken for Run Clock honesty. The first passes
    /// request an immediate Render Frame (focus, destination auto-select);
    /// settle before asserting.
    ///
    #[tokio::test]
    async fn a_stopped_console_with_cursor_effect_off_requests_no_timed_wake() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        console.reduced_motion = true;
        let mut delay = std::time::Duration::ZERO;
        for _ in 0..8 {
            delay = app_pass_repaint_delay(&ctx, screen, Vec::new(), &mut console, &mut host);
            if delay == std::time::Duration::MAX {
                break;
            }
        }
        assert_eq!(
            delay,
            std::time::Duration::MAX,
            "a Stopped console still requested a timed wake after settling: {delay:?}"
        );
    }

    fn origin_content(orcvs: &Orcvs) -> Option<char> {
        let frame = orcvs.render_frame();
        frame.at(frame.grid().origin()).content()
    }

    fn bpm_field_id(console: &Console) -> egui::Id {
        console.bpm_widget_id
    }

    fn focus_bpm_field(
        ctx: &egui::Context,
        screen: Rect,
        console: &mut Console,
        host: &mut eframe::Frame,
    ) {
        let target = ctx
            .read_response(bpm_field_id(console))
            .expect("the BPM field was not shown")
            .rect
            .center();
        app_pass(ctx, screen, click_at(target), console, host);
        app_pass(ctx, screen, release_at(target), console, host);
        assert!(
            ctx.memory(|memory| memory.has_focus(bpm_field_id(console))),
            "the BPM field did not take focus"
        );
    }

    fn bpm_selected_chars(ctx: &egui::Context, console: &Console) -> usize {
        egui::TextEdit::load_state(ctx, bpm_field_id(console))
            .and_then(|state| state.cursor.char_range())
            .map(|range| {
                let chars = range.as_sorted_char_range();
                chars.end.0.saturating_sub(chars.start.0)
            })
            .unwrap_or(0)
    }

    #[tokio::test]
    async fn clicking_the_bpm_field_selects_its_text() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        let shown = console.orcvs.bpm().beats_per_minute().to_string();
        focus_bpm_field(&ctx, screen, &mut console, &mut host);
        assert_eq!(
            bpm_selected_chars(&ctx, &console),
            shown.chars().count(),
            "a click left the BPM text unselected"
        );

        app_pass(
            &ctx,
            screen,
            vec![Event::Text("7".to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Enter, true)],
            &mut console,
            &mut host,
        );
        assert_eq!(
            console.orcvs.bpm().beats_per_minute(),
            7,
            "a click did not select the BPM text for replacement"
        );
    }

    ///
    /// With the BPM field focused, Tab is egui's own focus navigation and not
    /// the Source's: `Console::ui` only cancels it while the Source holds the
    /// keys, so a focused control keeps Tab exactly as it always has, and the
    /// Cursor does not move.
    ///
    #[tokio::test]
    async fn tab_with_the_bpm_field_focused_leaves_the_cursor_and_moves_focus_on() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        focus_bpm_field(&ctx, screen, &mut console, &mut host);

        let cursor_before = console.orcvs.render_frame().cursor();
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Tab, true)],
            &mut console,
            &mut host,
        );

        assert_eq!(
            console.orcvs.render_frame().cursor(),
            cursor_before,
            "Tab moved the Cursor while the BPM field held focus"
        );
        assert!(
            !ctx.memory(|memory| memory.has_focus(bpm_field_id(&console))),
            "Tab left focus on the BPM field rather than moving egui's focus on from it"
        );
    }

    #[tokio::test]
    async fn the_bpm_field_accepts_digits_only() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        focus_bpm_field(&ctx, screen, &mut console, &mut host);
        app_pass(
            &ctx,
            screen,
            vec![Event::Text("8".to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![Event::Text("a".to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![Event::Text("4".to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![Event::Text("0".to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Enter, true)],
            &mut console,
            &mut host,
        );
        assert_eq!(
            console.orcvs.bpm().beats_per_minute(),
            840,
            "the BPM field took a letter"
        );
    }

    #[tokio::test]
    async fn escape_reverts_a_valid_uncommitted_bpm() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();
        let start = console.orcvs.bpm().beats_per_minute();
        let grid = console.orcvs.grid();
        console
            .orcvs
            .extend(grid.position(3, 2).expect("inside the Grid"));
        let region = console.orcvs.region();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        focus_bpm_field(&ctx, screen, &mut console, &mut host);
        app_pass(
            &ctx,
            screen,
            vec![Event::Text("60".to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Escape, true)],
            &mut console,
            &mut host,
        );
        assert_eq!(
            console.orcvs.bpm().beats_per_minute(),
            start,
            "Escape committed the typed BPM"
        );
        assert_eq!(
            console.orcvs.region(),
            region,
            "Escape in the BPM field collapsed the Region"
        );
    }

    ///
    /// A fill armed by command Enter waits for the next event, and typing into
    /// the BPM field is that event even though the field, not the Source,
    /// received it. A character typed back in the Source afterwards writes one
    /// Cell rather than filling the Region the chord was pressed over.
    ///
    #[tokio::test]
    async fn keys_the_bpm_field_took_disarm_a_fill_armed_before_it() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();
        let grid = console.orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the Grid");
        console.orcvs.extend(at(2, 1));

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        app_pass(
            &ctx,
            screen,
            vec![command_key_event(Key::Enter)],
            &mut console,
            &mut host,
        );
        focus_bpm_field(&ctx, screen, &mut console, &mut host);
        app_pass(
            &ctx,
            screen,
            vec![Event::Text("120".to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Enter, true)],
            &mut console,
            &mut host,
        );
        assert!(
            !ctx.memory(|memory| memory.has_focus(bpm_field_id(&console))),
            "Enter left the BPM field focused"
        );

        app_pass(
            &ctx,
            screen,
            vec![Event::Text("x".to_owned())],
            &mut console,
            &mut host,
        );
        let frame = console.orcvs.render_frame();
        assert_eq!(
            frame.at(at(0, 0)).content(),
            None,
            "a fill armed before the BPM field took the keys filled the Region"
        );
    }

    ///
    /// A copy with the Source focused hands the Region's rows to the platform
    /// clipboard, and a paste writes the text the platform delivered.
    ///
    #[tokio::test]
    async fn copy_reaches_the_platform_clipboard_and_paste_writes_the_source() {
        use eframe::App as _;

        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();
        let grid = console.orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the Grid");

        app_pass(
            &ctx,
            screen,
            vec![Event::Paste("ab\ncd".to_owned())],
            &mut console,
            &mut host,
        );
        assert_eq!(origin_content(&console.orcvs), Some('a'));
        let landed = console.orcvs.region();
        assert_eq!((landed.columns(), landed.rows()), (0..2, 0..2));

        console.orcvs.select(at(1, 0));
        console.orcvs.extend(at(1, 1));
        let input = egui::RawInput {
            screen_rect: Some(screen),
            events: vec![Event::Copy],
            ..Default::default()
        };
        let output = ctx.run_ui(input, |root| console.ui(root, &mut host));
        assert!(
            output
                .platform_output
                .commands
                .contains(&egui::OutputCommand::CopyText("b\nd".to_owned())),
            "the copy never reached the platform: {:?}",
            output.platform_output.commands
        );
    }

    #[tokio::test]
    async fn dragging_the_bpm_field_does_not_change_the_tempo() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        let start = console.orcvs.bpm().beats_per_minute();
        let origin = ctx
            .read_response(bpm_field_id(&console))
            .expect("the BPM field was not shown")
            .rect
            .center();
        app_pass(&ctx, screen, click_at(origin), &mut console, &mut host);
        app_pass(
            &ctx,
            screen,
            vec![Event::PointerMoved(origin + Vec2::new(40.0, 0.0))],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            release_at(origin + Vec2::new(40.0, 0.0)),
            &mut console,
            &mut host,
        );
        assert_eq!(
            console.orcvs.bpm().beats_per_minute(),
            start,
            "dragging the BPM field changed the tempo"
        );
    }
    #[tokio::test]
    async fn the_bpm_field_pads_three_digits() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        let field = ctx
            .read_response(bpm_field_id(&console))
            .expect("the BPM field was not shown")
            .rect;
        let font = ctx
            .style_of(egui::Theme::Dark)
            .text_styles
            .get(&egui::TextStyle::Monospace)
            .cloned()
            .unwrap_or_else(|| egui::FontId::monospace(12.0));
        let digits = ctx.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap("000".to_owned(), font, Color32::WHITE)
                .size()
        });
        assert!(
            field.width() + 0.5 >= digits.x + BPM_FIELD_MARGIN.sum().x,
            "the BPM field is too narrow for its padding: {field:?}"
        );
        assert!(
            field.height() + 0.5 >= digits.y + BPM_FIELD_MARGIN.sum().y,
            "the BPM field is too short for its padding: {field:?}"
        );
    }

    ///
    /// While the BPM field is focused, digits stay in the field and Space does
    /// not toggle Playback. Escape (and a later Grid click) return keys to
    /// the console.
    ///
    #[tokio::test]
    async fn a_focused_bpm_field_owns_digits_and_space_until_escape_or_a_grid_click() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        focus_bpm_field(&ctx, screen, &mut console, &mut host);

        app_pass(
            &ctx,
            screen,
            vec![Event::Text("5".to_owned())],
            &mut console,
            &mut host,
        );
        assert_eq!(
            origin_content(&console.orcvs),
            None,
            "a digit typed into the focused BPM field wrote the Source Cell"
        );

        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Space, true)],
            &mut console,
            &mut host,
        );
        tokio::task::yield_now().await;
        assert_eq!(
            console.orcvs.playback_observation().state,
            orcvs::playback::PlaybackState::Stopped,
            "Space toggled Playback while the BPM field had focus"
        );

        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Escape, true)],
            &mut console,
            &mut host,
        );
        assert!(
            !ctx.memory(|memory| memory.has_focus(bpm_field_id(&console))),
            "Escape left the BPM field focused"
        );

        app_pass(
            &ctx,
            screen,
            vec![Event::Text("x".to_owned())],
            &mut console,
            &mut host,
        );
        assert_eq!(
            origin_content(&console.orcvs),
            Some('x'),
            "a digit after Escape did not write the Source Cell"
        );

        focus_bpm_field(&ctx, screen, &mut console, &mut host);
        let viewport = console_viewport(&ctx, &console);
        let grid_cell = viewport.rect.min + Vec2::new(1.5, 0.5) * viewport.cell_size;
        app_pass(&ctx, screen, click_at(grid_cell), &mut console, &mut host);
        app_pass(&ctx, screen, release_at(grid_cell), &mut console, &mut host);
        assert!(
            !ctx.memory(|memory| memory.has_focus(bpm_field_id(&console))),
            "a click on the Source Grid left the BPM field focused"
        );

        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Space, true)],
            &mut console,
            &mut host,
        );
        tokio::task::yield_now().await;
        assert_eq!(
            console.orcvs.playback_observation().state,
            orcvs::playback::PlaybackState::Playing,
            "Space did not toggle Playback after a Grid click returned keys"
        );
    }

    ///
    /// Out-of-range typed BPM is rejected rather than clamped into range.
    ///
    #[tokio::test]
    async fn out_of_range_bpm_input_does_not_change_the_tempo() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();
        let start = console.orcvs.bpm().beats_per_minute();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        focus_bpm_field(&ctx, screen, &mut console, &mut host);
        for text in ["0", "1500"] {
            app_pass(
                &ctx,
                screen,
                vec![Event::Text(text.to_owned())],
                &mut console,
                &mut host,
            );
            app_pass(
                &ctx,
                screen,
                vec![key_event(Key::Enter, true)],
                &mut console,
                &mut host,
            );
            assert_eq!(
                console.orcvs.bpm().beats_per_minute(),
                start,
                "committing {text} changed the tempo"
            );
            focus_bpm_field(&ctx, screen, &mut console, &mut host);
        }
    }

    ///
    /// Committing a new BPM while Playback is requested goes through
    /// `Orcvs::set_bpm`. Retune-while-playing is already the engine's test;
    /// this only asks that the Console's commit reaches it.
    ///
    #[tokio::test]
    async fn committing_bpm_while_playback_is_requested_sets_it_on_orcvs() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Space, true)],
            &mut console,
            &mut host,
        );
        tokio::task::yield_now().await;

        focus_bpm_field(&ctx, screen, &mut console, &mut host);
        app_pass(
            &ctx,
            screen,
            vec![Event::Key {
                key: Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            }],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![Event::Text("60".to_owned())],
            &mut console,
            &mut host,
        );
        app_pass(
            &ctx,
            screen,
            vec![key_event(Key::Enter, true)],
            &mut console,
            &mut host,
        );

        assert_eq!(console.orcvs.bpm().beats_per_minute(), 60);
    }

    #[tokio::test]
    async fn a_source_smaller_than_the_console_sits_at_the_top_left_and_holds_no_cell_in_the_surplus()
     {
        for screen_size in [
            Vec2::new(400.0, 200.0),
            Vec2::new(200.0, 400.0),
            Vec2::new(300.0, 300.0),
        ] {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, screen_size);
            let mut orcvs = running_orcvs(8, 8);
            let mut view = SourceView::default();

            let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            let half_cell = Vec2::splat(viewport.cell_size / 2.0);

            assert_eq!(
                viewport.cell_size, CELL_SIZE,
                "a {screen_size:?} console opened at a Cell size other than the Source's own"
            );
            assert_eq!(
                viewport.rect.min,
                screen.min + Vec2::splat(MARGIN),
                "a {screen_size:?} console did not sit the Source one margin in from its top-left"
            );

            click(
                &ctx,
                screen,
                viewport.rect.max - half_cell,
                &mut orcvs,
                &mut view,
            );
            assert_eq!(
                selected_cell(&orcvs),
                (7, 7),
                "the last Cell of a {screen_size:?} console"
            );

            if let Some(past_the_grid) = surplus(screen, viewport.rect) {
                click(&ctx, screen, past_the_grid, &mut orcvs, &mut view);
                assert_eq!(
                    selected_cell(&orcvs),
                    (7, 7),
                    "a click past the Grid of a {screen_size:?} console"
                );
            }

            click(
                &ctx,
                screen,
                viewport.rect.min + half_cell,
                &mut orcvs,
                &mut view,
            );
            assert_eq!(
                selected_cell(&orcvs),
                (0, 0),
                "the first Cell of a {screen_size:?} console"
            );
        }
    }

    ///
    /// The console opens on the default Grid at the Source's own Cell size,
    /// one margin in from its top-left, with the Grid running past the far
    /// edges so there is room to Pan, and no Glyph is resampled to be shown.
    ///
    #[tokio::test]
    async fn the_default_window_presents_the_default_grid_at_its_own_scale() {
        let ctx = egui::Context::default();
        let console = Vec2::new(
            DEFAULT_VIEW_SIZE[0],
            DEFAULT_VIEW_SIZE[1] - TOP_PANEL_HEIGHT - BOTTOM_PANEL_HEIGHT,
        );
        let screen = Rect::from_min_size(Pos2::ZERO, console);
        let mut orcvs = running_orcvs(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        assert_eq!(
            viewport.cell_size, CELL_SIZE,
            "the default console opened at a scale other than one"
        );
        assert_eq!(
            view.zoom, 1.0,
            "the default console opened at a Zoom other than 1.0"
        );
        assert_eq!(
            viewport.rect.min,
            screen.min + Vec2::splat(MARGIN),
            "the default console did not rest the Grid one margin in"
        );
        assert!(
            viewport.rect.max.x > screen.max.x && viewport.rect.max.y > screen.max.y,
            "the default Grid does not run past the default console, so there is nowhere to Pan"
        );
    }

    ///
    /// The default window size holds back exactly the height the top bar and
    /// the bottom Panel take, so the rest reaches the console. The menu bar is
    /// rebuilt here rather than shared, so this also asserts that no menu
    /// makes the top bar taller than its minimum, and that the Panel's
    /// Readouts do not make it taller than its minimum.
    ///
    #[test]
    fn the_top_panel_takes_the_height_the_default_window_holds_back() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Vec2::ZERO;

        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |root| {
                egui::Panel::top("top_panel")
                    .resizable(true)
                    .min_size(TOP_PANEL_HEIGHT)
                    .show(root, |ui| {
                        egui::MenuBar::new().ui(ui, |ui| {
                            ui.menu_button("File", |_ui| {});
                            ui.add_space(16.0);
                            ui.menu_button("View", |_ui| {});
                        });
                    });
                egui::Panel::bottom("bottom_panel")
                    .resizable(false)
                    .min_size(BOTTOM_PANEL_HEIGHT)
                    .show(root, |ui| {
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                let (label_value_gap, entry_gap) = super::panel_readout_gaps(ui);
                                ui.spacing_mut().item_spacing.x = 0.0;
                                super::panel_label(ui, "B");
                                ui.add_space(label_value_gap);
                                let mut bpm = 120usize;
                                super::add_bpm_field(ui, &mut bpm);
                                ui.add_space(label_value_gap);
                                super::reserved_monospace(
                                    ui,
                                    super::BEAT_MARKER,
                                    super::monospace_width(ui, super::BEAT_MARKER),
                                );
                                ui.add_space(entry_gap);
                                super::panel_label(ui, "T");
                                ui.add_space(label_value_gap);
                                super::reserved_monospace(
                                    ui,
                                    "00000",
                                    super::monospace_width(ui, "00000"),
                                );
                                ui.add_space(entry_gap);
                                super::panel_label(ui, "C");
                                ui.add_space(label_value_gap);
                                super::reserved_monospace(
                                    ui,
                                    "00:00",
                                    super::monospace_width(ui, "00:00"),
                                );
                                ui.add_space(entry_gap);
                                super::panel_label(ui, "O");
                                ui.add_space(label_value_gap);
                                super::apply_panel_field_spacing(ui);
                                egui::ComboBox::from_id_salt(super::DESTINATION_COMBO_ID)
                                    .selected_text(
                                        egui::RichText::new(crate::midi::OUTPUT_NONE)
                                            .text_style(egui::TextStyle::Monospace),
                                    )
                                    .width(super::OUTPUT_READOUT_WIDTH)
                                    .icon(|_ui, _rect, _visuals, _is_open| {})
                                    .show_ui(ui, |ui| {
                                        let _ = ui.button(super::OUTPUT_SCAN);
                                        ui.separator();
                                        let _ = ui.selectable_label(
                                            true,
                                            egui::RichText::new(crate::midi::OUTPUT_NONE)
                                                .text_style(egui::TextStyle::Monospace),
                                        );
                                    });
                            },
                        );
                    });
                egui::CentralPanel::default()
                    .frame(source_panel_frame(
                        crate::source_paint::SourcePaintSettings::default().source_background(),
                    ))
                    .show(root, |ui| {
                        console = ui.available_size_before_wrap();
                    });
            },
        );
        output.drop_without_applying_deltas();

        assert_eq!(
            console,
            Vec2::new(
                DEFAULT_VIEW_SIZE[0],
                DEFAULT_VIEW_SIZE[1] - TOP_PANEL_HEIGHT - BOTTOM_PANEL_HEIGHT
            )
        );
    }

    ///
    /// Tick and Run Clock occupy fixed slots, so a second Tick digit does not
    /// shove Run Clock.
    ///
    #[test]
    fn a_second_tick_digit_does_not_move_run_clock() {
        fn clock_left(tick: &str) -> i32 {
            let ctx = egui::Context::default();
            ctx.set_style_of(egui::Theme::Dark, crate::style::style());
            ctx.set_theme(egui::Theme::Dark);
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
            let clock_x = std::cell::Cell::new(0.0);
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |root| {
                    egui::Panel::bottom("bottom_panel")
                        .resizable(false)
                        .min_size(BOTTOM_PANEL_HEIGHT)
                        .show(root, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("T");
                                super::reserved_monospace(
                                    ui,
                                    tick,
                                    super::monospace_width(ui, "00000"),
                                );
                                ui.label("C");
                                clock_x.set(
                                    super::reserved_monospace(
                                        ui,
                                        "00:00",
                                        super::monospace_width(ui, "00:00"),
                                    )
                                    .rect
                                    .left(),
                                );
                            });
                        });
                },
            );
            output.drop_without_applying_deltas();
            clock_x.get().round() as i32
        }

        assert_eq!(
            clock_left("00009"),
            clock_left("00010"),
            "a second Tick digit shoved Run Clock"
        );
    }

    #[test]
    fn an_off_beat_does_not_move_tick() {
        fn tick_left(marker: &str) -> i32 {
            let ctx = egui::Context::default();
            ctx.set_style_of(egui::Theme::Dark, crate::style::style());
            ctx.set_theme(egui::Theme::Dark);
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
            let tick_x = std::cell::Cell::new(0.0);
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    ..Default::default()
                },
                |root| {
                    egui::Panel::bottom("bottom_panel")
                        .resizable(false)
                        .min_size(BOTTOM_PANEL_HEIGHT)
                        .show(root, |ui| {
                            ui.horizontal(|ui| {
                                super::reserved_monospace(
                                    ui,
                                    marker,
                                    super::monospace_width(ui, super::BEAT_MARKER),
                                );
                                ui.label("T");
                                tick_x.set(
                                    super::reserved_monospace(
                                        ui,
                                        "00000",
                                        super::monospace_width(ui, "00000"),
                                    )
                                    .rect
                                    .left(),
                                );
                            });
                        });
                },
            );
            output.drop_without_applying_deltas();
            tick_x.get().round() as i32
        }

        assert_eq!(
            tick_left("**"),
            tick_left(""),
            "hiding the beat marker shoved Tick"
        );
        assert_eq!(
            tick_left("**"),
            tick_left("//"),
            "the rest marker shoved Tick"
        );
    }

    #[test]
    fn panel_readouts_use_the_monospace_style_size_not_line_height() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let sizes = std::cell::Cell::new((0.0, 0.0));
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |root| {
                egui::Panel::bottom("bottom_panel").show(root, |ui| {
                    let style_size = ui
                        .style()
                        .text_styles
                        .get(&egui::TextStyle::Monospace)
                        .map(|font| font.size)
                        .unwrap_or(0.0);
                    sizes.set((style_size, super::panel_monospace_id(ui).size));
                });
            },
        );
        output.drop_without_applying_deltas();
        let (style_size, used) = sizes.get();
        assert_eq!(
            used, style_size,
            "Panel numbers were drawn at line height instead of the Monospace size"
        );
    }

    #[tokio::test]
    async fn panel_label_gaps_match_and_entry_gaps_match() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();
        let painted = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = painted.clone();
        ctx.on_end_pass(
            "capture-panel-spans",
            std::sync::Arc::new(move |ctx| {
                let mut spans = Vec::new();
                let layers = [
                    egui::LayerId::background(),
                    egui::LayerId::new(egui::Order::Background, egui::Id::new("bottom_panel")),
                    egui::LayerId::new(egui::Order::Background, egui::Id::new("top_panel")),
                    egui::LayerId::new(egui::Order::Middle, egui::Id::new("bottom_panel")),
                ];
                ctx.graphics(|graphics| {
                    for layer in layers {
                        if let Some(list) = graphics.get(layer) {
                            for clipped in list.all_entries() {
                                collect_shape_text_spans(&clipped.shape, &mut spans);
                            }
                        }
                    }
                });
                *sink.lock().unwrap() = spans;
            }),
        );
        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);

        let mut spans = painted.lock().unwrap().clone();
        spans.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        fn span<'a>(spans: &'a [(String, f32, f32)], text: &str) -> &'a (String, f32, f32) {
            spans
                .iter()
                .find(|(shown, _, _)| shown == text)
                .unwrap_or_else(|| panic!("the Panel is missing {text:?} in {spans:?}"))
        }

        let b = span(&spans, "B");
        let rest = span(&spans, "//");
        let t = span(&spans, "T");
        let tick = span(&spans, "00000");
        let c = span(&spans, "C");
        let clock = span(&spans, "00:00");

        let t_to_v = tick.1 - t.2;
        let c_to_v = clock.1 - c.2;
        let beat_to_t = t.1 - rest.2;
        let tick_to_c = c.1 - tick.2;

        assert!(
            (t_to_v - c_to_v).abs() < 0.5,
            "readout pairs differ: T {t_to_v} C {c_to_v}"
        );
        assert!(
            (beat_to_t - tick_to_c).abs() < 0.5,
            "entry gaps differ: //-to-T {beat_to_t} Tick-to-C {tick_to_c}"
        );
        assert!(
            beat_to_t > t_to_v + 0.5,
            "groups are as tight as a label and its value: group {beat_to_t} pair {t_to_v}"
        );

        let expected_left = egui::Frame::side_top_panel(&crate::style::style())
            .inner_margin
            .leftf()
            + f32::from(BOTTOM_PANEL_LEFT_PAD);
        assert!(
            (b.1 - expected_left).abs() < 1.0,
            "B starts at {} rather than {expected_left} in {spans:?}",
            b.1
        );
    }

    #[tokio::test]
    async fn the_bottom_panel_separator_is_the_grid_line() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();
        let painted = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = painted.clone();
        ctx.on_end_pass(
            "capture-panel-separator",
            std::sync::Arc::new(move |ctx| {
                *sink.lock().unwrap() = painted_strokes(ctx);
            }),
        );
        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);

        let strokes = painted.lock().unwrap().clone();
        assert!(
            strokes.iter().any(|(color, rect)| {
                *color == crate::style::style().visuals.window_stroke.color
                    && rect.height() <= 2.0
                    && rect.width() > screen.width() * 0.5
            }),
            "the Panel separator was not the grid-line stroke in {strokes:?}"
        );
    }

    #[tokio::test]
    async fn the_bpm_field_uses_the_selection_stroke_while_focused() {
        let ctx = egui::Context::default();
        ctx.set_style_of(egui::Theme::Dark, crate::style::style());
        ctx.set_theme(egui::Theme::Dark);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::from(DEFAULT_VIEW_SIZE));
        let mut console = Console::new(&eframe::CreationContext::_new_kittest(ctx.clone()))
            .expect("the test runtime");
        let mut host = eframe::Frame::_new_kittest();
        let painted = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = painted.clone();
        ctx.on_end_pass(
            "capture-bpm-strokes",
            std::sync::Arc::new(move |ctx| {
                *sink.lock().unwrap() = painted_strokes(ctx);
            }),
        );

        app_pass(&ctx, screen, Vec::new(), &mut console, &mut host);
        let field = ctx
            .read_response(bpm_field_id(&console))
            .expect("the BPM field was not shown")
            .rect;
        let rest = painted.lock().unwrap().clone();
        assert!(
            rest.iter().all(|(color, rect)| {
                !field.intersects(*rect)
                    || (*color != PALETTE.selection_stroke
                        && *color != PALETTE.selection_stroke_rest)
            }),
            "an unfocused BPM field still carried a selection border in {rest:?}"
        );

        focus_bpm_field(&ctx, screen, &mut console, &mut host);
        let focused_field = ctx
            .read_response(bpm_field_id(&console))
            .expect("the BPM field was not shown")
            .rect;
        let focused = painted.lock().unwrap().clone();
        assert!(
            focused.iter().any(|(color, rect)| {
                *color == PALETTE.selection_stroke && focused_field.intersects(*rect)
            }),
            "a focused BPM field lost the selection stroke in {focused:?}"
        );
    }

    #[tokio::test]
    async fn source_bounds_are_available_before_the_first_render() {
        let orcvs = running_orcvs(32, 16);
        let source_grid = orcvs.render_frame().grid();
        let bounds = source_bounds(source_grid);

        assert_eq!(
            bounds,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(CELL_SIZE * 32.0, CELL_SIZE * 16.0))
        );
    }

    ///
    /// The viewport `show_source_scene` presents a Grid of this shape at, in a
    /// console of this size, before any gesture has moved the view.
    ///
    /// The same transform that function builds at Zoom 1.0 with the Source at
    /// the console's top-left, so nothing about the geometry is restated here:
    /// the presented Cell side is `presented_grid`'s, asserted in
    /// `grid_viewport.rs`.
    ///
    fn presented(screen: Rect, columns: usize, rows: usize, pixels_per_point: f32) -> GridViewport {
        let grid = Grid::new(columns, rows);
        let source = source_bounds(grid);

        let side = crate::grid_viewport::snapped_cell_side(CELL_SIZE, pixels_per_point);
        presented_grid(
            TSTransform::new(
                screen.min.to_vec2() + Vec2::splat(SOURCE_MARGIN_CELLS * side),
                1.0,
            ),
            source,
            grid,
            pixels_per_point,
        )
    }

    ///
    /// A Paint of everything `viewport` shows of `frame` inside `clip`.
    ///
    /// The range is `GridViewport::visible_positions`' own answer, which is
    /// what `show_source` hands `Paint::derive`. A test that fits the whole
    /// Grid on screen therefore gets a Paint of the whole Grid without having
    /// to say so, and one that zooms gets exactly what the console would draw.
    ///
    fn painted(frame: &RenderFrame, viewport: GridViewport, clip: Rect) -> Paint {
        let grid = frame.grid();

        Paint::derive(FramePaint::new(
            frame,
            viewport.visible_positions(clip, grid),
        ))
    }

    ///
    /// The rectangles and strokes a Paint is drawn as at `viewport` — no
    /// galleys, no font atlas, no `egui::Context`.
    ///
    /// What is asserted through this is what the geometry step itself adds:
    /// borders, the Cursor's stroke, background runs and sector seams. Colour
    /// *decisions* stay in `paint.rs`; Glyph placement needs
    /// [`source_shapes`].
    ///
    fn source_geometry(
        paint: &Paint,
        viewport: GridViewport,
        pixels_per_point: f32,
    ) -> SourceShapes {
        SourceShapes::geometry(paint, &viewport, pixels_per_point)
    }

    ///
    /// What a Paint is drawn as at `viewport`, including Glyphs, without a
    /// console pass.
    ///
    /// An `egui::Context` is built here for one reason: a galley needs a font
    /// atlas, and a Glyph is a galley. Tests that assert nothing about a
    /// galley use [`source_geometry`] instead.
    ///
    fn source_shapes(paint: &Paint, viewport: GridViewport, pixels_per_point: f32) -> SourceShapes {
        let ctx = egui::Context::default();
        let mut shapes = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let table = GlyphTable::lay_out(
                ui.ctx(),
                egui::FontId::new(
                    DEFAULT_FONT_SIZE * glyph_scale(viewport.cell_scale()),
                    egui::FontFamily::Monospace,
                ),
            );
            shapes = Some(SourceShapes::new(
                paint,
                &viewport,
                &table,
                pixels_per_point,
                crate::cursor_effects::CursorEffectShapes::default(),
            ));
        });
        output.drop_without_applying_deltas();

        shapes.expect("the pass drew the Source")
    }

    /// The rectangle a `Shape::Rect` covers.
    fn rect_of(shape: &Shape) -> Rect {
        match shape {
            Shape::Rect(rect) => rect.rect,
            other => panic!("{other:?} is not a rectangle"),
        }
    }

    fn close(left: Rect, right: Rect) -> bool {
        (left.min - right.min).length() < 1e-3 && (left.max - right.max).length() < 1e-3
    }

    ///
    /// The Cursor Effect reaches what a Cell is painted *with* and never
    /// where it is painted.
    ///
    /// This is the property the retired `cell_line_width` test held over a
    /// shipped function that took the blink phase and ignored it. Under the
    /// painter the property is structural —
    /// `GridViewport::cell_rect` takes a Position and nothing else — so it is
    /// asserted here against the geometry that actually reached the Shapes.
    ///
    /// The Cursor's visibility is owned by `orcvs`; a console test does not
    /// need a wall-clock seam to assert geometry. What is asserted is the
    /// whole of what the presentation could move — every Cell,
    /// the Cursor's included, occupies exactly the rectangle its Position gives
    /// it, and the Cursor's own stroke is drawn on that same rectangle rather
    /// than beside it or around it.
    ///
    /// The Cell's rectangle is the one it is *stroked* at: a Cell is filled
    /// only where its background differs from the Source, and the Cells that
    /// are filled share their rectangles with their neighbours.
    ///
    #[tokio::test]
    async fn the_cursor_reaches_the_paint_of_a_cell_and_never_its_geometry() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let orcvs = running_orcvs(8, 8);
        let frame = orcvs.render_frame();
        let viewport = presented(screen, 8, 8, 1.0);
        let paint = painted(&frame, viewport, screen);
        let shapes = source_geometry(&paint, viewport, 1.0);

        assert_eq!(paint.cursor(), Some(frame.cursor()));
        // Every Cell but the Cursor's is stroked with its own border, in row
        // order.
        let mut expected = Vec::new();
        for row in 0..8 {
            for column in 0..8 {
                if (column, row) != (0, 0) {
                    expected.push(viewport.cell_rect(column, row));
                }
            }
        }
        assert_eq!(
            shapes.borders.len(),
            expected.len(),
            "stroked {} Cell borders",
            shapes.borders.len()
        );
        for (index, rect) in expected.iter().enumerate() {
            assert!(
                close(rect_of(&shapes.borders[index]), *rect),
                "Cell {index} was stroked at {:?} rather than {rect:?}",
                rect_of(&shapes.borders[index])
            );
        }
        // And the Cursor's Cell is stroked once, by the Cursor, on that same
        // rectangle.
        assert_eq!(shapes.cursor.len(), 1, "the Cursor is one stroke");
        assert!(
            close(rect_of(&shapes.cursor[0]), viewport.cell_rect(0, 0)),
            "the Cursor was stroked at {:?} rather than {:?}",
            rect_of(&shapes.cursor[0]),
            viewport.cell_rect(0, 0)
        );
        // Every background lies on the Cell geometry too, rather than beside
        // it: a run starts and ends on a Cell edge.
        for background in &shapes.backgrounds {
            assert!(
                (rect_of(background).height() - viewport.cell_size).abs() < 1e-3,
                "a background was {} tall against a Cell of {}",
                rect_of(background).height(),
                viewport.cell_size
            );
        }
    }

    ///
    /// A Region larger than one Cell hides the Cursor's own Cell border: the
    /// lasso around the Region is the Cursor's presentation then, so the
    /// Cursor's Cell is stroked as every other Cell is.
    ///
    #[tokio::test]
    async fn a_region_hides_the_cursors_cell_border() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(8, 8);
        let at = |x, y| orcvs.grid().position(x, y).expect("inside the grid");
        let (anchor, cursor) = (at(1, 1), at(4, 3));
        orcvs.select(anchor);
        orcvs.extend(cursor);
        let frame = orcvs.render_frame();
        let viewport = presented(screen, 8, 8, 1.0);
        let shapes = source_geometry(&painted(&frame, viewport, screen), viewport, 1.0);

        assert!(
            shapes.cursor.is_empty(),
            "the Cursor's Cell kept its border"
        );
        assert_eq!(shapes.borders.len(), 64, "every Cell keeps its grid line");
        assert!(!shapes.backgrounds.is_empty(), "the Region was not filled");
    }

    ///
    /// The lasso outlines the whole Region rather than the Cursor's Cell.
    ///
    #[tokio::test]
    async fn the_effect_outline_is_the_region_when_it_spans_and_the_cursor_otherwise() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(8, 8);
        let grid = orcvs.grid();
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        let viewport = presented(screen, 8, 8, 1.0);

        orcvs.select(at(4, 3));
        assert!(close(
            super::effect_outline(&orcvs.render_frame(), &viewport),
            viewport.cell_rect(4, 3)
        ));

        orcvs.select(at(4, 3));
        orcvs.extend(at(1, 1));
        assert!(close(
            super::effect_outline(&orcvs.render_frame(), &viewport),
            Rect::from_min_max(viewport.cell_rect(1, 1).min, viewport.cell_rect(4, 3).max)
        ));
    }

    ///
    /// A Cell's background never paints over a Glyph, whichever Cell that Glyph
    /// belongs to, and the Cursor is painted over both.
    ///
    /// The groups are what makes that expressible. Every fill is in
    /// `backgrounds`, every Glyph in `glyphs` and the Cursor alone in `cursor`,
    /// so chaining the groups orders the *kinds* however the Cells interleave:
    /// a later Cell in the row order cannot erase an earlier Cell's Glyph, and
    /// no neighbour's fill or seam can reach the Cursor. That the shape groups
    /// then arrive at the painter in that order is asserted by
    /// `the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them`.
    ///
    #[tokio::test]
    async fn every_background_is_painted_before_every_glyph_and_the_cursor_after_both() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(8, 8);
        // A Glyph in the first Cell of the Grid, so every other Cell's
        // background is built after it and would paint over it if the Shapes
        // were emitted Cell by Cell.
        orcvs.write("1");
        orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));
        let frame = orcvs.render_frame();
        let viewport = presented(screen, 8, 8, 1.0);
        let paint = painted(&frame, viewport, screen);
        let shapes = source_shapes(&paint, viewport, 1.0);

        assert!(
            !shapes.backgrounds.is_empty(),
            "the Grid painted no Cell background"
        );
        for background in &shapes.backgrounds {
            let Shape::Rect(rect) = background else {
                panic!("a background was {background:?}")
            };
            assert!(
                rect.fill.a() > 0 && rect.stroke.width == 0.0,
                "a background carried a stroke: {rect:?}"
            );
        }
        assert!(!shapes.glyphs.is_empty(), "the Grid painted no Glyph");
        for glyph in &shapes.glyphs {
            assert!(
                matches!(glyph, Shape::Text(_)),
                "a Glyph was {glyph:?} rather than text"
            );
        }
        assert_eq!(shapes.cursor.len(), 1, "the Cursor is one stroke");
        for stroked in shapes.borders.iter().chain(&shapes.cursor) {
            let Shape::Rect(rect) = stroked else {
                panic!("a border was {stroked:?}")
            };
            assert!(
                rect.fill.a() == 0 && rect.stroke.width > 0.0,
                "a border carried a fill: {rect:?}"
            );
        }
        assert_eq!(
            shapes.backgrounds.len(),
            1,
            "only the hidden-caret selection should fill a Cell"
        );
    }

    ///
    /// The shape groups reach the painter end to end, in the order
    /// `SourceShapes::into_shapes` chains them: backgrounds, borders, Glyphs,
    /// seams, the Cursor.
    ///
    /// Read off the flat shape list of a whole console pass, which is the one
    /// place paint order is observable and which also covers the three calls
    /// inside `show_source` that nothing else exercises — derive a Paint, draw
    /// it, extend the painter once. This is not a concession: `egui_kittest`'s
    /// own regression tests assert against `output().shapes` the same way.
    ///
    /// The panel's own fill is in that list too — `source_panel_frame` paints
    /// it before the Grid's first Shape — and it is a plain fill just as a
    /// background run is. It is not subtracted, because every claim below is
    /// that the *last* member of one group precedes the *first* member of the
    /// next, and a fill preceding every Grid Shape moves neither bound.
    ///
    /// A Cell border and the Cursor are both stroked rectangles and nothing
    /// about the Shape tells them apart; what parts them is that the Cursor is
    /// painted after everything else, which is what this asserts.
    ///
    /// The Grid is 16 Cells square because the default Sector Seam spacing is
    /// eight: an 8x8 Grid asks for no sector seam at all, and an empty group
    /// would let the claims either side of it hold vacuously.
    ///
    #[tokio::test]
    async fn the_shape_groups_reach_the_painter_in_the_order_into_shapes_chains_them() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = running_orcvs(16, 16);
        let mut view = SourceView::default();
        // A written Cell, so a Glyph is painted at all.
        orcvs.write("1");
        orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));

        let (_viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        let at = |kind: fn(&Shape) -> bool| -> Vec<usize> {
            shapes
                .iter()
                .enumerate()
                .filter_map(|(index, shape)| kind(shape).then_some(index))
                .collect()
        };
        fn is_fill(shape: &Shape) -> bool {
            matches!(shape, Shape::Rect(rect) if rect.fill.a() > 0 && rect.stroke.width == 0.0)
        }
        let fills = at(is_fill);
        let strokes = at(|shape| matches!(shape, Shape::Rect(rect) if rect.stroke.width > 0.0));
        let glyphs = at(|shape| matches!(shape, Shape::Text(_)));
        let seams = at(|shape| matches!(shape, Shape::LineSegment { .. }));

        assert!(!fills.is_empty(), "the pass painted no fill");
        assert!(strokes.len() > 1, "the pass painted no Cell border");
        assert!(!glyphs.is_empty(), "the pass painted no Glyph");
        assert!(!seams.is_empty(), "the pass painted no sector seam");

        let borders = &strokes;

        assert!(
            *borders.last().expect("a border") < glyphs[0],
            "a border at {:?} painted over the Glyph at {}",
            borders.last(),
            glyphs[0]
        );
        assert!(
            *glyphs.last().expect("a Glyph") < *seams.last().expect("a frame fragment"),
            "the fragmented Cursor frame was not painted last"
        );
    }

    ///
    /// The Source's Cells hold printable ASCII, so that is what the table
    /// covers. The space is the one printable character left out, because a
    /// Cell showing one paints no Glyph.
    ///
    #[test]
    fn the_glyph_table_covers_the_printable_ascii_a_cell_can_hold() {
        let ctx = egui::Context::default();
        let mut table = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            table = Some(GlyphTable::lay_out(
                ui.ctx(),
                egui::FontId::monospace(orcvs::opts::DEFAULT_FONT_SIZE),
            ));
        });
        output.drop_without_applying_deltas();
        let table = table.expect("the pass laid the alphabet out");

        for byte in ALPHABET_FIRST..=ALPHABET_LAST {
            assert!(
                table.galley(char::from(byte)).is_some(),
                "the table does not cover {:?}",
                char::from(byte)
            );
        }
        // Outside the alphabet on both sides, and beyond ASCII altogether.
        // Every one of these falls back to a single uncached layout for that
        // Cell alone.
        for character in [' ', '\n', '\u{7f}', 'é', '✦'] {
            assert!(
                table.galley(character).is_none(),
                "the table claims to cover {character:?}"
            );
        }
    }

    ///
    /// A character the table does not cover is still laid out, and is laid out
    /// as itself.
    ///
    /// `GlyphTable::glyph` answers a galley for every character, because the
    /// step that draws a Cell has to put *something* in the ordered sequence
    /// of Shapes rather than skip the Cell and paint it out of turn. The two
    /// ways it answers are asserted against each other here: inside the
    /// alphabet it hands back the table's own galley — the same `Arc`, not an
    /// equal one, which is the whole point of laying the alphabet out once —
    /// and outside it lays that one character out fresh.
    ///
    /// A Source Cell holds printable ASCII by construction, so the fallback is
    /// for a Source that found a way to hold something else. That is exactly
    /// why it is worth pinning: nothing else reaches it, so a fallback that
    /// silently answered the wrong Glyph would paint the wrong character with
    /// no other test noticing.
    ///
    #[test]
    fn a_character_outside_the_alphabet_is_laid_out_as_itself() {
        let ctx = egui::Context::default();
        let mut table = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            table = Some(GlyphTable::lay_out(
                ui.ctx(),
                egui::FontId::monospace(orcvs::opts::DEFAULT_FONT_SIZE),
            ));
        });
        output.drop_without_applying_deltas();
        let table = table.expect("the pass laid the alphabet out");

        for character in [' ', '\n', '\u{7f}', 'é', '✦'] {
            let galley = table.glyph(character);

            assert!(
                table.galley(character).is_none(),
                "{character:?} is in the alphabet, so this asserts nothing about the fallback"
            );
            assert_eq!(
                galley.text(),
                character.to_string(),
                "the fallback laid out something other than {character:?}"
            );
        }

        for byte in [ALPHABET_FIRST, b'A', ALPHABET_LAST] {
            let character = char::from(byte);
            let cached = table.galley(character).expect("inside the alphabet");

            assert!(
                std::sync::Arc::ptr_eq(&table.glyph(character), cached),
                "{character:?} was laid out again instead of read from the table"
            );
        }
    }

    ///
    /// Every Cell is stroked with its own border, one Grid line wide, and the
    /// Cursor Effect changes that colour rather than that width — which is
    /// what `cell_line_width` returned a constant for.
    ///
    /// The colours come from the Paint rather than from `cell_visuals`: which
    /// colour a Cell's border *is* is decided in the value layer and asserted
    /// there against `cell_visuals` itself. What is asserted here is that the
    /// shape step gives each Cell the border the Paint gave that Cell, and
    /// not its neighbour's.
    ///
    /// The Grid is wider than the Cursor effect, so the borders
    /// are not all one colour and a step that handed every Cell the same
    /// stroke would be caught.
    ///
    #[tokio::test]
    async fn a_cell_border_is_one_grid_line_wide_whatever_the_cell_is_doing() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(320.0, 320.0));
        let orcvs = running_orcvs(20, 20);
        let frame = orcvs.render_frame();
        let viewport = presented(screen, 20, 20, 1.0);
        let paint = painted(&frame, viewport, screen);
        let shapes = source_geometry(&paint, viewport, 1.0);
        // The owned transform scales the stroke with everything else, the way
        // the Scene's layer transform used to, so the width is asserted in the
        // Source's own points.
        let scale = viewport.cell_scale();
        let mut colours = std::collections::BTreeSet::new();

        for (position, cell) in paint.cells() {
            let rect = viewport.cell_rect(position.x(), position.y());
            let stroked = shapes
                .borders
                .iter()
                .chain(&shapes.cursor)
                .find_map(|shape| match shape {
                    Shape::Rect(painted) if close(painted.rect, rect) => Some(painted.stroke),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("Cell {position:?} was never stroked"));

            assert!(
                (stroked.width / scale - GRID_LINE_WIDTH).abs() < 1e-3,
                "the border at {position:?} was {} points wide",
                stroked.width / scale
            );
            assert_eq!(stroked.color, cell.border, "the border at {position:?}");
            colours.insert(stroked.color.to_array());
        }

        assert!(
            colours.len() > 1,
            "every Cell was stroked the same colour, so nothing was told apart"
        );
    }

    ///
    /// A Glyph is painted for every Cell that shows a character, at the centre
    /// of that Cell, in that Cell's own foreground — and for no Cell that shows
    /// the space.
    ///
    /// What a Cell shows and what colour it is are the Paint's answers, made
    /// with no `egui::Context` and asserted against `cell.content()` and
    /// `cell_visuals` in `paint.rs`. What this pins is the step between: a
    /// galley per Cell that has something to say, positioned on that Cell and
    /// handed that Cell's colour rather than its neighbour's.
    ///
    /// The Grid carries an Addition, whose claim reaches past the two Cells it
    /// is spelled in and leaves classified but empty operand Cells behind it.
    /// `syntax-highlighting/03` retired the blank spelling table that used to
    /// stand a placeholder letter in those Cells, so the Cells showing
    /// something are exactly the written ones — the unfilled operand Cells
    /// are tinted (`style::fill_tint_colour`) but spell nothing, which is what
    /// this test's exact count of two asserts rather than only a lower bound.
    ///
    #[tokio::test]
    async fn a_glyph_is_painted_for_every_cell_that_shows_one_and_no_other() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(8, 8);
        for (x, character) in ".+".chars().enumerate() {
            orcvs.select(orcvs.grid().position(x, 2).expect("inside the grid"));
            orcvs.write(&character.to_string());
        }
        orcvs.select(orcvs.grid().position(5, 5).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let viewport = presented(screen, 8, 8, 1.0);
        let paint = painted(&frame, viewport, screen);
        let shapes = source_shapes(&paint, viewport, 1.0);

        let shown: Vec<_> = paint
            .cells()
            .filter(|(_, cell)| cell.character != ' ')
            .map(|(position, _)| position)
            .collect();

        assert_eq!(
            shown.len(),
            2,
            "the two written Cells are the only Cells that should show a character, not {}",
            shown.len()
        );
        assert_eq!(
            shapes.glyphs.len(),
            shown.len(),
            "painted {} Glyphs for {} Cells showing a character",
            shapes.glyphs.len(),
            shown.len()
        );
        for (shape, position) in shapes.glyphs.iter().zip(&shown) {
            let Shape::Text(text) = shape else {
                panic!("the Glyph at {position:?} was {shape:?}")
            };
            let rect = viewport.cell_rect(position.x(), position.y());

            assert_eq!(
                text.fallback_color,
                paint.at(*position).foreground,
                "the Glyph at {position:?}"
            );
            assert!(
                (text.pos - (rect.center() - text.galley.size() / 2.0)).length() < 1e-3,
                "the Glyph at {position:?} was painted at {:?}, off the Cell at {rect:?}",
                text.pos
            );
        }
    }

    ///
    /// The background the Grid declines to paint is the one the panel paints.
    ///
    /// `show_source` omits a Cell's rectangle wherever `cell_visuals` asks for
    /// `SourcePaintSettings::source_background`, and what stands in its place
    /// is the `CentralPanel` frame. The two values are stated in different
    /// places, so nothing but this holds them together: give the panel any
    /// other fill and every ordinary Cell — outside the Cursor effect, most of
    /// the default Grid — renders on a ground the settings value never chose
    /// for it.
    ///
    /// The whole console is checked rather than the constant alone, because it
    /// is the painted result that has to sit on the right colour.
    ///
    #[test]
    fn the_omitted_background_is_the_colour_the_panel_is_filled_with() {
        let source_paint = crate::source_paint::SourcePaintSettings::default();
        assert_eq!(
            source_panel_frame(source_paint.source_background()).fill,
            source_paint.source_background(),
            "show_source omits a Cell's background wherever cell_visuals asks \
             for the settings value's source_background, so the panel \
             standing in for it must be filled with exactly that colour"
        );
    }

    ///
    /// The viewport the `presented` helper builds is the viewport a whole
    /// console pass presents.
    ///
    /// Every Shape assertion here is made at a viewport built from
    /// `grid_viewport` and `presented_grid` directly rather than read back from
    /// a pass, and this is what stops that standing in from drifting from what
    /// `show_source_scene` does. At a fractional device scale as well as at
    /// one, which is where `presented_grid` floors the Cell side to whole
    /// physical pixels and the two could part company.
    ///
    #[tokio::test]
    async fn the_presented_viewport_is_the_one_a_console_pass_presents() {
        for pixels_per_point in [1.0_f32, 1.5] {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
            let mut orcvs = running_orcvs(8, 8);
            let mut view = SourceView::default();

            let (viewport, _) = console_pass_at(
                &ctx,
                screen,
                Vec::new(),
                &mut orcvs,
                &mut view,
                pixels_per_point,
            );

            assert_eq!(
                ctx.pixels_per_point(),
                pixels_per_point,
                "the pass did not run at the device scale it was asked for"
            );
            assert_eq!(viewport, presented(screen, 8, 8, pixels_per_point));
        }
    }

    ///
    /// A coalesced run covers exactly the Cells it replaces.
    ///
    /// Which Cells coalesce is `Paint::background_runs`' answer and is pinned
    /// there against a written-out table of spans; where the rectangle lands is
    /// `a_background_run_is_the_rectangle_its_columns_span`'s, pinned against
    /// written-out coordinates. What this adds is the part only a real
    /// presented viewport has: the run starts at the corner its first Cell was
    /// given by `GridViewport::cell_rect` — the column-to-x function every
    /// other Shape goes through — rather than at a sum of Cell sides
    /// accumulated across the row, it spans the Cells it replaced and no
    /// others, and those Cells would have tiled at exactly the pixels it
    /// covers, every interior edge landing on the same physical pixel from
    /// either side.
    ///
    /// Run at a fractional device scale as well as at one, because at one the
    /// Cell side is a whole point and there is nothing for a snap to move.
    ///
    #[tokio::test]
    async fn a_background_run_covers_exactly_the_cells_it_replaces() {
        for pixels_per_point in [1.0_f32, 1.5] {
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
            let orcvs = running_orcvs(8, 8);
            let frame = orcvs.render_frame();
            let viewport = presented(screen, 8, 8, pixels_per_point);
            let paint = painted(&frame, viewport, screen);
            let shapes = source_geometry(&paint, viewport, pixels_per_point);
            let runs = paint.background_runs();

            assert_eq!(
                shapes.backgrounds.len(),
                runs.len(),
                "painted {} rectangles for {} runs",
                shapes.backgrounds.len(),
                runs.len()
            );

            for (shape, run) in shapes.backgrounds.iter().zip(&runs) {
                let Shape::Rect(painted) = shape else {
                    panic!("a background was {shape:?}")
                };

                assert_eq!(painted.fill, run.colour, "a run was filled wrongly");
                // The near corner is the one the run's first Cell was given,
                // and the far corner is a whole number of Cells away from it.
                // Stated as a span rather than as the far Cell's own corner,
                // because that is the expression `SourceShapes::new` evaluates
                // and a test that re-ran it would pass any simultaneous edit to
                // both. `a_background_run_is_the_rectangle_its_columns_span`
                // pins the same mapping against written-out coordinates.
                assert_eq!(
                    painted.rect.min,
                    viewport
                        .cell_rect(run.columns.start, run.row)
                        .min
                        .round_to_pixels(pixels_per_point),
                    "the run over columns {:?} of row {} starts elsewhere",
                    run.columns,
                    run.row
                );
                // Half a Cell, because the snap rounds the two corners
                // independently and so can move a side by up to half a physical
                // pixel each — while a run one Cell too wide or too narrow is
                // out by a whole Cell side.
                let spanned = run.columns.len() as f32 * viewport.cell_size;
                assert!(
                    (painted.rect.width() - spanned).abs() < viewport.cell_size / 2.0,
                    "the run over columns {:?} of row {} is {} wide, not the {spanned} its Cells span",
                    run.columns,
                    run.row,
                    painted.rect.width()
                );
                assert!(
                    (painted.rect.height() - viewport.cell_size).abs() < viewport.cell_size / 2.0,
                    "the run over columns {:?} of row {} is {} tall, not one Cell",
                    run.columns,
                    run.row,
                    painted.rect.height()
                );
                for column in run.columns.start..run.columns.end - 1 {
                    assert_eq!(
                        viewport
                            .cell_rect(column, run.row)
                            .max
                            .x
                            .round_to_pixels(pixels_per_point),
                        viewport
                            .cell_rect(column + 1, run.row)
                            .min
                            .x
                            .round_to_pixels(pixels_per_point),
                        "the Cells either side of column {column} in row {} do not tile",
                        run.row
                    );
                }
            }
        }
    }

    ///
    /// A coalesced run becomes the rectangle its columns span, at coordinates
    /// written out here rather than re-derived.
    ///
    /// The viewport is stated instead of presented — a 16 point Cell with the
    /// Grid's corner at the origin — so every expected rectangle below is a
    /// literal. That is the point: the assertion this replaced re-ran
    /// `SourceShapes::new`'s own `Rect::from_min_max(cell_rect(start).min,
    /// cell_rect(end - 1).max)`, which catches a one-sided edit and passes a
    /// simultaneous one. These numbers move for neither.
    ///
    /// At one device pixel per point every edge of that Grid is already on a
    /// pixel, so the snap moves nothing and the literals are the coordinates
    /// the paint lands on. What a *fractional* scale does to them is
    /// `a_background_run_covers_exactly_the_cells_it_replaces`'.
    ///
    /// Four runs of the 8 by 8 default Grid, chosen so no one mistake passes
    /// all four: the Cursor's own Cell at the origin, a run of three ending
    /// short of the Grid's right edge, a run spanning a whole row but its last
    /// Cell, and the Grid's last Cell alone. Which Cells those are is
    /// `Paint::background_runs`' answer and is pinned in `paint.rs`.
    ///
    #[tokio::test]
    async fn a_background_run_is_the_rectangle_its_columns_span() {
        let viewport = GridViewport {
            cell_size: 16.0,
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(128.0)),
        };
        let orcvs = running_orcvs(8, 8);
        let frame = orcvs.render_frame();
        let paint = Paint::derive_with_colours(
            FramePaint::new(
                &frame,
                viewport.visible_positions(viewport.rect, frame.grid()),
            ),
            Some(PALETTE.selection_fill),
            crate::cursor_effects::DEFAULT_REGION_COLOUR,
            None,
            crate::source_paint::SourcePaintSettings::default(),
        );
        let shapes = source_geometry(&paint, viewport, 1.0);
        let runs = paint.background_runs();

        assert_eq!(
            shapes.backgrounds.len(),
            runs.len(),
            "painted {} rectangles for {} runs",
            shapes.backgrounds.len(),
            runs.len()
        );

        let (row, columns, expected) = (
            0,
            0..1,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(16.0, 16.0)),
        );
        let index = runs
            .iter()
            .position(|run| run.row == row && run.columns == columns)
            .unwrap_or_else(|| {
                panic!("this Grid asks for no run over columns {columns:?} of row {row}")
            });
        assert_eq!(
            rect_of(&shapes.backgrounds[index]),
            expected,
            "the run over columns {columns:?} of row {row}"
        );
    }

    ///
    /// A whole console pass strokes the Grid at the zoom it presented the
    /// Source at, and snaps its background runs to the device scale it ran on.
    ///
    /// Both are `show_source`'s own arithmetic — the zoom is the presented Cell
    /// side over the Source's own, the device scale is the `Ui`'s — and both
    /// are handed to `SourceShapes::new` and reach the Shapes nowhere else.
    /// Every other Shape assertion here builds a `SourceShapes` through
    /// `source_geometry` or `source_shapes`, which are given a zoom and a device
    /// scale the test chose, so all of them still hold with either argument
    /// replaced by a constant one at the call site. What would ship then is a
    /// Grid whose lines and sector seams stay one Source point wide at every
    /// zoom instead of scaling with it, and runs snapped to whole points on a
    /// screen whose pixels are not whole points.
    ///
    /// The geometry is chosen so neither argument can be mistaken for one. A
    /// 161 point console over a 20 Cell Grid at Zoom 0.5, and at a
    /// device scale of 1.5 the presented Grid's corner is floored a physical
    /// pixel in — two thirds of a point — so every run edge is snapped
    /// somewhere a snap to whole points would not put it.
    ///
    #[tokio::test]
    async fn a_console_pass_strokes_at_its_own_zoom_and_snaps_its_runs_to_its_own_device_scale() {
        const DEVICE_SCALE: f32 = 1.5;
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(161.0));
        let mut orcvs = running_orcvs(20, 20);
        let mut view = SourceView::default();
        pinned_at(&mut view, Vec2::ZERO, 0.5);

        let (viewport, shapes) = console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            DEVICE_SCALE,
        );
        let scale = viewport.cell_scale();

        assert!(
            (scale - 0.5).abs() < 1e-6,
            "the pass fitted the Source at {scale}, and a zoom of one would be \
             indistinguishable from the constant"
        );

        // Every Cell is stroked once — the Cursor's by the Cursor — and every
        // one of those strokes carries the zoom.
        let mut stroked = 0;
        for shape in &shapes {
            if let Shape::Rect(painted) = shape
                && painted.stroke.width > 0.0
            {
                assert!(
                    (painted.stroke.width - GRID_LINE_WIDTH * scale).abs() < 1e-6,
                    "a Cell border was stroked {} points wide against {} at this zoom",
                    painted.stroke.width,
                    GRID_LINE_WIDTH * scale
                );
                stroked += 1;
            }
        }
        assert_eq!(stroked, 399, "the Cursor Cell uses the effect frame");

        // And so does every sector seam, which takes its own width.
        let mut seams = 0;
        for shape in &shapes {
            if let Shape::LineSegment { stroke, .. } = shape
                && (stroke.width - SECTOR_LINE_WIDTH * scale).abs() < 1e-6
            {
                seams += 1;
            }
        }
        assert!(seams > 0, "the pass drew no sector seam");

        // The default theme leaves the cursor-cell colour unset, so the
        // selected Cell uses the panel's source background and contributes no
        // background run to snap. Explicit colours are covered by the Paint
        // seam tests.
        let frame = orcvs.render_frame();
        let paint = Paint::derive_with_colours(
            FramePaint::new(&frame, viewport.visible_positions(screen, frame.grid())),
            None,
            crate::cursor_effects::DEFAULT_REGION_COLOUR,
            None,
            crate::source_paint::SourcePaintSettings::default(),
        );
        let runs = paint.background_runs();
        assert!(
            runs.is_empty(),
            "the default cursor colour should be transparent"
        );
    }

    ///
    /// A sector seam is drawn on the Cell edge the Paint asks for, in the
    /// colour it asks for, one sector line wide.
    ///
    /// Which Cells carry a seam, at what strength, and that the Cursor's own
    /// Cell carries none while it is framed on its own, are the Render
    /// Frame's and the derive's answers and
    /// are asserted in `paint.rs` with no Context at all. The seam's *width*
    /// scales with the Cell side, so it is geometry and belongs here.
    ///
    /// The Grid is 16 Cells square because the default Sector Seam spacing is
    /// eight: an 8x8 Grid has no column or row that is a non-zero multiple of
    /// it, so every seam strength in one is `None` and this would assert
    /// nothing.
    ///
    #[tokio::test]
    async fn a_sector_seam_is_drawn_on_the_cell_edge_the_paint_asks_for() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = running_orcvs(16, 16);
        // The Cursor goes on a Cell that would otherwise carry both seams, so
        // the suppression the derive applies is visible as an absence here too.
        orcvs.select(orcvs.grid().position(8, 8).expect("inside the grid"));

        let frame = orcvs.render_frame();
        let viewport = presented(screen, 16, 16, 1.0);
        let paint = painted(&frame, viewport, screen);
        let shapes = source_geometry(&paint, viewport, 1.0);
        let scale = viewport.cell_scale();

        let mut expected = Vec::new();
        for (position, cell) in paint.cells() {
            let rect = viewport.cell_rect(position.x(), position.y());

            for (colour, ends) in [
                (cell.sector_left, [rect.left_top(), rect.left_bottom()]),
                (cell.sector_top, [rect.left_top(), rect.right_top()]),
            ] {
                if let Some(colour) = colour {
                    expected.push((position, colour, ends));
                }
            }
        }

        assert!(
            !expected.is_empty(),
            "the Paint asked for no sector seam, so nothing was asserted"
        );
        assert_eq!(
            shapes.seams.len(),
            expected.len(),
            "drew {} seams against {} the Paint asked for",
            shapes.seams.len(),
            expected.len()
        );
        for (shape, (position, colour, ends)) in shapes.seams.iter().zip(&expected) {
            let Shape::LineSegment { points, stroke } = shape else {
                panic!("the seam at {position:?} was {shape:?}")
            };

            assert!(
                (points[0] - ends[0]).length() < 1e-3 && (points[1] - ends[1]).length() < 1e-3,
                "the seam at {position:?} ran {points:?} rather than {ends:?}"
            );
            assert_eq!(stroke.color, *colour, "the seam at {position:?}");
            assert!(
                (stroke.width / scale - SECTOR_LINE_WIDTH).abs() < 1e-3,
                "the seam at {position:?} was {} points wide",
                stroke.width / scale
            );
        }
    }

    ///
    /// A middle drag that starts on a Cell pans the Source.
    ///
    /// This is what `Sense::CLICK` on the Grid rectangle buys. Within one layer
    /// a later-registered child wins the click tie and would win the drag tie
    /// too if it sensed drag, and the Grid is registered after the pan
    /// response. Sensing clicks alone is what leaves the drag to the pan
    /// rectangle, and the drag has to be started *over the Grid* to assert it.
    ///
    #[tokio::test]
    async fn a_middle_drag_that_starts_on_a_cell_still_pans_the_source() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let over_a_cell = viewport.cell_rect(4, 4).center();
        assert!(
            viewport.rect.contains(over_a_cell),
            "the drag did not start over the Grid"
        );
        let before = view.pan;

        console_frame(
            &ctx,
            screen,
            middle_press_at(over_a_cell),
            &mut orcvs,
            &mut view,
        );
        console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(over_a_cell + Vec2::new(-40.0, -25.0))],
            &mut orcvs,
            &mut view,
        );

        assert_ne!(
            view.pan, before,
            "a middle drag over a Cell did not pan the Source"
        );
    }

    ///
    /// Alt (Option) held with a primary drag Pans by exactly what the
    /// pointer moved, at a scale that is not one — the same claim
    /// `a_middle_drag_pans_by_the_pointer_and_a_later_click_selects_the_cell_under_it`
    /// makes for the middle button, and the same reason Zoom 2.0 is chosen:
    /// a leftover multiply by the Zoom would move the Source by the Zoom
    /// times the pointer.
    ///
    #[tokio::test]
    async fn alt_held_with_a_primary_drag_pans_by_exactly_what_the_pointer_moved() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        pinned_at(&mut view, Vec2::ZERO, 2.0);

        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.to_global.scaling, 2.0);
        let anchor = view.to_global.translation;

        let from = screen.min + Vec2::splat(40.0);
        let moved = Vec2::new(-40.0, -24.0);
        console_frame(
            &ctx,
            screen,
            alt_primary_press_at(from),
            &mut orcvs,
            &mut view,
        );
        console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(from + moved)],
            &mut orcvs,
            &mut view,
        );

        assert_eq!(
            view.to_global.translation - anchor,
            moved,
            "the Source panned by {:?} for a pointer that moved {moved:?}",
            view.to_global.translation - anchor
        );

        console_frame(
            &ctx,
            screen,
            alt_primary_release_at(from + moved),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(
            selected_cell(&orcvs),
            (0, 0),
            "an Alt-drag moved the Cursor"
        );
    }

    ///
    /// A primary drag without Alt does not Pan — it is either a click, which
    /// `a_click_selects_the_cell_under_the_pointer_whatever_the_window_size`
    /// already covers, or a drag that selects a Region, which
    /// `a_primary_drag_spans_a_region_that_release_keeps` covers.
    ///
    #[tokio::test]
    async fn a_primary_drag_without_alt_does_not_pan() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let over_a_cell = viewport.cell_rect(4, 4).center();
        let before = view.pan;

        console_frame(&ctx, screen, click_at(over_a_cell), &mut orcvs, &mut view);
        console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(over_a_cell + Vec2::new(-40.0, -25.0))],
            &mut orcvs,
            &mut view,
        );

        assert_eq!(
            view.pan, before,
            "a primary drag without Alt panned the Source"
        );
    }

    fn region_of(orcvs: &Orcvs) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
        let region = orcvs.region();
        (region.columns(), region.rows())
    }

    ///
    /// A primary press sets the anchor on the pressed Cell, the drag moves the
    /// Cursor to the Cell under the pointer, and release keeps the Region.
    ///
    #[tokio::test]
    async fn a_primary_drag_spans_a_region_that_release_keeps() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let (from, to) = (
            viewport.cell_rect(6, 4).center(),
            viewport.cell_rect(2, 2).center(),
        );
        console_frame(&ctx, screen, click_at(from), &mut orcvs, &mut view);
        console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(to)],
            &mut orcvs,
            &mut view,
        );
        assert_eq!(region_of(&orcvs), (2..7, 2..5));
        assert_eq!(selected_cell(&orcvs), (2, 2), "the Cursor is the live end");

        console_frame(&ctx, screen, release_at(to), &mut orcvs, &mut view);
        assert_eq!(
            region_of(&orcvs),
            (2..7, 2..5),
            "release dropped the Region"
        );
    }

    ///
    /// A primary click collapses the Region onto the clicked Cell, and Shift
    /// with a click extends it from the anchor it already has.
    ///
    #[tokio::test]
    async fn a_click_collapses_the_region_and_a_shift_click_extends_it() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        let grid = orcvs.grid();
        orcvs.select(grid.position(1, 1).expect("inside the Grid"));
        orcvs.extend(grid.position(9, 9).expect("inside the Grid"));

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        click(
            &ctx,
            screen,
            viewport.cell_rect(3, 2).center(),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(region_of(&orcvs), (3..4, 2..3));

        let shift_at = viewport.cell_rect(7, 5).center();
        let shifted = |pressed| Event::PointerButton {
            pos: shift_at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::SHIFT,
        };
        console_frame(
            &ctx,
            screen,
            vec![
                Event::PointerMoved(shift_at),
                Event::ModifiersChanged(Modifiers::SHIFT),
                shifted(true),
            ],
            &mut orcvs,
            &mut view,
        );
        // Shift is let go after the button, as a viewer lets it go: the
        // click resolves on release and reads Shift then.
        console_frame(&ctx, screen, vec![shifted(false)], &mut orcvs, &mut view);
        console_frame(
            &ctx,
            screen,
            vec![Event::ModifiersChanged(Modifiers::NONE)],
            &mut orcvs,
            &mut view,
        );
        assert_eq!(region_of(&orcvs), (3..8, 2..6));
        assert_eq!(selected_cell(&orcvs), (7, 5));
    }

    ///
    /// A drag past the console's edge moves the Cursor past it and scrolls
    /// after it: faster the further the pointer is outside, never more than
    /// one Cell a frame, and never past the Grid.
    ///
    #[tokio::test]
    async fn a_drag_past_the_edge_scrolls_faster_the_further_out_and_stops_at_the_grid() {
        // The Pan step each of `frames` frames takes with the pointer held
        // `outside` points past the console's right edge.
        fn steps(outside: f32, frames: usize) -> (Vec<f32>, Orcvs, SourceView, Rect) {
            let ctx = egui::Context::default();
            let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
            let mut orcvs = running_orcvs(32, 32);
            let mut view = SourceView::default();
            let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            let from = viewport.cell_rect(20, 5).center();
            console_frame(&ctx, screen, click_at(from), &mut orcvs, &mut view);
            console_frame(
                &ctx,
                screen,
                vec![Event::PointerMoved(Pos2::new(
                    screen.max.x + outside,
                    from.y,
                ))],
                &mut orcvs,
                &mut view,
            );
            let mut steps = Vec::new();
            for _ in 0..frames {
                let before = view.pan.x;
                console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
                steps.push(before - view.pan.x);
            }
            (steps, orcvs, view, screen)
        }

        let (near, ..) = steps(8.0, 3);
        let (far, orcvs, view, screen) = steps(200.0, 40);
        let side = CELL_SIZE;

        assert!(near[0] > 0.0, "a drag just past the edge did not scroll");
        assert!(
            far[0] > near[0],
            "a drag further out scrolled no faster: {far:?} against {near:?}"
        );
        assert!(
            far.iter().chain(&near).all(|step| *step <= side),
            "a frame scrolled more than one Cell: {far:?}"
        );
        // Never past the Grid: a Cursor on the last Column brings the margin
        // past it into view, and the console's right edge meets the margin's
        // (ADR 0047).
        assert_eq!(view.pan.x, screen.width() - 32.0 * side - 2.0 * MARGIN);
        assert_eq!(selected_cell(&orcvs), (31, 5));
    }

    ///
    /// Releasing a drag held past the edge does not jump the Source View to
    /// the Cursor: the frame the button comes up still scrolls at most a Cell.
    ///
    #[tokio::test]
    async fn releasing_a_drag_past_the_edge_scrolls_no_more_than_a_cell() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(64, 32);
        let mut view = SourceView::default();
        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let from = viewport.cell_rect(20, 5).center();
        let far = Pos2::new(screen.max.x + 400.0, from.y);
        console_frame(&ctx, screen, click_at(from), &mut orcvs, &mut view);
        // Far enough out that the Cursor is many Cells past the edge.
        console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(far)],
            &mut orcvs,
            &mut view,
        );

        for events in [release_at(far), Vec::new(), Vec::new()] {
            let before = view.pan.x;
            console_frame(&ctx, screen, events, &mut orcvs, &mut view);
            assert!(
                before - view.pan.x <= CELL_SIZE,
                "a frame around the release scrolled {}",
                before - view.pan.x
            );
        }
    }

    ///
    /// Alt with a primary drag, and a middle drag, Pan and leave the Region as
    /// it was.
    ///
    #[tokio::test]
    async fn an_alt_drag_and_a_middle_drag_leave_the_region() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        let grid = orcvs.grid();
        orcvs.select(grid.position(1, 1).expect("inside the Grid"));
        orcvs.extend(grid.position(4, 3).expect("inside the Grid"));
        let region = orcvs.region();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let from = viewport.cell_rect(8, 8).center();
        let to = from + Vec2::new(-60.0, -40.0);
        for (press, release) in [
            (alt_primary_press_at(from), alt_primary_release_at(to)),
            (
                middle_press_at(from),
                vec![Event::PointerButton {
                    pos: to,
                    button: egui::PointerButton::Middle,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            ),
        ] {
            let before = view.pan;
            console_frame(&ctx, screen, press, &mut orcvs, &mut view);
            console_frame(
                &ctx,
                screen,
                vec![Event::PointerMoved(to)],
                &mut orcvs,
                &mut view,
            );
            console_frame(&ctx, screen, release, &mut orcvs, &mut view);
            assert_ne!(view.pan, before, "the drag did not Pan");
            assert_eq!(orcvs.region(), region, "the drag moved the Region");
        }
    }

    ///
    /// The view a viewer reaches by zooming and panning, set directly.
    ///
    /// The gestures that write these fields are asserted elsewhere. This is a
    /// test building its own input below the shipped entry point rather than a
    /// seam cut into one: nothing in `show_source` or `show_source_scene`
    /// exists for it.
    ///
    fn pinned_at(view: &mut SourceView, pan: Vec2, zoom: f32) {
        view.zoom = zoom;
        view.pan = pan;
    }

    ///
    /// The console draws the visible Position range and nothing else, so the
    /// cost of a Render Frame follows the viewport rather than the Source.
    ///
    /// Two claims at two layers, and both are here because only a console pass
    /// carries the wiring between them. That a zoom reaches fewer Positions and
    /// emits fewer Shapes is asserted through the pass itself. That every drawn
    /// Position leaves exactly one Cell-sized *stroked* rectangle — its own
    /// border, or the Cursor's stroke on the selected Cell — is asserted
    /// against the `SourceShapes` groups, counted rather than recovered from
    /// the flat list by a fill and a stroke width. Both are compared with
    /// `GridViewport::visible_positions`' real answer rather than with a
    /// counter the console keeps for a test's benefit.
    ///
    /// The shape total is a bound rather than an equality. Issue 04 coalesces
    /// consecutive backgrounds into one rectangle, so a row's shapes are not
    /// one per Cell and the two totals are not proportional. What the bound
    /// says is the claim this change makes: cost follows the viewport, not the
    /// Source. No frame time is asserted — epaint already discards off-screen
    /// shapes at tessellation (`coarse_tessellation_culling`), so what this
    /// saves is Cell iteration and Shape construction. The Cell iteration is
    /// counted in `paint.rs`, which needs no `Context` to count it.
    ///
    #[tokio::test]
    async fn the_draw_loop_paints_the_visible_range_rather_than_the_whole_source() {
        let ctx = egui::Context::default();
        // The default Grid at the Source's own Cell size, with its margin on
        // every side, is exactly this console, so the first pass at Zoom 1.0
        // has every Cell on screen.
        let screen = Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(
                DEFAULT_COL_COUNT as f32 * CELL_SIZE,
                DEFAULT_ROW_COUNT as f32 * CELL_SIZE,
            ) + Vec2::splat(2.0 * MARGIN),
        );
        let mut orcvs = running_orcvs(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT);
        let mut view = SourceView::default();

        let (whole, every_shape) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            whole.cell_size, CELL_SIZE,
            "the console did not open at Zoom 1.0"
        );
        assert!(
            screen.contains_rect(whole.rect),
            "the Grid at {:?} does not lie wholly inside the console {screen:?}",
            whole.rect
        );
        let all_positions = whole.visible_positions(screen, orcvs.grid());
        assert_eq!(
            all_positions.count(),
            DEFAULT_COL_COUNT * DEFAULT_ROW_COUNT,
            "the console did not show the whole Grid at Zoom 1.0"
        );

        pinned_at(&mut view, Vec2::new(-500.0, -300.0), MAX_ZOOM);
        let (zoomed, fewer_shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let some_positions = zoomed.visible_positions(screen, orcvs.grid());

        assert_eq!(
            zoomed.cell_size,
            CELL_SIZE * MAX_ZOOM,
            "the console did not zoom to {MAX_ZOOM}"
        );
        assert!(
            some_positions.columns.start > 0 && some_positions.rows.start > 0,
            "the zoom left the Grid's first Position on screen, so nothing was culled on the near side"
        );
        assert!(
            some_positions.count() * 2 < all_positions.count(),
            "the zoom showed {} of {} Positions, which is not a fraction worth asserting about",
            some_positions.count(),
            all_positions.count()
        );
        assert!(
            fewer_shapes.len() * 2 < every_shape.len(),
            "{} shapes for {} Positions against {} shapes for {}",
            fewer_shapes.len(),
            some_positions.count(),
            every_shape.len(),
            all_positions.count()
        );

        // One stroked Cell rectangle per drawn Position — the Cell's own
        // border, or the Cursor's stroke on the selected Cell — at both
        // viewports. Counted off the groups the shape step builds rather than
        // recovered from the flat list by their fill and stroke width.
        let frame = orcvs.render_frame();
        for (viewport, positions) in [(whole, all_positions), (zoomed, some_positions)] {
            let shapes = source_shapes(&painted(&frame, viewport, screen), viewport, 1.0);

            assert_eq!(
                shapes.borders.len() + shapes.cursor.len(),
                positions.count(),
                "at a Cell size of {} the shape step drew {} Cells for {} drawn Positions",
                viewport.cell_size,
                shapes.borders.len() + shapes.cursor.len(),
                positions.count()
            );
        }
    }

    ///
    /// A zoomed console fills every Cell its Paint asks to fill, and no other,
    /// with rectangles built from runs that begin and end mid-row.
    ///
    /// Runs are row-local state, opened and flushed inside one row. Under
    /// culling a row no longer starts at column zero or ends at the last
    /// column, so a run opens and is flushed at columns the Source's own row
    /// does not begin or end at. That the fold gets that right is
    /// `Paint::background_runs`' question and is asserted in `paint.rs`, where
    /// it needs no viewport. What is asserted here is the half that does need
    /// one: that the rectangle the shape step builds from a run whose columns
    /// start mid-row still covers exactly the Cells that run replaces.
    ///
    /// The console is small and the zoom is at the limit so both sides of the
    /// Source are culled. Cursor effects are geometry beneath the Grid and
    /// therefore must not reintroduce Cell background runs in this view.
    ///
    #[tokio::test]
    async fn a_zoomed_row_leaves_cursor_effects_out_of_cell_fills() {
        let ctx = egui::Context::default();
        // Narrow enough that every drawn row starts and ends inside the Grid.
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let mut orcvs = running_orcvs(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT);
        let mut view = SourceView::default();
        // Below the window the console shows, keeping the Cursor near the
        // visible range while its area remains separate geometry.
        orcvs.select(orcvs.grid().position(18, 20).expect("inside the grid"));

        // The Grid's near corner at (-700, -500), so the console shows a window
        // in the middle of it rather than a corner. The Pan places the margin,
        // which is two Cells of 32 points at `MAX_ZOOM`, so it is 64 further.
        pinned_at(&mut view, Vec2::new(-764.0, -564.0), MAX_ZOOM);
        let (viewport, _) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let visible = viewport.visible_positions(screen, orcvs.grid());

        assert!(
            visible.columns.start > 0 && visible.columns.end < DEFAULT_COL_COUNT,
            "the zoom culled nothing on one side, so no run begins or ends mid-row"
        );

        let frame = orcvs.render_frame();
        let paint = painted(&frame, viewport, screen);
        let shapes = source_shapes(&paint, viewport, 1.0);
        // The fill covering a point, and how many rectangles claim it. A Cell
        // covered twice is a run painted over a run, which the count catches
        // where a lookup of the first match would not.
        let covering = |point: Pos2| {
            shapes
                .backgrounds
                .iter()
                .filter_map(|shape| match shape {
                    Shape::Rect(rect) if rect.rect.contains(point) => Some(rect.fill),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        let mut filled = 0;
        let mut unfilled = 0;
        let mut opened_at_first_drawn = 0;
        let mut flushed_at_last_drawn = 0;
        for (position, cell) in paint.cells() {
            let rect = viewport.cell_rect(position.x(), position.y());

            match cell.background {
                Some(colour) => {
                    assert_eq!(
                        covering(rect.center()),
                        vec![colour],
                        "the Cell {position:?} the Paint fills was covered wrongly"
                    );
                    filled += 1;
                    opened_at_first_drawn += usize::from(position.x() == visible.columns.start);
                    flushed_at_last_drawn += usize::from(position.x() == visible.columns.end - 1);
                }
                None => {
                    assert!(
                        covering(rect.center()).is_empty(),
                        "the Cell {position:?} the Paint leaves to the Source fill was covered"
                    );
                    unfilled += 1;
                }
            }
        }

        assert_eq!(filled, 0, "Cursor geometry filled a Cell background");
        assert!(unfilled > 0, "the viewport drew no Cells");
        assert_eq!(opened_at_first_drawn, 0);
        assert_eq!(flushed_at_last_drawn, 0);
    }

    ///
    /// Every sector seam the Render Frame asks for inside a culled viewport is
    /// painted, so culling cannot drop a seam a viewer can see.
    ///
    /// This is the acceptance criterion about seams at the edges of the visible
    /// range, asserted from the Render Frame at a zoom that culls on all four
    /// sides. `sector_seams_are_painted_where_the_render_frame_asks_and_never_on_the_cursor`
    /// runs on a default `SourceView` — the fit, with every Position drawn — so
    /// it says nothing about a range-limited row.
    ///
    /// Only seams lying *strictly* inside the clip are asserted. A segment on
    /// the clip's own edge is what the clip is entitled to discard, and it is
    /// the only thing the one-Cell margin reaches: a margin Cell's left edge is
    /// the right boundary of what is shown, never inside it. So this test pins
    /// the criterion, and the margin is the separate, deliberate over-draw its
    /// own comment describes.
    ///
    /// Two pans rather than one, because which seams land strictly inside the
    /// clip is a property of the pan. The Sector Seam spacing is 8 and a Cell is 32
    /// points at this zoom, so one pan is chosen to put a seam column
    /// immediately inside the first drawn column and the other to put one on
    /// the last: between them a cull that is short by a Cell on any of the four
    /// sides drops a seam this asserts. At a single pan the nearest seam can
    /// sit six columns from the edge and a column-side error goes unseen.
    ///
    #[tokio::test]
    async fn a_zoomed_console_paints_every_sector_seam_inside_the_clip() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));

        let mut asserted = 0;
        for translation in [Vec2::new(-486.0, -333.0), Vec2::new(-525.0, -333.0)] {
            let mut orcvs = running_orcvs(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT);
            let mut view = SourceView::default();

            let frame = orcvs.render_frame();
            pinned_at(&mut view, translation, MAX_ZOOM);
            let (viewport, shapes) = console_pass(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
            let visible = viewport.visible_positions(screen, orcvs.grid());
            let painted: Vec<_> = shapes
                .iter()
                .filter_map(|shape| match shape {
                    Shape::LineSegment { points, .. } => Some(*points),
                    _ => None,
                })
                .collect();

            assert!(
                visible.columns.start > 0
                    && visible.columns.end < DEFAULT_COL_COUNT
                    && visible.rows.start > 0
                    && visible.rows.end < DEFAULT_ROW_COUNT,
                "the pan {translation:?} culled nothing on one side, so no seam is near a culled edge: {visible:?}"
            );

            for cell in frame.cells() {
                let position = cell.position();
                let rect = viewport.cell_rect(position.x(), position.y());
                let spacing = frame.sector_seam_spacing().cells();
                let selected = position == frame.cursor();

                for (strength, ends) in [
                    (
                        crate::marks::sector_left_strength(position, spacing),
                        [rect.left_top(), rect.left_bottom()],
                    ),
                    (
                        crate::marks::sector_top_strength(position, spacing),
                        [rect.left_top(), rect.right_top()],
                    ),
                ] {
                    if strength.is_none() || selected {
                        continue;
                    }
                    let span = Rect::from_two_pos(ends[0], ends[1]);
                    let inside = span.min.x > screen.min.x
                        && span.max.x < screen.max.x
                        && span.min.y > screen.min.y
                        && span.max.y < screen.max.y;
                    if !inside {
                        continue;
                    }

                    assert!(
                        painted.iter().any(|points| {
                            (points[0] - ends[0]).length() < 1e-3
                                && (points[1] - ends[1]).length() < 1e-3
                        }),
                        "the seam at {position:?} is inside the clip at pan \
                         {translation:?} and was not painted"
                    );
                    asserted += 1;
                }
            }
        }

        assert!(
            asserted > 0,
            "no seam lies strictly inside the clip, so this asserted nothing"
        );
    }

    const WIDE: Vec2 = Vec2::new(400.0, 200.0);
    const TALL: Vec2 = Vec2::new(200.0, 400.0);

    ///
    /// A Source smaller than the console has nowhere to Pan; one larger than
    /// the console reaches its own edges and no further.
    ///
    #[test]
    fn clamp_pan_pins_a_smaller_source_at_the_origin_and_a_larger_one_to_its_edges() {
        assert_eq!(
            clamp_pan(
                Vec2::new(-10.0, 5.0),
                Vec2::new(400.0, 200.0),
                Vec2::new(128.0, 128.0)
            ),
            Vec2::ZERO,
            "a smaller Source left the top-left"
        );
        assert_eq!(
            clamp_pan(
                Vec2::new(-200.0, -50.0),
                Vec2::new(400.0, 200.0),
                Vec2::new(512.0, 512.0)
            ),
            Vec2::new(-112.0, -50.0)
        );
        assert_eq!(
            clamp_pan(
                Vec2::new(20.0, -400.0),
                Vec2::new(400.0, 200.0),
                Vec2::new(512.0, 512.0)
            ),
            Vec2::new(0.0, -312.0)
        );
    }

    ///
    /// The console opens at Zoom 1.0 whatever the window size, so a resize
    /// shows more or less of the Source and never a different Cell size.
    ///
    #[tokio::test]
    async fn a_resize_keeps_the_cell_size_and_shows_more_or_less_of_the_source() {
        let ctx = egui::Context::default();
        let wide = Rect::from_min_size(Pos2::ZERO, WIDE);
        let tall = Rect::from_min_size(Pos2::ZERO, TALL);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        let before = console_frame(&ctx, wide, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.zoom, 1.0);
        assert_eq!(before.cell_size, CELL_SIZE);
        let after = console_frame(&ctx, tall, Vec::new(), &mut orcvs, &mut view);

        assert_eq!(view.zoom, 1.0, "a resize changed the Zoom");
        assert_eq!(after.cell_size, CELL_SIZE, "a resize changed the Cell size");
        assert_eq!(
            after.rect.min,
            tall.min + Vec2::splat(MARGIN),
            "a resize moved the Source off its margin in from the top-left"
        );
        click(
            &ctx,
            tall,
            after.rect.min + Vec2::new(3.5, 1.5) * after.cell_size,
            &mut orcvs,
            &mut view,
        );
        assert_eq!(selected_cell(&orcvs), (3, 1));
    }

    ///
    /// A Pan that would open a gap past an edge after a resize settles back
    /// inside the Grid.
    ///
    #[tokio::test]
    async fn a_resize_that_would_open_a_gap_settles_the_source_view_back_inside() {
        let ctx = egui::Context::default();
        let small = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let large = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        pinned_at(&mut view, Vec2::new(-200.0, -200.0), 1.0);
        console_frame(&ctx, small, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.pan, Vec2::new(-200.0, -200.0));

        // 512 points of Grid and a margin either side is 576, so the far edge
        // of a 400 point console is -176.
        console_frame(&ctx, large, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.pan,
            Vec2::new(-176.0, -176.0),
            "the resize left a gap past the Grid: {:?}",
            view.pan
        );
    }

    ///
    /// Pinch and command-wheel no longer Zoom. Zoom is a change of Cell size
    /// from the keyboard alone.
    ///
    #[tokio::test]
    async fn pinch_and_command_wheel_do_not_zoom() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let over = viewport.rect.min + Vec2::splat(viewport.cell_size);
        console_frame(&ctx, screen, pinch_at(over), &mut orcvs, &mut view);
        assert_eq!(view.zoom, 1.0, "a pinch changed the Zoom");
        assert_eq!(
            view.to_global.scaling, 1.0,
            "a pinch changed the presented scale"
        );

        console_frame(&ctx, screen, command_wheel_at(over), &mut orcvs, &mut view);
        assert_eq!(view.zoom, 1.0, "a command-wheel changed the Zoom");
        assert_eq!(
            view.to_global.scaling, 1.0,
            "a command-wheel changed the presented scale"
        );
    }

    ///
    /// Command `=` and command `+` step the Zoom in; command `-` steps it
    /// out; command `0` returns it to 1.0 whatever step it was on.
    ///
    #[tokio::test]
    async fn command_chords_step_the_zoom_and_command_zero_resets_it() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.zoom, 1.0);

        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Equals),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(view.zoom, 1.125, "command Equals did not step the Zoom in");

        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Plus),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(view.zoom, 1.25, "command Plus did not step the Zoom in");

        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Minus),
            &mut orcvs,
            &mut view,
        );
        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Minus),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(
            view.zoom, 1.0,
            "two command Minus did not undo two steps in"
        );

        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Minus),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(view.zoom, 0.875, "command Minus did not step the Zoom out");

        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Num0),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(view.zoom, 1.0, "command Num0 did not reset the Zoom to 1.0");
    }

    ///
    /// The Zoom stops exactly at [`MIN_ZOOM`] and [`MAX_ZOOM`] rather than
    /// passing them, however many times the chord repeats.
    ///
    #[tokio::test]
    async fn command_zoom_stops_at_the_range_limits() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        for _ in 0..16 {
            console_frame(
                &ctx,
                screen,
                command_zoom_at(Key::Equals),
                &mut orcvs,
                &mut view,
            );
        }
        assert_eq!(
            view.zoom, MAX_ZOOM,
            "command Equals passed the Zoom's ceiling"
        );

        for _ in 0..32 {
            console_frame(
                &ctx,
                screen,
                command_zoom_at(Key::Minus),
                &mut orcvs,
                &mut view,
            );
        }
        assert_eq!(view.zoom, MIN_ZOOM, "command Minus passed the Zoom's floor");
    }

    ///
    /// A command Zoom that would open a gap past an edge settles the Source
    /// View back inside the Grid, through the same `clamp_pan` a Pan uses.
    ///
    /// The Cursor is moved to the Cell the pinned Pan already shows, at the
    /// far corner, so issue 05's follow has nothing to do here: what settles
    /// the gap below is `clamp_pan` alone, which is this test's own claim.
    ///
    #[tokio::test]
    async fn a_command_zoom_that_would_open_a_gap_settles_the_source_view_back_inside() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        orcvs.select(orcvs.grid().position(31, 31).expect("inside the grid"));

        // 32 Cells of 16 points is 512 points; at `MAX_ZOOM` that is 1024, with
        // a 64 point margin either side, and panning fully to the far edge of
        // a 200 point console takes -952.
        pinned_at(&mut view, Vec2::new(-952.0, -952.0), MAX_ZOOM);
        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.pan,
            Vec2::new(-952.0, -952.0),
            "the fixture did not open already pinned to the far edge"
        );

        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Minus),
            &mut orcvs,
            &mut view,
        );
        assert_eq!(view.zoom, 1.875, "command Minus did not step the Zoom out");
        // At 1.875 the source is 960 points and its margins 60 each, so the
        // far edge of a 200 point console is -880: the old -952 Pan now opens
        // a 72 point gap past it.
        assert_eq!(
            view.pan,
            Vec2::new(-880.0, -880.0),
            "the Zoom left a gap past the Grid: {:?}",
            view.pan
        );
    }

    ///
    /// A fresh `SourceView`'s first frame does not Pan to the Cursor, however
    /// far a fixture puts it from an unpanned top-left origin. Issue 05's
    /// follow needs a previous Cursor to compare against, and there is none
    /// yet on the very first frame — the same reason a Source reload would
    /// not surprise a viewer either.
    ///
    #[tokio::test]
    async fn a_fresh_source_view_does_not_pan_to_the_cursor_on_its_first_frame() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        // Column 30 at Zoom 1.0 is far outside a 200 point console.
        orcvs.select(orcvs.grid().position(30, 30).expect("inside the grid"));

        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "the first frame panned to a Cursor it had no previous position for"
        );
    }

    ///
    /// A Cursor move that would leave the Cursor outside the Source View Pans
    /// the least distance that shows the whole Cursor Cell, still bounded by
    /// the Grid's edges.
    ///
    #[tokio::test]
    async fn a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

        // Column and row 30 at Zoom 1.0 sit at 512..528 once the margin is
        // counted, entirely past a 200 point console on both axes.
        orcvs.select(orcvs.grid().position(30, 30).expect("inside the grid"));
        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        assert_eq!(
            view.pan,
            Vec2::new(-328.0, -328.0),
            "the Cursor move did not Pan the least distance that shows it: {:?}",
            view.pan
        );
    }

    ///
    /// A Cursor move onto the Grid's last or first Column and Row brings the
    /// margin past that edge into view with it, so the Grid's edge shows as
    /// an edge rather than flush against the console's (ADR 0047).
    ///
    #[tokio::test]
    async fn a_cursor_follow_to_an_edge_cell_shows_the_margin_past_it() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

        // 32 Cells of 16 points and a 32 point margin either side is 576
        // points, so the far margin's edge meets a 200 point console at -376.
        orcvs.select(orcvs.grid().position(31, 31).expect("inside the grid"));
        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.pan,
            Vec2::new(-376.0, -376.0),
            "the follow to the last Column and Row left the far margin off screen: {:?}",
            view.pan
        );

        orcvs.select(orcvs.grid().position(0, 0).expect("inside the grid"));
        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "the follow back to the first Column and Row left the near margin off screen: {:?}",
            view.pan
        );
    }

    ///
    /// A Zoom that would leave the Cursor outside the Source View Pans the
    /// least distance that shows it, the same as a Cursor move does.
    ///
    #[tokio::test]
    async fn a_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        // Already past a 200 point console at Zoom 1.0 (240..256), but the
        // first frame below only records it: see
        // `a_fresh_source_view_does_not_pan_to_the_cursor_on_its_first_frame`.
        orcvs.select(orcvs.grid().position(15, 15).expect("inside the grid"));

        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "the fixture's first frame already Panned"
        );

        console_frame(
            &ctx,
            screen,
            command_zoom_at(Key::Equals),
            &mut orcvs,
            &mut view,
        );

        assert_eq!(view.zoom, 1.125, "command Equals did not step the Zoom in");
        assert_eq!(
            view.pan,
            Vec2::new(-124.0, -124.0),
            "the Zoom did not Pan the least distance that shows the Cursor: {:?}",
            view.pan
        );
    }

    ///
    /// A Pan with no Cursor move and no Zoom is not pulled back to the
    /// Cursor, even while the Cursor sits outside the Source View.
    ///
    #[tokio::test]
    async fn a_pan_with_no_cursor_move_and_no_zoom_is_not_pulled_back_to_the_cursor() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        orcvs.select(orcvs.grid().position(30, 30).expect("inside the grid"));

        // First frame: no previous Cursor to compare against, so no follow.
        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.pan, Vec2::ZERO);

        // A wheel Pan all the way to the far edge, with the Cursor still
        // unmoved at (30, 30) and no Zoom, repeated until the Pan settles
        // however far `smooth_scroll_delta` hands out a frame. Left to the
        // follow this would land at
        // (-328, -328) instead — see
        // `a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it`
        // — so landing on the margin's far edge at (-376, -376) is what
        // proves a Pan alone is not chasing the Cursor.
        let over = screen.min + Vec2::splat(50.0);
        wheel_until_settled(
            &ctx,
            screen,
            over,
            Vec2::new(-1_000.0, -1_000.0),
            &mut orcvs,
            &mut view,
        );

        assert_eq!(
            view.pan,
            Vec2::new(-376.0, -376.0),
            "a Pan alone was pulled back toward the Cursor: {:?}",
            view.pan
        );
    }

    ///
    /// The follow Pan is itself naive — it only asks whether the Cursor's
    /// Cell already shows inside the console — so an already out-of-bounds
    /// Pan it leaves untouched still has to settle back inside the Grid
    /// through `clamp_pan`, the same as an ordinary Pan or Zoom does.
    ///
    #[tokio::test]
    async fn the_follow_pan_is_still_bounded_by_the_grids_edges() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();

        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.pan, Vec2::ZERO, "the fixture did not open unpanned");

        // A Pan past the Grid's near edge, which nothing but `clamp_pan` can
        // answer: at Zoom 1.0 the Cursor's new Cell (5, 0) sits at 80..96,
        // already inside a 200 point console once this Pan is applied, so the
        // follow itself has nothing to add.
        pinned_at(&mut view, Vec2::new(50.0, 50.0), 1.0);
        orcvs.select(orcvs.grid().position(5, 0).expect("inside the grid"));
        console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);

        assert_eq!(
            view.pan,
            Vec2::ZERO,
            "clamp_pan did not settle the follow's own Pan back inside the Grid: {:?}",
            view.pan
        );
    }

    /// A device scale at which a Zoom step's Cell is not a whole number of
    /// physical pixels, and the Zoom step that shows it: 18 points at 1.25 is
    /// 22.5 pixels, which `presented_grid` floors to 22 — a Cell of 17.6
    /// points rather than the 18 a Zoom of 1.125 asks for.
    const FRACTIONAL_PPP: f32 = 1.25;
    const FRACTIONAL_ZOOM: f32 = 1.125;

    /// Half a physical pixel at [`FRACTIONAL_PPP`], the most the corner's own
    /// pixel rounding can move an edge.
    const HALF_A_PIXEL: f32 = 0.5 / FRACTIONAL_PPP;

    ///
    /// A Source smaller than the console starts at the console's top-left even
    /// where the snap has shrunk its Cells, rather than re-centred inside the
    /// unsnapped extent and drawn in from the corner.
    ///
    #[tokio::test]
    async fn a_snapped_grid_smaller_than_the_console_starts_at_its_top_left() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = running_orcvs(8, 8);
        let mut view = SourceView::default();
        pinned_at(&mut view, Vec2::ZERO, FRACTIONAL_ZOOM);

        let (viewport, _) = console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            FRACTIONAL_PPP,
        );

        assert!(
            viewport.cell_size < CELL_SIZE * FRACTIONAL_ZOOM,
            "the fixture snapped nothing, so it asserts nothing: {}",
            viewport.cell_size
        );
        assert_eq!(
            viewport.rect.min,
            screen.min + Vec2::splat(SOURCE_MARGIN_CELLS * viewport.cell_size),
            "a snapped Grid smaller than the console left its margin in from the top-left"
        );
    }

    ///
    /// A Pan to the far edge of a Source larger than the console leaves no
    /// gap past the Grid where the snap has shrunk its Cells: the bound is
    /// the extent the Cells are drawn at, not the one the Zoom asked for.
    ///
    #[tokio::test]
    async fn a_far_edge_pan_leaves_no_gap_past_a_snapped_grid() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = running_orcvs(64, 64);
        let mut view = SourceView::default();
        pinned_at(&mut view, Vec2::splat(-1_000_000.0), FRACTIONAL_ZOOM);

        let (viewport, _) = console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            FRACTIONAL_PPP,
        );

        assert!(
            viewport.cell_size < CELL_SIZE * FRACTIONAL_ZOOM,
            "the fixture snapped nothing, so it asserts nothing: {}",
            viewport.cell_size
        );
        let gap =
            screen.max - viewport.rect.max - Vec2::splat(SOURCE_MARGIN_CELLS * viewport.cell_size);
        assert!(
            gap.x.abs() <= HALF_A_PIXEL && gap.y.abs() <= HALF_A_PIXEL,
            "the far-edge Pan left {gap:?} past the margin between the Grid and the console's edge"
        );
    }

    ///
    /// A Cursor move past the console's far edge Pans the whole Cursor Cell,
    /// as drawn, into view — measured at the snapped Cell side the Cells are
    /// painted at rather than the unsnapped side the Zoom asked for.
    ///
    #[tokio::test]
    async fn a_cursor_follow_shows_the_whole_snapped_cursor_cell() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0));
        let mut orcvs = running_orcvs(64, 64);
        let mut view = SourceView::default();
        pinned_at(&mut view, Vec2::ZERO, FRACTIONAL_ZOOM);

        console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            FRACTIONAL_PPP,
        );
        // Column and row 22 end at 23 Cells, past a 400 point console at
        // either Cell side.
        orcvs.select(orcvs.grid().position(22, 22).expect("inside the grid"));
        let (viewport, _) = console_pass_at(
            &ctx,
            screen,
            Vec::new(),
            &mut orcvs,
            &mut view,
            FRACTIONAL_PPP,
        );

        let cell = viewport.cell_rect(22, 22);
        assert!(
            cell.min.x >= screen.min.x - HALF_A_PIXEL
                && cell.min.y >= screen.min.y - HALF_A_PIXEL
                && cell.max.x <= screen.max.x + HALF_A_PIXEL
                && cell.max.y <= screen.max.y + HALF_A_PIXEL,
            "the followed Cursor Cell {cell:?} is not wholly inside {screen:?}"
        );
    }

    ///
    /// The double click that returned to the fit is gone, along with the fit.
    /// A double click inside the Grid still selects the Cell under it.
    ///
    #[tokio::test]
    async fn a_double_click_selects_a_cell_and_does_not_reset_the_view() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        pinned_at(&mut view, Vec2::new(-48.0, -32.0), 1.0);

        let viewport = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        let before = view.to_global;
        let target = viewport.cell_rect(5, 5).center();
        double_click(&ctx, screen, target, &mut orcvs, &mut view);

        assert_eq!(
            selected_cell(&orcvs),
            (5, 5),
            "the double click did not reach the Cell under it"
        );
        assert_eq!(
            view.to_global, before,
            "the double click reset a view that no longer has a fit to return to"
        );
        assert_eq!(view.zoom, 1.0);
        assert_eq!(view.pan, Vec2::new(-48.0, -32.0));
    }

    ///
    /// A middle-button drag pans the Source by exactly what the pointer moved,
    /// at a scale that is not one, and a later click still selects the Cell
    /// under the pointer.
    ///
    /// `Response::drag_delta` divides by the layer transform's scaling *only
    /// when the layer has one* (`response.rs:452-465`). With the transform
    /// owned by the console there is no layer transform, so a leftover
    /// multiply by the Zoom would move the Source by the Zoom times the
    /// pointer. Zoom 2.0 is what makes that bug visible.
    ///
    #[tokio::test]
    async fn a_middle_drag_pans_by_the_pointer_and_a_later_click_selects_the_cell_under_it() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let mut orcvs = running_orcvs(32, 32);
        let mut view = SourceView::default();
        pinned_at(&mut view, Vec2::ZERO, 2.0);

        let before = console_frame(&ctx, screen, Vec::new(), &mut orcvs, &mut view);
        assert_eq!(view.to_global.scaling, 2.0);
        let anchor = view.to_global.translation;

        // Press, then move further than `max_click_dist` so the gesture
        // resolves as a drag rather than a click. Drag left and up: the Pan
        // is top-left-anchored, so a drag the other way is clamped at zero.
        let from = screen.min + Vec2::splat(40.0);
        let moved = Vec2::new(-40.0, -24.0);
        console_frame(&ctx, screen, middle_press_at(from), &mut orcvs, &mut view);
        let after = console_frame(
            &ctx,
            screen,
            vec![Event::PointerMoved(from + moved)],
            &mut orcvs,
            &mut view,
        );

        assert_eq!(
            view.to_global.translation - anchor,
            moved,
            "the Source panned by {:?} for a pointer that moved {moved:?}",
            view.to_global.translation - anchor
        );
        assert_eq!(
            after.rect.min - before.rect.min,
            moved,
            "the presented Grid moved by {:?}",
            after.rect.min - before.rect.min
        );
        assert_eq!(
            after.cell_size, before.cell_size,
            "the pan changed the Cell size"
        );

        console_frame(
            &ctx,
            screen,
            vec![Event::PointerButton {
                pos: from + moved,
                button: egui::PointerButton::Middle,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
            &mut orcvs,
            &mut view,
        );
        let target = after.rect.min + Vec2::new(3.5, 1.5) * after.cell_size;
        click(&ctx, screen, target, &mut orcvs, &mut view);

        assert_eq!(selected_cell(&orcvs), (3, 1), "a click at {target:?}");
    }

    ///
    /// Wheel and two-finger scroll Pan a Source larger than the console as far
    /// as its edges and no further. A Source that already fits does not Pan.
    ///
    #[tokio::test]
    async fn wheel_pans_a_larger_source_to_its_edges_and_a_smaller_one_nowhere() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, WIDE);
        let over = screen.min + Vec2::splat(16.0);

        let mut large = running_orcvs(32, 32);
        let mut view = SourceView::default();
        console_frame(&ctx, screen, Vec::new(), &mut large, &mut view);
        wheel_until_settled(
            &ctx,
            screen,
            over,
            Vec2::new(-1_000.0, -1_000.0),
            &mut large,
            &mut view,
        );
        assert_eq!(
            view.pan,
            Vec2::new(
                WIDE.x - CELL_SIZE * 32.0 - 2.0 * MARGIN,
                WIDE.y - CELL_SIZE * 32.0 - 2.0 * MARGIN
            ),
            "the wheel left the Source short of its edge: {:?}",
            view.pan
        );

        let mut small = running_orcvs(8, 8);
        let mut small_view = SourceView::default();
        console_frame(&ctx, screen, Vec::new(), &mut small, &mut small_view);
        console_frame(
            &ctx,
            screen,
            wheel_at(over, Vec2::new(-80.0, -40.0)),
            &mut small,
            &mut small_view,
        );
        assert_eq!(
            small_view.pan,
            Vec2::ZERO,
            "a Source smaller than the console panned"
        );
    }

    ///
    /// **The acceptance criterion the whole effort exists for.**
    ///
    /// No layer the Source Grid is painted into carries a transform, so no
    /// `TextShape` in it reaches `Arc::make_mut`. `GraphicLayers::drain`
    /// applies a layer's transform to every shape in it at end of pass, and for
    /// a `TextShape` that is `TextShape::transform`, which reaches the galley
    /// through `Arc::make_mut` and each of its rows through `Arc::make_mut`
    /// again. epaint's `GalleyCache` holds an `Arc` to every galley it hands
    /// out, so the refcount is never one and the clone is never elided. Given a
    /// transform the clone is unconditional, so the *absence* of the transform
    /// is the whole proof — there is nothing else to observe, and a profile
    /// would not show it anyway, because the clone happens inside `end_pass`
    /// rather than in `tessellate_shapes`.
    ///
    /// Zoom is pinned at two rather than left at one on purpose:
    /// `Context::set_transform_layer` *removes* the entry for an identity
    /// transform, so Zoom 1.0 would let a Scene pass this.
    ///
    #[tokio::test]
    async fn no_layer_carrying_the_source_grid_is_transformed() {
        let ctx = egui::Context::default();
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(256.0));
        let mut orcvs = running_orcvs(8, 8);
        orcvs.write("1");
        let mut view = SourceView::default();
        pinned_at(&mut view, Vec2::ZERO, 2.0);
        let frame = orcvs.render_frame();
        let mut grid_layer = None;

        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |root| {
                egui::CentralPanel::default()
                    .frame(source_panel_frame(
                        crate::source_paint::SourcePaintSettings::default().source_background(),
                    ))
                    .show(root, |ui| {
                        grid_layer = Some(ui.layer_id());
                        show_source_scene(
                            ui,
                            &frame,
                            &egui::FontFamily::Monospace,
                            &mut view,
                            crate::cursor_effects::CursorEffectSample::default(),
                            crate::cursor_effects::CursorEffectSettings::default(),
                            crate::source_paint::SourcePaintSettings::default(),
                        );
                    });
            },
        );
        let mut painted = Vec::new();
        for clipped in &output.shapes {
            flatten(clipped.shape.clone(), &mut painted);
        }
        output.drop_without_applying_deltas();

        assert_eq!(
            view.to_global.scaling, 2.0,
            "the console did not present at Zoom 2.0"
        );
        assert!(
            painted.iter().any(|shape| matches!(shape, Shape::Text(_))),
            "the pass painted no Glyph, so it proves nothing about galleys"
        );

        let grid_layer = grid_layer.expect("the central panel showed the Source");
        assert_eq!(
            ctx.layer_transform_to_global(grid_layer),
            None,
            "the Source Grid's own layer carries a transform"
        );
        // Not just that layer: no layer at all. The Scene painted into a
        // sublayer of its own, so checking only the layer the console paints
        // into would miss the transform that used to exist.
        assert!(
            ctx.memory(|memory| memory.to_global.is_empty()),
            "a layer transform survived: {:?}",
            ctx.memory(|memory| memory.to_global.clone())
        );
    }
}

///
/// The console's own end of the storage seam: the two lines that wire the
/// running Console to `console::persistence`. Every other persistence test drives
/// the persistence interface directly and would still pass with the
/// Console unwired, so these construct a real `Console` and call the real
/// `eframe::App::save`.
///
#[cfg(all(test, feature = "persistence"))]
mod storage_tests {
    use eframe::App as _;
    use orcvs::source::SourceCommander;

    use super::Console;
    use crate::persistence::{
        InMemoryStorage, REFUSED_KEY, SOURCE_KEY, edited_source, starting_source, store,
    };
    #[cfg(not(target_arch = "wasm32"))]
    use crate::persistence::{IsolatedRonDir, RonFileStorage};

    ///
    /// Storage holding a value no build can read back, and that value, so a
    /// test can assert on the thing that was refused.
    ///
    fn storage_holding_a_refused_value() -> (InMemoryStorage, String) {
        let mut written = InMemoryStorage::default();
        store(&mut written, &edited_source());
        let refused = eframe::Storage::get_string(&written, SOURCE_KEY)
            .expect("the save call stored the revision")
            .replace("cols:6", "cols:7");

        let mut storage = InMemoryStorage::default();
        eframe::Storage::set_string(&mut storage, SOURCE_KEY, refused.clone());
        (storage, refused)
    }

    ///
    /// A Console started the way eframe starts it, over `storage`.
    ///
    /// `_new_kittest` is eframe's own headless `CreationContext`, which is how
    /// an `App` is constructed outside a window; it opens with no storage, and
    /// the field is public precisely so a test can supply one.
    ///
    fn console_over(storage: &dyn eframe::Storage) -> Console {
        let mut cc = eframe::CreationContext::_new_kittest(egui::Context::default());
        cc.storage = Some(storage);
        Console::new(&cc).expect("the test runtime")
    }

    ///
    /// A refused value is Cells a viewer may still recover by hand, and the
    /// console's own save is what would otherwise destroy them: eframe calls it
    /// every thirty seconds and it writes the key the refused value sits under.
    ///
    #[tokio::test]
    async fn a_refused_value_is_preserved_before_the_next_save_overwrites_it() {
        let (mut storage, refused) = storage_holding_a_refused_value();

        let mut console = console_over(&storage);
        console.save(&mut storage);

        assert_eq!(
            eframe::Storage::get_string(&storage, REFUSED_KEY).as_deref(),
            Some(refused.as_str()),
            "the refused value was not preserved"
        );
        // The save still happened: a console that stopped saving after a
        // refusal would lose the session that followed it instead.
        assert!(
            eframe::Storage::get_string(&storage, SOURCE_KEY).is_some(),
            "the console stopped saving after a refusal"
        );
    }

    ///
    /// A viewer looking at a Grid that is not theirs is told so by the running
    /// Console, and stays told after the save that moves the refused value
    /// aside. `persistence.rs` proves the obligations end independently; this
    /// proves the Console is wired to them at all, which is the one thing an
    /// interface-level test cannot see.
    ///
    #[tokio::test]
    async fn a_refused_start_raises_a_console_notice_that_outlives_the_save() {
        let (mut storage, _) = storage_holding_a_refused_value();

        let mut console = console_over(&storage);
        assert!(
            console.persistence.notice_visible(),
            "a refused start told the viewer nothing"
        );

        console.save(&mut storage);

        assert!(
            console.persistence.notice_visible(),
            "the notice went with the value the save moved aside"
        );
    }

    ///
    /// A start with nothing wrong raises nothing. A notice a viewer sees on an
    /// ordinary start is a notice they learn to ignore.
    ///
    #[tokio::test]
    async fn an_absent_or_restored_start_raises_no_console_notice() {
        let mut restored = InMemoryStorage::default();
        store(&mut restored, &edited_source());

        for storage in [InMemoryStorage::default(), restored] {
            assert!(!console_over(&storage).persistence.notice_visible());
        }
    }

    #[tokio::test]
    async fn a_console_starts_the_revision_its_creation_storage_holds() {
        let saved = edited_source();
        let mut storage = InMemoryStorage::default();
        store(&mut storage, &saved);

        let console = console_over(&storage);

        assert_eq!(
            console.orcvs.source().snapshot(),
            saved.snapshot(),
            "the Console did not start the revision storage held"
        );
        assert_eq!(console.orcvs.source().grid().count(), 18);
    }

    ///
    /// `Console::load_function_reference` — what `File → Load Function
    /// reference` calls — replaces the whole running Orcvs, Grid included,
    /// rather than only clearing the Cells of the one it already had.
    ///
    /// `kittest_tests::the_file_menu_loads_the_function_reference_on_demand`
    /// proves the menu item reaches this call; this proves what the call
    /// itself does, against a starting revision on a Grid the reference does
    /// not share.
    ///
    #[tokio::test]
    async fn loading_the_function_reference_replaces_the_source_and_its_grid() {
        let mut storage = InMemoryStorage::default();
        store(&mut storage, &edited_source());
        let mut console = console_over(&storage);
        assert_eq!(
            console.orcvs.source().grid().count(),
            18,
            "the console did not start the 6x3 revision storage held"
        );

        console.load_function_reference();

        let reference = crate::function_reference::function_reference();
        assert_eq!(
            console.orcvs.source().snapshot(),
            reference.snapshot(),
            "loading the Function reference did not replace the running Source"
        );
        assert_eq!(
            console.orcvs.render_frame().grid().columns(),
            reference.grid().columns()
        );
        assert_eq!(
            console.orcvs.render_frame().grid().rows(),
            reference.grid().rows()
        );

        // With persistence on, the loaded reference then saves like any other
        // Source.
        let mut saved = InMemoryStorage::default();
        console.save(&mut saved);
        assert_eq!(
            starting_source(Some(&saved)).source.snapshot(),
            reference.snapshot(),
            "the loaded reference was not saved like any other Source"
        );
    }

    #[tokio::test]
    async fn the_console_save_call_stores_the_current_revision() {
        let mut restored_from = InMemoryStorage::default();
        store(&mut restored_from, &edited_source());
        let mut console = console_over(&restored_from);
        let mut storage = InMemoryStorage::default();

        console.save(&mut storage);

        // A Console that never saves leaves storage empty, and the start that
        // reads it opens the ordinary default Grid instead of this revision.
        let next_start = SourceCommander::with_source(starting_source(Some(&storage)).source);
        assert_eq!(
            next_start.snapshot(),
            edited_source().snapshot(),
            "the next start did not open the revision the Console saved"
        );
    }

    ///
    /// Native FileStorage is crate-private, so this drives the same RON kv file
    /// the binary writes to `eframe::storage_dir("Orcvs")/app.ron`. Isolate
    /// under a unique temp directory; never write Application Support.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn a_console_save_restarts_from_the_native_ron_file() {
        let dir = IsolatedRonDir::new();
        let saved = edited_source();
        {
            // A Console that already holds the 6×3 revision: FileStorage
            // cannot be constructed, so InMemoryStorage is only how this
            // session is primed. The write under test is `App::save` into the
            // RON file, then `flush`, which is what eframe does after save.
            let mut primed = InMemoryStorage::default();
            store(&mut primed, &saved);
            let mut console = console_over(&primed);
            let mut file = RonFileStorage::create(dir.path());
            console.save(&mut file);
            eframe::Storage::flush(&mut file);
        }

        let storage = RonFileStorage::from_file(dir.path());
        let restored = starting_source(Some(&storage)).source;
        assert_eq!(restored.snapshot(), saved.snapshot());
        assert_eq!(restored.grid().count(), 18);
        assert!(restored.grid().position(5, 2).is_some());
        assert!(restored.grid().position(6, 2).is_none());

        let console = console_over(&storage);
        assert_eq!(
            console.orcvs.source().snapshot(),
            saved.snapshot(),
            "Console::new did not restore the revision the native RON file held"
        );
    }
}
