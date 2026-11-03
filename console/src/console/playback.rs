//! The console's Playback start policy: one home for whether a request from
//! Space or the Panel's `P` toggles Playback now, waits for the page's audio,
//! or cancels a waiting start (ADR 0087, ADR 0089).

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Console routes Space and the Panel's P through this once both callers are wired"
    )
)]

/// The longest a start waits for the page's audio, in seconds (ADR 0087).
pub(super) const AUDIO_WAIT: f64 = 0.5;

///
/// What a Playback request or a frame's poll asks of the console.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Decision {
    /// Toggle Playback now.
    Toggle,
    /// A start waits for the page's audio; paint again to release it.
    Held,
    /// Nothing to do.
    Nothing,
}

///
/// Holds a Playback start while the page's audio wakes.
///
#[derive(Debug, Default)]
pub(super) struct PlaybackStart {
    /// When a held start was asked for, on egui's input clock.
    held_since: Option<f64>,
}

impl PlaybackStart {
    ///
    /// Answers a request from Space or `P` at `now`, on egui's input clock.
    /// `early` is whether a start now would sound before the page's audio
    /// runs.
    ///
    pub(super) fn request(&mut self, now: f64, early: bool) -> Decision {
        if early {
            self.held_since = Some(now);
            Decision::Held
        } else {
            Decision::Toggle
        }
    }

    ///
    /// Answers a frame at `now`: releases a held start once the page's audio
    /// is `awake` or [`AUDIO_WAIT`] has passed.
    ///
    pub(super) fn poll(&mut self, now: f64, awake: bool) -> Decision {
        match self.held_since {
            None => Decision::Nothing,
            Some(since) if awake || now - since >= AUDIO_WAIT => {
                self.held_since = None;
                Decision::Toggle
            }
            Some(_) => Decision::Held,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AUDIO_WAIT, Decision, PlaybackStart};

    #[test]
    fn an_early_request_is_held_rather_than_granted() {
        let mut start = PlaybackStart::default();
        assert_eq!(start.request(1.0, true), Decision::Held);
        assert_eq!(start.poll(1.1, false), Decision::Held);
    }

    #[test]
    fn a_held_start_is_granted_once_the_audio_wakes() {
        let mut start = PlaybackStart::default();
        start.request(1.0, true);
        assert_eq!(start.poll(1.1, true), Decision::Toggle);
        assert_eq!(start.poll(1.2, true), Decision::Nothing);
    }

    #[test]
    fn a_held_start_is_granted_once_it_has_waited_long_enough() {
        let mut start = PlaybackStart::default();
        start.request(1.0, true);
        assert_eq!(start.poll(1.0 + AUDIO_WAIT - 0.01, false), Decision::Held);
        assert_eq!(start.poll(1.0 + AUDIO_WAIT, false), Decision::Toggle);
    }

    #[test]
    fn a_request_that_is_not_early_toggles_at_once() {
        let mut start = PlaybackStart::default();
        assert_eq!(start.request(1.0, false), Decision::Toggle);
        assert_eq!(start.poll(1.1, false), Decision::Nothing);
    }
}
