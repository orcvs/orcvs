//! MIDI selection requests and nonblocking observation.
use super::{MidiDestination, MidiDestinationId, MidiError};
use crate::playback::PlaybackCommand;
use tokio::sync::{mpsc, watch};

///
/// What an output adapter publishes about its MIDI destinations: the ones the
/// last discovery found, or the failure it reported, and the one the adapter is
/// connected to.
///
/// One value rather than two channels, because the console reads both while
/// drawing one frame and a menu drawn from two channels can show a checkmark
/// against a row the other channel has already withdrawn. ADR 0041 has the
/// engine's task own the adapter, so this is the whole of what a caller can see
/// of it without asking.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MidiDestinations {
    pub discovered: Result<Vec<MidiDestination>, MidiError>,
    pub selected: Option<MidiDestinationId>,
}

impl Default for MidiDestinations {
    ///
    /// What an adapter publishes before anything has asked it to look: no
    /// destinations found, because none have been looked for, and none
    /// selected.
    ///
    fn default() -> Self {
        Self {
            discovered: Ok(Vec::new()),
            selected: None,
        }
    }
}

pub(crate) enum MidiRequest {
    Discover,
    Select(MidiDestinationId),
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

    ///
    /// Queues `request` for the engine's task, or answers that there is no
    /// longer a running Orcvs to queue it for.
    ///
    fn ensure_available(&self) -> Result<(), crate::midi::MidiError> {
        if self.commands.strong_count() == 0 || self.destinations.has_changed().is_err() {
            return Err(crate::midi::MidiError::new(
                "running Orcvs is no longer available",
            ));
        }
        Ok(())
    }

    fn request(&self, request: MidiRequest) -> Result<(), crate::midi::MidiError> {
        self.ensure_available()?;
        let commands = self
            .commands
            .upgrade()
            .ok_or_else(|| crate::midi::MidiError::new("running Orcvs is no longer available"))?;
        commands
            .send(PlaybackCommand::Output(request))
            .map_err(|_| crate::midi::MidiError::new("running Orcvs is no longer available"))
    }

    ///
    /// Asks the engine to discover the destinations its backend offers.
    ///
    /// Discovery reaches a platform MIDI service through the adapter, which
    /// the engine's task owns, so this asks rather than answers: what it
    /// found — or the failure it reported — arrives through
    /// [`destinations`](Self::destinations) once the task has run.
    ///
    pub fn refresh_destinations(&self) -> Result<(), crate::midi::MidiError> {
        self.request(MidiRequest::Discover)
    }

    ///
    /// The destinations the engine last published, or the failure the last
    /// discovery reported.
    ///
    /// Read from the published value rather than asked of the engine, for the
    /// reason every other observation is: the console compares this list
    /// against its menu while drawing a frame, and the browser main thread has
    /// no blocking receive with which to wait for an answer.
    ///
    pub fn destinations(
        &self,
    ) -> Result<Vec<crate::midi::MidiDestination>, crate::midi::MidiError> {
        self.ensure_available()?;
        self.destinations.borrow().discovered.clone()
    }

    ///
    /// Marks the published destinations as seen at their current value.
    ///
    /// Call this when a refresh is queued so a later
    /// [`destinations_changed`](Self::destinations_changed) distinguishes the
    /// answer from the still-stale publication.
    ///
    pub fn mark_destinations_seen(&mut self) {
        let _ = self.destinations.borrow_and_update();
    }

    ///
    /// Whether the engine has published a new destinations value since the last
    /// [`mark_destinations_seen`](Self::mark_destinations_seen).
    ///
    pub fn destinations_changed(&mut self) -> bool {
        self.destinations.has_changed().unwrap_or(false)
    }

    ///
    /// Asks the engine to connect its output to `destination_id`.
    ///
    /// The answer this returns is whether there is still a running Orcvs to
    /// ask. Whether the device accepted the connection is the engine's to
    /// report: a refusal becomes a Playback diagnostic, on the one ordered
    /// stream every other output failure is reported on, and the selection
    /// that succeeded appears in the published destinations.
    ///
    pub fn select(
        &self,
        destination_id: &crate::midi::MidiDestinationId,
    ) -> Result<(), crate::midi::MidiError> {
        self.request(MidiRequest::Select(destination_id.clone()))
    }

    ///
    /// The destination the engine last published, read without awaiting and
    /// without reaching the engine.
    ///
    /// The console compares this against every row of its menu while drawing a
    /// frame, which is why it is read from the published value rather than
    /// asked of the engine. The weak sender detects owner loss immediately,
    /// before the task has closed publication; the publication channel also
    /// detects a task that ended while its owner remains alive.
    ///
    pub fn selected_destination_id(
        &self,
    ) -> Result<Option<crate::midi::MidiDestinationId>, crate::midi::MidiError> {
        self.ensure_available()?;
        Ok(self.destinations.borrow().selected.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_honour_publication_unavailability() {
        let (commands, _receiver) = mpsc::unbounded_channel();
        let weak = commands.downgrade();
        let (destinations, destinations_rx) = watch::channel(MidiDestinations::default());
        drop(destinations);

        let handle = MidiSelectionHandle::new(weak, destinations_rx);
        let unavailable = MidiError::new("running Orcvs is no longer available");
        assert_eq!(handle.refresh_destinations(), Err(unavailable.clone()));
        assert_eq!(
            handle.select(&MidiDestinationId::new("studio")),
            Err(unavailable)
        );
    }
}
