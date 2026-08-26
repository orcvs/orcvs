pub mod error;
mod expression_map;
pub mod source;
use crate::{glyph::Glyph, grid::Grid};
pub use error::SourceError;
pub use source::{Cell, CellWrite, Change, Diagnostic, PlayCommand, Source, TickPlan, TickResult};
use std::sync::{Arc, RwLock};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task,
};
use tracing::info;

pub struct SourceCommander {
    inner: Arc<RwLock<Source>>,
    sender: Sender<Command>,
}

impl SourceCommander {
    pub fn spawn(grid: Grid) -> Self {
        let (sender, mut receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel(16);

        let source = Arc::new(RwLock::new(Source::new(grid)));
        let clone = source.clone();

        task::spawn(async move {
            while let Some(cmd) = receiver.recv().await {
                match cmd {
                    Command::Tick => {
                        info!("Command::Tick");

                        let mut src = clone.write().unwrap();
                        src.execute();
                    }
                }
            }
        });

        Self {
            inner: source,
            sender,
        }
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

    pub fn sender(&self) -> Sender<Command> {
        self.sender.clone()
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
}

pub enum Command {
    Tick,
}
