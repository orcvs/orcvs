use std::time::Duration;
use web_time::Instant;

use crate::grid::Position;

#[derive(Debug)]
pub struct Cursor {
    pub on: bool,

    position: Position,

    at: Instant,
    delay_ms: u64,
}

impl Cursor {
    pub fn new(position: Position, delay: u64) -> Self {
        Self {
            position,
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
    pub fn position(&self) -> Position {
        self.position
    }

    #[inline]
    pub fn is_at(&self, position: Position) -> bool {
        self.position == position
    }

    #[inline]
    pub fn select(&mut self, selected: Position) {
        self.position = selected;
        self.on = false;
        self.at = Instant::now();
    }
}

#[cfg(test)]
mod test {
    use crate::{cursor::Cursor, grid::Grid, test::trace};

    #[test]
    fn test_cursor_starts_where_it_is_placed() {
        trace();

        let grid = Grid::new(10, 4);
        let at = |x, y| grid.position(x, y).expect("inside the grid");
        let cursor = Cursor::new(grid.origin(), 1000);

        assert_eq!(cursor.position(), grid.origin());
        assert!(cursor.is_at(grid.origin()));
        assert!(!cursor.is_at(at(1, 0)));
        // transposed: (0, 1) is a different Cell from (1, 0)
        assert!(!cursor.is_at(at(0, 1)));
    }

    #[test]
    fn test_cursor_follows_the_positions_the_grid_yields() {
        trace();

        let grid = Grid::new(10, 4);
        let mut cursor = Cursor::new(grid.origin(), 1000);

        cursor.select(grid.down(cursor.position()));
        assert_eq!(cursor.position().y(), 1);

        cursor.select(grid.left(cursor.position()));
        assert_eq!(cursor.position().x(), 0);

        for _ in 0..4 {
            cursor.select(grid.right(cursor.position()));
        }
        assert_eq!(cursor.position().x(), 4);

        cursor.select(grid.up(cursor.position()));
        assert_eq!(cursor.position().y(), 0);

        cursor.select(grid.right(cursor.position()));
        assert_eq!(cursor.position().x(), 5);

        for _ in 0..4 {
            cursor.select(grid.up(cursor.position()));
        }
        assert_eq!(cursor.position().y(), 0);
    }
}
