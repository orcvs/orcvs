//!
//! Which Panel status each Playback failure belongs to, and how long it is
//! shown.
//!

use orcvs::playback::{PlaybackDiagnostic, PlaybackObservation};

///
/// The failures the Panel shows beside the MIDI destination, the network
/// destination and Playback itself, from each frame's diagnostics and the
/// Playback observation.
///
#[derive(Default)]
pub struct PlaybackStatus {
    midi: Option<String>,
    network: Option<String>,
    engine: Option<String>,
    /// The observation the last frame saw, whose run start tells a new run
    /// from the one an engine failure belongs to.
    seen: PlaybackObservation,
}

impl PlaybackStatus {
    ///
    /// Takes this frame's drained `diagnostics` and the latest `observation`.
    ///
    pub fn observe(
        &mut self,
        diagnostics: Vec<PlaybackDiagnostic>,
        observation: &PlaybackObservation,
    ) {
        if !observation.midi_failed {
            self.midi = None;
        }
        if !observation.network_failed {
            self.network = None;
        }
        // A rewind while playing begins a new run without leaving Playing, so
        // the run is told apart by its start rather than by the state.
        if observation.run_started_at.is_some()
            && observation.run_started_at != self.seen.run_started_at
        {
            self.engine = None;
        }
        self.seen = *observation;
        for diagnostic in diagnostics {
            match diagnostic {
                PlaybackDiagnostic::OutputFailure(error) => self.midi = Some(error.message),
                PlaybackDiagnostic::NetworkFailure(error) => {
                    crate::report::error!("Playback failure: {}", error.message);
                    self.network = Some(error.message);
                }
                PlaybackDiagnostic::ClockFailure { message }
                | PlaybackDiagnostic::StartFailure { message }
                | PlaybackDiagnostic::RetuneFailure { message } => self.engine = Some(message),
                PlaybackDiagnostic::Overrun { .. } | PlaybackDiagnostic::Omitted(_) => {}
            }
        }
    }

    ///
    /// Forgets the network failure: it belongs to the destination the
    /// performer just left, and the new one has not been sent to.
    ///
    pub fn network_destination_changed(&mut self) {
        self.network = None;
    }

    /// The MIDI output failure the Panel shows.
    pub fn midi(&self) -> Option<&str> {
        self.midi.as_deref()
    }

    /// The failed UDP or OSC send the Panel shows.
    pub fn network(&self) -> Option<&str> {
        self.network.as_deref()
    }

    /// The clock, start or retune failure the Panel shows.
    pub fn engine(&self) -> Option<&str> {
        self.engine.as_deref()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::time::Duration;

    use orcvs::playback::{
        OmittedDiagnostics, OutputAdapterError, PlaybackDiagnostic, PlaybackObservation,
        PlaybackState,
    };

    use super::PlaybackStatus;

    fn output_failure(message: &str) -> PlaybackDiagnostic {
        PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(message))
    }

    fn midi_failed(failed: bool) -> PlaybackObservation {
        PlaybackObservation {
            midi_failed: failed,
            ..PlaybackObservation::default()
        }
    }

    fn network_failure(message: &str) -> PlaybackDiagnostic {
        PlaybackDiagnostic::NetworkFailure(OutputAdapterError::new(message))
    }

    fn network_failed(failed: bool) -> PlaybackObservation {
        PlaybackObservation {
            network_failed: failed,
            ..PlaybackObservation::default()
        }
    }

    #[test]
    fn each_output_failure_goes_to_its_own_destination() {
        let mut status = PlaybackStatus::default();

        status.observe(
            vec![
                network_failure("first"),
                output_failure("device lost"),
                network_failure("second"),
            ],
            &PlaybackObservation {
                midi_failed: true,
                network_failed: true,
                ..PlaybackObservation::default()
            },
        );

        assert_eq!(status.midi(), Some("device lost"));
        assert_eq!(status.network(), Some("second"));
    }

    #[test]
    fn a_network_failure_is_shown_until_sends_leave_or_the_destination_changes() {
        let mut status = PlaybackStatus::default();

        status.observe(vec![network_failure("send failed")], &network_failed(true));
        status.observe(vec![], &network_failed(true));
        assert_eq!(status.network(), Some("send failed"), "sends still fail");
        status.observe(vec![], &network_failed(false));
        assert_eq!(status.network(), None, "the last sends left");

        status.observe(vec![network_failure("send failed")], &network_failed(true));
        status.network_destination_changed();
        assert_eq!(status.network(), None);
        status.observe(vec![], &network_failed(true));
        assert_eq!(
            status.network(),
            None,
            "the new destination has not been sent to yet"
        );
    }

    fn playing_since(started: std::time::Instant) -> PlaybackObservation {
        PlaybackObservation {
            state: PlaybackState::Playing,
            run_started_at: Some(started),
            ..PlaybackObservation::default()
        }
    }

    #[test]
    fn an_engine_failure_is_shown_until_the_next_run_begins() {
        let mut status = PlaybackStatus::default();
        let first = std::time::Instant::now();

        status.observe(vec![], &playing_since(first));
        status.observe(
            vec![PlaybackDiagnostic::RetuneFailure {
                message: "too slow".to_owned(),
            }],
            &playing_since(first),
        );
        status.observe(vec![], &playing_since(first));
        assert_eq!(status.engine(), Some("too slow"), "the run goes on");

        status.observe(
            vec![PlaybackDiagnostic::ClockFailure {
                message: "clock stopped".to_owned(),
            }],
            &PlaybackObservation::default(),
        );
        assert_eq!(status.engine(), Some("clock stopped"), "the newer one");
        status.observe(
            vec![PlaybackDiagnostic::StartFailure {
                message: "cannot start".to_owned(),
            }],
            &PlaybackObservation::default(),
        );
        assert_eq!(status.engine(), Some("cannot start"));

        status.observe(vec![], &playing_since(first + Duration::from_secs(1)));
        assert_eq!(status.engine(), None);
        assert_eq!(status.midi(), None);
        assert_eq!(status.network(), None);
    }

    #[test]
    fn a_rewind_while_playing_begins_a_new_run() {
        let mut status = PlaybackStatus::default();
        let first = std::time::Instant::now();
        status.observe(
            vec![PlaybackDiagnostic::RetuneFailure {
                message: "too slow".to_owned(),
            }],
            &playing_since(first),
        );

        status.observe(vec![], &playing_since(first + Duration::from_secs(1)));

        assert_eq!(status.engine(), None);
    }

    #[test]
    fn an_overrun_or_an_omission_summary_is_no_failure() {
        let mut status = PlaybackStatus::default();

        status.observe(
            vec![
                PlaybackDiagnostic::Overrun {
                    scheduled_at: Duration::from_millis(750),
                    observed_at: Duration::from_millis(1_600),
                },
                PlaybackDiagnostic::Omitted(OmittedDiagnostics::default()),
            ],
            &PlaybackObservation::default(),
        );

        assert_eq!(
            [status.midi(), status.network(), status.engine()],
            [None, None, None]
        );
    }

    #[test]
    fn a_midi_failure_is_shown_until_the_observation_clears_it() {
        let mut status = PlaybackStatus::default();

        status.observe(vec![output_failure("device lost")], &midi_failed(true));
        assert_eq!(status.midi(), Some("device lost"));

        status.observe(vec![], &midi_failed(true));
        assert_eq!(status.midi(), Some("device lost"));

        status.observe(vec![], &midi_failed(false));
        assert_eq!(status.midi(), None);
    }
}
