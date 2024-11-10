use std::time::Duration;
use tokio::time::Instant;

use crate::coord::Coord;

#[derive(Debug)]
pub struct Cursor {
    pub coord: Coord,
    pub on: bool,

    cols: usize,
    rows: usize,

    at: Instant,
    delay_ms: u64,
}

impl Cursor {
    pub fn new(cols: usize, rows: usize, delay: u64) -> Self {
        Self {
            coord: Coord::new(0, 0),
            cols,
            rows,
            at: Instant::now(),
            on: false,
            delay_ms: delay,
        }
    }

    pub fn blink(&mut self) {
        if self.at.elapsed() >= Duration::from_millis(self.delay_ms) {
            self.at = Instant::now();
            self.on = !self.on;
        }
    }

    #[inline]
    pub fn is_at(&self, x: usize, y: usize) -> bool {
        self.coord.is_at(x, y)
    }

    #[inline]
    pub fn select(&mut self, selected: Coord) {
        self.coord = selected;
        self.on = false;
        self.at = Instant::now();
    }

    #[inline]
    pub fn select_at(&mut self, x: usize, y: usize) {
        self.select(self.coord.at(x, y));
    }

    pub fn up(&mut self) {
        let y = match self.coord.y {
            0 | 1 => 0,
            _ => self.coord.y - 1,
        };

        let coord = self.coord.from_y(y);
        self.select(coord);
    }

    pub fn down(&mut self) {
        let y = (self.coord.y + 1).clamp(0, self.rows - 1);

        let coord = self.coord.from_y(y);
        self.select(coord);
    }

    pub fn left(&mut self) {
        let x = match self.coord.x {
            0 | 1 => 0,
            _ => self.coord.x - 1,
        };

        let coord = self.coord.from_x(x);
        self.select(coord);
    }

    pub fn right(&mut self) {
        let x = (self.coord.x + 1).clamp(0, self.cols - 1);
        let coord = self.coord.from_x(x);
        self.select(coord);
    }
}

#[cfg(test)]
mod test {
    use crate::{cursor::Cursor, test::trace};

    #[test]
    fn test_coord() {
        trace();
        let mut cursor = Cursor::new(10, 10, 1000);

        cursor.down();
        assert_eq!(cursor.coord.y, 1);

        cursor.left();
        assert_eq!(cursor.coord.x, 0);

        cursor.right();
        cursor.right();
        cursor.right();
        cursor.right();
        assert_eq!(cursor.coord.x, 4);

        cursor.up();
        assert_eq!(cursor.coord.y, 0);

        cursor.right();
        assert_eq!(cursor.coord.x, 5);

        cursor.up();
        cursor.up();
        cursor.up();
        cursor.up();
        assert_eq!(cursor.coord.y, 0);
    }
}
