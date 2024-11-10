use egui::FontId;

pub const DEFAULT_FONT_SIZE: f32 = 20.0;
pub const DEFAULT_GRID_SIZE: f32 = 8.0;

pub const DEFAULT_COL_COUNT: usize = 2 * (DEFAULT_GRID_SIZE as usize);
pub const DEFAULT_ROW_COUNT: usize = 2 * (DEFAULT_GRID_SIZE as usize);

pub const DEFAULT_GRID_SELECTED_DOT_SPACING: usize = 2;

pub const DEFAULT_CURSOR_DELAY: u64 = 800;

#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub struct Opts {
    pub bpm: Bpm,
    pub cols: usize,
    pub cursor_delay: u64,
    pub font_id: FontId,
    pub grid_selected_dot_spacing: usize,
    pub grid_size: f32,
    pub mode: Mode,
    pub rows: usize,
}

#[derive(PartialEq)]
#[cfg_attr(feature = "persistence", derive(Serialize, Deserialize))]
pub enum Mode {
    Insert,
    Command,
}

pub struct Bpm(usize);

impl Bpm {
    pub fn delay_ms(&self) -> u64 {
        let ms = (60000 / self.0) / 4;
        ms as u64
    }
}

impl Opts {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            bpm: Bpm(20),
            cols,
            cursor_delay: DEFAULT_CURSOR_DELAY,
            font_id: egui::FontId::monospace(DEFAULT_FONT_SIZE),
            grid_selected_dot_spacing: DEFAULT_GRID_SELECTED_DOT_SPACING,
            grid_size: DEFAULT_GRID_SIZE,
            mode: Mode::Insert,
            rows,
        }
    }

    pub fn count(&self) -> usize {
        self.rows * self.cols
    }
}
