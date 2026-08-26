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

    ///
    /// Convert a linear index into a Coord in a Source `cols` wide
    /// panics if `cols` is zero
    ///
    pub fn from_index(index: usize, cols: usize) -> Self {
        assert!(cols != 0, "cols must be non-zero to convert index {index}");

        let x = index % cols;
        let y = index / cols;
        Coord { x, y }
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

        x > min_x && (x) <= max_x && y > min_y && (y) <= max_y
    }

    pub fn from_x(self, x: usize) -> Self {
        Self { x, y: self.y }
    }

    pub fn from_y(self, y: usize) -> Self {
        Self { x: self.x, y }
    }

    ///
    /// Convert this Coord into a linear index in a Source `cols` wide
    ///
    pub fn index(&self, cols: usize) -> usize {
        self.y * cols + self.x
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
    fn test_from_index() {
        trace();

        let cols = 10;

        let expected = Coord::new(0, 0);
        let coord = Coord::from_index(0, cols);

        assert_eq!(coord, expected);

        let expected = Coord::new(4, 0);
        let coord = Coord::from_index(expected.index(cols), cols);

        assert_eq!(coord, expected);

        let expected = Coord::new(4, 4);
        let coord = Coord::from_index(expected.index(cols), cols);

        assert_eq!(coord, expected);
    }

    #[test]
    #[should_panic(expected = "cols must be non-zero")]
    fn test_from_index_zero_cols_panics() {
        trace();

        let _ = Coord::from_index(0, 0);
    }

    #[test]
    fn test_index_round_trip() {
        trace();

        let cols = 10;

        for index in 0..(cols * 4) {
            assert_eq!(Coord::from_index(index, cols).index(cols), index);
        }
    }

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
