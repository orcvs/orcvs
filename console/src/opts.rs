use egui::FontId;
use std::num::NonZeroUsize;

pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_MARKER_SPACING: usize = 8;

pub const DEFAULT_HIGHLIGHT_DOT_SPACING: usize = 2;

pub const DEFAULT_CURSOR_DELAY: u64 = 800;

///
/// How the console presents and plays a Source. Nothing in this file is a
/// Source dimension: column and row counts belong to the Grid, which is the
/// only thing that states them. `marker_spacing` counts the Cells between
/// visual markers; it is not a Source dimension.
///
#[derive(Clone, Debug)]
pub struct Opts {
    pub bpm: Bpm,
    pub cursor_delay: u64,
    pub font_id: FontId,
    pub highlight_dot_spacing: usize,
    pub marker_spacing: MarkerSpacing,
    pub mode: Mode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Insert,
    Command,
}

#[derive(Clone, Debug)]
pub struct Bpm(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkerSpacing(NonZeroUsize);

impl MarkerSpacing {
    pub fn new(cells: usize) -> Option<Self> {
        NonZeroUsize::new(cells).map(Self)
    }

    pub fn cells(self) -> usize {
        self.0.get()
    }
}

impl Bpm {
    pub fn delay_ms(&self) -> u64 {
        let ms = (60000 / self.0) / 4;
        ms as u64
    }
}

impl Opts {
    pub fn new() -> Self {
        Self {
            bpm: Bpm(20),
            cursor_delay: DEFAULT_CURSOR_DELAY,
            font_id: egui::FontId::monospace(DEFAULT_FONT_SIZE),
            highlight_dot_spacing: DEFAULT_HIGHLIGHT_DOT_SPACING,
            marker_spacing: MarkerSpacing::new(DEFAULT_MARKER_SPACING)
                .expect("default marker spacing is positive"),
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
    use super::MarkerSpacing;

    #[test]
    fn marker_spacing_accepts_only_whole_positive_cell_counts() {
        assert_eq!(MarkerSpacing::new(1).map(MarkerSpacing::cells), Some(1));
        assert_eq!(MarkerSpacing::new(8).map(MarkerSpacing::cells), Some(8));
        assert_eq!(MarkerSpacing::new(0), None);
    }
}
