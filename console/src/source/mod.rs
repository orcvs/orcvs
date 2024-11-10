pub mod expression_map;
pub mod source;

pub use expression_map::ExpressionMap;
pub use source::Source;
use std::{borrow::Cow, fmt, sync::Arc};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    task,
};
use tracing::info;

pub enum Command {
    Set {
        idx: usize,
        s: String,
        // responder: oneshot::Sender<Cow<'a, String>>,
        // responder: oneshot::Sender<String>,
    },
    Tick,
}

pub fn source(size: usize) -> Arc<Source> {
    let (sender, mut receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel(16);

    let source = Arc::new(Source::new(size, sender));
    let source_clone = source.clone();

    task::spawn(async move {
        while let Some(cmd) = receiver.recv().await {
            match cmd {
                Command::Set { idx, s } => {
                    info!("Command::Set {idx:?} [{s}]");
                    info!("Source {source_clone}]");
                    // let src = source.to_string();
                    // let _ = responder.send(src);
                }
                Command::Tick => {
                    info!("Command::Tick");
                }
            }
        }
    });

    source
}
