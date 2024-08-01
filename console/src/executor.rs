// use futures::{stream, StreamExt};
use std::{
    future::Future,
    time::{Duration, Instant},
};
use tokio::{task, time};
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct Executor {
    bpm: usize,
    // state: State,
    // callback: Box<dyn Fn() + Send + Sync>,
    // callback: Box<dyn Fn(String) -> dyn Future<Output = ()>>,
    token: Option<CancellationToken>,
}

impl Default for Executor {
    fn default() -> Self {
        Executor {
            bpm: 120,
            token: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Play,
    Pause,
    Stop,
}

impl Executor {
    fn duration(&self) -> u64 {
        let ms = (60000 / self.bpm) / 4;
        ms as u64
    }

    // async fn play(&mut self) {
    //     self._task();
    //     if let Some(ref mut handle) = self.handle {
    //         handle.await;
    //     }
    // }

    async fn pause(&mut self) {
        if let Some(token) = &self.token {
            token.cancel();
        }
    }

    fn stop(&mut self) {
        info!("stop");
        if let Some(token) = &self.token {
            info!("stop.cancel");
            token.cancel();
        }
    }

    async fn play(&mut self) {
        let token = CancellationToken::new();
        let cln_token = token.clone();
        self.token = Some(token);

        let ms = self.duration();
        // info!("{ms}");
        // let state = self.state;
        task::spawn(async move {
            info!("spawn");
            tokio::select! {
                _ = cln_token.cancelled() => {
                    info!("cancelled");
                }
                _ = Self::ticker(ms) => {
                    info!("done");
                }
            }
        });
        info!("here");
    }

    async fn ticker(ms: u64) {
        let mut interval = time::interval(Duration::from_millis(ms));
        loop {
            info!("interval");
            interval.tick().await;
            // tick().await;
        }
    }
}

async fn tick() {
    info!("tick");
}

#[cfg(test)]
mod test {

    use std::time::Duration;

    use tracing::info;

    use crate::test::trace;

    use super::Executor;

    // #[tokio::test(start_paused = true)]
    #[tokio::test]
    async fn test_executor() {
        trace();
        // tokio::time::pause();
        // tokio::time::advance();
        let mut exec = Executor::default();
        // exec.bpm = 1;

        // tokio::spawn(async move {});
        exec.play().await;
        info!("HERE");
        tokio::time::sleep(Duration::from_millis(1000)).await;
        exec.stop();

        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
