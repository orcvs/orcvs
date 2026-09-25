//! File workflows: opening a Source as the environment, New, Open, Save and
//! Save As, the question asked before discarding unsaved changes, and on
//! native the window title and the close guard.

use crate::function_reference::function_reference;
use crate::persistence::default_source;
use orcvs::source::Source;

use super::input::FileCommand;
use super::menu_bar::MENU_BAR_GAP;
use super::source_view::SourceView;
use super::{Console, environment};

impl Console {
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
    pub(super) fn load_function_reference(&mut self) {
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
    pub(super) fn open_picked(&mut self, picked: Option<std::path::PathBuf>) {
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
    pub(super) fn save_picked(&mut self, picked: Option<std::path::PathBuf>) {
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
    pub(super) fn run_file_command(&mut self, command: FileCommand) {
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
    pub(super) fn guard_close(&mut self, ctx: &egui::Context) {
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
    pub(super) fn show_title(&mut self, ctx: &egui::Context) {
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
    pub(super) fn discard_asking_first(&mut self, confirmation: DiscardConfirmation) {
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
    pub(super) fn show_discard_confirmation(&mut self, ctx: &egui::Context) {
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
pub(super) struct DiscardConfirmation {
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
pub(super) const FUNCTION_REFERENCE_CONFIRMATION: DiscardConfirmation = DiscardConfirmation {
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
