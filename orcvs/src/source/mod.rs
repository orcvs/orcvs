pub mod error;
mod language_map;
pub use language_map::{ExpressionEntry, LanguageMap, LanguageUnit, LanguageUnitKind, Span};
mod model;
mod tick;
use crate::grid::{CellIndex, Grid, Position};
pub use error::SourceError;
pub use model::{CellWrite, Diagnostic, PlayCommand, Source, TickPlan};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

fn read_recover<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write_recover<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Clone)]
pub struct SourceCommander {
    inner: Arc<RwLock<Source>>,
}

/// Character Cells and semantic language information observed from exactly one
/// Source revision.
#[derive(Clone)]
pub struct SourceRevision {
    grid: Grid,
    source: String,
    language_map: Arc<LanguageMap>,
}

impl SourceRevision {
    pub fn grid(&self) -> Grid {
        self.grid
    }

    pub fn content_at(&self, position: Position) -> Option<char> {
        self.grid.assert_owns(position);
        let byte = self.source.as_bytes()[self.grid.index(position).get()];
        (byte != b' ').then_some(char::from(byte))
    }

    pub fn language_map(&self) -> &LanguageMap {
        &self.language_map
    }
}

impl SourceCommander {
    pub fn new(grid: Grid) -> Self {
        let source = Arc::new(RwLock::new(Source::new(grid)));
        Self { inner: source }
    }

    ///
    /// Synchronous edit: when this returns, every observable part of the
    /// Source describes the new revision.
    ///
    pub fn set(&self, idx: usize, s: &str) -> Result<(), SourceError> {
        write_recover(&self.inner).set(idx, s)
    }

    ///
    /// Synchronous edit addressed by a Grid-minted Cell index: when this
    /// returns, every observable part of the Source describes the new revision.
    ///
    pub fn set_cell(&self, cell: CellIndex, s: &str) -> Result<(), SourceError> {
        write_recover(&self.inner).set_cell(cell, s)
    }

    ///
    /// Synchronous delete: when this returns, every observable part of the
    /// Source describes the new revision.
    ///
    pub fn unset(&self, idx: usize) -> Result<(), SourceError> {
        write_recover(&self.inner).unset(idx)
    }

    ///
    /// Synchronous delete addressed by a Grid-minted Cell index: when this
    /// returns, every observable part of the Source describes the new revision.
    ///
    pub fn unset_cell(&self, cell: CellIndex) {
        write_recover(&self.inner).unset_cell(cell);
    }

    pub fn get(&self, idx: usize) -> Option<String> {
        read_recover(&self.inner).get(idx)
    }

    /// What `cell` holds at the current revision, or `None` when it is empty.
    pub fn get_cell(&self, cell: CellIndex) -> Option<String> {
        read_recover(&self.inner).get_cell(cell)
    }

    ///
    /// The full grid contents, read consistently at one revision.
    ///
    pub fn snapshot(&self) -> String {
        read_recover(&self.inner).snapshot()
    }

    /// Every character Cell and its Language Map from one Source revision.
    pub fn read_revision(&self) -> SourceRevision {
        let source = read_recover(&self.inner);
        SourceRevision {
            grid: source.grid(),
            source: source.snapshot(),
            language_map: source.shared_language_map(),
        }
    }

    pub(crate) fn execute(&self) -> TickPlan {
        write_recover(&self.inner).execute()
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceCommander, SourceError};
    use crate::grid::Grid;

    #[test]
    fn source_access_recovers_after_the_lock_is_poisoned() {
        let source = SourceCommander::new(Grid::new(2, 1));
        let poisoned = source.clone();

        assert!(
            std::thread::spawn(move || {
                let _guard = poisoned.inner.write().unwrap();
                panic!("poison the Source lock");
            })
            .join()
            .is_err()
        );

        assert_eq!(source.snapshot(), "  ");
        source.set(0, "x").unwrap();
        assert_eq!(source.get(0).as_deref(), Some("x"));
    }

    #[test]
    fn rejected_overlong_expression_does_not_poison_source_access() {
        let source = SourceCommander::new(Grid::new(80, 3));
        let at_capacity = ".+".repeat(15) + &"00".repeat(16);
        for (offset, content) in at_capacity.chars().enumerate() {
            source.set(offset + 2, &content.to_string()).unwrap();
        }
        source.set(1, "+").unwrap();
        let before = source.snapshot();

        assert_eq!(
            source.set(0, "."),
            Err(SourceError::ExpressionTooLong {
                start: 0,
                end: 63,
                capacity: 32,
            })
        );
        assert_eq!(source.snapshot(), before);

        source.unset(1).unwrap();
        source.set(150, ".").unwrap();
        source.set(151, "+").unwrap();
        source.set(152, "0").unwrap();
        source.set(153, "1").unwrap();
        source.set(154, "0").unwrap();
        source.set(155, "2").unwrap();
        let tick = source.execute();

        assert!(tick.diagnostics.is_empty());
        assert_eq!(source.get(150), Some(".".to_string()));
    }

    #[test]
    fn tick_suppresses_an_overlong_expression_created_by_its_writes() {
        let source = SourceCommander::new(Grid::new(100, 3));
        for (offset, content) in ".+".repeat(15).chars().enumerate() {
            source.set(100 + offset, &content.to_string()).unwrap();
            source.set(132 + offset, &content.to_string()).unwrap();
        }
        for (offset, content) in ".+0102".chars().enumerate() {
            source.set(30 + offset, &content.to_string()).unwrap();
            source.set(70 + offset, &content.to_string()).unwrap();
        }

        source.execute();

        assert_eq!(source.get(130), Some("0".to_string()));
        assert_eq!(source.get(131), Some("3".to_string()));
        assert_eq!(source.get(170), Some("0".to_string()));
        assert_eq!(source.get(171), Some("3".to_string()));
        assert!(
            source
                .read_revision()
                .language_map()
                .diagnostics()
                .any(|diagnostic| {
                    diagnostic.start() == 100
                        && diagnostic.end() == 161
                        && diagnostic.message
                            == "expression exceeds the parser capacity of 32 atoms"
                })
        );

        source.execute();
        source.set(199, "x").unwrap();

        assert_eq!(source.get(199), Some("x".to_string()));
        assert_eq!(source.get(170), Some("0".to_string()));
        assert_eq!(source.get(171), Some("3".to_string()));
    }

    #[test]
    fn a_cell_is_addressable_by_a_grid_minted_index_and_by_a_number() {
        // The expand half of the two-step change. Both forms reach the same
        // Cell and the same rules: setting, clearing and reading each accept
        // an index the Grid minted, and the number-taking forms still work
        // because nothing is migrated yet.
        let grid = Grid::new(4, 2);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");

        source.set_cell(cell(0), ".").unwrap();
        source.set(1, "+").unwrap();

        assert_eq!(source.get_cell(cell(0)).as_deref(), Some("."));
        assert_eq!(source.get(0).as_deref(), Some("."));
        assert_eq!(source.get_cell(cell(1)).as_deref(), Some("+"));

        // A Cell it cannot store is still refused; that rule is about content,
        // not about addressing.
        assert_eq!(
            source.set_cell(cell(2), "ab"),
            Err(SourceError::InvalidCell {
                content: "ab".to_string()
            })
        );

        source.unset_cell(cell(0));

        assert_eq!(source.get_cell(cell(0)), None);
        assert_eq!(source.get(0), None);
    }

    #[test]
    #[should_panic(expected = "CellIndex belongs to another Grid")]
    fn the_typed_editing_form_refuses_an_index_minted_by_another_grid() {
        // What the typed form has instead of an out-of-range error: an index
        // this Source has no Cell for cannot be presented to it at all.
        let source = SourceCommander::new(Grid::new(4, 2));
        let foreign = Grid::new(4, 2).cell_index(0).expect("inside the Grid");

        let _ = source.set_cell(foreign, "x");
    }

    #[test]
    fn coherent_read_pairs_every_cell_with_its_source_derived_glyph() {
        let grid = Grid::new(4, 2);
        let source = SourceCommander::new(grid);
        source.set(0, ".").unwrap();
        source.set(1, "+").unwrap();

        let read = source.read_revision();

        assert_eq!(read.grid(), grid);
        assert_eq!(read.content_at(grid.position(0, 0).unwrap()), Some('.'));
        assert_eq!(
            read.language_map().glyph_at(grid.position(0, 0).unwrap()),
            Some(crate::glyph::Glyph::Function)
        );
        assert_eq!(read.content_at(grid.position(1, 0).unwrap()), Some('+'));
        assert!(
            grid.rows()
                .flatten()
                .skip(2)
                .all(|position| read.content_at(position).is_none())
        );
    }

    #[test]
    fn unchanged_revision_reads_share_the_language_map() {
        let source = SourceCommander::new(Grid::new(4, 2));
        source.set(0, ".").unwrap();
        source.set(1, "+").unwrap();

        let first = source.read_revision();
        let second = source.read_revision();

        assert!(std::ptr::eq(first.language_map(), second.language_map()));
    }
}
