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
pub(crate) fn starting_source(_storage: Option<&dyn eframe::Storage>) -> Source {
    default_source()
}

///
/// What storage holds under the console's key.
///
#[cfg(feature = "persistence")]
enum StoredSource {
    /// Storage holds no revision: an ordinary first start.
    Absent,
    /// Storage holds a value this build will not read.
    Refused,
    /// The stored revision, with every derived view rebuilt from it.
    Restored(Source),
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
    if storage.get_string(eframe::APP_KEY).is_none() {
        return StoredSource::Absent;
    }

    eframe::get_value::<Source>(storage, eframe::APP_KEY)
        .map_or(StoredSource::Refused, StoredSource::Restored)
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
pub(crate) fn starting_source(storage: Option<&dyn eframe::Storage>) -> Source {
    match stored_source(storage) {
        StoredSource::Restored(source) => source,
        StoredSource::Absent => default_source(),
        StoredSource::Refused => {
            report_refusal();
            default_source()
        }
    }
}

///
/// What is stored is the same shape on both targets; what reports a refusal is
/// not, so the refusal is put on both channels.
///
/// The native binary installs a `tracing` subscriber (`shell/src/main.rs`) and
/// reads `tracing`. The browser build installs `eframe::WebLogger` instead,
/// which forwards `log` records to the developer console and knows nothing of
/// `tracing`; with no `tracing` subscriber on that target a `tracing` event is
/// dropped where nobody sees it. Reporting on the channel each target actually
/// reads is what makes "refused, and reported" true in the browser as well.
///
/// `log` is already a `wasm32`-only dependency of this crate, so the second
/// line costs no manifest change. The alternative — enabling `tracing`'s `log`
/// feature in the workspace manifest, which bridges every `tracing` event to
/// `log` wherever no subscriber is installed — is one line but a much wider
/// change: it routes every event in the workspace to the browser console at
/// `WebLogger`'s `Debug` filter, and `orcvs/src/source/model.rs` emits
/// `debug!` on every Cell write, which is the Tick and edit path. Turning one
/// refusal report into per-Cell console traffic in the browser is an unmeasured
/// cost on a hot path, so the report is placed at the one site that needs it.
///
#[cfg(feature = "persistence")]
fn report_refusal() {
    const REFUSED: &str = "refused the stored Source: it is not a Source this build can read; \
                           starting the default Grid";

    tracing::error!(key = eframe::APP_KEY, "{}", REFUSED);
    #[cfg(target_arch = "wasm32")]
    log::error!("{}: {}", eframe::APP_KEY, REFUSED);
}

///
/// Stores the current Source revision, replacing the revision stored before
/// it.
///
#[cfg(feature = "persistence")]
pub(crate) fn store_source(
    storage: &mut dyn eframe::Storage,
    source: &orcvs::source::SourceCommander,
) {
    source.read_source(|source| eframe::set_value(storage, eframe::APP_KEY, source));
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

#[cfg(test)]
mod tests {
    use super::{assert_default_grid, starting_source};

    #[test]
    fn a_console_with_no_storage_starts_the_default_grid() {
        // The whole of what a build without the `persistence` feature does,
        // and what a first start does with the feature.
        assert_default_grid(&starting_source(None));
    }
}

#[cfg(all(test, feature = "persistence"))]
mod stored_source_tests {
    use orcvs::glyph::Glyph;
    use orcvs::source::SourceCommander;

    use super::{
        InMemoryStorage, StoredSource, assert_default_grid, edited_source, starting_source,
        store_source, stored_source,
    };

    fn stored(storage: &InMemoryStorage) -> Option<String> {
        eframe::Storage::get_string(storage, eframe::APP_KEY)
    }

    #[test]
    fn the_saved_revision_restores_its_cells_and_rebuilds_its_derived_views() {
        let saved = edited_source();
        let mut storage = InMemoryStorage::default();

        store_source(&mut storage, &saved);

        // The save call stores the revision under eframe's own key, which is
        // where the next start looks for it.
        assert!(stored(&storage).is_some());

        let restored = SourceCommander::with_source(starting_source(Some(&storage)));

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
        assert_default_grid(&starting_source(Some(&storage)));
    }

    #[test]
    fn a_malformed_value_is_refused_whole_and_starts_the_default_grid() {
        let mut written = InMemoryStorage::default();
        store_source(&mut written, &edited_source());
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
                .insert(eframe::APP_KEY.to_owned(), value.to_owned());

            assert!(
                matches!(stored_source(Some(&storage)), StoredSource::Refused),
                "{value} is not a Source this build can read"
            );
            assert_default_grid(&starting_source(Some(&storage)));
        }
    }
}
