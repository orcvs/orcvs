//!
//! The console's storage seam: the Source revision eframe storage holds.
//!
//! Settings are not stored here: the dark and light Theme and the Cursor
//! Effect's motion come from `~/.orcvs/config.toml` (`crate::config`), and
//! the appearance mode is egui's own `ThemePreference`, which eframe stores
//! with egui memory.
//!
//! The Language Map, Tokens, diagnostics and parsed Expressions are derived
//! from the Source, so a restore rebuilds them. The console runtime and
//! animation state are not stored.
//!

use orcvs::grid::Grid;
use orcvs::source::Source;

///
/// The Storage key one stored Source revision lives under.
///
/// Deliberately not `eframe::APP_KEY`: that key names the whole App value, and
/// the Console is a runtime coordinator rather than a serializable application
/// value. This key stores only the Source payload.
///
#[cfg(feature = "persistence")]
pub const SOURCE_KEY: &str = "orcvs_source";

///
/// The Source a console starts from when it restores nothing, and the one
/// `File → New` opens: the one Grid, empty.
///
pub(crate) fn default_source() -> Source {
    Source::new(Grid::new())
}

///
/// The Source the console starts from.
///
/// Without the `persistence` feature no revision is ever stored, so the
/// console always starts an empty Source on the one Grid.
///
#[cfg(not(feature = "persistence"))]
pub(crate) fn starting_source(_storage: Option<&dyn eframe::Storage>) -> Source {
    default_source()
}

///
/// The Source the console starts from: the stored revision, or an empty
/// Source on the one Grid.
///
/// eframe answers `None` both for an absent key and for a value that does not
/// decode, so the key is read before it is decoded: holding nothing is an
/// ordinary first start, and holding a value that does not decode — a Grid
/// shape other than the one, or a Cell count or character the
/// Source refuses — is reported. Either way the console starts an empty
/// Source rather than a partly restored one, and its next save overwrites
/// [`SOURCE_KEY`].
///
#[cfg(feature = "persistence")]
pub(crate) fn starting_source(storage: Option<&dyn eframe::Storage>) -> Source {
    let Some(storage) = storage else {
        return default_source();
    };
    if storage.get_string(SOURCE_KEY).is_none() {
        return default_source();
    }
    eframe::get_value::<Source>(storage, SOURCE_KEY).unwrap_or_else(|| {
        // `crate::report` sends this to whichever channel the target reads.
        // The key is in the message rather than a `tracing` field because the
        // report has to read the same in a browser developer console, which
        // has no fields.
        crate::report::error!(
            "{SOURCE_KEY}: the stored Source could not be read and was discarded; \
             starting an empty Grid"
        );
        default_source()
    })
}

///
/// Stores the current Source revision under [`SOURCE_KEY`].
///
#[cfg(feature = "persistence")]
pub(crate) fn save(storage: &mut dyn eframe::Storage, source: &orcvs::source::SourceCommander) {
    source.read_source(|source| eframe::set_value(storage, SOURCE_KEY, source));
}

///
/// The Grid the console opens on, holding no Cells: what every start that
/// restores nothing produces.
///
#[cfg(test)]
fn assert_default_grid(source: &Source) {
    use orcvs::grid::{COL_COUNT, ROW_COUNT};

    let grid = source.grid();

    assert_eq!((COL_COUNT, ROW_COUNT), (256, 256));
    assert_eq!(grid.count(), COL_COUNT * ROW_COUNT);
    assert!(grid.position(COL_COUNT - 1, ROW_COUNT - 1).is_some());
    assert!(grid.position(COL_COUNT, 0).is_none());
    assert!(grid.position(0, ROW_COUNT).is_none());
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
/// An edited Source on the one Grid: a start that read nothing holds no Cells
/// at all, so the edit is what tells a restore from a fresh start.
///
#[cfg(all(test, feature = "persistence"))]
pub(crate) fn edited_source() -> orcvs::source::SourceCommander {
    use orcvs::source::SourceCommander;

    let grid = Grid::new();
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "persistence")]
    use super::{InMemoryStorage, SOURCE_KEY, edited_source, save};
    use super::{assert_default_grid, starting_source};

    #[test]
    fn a_console_with_no_storage_starts_the_default_grid() {
        // The whole of what a build without the `persistence` feature does,
        // and what a first start does with the feature.
        assert_default_grid(&starting_source(None));
    }

    ///
    /// A save writes the Source and nothing under the settings keys below.
    ///
    #[cfg(feature = "persistence")]
    #[test]
    fn a_save_writes_the_source_and_no_settings() {
        let current = edited_source();
        let mut storage = InMemoryStorage::default();
        save(&mut storage, &current);

        for retired in ["cursor_effects", "dark_theme", "light_theme"] {
            assert_eq!(
                eframe::Storage::get_string(&storage, retired),
                None,
                "a save wrote the retired {retired} key"
            );
        }
        assert!(eframe::Storage::get_string(&storage, SOURCE_KEY).is_some());
        assert_eq!(
            starting_source(Some(&storage)).snapshot(),
            current.snapshot()
        );
    }
}

#[cfg(all(test, feature = "persistence"))]
mod stored_source_tests {
    use orcvs::source::{SourceCommander, Token};

    use super::{
        InMemoryStorage, SOURCE_KEY, assert_default_grid, edited_source, save, starting_source,
    };

    fn stored(storage: &InMemoryStorage) -> Option<String> {
        eframe::Storage::get_string(storage, SOURCE_KEY)
    }

    #[test]
    fn the_saved_revision_restores_its_cells_and_rebuilds_its_derived_views() {
        let saved = edited_source();
        let mut storage = InMemoryStorage::default();

        save(&mut storage, &saved);

        // The save call stores the revision under the Source key, which is
        // where the next start looks for it.
        assert!(stored(&storage).is_some());

        let restored = SourceCommander::with_source(starting_source(Some(&storage)));

        assert_eq!(restored.snapshot(), saved.snapshot());
        assert_eq!(restored.grid().count(), 256 * 256);
        assert!(restored.grid().position(255, 255).is_some());
        assert!(restored.grid().position(256, 255).is_none());
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
    /// A Source stored at any shape but the one is discarded rather than
    /// migrated (ADR 0054): the console starts the empty Grid, and its next
    /// save writes that Grid over the stored value.
    ///
    #[test]
    fn a_source_stored_at_a_previous_default_grid_is_discarded_rather_than_migrated() {
        use orcvs::grid::Grid;

        for (cols, rows) in [(128, 80), (64, 40)] {
            let previous = SourceCommander::new(Grid::with_shape(cols, rows));
            let cell = previous.grid().cell_index(0).expect("inside the Grid");
            previous.set(cell, "1").expect("a Cell the Source accepts");
            let mut storage = InMemoryStorage::default();
            save(&mut storage, &previous);
            let value = stored(&storage).expect("the stored revision");

            let started = starting_source(Some(&storage));
            assert_default_grid(&started);

            save(&mut storage, &SourceCommander::with_source(started));
            assert_ne!(
                stored(&storage),
                Some(value),
                "the save left the {cols} by {rows} Source in place"
            );
            assert_default_grid(&starting_source(Some(&storage)));
        }
    }

    #[test]
    fn an_absent_value_starts_the_default_grid() {
        let storage = InMemoryStorage::default();

        assert_default_grid(&starting_source(Some(&storage)));
    }

    #[test]
    fn a_malformed_value_is_discarded_whole_and_starts_the_default_grid() {
        let mut written = InMemoryStorage::default();
        save(&mut written, &edited_source());
        let encoded = stored(&written).expect("the save call stored the revision");
        assert!(
            encoded.contains("cols:256"),
            "the stored revision names its Grid: {encoded}"
        );

        // Three ways a stored value goes bad: bytes that are not the stored
        // encoding, Cells that do not match the Grid beside them, and a Grid
        // of another shape.
        //
        // What refuses the second is `Source`'s own `Deserialize`, and
        // `orcvs/src/source/model.rs` already covers that validation directly.
        // What this adds is the end-to-end assertion that the refusal survives
        // eframe's codec and reaches the console as an empty Grid, not new
        // coverage of the validation itself.
        for value in [
            "not a stored Source",
            &encoded.replacen(".+0102 ", ".+0102", 1),
            &encoded.replace("rows:256", "rows:255"),
        ] {
            let mut storage = InMemoryStorage::default();
            storage
                .entries
                .insert(SOURCE_KEY.to_owned(), value.to_owned());

            assert_default_grid(&starting_source(Some(&storage)));
        }
    }
}
