use std::num::NonZeroUsize;
use std::time::Duration;

use lang::Tick;

pub const DEFAULT_FONT_SIZE: f32 = 11.5;
pub const DEFAULT_SECTOR_SEAM_SPACING: usize = 8;

pub const DEFAULT_CURSOR_BLOOM_RADIUS: usize = 7;

pub const DEFAULT_CURSOR_DELAY: u64 = 800;

/// Ticks that make one displayed BPM beat. [`Bpm::tick_period`] is one of these.
const TICKS_PER_BEAT: usize = 4;

/// One Tick at 1 BPM, in nanoseconds: a minute over [`TICKS_PER_BEAT`]. Every
/// Tick period is this divided by the BPM.
const QUARTER_MINUTE_NANOS: u64 = 60_000_000_000 / TICKS_PER_BEAT as u64;

/// The fastest tempo, whose Tick period is one millisecond.
const MAX_BPM: usize = (QUARTER_MINUTE_NANOS / 1_000_000) as usize;

///
/// How the console presents and plays a Source. Nothing in this file is a
/// Source dimension: column and row counts belong to the Grid, which is the
/// only thing that states them. `sector_seam_spacing` counts the Sector Seam
/// period in Cells; it is not a Source dimension.
///
#[derive(Clone, Debug)]
pub struct Opts {
    pub bpm: Bpm,
    pub cursor_delay: u64,
    pub cursor_bloom_radius: CursorBloomRadius,
    pub sector_seam_spacing: SectorSeamSpacing,
    pub mode: Mode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Insert,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bpm(NonZeroUsize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectorSeamSpacing(NonZeroUsize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorBloomRadius(NonZeroUsize);

impl CursorBloomRadius {
    pub fn new(cells: usize) -> Option<Self> {
        NonZeroUsize::new(cells).map(Self)
    }

    pub fn cells(self) -> usize {
        self.0.get()
    }
}

impl SectorSeamSpacing {
    pub fn new(cells: usize) -> Option<Self> {
        NonZeroUsize::new(cells).map(Self)
    }

    pub fn cells(self) -> usize {
        self.0.get()
    }
}

impl Bpm {
    pub fn new(beats_per_minute: usize) -> Option<Self> {
        NonZeroUsize::new(beats_per_minute)
            .filter(|bpm| bpm.get() <= MAX_BPM)
            .map(Self)
    }

    ///
    /// One Tick at this tempo: a quarter of a minute over the BPM, 125 ms at
    /// 120 BPM.
    ///
    /// The period is rounded to the nearest nanosecond, halves up, so it is
    /// within half a nanosecond of the mathematical period. Playback adds the
    /// same period once per Tick, so a run drifts from the mathematical tempo
    /// by at most half a nanosecond per Tick: under 16 µs over an hour at
    /// 130 BPM. Do not round to whole milliseconds: at 130 BPM that plays
    /// 0.33% fast, and every BPM above 7500 collapses onto a 1 ms period.
    ///
    pub fn tick_period(self) -> Duration {
        let beats_per_minute = self.0.get() as u64;
        Duration::from_nanos((QUARTER_MINUTE_NANOS + beats_per_minute / 2) / beats_per_minute)
    }

    pub fn beats_per_minute(self) -> usize {
        self.0.get()
    }

    ///
    /// Whether `tick` is a displayed BPM beat.
    ///
    /// [`Self::tick_period`] is one Tick of that grouping, so Tick `0` and every
    /// multiple of the grouping after it are beats; the Ticks between them
    /// are not.
    ///
    pub fn on_beat(tick: Tick) -> bool {
        tick.get().is_multiple_of(TICKS_PER_BEAT as u64)
    }
}

impl Opts {
    pub fn new() -> Self {
        Self {
            bpm: Bpm::new(120).expect("default tempo is positive"),
            cursor_delay: DEFAULT_CURSOR_DELAY,
            cursor_bloom_radius: CursorBloomRadius::new(DEFAULT_CURSOR_BLOOM_RADIUS)
                .expect("default cursor bloom radius is positive"),
            sector_seam_spacing: SectorSeamSpacing::new(DEFAULT_SECTOR_SEAM_SPACING)
                .expect("default sector seam spacing is positive"),
            mode: Mode::Insert,
        }
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Bpm, CursorBloomRadius, DEFAULT_CURSOR_BLOOM_RADIUS, MAX_BPM, Opts, QUARTER_MINUTE_NANOS,
        SectorSeamSpacing, TICKS_PER_BEAT,
    };
    use lang::Tick;
    use std::time::Duration;

    fn bpm(beats_per_minute: usize) -> Bpm {
        Bpm::new(beats_per_minute).expect("an accepted BPM")
    }

    ///
    /// How far `ticks` Ticks of `bpm`'s period land from the same number of
    /// mathematical periods, in nanoseconds. The exact period is a quarter of
    /// a minute over the BPM, so both sides are scaled by the BPM to stay in
    /// whole numbers; the difference is then divided back out, rounding up so
    /// the answer never understates the error.
    ///
    fn drift_nanos(bpm: Bpm, ticks: u32) -> u128 {
        let beats_per_minute = bpm.beats_per_minute() as u128;
        let played = (bpm.tick_period() * ticks).as_nanos() * beats_per_minute;
        let exact = u128::from(QUARTER_MINUTE_NANOS) * u128::from(ticks);
        played.abs_diff(exact).div_ceil(beats_per_minute)
    }

    #[test]
    fn bpm_accepts_only_positive_tick_rates() {
        assert_eq!(
            Bpm::new(20).map(Bpm::tick_period),
            Some(Duration::from_millis(750))
        );
        assert_eq!(Bpm::new(0), None);
        assert_eq!(Bpm::new(15_001), None);
    }

    #[test]
    fn default_bpm_is_one_hundred_and_twenty() {
        assert_eq!(Opts::new().bpm.beats_per_minute(), 120);
        assert_eq!(Opts::new().bpm.tick_period(), Duration::from_millis(125));
    }

    ///
    /// 130 and 9000 do not divide a quarter minute into whole milliseconds,
    /// and 15000 is the fastest accepted tempo. Each period is the nearest
    /// nanosecond to a quarter minute over the BPM.
    ///
    #[test]
    fn a_tick_period_keeps_the_fraction_of_a_millisecond() {
        assert_eq!(bpm(1).tick_period(), Duration::from_secs(15));
        assert_eq!(bpm(130).tick_period(), Duration::from_nanos(115_384_615));
        assert_eq!(bpm(9_000).tick_period(), Duration::from_nanos(1_666_667));
        assert_eq!(bpm(7_501).tick_period(), Duration::from_nanos(1_999_733));
        assert_eq!(bpm(15_000).tick_period(), Duration::from_millis(1));
    }

    ///
    /// Every accepted BPM, not a sample: the period is within half a
    /// nanosecond of the mathematical one, well inside a microsecond.
    ///
    #[test]
    fn every_tick_period_is_within_half_a_nanosecond_of_a_quarter_minute_over_the_bpm() {
        for beats_per_minute in 1..=MAX_BPM {
            let bpm = bpm(beats_per_minute);
            let scaled = bpm.tick_period().as_nanos() * beats_per_minute as u128;
            let error = scaled.abs_diff(u128::from(QUARTER_MINUTE_NANOS));
            assert!(
                2 * error <= beats_per_minute as u128,
                "{beats_per_minute} BPM is {error}/{beats_per_minute} ns from its period"
            );
        }
    }

    ///
    /// The rounding error of a fixed period accumulates once per Tick, so an
    /// hour of Ticks stays within the half nanosecond per Tick that rounding
    /// to the nearest promises. An hour at 130 BPM is 31200 Ticks, so it lands
    /// within 15.6 µs; a whole-millisecond period would land 12 seconds early.
    ///
    #[test]
    fn an_hour_of_ticks_drifts_at_most_half_a_nanosecond_per_tick() {
        for beats_per_minute in [130, 9_000, MAX_BPM] {
            let bpm = bpm(beats_per_minute);
            let ticks = u32::try_from(beats_per_minute * TICKS_PER_BEAT * 60)
                .expect("an hour of Ticks fits in u32");
            let drift = drift_nanos(bpm, ticks);
            assert!(
                2 * drift <= u128::from(ticks),
                "{beats_per_minute} BPM drifts {drift} ns over {ticks} Ticks"
            );
        }
        assert!(drift_nanos(bpm(130), 31_200) <= 15_600);
        assert_eq!(drift_nanos(bpm(MAX_BPM), 3_600_000), 0);
    }

    #[test]
    fn a_displayed_beat_is_every_fourth_tick() {
        assert!(Bpm::on_beat(Tick::ZERO));
        assert!(!Bpm::on_beat(Tick::new(1)));
        assert!(!Bpm::on_beat(Tick::new(2)));
        assert!(!Bpm::on_beat(Tick::new(3)));
        assert!(Bpm::on_beat(Tick::new(4)));
        assert!(Bpm::on_beat(Tick::new(8)));
    }

    #[test]
    fn default_cursor_field_reaches_seven_cells_from_the_cursor() {
        assert_eq!(
            Opts::default().cursor_bloom_radius.cells(),
            DEFAULT_CURSOR_BLOOM_RADIUS
        );
        assert_eq!(DEFAULT_CURSOR_BLOOM_RADIUS, 7);
    }

    #[test]
    fn sector_seam_spacing_accepts_only_whole_positive_cell_counts() {
        assert_eq!(
            SectorSeamSpacing::new(1).map(SectorSeamSpacing::cells),
            Some(1)
        );
        assert_eq!(
            SectorSeamSpacing::new(8).map(SectorSeamSpacing::cells),
            Some(8)
        );
        assert_eq!(SectorSeamSpacing::new(0), None);
    }

    #[test]
    fn cursor_bloom_radius_accepts_only_whole_positive_cell_counts() {
        assert_eq!(
            CursorBloomRadius::new(2).map(CursorBloomRadius::cells),
            Some(2)
        );
        assert_eq!(CursorBloomRadius::new(0), None);
    }
}
