use orcvs::playback::PlaybackDiagnostic;

///
/// The message the shell shows for a Playback diagnostic, or `None` when the
/// diagnostic reports no failure. An Overrun is a late Tick the Playback Engine
/// deliberately skips, so it is not a failure and the user is not told about it.
///
/// Every platform reports through this one decision, so a diagnostic cannot
/// count as a failure in the browser and as routine on the desktop.
///
pub(crate) fn failure_message(diagnostic: &PlaybackDiagnostic) -> Option<String> {
    match diagnostic {
        PlaybackDiagnostic::OutputFailure(error) => Some(error.message.clone()),
        PlaybackDiagnostic::ClockFailure { message }
        | PlaybackDiagnostic::StartFailure { message }
        | PlaybackDiagnostic::RetuneFailure { message } => Some(message.clone()),
        PlaybackDiagnostic::Overrun { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use orcvs::playback::{OutputAdapterError, PlaybackDiagnostic};

    use super::failure_message;

    #[test]
    fn an_overrun_is_not_a_failure() {
        assert_eq!(
            failure_message(&PlaybackDiagnostic::Overrun {
                scheduled_at: Duration::from_millis(750),
                observed_at: Duration::from_millis(1_600),
            }),
            None
        );
    }

    #[test]
    fn output_clock_and_start_failures_carry_their_message() {
        assert_eq!(
            failure_message(&PlaybackDiagnostic::OutputFailure(OutputAdapterError::new(
                "device lost"
            ))),
            Some("device lost".to_owned())
        );
        assert_eq!(
            failure_message(&PlaybackDiagnostic::ClockFailure {
                message: "Playback clock terminated unexpectedly".to_owned(),
            }),
            Some("Playback clock terminated unexpectedly".to_owned())
        );
        assert_eq!(
            failure_message(&PlaybackDiagnostic::StartFailure {
                message: "Tick period must be greater than zero".to_owned(),
            }),
            Some("Tick period must be greater than zero".to_owned())
        );
    }
}
