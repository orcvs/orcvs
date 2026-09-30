//! Shared ownership of asynchronous browser ports. Browser callbacks and tests
//! enter the same lifecycle; only the actual open, close and send are adapters.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::task::Poll;

use orcvs::midi::{MidiConnection, MidiConnectionRequest, MidiError, PendingMidiConnection};

use super::{DESTINATION_GONE, WebMidiAccess};

#[derive(Clone)]
pub(super) struct Ports<A> {
    access: A,
    state: Arc<Mutex<HashMap<String, Port>>>,
}

struct Port {
    claims: usize,
    phase: Phase,
}

#[derive(Clone)]
enum Phase {
    Opening,
    Open,
    Refused(MidiError),
    Closing,
}

impl<A: WebMidiAccess> Ports<A> {
    pub(super) fn new(access: A) -> Self {
        Self {
            access,
            state: Arc::default(),
        }
    }

    pub(super) fn access(&self) -> &A {
        &self.access
    }

    pub(super) fn acquire(&self, id: String) -> MidiConnectionRequest {
        let start = {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let start = !state.contains_key(&id);
            let port = state.entry(id.clone()).or_insert(Port {
                claims: 0,
                phase: Phase::Opening,
            });
            port.claims += 1;
            start
        };
        if start {
            self.open(&id);
        }
        MidiConnectionRequest::Pending(Box::new(Opening(Some(Claim {
            ports: self.clone(),
            id,
        }))))
    }

    fn open(&self, id: &str) {
        let ports = self.clone();
        let owned_id = id.to_owned();
        self.access
            .open(id, Box::new(move |answer| ports.opened(&owned_id, answer)));
    }

    fn opened(&self, id: &str, answer: Result<(), MidiError>) {
        let close = {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let port = state
                .get_mut(id)
                .expect("an opening retains its port until completion");
            match answer {
                Ok(()) if port.claims == 0 => {
                    port.phase = Phase::Closing;
                    true
                }
                Ok(()) => {
                    port.phase = Phase::Open;
                    false
                }
                Err(error) => {
                    if port.claims == 0 {
                        state.remove(id);
                    } else {
                        port.phase = Phase::Refused(error);
                    }
                    false
                }
            }
        };
        if close {
            self.close(id);
        }
    }

    fn release(&self, id: &str) {
        let close = {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let port = state.get_mut(id).expect("a claim retains its port");
            port.claims -= 1;
            if port.claims != 0 {
                return;
            }
            match port.phase {
                Phase::Open => {
                    port.phase = Phase::Closing;
                    true
                }
                Phase::Refused(_) => {
                    state.remove(id);
                    false
                }
                Phase::Opening | Phase::Closing => false,
            }
        };
        if close {
            self.close(id);
        }
    }

    fn close(&self, id: &str) {
        let ports = self.clone();
        let owned_id = id.to_owned();
        self.access.close(
            id,
            Box::new(move |answer| {
                if let Err(error) = answer {
                    tracing::warn!("MIDI close refused: {}", error.message);
                }
                let reopen = {
                    let mut state = ports.state.lock().unwrap_or_else(PoisonError::into_inner);
                    let port = state
                        .get_mut(&owned_id)
                        .expect("a closing retains its port until completion");
                    if port.claims == 0 {
                        state.remove(&owned_id);
                        false
                    } else {
                        port.phase = Phase::Opening;
                        true
                    }
                };
                // A new claim waits for an in-flight close before opening again.
                if reopen {
                    ports.open(&owned_id);
                }
            }),
        );
    }
}

struct Claim<A: WebMidiAccess> {
    ports: Ports<A>,
    id: String,
}

impl<A: WebMidiAccess> Drop for Claim<A> {
    fn drop(&mut self) {
        self.ports.release(&self.id);
    }
}

struct Opening<A: WebMidiAccess>(Option<Claim<A>>);

impl<A: WebMidiAccess> PendingMidiConnection for Opening<A> {
    fn poll(&mut self) -> Poll<Result<Box<dyn MidiConnection>, MidiError>> {
        let claim = self
            .0
            .as_ref()
            .expect("an opening is polled only until ready");
        if !claim
            .ports
            .access
            .outputs()
            .iter()
            .any(|output| output.id == claim.id && output.connected)
        {
            return Poll::Ready(Err(MidiError::new(DESTINATION_GONE)));
        }
        let phase = claim
            .ports
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&claim.id)
            .expect("a claim retains its port")
            .phase
            .clone();
        match phase {
            Phase::Opening | Phase::Closing => Poll::Pending,
            Phase::Refused(error) => Poll::Ready(Err(error)),
            Phase::Open => Poll::Ready(Ok(Box::new(Connection(
                self.0.take().expect("the opening owns its claim"),
            )))),
        }
    }
}

struct Connection<A: WebMidiAccess>(Claim<A>);

impl<A: WebMidiAccess> MidiConnection for Connection<A> {
    fn send(&mut self, message: &[u8]) -> Result<(), MidiError> {
        self.0.ports.access.send(&self.0.id, message)
    }
}
