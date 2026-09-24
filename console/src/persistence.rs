//!
//! The console's storage seam: the Source revision and presentation settings
//! eframe storage holds.
//!
//! The Language Map, Tokens, diagnostics and parsed Expressions are derived
//! from the Source, so a restore rebuilds them. The console runtime and
//! animation state are not stored.
//!

use orcvs::grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT, Grid};
use orcvs::source::Source;

use crate::cursor_effects::CursorEffectSettings;
#[cfg(feature = "persistence")]
use crate::theme::{Appearance, ThemeIdentity};
use crate::theme_selection::ThemeSelection;

///
/// The Storage key one stored Source revision lives under.
///
/// Deliberately not `eframe::APP_KEY`: that key names the whole App value, and
/// `source-playback-engine/18` settled that the Console is a runtime
/// coordinator rather than a serializable application value. This key stores
/// only the Source payload; presentation settings use their own key.
///
#[cfg(feature = "persistence")]
pub const SOURCE_KEY: &str = "orcvs_source";

///
/// The Storage key Glitch amount and Glitch frequency are restored from,
/// encoded as `amount;frequency`. An absent or malformed value falls back to
/// [`CursorEffectSettings::default`].
///
#[cfg(feature = "persistence")]
pub(crate) const CURSOR_EFFECTS_KEY: &str = "cursor_effects";

///
/// The Storage keys the dark and light Theme selections live under: each
/// holds a Theme identity as a plain string, never a Theme's values —
/// `SourcePaintSettings`, `CursorEffectSettings`' colours and the old
/// `source_paint` key are gone rather than migrated (ADR 0053). An absent
/// key takes that appearance's default built-in; the View menu's dark and
/// light Theme pickers write another. The mode beside them is egui's own
/// `ThemePreference`, which eframe stores with egui memory, not here.
///
#[cfg(feature = "persistence")]
pub(crate) const DARK_THEME_KEY: &str = "dark_theme";
#[cfg(feature = "persistence")]
pub(crate) const LIGHT_THEME_KEY: &str = "light_theme";

/// The Storage key holding `appearance`'s Theme selection.
#[cfg(feature = "persistence")]
const fn theme_key(appearance: Appearance) -> &'static str {
    match appearance {
        Appearance::Dark => DARK_THEME_KEY,
        Appearance::Light => LIGHT_THEME_KEY,
    }
}

///
/// The Storage key the web target's imported Theme documents live under: a
/// JSON list of each document's file name and source text, never its
/// resolved values, so every restore decodes and validates it again
/// (`.scratch/theming/issues/07`). Native Theme files are authoritative and
/// are never stored; only the web imports anything.
///
#[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
pub(crate) const IMPORTED_THEMES_KEY: &str = "imported_themes";

///
/// The Storage key an [`IMPORTED_THEMES_KEY`] value that could not be decoded
/// is moved to before anything writes that key again — the same refuse-aside
/// rule [`REFUSED_KEY`] applies to the Source.
///
#[cfg(all(feature = "persistence", any(target_arch = "wasm32", test)))]
pub(crate) const IMPORTED_THEMES_REFUSED_KEY: &str = "imported_themes_refused";

///
/// The Storage key a value that could not be read back is moved to.
///
/// A refused value is Cells a viewer may still recover by hand, and the
/// console's own save is the one thing certain to destroy them: eframe calls
/// it every thirty seconds and it writes [`SOURCE_KEY`]. Moving the value here
/// first is what makes a refusal mean "not restored" rather than "deleted".
///
#[cfg(feature = "persistence")]
pub const REFUSED_KEY: &str = "orcvs_source_refused";

///
/// The Source a console starts from when it restores nothing: the ordinary
/// default Grid, empty.
///
fn default_source() -> Source {
    Source::new(Grid::new(DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT))
}

///
/// The Source the console starts from.
///
/// Without the `persistence` feature no revision is ever stored, so the
/// console always starts the ordinary default Grid.
///
#[cfg(not(feature = "persistence"))]
pub(crate) fn starting_source(_storage: Option<&dyn eframe::Storage>) -> Start {
    Start {
        source: default_source(),
        cursor_effects: CursorEffectSettings::default(),
        theme_selection: ThemeSelection::default(),
    }
}

///
/// What storage holds under the console's key.
///
#[cfg(feature = "persistence")]
enum StoredSource {
    /// Storage holds no revision: an ordinary first start.
    Absent,
    /// Storage holds a value this build will not read. The value is kept, so
    /// the console's next save can move it aside rather than write over it.
    Refused(String),
    /// The stored revision, with every derived view rebuilt from it.
    Restored(Source),
}

///
/// The Source to open and the persistence state for the session that follows.
///
pub(crate) struct Start {
    pub(crate) source: Source,
    pub(crate) cursor_effects: CursorEffectSettings,
    /// The dark and light Theme identities, restored independently of the
    /// Source and of Cursor effects.
    pub(crate) theme_selection: ThemeSelection,
    #[cfg(feature = "persistence")]
    pub(crate) persistence: Persistence,
}

///
/// Owns refusal recovery across saves and notice dismissal. The running Orcvs
/// owns the Source and supplies its current revision at each save.
///
/// A refusal leaves two independent obligations: preserve its payload before
/// overwriting the stored revision, and show its notice until dismissed.
/// Saving ends only the first; dismissal ends only the second.
///
#[cfg(feature = "persistence")]
pub(crate) struct Persistence {
    refused: Option<String>,
    notice: bool,
}

#[cfg(feature = "persistence")]
impl Persistence {
    /// Preserves a refused payload once, then stores the current Source and
    /// the presentation settings.
    ///
    /// The Theme identities are plain strings, stored verbatim rather than
    /// through an `encode`/`decode` pair: a Theme identity is already the
    /// value a key holds, with no packed fields of its own to round-trip.
    /// They are the selection as the viewer made it, not the Themes it
    /// presents, so a selection that fell back is saved unchanged.
    pub(crate) fn save(
        &mut self,
        storage: &mut dyn eframe::Storage,
        source: &orcvs::source::SourceCommander,
        cursor_effects: CursorEffectSettings,
        theme_selection: &ThemeSelection,
    ) {
        if let Some(refused) = self.refused.take() {
            storage.set_string(REFUSED_KEY, refused);
        }
        source.read_source(|source| eframe::set_value(storage, SOURCE_KEY, source));
        storage.set_string(CURSOR_EFFECTS_KEY, cursor_effects.encode());
        for appearance in [Appearance::Dark, Appearance::Light] {
            storage.set_string(
                theme_key(appearance),
                theme_selection.identity(appearance).as_str().to_owned(),
            );
        }
    }

    pub(crate) fn notice_visible(&self) -> bool {
        self.notice
    }

    pub(crate) fn dismiss_notice(&mut self) {
        self.notice = false;
    }
}

///
/// Reads the stored revision, separating a first start from a stored value
/// that will not decode.
///
/// eframe answers `None` for both, so the key is read before it is decoded:
/// holding nothing is ordinary, and holding a value that does not decode is a
/// refusal the console reports.
///
#[cfg(feature = "persistence")]
fn stored_source(storage: Option<&dyn eframe::Storage>) -> StoredSource {
    let Some(storage) = storage else {
        return StoredSource::Absent;
    };
    let Some(stored) = storage.get_string(SOURCE_KEY) else {
        return StoredSource::Absent;
    };

    eframe::get_value::<Source>(storage, SOURCE_KEY)
        .map_or(StoredSource::Refused(stored), StoredSource::Restored)
}

///
/// The Source the console starts from: the stored revision, or the ordinary
/// default Grid.
///
/// A stored value that does not decode — a Grid dimension, a Cell count or a
/// Cell character the Source refuses among the reasons — is refused whole and
/// reported, so the console starts the default Grid rather than a partly
/// restored one.
///
#[cfg(feature = "persistence")]
pub(crate) fn starting_source(storage: Option<&dyn eframe::Storage>) -> Start {
    let cursor_effects = storage
        .and_then(|storage| storage.get_string(CURSOR_EFFECTS_KEY))
        .as_deref()
        .and_then(CursorEffectSettings::decode)
        .unwrap_or_default();
    // Restored as plain strings: an absent key, like a first start, takes
    // that appearance's default built-in. Neither is validated here: a
    // restored identity that names no available Theme of its appearance is
    // kept as it is, and `SelectedThemes` presents the default built-in in
    // its place without rewriting it.
    let restored_identity = |appearance: Appearance| {
        storage
            .and_then(|storage| storage.get_string(theme_key(appearance)))
            .map_or_else(
                || ThemeIdentity::default_for(appearance),
                ThemeIdentity::restored,
            )
    };
    let theme_selection = ThemeSelection::new(
        restored_identity(Appearance::Dark),
        restored_identity(Appearance::Light),
    );
    match stored_source(storage) {
        StoredSource::Restored(source) => Start {
            source,
            cursor_effects,
            theme_selection,
            persistence: Persistence {
                refused: None,
                notice: false,
            },
        },
        StoredSource::Absent => Start {
            source: default_source(),
            cursor_effects,
            theme_selection,
            persistence: Persistence {
                refused: None,
                notice: false,
            },
        },
        StoredSource::Refused(stored) => {
            report_refusal();
            Start {
                source: default_source(),
                cursor_effects,
                theme_selection,
                persistence: Persistence {
                    refused: Some(stored),
                    notice: true,
                },
            }
        }
    }
}

///
/// Reports the refusal, on whichever channel the target being built reads.
///
/// `crate::report` is the one place that knows which those are; the storage
/// seam only has to say that a refusal happened. The key is in the message
/// rather than beside it as a `tracing` field because that report has to read
/// the same in a browser developer console, which has no fields.
///
#[cfg(feature = "persistence")]
fn report_refusal() {
    const REFUSED: &str = "refused the stored Source: it is not a Source this build can read; \
                           starting the default Grid";

    crate::report::error!(
        "{}: {}; the stored value is kept under {}",
        SOURCE_KEY,
        REFUSED,
        REFUSED_KEY
    );
}

///
/// The Grid the console opens on, holding no Cells: what every start that
/// restores nothing produces.
///
#[cfg(test)]
fn assert_default_grid(source: &Source) {
    let grid = source.grid();

    assert_eq!(grid.count(), DEFAULT_COL_COUNT * DEFAULT_ROW_COUNT);
    assert!(
        grid.position(DEFAULT_COL_COUNT - 1, DEFAULT_ROW_COUNT - 1)
            .is_some()
    );
    assert!(grid.position(DEFAULT_COL_COUNT, 0).is_none());
    assert!(source.snapshot().bytes().all(|byte| byte == b' '));
}

///
/// Storage with no backing file, so a test drives the same seam the shipped
/// backends drive.
///
#[cfg(all(test, feature = "persistence"))]
#[derive(Default)]
pub(crate) struct InMemoryStorage {
    entries: std::collections::BTreeMap<String, String>,
}

#[cfg(all(test, feature = "persistence"))]
impl eframe::Storage for InMemoryStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.entries.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.entries.insert(key.to_owned(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.entries.remove(key);
    }

    fn flush(&mut self) {}
}

///
/// Storage that persists a RON `HashMap<String, String>` the same way native
/// eframe `FileStorage` does.
///
/// `FileStorage::from_ron_filepath` is `pub(crate)`, so a product-path test
/// cannot construct the shipped type. This is the same file codec the native
/// binary writes to `eframe::storage_dir("Orcvs")/app.ron` — not
/// [`InMemoryStorage`]. Isolate every use under a unique temp directory; never
/// write the user's Application Support path.
///
#[cfg(all(test, feature = "persistence", not(target_arch = "wasm32")))]
pub(crate) struct IsolatedRonDir(std::path::PathBuf);

#[cfg(all(test, feature = "persistence", not(target_arch = "wasm32")))]
impl IsolatedRonDir {
    pub(crate) fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "orcvs-native-persistence-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("a clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("a unique temp directory");
        Self(dir)
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.0
    }
}

#[cfg(all(test, feature = "persistence", not(target_arch = "wasm32")))]
impl Drop for IsolatedRonDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

///
/// The native file codec: a RON key-value map flushed to `app.ron`.
///
#[cfg(all(test, feature = "persistence", not(target_arch = "wasm32")))]
pub(crate) struct RonFileStorage {
    path: std::path::PathBuf,
    kv: std::collections::HashMap<String, String>,
    dirty: bool,
}

#[cfg(all(test, feature = "persistence", not(target_arch = "wasm32")))]
impl RonFileStorage {
    pub(crate) fn create(dir: &std::path::Path) -> Self {
        Self {
            path: dir.join("app.ron"),
            kv: std::collections::HashMap::new(),
            dirty: false,
        }
    }

    pub(crate) fn from_file(dir: &std::path::Path) -> Self {
        let path = dir.join("app.ron");
        let kv = std::fs::File::open(&path)
            .ok()
            .and_then(|file| ron::de::from_reader(std::io::BufReader::new(file)).ok())
            .unwrap_or_default();
        Self {
            path,
            kv,
            dirty: false,
        }
    }
}

#[cfg(all(test, feature = "persistence", not(target_arch = "wasm32")))]
impl eframe::Storage for RonFileStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.kv.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        if self.kv.get(key) != Some(&value) {
            self.kv.insert(key.to_owned(), value);
            self.dirty = true;
        }
    }

    fn remove_string(&mut self, key: &str) {
        self.kv.remove(key);
        self.dirty = true;
    }

    fn flush(&mut self) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        let file = std::fs::File::create(&self.path).expect("the native RON file");
        let mut writer = std::io::BufWriter::new(file);
        ron::Options::default()
            .to_io_writer_pretty(&mut writer, &self.kv, ron::ser::PrettyConfig::default())
            .expect("the native RON codec");
        std::io::Write::flush(&mut writer).expect("the native RON file");
    }
}

///
/// An edited Source on a non-square Grid: a restore that read the two
/// dimensions the wrong way round addresses different Cells than the Source
/// that was stored, and a start that read nothing holds no Cells at all.
///
#[cfg(all(test, feature = "persistence"))]
pub(crate) fn edited_source() -> orcvs::source::SourceCommander {
    use orcvs::source::SourceCommander;

    let grid = Grid::new(6, 3);
    let source = SourceCommander::new(grid);
    for (index, content) in ".+0102".chars().enumerate() {
        source
            .set(
                grid.cell_index(index).expect("inside the Grid"),
                &content.to_string(),
            )
            .expect("a Cell the Source accepts");
    }
    source
}

///
/// Stores `source` the way a session with nothing to recover stores it.
///
/// A test that only needs a stored revision says that. Reaching `save`
/// through `starting_source` would start a session over the storage the test
/// is about to overwrite, and read a refusal out of it on the way past.
///
#[cfg(all(test, feature = "persistence"))]
pub(crate) fn store(storage: &mut dyn eframe::Storage, source: &orcvs::source::SourceCommander) {
    Persistence {
        refused: None,
        notice: false,
    }
    .save(
        storage,
        source,
        CursorEffectSettings::default(),
        &ThemeSelection::default(),
    );
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "persistence")]
    use super::{
        CURSOR_EFFECTS_KEY, DARK_THEME_KEY, InMemoryStorage, LIGHT_THEME_KEY, Persistence,
        edited_source,
    };
    use super::{assert_default_grid, starting_source};
    #[cfg(feature = "persistence")]
    use crate::cursor_effects::CursorEffectSettings;
    #[cfg(feature = "persistence")]
    use crate::theme::{Appearance, ThemeIdentity};
    use crate::theme_selection::ThemeSelection;

    #[test]
    fn a_console_with_no_storage_starts_the_default_grid() {
        // The whole of what a build without the `persistence` feature does,
        // and what a first start does with the feature.
        let start = starting_source(None);
        assert_default_grid(&start.source);
        assert_eq!(start.theme_selection, ThemeSelection::default());
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn absent_or_malformed_cursor_effect_settings_fall_back_to_their_own_defaults() {
        let empty = InMemoryStorage::default();
        assert_eq!(
            starting_source(Some(&empty)).cursor_effects,
            CursorEffectSettings::default()
        );

        let mut malformed = InMemoryStorage::default();
        eframe::Storage::set_string(&mut malformed, CURSOR_EFFECTS_KEY, "broken".to_owned());
        assert_eq!(
            starting_source(Some(&malformed)).cursor_effects,
            CursorEffectSettings::default()
        );
    }

    ///
    /// An absent dark/light Theme key is an ordinary first start: each takes
    /// its appearance's default built-in, `okabe-ito` and `orcvs-light`.
    /// Unlike Cursor effects there is no decode to refuse here — a Theme
    /// identity is a plain string, and one no available Theme answers to is
    /// kept rather than rejected (`theme_selection`'s fallback tests).
    ///
    #[cfg(feature = "persistence")]
    #[test]
    fn an_absent_theme_selection_defaults_to_each_appearances_built_in() {
        let empty = InMemoryStorage::default();
        let start = starting_source(Some(&empty));
        assert_eq!(
            start.theme_selection.identity(Appearance::Dark).as_str(),
            "okabe-ito"
        );
        assert_eq!(
            start.theme_selection.identity(Appearance::Light).as_str(),
            "orcvs-light"
        );
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn cursor_effect_settings_round_trip_without_affecting_the_source() {
        let current = edited_source();
        let mut settings = CursorEffectSettings::default();
        *settings.amount_mut() = 82;
        *settings.frequency_mut() = 0;
        let mut storage = InMemoryStorage::default();
        Persistence {
            refused: None,
            notice: false,
        }
        .save(&mut storage, &current, settings, &ThemeSelection::default());

        let restored = starting_source(Some(&storage));
        assert_eq!(restored.cursor_effects, settings);
        assert_eq!(restored.theme_selection, ThemeSelection::default());
        assert_eq!(restored.source.snapshot(), current.snapshot());
    }

    ///
    /// The dark and light Theme identities round trip at their own keys,
    /// independently of the Source and Cursor effects — saved alongside them
    /// in the same call — and of each other.
    ///
    #[cfg(feature = "persistence")]
    #[test]
    fn theme_identities_round_trip_independently_of_the_source_and_cursor_effects_and_each_other() {
        let current = edited_source();
        let cursor_effects = CursorEffectSettings::default();
        let mut storage = InMemoryStorage::default();
        Persistence {
            refused: None,
            notice: false,
        }
        .save(
            &mut storage,
            &current,
            cursor_effects,
            &ThemeSelection::new(
                ThemeIdentity::restored("my-dark".to_owned()),
                ThemeIdentity::restored("my-light".to_owned()),
            ),
        );

        let restored = starting_source(Some(&storage));
        assert_eq!(
            restored.theme_selection.identity(Appearance::Dark).as_str(),
            "my-dark"
        );
        assert_eq!(
            restored
                .theme_selection
                .identity(Appearance::Light)
                .as_str(),
            "my-light"
        );
        assert_eq!(restored.cursor_effects, cursor_effects);
        assert_eq!(restored.source.snapshot(), current.snapshot());
        assert_eq!(
            eframe::Storage::get_string(&storage, DARK_THEME_KEY).as_deref(),
            Some("my-dark")
        );
        assert_eq!(
            eframe::Storage::get_string(&storage, LIGHT_THEME_KEY).as_deref(),
            Some("my-light")
        );
    }
}

#[cfg(all(test, feature = "persistence"))]
mod stored_source_tests {
    use orcvs::source::{SourceCommander, Token};

    use super::{
        InMemoryStorage, REFUSED_KEY, SOURCE_KEY, StoredSource, assert_default_grid, edited_source,
        starting_source, store, stored_source,
    };
    use crate::cursor_effects::CursorEffectSettings;
    use crate::theme_selection::ThemeSelection;

    fn stored(storage: &InMemoryStorage) -> Option<String> {
        eframe::Storage::get_string(storage, SOURCE_KEY)
    }

    #[test]
    fn saving_and_dismissing_end_independent_recovery_obligations() {
        for dismiss_before_save in [false, true] {
            let mut storage = InMemoryStorage::default();
            let refused = "not a stored Source";
            eframe::Storage::set_string(&mut storage, SOURCE_KEY, refused.to_owned());
            let start = starting_source(Some(&storage));
            assert_default_grid(&start.source);
            let mut persistence = start.persistence;
            assert!(persistence.notice_visible());

            if dismiss_before_save {
                persistence.dismiss_notice();
                assert!(!persistence.notice_visible());
            }

            let current = edited_source();
            persistence.save(
                &mut storage,
                &current,
                CursorEffectSettings::default(),
                &ThemeSelection::default(),
            );
            assert_eq!(persistence.notice_visible(), !dismiss_before_save);
            assert_eq!(
                eframe::Storage::get_string(&storage, REFUSED_KEY).as_deref(),
                Some(refused)
            );
            assert_eq!(
                starting_source(Some(&storage)).source.snapshot(),
                current.snapshot()
            );

            // Dismissal is idempotent and must not undo preservation or
            // prevent subsequent saves from writing the latest revision.
            persistence.dismiss_notice();
            persistence.dismiss_notice();
            let cell = current.grid().cell_index(0).expect("inside the Grid");
            current.set(cell, " ").expect("a valid empty Cell");
            persistence.save(
                &mut storage,
                &current,
                CursorEffectSettings::default(),
                &ThemeSelection::default(),
            );
            assert!(!persistence.notice_visible());
            assert_eq!(
                eframe::Storage::get_string(&storage, REFUSED_KEY).as_deref(),
                Some(refused),
                "a later save must not displace the refused value"
            );
            assert_eq!(
                starting_source(Some(&storage)).source.snapshot(),
                current.snapshot()
            );
        }
    }

    #[test]
    fn an_absent_or_restored_start_has_no_notice_and_preserves_existing_recovery() {
        let mut restored = InMemoryStorage::default();
        store(&mut restored, &edited_source());
        for mut storage in [InMemoryStorage::default(), restored] {
            eframe::Storage::set_string(&mut storage, REFUSED_KEY, "previous refusal".to_owned());
            let mut persistence = starting_source(Some(&storage)).persistence;
            assert!(!persistence.notice_visible());
            persistence.dismiss_notice();
            persistence.save(
                &mut storage,
                &edited_source(),
                CursorEffectSettings::default(),
                &ThemeSelection::default(),
            );
            assert!(!persistence.notice_visible());
            assert_eq!(
                eframe::Storage::get_string(&storage, REFUSED_KEY).as_deref(),
                Some("previous refusal")
            );
        }
    }

    #[test]
    fn the_saved_revision_restores_its_cells_and_rebuilds_its_derived_views() {
        let saved = edited_source();
        let mut storage = InMemoryStorage::default();

        store(&mut storage, &saved);

        // The save call stores the revision under the Source key, which is
        // where the next start looks for it.
        assert!(stored(&storage).is_some());

        let restored = SourceCommander::with_source(starting_source(Some(&storage)).source);

        assert_eq!(restored.snapshot(), saved.snapshot());
        assert_eq!(restored.grid().count(), 18);
        assert!(restored.grid().position(5, 2).is_some());
        assert!(restored.grid().position(6, 2).is_none());
        // The Language Map is derived, never stored: a restored Source parses
        // its Cells again, so the Tokens the console draws come back with it.
        let revision = restored.read_revision();
        let grid = revision.grid();
        assert_eq!(
            revision.token_at(grid.position(0, 0).expect("inside the Grid")),
            Some(Token::Function)
        );
    }

    ///
    /// ADR 0045: "Nothing resizes a Grid, so a Source stored at the previous
    /// 40 by 25 default opens at 40 by 25." The default has since moved to 64
    /// by 40 (`orcvs/src/grid.rs`), so this pins the Grid dimensions
    /// themselves to the stored Source rather than to whatever the console
    /// opens with no storage — a restore that read today's default instead of
    /// the stored Grid would silently resize a Source nothing asked to
    /// resize.
    ///
    #[test]
    fn a_source_stored_at_the_previous_default_grid_reopens_at_that_grid() {
        use orcvs::grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT, Grid};

        assert_ne!(
            (DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT),
            (40, 25),
            "the default Grid is 40 by 25 again, so this test no longer proves anything"
        );

        let stored = SourceCommander::new(Grid::new(40, 25));
        let mut storage = InMemoryStorage::default();
        store(&mut storage, &stored);

        let restored = starting_source(Some(&storage)).source;
        assert_eq!(restored.grid().columns(), 40);
        assert_eq!(restored.grid().rows(), 25);
    }

    #[test]
    fn an_absent_value_starts_the_default_grid() {
        let storage = InMemoryStorage::default();

        assert!(matches!(
            stored_source(Some(&storage)),
            StoredSource::Absent
        ));
        assert_default_grid(&starting_source(Some(&storage)).source);
    }

    #[test]
    fn a_malformed_value_is_refused_whole_and_starts_the_default_grid() {
        let mut written = InMemoryStorage::default();
        store(&mut written, &edited_source());
        let encoded = stored(&written).expect("the save call stored the revision");
        assert!(
            encoded.contains("cols:6"),
            "the stored revision names its Grid: {encoded}"
        );

        // Two ways a stored value goes bad: bytes that are not the stored
        // encoding at all, and a well-formed encoding whose Grid no longer
        // matches the Cells beside it. A restore that trusted the second would
        // start a partly restored Source.
        //
        // What refuses the second is `Source`'s own `Deserialize`, and
        // `orcvs/src/source/model.rs` already covers that validation directly.
        // What this adds is the end-to-end assertion that the refusal survives
        // eframe's codec and reaches the console as a default Grid, not new
        // coverage of the validation itself.
        for value in ["not a stored Source", &encoded.replace("cols:6", "cols:7")] {
            let mut storage = InMemoryStorage::default();
            storage
                .entries
                .insert(SOURCE_KEY.to_owned(), value.to_owned());

            assert!(
                matches!(stored_source(Some(&storage)), StoredSource::Refused(_)),
                "{value} is not a Source this build can read"
            );
            assert_default_grid(&starting_source(Some(&storage)).source);
        }
    }
}
