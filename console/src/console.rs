use std::time::Duration;

use egui::{FontId, Rect};

mod diagnostics_window;
mod files;
mod glyphs;
mod input;
mod menu_bar;
mod panel;
mod repaint;
mod shapes;
mod source_view;

use self::diagnostics_window::show_diagnostics;
use self::files::DiscardConfirmation;
use self::menu_bar::TOP_PANEL_HEIGHT;
use self::panel::BOTTOM_PANEL_HEIGHT;
#[cfg(test)]
use self::panel::BPM_FIELD_ID;
use self::repaint::{moving_run_clock, request_timed_repaint, wake_panel_when_playback_publishes};
use self::shapes::effect_outline;
use self::source_view::{SourceView, show_source_scene, source_panel_frame};
use crate::config::Config;
use crate::cursor_effects::{CursorEffectAnimation, CursorEffectSettings, effect_bounds};
use crate::grid_viewport::CELL_SIZE;
use crate::midi::MidiDeviceSelection;
use crate::native_midi::NativeMidiBackend;
use crate::persistence::starting_source;
use crate::theme::Appearance;
use crate::theme_registry::ThemeRegistry;
use crate::theme_selection::SelectedThemes;
use orcvs::{app::Orcvs, opts::DEFAULT_FONT_SIZE, playback::PlaybackStartError, source::Source};

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
        let source_file = crate::source_file::OpenSourceFile::untitled(
            crate::persistence::default_source().snapshot(),
        );
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
}

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
        self.keep_tab_for_source(&ctx);
        self.observe_playback_diagnostics();
        let menu = self.show_menu_bar(root);
        // A File command chosen from the menu or by its chord, run once the
        // menu bar is done and the chords are read.
        let file_chord = self.route_keys(&ctx);
        if let Some(command) = menu.file_command.or(file_chord) {
            self.run_file_command(command);
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.guard_close(&ctx);
        let frame = self.orcvs.render_frame();
        let observation = self.orcvs.playback_observation();
        let sampled_run_clock = observation.run_clock();
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

        // Shown before the Source so it takes height rather than overlaying
        // the Grid.
        self.show_panel(root, &observation, sampled_run_clock);

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

        request_timed_repaint(
            &ctx,
            moving_run_clock(&observation, sampled_run_clock),
            cursor_delay,
        );

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
        if let Some(change) = menu.appearance_change {
            Self::change_appearance(&ctx, change);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod kittest_tests;

#[cfg(test)]
mod run_clock_tests;

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "persistence"))]
mod storage_tests;
