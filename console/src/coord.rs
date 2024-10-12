#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
    max_x: usize,
    max_y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize, max_x: usize, max_y: usize) -> Self {
        Self { x, y, max_x, max_y }
    }

    pub fn is_at(&self, x: usize, y: usize) -> bool {
        self.x == x && self.y == y
    }

    pub fn at(&self, x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            max_x: self.max_x,
            max_y: self.max_y,
        }
    }

    pub fn up(&self) -> Self {
        let y = match self.y {
            0 | 1 => 0,
            _ => self.y - 1,
        };
        self.from_y(y)
    }

    pub fn down(&self) -> Self {
        let y = (self.y + 1).clamp(0, self.max_y - 1);
        self.from_y(y)
    }

    pub fn left(&self) -> Self {
        let x = match self.x {
            0 | 1 => 0,
            _ => self.x - 1,
        };
        self.from_x(x)
    }

    pub fn right(&self) -> Self {
        let x = (self.x + 1).clamp(0, self.max_x - 1);
        self.from_x(x)
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

    ///
    /// Convert x, y coordinates to a linear index
    /// panic if the index is out of bounds
    ///
    pub fn index(&self) -> usize {
        let idx = self.y * self.max_x + self.x;
        assert!(
            idx <= self.max_x * self.max_y,
            "index {idx} out of bounds for [{},{}]",
            self.x,
            self.y,
        );
        idx
    }

    fn from_x(self, x: usize) -> Self {
        Self {
            x,
            y: self.y,
            max_x: self.max_x,
            max_y: self.max_y,
        }
    }

    fn from_y(self, y: usize) -> Self {
        Self {
            x: self.x,
            y,
            max_x: self.max_x,
            max_y: self.max_y,
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{coord::Coord, test::trace};

    #[test]
    fn test_to_idx() {
        trace();

        let idx = Coord::new(0, 0, 10, 10).index();

        assert_eq!(idx, 0);

        let idx = Coord::new(5, 5, 10, 10).index();
        assert_eq!(idx, 55);
    }

    #[test]
    #[should_panic(expected = "index 121 out of bounds for [11,11]")]
    fn test_to_idx_out_of_bounds() {
        let _idx = Coord::new(11, 11, 10, 10).index();
    }

    #[test]
    fn test_in_grid() {
        trace();
        let grid_size = 8.0;

        let selected = Coord::new(5, 5, 100, 100);
        assert!(!selected.in_grid(10, 10, grid_size));
        for x in 0..grid_size as usize {
            for y in 0..grid_size as usize {
                assert!(selected.in_grid(x, y, grid_size));
            }
        }

        let selected = Coord::new(8, 8, 100, 100);
        assert!(!selected.in_grid(1, 1, grid_size));

        for x in 8..=16 as usize {
            for y in 8..=16 as usize {
                assert!(selected.in_grid(x, y, grid_size));
            }
        }

        // Grid X 5 Y 6
        let selected = Coord::new(42, 51, 100, 100);
        assert!(!selected.in_grid(1, 1, grid_size));
        for x in 0..=grid_size as usize {
            for y in 0..=grid_size as usize {
                let x = x + 40;
                let y = y + 48;
                assert!(selected.in_grid(x, y, grid_size));
            }
        }
    }

    #[test]
    fn test_coord() {
        let coord = Coord::new(3, 3, 10, 10);

        let coord = coord.down();
        assert_eq!(coord.y, 4);

        let coord = coord.left();
        assert_eq!(coord.x, 2);

        let coord = coord.left();
        let coord = coord.left();
        let coord = coord.left();
        let coord = coord.left();
        assert_eq!(coord.x, 0);

        let coord = coord.up();
        assert_eq!(coord.y, 3);

        let coord = coord.right();
        assert_eq!(coord.x, 1);

        let coord = coord.up();
        let coord = coord.up();
        let coord = coord.up();
        let coord = coord.up();
        assert_eq!(coord.y, 0);
    }
}
