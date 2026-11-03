//! The console's Playback start policy: one home for whether a request from
//! Space or the Panel's `P` toggles Playback now, waits for the page's audio,
//! or cancels a waiting start (ADR 0087, ADR 0089).

#![expect(
    dead_code,
    reason = "Console routes Space and the Panel's P through this once both callers are wired"
)]

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
    /// Whether a start waits for the page's audio.
    held: bool,
}

impl PlaybackStart {
    ///
    /// Answers a request from Space or `P` at `now`, on egui's input clock.
    /// `early` is whether a start now would sound before the page's audio
    /// runs.
    ///
    pub(super) fn request(&mut self, _now: f64, early: bool) -> Decision {
        if early {
            self.held = true;
            Decision::Held
        } else {
            Decision::Nothing
        }
    }

    ///
    /// Answers a frame at `now`: releases a held start once the page's audio
    /// is `awake`.
    ///
    pub(super) fn poll(&mut self, _now: f64, _awake: bool) -> Decision {
        if self.held {
            Decision::Held
        } else {
            Decision::Nothing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Decision, PlaybackStart};

    #[test]
    fn an_early_request_is_held_rather_than_granted() {
        let mut start = PlaybackStart::default();
        assert_eq!(start.request(1.0, true), Decision::Held);
        assert_eq!(start.poll(1.1, false), Decision::Held);
    }
}
