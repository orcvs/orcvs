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
    /// The shape this Source was built from, and so the only Grid that can
    /// mint an index addressing one of its Cells.
    ///
    /// A caller editing the Source needs it: a Cell is named by an index, and
    /// only this Grid mints one. `read_revision` also answers, but copies a
    /// whole revision to do it.
    ///
    pub fn grid(&self) -> Grid {
        read_recover(&self.inner).grid()
    }

    ///
    /// Synchronous edit: when this returns, every observable part of the
    /// Source describes the new revision.
    ///
    pub fn set(&self, cell: CellIndex, s: &str) -> Result<(), SourceError> {
        write_recover(&self.inner).set(cell, s)
    }

    ///
    /// Synchronous delete: when this returns, every observable part of the
    /// Source describes the new revision.
    ///
    pub fn unset(&self, cell: CellIndex) {
        write_recover(&self.inner).unset(cell);
    }

    /// What `cell` holds at the current revision, or `None` when it is empty.
    pub fn get(&self, cell: CellIndex) -> Option<String> {
        read_recover(&self.inner).get(cell)
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
        let grid = Grid::new(2, 1);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
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
        source.set(cell(0), "x").unwrap();
        assert_eq!(source.get(cell(0)).as_deref(), Some("x"));
    }

    #[test]
    fn rejected_overlong_expression_does_not_poison_source_access() {
        let grid = Grid::new(80, 3);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        let at_capacity = ".+".repeat(15) + &"00".repeat(16);
        for (offset, content) in at_capacity.chars().enumerate() {
            source.set(cell(offset + 2), &content.to_string()).unwrap();
        }
        source.set(cell(1), "+").unwrap();
        let before = source.snapshot();

        assert_eq!(
            source.set(cell(0), "."),
            Err(SourceError::ExpressionTooLong {
                start: 0,
                end: 63,
                capacity: 32,
            })
        );
        assert_eq!(source.snapshot(), before);

        source.unset(cell(1));
        source.set(cell(150), ".").unwrap();
        source.set(cell(151), "+").unwrap();
        source.set(cell(152), "0").unwrap();
        source.set(cell(153), "1").unwrap();
        source.set(cell(154), "0").unwrap();
        source.set(cell(155), "2").unwrap();
        let tick = source.execute();

        assert!(tick.diagnostics.is_empty());
        assert_eq!(source.get(cell(150)), Some(".".to_string()));
    }

    #[test]
    fn tick_suppresses_an_overlong_expression_created_by_its_writes() {
        let grid = Grid::new(100, 3);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        for (offset, content) in ".+".repeat(15).chars().enumerate() {
            source
                .set(cell(100 + offset), &content.to_string())
                .unwrap();
            source
                .set(cell(132 + offset), &content.to_string())
                .unwrap();
        }
        for (offset, content) in ".+0102".chars().enumerate() {
            source.set(cell(30 + offset), &content.to_string()).unwrap();
            source.set(cell(70 + offset), &content.to_string()).unwrap();
        }

        source.execute();

        assert_eq!(source.get(cell(130)), Some("0".to_string()));
        assert_eq!(source.get(cell(131)), Some("3".to_string()));
        assert_eq!(source.get(cell(170)), Some("0".to_string()));
        assert_eq!(source.get(cell(171)), Some("3".to_string()));
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
        source.set(cell(199), "x").unwrap();

        assert_eq!(source.get(cell(199)), Some("x".to_string()));
        assert_eq!(source.get(cell(170)), Some("0".to_string()));
        assert_eq!(source.get(cell(171)), Some("3".to_string()));
    }

    #[test]
    fn setting_clearing_and_reading_a_cell_all_take_a_grid_minted_index() {
        // The whole editing seam in one place, now that it has one shape.
        // Addressing is settled before the Source is asked anything, so the
        // only rules left are about content.
        let grid = Grid::new(4, 2);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");

        source.set(cell(0), ".").unwrap();
        source.set(cell(1), "+").unwrap();

        assert_eq!(source.get(cell(0)).as_deref(), Some("."));
        assert_eq!(source.get(cell(1)).as_deref(), Some("+"));

        // A Cell it cannot store is still refused, and refusing it leaves the
        // Cell as it was. That rule is about content, and it is a separate rule
        // that keeps its own error.
        assert_eq!(
            source.set(cell(2), "ab"),
            Err(SourceError::InvalidCell {
                content: "ab".to_string()
            })
        );
        assert_eq!(source.get(cell(2)), None);

        source.unset(cell(0));

        assert_eq!(source.get(cell(0)), None);
        assert_eq!(source.get(cell(1)).as_deref(), Some("+"));
    }

    #[test]
    #[should_panic(expected = "CellIndex belongs to another Grid")]
    fn the_editing_seam_refuses_an_index_minted_by_another_grid() {
        // What the seam has instead of an out-of-range error: a Cell this
        // Source does not have cannot be presented to it at all, and an index
        // from a Grid of the same shape is still not one of this Source's.
        let source = SourceCommander::new(Grid::new(4, 2));
        let foreign = Grid::new(4, 2).cell_index(0).expect("inside the Grid");

        let _ = source.set(foreign, "x");
    }

    #[test]
    fn coherent_read_pairs_every_cell_with_its_source_derived_glyph() {
        let grid = Grid::new(4, 2);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        source.set(cell(0), ".").unwrap();
        source.set(cell(1), "+").unwrap();

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
        let grid = Grid::new(4, 2);
        let source = SourceCommander::new(grid);
        let cell = |idx| grid.cell_index(idx).expect("inside the Grid");
        source.set(cell(0), ".").unwrap();
        source.set(cell(1), "+").unwrap();

        let first = source.read_revision();
        let second = source.read_revision();

        assert!(std::ptr::eq(first.language_map(), second.language_map()));
    }
}
