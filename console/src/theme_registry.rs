//!
//! The Themes a console can select: the built-ins, the custom Themes loaded
//! from Theme documents, and what went wrong loading them.
//! `.scratch/theming/issues/07`, following `schema.md`'s "Discovery,
//! persistence and failures".
//!
//! Every custom document reaches the registry through one path —
//! [`crate::theme_document::decode`], then [`crate::theme::resolve`] against
//! the built-ins under the identity its file name's stem supplies — whether
//! it came from a native file or a web import. Built-in identities are
//! reserved by `resolve` itself.
//!
//! On native, [`ThemeRegistry::start`] scans `~/.orcvs/themes/` once, while
//! `Console::start` runs and before any frame is shown; `Console::new` takes
//! the registry as a parameter, so tests build their own. The files are
//! authoritative: they are read at every launch, never cached in application
//! storage, and never watched. On the web, a Theme file dropped on the console
//! is imported: its bytes are read asynchronously by eframe's own
//! drag-and-drop support, and a valid document atomically replaces the one of
//! the same identity. With `persistence`, imported source documents are
//! stored, within [`MAX_STORED_IMPORTED_BYTES`], and restored on the next
//! start; without it they are session-only.
//!
//! Nothing here chooses what the console paints: `theme_selection`'s
//! `SelectedThemes` holds the registry, resolves the saved dark and light
//! selections against it, and lists its Themes in the View menu's pickers.
//!

use std::collections::BTreeMap;

use crate::contrast;
use crate::theme::{
    Appearance, Theme, ThemeDocument, ThemeIdentity, okabe_ito, orcvs_light, resolve,
};
use crate::theme_document::decode;

/// The file name suffixes a Theme document may carry, matched
/// case-insensitively. `theme_document::decode` selects its decoder from the
/// same four.
const SUFFIXES: [&str; 4] = [".toml", ".json", ".yaml", ".yml"];

///
/// The Theme identity a file name supplies: the name without its Theme
/// document suffix. `None` when the name carries none of [`SUFFIXES`].
/// Matching is case-insensitive; the identity keeps its own case.
///
fn stem(file_name: &str) -> Option<&str> {
    SUFFIXES.iter().find_map(|suffix| {
        let split = file_name.len().checked_sub(suffix.len())?;
        let (stem, tail) = (file_name.get(..split)?, file_name.get(split..)?);
        tail.eq_ignore_ascii_case(suffix).then_some(stem)
    })
}

///
/// Why a selected identity could not be resolved to a Theme.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unavailable {
    /// No built-in and no loaded document has this identity.
    Missing,
    /// A document with this identity exists but was refused: malformed,
    /// unreadable, or in conflict with another file of the same stem.
    Refused(String),
    /// The Theme exists but is not of the slot's appearance.
    WrongAppearance(Appearance),
}

impl std::fmt::Display for Unavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => f.write_str("no Theme file has that identity"),
            Self::Refused(reason) => f.write_str(reason),
            Self::WrongAppearance(appearance) => {
                write!(f, "it is a {} Theme", appearance.word())
            }
        }
    }
}

///
/// A stored web-imported document's source: the file name selects its decoder
/// again on the next start.
///
#[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Imported {
    file_name: String,
    text: String,
}

///
/// The most the stored imported documents may take together, as the exact
/// string written to storage. They share the browser's local storage with the
/// Source, and a full store would make the Source's own autosave fail, so an
/// import that would pass this stays usable for the session but is not stored.
///
#[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
pub(crate) const MAX_STORED_IMPORTED_BYTES: usize = 2 * 1024 * 1024;

///
/// The built-in Themes, the resolved custom Themes, and the notices a viewer
/// is shown about them.
///
pub(crate) struct ThemeRegistry {
    built_ins: [Theme; 2],
    /// Resolved custom Themes by identity (filename stem).
    custom: BTreeMap<ThemeIdentity, Theme>,
    /// Identities a document claimed but that could not be loaded, with the
    /// reason — what a selection of one of them reports.
    refused: BTreeMap<ThemeIdentity, String>,
    /// Load, import and contrast messages, in the order they arose, until the
    /// viewer dismisses them.
    notices: Vec<String>,
    /// Every stored imported document, including one restored from storage
    /// that no longer loads, so the next store writes it back rather than
    /// losing it. Exactly what `store_imported` writes.
    #[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
    imported: Vec<Imported>,
    /// Whether `imported` changed since the last write was attempted.
    #[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
    unstored: bool,
    /// Whether the last write of `imported` was not kept, so a failure is
    /// reported once rather than at every attempt.
    #[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
    store_failed: bool,
    /// A stored `imported_themes` value that could not be decoded, to be moved
    /// aside, and out of that key, at the next store.
    #[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
    refused_store: Option<String>,
}

impl ThemeRegistry {
    ///
    /// The built-ins alone: what a start with no custom Theme holds, and the
    /// registry a test builds when it needs no custom Theme.
    ///
    pub(crate) fn built_in() -> Self {
        Self {
            built_ins: [okabe_ito(), orcvs_light()],
            custom: BTreeMap::new(),
            refused: BTreeMap::new(),
            notices: Vec::new(),
            #[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
            imported: Vec::new(),
            #[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
            unstored: false,
            #[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
            store_failed: false,
            #[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
            refused_store: None,
        }
    }

    ///
    /// The registry the shipped console starts with, built by
    /// `Console::start` before `Console::new`. Native reads the canonical
    /// Theme directory, `~/.orcvs/themes/`, and never application storage;
    /// the web restores its imported documents from storage when
    /// `persistence` stores them.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn start(_storage: Option<&dyn eframe::Storage>) -> Self {
        match std::env::home_dir() {
            Some(home) => Self::discover(&home.join(".orcvs").join("themes")),
            None => {
                let mut registry = Self::built_in();
                registry.notice(
                    "Could not find the home directory, so no Theme files were loaded from \
                     ~/.orcvs/themes"
                        .to_owned(),
                );
                registry
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn start(storage: Option<&dyn eframe::Storage>) -> Self {
        Self::web_start(storage)
    }

    ///
    /// Records a notice for the viewer and reports it on the console's error
    /// channel.
    ///
    fn notice(&mut self, message: String) {
        crate::report::error!("{message}");
        self.notices.push(message);
    }

    /// Every load, import and contrast notice the viewer has not dismissed,
    /// in the order they arose.
    pub(crate) fn notices(&self) -> impl Iterator<Item = &String> {
        self.notices.iter()
    }

    /// Every notice, owned, so a test can index, compare and print them.
    #[cfg(test)]
    pub(crate) fn notice_list(&self) -> Vec<String> {
        self.notices.clone()
    }

    pub(crate) fn dismiss_notices(&mut self) {
        self.notices.clear();
    }

    ///
    /// Every Theme a selection can name: the built-ins, then the loaded
    /// Themes in identity order.
    ///
    pub(crate) fn themes(&self) -> impl Iterator<Item = &Theme> {
        self.built_ins.iter().chain(self.custom.values())
    }

    ///
    /// Decodes and resolves one document under the identity its file name
    /// supplies. Pure: the registry is not touched, so a refusal leaves it
    /// exactly as it was.
    ///
    fn load(&self, file_name: &str, bytes: &[u8]) -> Result<Theme, String> {
        let stem = stem(file_name).ok_or_else(|| {
            format!("{file_name}: not a Theme file; expected .toml, .json, .yaml or .yml")
        })?;
        let identity = ThemeIdentity::from_stem(stem)
            .ok_or_else(|| format!("{file_name}: a Theme file's name needs a stem"))?;
        let document: ThemeDocument = decode(file_name, bytes).map_err(|e| e.to_string())?;
        resolve(&self.built_ins, &identity, &document).map_err(|e| format!("{file_name}: {e}"))
    }

    ///
    /// Adds a loaded Theme, with a notice when `08`'s contrast validator finds
    /// a state below the floor. A contrast failure never refuses the Theme.
    ///
    fn add(&mut self, theme: Theme) {
        if let Some(message) = contrast_notice(&theme) {
            self.notice(message);
        }
        self.refused.remove(&theme.identity);
        self.custom.insert(theme.identity.clone(), theme);
    }

    ///
    /// The Theme `identity` names for the `slot` appearance, or why it cannot
    /// be used.
    ///
    pub(crate) fn select(
        &self,
        slot: Appearance,
        identity: &ThemeIdentity,
    ) -> Result<&Theme, Unavailable> {
        let theme = self
            .built_ins
            .iter()
            .find(|theme| theme.identity == *identity)
            .or_else(|| self.custom.get(identity));
        match theme {
            Some(theme) if theme.appearance == slot => Ok(theme),
            Some(theme) => Err(Unavailable::WrongAppearance(theme.appearance)),
            None => Err(self
                .refused
                .get(identity)
                .map_or(Unavailable::Missing, |reason| {
                    Unavailable::Refused(reason.clone())
                })),
        }
    }

    ///
    /// The built-in a `slot` falls back to, [`ThemeIdentity::default_for`].
    ///
    pub(crate) fn fallback(&self, slot: Appearance) -> &Theme {
        let identity = ThemeIdentity::default_for(slot);
        self.built_ins
            .iter()
            .find(|theme| theme.identity == identity)
            .expect("every slot's default identity is a built-in")
    }
}

///
/// `08`'s report on `theme`, as a notice naming every state below the floor,
/// or `None` when every state clears it.
///
fn contrast_notice(theme: &Theme) -> Option<String> {
    let report = contrast::validate(theme);
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|result| !result.passes() && !result.accepted)
        .map(|result| format!("{} ({}) {:.2}:1", result.role, result.state, result.ratio))
        .collect();
    (!failing.is_empty()).then(|| {
        format!(
            "Theme \"{}\" is loaded, but {} of {} text states measure below the {}:1 contrast \
             floor: {}",
            theme.identity,
            failing.len(),
            report.results.len(),
            report.floor,
            failing.join("; ")
        )
    })
}

// === Native discovery ===

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::collections::BTreeMap;
    use std::io::Read as _;
    use std::path::{Path, PathBuf};

    use super::{ThemeRegistry, stem};
    use crate::theme::ThemeIdentity;
    use crate::theme_document::MAX_DOCUMENT_BYTES;

    ///
    /// One directory entry whose name carries a Theme document suffix.
    ///
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(super) struct Candidate {
        pub(super) path: PathBuf,
        pub(super) file_name: String,
        pub(super) identity: ThemeIdentity,
        /// Whether the entry is a file that can be loaded. One that is not
        /// has already been reported, but still claims its identity, so it
        /// conflicts with any other file of the same stem.
        pub(super) loadable: bool,
    }

    ///
    /// Groups candidates by identity. An identity with one file loads it; an
    /// identity with several is a conflict, reported with every file's path.
    /// Both halves are keyed and sorted, so directory enumeration order
    /// decides nothing.
    ///
    pub(super) fn group(
        candidates: Vec<Candidate>,
    ) -> (
        BTreeMap<ThemeIdentity, Candidate>,
        BTreeMap<ThemeIdentity, Vec<PathBuf>>,
    ) {
        let mut by_identity: BTreeMap<ThemeIdentity, Vec<Candidate>> = BTreeMap::new();
        for candidate in candidates {
            by_identity
                .entry(candidate.identity.clone())
                .or_default()
                .push(candidate);
        }
        let mut unique = BTreeMap::new();
        let mut conflicts = BTreeMap::new();
        for (identity, mut files) in by_identity {
            if files.len() == 1 {
                unique.insert(identity, files.remove(0));
            } else {
                let mut paths: Vec<PathBuf> = files.into_iter().map(|file| file.path).collect();
                paths.sort();
                conflicts.insert(identity, paths);
            }
        }
        (unique, conflicts)
    }

    ///
    /// Reads at most one byte past the document limit, so an oversized file is
    /// refused by the bytes actually read rather than by a size its metadata
    /// claimed, and never read whole.
    ///
    fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.take(MAX_DOCUMENT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(format!(
                "the Theme document is over the {MAX_DOCUMENT_BYTES}-byte limit"
            ));
        }
        Ok(bytes)
    }

    impl ThemeRegistry {
        ///
        /// The built-ins plus every Theme document directly in `dir`.
        ///
        /// Only direct children whose names end in a Theme document suffix
        /// are considered; other files are ignored. A directory, or a
        /// symbolic link to one, is never entered. A file symbolic link is
        /// followed. A missing `dir` means no custom Themes; an unreadable
        /// `dir` or file is reported and skipped, though a file still refuses
        /// every other file of its stem.
        ///
        pub(crate) fn discover(dir: &Path) -> Self {
            let mut registry = Self::built_in();
            let entries = match std::fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return registry,
                Err(error) => {
                    registry.notice(format!(
                        "Could not read the Theme directory {}: {error}",
                        dir.display()
                    ));
                    return registry;
                }
            };

            let mut candidates = Vec::new();
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        registry.notice(format!(
                            "Could not read the Theme directory {}: {error}",
                            dir.display()
                        ));
                        continue;
                    }
                };
                let path = entry.path();
                let name = entry.file_name();
                let Some(file_name) = name.to_str() else {
                    if stem(&name.to_string_lossy()).is_some() {
                        registry.notice(format!(
                            "{}: a Theme file's name must be UTF-8",
                            path.display()
                        ));
                    }
                    continue;
                };
                let Some(stem) = stem(file_name) else {
                    continue;
                };
                // `metadata` follows a symbolic link, so a link to a
                // directory is skipped here like a directory, and a dangling
                // link is an unreadable file. An entry that cannot be loaded
                // is reported here and still grouped, so it refuses every
                // other file of its stem.
                let loadable = match std::fs::metadata(&path) {
                    Ok(metadata) if metadata.is_dir() => continue,
                    Ok(metadata) if metadata.is_file() => true,
                    Ok(_) => {
                        registry.notice(format!("{}: not a regular Theme file", path.display()));
                        false
                    }
                    Err(error) => {
                        registry.notice(format!(
                            "Could not read the Theme file {}: {error}",
                            path.display()
                        ));
                        false
                    }
                };
                let Some(identity) = ThemeIdentity::from_stem(stem) else {
                    // An entry that cannot be loaded was reported above, and
                    // one without an identity conflicts with nothing.
                    if loadable {
                        registry.notice(format!(
                            "{}: a Theme file's name needs a stem, which is its identity",
                            path.display()
                        ));
                    }
                    continue;
                };
                candidates.push(Candidate {
                    identity,
                    file_name: file_name.to_owned(),
                    path,
                    loadable,
                });
            }

            let (unique, conflicts) = group(candidates);
            for (identity, paths) in conflicts {
                let listed = paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let reason = format!("several Theme files share its identity: {listed}");
                registry.notice(format!(
                    "Refused every Theme file with the identity \"{identity}\": {reason}"
                ));
                registry.refused.insert(identity, reason);
            }
            for (identity, candidate) in unique {
                if !candidate.loadable {
                    continue;
                }
                let loaded = read_bounded(&candidate.path)
                    .map_err(|error| format!("{}: {error}", candidate.file_name))
                    .and_then(|bytes| registry.load(&candidate.file_name, &bytes));
                match loaded {
                    Ok(theme) => registry.add(theme),
                    Err(reason) => {
                        registry.notice(format!(
                            "Refused the Theme file {}: {reason}",
                            candidate.path.display()
                        ));
                        // A reserved identity is still the built-in's: the
                        // refused file never shadows it.
                        if !identity.is_reserved() {
                            registry.refused.insert(identity, reason);
                        }
                    }
                }
            }
            registry
        }
    }
}

// === Web import ===

#[cfg(any(target_arch = "wasm32", test))]
impl ThemeRegistry {
    ///
    /// The web target's start: the built-ins, plus the documents storage
    /// holds when `persistence` stored them.
    ///
    pub(crate) fn web_start(storage: Option<&dyn eframe::Storage>) -> Self {
        #[cfg_attr(
            not(feature = "persistence"),
            expect(unused_mut, reason = "restored only with persistence")
        )]
        let mut registry = Self::built_in();
        #[cfg(feature = "persistence")]
        if let Some(storage) = storage {
            registry.restore_imported(storage);
        }
        #[cfg(not(feature = "persistence"))]
        let _ = storage;
        registry
    }

    ///
    /// The file names in one drop that may be read and imported. Files that
    /// share a stem would otherwise race, the last read to finish winning, so
    /// every file of such a stem is refused and reported by name, as native
    /// discovery refuses a duplicate identity. A name with no Theme suffix
    /// passes through, for `import` to refuse.
    ///
    pub(crate) fn refuse_conflicting_drops(&mut self, file_names: Vec<String>) -> Vec<String> {
        let mut by_stem: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for file_name in &file_names {
            if let Some(identity) = stem(file_name) {
                by_stem.entry(identity).or_default().push(file_name);
            }
        }
        let conflicted: BTreeMap<String, Vec<String>> = by_stem
            .into_iter()
            .filter(|(_, names)| names.len() > 1)
            .map(|(identity, mut names)| {
                names.sort_unstable();
                (
                    identity.to_owned(),
                    names.into_iter().map(str::to_owned).collect(),
                )
            })
            .collect();
        for (identity, names) in &conflicted {
            self.notice(format!(
                "Refused every dropped Theme file with the identity \"{identity}\": several \
                 files in one drop share it: {}",
                names.join(", ")
            ));
        }
        file_names
            .into_iter()
            .filter(|file_name| stem(file_name).is_none_or(|s| !conflicted.contains_key(s)))
            .collect()
    }

    ///
    /// Imports one Theme file. A valid document replaces the one of the same
    /// identity whole, and its identity is returned. An invalid one is
    /// reported and leaves the registry, and any previous document of that
    /// identity, exactly as they were.
    ///
    pub(crate) fn import(
        &mut self,
        file_name: &str,
        bytes: &[u8],
    ) -> Result<ThemeIdentity, String> {
        match self.load(file_name, bytes) {
            Ok(theme) => {
                #[cfg(feature = "persistence")]
                self.stage_for_storage(&theme.identity, file_name, bytes);
                let identity = theme.identity.clone();
                self.add(theme);
                Ok(identity)
            }
            Err(reason) => {
                self.notice(format!("Refused the imported Theme file {reason}"));
                Err(reason)
            }
        }
    }

    ///
    /// Reports a dropped file the browser could not read.
    ///
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn refuse_unreadable(&mut self, file_name: &str, error: &str) {
        self.notice(format!(
            "Could not read the dropped file {file_name}: {error}"
        ));
    }
}

#[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
impl ThemeRegistry {
    ///
    /// The exact string `store_imported` writes for `documents`.
    ///
    fn serialised(documents: &[Imported]) -> String {
        let pairs: Vec<(&str, &str)> = documents
            .iter()
            .map(|imported| (imported.file_name.as_str(), imported.text.as_str()))
            .collect();
        serde_json::to_string(&pairs).expect("a list of string pairs always serialises")
    }

    ///
    /// Replaces the stored document of `identity` with a newly imported one,
    /// unless the stored documents would then pass
    /// [`MAX_STORED_IMPORTED_BYTES`]: then the import is used for this
    /// session only, the stored documents are left as they were, and a notice
    /// says so, naming any stored document of `identity` the next start
    /// restores in its place.
    ///
    fn stage_for_storage(&mut self, identity: &ThemeIdentity, file_name: &str, bytes: &[u8]) {
        // `load` refused anything that is not UTF-8.
        let text = String::from_utf8_lossy(bytes).into_owned();
        let mut staged: Vec<Imported> = self
            .imported
            .iter()
            .filter(|imported| stem(&imported.file_name) != Some(identity.as_str()))
            .cloned()
            .collect();
        staged.push(Imported {
            file_name: file_name.to_owned(),
            text,
        });
        if Self::serialised(&staged).len() > MAX_STORED_IMPORTED_BYTES {
            // The stored documents are left as they were, so a stored
            // document of this identity is what the next start restores.
            let returning = self
                .imported
                .iter()
                .find(|imported| stem(&imported.file_name) == Some(identity.as_str()))
                .map_or_else(String::new, |stored| {
                    format!(
                        "; the stored {} returns on the next start",
                        stored.file_name
                    )
                });
            self.notice(format!(
                "Imported {file_name} for this session only: storing it would take the imported \
                 Themes past their {MAX_STORED_IMPORTED_BYTES}-byte storage budget{returning}"
            ));
            return;
        }
        self.imported = staged;
        self.unstored = true;
    }

    ///
    /// Loads every stored imported document. One that no longer loads — a
    /// malformed document, or a name with no Theme suffix — is reported and
    /// kept, so a selection of it falls back with the reason and the next
    /// store does not discard it. A stored value that is not a list of
    /// documents at all is kept whole, moved aside at the next store, and
    /// then replaced under its key by the documents imported since.
    ///
    fn restore_imported(&mut self, storage: &dyn eframe::Storage) {
        use crate::persistence::{IMPORTED_THEMES_KEY, IMPORTED_THEMES_REFUSED_KEY};

        let Some(raw) = storage.get_string(IMPORTED_THEMES_KEY) else {
            return;
        };
        let Ok(stored) = serde_json::from_str::<Vec<(String, String)>>(&raw) else {
            self.notice(format!(
                "The stored imported Themes could not be read back; they are kept under \
                 \"{IMPORTED_THEMES_REFUSED_KEY}\""
            ));
            self.refused_store = Some(raw);
            return;
        };
        for (file_name, text) in stored {
            match self.load(&file_name, text.as_bytes()) {
                Ok(theme) => self.add(theme),
                Err(reason) => {
                    self.notice(format!("Refused the stored imported Theme {reason}"));
                    if let Some(identity) = stem(&file_name).and_then(ThemeIdentity::from_stem) {
                        self.refused.insert(identity, reason);
                    }
                }
            }
            self.imported.push(Imported { file_name, text });
        }
    }

    ///
    /// Moves an undecodable stored value aside, then writes the imported
    /// documents when they changed or when that value must leave their key,
    /// and reports a write storage did not keep
    /// rather than claiming it did. A failed write is not retried until the
    /// imported documents change again, and is reported once. A copy of the
    /// undecodable value that storage did not keep is retried at every store,
    /// and until one is kept nothing is written over the value.
    ///
    pub(crate) fn store_imported(&mut self, storage: &mut dyn eframe::Storage) {
        use crate::persistence::{IMPORTED_THEMES_KEY, IMPORTED_THEMES_REFUSED_KEY};

        if let Some(refused) = &self.refused_store {
            storage.set_string(IMPORTED_THEMES_REFUSED_KEY, refused.clone());
            // Only once the copy is kept does the undecodable value leave its
            // key, so the next start neither reports it again nor overwrites
            // the copy. A copy storage did not keep leaves the value where it
            // was, and nothing is written over it until a later store keeps
            // the copy.
            if storage.get_string(IMPORTED_THEMES_REFUSED_KEY).as_deref() == Some(refused.as_str())
            {
                self.refused_store = None;
                self.unstored = true;
            } else {
                self.report_store_failure();
                return;
            }
        }
        if !self.unstored {
            return;
        }
        self.unstored = false;
        let written = Self::serialised(&self.imported);
        storage.set_string(IMPORTED_THEMES_KEY, written.clone());
        if storage.get_string(IMPORTED_THEMES_KEY).as_deref() == Some(written.as_str()) {
            self.store_failed = false;
        } else {
            self.report_store_failure();
        }
    }

    ///
    /// Reports a write storage did not keep, once until a write is kept.
    ///
    fn report_store_failure(&mut self) {
        if !self.store_failed {
            self.store_failed = true;
            self.notice(
                "Could not store the imported Themes; they remain for this session only".to_owned(),
            );
        }
    }
}

// === Web file drops ===

///
/// One dropped file's name and what reading it produced.
///
#[cfg(target_arch = "wasm32")]
type Read = (String, Result<Vec<u8>, String>);

#[cfg(target_arch = "wasm32")]
use crate::theme_selection::SelectedThemes;

///
/// Reads Theme files dropped on the web console and hands their bytes to the
/// console's [`SelectedThemes`], which imports them into its registry.
///
/// eframe's web backend turns a browser `drop` into
/// [`egui::RawInput::dropped_files`], and each file reads its own bytes
/// asynchronously through [`egui::DroppedFile::bytes_async`]. The console
/// takes the files out of each `RawInput` in `App::raw_input_hook`, which
/// sees every `RawInput` once, so a drop is read exactly once — `App::logic`
/// would see a hidden tab's last input again on every call. A read runs on
/// the page's event loop and sends its result here; a later frame applies it.
/// A drag the viewer abandons delivers no file, so it changes nothing.
///
#[cfg(target_arch = "wasm32")]
pub(crate) struct WebImport {
    sender: std::sync::mpsc::Sender<Read>,
    receiver: std::sync::mpsc::Receiver<Read>,
}

#[cfg(target_arch = "wasm32")]
impl WebImport {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self { sender, receiver }
    }

    ///
    /// Takes the files dropped in `raw_input` and starts reading each one
    /// `registry` does not refuse outright. A file the browser says is over
    /// the document limit is refused before any of it is read;
    /// `theme_document::decode` checks the bytes actually read again.
    ///
    pub(crate) fn take_drops(
        &self,
        ctx: &egui::Context,
        raw_input: &mut egui::RawInput,
        themes: &mut SelectedThemes,
    ) {
        use crate::theme_document::MAX_DOCUMENT_BYTES;

        let dropped = std::mem::take(&mut raw_input.dropped_files);
        if dropped.is_empty() {
            return;
        }
        let named: Vec<(String, egui::DroppedFileHandle)> = dropped
            .into_iter()
            .map(|file| {
                let path = file.path();
                let name = path
                    .file_name()
                    .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy())
                    .into_owned();
                (name, file)
            })
            .collect();
        let accepted =
            themes.refuse_conflicting_drops(named.iter().map(|(name, _)| name.clone()).collect());
        #[expect(
            clippy::cast_precision_loss,
            reason = "the limit is 1 MiB, exactly representable as an f64"
        )]
        let limit = MAX_DOCUMENT_BYTES as f64;
        for (file_name, file) in named {
            if !accepted.contains(&file_name) {
                continue;
            }
            let sender = self.sender.clone();
            if file.web_file().is_some_and(|web| web.size() > limit) {
                let _ = sender.send((
                    file_name,
                    Err(format!(
                        "the Theme document is over the {MAX_DOCUMENT_BYTES}-byte limit"
                    )),
                ));
                continue;
            }
            let ctx = ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let bytes = file.bytes_async().await;
                // The receiver lives as long as the console; a send can only
                // fail once the console is gone, when there is nothing left
                // to import into.
                let _ = sender.send((file_name, bytes));
                ctx.request_repaint();
            });
        }
    }

    ///
    /// Imports every file whose read has finished, and answers whether any
    /// import succeeded, which may change what is presented. Draining is
    /// idempotent, so calling this more than once a frame imports nothing
    /// twice.
    ///
    pub(crate) fn apply(&self, themes: &mut SelectedThemes) -> bool {
        let mut imported = false;
        while let Ok((file_name, read)) = self.receiver.try_recv() {
            match read {
                // `import` records its own refusal notice.
                Ok(bytes) => imported |= themes.import(&file_name, &bytes).is_ok(),
                Err(error) => themes.refuse_unreadable(&file_name, &error),
            }
        }
        imported
    }
}

#[cfg(test)]
mod tests {
    use super::tests_support::{dark, dark_json, id};
    use super::{ThemeRegistry, Unavailable, stem};
    use crate::theme::{
        Appearance, OKABE_ITO_IDENTITY, ORCVS_LIGHT_IDENTITY, okabe_ito, orcvs_light,
    };
    use crate::theme_selection::{SelectedThemes, ThemeSelection};

    fn notices_mention(registry: &ThemeRegistry, needle: &str) -> bool {
        registry
            .notice_list()
            .iter()
            .any(|notice| notice.contains(needle))
    }

    #[test]
    fn a_file_name_stem_is_its_identity_matched_case_insensitively_by_suffix() {
        assert_eq!(stem("my-dark.toml"), Some("my-dark"));
        assert_eq!(stem("My-Dark.TOML"), Some("My-Dark"));
        assert_eq!(stem("a.b.Json"), Some("a.b"));
        assert_eq!(stem("x.yaml"), Some("x"));
        assert_eq!(stem("x.YML"), Some("x"));
        assert_eq!(stem(".toml"), Some(""));
        assert_eq!(stem("notes.txt"), None);
        assert_eq!(stem("x.toml.bak"), None);
        assert_eq!(stem("toml"), None);
        assert_eq!(stem("é.toml"), Some("é"));
    }

    #[test]
    fn built_in_identities_select_their_built_in_and_the_default_is_the_fallback() {
        let registry = ThemeRegistry::built_in();
        assert_eq!(
            registry.select(Appearance::Dark, &OKABE_ITO_IDENTITY),
            Ok(&okabe_ito())
        );
        assert_eq!(
            registry.select(Appearance::Light, &ORCVS_LIGHT_IDENTITY),
            Ok(&orcvs_light())
        );
        assert_eq!(registry.fallback(Appearance::Dark), &okabe_ito());
        assert_eq!(registry.fallback(Appearance::Light), &orcvs_light());
        assert_eq!(
            registry.select(Appearance::Light, &OKABE_ITO_IDENTITY),
            Err(Unavailable::WrongAppearance(Appearance::Dark)),
            "a built-in is available only to its own appearance"
        );
        assert_eq!(
            registry.select(Appearance::Dark, &ORCVS_LIGHT_IDENTITY),
            Err(Unavailable::WrongAppearance(Appearance::Light))
        );
        assert_eq!(
            registry.select(Appearance::Dark, &id("nobody")),
            Err(Unavailable::Missing)
        );
    }

    // === Web import ===

    #[test]
    fn a_web_import_takes_its_identity_from_the_file_name_and_its_label_from_the_document() {
        let mut registry = ThemeRegistry::built_in();
        registry
            .import("my-dark.toml", dark("Midnight").as_bytes())
            .expect("a valid document");

        let theme = registry
            .select(Appearance::Dark, &id("my-dark"))
            .expect("imported");
        assert_eq!(theme.identity.as_str(), "my-dark");
        assert_eq!(theme.name, "Midnight");
        assert!(registry.select(Appearance::Dark, &id("Midnight")).is_err());
    }

    #[test]
    fn a_valid_reimport_replaces_the_document_of_the_same_identity_whole() {
        let mut registry = ThemeRegistry::built_in();
        let with_text = "format = \"orcvs-theme\"\nversion = 1\nname = \"First\"\n\
                         inherits = \"okabe-ito\"\n[style]\ntext = \"#123456\"\n";
        registry
            .import("my-dark.toml", with_text.as_bytes())
            .expect("a valid document");

        registry
            .import("my-dark.json", dark_json("Second").as_bytes())
            .expect("a valid document");

        let theme = registry
            .select(Appearance::Dark, &id("my-dark"))
            .expect("replaced");
        assert_eq!(theme.name, "Second");
        assert_eq!(
            theme.text,
            okabe_ito().text,
            "the replacement merged with the document it replaced"
        );
        #[cfg(feature = "persistence")]
        {
            assert_eq!(registry.imported.len(), 1);
            assert_eq!(registry.imported[0].file_name, "my-dark.json");
        }
    }

    #[test]
    fn a_failed_reimport_preserves_the_previous_valid_document() {
        let mut registry = ThemeRegistry::built_in();
        registry
            .import("my-dark.toml", dark("Kept").as_bytes())
            .expect("a valid document");
        let before = registry
            .select(Appearance::Dark, &id("my-dark"))
            .expect("imported")
            .clone();
        #[cfg(feature = "persistence")]
        let stored_before = registry.imported.clone();

        for (file_name, bytes) in [
            (
                "my-dark.toml",
                "format = \"orcvs-theme\"\nversion = 1\nname = \"Broken\"\n\
                 inherits = \"okabe-ito\"\n[style]\n\"grid.border.width\" = 5\n"
                    .as_bytes(),
            ),
            ("my-dark.json", b"{\"format\": \"orcvs-theme\"".as_slice()),
            ("my-dark.yaml", b"\xFF\xFE".as_slice()),
        ] {
            let error = registry
                .import(file_name, bytes)
                .expect_err("an invalid document");
            assert!(error.contains(file_name), "{error}");
            assert_eq!(
                registry.select(Appearance::Dark, &id("my-dark")),
                Ok(&before),
                "a failed reimport of {file_name} changed the Theme"
            );
            #[cfg(feature = "persistence")]
            assert_eq!(registry.imported, stored_before);
            assert!(notices_mention(&registry, file_name));
        }
        assert!(notices_mention(&registry, "grid.border.width"));
    }

    #[test]
    fn a_web_import_refuses_reserved_identities_and_names_that_are_not_theme_files() {
        let mut registry = ThemeRegistry::built_in();
        for (file_name, document) in [
            ("okabe-ito.toml", dark("Impostor")),
            ("orcvs-light.json", dark_json("Impostor")),
        ] {
            let error = registry
                .import(file_name, document.as_bytes())
                .expect_err("a reserved identity");
            assert!(error.contains("reserved"), "{error}");
        }
        assert_eq!(
            registry.select(Appearance::Dark, &OKABE_ITO_IDENTITY),
            Ok(&okabe_ito())
        );

        for file_name in ["theme.txt", ".toml"] {
            registry
                .import(file_name, dark("Nameless").as_bytes())
                .expect_err("not a Theme file name");
        }
        #[cfg(feature = "persistence")]
        assert!(registry.imported.is_empty());
        assert!(registry.custom.is_empty());
    }

    #[test]
    fn a_theme_below_the_contrast_floor_is_loaded_and_its_report_shown() {
        let mut registry = ThemeRegistry::built_in();
        let unreadable = "format = \"orcvs-theme\"\nversion = 1\nname = \"Dim\"\n\
                          inherits = \"okabe-ito\"\n[style]\n\"source.ordinary\" = \"#050505\"\n";
        registry
            .import("dim.toml", unreadable.as_bytes())
            .expect("a contrast failure never refuses a Theme");

        assert!(registry.select(Appearance::Dark, &id("dim")).is_ok());
        assert!(
            registry
                .notice_list()
                .iter()
                .any(|notice| notice.contains("\"dim\"")
                    && notice.contains("contrast")
                    && notice.contains("Ordinary")),
            "{:?}",
            registry.notice_list()
        );
    }

    #[test]
    fn a_theme_that_clears_the_contrast_floor_raises_no_notice() {
        let mut registry = ThemeRegistry::built_in();
        registry
            .import("copy.toml", dark("Copy").as_bytes())
            .expect("a valid document");
        assert!(
            registry.notice_list().is_empty(),
            "{:?}",
            registry.notice_list()
        );
    }

    #[test]
    fn a_light_custom_theme_selected_for_the_dark_slot_falls_back() {
        let mut registry = ThemeRegistry::built_in();
        let light = "format = \"orcvs-theme\"\nversion = 1\nname = \"Paper\"\n\
                     inherits = \"orcvs-light\"\n";
        registry
            .import("paper.toml", light.as_bytes())
            .expect("a valid document");

        assert_eq!(
            registry.select(Appearance::Dark, &id("paper")),
            Err(Unavailable::WrongAppearance(Appearance::Light))
        );
        assert!(registry.select(Appearance::Light, &id("paper")).is_ok());
        let themes = SelectedThemes::new(
            registry,
            ThemeSelection::new(id("paper"), ORCVS_LIGHT_IDENTITY),
        );
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert!(
            themes
                .notices()
                .any(|notice| notice.contains("\"paper\"") && notice.contains("light Theme")),
            "{:?}",
            themes.notice_list()
        );
    }

    #[test]
    fn files_in_one_drop_that_share_a_stem_are_all_refused_and_named() {
        let mut registry = ThemeRegistry::built_in();
        let dropped = [
            "dup.toml",
            "solo.toml",
            "dup.json",
            "readme.txt",
            "dup.YAML",
        ]
        .map(str::to_owned)
        .to_vec();
        let mut reversed = dropped.clone();
        reversed.reverse();

        let accepted = registry.refuse_conflicting_drops(dropped);
        assert_eq!(accepted, ["solo.toml", "readme.txt"]);
        let notices = registry.notice_list();
        assert_eq!(notices.len(), 1, "{notices:?}");
        assert!(
            notices[0].contains("\"dup\"") && notices[0].contains("dup.YAML, dup.json, dup.toml"),
            "{notices:?}"
        );

        let mut other = ThemeRegistry::built_in();
        let accepted = other.refuse_conflicting_drops(reversed);
        assert_eq!(accepted, ["readme.txt", "solo.toml"]);
        assert_eq!(
            other.notice_list(),
            notices,
            "drop order chose a different outcome"
        );
    }

    #[test]
    fn a_conflicting_drop_leaves_the_previous_document_of_that_identity() {
        let mut registry = ThemeRegistry::built_in();
        registry
            .import("dup.toml", dark("Kept").as_bytes())
            .expect("a valid document");
        let accepted =
            registry.refuse_conflicting_drops(["dup.toml", "dup.json"].map(str::to_owned).to_vec());
        assert!(accepted.is_empty());
        assert_eq!(
            registry
                .select(Appearance::Dark, &id("dup"))
                .map(|theme| theme.name.as_str()),
            Ok("Kept")
        );
    }

    #[cfg(not(feature = "persistence"))]
    #[test]
    fn without_persistence_imported_documents_are_session_only() {
        /// Storage holding the key a persistence build would restore from.
        struct Holding;
        impl eframe::Storage for Holding {
            fn get_string(&self, _key: &str) -> Option<String> {
                Some("[(\"my-dark.toml\", \"format = \\\"orcvs-theme\\\"\")]".to_owned())
            }
            fn set_string(&mut self, _key: &str, _value: String) {}
            fn remove_string(&mut self, _key: &str) {}
            fn flush(&mut self) {}
        }

        let mut session = ThemeRegistry::web_start(Some(&Holding));
        session
            .import("my-dark.toml", dark("Session").as_bytes())
            .expect("a valid document");
        assert!(session.select(Appearance::Dark, &id("my-dark")).is_ok());

        let restarted = ThemeRegistry::web_start(Some(&Holding));
        assert_eq!(
            restarted.select(Appearance::Dark, &id("my-dark")),
            Err(Unavailable::Missing)
        );
        assert!(restarted.notice_list().is_empty());
    }

    #[cfg(feature = "persistence")]
    mod stored {
        use super::super::tests_support::{dark, id};
        use super::super::{MAX_STORED_IMPORTED_BYTES, ThemeRegistry, Unavailable};
        use super::notices_mention;
        use crate::persistence::{
            IMPORTED_THEMES_KEY, IMPORTED_THEMES_REFUSED_KEY, InMemoryStorage,
        };
        use crate::theme::Appearance;

        #[test]
        fn imported_documents_survive_a_restart_and_reload_from_their_source() {
            let mut storage = InMemoryStorage::default();
            let mut session = ThemeRegistry::web_start(Some(&storage));
            session
                .import("my-dark.toml", dark("Stored").as_bytes())
                .expect("a valid document");
            session.store_imported(&mut storage);
            assert!(session.notice_list().is_empty());

            let stored = eframe::Storage::get_string(&storage, IMPORTED_THEMES_KEY)
                .expect("the imported documents were stored");
            assert!(
                stored.contains("name = \\\"Stored\\\""),
                "storage holds the source document, not resolved values: {stored}"
            );

            let restarted = ThemeRegistry::web_start(Some(&storage));
            let theme = restarted
                .select(Appearance::Dark, &id("my-dark"))
                .expect("restored");
            assert_eq!(theme.name, "Stored");
        }

        #[test]
        fn a_replacement_is_what_the_next_restart_restores() {
            let mut storage = InMemoryStorage::default();
            let mut session = ThemeRegistry::web_start(Some(&storage));
            session
                .import("my-dark.toml", dark("First").as_bytes())
                .expect("a valid document");
            session.store_imported(&mut storage);
            session
                .import("my-dark.toml", dark("Second").as_bytes())
                .expect("a valid document");
            session
                .import("my-dark.toml", b"not a document")
                .expect_err("an invalid document");
            session.store_imported(&mut storage);

            let restarted = ThemeRegistry::web_start(Some(&storage));
            assert_eq!(
                restarted
                    .select(Appearance::Dark, &id("my-dark"))
                    .map(|theme| theme.name.as_str()),
                Ok("Second")
            );
        }

        /// `documents` stored the way `store_imported` stores them.
        fn storage_holding(documents: &[(&str, &str)]) -> InMemoryStorage {
            let mut storage = InMemoryStorage::default();
            eframe::Storage::set_string(
                &mut storage,
                IMPORTED_THEMES_KEY,
                serde_json::to_string(documents).expect("string pairs"),
            );
            storage
        }

        #[test]
        fn a_stored_document_that_no_longer_loads_falls_back_and_is_kept_for_the_next_save() {
            // A malformed document, and a name with no Theme suffix at all.
            let mut storage =
                storage_holding(&[("my-dark.toml", "version = 7"), ("notes", "no suffix")]);
            let stored = eframe::Storage::get_string(&storage, IMPORTED_THEMES_KEY);

            let mut restarted = ThemeRegistry::web_start(Some(&storage));
            assert!(matches!(
                restarted.select(Appearance::Dark, &id("my-dark")),
                Err(Unavailable::Refused(reason)) if reason.contains("my-dark.toml")
            ));
            assert!(notices_mention(&restarted, "my-dark.toml"));
            assert!(notices_mention(&restarted, "notes"));

            restarted.store_imported(&mut storage);
            assert_eq!(
                eframe::Storage::get_string(&storage, IMPORTED_THEMES_KEY),
                stored,
                "a save with nothing new imported rewrote storage"
            );

            restarted
                .import("other.toml", dark("Other").as_bytes())
                .expect("a valid document");
            restarted.store_imported(&mut storage);
            let again = ThemeRegistry::web_start(Some(&storage));
            assert!(again.select(Appearance::Dark, &id("other")).is_ok());
            assert!(
                again.refused.contains_key(&id("my-dark")),
                "the malformed document was dropped by a later save"
            );
            let names: Vec<&str> = again
                .imported
                .iter()
                .map(|imported| imported.file_name.as_str())
                .collect();
            assert_eq!(names, ["my-dark.toml", "notes", "other.toml"]);
        }

        #[test]
        fn an_undecodable_stored_value_is_moved_aside_before_anything_writes_over_it() {
            let mut storage = InMemoryStorage::default();
            eframe::Storage::set_string(
                &mut storage,
                IMPORTED_THEMES_KEY,
                "not a list of documents".to_owned(),
            );

            let mut restarted = ThemeRegistry::web_start(Some(&storage));
            assert!(
                notices_mention(&restarted, IMPORTED_THEMES_REFUSED_KEY),
                "{:?}",
                restarted.notice_list()
            );
            restarted
                .import("mine.toml", dark("Mine").as_bytes())
                .expect("a valid document");
            restarted.store_imported(&mut storage);

            assert_eq!(
                eframe::Storage::get_string(&storage, IMPORTED_THEMES_REFUSED_KEY).as_deref(),
                Some("not a list of documents"),
                "the undecodable value was lost"
            );
            let again = ThemeRegistry::web_start(Some(&storage));
            assert!(again.select(Appearance::Dark, &id("mine")).is_ok());
        }

        #[test]
        fn an_undecodable_stored_value_leaves_its_key_at_the_first_save_and_is_reported_once() {
            let mut storage = InMemoryStorage::default();
            eframe::Storage::set_string(
                &mut storage,
                IMPORTED_THEMES_KEY,
                "not a list of documents".to_owned(),
            );

            // A session that imports nothing still moves the value aside.
            let mut restarted = ThemeRegistry::web_start(Some(&storage));
            restarted.store_imported(&mut storage);
            assert_eq!(
                eframe::Storage::get_string(&storage, IMPORTED_THEMES_REFUSED_KEY).as_deref(),
                Some("not a list of documents")
            );
            assert_ne!(
                eframe::Storage::get_string(&storage, IMPORTED_THEMES_KEY).as_deref(),
                Some("not a list of documents"),
                "the undecodable value stayed under its key"
            );

            let again = ThemeRegistry::web_start(Some(&storage));
            assert!(
                !notices_mention(&again, IMPORTED_THEMES_REFUSED_KEY),
                "the next start reported the same value again: {:?}",
                again.notice_list()
            );
        }

        #[test]
        fn an_undecodable_stored_value_stays_under_its_key_until_its_copy_is_kept() {
            /// Storage whose quota is spent, as a browser's local storage is
            /// when full: replacing an existing value still succeeds, while a
            /// new key is not kept — until space is freed.
            #[derive(Default)]
            struct QuotaSpent {
                entries: std::collections::BTreeMap<String, String>,
                full: bool,
            }
            impl eframe::Storage for QuotaSpent {
                fn get_string(&self, key: &str) -> Option<String> {
                    self.entries.get(key).cloned()
                }
                fn set_string(&mut self, key: &str, value: String) {
                    if !self.full || self.entries.contains_key(key) {
                        self.entries.insert(key.to_owned(), value);
                    }
                }
                fn remove_string(&mut self, key: &str) {
                    self.entries.remove(key);
                }
                fn flush(&mut self) {}
            }

            let mut storage = QuotaSpent::default();
            eframe::Storage::set_string(
                &mut storage,
                IMPORTED_THEMES_KEY,
                "not a list of documents".to_owned(),
            );
            storage.full = true;

            let mut restarted = ThemeRegistry::web_start(Some(&storage));
            restarted
                .import("mine.toml", dark("Mine").as_bytes())
                .expect("a valid document");
            restarted.store_imported(&mut storage);
            restarted.store_imported(&mut storage);
            assert_eq!(
                eframe::Storage::get_string(&storage, IMPORTED_THEMES_KEY).as_deref(),
                Some("not a list of documents"),
                "the undecodable value was overwritten before its copy was kept"
            );
            assert!(
                notices_mention(&restarted, "Could not store"),
                "{:?}",
                restarted.notice_list()
            );
            assert!(
                restarted.select(Appearance::Dark, &id("mine")).is_ok(),
                "the imported Theme stays usable for the session"
            );

            // Once space is freed, the next save moves the value aside and
            // stores the imported documents.
            storage.full = false;
            restarted.store_imported(&mut storage);
            assert_eq!(
                eframe::Storage::get_string(&storage, IMPORTED_THEMES_REFUSED_KEY).as_deref(),
                Some("not a list of documents")
            );
            let again = ThemeRegistry::web_start(Some(&storage));
            assert!(again.select(Appearance::Dark, &id("mine")).is_ok());
        }

        /// A valid dark document named `name`, padded just under the 1 MiB
        /// document limit: two fit the 2 MiB budget, three cannot.
        fn padded(name: &str) -> String {
            let mut text = dark(name);
            text.push('#');
            text.push_str(&"x".repeat(900 * 1024));
            text.push('\n');
            text
        }

        #[test]
        fn an_import_past_the_storage_budget_is_kept_for_the_session_and_not_stored() {
            let mut storage = InMemoryStorage::default();
            let mut session = ThemeRegistry::web_start(Some(&storage));
            for name in ["one", "two"] {
                session
                    .import(&format!("{name}.toml"), padded(name).as_bytes())
                    .expect("a valid document");
            }
            session.store_imported(&mut storage);
            let before = eframe::Storage::get_string(&storage, IMPORTED_THEMES_KEY)
                .expect("the first two were stored");
            assert!(before.len() <= MAX_STORED_IMPORTED_BYTES);

            session
                .import("three.toml", padded("three").as_bytes())
                .expect("a valid document");
            assert!(
                session.select(Appearance::Dark, &id("three")).is_ok(),
                "an import past the budget stays usable for the session"
            );
            assert!(
                notices_mention(&session, "three.toml")
                    && notices_mention(&session, "session only"),
                "{:?}",
                session.notice_list()
            );
            session.store_imported(&mut storage);
            assert_eq!(
                eframe::Storage::get_string(&storage, IMPORTED_THEMES_KEY).as_deref(),
                Some(before.as_str()),
                "storage was written past its budget"
            );

            let restarted = ThemeRegistry::web_start(Some(&storage));
            assert!(restarted.select(Appearance::Dark, &id("one")).is_ok());
            assert!(restarted.select(Appearance::Dark, &id("three")).is_err());
        }

        #[test]
        fn a_replacement_past_the_storage_budget_says_the_stored_copy_returns_on_restart() {
            let mut storage = InMemoryStorage::default();
            let mut session = ThemeRegistry::web_start(Some(&storage));
            for name in ["one", "two"] {
                session
                    .import(&format!("{name}.toml"), padded(name).as_bytes())
                    .expect("a valid document");
            }
            session
                .import("foo.toml", dark("Stored").as_bytes())
                .expect("a valid document");
            session.store_imported(&mut storage);

            session
                .import("foo.toml", padded("Larger").as_bytes())
                .expect("a valid document");
            assert_eq!(
                session
                    .select(Appearance::Dark, &id("foo"))
                    .map(|theme| theme.name.as_str()),
                Ok("Larger"),
                "the larger import is used for the session"
            );
            assert!(
                notices_mention(&session, "the stored foo.toml")
                    && notices_mention(&session, "next start"),
                "the notice does not say the stored copy comes back: {:?}",
                session.notice_list()
            );

            // What the notice promises is what a restart does.
            session.store_imported(&mut storage);
            let restarted = ThemeRegistry::web_start(Some(&storage));
            assert_eq!(
                restarted
                    .select(Appearance::Dark, &id("foo"))
                    .map(|theme| theme.name.as_str()),
                Ok("Stored")
            );
        }

        #[test]
        fn a_storage_write_that_is_not_kept_is_reported_once_and_retried_only_on_change() {
            /// Storage that accepts every write and keeps none of them, as a
            /// browser does when local storage is full.
            struct Full;
            impl eframe::Storage for Full {
                fn get_string(&self, _key: &str) -> Option<String> {
                    None
                }
                fn set_string(&mut self, _key: &str, _value: String) {}
                fn remove_string(&mut self, _key: &str) {}
                fn flush(&mut self) {}
            }

            let mut registry = ThemeRegistry::web_start(None);
            registry
                .import("my-dark.toml", dark("Unstored").as_bytes())
                .expect("a valid document");
            registry.store_imported(&mut Full);
            registry.store_imported(&mut Full);

            let failures = registry
                .notice_list()
                .iter()
                .filter(|notice| notice.contains("Could not store"))
                .count();
            assert_eq!(failures, 1, "{:?}", registry.notice_list());
            assert!(
                registry.select(Appearance::Dark, &id("my-dark")).is_ok(),
                "the imported Theme stays usable for the session"
            );

            // No retry at the next autosave: nothing has changed.
            let mut storage = InMemoryStorage::default();
            registry.store_imported(&mut storage);
            assert_eq!(
                eframe::Storage::get_string(&storage, IMPORTED_THEMES_KEY),
                None
            );

            // The imported set changes, so the next save writes it.
            registry
                .import("other.toml", dark("Other").as_bytes())
                .expect("a valid document");
            registry.store_imported(&mut storage);
            let restarted = ThemeRegistry::web_start(Some(&storage));
            assert!(restarted.select(Appearance::Dark, &id("my-dark")).is_ok());
            assert!(restarted.select(Appearance::Dark, &id("other")).is_ok());
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod discovery_tests {
    use std::path::{Path, PathBuf};

    use super::ThemeRegistry;
    use super::native::{Candidate, group};
    use super::tests_support::{TempDir, dark, dark_json, id};
    use crate::theme::{
        Appearance, OKABE_ITO_IDENTITY, ORCVS_LIGHT_IDENTITY, ThemeIdentity, okabe_ito, orcvs_light,
    };
    use crate::theme_document::MAX_DOCUMENT_BYTES;
    use crate::theme_selection::{SelectedThemes, ThemeSelection};

    fn names(registry: &ThemeRegistry) -> Vec<&str> {
        registry.custom.keys().map(ThemeIdentity::as_str).collect()
    }

    #[test]
    fn a_missing_directory_means_no_custom_themes_and_nothing_to_report() {
        let dir = TempDir::new();
        let registry = ThemeRegistry::discover(&dir.path().join("themes"));
        assert!(registry.custom.is_empty());
        assert!(registry.notice_list().is_empty());
    }

    #[test]
    fn an_unreadable_directory_is_reported_and_startup_continues() {
        let dir = TempDir::new();
        let not_a_directory = dir.write("themes", "a file where the directory belongs");
        let registry = ThemeRegistry::discover(&not_a_directory);
        assert!(registry.custom.is_empty());
        assert_eq!(
            registry.notice_list().len(),
            1,
            "{:?}",
            registry.notice_list()
        );
        assert!(registry.notice_list()[0].contains("Theme directory"));
        assert_eq!(
            registry.select(Appearance::Dark, &OKABE_ITO_IDENTITY),
            Ok(&okabe_ito())
        );
    }

    #[test]
    fn only_direct_children_with_a_theme_suffix_load_and_other_files_are_ignored() {
        let dir = TempDir::new();
        dir.write("lower.toml", &dark("Lower"));
        dir.write("Upper.JSON", &dark_json("Upper"));
        dir.write(
            "short.Yml",
            "format: orcvs-theme\nversion: 1\nname: Short\ninherits: okabe-ito\n",
        );
        dir.write(
            "long.yaml",
            "format: orcvs-theme\nversion: 1\nname: Long\ninherits: okabe-ito\n",
        );
        dir.write("notes.txt", "not a Theme");
        dir.write("backup.toml.bak", &dark("Backup"));
        dir.write("README", "not a Theme");
        std::fs::create_dir(dir.path().join("nested")).expect("a subdirectory");
        std::fs::write(dir.path().join("nested").join("deep.toml"), dark("Deep"))
            .expect("a nested file");
        std::fs::create_dir(dir.path().join("folder.toml")).expect("a directory with a suffix");

        let registry = ThemeRegistry::discover(dir.path());
        assert_eq!(names(&registry), ["Upper", "long", "lower", "short"]);
        assert!(
            registry.notice_list().is_empty(),
            "{:?}",
            registry.notice_list()
        );
    }

    #[test]
    fn an_empty_stem_is_refused_and_reported() {
        let dir = TempDir::new();
        dir.write(".toml", &dark("Nameless"));
        let registry = ThemeRegistry::discover(dir.path());
        assert!(registry.custom.is_empty());
        assert!(
            registry.notice_list()[0].contains("stem"),
            "{:?}",
            registry.notice_list()
        );
    }

    #[test]
    fn the_byte_limit_applies_to_the_bytes_actually_read() {
        let dir = TempDir::new();
        let pad = |len: usize| {
            let mut text = dark("Padded");
            text.push('#');
            text.push_str(&"x".repeat(len - text.len() - 1));
            text.push('\n');
            assert_eq!(text.len(), len);
            text
        };
        dir.write("at-limit.toml", &pad(MAX_DOCUMENT_BYTES));
        dir.write("over-limit.toml", &pad(MAX_DOCUMENT_BYTES + 1));

        let registry = ThemeRegistry::discover(dir.path());
        assert_eq!(names(&registry), ["at-limit"]);
        assert!(
            registry
                .notice_list()
                .iter()
                .any(|notice| notice.contains("over-limit.toml")
                    && notice.contains(&format!("{MAX_DOCUMENT_BYTES}-byte limit"))),
            "{:?}",
            registry.notice_list()
        );
    }

    #[test]
    fn a_reserved_identity_is_refused_and_never_replaces_the_built_in() {
        let dir = TempDir::new();
        let impostor = "format = \"orcvs-theme\"\nversion = 1\nname = \"Impostor\"\n\
                        inherits = \"okabe-ito\"\n[style]\ntext = \"#FF0000\"\n";
        dir.write("okabe-ito.toml", impostor);
        dir.write("orcvs-light.json", &dark_json("Impostor"));

        let mut registry = ThemeRegistry::discover(dir.path());
        assert!(registry.custom.is_empty());
        assert_eq!(
            registry.notice_list().len(),
            2,
            "{:?}",
            registry.notice_list()
        );
        assert!(
            registry
                .notice_list()
                .iter()
                .all(|notice| notice.contains("reserved"))
        );
        assert_eq!(
            registry.select(Appearance::Dark, &OKABE_ITO_IDENTITY),
            Ok(&okabe_ito())
        );
        assert_eq!(
            registry.select(Appearance::Light, &ORCVS_LIGHT_IDENTITY),
            Ok(&orcvs_light())
        );
        registry.dismiss_notices();
        let themes = SelectedThemes::new(registry, ThemeSelection::default());
        assert!(
            themes.notice_list().is_empty(),
            "{:?}",
            themes.notice_list()
        );
    }

    #[test]
    fn a_display_label_change_keeps_the_selection_and_a_rename_changes_the_identity() {
        let dir = TempDir::new();
        dir.write("my-dark.toml", &dark("Before"));
        let first = ThemeRegistry::discover(dir.path());
        assert_eq!(
            first
                .select(Appearance::Dark, &id("my-dark"))
                .map(|theme| theme.name.as_str()),
            Ok("Before")
        );

        dir.write("my-dark.toml", &dark("After"));
        let relabelled = ThemeRegistry::discover(dir.path());
        assert_eq!(
            relabelled
                .select(Appearance::Dark, &id("my-dark"))
                .map(|theme| theme.name.as_str()),
            Ok("After"),
            "a new display label lost the selection"
        );

        std::fs::rename(
            dir.path().join("my-dark.toml"),
            dir.path().join("renamed.toml"),
        )
        .expect("a rename");
        let renamed = ThemeRegistry::discover(dir.path());
        assert_eq!(
            renamed.select(Appearance::Dark, &id("my-dark")),
            Err(super::Unavailable::Missing)
        );
        assert_eq!(
            renamed
                .select(Appearance::Dark, &id("renamed"))
                .map(|theme| theme.name.as_str()),
            Ok("After")
        );
    }

    #[test]
    fn duplicate_identities_conflict_whatever_the_enumeration_order() {
        let candidate = |file_name: &str| Candidate {
            path: PathBuf::from("/themes").join(file_name),
            file_name: file_name.to_owned(),
            identity: id(super::stem(file_name).expect("a Theme file")),
            loadable: true,
        };
        let forward = vec![
            candidate("dup.toml"),
            candidate("solo.toml"),
            candidate("dup.json"),
            candidate("dup.YAML"),
        ];
        let mut backward = forward.clone();
        backward.reverse();

        let (unique, conflicts) = group(forward);
        assert_eq!(group(backward), (unique.clone(), conflicts.clone()));
        assert_eq!(unique.keys().collect::<Vec<_>>(), [&id("solo")]);
        assert_eq!(
            conflicts[&id("dup")],
            [
                Path::new("/themes/dup.YAML"),
                Path::new("/themes/dup.json"),
                Path::new("/themes/dup.toml"),
            ]
        );
    }

    #[test]
    fn every_file_of_a_duplicate_identity_is_refused_and_reported_by_path() {
        let dir = TempDir::new();
        let toml = dir.write("dup.toml", &dark("Toml"));
        let json = dir.write("dup.json", &dark_json("Json"));
        dir.write("solo.toml", &dark("Solo"));

        let registry = ThemeRegistry::discover(dir.path());
        assert_eq!(names(&registry), ["solo"]);
        let notices = registry.notice_list();
        let conflict = notices
            .iter()
            .find(|notice| notice.contains("\"dup\""))
            .expect("the conflict is reported");
        assert!(conflict.contains(&toml.display().to_string()), "{conflict}");
        assert!(conflict.contains(&json.display().to_string()), "{conflict}");
        assert!(matches!(
            registry.select(Appearance::Dark, &id("dup")),
            Err(super::Unavailable::Refused(reason)) if reason.contains("dup.json")
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_directory_symlink_is_never_traversed_and_a_file_symlink_is_followed() {
        let outside = TempDir::new();
        let target = outside.write("target.toml", &dark("Linked"));
        std::fs::create_dir(outside.path().join("elsewhere")).expect("a directory");
        std::fs::write(
            outside.path().join("elsewhere").join("inner.toml"),
            dark("Inner"),
        )
        .expect("a file");

        let dir = TempDir::new();
        std::os::unix::fs::symlink(&target, dir.path().join("alias.toml")).expect("a file link");
        std::os::unix::fs::symlink(outside.path().join("elsewhere"), dir.path().join("linked"))
            .expect("a directory link");
        std::os::unix::fs::symlink(
            outside.path().join("elsewhere"),
            dir.path().join("linked.toml"),
        )
        .expect("a directory link with a Theme suffix");

        let registry = ThemeRegistry::discover(dir.path());
        assert_eq!(names(&registry), ["alias"]);
        assert!(
            registry.notice_list().is_empty(),
            "{:?}",
            registry.notice_list()
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_is_reported_and_the_others_still_load() {
        use std::os::unix::fs::PermissionsExt as _;

        let dir = TempDir::new();
        dir.write("fine.toml", &dark("Fine"));
        std::os::unix::fs::symlink(dir.path().join("absent"), dir.path().join("dangling.toml"))
            .expect("a dangling link");
        let locked = dir.write("locked.toml", &dark("Locked"));
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
            .expect("permissions");
        // A superuser reads a mode-0 file anyway; only the dangling link is
        // then unreadable.
        let locked_is_unreadable = std::fs::read(&locked).is_err();

        let registry = ThemeRegistry::discover(dir.path());
        assert!(names(&registry).contains(&"fine"));
        assert!(
            registry
                .notice_list()
                .iter()
                .any(|n| n.contains("dangling.toml")),
            "{:?}",
            registry.notice_list()
        );
        if locked_is_unreadable {
            assert_eq!(names(&registry), ["fine"]);
            assert!(
                registry
                    .notice_list()
                    .iter()
                    .any(|n| n.contains("locked.toml")),
                "{:?}",
                registry.notice_list()
            );
            assert!(matches!(
                registry.select(Appearance::Dark, &id("locked")),
                Err(super::Unavailable::Refused(_))
            ));
        }
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644))
            .expect("permissions");
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_still_conflicts_with_a_readable_one_of_the_same_stem() {
        let dir = TempDir::new();
        let toml = dir.write("ocean.toml", &dark("Ocean"));
        let dangling = dir.path().join("ocean.yaml");
        std::os::unix::fs::symlink(dir.path().join("absent"), &dangling).expect("a dangling link");

        let registry = ThemeRegistry::discover(dir.path());
        assert!(names(&registry).is_empty(), "{:?}", names(&registry));
        let notices = registry.notice_list();
        assert!(
            notices
                .iter()
                .any(|n| n.contains("Could not read") && n.contains("ocean.yaml")),
            "the unreadable file is still reported: {notices:?}"
        );
        let conflict = notices
            .iter()
            .find(|notice| notice.contains("\"ocean\""))
            .expect("the conflict is reported");
        assert!(conflict.contains(&toml.display().to_string()), "{conflict}");
        assert!(
            conflict.contains(&dangling.display().to_string()),
            "{conflict}"
        );
        assert!(matches!(
            registry.select(Appearance::Dark, &id("ocean")),
            Err(super::Unavailable::Refused(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_without_a_stem_is_reported_once() {
        let dir = TempDir::new();
        let dangling = dir.path().join(".toml");
        std::os::unix::fs::symlink(dir.path().join("absent"), &dangling).expect("a dangling link");

        let registry = ThemeRegistry::discover(dir.path());
        let notices = registry.notice_list();
        assert_eq!(notices.len(), 1, "{notices:?}");
        assert!(notices[0].contains("Could not read"), "{notices:?}");
    }

    ///
    /// `schema.md`: "Unavailable, malformed, conflicted or appearance-mismatched
    /// selections keep their saved references and show the matching built-in
    /// fallback plus an error. Autosave must not overwrite that reference ...
    /// Repair restores the intended Theme when next loaded."
    ///
    /// Each launch is what `Console::start` and `Console::new` do: discover
    /// the directory, restore the saved selection, and resolve it into the
    /// Themes presented. Each save is what eframe's autosave calls.
    ///
    #[cfg(feature = "persistence")]
    mod selection {
        use super::super::tests_support::{TempDir, dark};
        use super::super::{ThemeRegistry, Unavailable};
        use crate::cursor_effects::CursorEffectSettings;
        use crate::persistence::{
            DARK_THEME_KEY, InMemoryStorage, LIGHT_THEME_KEY, edited_source, starting_source,
        };
        use crate::theme::{Appearance, okabe_ito};
        use crate::theme_selection::SelectedThemes;

        ///
        /// One launch followed by an autosave: the dark selection's
        /// resolution, the notices the launch raised, and what the save
        /// left in storage. The Theme presented for dark is the one the
        /// selection resolved to, or Okabe–Ito when it resolved to none.
        ///
        fn launch_and_save(
            storage: &mut InMemoryStorage,
            dir: &std::path::Path,
        ) -> (Result<String, Unavailable>, Vec<String>) {
            let start = starting_source(Some(storage));
            let registry = ThemeRegistry::discover(dir);
            let selected = registry
                .select(
                    Appearance::Dark,
                    start.theme_selection.identity(Appearance::Dark),
                )
                .map(|theme| theme.identity.as_str().to_owned());
            let themes = SelectedThemes::new(registry, start.theme_selection);
            let presented = themes.presented(Appearance::Dark);
            match &selected {
                Ok(identity) => assert_eq!(presented.identity.as_str(), identity),
                Err(_) => assert_eq!(*presented, okabe_ito()),
            }
            let mut persistence = start.persistence;
            persistence.save(
                storage,
                &edited_source(),
                CursorEffectSettings::default(),
                themes.selection(),
            );
            (selected, themes.notice_list())
        }

        fn saved_dark(storage: &InMemoryStorage) -> Option<String> {
            eframe::Storage::get_string(storage, DARK_THEME_KEY)
        }

        #[test]
        fn fallback_then_autosave_then_repair_then_restart_restores_the_selection() {
            let dir = TempDir::new();
            dir.write(
                "my-dark.toml",
                "format = \"orcvs-theme\"\nversion = 2\nname = \"Mine\"\ninherits = \"okabe-ito\"\n",
            );
            let mut storage = InMemoryStorage::default();
            eframe::Storage::set_string(&mut storage, DARK_THEME_KEY, "my-dark".to_owned());
            eframe::Storage::set_string(&mut storage, LIGHT_THEME_KEY, "orcvs-light".to_owned());

            let (selected, notices) = launch_and_save(&mut storage, dir.path());
            assert!(matches!(
                selected,
                Err(Unavailable::Refused(ref reason)) if reason.contains("my-dark.toml")
            ));
            assert!(
                notices.iter().any(|notice| notice.contains("\"my-dark\"")
                    && notice.contains("my-dark.toml")
                    && notice.contains("Okabe–Ito")),
                "{notices:?}"
            );
            assert_eq!(saved_dark(&storage).as_deref(), Some("my-dark"));

            // A second launch before the repair still keeps it.
            let (selected, _) = launch_and_save(&mut storage, dir.path());
            assert!(selected.is_err());
            assert_eq!(saved_dark(&storage).as_deref(), Some("my-dark"));

            dir.write("my-dark.toml", &dark("Mine"));
            let (selected, notices) = launch_and_save(&mut storage, dir.path());
            assert_eq!(selected, Ok("my-dark".to_owned()));
            assert!(notices.is_empty(), "{notices:?}");
        }

        #[test]
        fn a_missing_selected_file_falls_back_and_returns_when_the_file_does() {
            let dir = TempDir::new();
            let mut storage = InMemoryStorage::default();
            eframe::Storage::set_string(&mut storage, DARK_THEME_KEY, "my-dark".to_owned());

            let (selected, notices) = launch_and_save(&mut storage, dir.path());
            assert_eq!(selected, Err(Unavailable::Missing));
            assert!(
                notices.iter().any(|n| n.contains("\"my-dark\"")),
                "{notices:?}"
            );
            assert_eq!(saved_dark(&storage).as_deref(), Some("my-dark"));

            dir.write("my-dark.toml", &dark("Mine"));
            let (selected, _) = launch_and_save(&mut storage, dir.path());
            assert_eq!(selected, Ok("my-dark".to_owned()));
        }

        #[test]
        fn a_selected_conflicted_identity_falls_back_keeps_the_selection_and_recovers() {
            let dir = TempDir::new();
            dir.write("dup.toml", &dark("Toml"));
            let json = dir.write("dup.json", &super::dark_json("Json"));
            let mut storage = InMemoryStorage::default();
            eframe::Storage::set_string(&mut storage, DARK_THEME_KEY, "dup".to_owned());

            let (selected, notices) = launch_and_save(&mut storage, dir.path());
            assert!(matches!(
                selected,
                Err(Unavailable::Refused(ref reason)) if reason.contains("dup.json")
                    && reason.contains("dup.toml")
            ));
            assert!(
                notices
                    .iter()
                    .any(|n| n.contains("selected dark Theme \"dup\"")),
                "{notices:?}"
            );
            assert_eq!(saved_dark(&storage).as_deref(), Some("dup"));

            std::fs::remove_file(json).expect("resolving the conflict");
            let (selected, notices) = launch_and_save(&mut storage, dir.path());
            assert_eq!(selected, Ok("dup".to_owned()));
            assert!(notices.is_empty(), "{notices:?}");
        }
    }
}

#[cfg(test)]
pub(crate) mod tests_support {
    #[cfg(not(target_arch = "wasm32"))]
    use std::path::{Path, PathBuf};

    /// A custom Theme's identity, as a file named `stem` supplies it.
    pub(crate) fn id(stem: &str) -> crate::theme::ThemeIdentity {
        crate::theme::ThemeIdentity::from_stem(stem).expect("a non-empty stem")
    }

    ///
    /// A dark document, `my-dark.toml`, whose window, panel, input and Grid
    /// backgrounds differ from every other Theme's here, so a test can tell
    /// which Theme painted a frame.
    ///
    pub(crate) const MY_DARK: &str = "format = \"orcvs-theme\"\nversion = 1\n\
        name = \"My Dark\"\ninherits = \"okabe-ito\"\n[style]\n\
        \"window.background\" = \"#201030\"\n\"panel.background\" = \"#201030\"\n\
        \"grid.background\" = \"#100020\"\n\"input.background\" = \"#180828\"\n";

    /// A light document, `my-light.toml`, distinguishable the same way.
    pub(crate) const MY_LIGHT: &str = "format = \"orcvs-theme\"\nversion = 1\n\
        name = \"My Light\"\ninherits = \"orcvs-light\"\n[style]\n\
        \"window.background\" = \"#F4ECDC\"\n\"panel.background\" = \"#F4ECDC\"\n\
        \"grid.background\" = \"#FEF8EE\"\n\"input.background\" = \"#FAF2E4\"\n";

    ///
    /// The built-ins with [`MY_DARK`] and [`MY_LIGHT`] loaded beside them,
    /// through the same import a dropped file takes.
    ///
    pub(crate) fn with_my_themes() -> super::ThemeRegistry {
        let mut registry = super::ThemeRegistry::built_in();
        registry
            .import("my-dark.toml", MY_DARK.as_bytes())
            .expect("a valid document");
        registry
            .import("my-light.toml", MY_LIGHT.as_bytes())
            .expect("a valid document");
        registry
    }

    /// The Theme [`MY_DARK`] resolves to.
    pub(crate) fn my_dark() -> crate::theme::Theme {
        with_my_themes()
            .select(crate::theme::Appearance::Dark, &id("my-dark"))
            .expect("loaded")
            .clone()
    }

    /// The Theme [`MY_LIGHT`] resolves to.
    pub(crate) fn my_light() -> crate::theme::Theme {
        with_my_themes()
            .select(crate::theme::Appearance::Light, &id("my-light"))
            .expect("loaded")
            .clone()
    }

    /// A minimal valid dark document labelled `name`.
    pub(crate) fn dark(name: &str) -> String {
        format!(
            "format = \"orcvs-theme\"\nversion = 1\nname = \"{name}\"\ninherits = \"okabe-ito\"\n"
        )
    }

    /// The same document in JSON.
    pub(crate) fn dark_json(name: &str) -> String {
        format!(
            "{{\"format\": \"orcvs-theme\", \"version\": 1, \"name\": \"{name}\", \
             \"inherits\": \"okabe-ito\"}}"
        )
    }

    ///
    /// A directory of its own under the system temporary directory, removed
    /// on drop: the Theme directory a test discovers, never the viewer's
    /// `~/.orcvs/themes`.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) struct TempDir(PathBuf);

    #[cfg(not(target_arch = "wasm32"))]
    impl TempDir {
        pub(crate) fn new() -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "orcvs-theme-registry-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("a unique temp directory");
            Self(dir)
        }

        pub(crate) fn path(&self) -> &Path {
            &self.0
        }

        /// Writes `contents` to `name` in this directory, returning its path.
        pub(crate) fn write(&self, name: &str, contents: &str) -> PathBuf {
            let path = self.0.join(name);
            std::fs::write(&path, contents).expect("a test file");
            path
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
