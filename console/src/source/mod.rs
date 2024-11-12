pub mod expression_map;
pub mod source;
pub use expression_map::ExpressionMap;
pub use source::Source;
use source::SPACE;
use std::sync::{Arc, RwLock};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task,
};
use tracing::{error, info};

use crate::glyph::Glyph;

pub struct SourceCommander {
    inner: Arc<RwLock<Source>>,
}

impl SourceCommander {
    pub fn send(&self, cmd: Command) {
        let src = self.inner.read().unwrap();
        let snd = src.sender();

        tokio::spawn(async move {
            match snd.send(cmd).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Error {e:?}");
                }
            }
        });
    }

    pub fn sender(&self) -> Sender<Command> {
        let src = self.inner.read().unwrap();
        src.sender()
    }

    pub fn spawn(size: usize) -> Self {
        let (sender, mut receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel(16);

        let source = Arc::new(RwLock::new(Source::new(size, sender)));
        let clone = source.clone();

        task::spawn(async move {
            while let Some(cmd) = receiver.recv().await {
                match cmd {
                    Command::Set { idx, s } => {
                        info!("Command::Set {idx} [{s}]");
                        let mut src = clone.write().unwrap();
                        src.set(idx, &s);
                        // let src = source.to_string();
                        // let _ = responder.send(src);
                    }
                    Command::Unset { idx } => {
                        info!("Command::Unset {idx}");
                        let mut src = clone.write().unwrap();
                        src.set(idx, &SPACE);
                    }
                    Command::Tick => {
                        info!("Command::Tick");

                        let src = clone.read().unwrap();
                        src.execute();
                    }
                }
            }
        });

        Self { inner: source }
    }

    pub fn get(&self, idx: usize) -> (Option<String>, Option<Glyph>) {
        let source = self.inner.read().unwrap();
        let s = source.get(idx);
        let g = source.get_glyph_at(idx);
        (s, g)
    }
}

pub enum Command {
    Set { idx: usize, s: String },
    Unset { idx: usize },
    Tick,
}
