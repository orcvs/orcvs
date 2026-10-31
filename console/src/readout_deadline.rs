//!
//! When the next Render Frame should run so Run Clock glyphs stay honest
//! without a second Tick clock.
//!

use std::time::Duration;

///
/// Delay until the next timed Render Frame from this pass's samples.
///
/// `run_clock` is the Duration this Render Frame already painted. The delay is
/// the remainder until the next whole second of that sample; egui wakes after
/// the delay it is asked for, so no frame time is added to it.
///
pub(crate) fn until_next(run_clock: Duration) -> Duration {
    let into_second = run_clock - Duration::from_secs(run_clock.as_secs());
    Duration::from_secs(1) - into_second
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::until_next;

    #[test]
    fn until_next_answers_the_remainder_to_the_next_whole_second() {
        assert_eq!(
            until_next(Duration::from_millis(900)),
            Duration::from_millis(100)
        );
    }

    #[test]
    fn an_exact_whole_second_waits_one_second() {
        assert_eq!(until_next(Duration::from_secs(1)), Duration::from_secs(1));
    }
}
