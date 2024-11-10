use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn is_at(&self, x: usize, y: usize) -> bool {
        self.x == x && self.y == y
    }

    pub fn at(&self, x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn in_grid(self, x: usize, y: usize, grid_size: f32) -> bool {
        assert!(grid_size != 0.0);

        let x = x as i32;
        let y = y as i32;
        let cursor_x = self.x as i32;
        let cursor_y = self.y as i32;
        let grid_size = grid_size as i32;

        let min_x = ((cursor_x / grid_size) * grid_size) - 1;
        let max_x = (1 + (cursor_x / grid_size)) * grid_size;
        let min_y = ((cursor_y / grid_size) * grid_size) - 1;
        let max_y = (1 + (cursor_y / grid_size)) * grid_size;

        // info!("{min_x}/{max_x}");
        // info!("{min_y}/{max_y}");

        x > min_x && (x) <= max_x && y > min_y && (y) <= max_y
    }

    pub fn from_x(self, x: usize) -> Self {
        Self { x, y: self.y }
    }

    pub fn from_y(self, y: usize) -> Self {
        Self { x: self.x, y }
    }

    pub fn index(&self, max_x: usize) -> usize {
        self.y * max_x + self.x
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x {} y {}", self.x, self.y)
    }
}

#[cfg(test)]
mod test {

    use crate::{coord::Coord, test::trace};

    #[test]
    fn test_in_grid() {
        trace();
        let grid_size = 8.0;

        let selected = Coord::new(5, 5);
        assert!(!selected.in_grid(10, 10, grid_size));
        for x in 0..grid_size as usize {
            for y in 0..grid_size as usize {
                assert!(selected.in_grid(x, y, grid_size));
            }
        }

        let selected = Coord::new(8, 8);
        assert!(!selected.in_grid(1, 1, grid_size));

        for x in 8..=16 as usize {
            for y in 8..=16 as usize {
                assert!(selected.in_grid(x, y, grid_size));
            }
        }

        // Grid X 5 Y 6
        let selected = Coord::new(42, 51);
        assert!(!selected.in_grid(1, 1, grid_size));
        for x in 0..=grid_size as usize {
            for y in 0..=grid_size as usize {
                let x = x + 40;
                let y = y + 48;
                assert!(selected.in_grid(x, y, grid_size));
            }
        }
    }
}
