use std::time::Duration;

use egui::{Color32, FontId, Key, Rect, emath::TSTransform};

mod glyphs;
mod input;
mod shapes;
mod source_view;

use self::input::{FileCommand, ZoomCommand};
use self::shapes::effect_outline;
use self::source_view::{SourceView, show_source_scene, source_panel_frame};
use crate::config::Config;
use crate::cursor_effects::{CursorEffectAnimation, CursorEffectSettings, effect_bounds};
use crate::function_reference::function_reference;
use crate::grid_viewport::CELL_SIZE;
use crate::midi::{MidiDeviceSelection, destination_presentation};
use crate::native_midi::{self, NativeMidiBackend};
use crate::persistence::{default_source, starting_source};
use crate::readout_deadline::until_next;
use crate::theme::Appearance;
use crate::theme_registry::ThemeRegistry;
use crate::theme_selection::SelectedThemes;
use orcvs::{
    app::Orcvs,
    opts::{Bpm, DEFAULT_FONT_SIZE},
    playback::{PlaybackStartError, PlaybackState},
    source::Source,
};

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

#[cfg(test)]
mod run_clock_tests;

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
        self.keep_tab_for_source(&ctx);
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

        let file_chord = self.route_keys(&ctx);
        if let Some(command) = file_command_chosen.or(file_chord) {
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

        self.latch_keyboard_owner(&ctx);

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
