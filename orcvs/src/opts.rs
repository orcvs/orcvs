use std::num::NonZeroUsize;

pub const DEFAULT_FONT_SIZE: f32 = 18.0;
pub const DEFAULT_SECTOR_SEAM_SPACING: usize = 8;

pub const DEFAULT_CURSOR_BLOOM_RADIUS: usize = 7;

pub const DEFAULT_CURSOR_DELAY: u64 = 800;

const MAX_BPM: usize = 60_000 / 4;

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

    pub fn delay_ms(&self) -> u64 {
        let ms = (60000 / self.0.get()) / 4;
        ms as u64
    }

    pub fn beats_per_minute(self) -> usize {
        self.0.get()
    }
}

impl Opts {
    pub fn new() -> Self {
        Self {
            bpm: Bpm::new(20).expect("default tempo is positive"),
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
    use super::{Bpm, CursorBloomRadius, DEFAULT_CURSOR_BLOOM_RADIUS, Opts, SectorSeamSpacing};

    #[test]
    fn bpm_accepts_only_positive_tick_rates() {
        assert_eq!(Bpm::new(20).map(|bpm| bpm.delay_ms()), Some(750));
        assert_eq!(Bpm::new(0), None);
        assert_eq!(Bpm::new(15_001), None);
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
