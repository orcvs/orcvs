//!
//! The console's storage seam: the one Source revision eframe storage holds.
//!
//! `Source` is the only supported persistence root. The Language Map, Glyphs,
//! diagnostics and parsed Expressions are derived from it, so a restore
//! rebuilds them and storage never holds them; the console itself is a runtime
//! coordinator and is not stored either.
//!

use orcvs::grid::{DEFAULT_COL_COUNT, DEFAULT_ROW_COUNT, Grid};
use orcvs::source::Source;

///
/// The Storage key one stored Source revision lives under.
///
/// Deliberately not `eframe::APP_KEY`: that key names the whole App value, and
/// `source-playback-engine/18` settled that the Console is a runtime
/// coordinator with nothing to restore. What is stored here is the one
/// supported persistence root, the Source, so a reader of the stored file is
/// not told the Console was saved.
///
#[cfg(feature = "persistence")]
pub const SOURCE_KEY: &str = "orcvs_source";

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
    /// Preserves a refused payload once, then stores the current Source.
    pub(crate) fn save(
        &mut self,
        storage: &mut dyn eframe::Storage,
        source: &orcvs::source::SourceCommander,
    ) {
        if let Some(refused) = self.refused.take() {
            storage.set_string(REFUSED_KEY, refused);
        }
        source.read_source(|source| eframe::set_value(storage, SOURCE_KEY, source));
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
    match stored_source(storage) {
        StoredSource::Restored(source) => Start {
            source,
            persistence: Persistence {
                refused: None,
                notice: false,
            },
        },
        StoredSource::Absent => Start {
            source: default_source(),
            persistence: Persistence {
                refused: None,
                notice: false,
            },
        },
        StoredSource::Refused(stored) => {
            report_refusal();
            Start {
                source: default_source(),
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
    .save(storage, source);
}

#[cfg(test)]
mod tests {
    use super::{assert_default_grid, starting_source};

    #[test]
    fn a_console_with_no_storage_starts_the_default_grid() {
        // The whole of what a build without the `persistence` feature does,
        // and what a first start does with the feature.
        assert_default_grid(&starting_source(None).source);
    }
}

#[cfg(all(test, feature = "persistence"))]
mod stored_source_tests {
    use orcvs::glyph::Glyph;
    use orcvs::source::SourceCommander;

    use super::{
        InMemoryStorage, REFUSED_KEY, SOURCE_KEY, StoredSource, assert_default_grid, edited_source,
        starting_source, store, stored_source,
    };

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
            persistence.save(&mut storage, &current);
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
            persistence.save(&mut storage, &current);
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
            persistence.save(&mut storage, &edited_source());
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
        // its Cells again, so the Glyphs the console draws come back with it.
        let revision = restored.read_revision();
        let grid = revision.grid();
        assert_eq!(
            revision
                .language_map()
                .glyph_at(grid.position(0, 0).expect("inside the Grid")),
            Some(Glyph::Function)
        );
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
