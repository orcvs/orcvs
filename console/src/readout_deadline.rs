//!
//! When the next Render Frame should run so Run Clock glyphs and Cursor Effect
//! stay honest without a second Tick clock.
//!

use std::time::Duration;

///
/// Delay until the next timed Render Frame from this pass's samples.
///
/// `run_clock` is the Duration this Render Frame already painted, `Some` only
/// while that Readout is moving. The Run Clock term is the remainder until the
/// next whole second of that sample, plus `predicted_dt` so egui's early-wake
/// subtraction lands on the boundary. The Cursor Effect term is the remaining
/// delay already computed for this pass and is not offset.
///
pub(crate) fn until_next(
    run_clock: Option<Duration>,
    cursor_effect: Option<Duration>,
    predicted_dt: Duration,
) -> Option<Duration> {
    let run_clock = run_clock.map(|clock| {
        let into_second = clock - Duration::from_secs(clock.as_secs());
        Duration::from_secs(1) - into_second + predicted_dt
    });
    [run_clock, cursor_effect].into_iter().flatten().min()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::until_next;

    #[test]
    fn until_next_answers_the_remainder_to_the_next_whole_second() {
        assert_eq!(
            until_next(Some(Duration::from_millis(900)), None, Duration::ZERO),
            Some(Duration::from_millis(100))
        );
    }

    #[test]
    fn an_exact_whole_second_waits_one_second() {
        assert_eq!(
            until_next(Some(Duration::from_secs(1)), None, Duration::ZERO),
            Some(Duration::from_secs(1))
        );
    }

    #[test]
    fn predicted_frame_time_is_added_only_to_the_run_clock_term() {
        assert_eq!(
            until_next(
                Some(Duration::from_millis(900)),
                None,
                Duration::from_millis(16)
            ),
            Some(Duration::from_millis(116))
        );
    }

    #[test]
    fn cursor_effect_remaining_delay_is_not_offset() {
        assert_eq!(
            until_next(
                None,
                Some(Duration::from_millis(50)),
                Duration::from_millis(16)
            ),
            Some(Duration::from_millis(50))
        );
    }

    #[test]
    fn neither_term_requests_no_timed_wake() {
        assert_eq!(until_next(None, None, Duration::from_millis(16)), None);
        assert_eq!(until_next(None, None, Duration::ZERO), None);
        assert_eq!(until_next(None, None, Duration::from_secs(1)), None);
    }

    #[test]
    fn both_terms_request_the_sooner_delay() {
        assert_eq!(
            until_next(
                Some(Duration::from_millis(900)),
                Some(Duration::from_millis(50)),
                Duration::from_millis(16)
            ),
            Some(Duration::from_millis(50)),
            "the Cursor Effect term is sooner than the compensated Run Clock remainder"
        );
        assert_eq!(
            until_next(
                Some(Duration::from_millis(900)),
                Some(Duration::from_millis(200)),
                Duration::ZERO
            ),
            Some(Duration::from_millis(100)),
            "the Run Clock remainder is sooner than the Cursor Effect term"
        );
    }
}
