pub mod error;
mod expression_map;
mod model;
use crate::{glyph::Glyph, grid::Grid};
pub use error::SourceError;
pub use model::{Cell, CellWrite, Change, Diagnostic, PlayCommand, Source, TickPlan, TickResult};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SourceCommander {
    inner: Arc<RwLock<Source>>,
}

pub(crate) struct SourceRevisionCells {
    pub(crate) grid: Grid,
    pub(crate) cells: Vec<Cell>,
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
    pub fn set(&self, idx: usize, s: &str) -> Result<Change, SourceError> {
        self.inner.write().unwrap().set(idx, s)
    }

    ///
    /// Synchronous delete: when this returns, every observable part of the
    /// Source describes the new revision.
    ///
    pub fn unset(&self, idx: usize) -> Result<Change, SourceError> {
        self.inner.write().unwrap().unset(idx)
    }

    pub fn get(&self, idx: usize) -> (Option<String>, Option<Glyph>) {
        let source = self.inner.read().unwrap();
        let s = source.get(idx);
        let g = source.get_glyph_at(idx);
        (s, g)
    }

    ///
    /// The full grid contents, read consistently at one revision.
    ///
    pub fn snapshot(&self) -> String {
        self.inner.read().unwrap().snapshot()
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        self.inner.read().unwrap().diagnostics()
    }

    ///
    /// Every Cell and its Source-derived Glyph from one Source revision.
    ///
    pub(crate) fn read_revision_cells(&self) -> SourceRevisionCells {
        let source = self.inner.read().unwrap();
        SourceRevisionCells {
            grid: source.grid(),
            cells: source.cells(),
        }
    }

    pub(crate) fn execute(&self) -> TickResult {
        self.inner.write().unwrap().execute()
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceCommander, SourceError};
    use crate::grid::Grid;

    #[test]
    fn rejected_overlong_expression_does_not_poison_source_access() {
        let source = SourceCommander::new(Grid::new(80, 3));
        for (idx, content) in "id".repeat(31).chars().enumerate() {
            source.set(idx, &content.to_string()).unwrap();
        }
        source.set(62, "i").unwrap();
        let before = source.snapshot();

        assert_eq!(
            source.set(63, "d"),
            Err(SourceError::ExpressionTooLong {
                start: 0,
                end: 63,
                capacity: 32,
            })
        );
        assert_eq!(source.snapshot(), before);

        source.set(150, "+").unwrap();
        source.set(151, "+").unwrap();
        source.set(152, "0").unwrap();
        source.set(153, "1").unwrap();
        source.set(154, "0").unwrap();
        source.set(155, "2").unwrap();
        let tick = source.execute();

        assert!(tick.plan.diagnostics.is_empty());
        assert_eq!(source.get(150).0, Some("+".to_string()));
    }

    #[test]
    fn tick_suppresses_an_overlong_expression_created_by_its_writes() {
        let source = SourceCommander::new(Grid::new(100, 3));
        for (offset, content) in "++".repeat(15).chars().enumerate() {
            source.set(100 + offset, &content.to_string()).unwrap();
            source.set(132 + offset, &content.to_string()).unwrap();
        }
        for (offset, content) in "++0102".chars().enumerate() {
            source.set(30 + offset, &content.to_string()).unwrap();
            source.set(70 + offset, &content.to_string()).unwrap();
        }

        source.execute();

        assert_eq!(source.get(130).0, Some("0".to_string()));
        assert_eq!(source.get(131).0, Some("3".to_string()));
        assert_eq!(source.get(170).0, Some("0".to_string()));
        assert_eq!(source.get(171).0, Some("3".to_string()));
        assert!(source.diagnostics().iter().any(|diagnostic| {
            diagnostic.start == 100
                && diagnostic.end == 161
                && diagnostic.message == "expression exceeds the parser capacity of 32 atoms"
        }));

        source.execute();
        source.set(199, "x").unwrap();

        assert_eq!(source.get(199).0, Some("x".to_string()));
        assert_eq!(source.get(170).0, Some("0".to_string()));
        assert_eq!(source.get(171).0, Some("3".to_string()));
    }

    #[test]
    fn coherent_read_pairs_every_cell_with_its_source_derived_glyph() {
        let grid = Grid::new(4, 2);
        let source = SourceCommander::new(grid);
        source.set(0, "+").unwrap();
        source.set(1, "+").unwrap();

        let read = source.read_revision_cells();

        assert_eq!(read.grid, grid);
        assert_eq!(read.cells.len(), 8);
        assert_eq!(read.cells[0].content, Some('+'));
        assert_eq!(read.cells[0].glyph, Some(crate::glyph::Glyph::Function));
        assert_eq!(read.cells[1].content, Some('+'));
        assert_eq!(read.cells[1].glyph, Some(crate::glyph::Glyph::Function));
        assert!(read.cells[2..].iter().all(|cell| cell.content.is_none()));
    }
}
