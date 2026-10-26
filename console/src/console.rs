//! The console: the running Orcvs presented through egui.
//!
//! `Console::ui` composes each Render Frame from the modules that own its
//! parts, in this order: `input` keeps Tab for the Source and routes the
//! keys; `menu_bar` shows the top bar; `files` runs the File command and, on
//! native, guards the close; `panel` shows the bottom Panel; `source_view`
//! presents the Source, which `shapes` draws in the Glyphs `glyphs` lays
//! out; `overview` floats the Overview over the window's corner when it is
//! open and answers where a press on it Pans the Source View; `repaint`
//! schedules the next timed Render Frame; `diagnostics_panel` fills the
//! Panel's Diagnostics readouts; `files` asks its discard question and titles
//! the window; `input` latches the keyboard owner; and a View menu appearance
//! change applies last, once every widget of the frame has been styled.

mod diagnostics_panel;
mod files;
mod glyphs;
mod input;
mod menu_bar;
pub(crate) mod overview;
mod panel;
mod repaint;
pub(crate) mod shapes;
mod source_view;
// The splash the native console opens through.
#[cfg(not(target_arch = "wasm32"))]
mod splash;
#[cfg(not(target_arch = "wasm32"))]
pub use splash::SPLASH_WINDOW;

use self::diagnostics_panel::{TimingReadouts, show_diagnostics};
use self::files::DiscardConfirmation;
use self::menu_bar::TOP_PANEL_HEIGHT;
use self::overview::Overview;
use self::panel::BOTTOM_PANEL_HEIGHT;
#[cfg(test)]
use self::panel::BPM_FIELD_ID;
use self::repaint::{moving_run_clock, request_timed_repaint, wake_panel_when_playback_publishes};
use self::source_view::SourceView;
use crate::config::Config;
use crate::console_midi::ConsoleMidiBackend;
use crate::linked_source::Opening;
use crate::midi::MidiDeviceSelection;
use crate::persistence::starting_source;
use crate::theme::Appearance;
use crate::theme_registry::ThemeRegistry;
use crate::theme_selection::SelectedThemes;
use orcvs::{app::Orcvs, playback::PlaybackStartError, source::Source};

///
/// The Source area of the default window in points at egui's zoom factor of
/// 1.0, margin included. Fixed in points rather than counted in Cells, so the
/// window keeps its size whatever Cell it shows: about 85 columns and 35 rows
/// of the default 12 by 18 point Cell (ADR 0079).
///
pub(crate) const DEFAULT_SOURCE_AREA: [f32; 2] = [1024.0, 640.0];

///
/// The default window: a 1024 by 640 point Source area plus the top bar and compact
/// bottom Panel. Diagnostics expand the Panel within this size and reduce the
/// visible rows; they are enabled by default.
///
/// A larger window shows more of the Grid rather than larger Cells; a smaller
/// one shows less of it.
///
pub const DEFAULT_VIEW_SIZE: [f32; 2] = [
    DEFAULT_SOURCE_AREA[0],
    DEFAULT_SOURCE_AREA[1] + TOP_PANEL_HEIGHT + BOTTOM_PANEL_HEIGHT,
];

///
/// The smallest window the native console lets itself be resized to.
///
pub const DEFAULT_VIEW_SIZE_MIN: [f32; 2] = [300.0, 220.0];

/// Console wraps the running Orcvs with egui presentation concerns.
///
pub struct Console {
    orcvs: Orcvs,
    /// Device discovery and selection for whatever MIDI backend `orcvs` has on
    /// this target. The console never asks what target it is on: a target with
    /// no MIDI service answers an empty destination list here, and
    /// `console_midi::AVAILABLE` says whether the ComboBox is enabled and
    /// whether Scan is shown.
    midi: MidiDeviceSelection,
    font_family: egui::FontFamily,
    source_view: SourceView,
    /// Whether the View menu shows Diagnostics. Off at start and not saved.
    diagnostics_open: bool,
    diagnostics_timing: TimingReadouts,
    /// Whether the View menu shows the Overview. On at start and not saved.
    overview_open: bool,
    overview: Overview,
    /// The BPM field's widget id, so a later pass can find the rectangle it
    /// occupied and so focus is the same id the field is shown under.
    #[cfg(test)]
    bpm_widget_id: egui::Id,
    /// Whether keyboard input belonged to a control rather than the Source
    /// when the last frame's widgets were done: any widget holding egui's
    /// keyboard focus (`Context::egui_wants_keyboard_input`, which is
    /// `Memory::focused().is_some()`), or any open popup — a menu, or the
    /// destination ComboBox's list — which a click opens without taking focus
    /// (`Popup::is_any_open`). Latched
    /// rather than asked where it is read, because
    /// `event_handler` runs before this frame's widgets are shown, and
    /// `Memory::begin_pass` has already let Escape clear the focus it was
    /// pressed to leave.
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
    /// The splash the window opened as, until its logo has faded.
    #[cfg(not(target_arch = "wasm32"))]
    splash: Option<splash::Splash>,
    /// The context an Open wakes the Panel through when the new Orcvs's
    /// Playback Engine publishes.
    ctx: egui::Context,
    /// Whether `save` stores the Source. A console its URL opened stores
    /// nothing, so the Source a reader keeps is never written over.
    #[cfg(feature = "persistence")]
    stores_source: bool,
}

impl Console {
    ///
    /// The console over the running Orcvs its storage last held.
    ///
    /// Fallible because a running Orcvs is: its Playback Engine is a task, and
    /// a task needs a runtime to be spawned on. The native binary
    /// is inside `#[tokio::main]` when `eframe` calls this, and the browser
    /// spawns onto the page's event loop and needs nothing; a build that
    /// reached here with neither has no console to show, which is what handing
    /// the error to `eframe` says.
    ///
    pub fn start(cc: &eframe::CreationContext<'_>) -> Result<Self, PlaybackStartError> {
        // Custom Themes load here, during construction and never inside a
        // frame: native reads `~/.orcvs/themes/`, and the web has the
        // built-ins alone. This is the only place the shipped console
        // discovers Themes; `new` takes the registry, so a test builds the
        // one it needs and never reads the machine's own Theme directory.
        // Settings are read here for the same reason: native reads
        // `~/.orcvs/config.toml` and the web runs on the defaults.
        let console = Self::new(
            cc,
            ThemeRegistry::start(cc.storage),
            Config::start(),
            Opening::Stored,
        )?;
        #[cfg(not(target_arch = "wasm32"))]
        let console = console.through_splash(crate::window_geometry::stored(cc.storage));
        Ok(console)
    }

    ///
    /// The console at a URL with `search` and `fragment`, as the web page's
    /// `Location` gives them: the Source the URL names, or the stored one when
    /// it names none (`crate::linked_source`).
    ///
    #[cfg(target_arch = "wasm32")]
    pub fn start_at(
        cc: &eframe::CreationContext<'_>,
        search: &str,
        fragment: &str,
    ) -> Result<Self, PlaybackStartError> {
        Self::new(
            cc,
            ThemeRegistry::start(cc.storage),
            Config::start(),
            crate::linked_source::opening(search, fragment),
        )
    }

    ///
    /// The console over the Source `opening` names, choosing among the Themes
    /// `registry` holds under the settings `config` holds. [`Console::start`]
    /// is the shipped entry point, and discovers `registry` and reads `config`
    /// itself.
    ///
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        mut registry: ThemeRegistry,
        config: Config,
        opening: Opening,
    ) -> Result<Self, PlaybackStartError> {
        #[cfg(feature = "persistence")]
        let stores_source = matches!(opening, Opening::Stored);
        let source = match opening {
            // The stored Source revision, or an empty Source when storage
            // holds none.
            Opening::Stored => starting_source(cc.storage),
            #[cfg(any(target_arch = "wasm32", test))]
            Opening::Linked(Ok(source)) => source,
            #[cfg(any(target_arch = "wasm32", test))]
            Opening::Linked(Err(message)) => {
                registry.notice(message);
                crate::persistence::default_source()
            }
        };

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
        // `set_theme`.
        themes.install(&cc.egui_ctx);

        // egui's default fonts, with the bundled Monaspace Neon put first in
        // both the proportional and the monospace family.
        let mut fonts = egui::FontDefinitions::default();
        let key = "MonaspaceNeon";
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
        let (orcvs, midi) = environment(&cc.egui_ctx, source)?;
        Ok(Self {
            orcvs,
            midi,
            font_family: egui::FontFamily::Monospace,
            source_view: SourceView::new(config.cell),
            diagnostics_open: false,
            diagnostics_timing: TimingReadouts::default(),
            overview_open: true,
            overview: Overview::default(),
            #[cfg(test)]
            bpm_widget_id: egui::Id::new(BPM_FIELD_ID),
            keyboard_elsewhere: false,
            #[cfg(any(target_arch = "wasm32", test))]
            painted_backdrop: themes
                .presented(Appearance::from(cc.egui_ctx.theme()))
                .window_background,
            themes,
            discard_confirmation: None,
            #[cfg(not(target_arch = "wasm32"))]
            source_file,
            #[cfg(not(target_arch = "wasm32"))]
            file_notices: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            shown_title: None,
            #[cfg(not(target_arch = "wasm32"))]
            closing: false,
            #[cfg(not(target_arch = "wasm32"))]
            splash: None,
            ctx: cc.egui_ctx.clone(),
            #[cfg(feature = "persistence")]
            stores_source,
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
        Box::new(ConsoleMidiBackend::new()),
    );
    midi.refresh_destinations();
    #[cfg(target_arch = "wasm32")]
    {
        repaint::wake_panel_when_midi_access_answers(ctx.clone());
        repaint::wake_panel_when_a_midi_output_opens(ctx.clone());
    }
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
    /// intervals while running. This stores the current Source revision and,
    /// natively, the window's geometry; settings are never saved.
    ///
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.stores_source {
            crate::persistence::save(storage, self.orcvs.source());
        }
        // While the splash shows, the window is the splash's, so the save
        // stores the geometry it grows to, or nothing when that is the default.
        #[cfg(not(target_arch = "wasm32"))]
        let window = match &self.splash {
            Some(splash) => splash.console_window(),
            None => Some(crate::window_geometry::WindowGeometry::of(&self.ctx)),
        };
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(window) = window {
            crate::window_geometry::store(storage, &window);
        }
    }

    ///
    /// The window's own clear colour: `.scratch/theming/schema.md`'s Chrome
    /// mapping table, "Application backdrop | `window.background`, opaque."
    ///
    /// This is the window backdrop's own consumer, wired independently of
    /// `source_panel_frame`'s `theme.grid_background`. Without this override
    /// `eframe::App::clear_color`'s own default
    /// (`Color32::from_rgba_unmultiplied(12, 12, 12, 180)`) shows through
    /// wherever a Theme's partly transparent panel, Grid or Cell layer
    /// reveals the console surface beneath it, rather than the Theme's own
    /// opaque backdrop — a translucent grey the Theme never chose, in place
    /// of the surface the Theme format describes: "Transparency reveals the
    /// underlying console surface; the application window remains opaque."
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
    /// (`theme_document::resolve`'s `ThemeError::NonOpaqueWindowBackground`), so
    /// nothing presented can hold a nonopaque value and
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
        #[cfg(not(target_arch = "wasm32"))]
        if self.hold_splash(&ctx, appearance) {
            return;
        }
        self.keep_tab_for_source(&ctx);
        self.observe_playback_diagnostics();
        let menu = self.show_menu_bar(root, appearance);
        // The Overview fits below this. Nothing shown before the Overview
        // may take room from the top, or it would overlap what does.
        let below_menu = root.available_rect_before_wrap().top();
        // A File command chosen from the menu or by its chord, run once the
        // menu bar is done and the chords are read.
        let file_chord = self.route_keys(&ctx);
        if let Some(command) = menu.file_command.or(file_chord) {
            self.run_file_command(command);
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.guard_close(&ctx);
        // Read before the Render Frame, so the Overview's picture can never
        // be marked with a revision newer than the Cells it was drawn from.
        let revision = self.orcvs.source().revision();
        let frame = self.orcvs.render_frame();
        let observation = self.orcvs.playback_observation();
        let sampled_run_clock = observation.run_clock();
        #[cfg(any(target_arch = "wasm32", test))]
        {
            self.painted_backdrop = self.themes.presented(appearance).window_background;
        }

        // Shown before the Source so it takes height rather than overlaying
        // the Grid.
        let diagnostics = self.show_panel(root, &observation, sampled_run_clock, appearance);
        self.show_source_panel(root, &frame, appearance);
        // After the Source, so it outlines the Pan this Render Frame settled.
        if let Some(centre) = self.show_overview(root, below_menu, revision, &frame, appearance) {
            self.source_view.centre_on(centre);
        }
        let shown = self.source_view.shown();

        request_timed_repaint(&ctx, moving_run_clock(&observation, sampled_run_clock));

        if let Some(mut diagnostics) = diagnostics {
            show_diagnostics(
                &mut diagnostics,
                eframe,
                &mut self.diagnostics_timing,
                &shown,
            );
        }

        self.show_discard_confirmation(&ctx);
        #[cfg(not(target_arch = "wasm32"))]
        self.show_title(&ctx);

        self.latch_keyboard_owner(&ctx);

        // Over everything the console showed, while the logo fades.
        #[cfg(not(target_arch = "wasm32"))]
        self.fade_splash(&ctx, appearance);

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
