use egui::FontId;

pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_MARKER_SPACING: f32 = 8.0;

pub const DEFAULT_HIGHLIGHT_DOT_SPACING: usize = 2;

pub const DEFAULT_CURSOR_DELAY: u64 = 800;

///
/// How the console presents and plays a Source. Nothing in this file is a
/// Source dimension: column and row counts belong to the Grid, which is the
/// only thing that states them. `marker_spacing` is a visual rhythm drawn
/// over the Cells, not a count of them.
///
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Opts {
    pub bpm: Bpm,
    pub cursor_delay: u64,
    pub font_id: FontId,
    pub highlight_dot_spacing: usize,
    pub marker_spacing: f32,
    pub mode: Mode,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum Mode {
    Insert,
    Command,
}

#[derive(Clone, Debug)]
pub struct Bpm(usize);

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
            marker_spacing: DEFAULT_MARKER_SPACING,
            mode: Mode::Insert,
        }
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self::new()
    }
}
