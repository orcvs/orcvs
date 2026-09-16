//! MIDI selection requests and nonblocking observation.
use super::{MidiConnection, MidiDestinationId, MidiError};
use crate::playback::PlaybackCommand;
use tokio::sync::{mpsc, watch};

///
/// What an output adapter publishes about its MIDI output: the destination it
/// is connected to.
///
/// Discovery belongs to the console on the thread that can perform it; the
/// engine publishes only the selected identity for the menu to read without
/// blocking.
///
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MidiDestinations {
    pub selected: Option<MidiDestinationId>,
}

pub(crate) enum MidiRequest {
    Install {
        destination_id: MidiDestinationId,
        connection: Box<dyn MidiConnection>,
    },
}

///
/// The MIDI configuration capability, without Playback lifecycle control.
///
/// It holds a weak sender rather than a clone of one, so that it cannot keep
/// the engine's task alive. Every method answers "running Orcvs is no longer
/// available" once the last `PlaybackEngine` has been dropped, or once the
/// publication channel reports that the task has ended while its owner still
/// lives — the weak sender and the publication channel draw that guarantee
/// together.
///
#[derive(Clone)]
pub struct MidiSelectionHandle {
    commands: mpsc::WeakUnboundedSender<PlaybackCommand<MidiRequest>>,
    destinations: watch::Receiver<MidiDestinations>,
}

impl MidiSelectionHandle {
    pub(crate) fn new(
        commands: mpsc::WeakUnboundedSender<PlaybackCommand<MidiRequest>>,
        destinations: watch::Receiver<MidiDestinations>,
    ) -> Self {
        Self {
            commands,
            destinations,
        }
    }

    fn ensure_available(&self) -> Result<(), MidiError> {
        if self.commands.strong_count() == 0 || self.destinations.has_changed().is_err() {
            return Err(MidiError::new("running Orcvs is no longer available"));
        }
        Ok(())
    }

    fn request(&self, request: MidiRequest) -> Result<(), MidiError> {
        self.ensure_available()?;
        let commands = self
            .commands
            .upgrade()
            .ok_or_else(|| MidiError::new("running Orcvs is no longer available"))?;
        commands
            .send(PlaybackCommand::Output(request))
            .map_err(|_| MidiError::new("running Orcvs is no longer available"))
    }

    ///
    /// Queues an already-open connection for the engine's task to install.
    ///
    /// The port was opened by the caller on the thread that could open it.
    /// What this returns is whether there is still a running Orcvs to queue
    /// the installation for.
    ///
    pub fn install(
        &self,
        destination_id: MidiDestinationId,
        connection: Box<dyn MidiConnection>,
    ) -> Result<(), MidiError> {
        self.request(MidiRequest::Install {
            destination_id,
            connection,
        })
    }

    ///
    /// The destination the engine last published, read without awaiting and
    /// without reaching the engine.
    ///
    pub fn selected_destination_id(&self) -> Result<Option<MidiDestinationId>, MidiError> {
        self.ensure_available()?;
        Ok(self.destinations.borrow().selected.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeConnection;

    impl MidiConnection for FakeConnection {
        fn send(&mut self, _message: &[u8]) -> Result<(), MidiError> {
            Ok(())
        }
    }

    #[test]
    fn install_honours_publication_unavailability() {
        let (commands, _receiver) = mpsc::unbounded_channel();
        let weak = commands.downgrade();
        let (destinations, destinations_rx) = watch::channel(MidiDestinations::default());
        drop(destinations);

        let handle = MidiSelectionHandle::new(weak, destinations_rx);
        let unavailable = MidiError::new("running Orcvs is no longer available");
        assert_eq!(
            handle.install(MidiDestinationId::new("studio"), Box::new(FakeConnection)),
            Err(unavailable)
        );
    }
}
