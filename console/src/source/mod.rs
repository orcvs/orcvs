pub mod error;
mod expression_map;
pub mod source;
use crate::{glyph::Glyph, grid::Grid};
pub use error::SourceError;
pub use source::{Cell, CellWrite, Change, Diagnostic, PlayCommand, Source, TickPlan, TickResult};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SourceCommander {
    inner: Arc<RwLock<Source>>,
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

    pub(crate) fn execute(&self) -> TickResult {
        self.inner.write().unwrap().execute()
    }
}
