//!
//! The Themes a console can select: the built-ins, the custom Themes loaded
//! from Theme documents, and what went wrong loading them.
//! `.scratch/theming/issues/07`, following `schema.md`'s "Discovery,
//! persistence and failures".
//!
//! Every custom document reaches the registry through one path —
//! [`crate::theme_document::decode`], then [`crate::theme::resolve`] against
//! the built-ins under the identity its file name's stem supplies. Built-in
//! identities are reserved by `resolve` itself.
//!
//! On native, [`ThemeRegistry::start`] scans `~/.orcvs/themes/` once, while
//! `Console::start` runs and before any frame is shown; `Console::new` takes
//! the registry as a parameter, so tests build their own. The files are
//! authoritative: they are read at every launch, never cached in application
//! storage, and never watched. The web reads no Theme files: it has the
//! built-ins alone.
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
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
const SUFFIXES: [&str; 4] = [".toml", ".json", ".yaml", ".yml"];

///
/// The Theme identity a file name supplies: the name without its Theme
/// document suffix. `None` when the name carries none of [`SUFFIXES`].
/// Matching is case-insensitive; the identity keeps its own case.
///
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
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
    /// Load and contrast messages, in the order they arose, until the
    /// viewer dismisses them.
    notices: Vec<String>,
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
        }
    }

    ///
    /// The registry the shipped console starts with, built by
    /// `Console::start` before `Console::new`. Native reads the canonical
    /// Theme directory, `~/.orcvs/themes/`, and never application storage;
    /// the web has the built-ins alone.
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
    pub(crate) fn start(_storage: Option<&dyn eframe::Storage>) -> Self {
        Self::built_in()
    }

    ///
    /// Records a notice for the viewer and reports it on the console's error
    /// channel.
    ///
    pub(crate) fn notice(&mut self, message: String) {
        crate::report::error!("{message}");
        self.notices.push(message);
    }

    /// Every load and contrast notice the viewer has not dismissed,
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
    /// Decodes and resolves one document under the identity its file name
    /// supplies. Pure: the registry is not touched, so a refusal leaves it
    /// exactly as it was.
    ///
    #[cfg_attr(
        all(target_arch = "wasm32", not(test)),
        expect(
            dead_code,
            reason = "only native discovery loads a Theme document; the web has the built-ins alone"
        )
    )]
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
    #[cfg_attr(
        all(target_arch = "wasm32", not(test)),
        expect(
            dead_code,
            reason = "only native discovery loads a Theme document; the web has the built-ins alone"
        )
    )]
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
#[cfg_attr(
    all(target_arch = "wasm32", not(test)),
    expect(
        dead_code,
        reason = "only native discovery loads a Theme document; the web has the built-ins alone"
    )
)]
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
        /// Why the entry cannot be loaded, if it cannot. Such an entry has
        /// already been reported, but still claims its identity: it conflicts
        /// with any other file of the same stem, and alone it refuses the
        /// identity with this reason.
        pub(super) unloadable: Option<String>,
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
        /// `dir` is reported and skipped; an unreadable file is reported and
        /// refuses its identity, along with every other file of its stem.
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
                // is reported here and still grouped, so it refuses its
                // identity and every other file of its stem.
                let unloadable = match std::fs::metadata(&path) {
                    Ok(metadata) if metadata.is_dir() => continue,
                    Ok(metadata) if metadata.is_file() => None,
                    Ok(_) => {
                        registry.notice(format!("{}: not a regular Theme file", path.display()));
                        Some(format!("{file_name}: not a regular Theme file"))
                    }
                    Err(error) => {
                        registry.notice(format!(
                            "Could not read the Theme file {}: {error}",
                            path.display()
                        ));
                        Some(format!("{file_name}: {error}"))
                    }
                };
                let Some(identity) = ThemeIdentity::from_stem(stem) else {
                    // An entry that cannot be loaded was reported above, and
                    // one without an identity conflicts with nothing.
                    if unloadable.is_none() {
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
                    unloadable,
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
                // An entry that cannot be loaded was reported when it was
                // found; it refuses its identity without a second notice.
                if let Some(reason) = candidate.unloadable {
                    if !identity.is_reserved() {
                        registry.refused.insert(identity, reason);
                    }
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

#[cfg(test)]
mod tests {
    use super::tests_support::{dark, id, load};
    use super::{ThemeRegistry, Unavailable, stem};
    use crate::theme::{
        Appearance, OKABE_ITO_IDENTITY, ORCVS_LIGHT_IDENTITY, okabe_ito, orcvs_light,
    };
    use crate::theme_selection::{SelectedThemes, ThemeSelection};

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

    #[test]
    fn a_theme_below_the_contrast_floor_is_loaded_and_its_report_shown() {
        let mut registry = ThemeRegistry::built_in();
        let unreadable = "format = \"orcvs-theme\"\nversion = 1\nname = \"Dim\"\n\
                          inherits = \"okabe-ito\"\n[style]\n\"source.ordinary\" = \"#050505\"\n";
        load(&mut registry, "dim.toml", unreadable.as_bytes());

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
        load(&mut registry, "copy.toml", dark("Copy").as_bytes());
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
        load(&mut registry, "paper.toml", light.as_bytes());

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
            unloadable: None,
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
    fn a_lone_unreadable_file_refuses_its_identity_with_the_file_problem() {
        let dir = TempDir::new();
        let dangling = dir.path().join("ocean.toml");
        std::os::unix::fs::symlink(dir.path().join("absent"), &dangling).expect("a dangling link");

        let registry = ThemeRegistry::discover(dir.path());
        let notices = registry.notice_list();
        assert_eq!(notices.len(), 1, "{notices:?}");
        assert!(matches!(
            registry.select(Appearance::Dark, &id("ocean")),
            Err(super::Unavailable::Refused(reason)) if reason.contains("ocean.toml")
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
    /// fallback plus an error. ... Repair restores the intended Theme when
    /// next loaded."
    ///
    /// The reference is `~/.orcvs/config.toml`'s `theme.dark`, which the
    /// console only reads, so it is kept by never being written. Each launch
    /// is what `Console::start` and `Console::new` do: discover the
    /// directory, read the settings, and resolve the selection into the
    /// Themes presented. Both are read from directories the test builds.
    ///
    mod selection {
        use super::super::tests_support::{TempDir, dark};
        use super::super::{ThemeRegistry, Unavailable};
        use crate::config::{CONFIG_FILE, Config};
        use crate::theme::{Appearance, okabe_ito};
        use crate::theme_selection::SelectedThemes;

        ///
        /// One launch: the dark selection's resolution and the notices the
        /// launch raised. The Theme presented for dark is the one the
        /// selection resolved to, or Okabe–Ito when it resolved to none.
        ///
        fn launch(
            settings: &TempDir,
            themes: &TempDir,
        ) -> (Result<String, Unavailable>, Vec<String>) {
            let config = Config::read(&settings.path().join(CONFIG_FILE));
            let registry = ThemeRegistry::discover(themes.path());
            let selected = registry
                .select(
                    Appearance::Dark,
                    config.theme_selection.identity(Appearance::Dark),
                )
                .map(|theme| theme.identity.as_str().to_owned());
            let themes = SelectedThemes::new(registry, config.theme_selection);
            let presented = themes.presented(Appearance::Dark);
            match &selected {
                Ok(identity) => assert_eq!(presented.identity.as_str(), identity),
                Err(_) => assert_eq!(*presented, okabe_ito()),
            }
            (selected, themes.notice_list())
        }

        /// A settings directory whose `config.toml` selects `identity` for dark.
        fn selecting_dark(identity: &str) -> TempDir {
            let settings = TempDir::new();
            settings.write(CONFIG_FILE, &format!("[theme]\ndark = \"{identity}\"\n"));
            settings
        }

        #[test]
        fn a_refused_selected_file_falls_back_and_is_restored_by_its_repair() {
            let settings = selecting_dark("my-dark");
            let dir = TempDir::new();
            dir.write(
                "my-dark.toml",
                "format = \"orcvs-theme\"\nversion = 2\nname = \"Mine\"\ninherits = \"okabe-ito\"\n",
            );

            let (selected, notices) = launch(&settings, &dir);
            assert!(matches!(
                selected,
                Err(Unavailable::Refused(ref reason)) if reason.contains("my-dark.toml")
            ));
            assert!(
                notices.iter().any(|notice| notice.contains("\"my-dark\"")
                    && notice.contains("theme.dark")
                    && notice.contains("my-dark.toml")
                    && notice.contains("Okabe–Ito")),
                "{notices:?}"
            );

            dir.write("my-dark.toml", &dark("Mine"));
            let (selected, notices) = launch(&settings, &dir);
            assert_eq!(selected, Ok("my-dark".to_owned()));
            assert!(notices.is_empty(), "{notices:?}");
        }

        #[test]
        fn a_missing_selected_file_falls_back_and_returns_when_the_file_does() {
            let settings = selecting_dark("my-dark");
            let dir = TempDir::new();

            let (selected, notices) = launch(&settings, &dir);
            assert_eq!(selected, Err(Unavailable::Missing));
            assert!(
                notices.iter().any(|n| n.contains("\"my-dark\"")),
                "{notices:?}"
            );

            dir.write("my-dark.toml", &dark("Mine"));
            let (selected, _) = launch(&settings, &dir);
            assert_eq!(selected, Ok("my-dark".to_owned()));
        }

        ///
        /// `theme.dark` naming a light Theme is refused the same way as one
        /// naming no Theme.
        ///
        #[test]
        fn a_selection_of_the_wrong_appearance_falls_back_with_a_notice() {
            let settings = selecting_dark("orcvs-light");
            let dir = TempDir::new();

            let (selected, notices) = launch(&settings, &dir);
            assert_eq!(
                selected,
                Err(Unavailable::WrongAppearance(Appearance::Light))
            );
            assert!(
                notices
                    .iter()
                    .any(|n| n.contains("\"orcvs-light\"") && n.contains("theme.dark")),
                "{notices:?}"
            );
        }

        #[test]
        fn a_selected_conflicted_identity_falls_back_and_recovers() {
            let settings = selecting_dark("dup");
            let dir = TempDir::new();
            dir.write("dup.toml", &dark("Toml"));
            let json = dir.write("dup.json", &super::dark_json("Json"));

            let (selected, notices) = launch(&settings, &dir);
            assert!(matches!(
                selected,
                Err(Unavailable::Refused(ref reason)) if reason.contains("dup.json")
                    && reason.contains("dup.toml")
            ));
            assert!(
                notices.iter().any(|n| n.contains("dark Theme \"dup\"")),
                "{notices:?}"
            );

            std::fs::remove_file(json).expect("resolving the conflict");
            let (selected, notices) = launch(&settings, &dir);
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
    /// Loads the valid Theme file `file_name` into `registry`, as native
    /// discovery loads each file it reads, without a Theme directory.
    ///
    pub(crate) fn load(registry: &mut super::ThemeRegistry, file_name: &str, bytes: &[u8]) {
        let theme = registry.load(file_name, bytes).expect("a valid document");
        registry.add(theme);
    }

    ///
    /// The built-ins with [`MY_DARK`] and [`MY_LIGHT`] loaded beside them.
    ///
    pub(crate) fn with_my_themes() -> super::ThemeRegistry {
        let mut registry = super::ThemeRegistry::built_in();
        load(&mut registry, "my-dark.toml", MY_DARK.as_bytes());
        load(&mut registry, "my-light.toml", MY_LIGHT.as_bytes());
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
    #[cfg(not(target_arch = "wasm32"))]
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
