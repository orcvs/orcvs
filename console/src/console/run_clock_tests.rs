use super::panel::{format_beat_marker, format_run_clock, format_tick};
use std::time::Duration;

#[test]
fn run_clock_copy_is_mm_ss_until_an_hour_then_h_mm_ss() {
    let cases = [
        (0, "00:00"),
        (59, "00:59"),
        (60, "01:00"),
        (3599, "59:59"),
        (3600, "1:00:00"),
        (36000, "10:00:00"),
    ];
    for (seconds, copy) in cases {
        assert_eq!(
            format_run_clock(Duration::from_secs(seconds)),
            copy,
            "{seconds} seconds"
        );
    }
}

#[test]
fn tick_copy_is_five_zero_padded_digits() {
    assert_eq!(format_tick(0), "00000");
    assert_eq!(format_tick(1), "00001");
    assert_eq!(format_tick(99999), "99999");
}

#[test]
fn beat_marker_is_stars_when_the_engine_publishes_a_beat() {
    use orcvs::playback::PlaybackState;
    assert_eq!(format_beat_marker(PlaybackState::Playing, true), "**");
    assert_eq!(format_beat_marker(PlaybackState::Playing, false), "");
    assert_eq!(format_beat_marker(PlaybackState::Stopped, true), "//");
    assert_eq!(format_beat_marker(PlaybackState::Stopped, false), "//");
}
