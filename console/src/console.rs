use std::time::Duration;

use egui::{
    Color32, CornerRadius, CursorIcon, Event, EventFilter, FontId, Key, PointerButton, Pos2, Rect,
    Sense, Shape, Stroke, StrokeKind, Vec2, emath::GuiRounding as _, emath::TSTransform,
    epaint::RectShape,
};

mod glyphs;

use self::glyphs::{GLYPH_SCALE_STEP, GlyphTable, glyph_scale};
use crate::config::Config;
use crate::cursor_effects::{
    CursorEffectAnimation, CursorEffectSample, CursorEffectSettings, cursor_effect_shapes,
    effect_bounds,
};
use crate::function_reference::function_reference;
use crate::grid_viewport::{CELL_SIZE, GridViewport, presented_grid, snapped_cell_side};
use crate::midi::{MidiDeviceSelection, destination_presentation};
use crate::native_midi::{self, NativeMidiBackend};
use crate::paint::{FramePaint, Paint};
use crate::persistence::{default_source, starting_source};
use crate::readout_deadline::until_next;
use crate::theme::{Appearance, Theme};
use crate::theme_registry::ThemeRegistry;
use crate::theme_selection::SelectedThemes;
use orcvs::{
    app::{Arrow, InputEvent, InputKey, Orcvs},
    grid::{Grid, Position},
    opts::{Bpm, DEFAULT_FONT_SIZE},
    playback::{PlaybackStartError, PlaybackState},
    render_frame::RenderFrame,
    source::Source,
};

const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 2.0;
///
/// How far past the console's edge, in points, a Region drag's pointer has to
/// be for each point the Source View scrolls a frame after it. A pointer four
/// Cells out scrolls one Cell a frame, which is the most a frame scrolls.
///
const EDGE_SCROLL_REACH: f32 = 4.0;

/// The height the top panel takes from the window, leaving the rest to the
/// console. It is the panel's own minimum, which the menu bar does not exceed.
const TOP_PANEL_HEIGHT: f32 = 32.0;

/// The horizontal gap between the top menu bar's own items — its menu
/// buttons and the persistence notice.
const MENU_BAR_GAP: f32 = 16.0;

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
/// How many Cells the default window (ADR 0047) shows at Zoom 1.0, margin
/// included. The Grid (ADR 0054) is larger, so the rest of it is a Pan away.
///
const DEFAULT_VIEW_COLUMNS: usize = 64;
const DEFAULT_VIEW_ROWS: usize = 40;

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
mod run_clock_tests;

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
        // into `FocusDirection::Next`/`Previous` (`egui-0.36.2/src/memory/mod.rs:596-597`),
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

impl ZoomCommand {
    const ALL: [Self; 3] = [Self::In, Self::Out, Self::Reset];

    ///
    /// The key that, with a command modifier, is this command's chord: what
    /// [`zoom_command`] answers and the View menu shows. `+` is also Zoom In,
    /// since it shares `=`'s key on most layouts.
    ///
    fn key(self) -> Key {
        match self {
            Self::In => Key::Equals,
            Self::Out => Key::Minus,
            Self::Reset => Key::Num0,
        }
    }

    /// The chord as the View menu shows it.
    fn shortcut(self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, self.key())
    }

    /// The View menu item's label.
    fn label(self) -> &'static str {
        match self {
            Self::In => "Zoom In",
            Self::Out => "Zoom Out",
            Self::Reset => "Reset Zoom",
        }
    }
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
            Key::Plus => Some(ZoomCommand::In),
            key => ZoomCommand::ALL
                .into_iter()
                .find(|command| command.key() == *key),
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

///
/// A File menu command, chosen from the menu or, on native, by its chord.
///
/// The web binds no chord, since the browser reserves ⌘N and ⌘Q, and offers
/// New alone.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileCommand {
    New,
    #[cfg(not(target_arch = "wasm32"))]
    Open,
    #[cfg(not(target_arch = "wasm32"))]
    Save,
    #[cfg(not(target_arch = "wasm32"))]
    SaveAs,
}

impl FileCommand {
    /// Every command a chord reaches.
    #[cfg(not(target_arch = "wasm32"))]
    const CHORDED: [Self; 4] = [Self::New, Self::Open, Self::Save, Self::SaveAs];

    /// The File menu item's label.
    fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            #[cfg(not(target_arch = "wasm32"))]
            Self::Open => "Open…",
            #[cfg(not(target_arch = "wasm32"))]
            Self::Save => "Save",
            #[cfg(not(target_arch = "wasm32"))]
            Self::SaveAs => "Save As…",
        }
    }

    ///
    /// The chord that runs this command and that its File menu item shows:
    /// what [`file_command`] answers.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn shortcut(self) -> egui::KeyboardShortcut {
        let (modifiers, key) = match self {
            Self::New => (egui::Modifiers::COMMAND, Key::N),
            Self::Open => (egui::Modifiers::COMMAND, Key::O),
            Self::Save => (egui::Modifiers::COMMAND, Key::S),
            Self::SaveAs => (egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, Key::S),
        };
        egui::KeyboardShortcut::new(modifiers, key)
    }
}

///
/// The File command a toolkit event's chord asks for, or none.
///
/// Matches the command modifier with exactly the shortcut's Shift and no Alt,
/// so ⌘⇧S is Save As and ⌘⇧N is nothing. Key repeats run nothing.
///
#[cfg(not(target_arch = "wasm32"))]
fn file_command(event: &Event) -> Option<FileCommand> {
    match event {
        Event::Key {
            key,
            pressed: true,
            repeat: false,
            modifiers,
            ..
        } if modifiers.command && !modifiers.alt => {
            FileCommand::CHORDED.into_iter().find(|command| {
                let shortcut = command.shortcut();
                shortcut.logical_key == *key && shortcut.modifiers.shift == modifiers.shift
            })
        }
        _ => None,
    }
}

///
/// A File menu item for `command`, carrying its chord as shortcut text on
/// native, where the chord is bound. Answers whether it was chosen.
///
fn file_menu_item(ui: &mut egui::Ui, command: FileCommand) -> bool {
    let item = egui::Button::new(command.label());
    #[cfg(not(target_arch = "wasm32"))]
    let item = item.shortcut_text(ui.ctx().format_shortcut(&command.shortcut()));
    ui.add(item).clicked()
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
    /// A Zoom command the View menu asked for this frame, applied by the
    /// next [`show_source_scene`] exactly as the chord it names would be.
    requested_zoom: Option<ZoomCommand>,
    to_global: TSTransform,
}

impl Default for SourceView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            previous_cursor: None,
            region_drag: None,
            requested_zoom: None,
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
/// (`egui-0.36.2/src/containers/scene.rs:151-152, 168-173`) and nothing
/// replaces that once the container is gone. `grid_viewport` answers a Cell
/// size of zero for a console with no area, so the fit it yields has a scaling
/// of zero, and `TSTransform::inverse` divides by the scaling — which
/// `Scene::register_pan_and_zoom` does on every frame the pointer is over the
/// console. An unguarded zero therefore resolves every pointer position to NaN.
///
/// `TSTransform::is_valid` is not enough on its own: it checks only
/// `translation.x` (`emath-0.36.2/src/ts_transform.rs:55-57`) and admits a
/// negative scaling, which would present the Source mirrored.
///
fn is_presentable(to_global: TSTransform) -> bool {
    to_global.scaling.is_finite() && to_global.scaling > 0.0 && to_global.translation.is_finite()
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
    /// `Memory::focused().is_some()`, `egui-0.36.2/src/context.rs:2985-2988`),
    /// or any open popup — a menu, or the destination ComboBox's list — which
    /// a click opens without taking focus (`Popup::is_any_open`). Latched
    /// rather than asked where it is read, because
    /// `event_handler` runs before this frame's widgets are shown, and
    /// `Memory::begin_pass` has already let Escape clear the focus it was
    /// pressed to leave (`egui-0.36.2/src/memory/mod.rs:596-601`).
    keyboard_elsewhere: bool,
    /// The dark and light Theme selections, the Themes they choose from, and
    /// the Theme each presents. The same pair is installed as egui's dark and
    /// light styles, so whichever appearance egui presents a frame in, chrome
    /// and Source read the same Theme.
    themes: SelectedThemes,
    /// The `window.background` of the Theme the last frame was painted
    /// from: the backdrop the web runner clears to, since it asks after the
    /// frame, when a View menu change may already have switched the style.
    #[cfg(any(target_arch = "wasm32", test))]
    painted_backdrop: egui::Color32,
    /// The operating system's reduced-motion preference, read once at
    /// startup (`prefers_reduced_motion`) and combined with `cursor_effects`
    /// each frame through `CursorEffectSettings::respecting_reduced_motion`.
    reduced_motion: bool,
    /// Glitch amount and Glitch frequency: the Cursor Effect's motion
    /// settings. The Effect's colours are on the resolved Theme instead.
    cursor_effects: CursorEffectSettings,
    cursor_effect_animation: CursorEffectAnimation,
    #[cfg(feature = "persistence")]
    persistence: crate::persistence::Persistence,
    /// The question asked before discarding the Source, while it is showing.
    /// It holds the keys, as an open popup does.
    discard_confirmation: Option<DiscardConfirmation>,
    /// The open Source File, if any, and the Cells it last held: what the
    /// window title names and what "unsaved" compares against.
    #[cfg(not(target_arch = "wasm32"))]
    source_file: crate::source_file::OpenSourceFile,
    /// Why an Open or a Save failed, shown under the top bar's Notices until
    /// the viewer dismisses them.
    #[cfg(not(target_arch = "wasm32"))]
    file_notices: Vec<String>,
    /// The window title last sent, so it is sent again only when it changes.
    #[cfg(not(target_arch = "wasm32"))]
    shown_title: Option<String>,
    /// Whether the viewer has chosen to quit, so the close that follows is
    /// let through rather than asked about again.
    #[cfg(not(target_arch = "wasm32"))]
    closing: bool,
    /// The context an Open wakes the Panel through when the new Orcvs's
    /// Playback Engine publishes.
    ctx: egui::Context,
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
    pub fn start(cc: &eframe::CreationContext<'_>) -> Result<Self, PlaybackStartError> {
        // Custom Themes load here, during construction and never inside a
        // frame: native reads `~/.orcvs/themes/`, the web restores its
        // imported documents. This is the only place the shipped console
        // discovers Themes; `new` takes the registry, so a test builds the
        // one it needs and never reads the machine's own Theme directory.
        // Settings are read here for the same reason: native reads
        // `~/.orcvs/config.toml` and the web runs on the defaults.
        Self::new(cc, ThemeRegistry::start(cc.storage), Config::start())
    }

    ///
    /// The console over the running Orcvs its storage last held, choosing
    /// among the Themes `registry` holds under the settings `config` holds.
    /// [`Console::start`] is the shipped entry point, and discovers
    /// `registry` and reads `config` itself.
    ///
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        mut registry: ThemeRegistry,
        config: Config,
    ) -> Result<Self, PlaybackStartError> {
        // The stored Source revision, or an empty Source when storage holds none.
        let start = starting_source(cc.storage);

        // What the settings file could not supply reaches the viewer through
        // the Theme notice channel, as a Theme file's problem does.
        for notice in config.notices {
            registry.notice(notice);
        }
        // Resolved once so `install` and `Self::themes` hold the same pair. A
        // selection the registry cannot supply falls back to its appearance's
        // built-in and raises a notice.
        let themes = SelectedThemes::new(registry, config.theme_selection);

        // eframe restores egui memory — `ThemePreference` included — before
        // calling this constructor, but never reinstalls a style. Register
        // each appearance's Theme in its egui slot and leave the restored
        // (or default `System`) preference alone: `install` never calls
        // `set_theme`. `.scratch/theming/issues/02-…` is the decision.
        themes.install(&cc.egui_ctx);

        // egui's own `Context::end_pass` answers the same command `=`/`+`,
        // `-` and `0` chords by changing `zoom_factor` — the whole UI's
        // scale, not the Source View's (`egui-0.36.2/src/gui_zoom.rs`,
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

        // A restored Source is Untitled: storage keeps the Source, not its
        // file, so it is unsaved against the empty Source.
        #[cfg(not(target_arch = "wasm32"))]
        let source_file = crate::source_file::OpenSourceFile::untitled(default_source().snapshot());
        let (orcvs, midi) = environment(&cc.egui_ctx, start.source)?;
        Ok(Self {
            orcvs,
            midi,
            font_family: FontId::monospace(DEFAULT_FONT_SIZE).family,
            source_view: SourceView::default(),
            diagnostics_open: false,
            #[cfg(test)]
            bpm_widget_id: egui::Id::new(BPM_FIELD_ID),
            keyboard_elsewhere: false,
            #[cfg(any(target_arch = "wasm32", test))]
            painted_backdrop: themes
                .presented(Appearance::from(cc.egui_ctx.theme()))
                .window_background,
            themes,
            reduced_motion: prefers_reduced_motion(),
            cursor_effects: config.cursor_effects,
            cursor_effect_animation: CursorEffectAnimation::default(),
            #[cfg(feature = "persistence")]
            persistence: start.persistence,
            discard_confirmation: None,
            #[cfg(not(target_arch = "wasm32"))]
            source_file,
            #[cfg(not(target_arch = "wasm32"))]
            file_notices: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            shown_title: None,
            #[cfg(not(target_arch = "wasm32"))]
            closing: false,
            ctx: cc.egui_ctx.clone(),
        })
    }

    ///
    /// Replaces the environment with `source`: the running Orcvs (Source,
    /// Grid, Cursor, Region, Playback, opts), MIDI device selection, and the
    /// Source View. Settings — Theme, Cursor effects, Diagnostics — stand.
    ///
    /// On native the opened Source is Untitled and saved; a caller opening a
    /// file names it afterwards. A Source whose Orcvs cannot start is reported
    /// and not opened. Answers whether the Source was opened.
    ///
    fn open(&mut self, source: Source) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        let saved = source.snapshot();
        match environment(&self.ctx, source) {
            Ok((orcvs, midi)) => {
                self.orcvs = orcvs;
                self.midi = midi;
                self.source_view = SourceView::default();
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.source_file = crate::source_file::OpenSourceFile::untitled(saved);
                }
                true
            }
            Err(error) => {
                crate::report::error!("failed to open a Source: {error}");
                false
            }
        }
    }

    ///
    /// Opens the Function reference: `Help → Function Reference`.
    ///
    fn load_function_reference(&mut self) {
        if !self.open(function_reference()) {
            self.start_notice("The Function Reference");
        }
    }

    ///
    /// Opens an empty Source on the one Grid (ADR 0054): `File → New`.
    ///
    fn new_source(&mut self) {
        if !self.open(default_source()) {
            self.start_notice("An empty Source");
        }
    }

    ///
    /// Raises a notice that `what` was not opened because its Source could not
    /// start. The web has no notice for it; `open` reported why to the
    /// developer console on both.
    ///
    fn start_notice(&mut self, what: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        self.file_notice(format!(
            "{what} could not be opened: its Source could not start"
        ));
        #[cfg(target_arch = "wasm32")]
        let _ = what;
    }

    ///
    /// Whether discarding the running Source would lose anything, so an
    /// action that discards it asks first.
    ///
    /// On native, whether the Source has unsaved changes. The web tracks no
    /// file, and asks whether any Cell is written.
    ///
    fn asks_before_discarding(&mut self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.source_file.unsaved(self.orcvs.source())
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.source_is_written()
        }
    }

    ///
    /// Opens the Source File at `path` as the environment, as the open file
    /// and saved. A file that cannot be read or parsed opens nothing and
    /// raises a notice.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn open_path(&mut self, path: &std::path::Path) {
        match crate::source_file::read_source_file(path) {
            Ok(source) => {
                if self.open(source) {
                    self.source_file.name(path.to_path_buf());
                } else {
                    self.start_notice(&path.display().to_string());
                }
            }
            Err(problem) => self.file_notice(problem),
        }
    }

    ///
    /// Opens the Source File the viewer picked, or nothing when the dialog
    /// was cancelled.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn open_picked(&mut self, picked: Option<std::path::PathBuf>) {
        if let Some(path) = picked {
            self.open_path(&path);
        }
    }

    ///
    /// `File → Open…`: the native dialog, then the file it picked.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn choose_and_open(&mut self) {
        let dialog = rfd::FileDialog::new().set_title("Open a Source File");
        // rfd's macOS panel merges every filter into one list of allowed
        // extensions, where `*` is no wildcard, so any filter there would
        // refuse a Source File under another extension.
        #[cfg(not(target_os = "macos"))]
        let dialog = dialog
            .add_filter("Orcvs Source File", &[crate::source_file::EXTENSION])
            .add_filter("All files", &["*"]);
        let picked = dialog.pick_file();
        self.open_picked(picked);
    }

    ///
    /// `File → Save`: writes the open file, or runs Save As when none is open.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn save_file(&mut self) {
        match self.source_file.path() {
            Some(path) => self.save_to(path.to_path_buf()),
            None => self.save_file_as(),
        }
    }

    ///
    /// `File → Save As…`: the native save dialog, offering the open file's
    /// name (or `Untitled.orcvs`) in its folder, then the path it picked.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn save_file_as(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .set_title("Save the Source File")
            .add_filter("Orcvs Source File", &[crate::source_file::EXTENSION]);
        let path = self.source_file.path();
        if let Some(folder) = path.and_then(std::path::Path::parent) {
            dialog = dialog.set_directory(folder);
        }
        let name = path.and_then(std::path::Path::file_name).map_or_else(
            || format!("Untitled.{}", crate::source_file::EXTENSION),
            |name| name.to_string_lossy().into_owned(),
        );
        let picked = dialog.set_file_name(name).save_file();
        self.save_picked(picked);
    }

    ///
    /// Saves to the path the viewer picked — given the Source File extension
    /// when it has none — or nothing when the dialog was cancelled.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn save_picked(&mut self, picked: Option<std::path::PathBuf>) {
        let Some(picked) = picked else {
            return;
        };
        let path = crate::source_file::with_source_file_extension(picked.clone());
        // The dialog confirmed replacing `picked`, not `path`, so an existing
        // file at `path` is refused rather than replaced unasked.
        if path != picked && path.exists() {
            self.file_notice(format!(
                "{} was not saved: {} already exists; choose Save As… and pick it to replace it",
                picked.display(),
                path.display()
            ));
            return;
        }
        self.save_to(path);
    }

    ///
    /// Writes the Source to `path` beside it and renames it over, so a failure
    /// never truncates it. Success makes `path` the open, saved file; failure
    /// raises a notice.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn save_to(&mut self, path: std::path::PathBuf) {
        // One lock for the text and the snapshot, so a Tick cannot land between.
        let mut written = (String::new(), String::new());
        self.orcvs.source().read_source(|source| {
            written = (orcvs::source::file::write(source), source.snapshot());
        });
        let (text, saved) = written;
        match crate::source_file::write_beside_then_rename(&path, &text) {
            Ok(()) => self.source_file.saved(path, saved),
            Err(error) => {
                self.file_notice(format!("{} could not be saved: {error}", path.display()));
            }
        }
    }

    ///
    /// Reports and raises a notice about a Source File.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn file_notice(&mut self, message: String) {
        crate::report::error!("{message}");
        self.file_notices.push(message);
    }

    ///
    /// Closes the window without asking again.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn quit(&mut self) {
        self.closing = true;
        self.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    ///
    /// Runs `command` for the File menu and the chords alike, asking first
    /// where it would discard unsaved changes.
    ///
    fn run_file_command(&mut self, command: FileCommand) {
        match command {
            FileCommand::New => {
                self.discard_asking_first(NEW_CONFIRMATION);
            }
            #[cfg(not(target_arch = "wasm32"))]
            FileCommand::Open => {
                self.discard_asking_first(OPEN_CONFIRMATION);
            }
            // Saving discards nothing, so neither asks.
            #[cfg(not(target_arch = "wasm32"))]
            FileCommand::Save => self.save_file(),
            #[cfg(not(target_arch = "wasm32"))]
            FileCommand::SaveAs => self.save_file_as(),
        }
    }

    ///
    /// Cancels a close request while there are unsaved changes and asks the
    /// Quit question instead. Every close — the window's button, `File → Quit`
    /// or egui's quit shortcut — is asked about here.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn guard_close(&mut self, ctx: &egui::Context) {
        if ctx.input(|input| input.viewport().close_requested())
            && !self.closing
            && self.asks_before_discarding()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.discard_confirmation = Some(QUIT_CONFIRMATION);
        }
    }

    ///
    /// Sends the window title — the open file's name or `Untitled`, marked
    /// while unsaved — when it changes.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    fn show_title(&mut self, ctx: &egui::Context) {
        let unsaved = self.source_file.unsaved(self.orcvs.source());
        let title = self.source_file.title(unsaved);
        if self.shown_title.as_ref() != Some(&title) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.shown_title = Some(title);
        }
    }

    ///
    /// Whether any Cell of the running Source is written.
    ///
    #[cfg(target_arch = "wasm32")]
    fn source_is_written(&self) -> bool {
        self.orcvs
            .render_frame()
            .cells()
            .iter()
            .any(|cell| cell.content().is_some())
    }

    ///
    /// Discards the Source through `confirmation.discard`, first asking
    /// `confirmation.question` when there is anything to lose
    /// ([`Console::asks_before_discarding`]). One question shows at a time.
    ///
    fn discard_asking_first(&mut self, confirmation: DiscardConfirmation) {
        if self.asks_before_discarding() {
            self.discard_confirmation = Some(confirmation);
        } else {
            (confirmation.discard)(self);
        }
    }

    ///
    /// Shows the discard confirmation as a modal over the whole console. It
    /// opens with Cancel focused; Escape and a click outside cancel, and only
    /// Discard runs the discarding action.
    ///
    fn show_discard_confirmation(&mut self, ctx: &egui::Context) {
        let Some(confirmation) = self.discard_confirmation.take() else {
            return;
        };
        let modal = egui::Modal::new(egui::Id::new(DISCARD_CONFIRMATION_ID)).show(ctx, |ui| {
            ui.set_max_width(DISCARD_CONFIRMATION_WIDTH);
            ui.label(confirmation.question);
            ui.add_space(MENU_BAR_GAP);
            ui.horizontal(|ui| {
                let cancel = ui.button("Cancel");
                if ui.memory(|memory| memory.focused().is_none()) {
                    cancel.request_focus();
                }
                let discard = ui.button("Discard");
                if discard.clicked() {
                    Some(true)
                } else if cancel.clicked() {
                    Some(false)
                } else {
                    None
                }
            })
            .inner
        });
        match modal.inner {
            Some(true) => (confirmation.discard)(self),
            Some(false) => {}
            None if modal.should_close() => {}
            None => self.discard_confirmation = Some(confirmation),
        }
    }
}

///
/// A question asked before an action that discards the running Source.
///
/// `discard` is the action itself, run only when the viewer confirms.
///
struct DiscardConfirmation {
    question: &'static str,
    discard: fn(&mut Console),
}

/// What `File → New` asks before discarding the Source.
const NEW_CONFIRMATION: DiscardConfirmation = DiscardConfirmation {
    #[cfg(not(target_arch = "wasm32"))]
    question: "The Source has unsaved changes. Discard them and open an empty Source?",
    #[cfg(target_arch = "wasm32")]
    question: "The Source holds written content. Discard it and open an empty Source?",
    discard: Console::new_source,
};

/// What `Help → Function Reference` asks, on native, before discarding the
/// Source.
#[cfg(not(target_arch = "wasm32"))]
const FUNCTION_REFERENCE_CONFIRMATION: DiscardConfirmation = DiscardConfirmation {
    question: "The Source has unsaved changes. Discard them and open the Function Reference?",
    discard: Console::load_function_reference,
};

/// What `File → Open…` asks before its dialog, while there is anything to
/// lose.
#[cfg(not(target_arch = "wasm32"))]
const OPEN_CONFIRMATION: DiscardConfirmation = DiscardConfirmation {
    question: "The Source has unsaved changes. Discard them and open a Source File?",
    discard: Console::choose_and_open,
};

/// What a close request — `File → Quit` or the window's close button — asks
/// before discarding the Source.
#[cfg(not(target_arch = "wasm32"))]
const QUIT_CONFIRMATION: DiscardConfirmation = DiscardConfirmation {
    question: "The Source has unsaved changes. Discard them and quit?",
    discard: Console::quit,
};

/// The discard confirmation's modal: one at a time, so one id.
const DISCARD_CONFIRMATION_ID: &str = "orcvs-discard-confirmation";
/// How wide the discard confirmation grows before its question wraps.
const DISCARD_CONFIRMATION_WIDTH: f32 = 360.0;

///
/// A running Orcvs over `source` that wakes the Panel when its Playback Engine
/// publishes, and MIDI device selection over its handle. Shared by
/// [`Console::new`] and [`Console::open`].
///
fn environment(
    ctx: &egui::Context,
    source: Source,
) -> Result<(Orcvs, MidiDeviceSelection), PlaybackStartError> {
    let orcvs = Orcvs::with_source(source)?;
    wake_panel_when_playback_publishes(ctx.clone(), orcvs.playback_observation_watch());
    let mut midi = MidiDeviceSelection::new(
        orcvs.midi_selection_handle(),
        Box::new(NativeMidiBackend::new()),
    );
    midi.refresh_destinations();
    Ok((orcvs, midi))
}

///
/// The top bar's Notices menu, when there are any: the Theme and settings
/// notices, then the Source File notices in `files`, with a Dismiss button.
///
fn show_notices(ui: &mut egui::Ui, themes: &mut SelectedThemes, files: &mut Vec<String>) {
    let count = themes.notice_count() + files.len();
    if count == 0 {
        return;
    }
    ui.add_space(MENU_BAR_GAP);
    let title =
        egui::RichText::new(format!("Notices ({count})")).color(ui.visuals().error_fg_color);
    ui.menu_button(title, |ui| {
        ui.set_max_width(THEME_NOTICE_WIDTH);
        egui::ScrollArea::vertical()
            .max_height(THEME_NOTICE_HEIGHT)
            .show(ui, |ui| {
                for notice in themes.notices().chain(files.iter()) {
                    ui.label(notice);
                }
            });
        if ui.button("Dismiss").clicked() {
            themes.dismiss_notices();
            files.clear();
            ui.close();
        }
    });
}

/// How wide the notices menu grows before its messages wrap.
const THEME_NOTICE_WIDTH: f32 = 480.0;
/// How tall the notices list grows before it scrolls.
const THEME_NOTICE_HEIGHT: f32 = 320.0;

fn frames_per_second(frame_time: f32) -> Option<f32> {
    frame_time.is_normal().then(|| frame_time.recip())
}

///
/// Paints the Panel from a published Tick. A wait started from this Render
/// Frame is a second clock; this asks for a paint when the engine publishes.
///
fn wake_panel_when_playback_publishes(
    ctx: egui::Context,
    observation: orcvs::playback::PlaybackObservationWatch,
) {
    let wake = panel_wake(ctx, observation);
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::spawn(wake);
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(wake);
    }
}

///
/// Requests a repaint of `ctx` each time `observation` publishes, and ends
/// once the Playback Engine that owns the observation's sender is gone. An
/// Open replaces the Orcvs, so each wake-up has to end with the Orcvs it
/// watches rather than outlive it.
///
/// The observation is marked seen before this returns, so only a publish
/// after the wake-up exists requests a repaint.
///
fn panel_wake(
    ctx: egui::Context,
    mut observation: orcvs::playback::PlaybackObservationWatch,
) -> impl std::future::Future<Output = ()> {
    let _ = observation.borrow_and_update();
    async move {
        while observation.changed().await.is_ok() {
            ctx.request_repaint();
        }
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
/// One run of consecutive Cells that share a background colour, as the single
/// rectangle that fills them all.
///
/// # Why the rectangle is snapped
///
/// Not to prevent a seam. epaint snaps the rectangle whether or not this does:
/// `RectShape::filled` leaves `round_to_pixels: None`,
/// `TessellationOptions::round_rects_to_pixels` defaults to true, and
/// `tessellate_rect` then applies `Rect::round_to_pixels` — the same `emath`
/// function this calls — at `epaint-0.36.2/src/tessellator.rs:1829-1862`. A run
/// left unsnapped here would reach the screen as the same pixels.
///
/// It is snapped so the rectangle is one a test can predict. The snap is the
/// last thing that moves an edge, so doing it here puts the Shape a Render
/// Frame carries at the coordinates the paint lands on, and an assertion can
/// state them exactly rather than within a pixel. `Rect::round_to_pixels`
/// rounds the two corners independently
/// (`emath-0.36.2/src/gui_rounding.rs:155-186`), so a run's far edge lands on
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
    /// Cell's, or the lasso around a Region larger than one Cell, and empty
    /// when a zero width hid it — or the selected Cell's border when no
    /// effect frame was built.
    cursor: Vec<Shape>,
}

impl SourceShapes {
    ///
    /// Draws a Paint at `viewport`: the geometry the value layer carries none
    /// of, applied to the colours and characters it carries all of.
    ///
    /// Stroke widths are fixed display points that stay the same visible
    /// thickness at every Grid zoom (`.scratch/theming/issues/06` slice C),
    /// unlike the [`GridViewport::cell_scale`]-multiplied constants this
    /// replaced: `sector.seam.width` from the resolved `theme`, and each
    /// Cell's own border width already resolved onto it as `cell.
    /// border_width` (`grid.border.width`, `cell.selection.border.width`, or
    /// either composited with Diagnostic/Output Portal, by fact priority).
    /// `pixels_per_point` is the device scale the background runs are
    /// snapped to; see [`background_run`].
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
        theme: &Theme,
    ) -> Self {
        let mut shapes = Self::geometry(paint, viewport, pixels_per_point, theme);
        shapes.area = cursor_effect.area;
        if let Some(frame) = cursor_effect.frame {
            shapes.cursor = frame;
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
    /// `theme.sector_seam_width` is read once, here, rather than once per
    /// Cell inside the loop below — `.scratch/theming/issues/06`'s "resolve
    /// widths once per frame" — and a width of exactly zero skips building
    /// that Shape outright rather than emitting a zero-width one for the
    /// painter to drop, per the same issue's "Width 0 hides the stroke." Each
    /// Cell's own border width has no one frame-level constant to read here:
    /// `cell.border_width` already carries the fact-priority pick
    /// `crate::style::cell_visuals_with_cursor_colour` and
    /// `crate::style::ordinary_border` resolved for it — `grid.border.width`
    /// for an ordinary Cell composited with Diagnostic/Output Portal,
    /// `cell.selection.border.width` for a single-Cell Cursor — so this step
    /// reads that answer per Cell rather than choosing among Theme fields
    /// itself.
    ///
    fn geometry(
        paint: &Paint,
        viewport: &GridViewport,
        pixels_per_point: f32,
        theme: &Theme,
    ) -> Self {
        // Border widths no longer read `theme` here: `cell.border_width` is
        // already the resolved, fact-priority-picked answer — see the loop
        // below. Sector Seam width has no per-Cell fact to vary by, so it
        // stays a plain frame-level read.
        let sector_seam_width = theme.sector_seam_width.points();
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
            //
            // `cell.border_width` is already the resolved answer —
            // `crate::style::cell_visuals_with_cursor_colour` and
            // `crate::style::ordinary_border`'s fact-priority pick, threaded
            // through `CellPaint` — so this step is purely mechanical: a
            // single-Cell Cursor's own `cell.selection.border.width`, or the
            // ordinary Grid border's width, itself composited with
            // Diagnostic/Output Portal by the same priority
            // (`.scratch/theming/schema.md`'s Source composition steps 4 and
            // 5). Each width independently hides its own stroke at zero,
            // since it is this Cell's *only* width, decided once above.
            if cell.border_width > 0.0 {
                let border = Shape::Rect(RectShape::stroke(
                    rect,
                    CornerRadius::ZERO,
                    Stroke::new(cell.border_width, cell.border),
                    StrokeKind::Inside,
                ));

                // The selected Cell's border is the Cursor, and the Cursor is
                // painted last. A Cursor the viewport does not reach is no
                // Cell of this Paint, so the comparison never matches and the
                // group stays empty.
                if paint.cursor() == Some(position) && !paint.region_spans() {
                    cursor.push(border);
                } else {
                    borders.push(border);
                }
            }

            // A seam is absent on the Cursor's Cell, while it is framed on its
            // own, because the derive suppressed it there, so this step never
            // learns that rule.
            if sector_seam_width > 0.0 {
                for (colour, ends) in [
                    (cell.sector_left, [rect.left_top(), rect.left_bottom()]),
                    (cell.sector_top, [rect.left_top(), rect.right_top()]),
                ] {
                    if let Some(colour) = colour {
                        seams.push(Shape::line_segment(
                            ends,
                            Stroke::new(sector_seam_width, colour),
                        ));
                    }
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
/// Eight parameters, one over clippy's default: `theme` is the eighth,
/// added by `syntax-highlighting/01` as `source_paint` and repurposed by
/// `.scratch/theming/issues/06`. Each of the eight is an independent,
/// already-tested value threaded straight through from `show_source_scene`'s
/// own parameters of the same names — geometry, a Render Frame, and the two
/// presentation values `Console::ui` owns — so grouping any of them into a
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
    theme: &Theme,
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
    let paint = Paint::derive_with_theme(FramePaint::new(frame, visible), theme);
    let cursor_rect = viewport.cell_rect(frame.cursor().x(), frame.cursor().y());
    // The Cursor's own frame outlines its own Cell; the lasso around a
    // Region larger than one Cell takes the Region's own outline colour
    // instead — `.scratch/theming/schema.md`'s "the effect outline uses
    // `cursor.border` or `region.border`". `paint` already answered which
    // this Render Frame is, so this reads that rather than re-deriving it.
    // The same choice picks the animated frame's nominal width, as one
    // `Stroke` so colour and width cannot come from different answers:
    // `cursor.border.width` for the Cursor's own frame, `region.border.width`
    // for the lasso — a fixed display-point value `cursor_effect_shapes` never
    // scales with Grid zoom (`.scratch/theming/issues/06` slice C).
    let frame_stroke = if paint.region_spans() {
        egui::Stroke::new(theme.region_border_width.points(), theme.region_border)
    } else {
        egui::Stroke::new(theme.cursor_border_width.points(), theme.cursor_border)
    };
    let cursor_effect = cursor_effect_shapes(
        cursor_rect,
        effect_outline(frame, &viewport),
        clip,
        viewport.cell_size,
        cursor_effect_sample,
        cursor_effect_settings,
        theme.cursor_area,
        frame_stroke,
    );
    let shapes = SourceShapes::new(
        &paint,
        &viewport,
        &table,
        pixels_per_point,
        cursor_effect,
        theme,
    );

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
    theme: &Theme,
) -> PresentedSource {
    let source_grid = frame.grid();
    let source = source_bounds(source_grid);
    // `Sense::CLICK | Sense::DRAG` rather than `Sense::click_and_drag()`,
    // which adds `FOCUSABLE` (`egui-0.36.2/src/sense.rs:81-83`) and would let
    // Tab focus the console area, where a focused widget keeps every key from
    // the Source.
    let (console, mut pan) =
        ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::CLICK | Sense::DRAG);

    if !view.zoom.is_finite() || view.zoom <= 0.0 {
        view.zoom = 1.0;
    }
    view.zoom = view.zoom.clamp(MIN_ZOOM, MAX_ZOOM);

    let zoom_before_command = view.zoom;
    // A View menu item and a chord are the same command: the menu's is
    // taken first, since a click that closed the menu carries no chord.
    let command = view
        .requested_zoom
        .take()
        .or_else(|| ui.input(|i| i.events.iter().find_map(zoom_command)));
    if let Some(command) = command {
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
        theme,
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
/// `background` is the resolved Theme's live `grid_background`, not a
/// constant: a loaded Theme has to repaint this panel on the very next frame
/// for the Cell it stands in for to still agree with it
/// (`.scratch/theming/issues/06`/`07`).
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

///
/// A viewer's change to the console's appearance, made with the top bar's
/// mode control. The dark and light Themes themselves are settings.
///
#[derive(Debug)]
enum AppearanceChange {
    /// Follow the operating system's appearance, or hold dark or light.
    Mode(egui::ThemePreference),
}

///
/// The mode control's glyphs: Follow the OS, Dark, Light. Each must be in the
/// console's one font (`kittest_tests::the_mode_glyphs_are_in_the_console_font`).
///
const MODE_GLYPHS: [&str; 3] = ["◐", "☾", "☼"];

///
/// The top bar's mode control: Follow the OS, Dark and Light as icon-only
/// selectable buttons, each named for accessibility and explained on hover.
/// Answers the choice a viewer made this frame, if any.
///
/// Shown into a right-to-left layout, so the buttons are added in reverse.
///
fn mode_control(ui: &mut egui::Ui) -> Option<AppearanceChange> {
    let mode = ui.ctx().options(|options| options.theme_preference);
    let system = ui.input(|input| input.raw.system_theme);
    let choices = [
        (
            egui::ThemePreference::System,
            "Follow the OS",
            "Follow the operating system's appearance.",
        ),
        (
            egui::ThemePreference::Dark,
            "Dark",
            "Always use the dark appearance.",
        ),
        (
            egui::ThemePreference::Light,
            "Light",
            "Always use the light appearance.",
        ),
    ];
    let mut change = None;
    for ((preference, name, explanation), glyph) in choices.into_iter().zip(MODE_GLYPHS).rev() {
        let selected = mode == preference;
        let response = ui.selectable_label(selected, glyph);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, true, selected, name)
        });
        let response = response.on_hover_ui(|ui| {
            ui.label(explanation);
            if preference == egui::ThemePreference::System {
                ui.label(match system {
                    Some(egui::Theme::Dark) => "The operating system's appearance is dark.",
                    Some(egui::Theme::Light) => "The operating system's appearance is light.",
                    None => "The operating system's appearance is unknown.",
                });
            }
        });
        if response.clicked() {
            change = Some(AppearanceChange::Mode(preference));
        }
    }
    change
}

impl Console {
    ///
    /// Applies an appearance change. A mode is egui's own `ThemePreference`,
    /// which egui memory holds and eframe persists.
    ///
    fn change_appearance(ctx: &egui::Context, change: AppearanceChange) {
        match change {
            AppearanceChange::Mode(preference) => ctx.set_theme(preference),
        }
        ctx.request_repaint();
    }
}

impl Console {
    ///
    /// The web runner's clear colour: the backdrop of the Theme the frame it
    /// clears under was painted from. eframe's web runner asks
    /// `clear_color` after the frame (`eframe-0.36.2/src/web/app_runner.rs`,
    /// `paint` after `logic`), so the active style can already be the one a
    /// View menu change in that frame switched to.
    ///
    #[cfg(any(target_arch = "wasm32", test))]
    fn web_clear_color(&self) -> [f32; 4] {
        self.painted_backdrop.to_normalized_gamma_f32()
    }
}

impl eframe::App for Console {
    ///
    /// Called by the framework to save state before shutdown, and at
    /// intervals while running. This stores the current Source revision;
    /// settings are never saved.
    ///
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.persistence.save(storage, self.orcvs.source());
    }

    ///
    /// The window's own clear colour: `.scratch/theming/schema.md`'s Chrome
    /// mapping table, "Application backdrop | `window.background`, opaque."
    ///
    /// This is the resolved Theme's answer to "confirm both [the window
    /// backdrop and the Grid background] are wired" (`.scratch/theming/
    /// issues/06` slice C) — the window backdrop's own consumer, wired
    /// independently of `source_panel_frame`'s `theme.grid_background`.
    /// Without this override `eframe::App::clear_color`'s own default
    /// (`Color32::from_rgba_unmultiplied(12, 12, 12, 180)`) shows through
    /// wherever a Theme's partly transparent panel, Grid or Cell layer
    /// reveals the console surface beneath it, rather than the Theme's own
    /// opaque backdrop — a translucent grey the Theme never chose, in place
    /// of the surface ADR 0053 and `.scratch/theming/schema.md` describe:
    /// "Transparency reveals the underlying console surface; the application
    /// window remains opaque."
    ///
    /// The native glow integration asks before it runs the frame, passing
    /// egui's active style, so the Theme is the one of the appearance
    /// `visuals` belongs to (`style::style` sets `dark_mode` from the Theme's
    /// declared appearance), which the frame is about to be styled from. The
    /// web runner asks after the frame, when a View menu change made in it
    /// has already switched the style, so it clears to the backdrop of the
    /// Theme that frame was painted from instead (`Console::web_clear_color`).
    ///
    /// Every built-in's `window_background` is opaque by construction, and a
    /// custom Theme's is refused at resolution if it is not
    /// (`theme::resolve`'s `ThemeError::NonOpaqueWindowBackground`,
    /// `.scratch/theming/schema.md`: "alpha other than 255 on this property
    /// is an error"), so nothing presented can hold a nonopaque value and
    /// this reads the field directly rather than re-validating it here.
    ///
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = visuals;
            self.web_clear_color()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let appearance = egui::Theme::from_dark_mode(visuals.dark_mode).into();
            let theme = self.themes.presented(appearance);
            theme.window_background.to_normalized_gamma_f32()
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, root: &mut egui::Ui, eframe: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        // The appearance egui presents this frame in, from the viewer's mode
        // and the operating system's appearance. egui built the root `Ui`
        // from that appearance's style before this ran, and nothing changes
        // it until the frame is done — a View menu change is held until
        // then — so the Source reads the Theme the chrome was styled from.
        let appearance = Appearance::from(ctx.theme());
        let mut appearance_change = None;
        // A File command chosen from the menu or by its chord, run once the
        // menu bar is done and the chords are read.
        let mut file_command_chosen = None;
        // Tab belongs to the Source while it holds the keys. `Memory::begin_pass`
        // already turned an unmodified Tab into `FocusDirection::Next` and a
        // Shift Tab into `FocusDirection::Previous` before this runs
        // (`egui-0.36.2/src/memory/mod.rs:596-597`), and the first focusable
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
                // New, Open…, Save, Save As… and Quit on native; New alone on
                // the web.
                ui.menu_button("File", |ui| {
                    if file_menu_item(ui, FileCommand::New) {
                        file_command_chosen = Some(FileCommand::New);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        for command in [FileCommand::Open, FileCommand::Save, FileCommand::SaveAs] {
                            if file_menu_item(ui, command) {
                                file_command_chosen = Some(command);
                            }
                        }
                        ui.separator();
                        // Quit is a close request, asked about in `guard_close`.
                        // It shows no chord: on macOS the app menu's ⌘Q ends the
                        // process without a close request.
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.add_space(MENU_BAR_GAP);
                ui.menu_button("View", |ui| {
                    for command in ZoomCommand::ALL {
                        let item = egui::Button::new(command.label())
                            .shortcut_text(ctx.format_shortcut(&command.shortcut()));
                        if ui.add(item).clicked() {
                            // Applied by this frame's `show_source_scene`,
                            // which runs after the menu bar, as the chord is.
                            self.source_view.requested_zoom = Some(command);
                        }
                    }
                    ui.separator();
                    ui.checkbox(&mut self.diagnostics_open, "Diagnostics");
                });
                ui.add_space(MENU_BAR_GAP);
                ui.menu_button("Help", |ui| {
                    if ui.button("Function Reference").clicked() {
                        // The web keeps its unasked Function reference; on
                        // native it discards unsaved changes as New does.
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            self.discard_asking_first(FUNCTION_REFERENCE_CONFIRMATION);
                        }
                        #[cfg(target_arch = "wasm32")]
                        self.load_function_reference();
                    }
                });
                // Notices are status, not menus: they sit at the bar's right
                // edge. Right to left, so what is shown first is rightmost.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Rightmost: the mode, which is a control rather than a
                    // menu, and applied once the frame is done.
                    appearance_change = mode_control(ui);
                    // Theme, settings, Source File and persistence notices sit
                    // in the bar until dismissed; `report` has already sent
                    // each to the developer console.
                    #[cfg(not(target_arch = "wasm32"))]
                    show_notices(ui, &mut self.themes, &mut self.file_notices);
                    // The web opens and saves no file.
                    #[cfg(target_arch = "wasm32")]
                    show_notices(ui, &mut self.themes, &mut Vec::new());
                    #[cfg(feature = "persistence")]
                    if self.persistence.notice_visible() {
                        ui.add_space(MENU_BAR_GAP);
                        if ui.button("Dismiss").clicked() {
                            self.persistence.dismiss_notice();
                        }
                        // Truncated to the space the menus leave, with its
                        // whole text on hover, so a narrow window never
                        // lays it over them.
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!(
                                    "Stored Source could not be read back; it was kept under \
                                     \"{}\"",
                                    crate::persistence::REFUSED_KEY
                                ))
                                .color(ui.visuals().error_fg_color),
                            )
                            .truncate(),
                        );
                    }
                });
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
        // (`egui-0.36.2/src/widgets/text_edit/builder.rs:1098`) — and names
        // `egui_wants_keyboard_input` as the question to ask instead
        // (`egui-0.36.2/src/data/input/raw_input.rs:56-60`).
        if ctx.memory(|memory| memory.had_focus_last_frame(egui::Id::new(BPM_FIELD_ID))) {
            ctx.input_mut(|i| keep_digits_in_text_events(&mut i.events));
        }
        if !self.keyboard_elsewhere {
            // A command Zoom chord answers `show_source_scene`, not the
            // Source. `egui-winit` and eframe's web backend both withhold
            // `Event::Text` while a command modifier is held
            // (`egui-winit-0.36.2/src/lib.rs:1059-1065`,
            // `eframe-0.36.2/src/web/events.rs:155-162`), so a shipped build
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
            // A File chord runs `run_file_command`; `translate_event` already
            // keeps command-modified letters from the Source.
            #[cfg(not(target_arch = "wasm32"))]
            if file_command_chosen.is_none() {
                file_command_chosen = ctx.input(|i| i.events.iter().find_map(file_command));
            }
        } else {
            // Keys a control took are still the event that follows a command
            // Enter, so they disarm its fill as one reaching the Source would.
            self.orcvs.disarm_fill();
            // `show_source_scene` reads Zoom chords, so drop them here while
            // the keys are elsewhere, such as a discard confirmation.
            ctx.input_mut(|i| i.events.retain(|event| zoom_command(event).is_none()));
        }
        if let Some(command) = file_command_chosen {
            self.run_file_command(command);
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.guard_close(&ctx);
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
        let theme = self.themes.presented(appearance).clone();
        #[cfg(any(target_arch = "wasm32", test))]
        {
            self.painted_backdrop = theme.window_background;
        }

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
            .frame(source_panel_frame(theme.grid_background))
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
                    themes: _,
                    #[cfg(any(target_arch = "wasm32", test))]
                        painted_backdrop: _,
                    reduced_motion: _,
                    cursor_effects: _,
                    cursor_effect_animation: _,
                    #[cfg(feature = "persistence")]
                        persistence: _,
                    discard_confirmation: _,
                    #[cfg(not(target_arch = "wasm32"))]
                        source_file: _,
                    #[cfg(not(target_arch = "wasm32"))]
                        file_notices: _,
                    #[cfg(not(target_arch = "wasm32"))]
                        shown_title: _,
                    #[cfg(not(target_arch = "wasm32"))]
                        closing: _,
                    ctx: _,
                } = self;
                let presented = show_source_scene(
                    ui,
                    &frame,
                    font_family,
                    source_view,
                    cursor_effect_sample,
                    cursor_effect_settings,
                    &theme,
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

        self.show_discard_confirmation(&ctx);
        #[cfg(not(target_arch = "wasm32"))]
        self.show_title(&ctx);

        // Sampled once every widget has taken or surrendered focus and every
        // popup has opened or closed. A showing confirmation holds the keys.
        self.keyboard_elsewhere = ctx.egui_wants_keyboard_input()
            || egui::Popup::is_any_open(&ctx)
            || self.discard_confirmation.is_some();

        // Last, once every widget of this frame has been styled from the
        // appearance it began in, so no frame mixes two Themes. The next
        // frame presents the change.
        if let Some(change) = appearance_change {
            Self::change_appearance(&ctx, change);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod kittest_tests;

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "persistence"))]
mod storage_tests;
