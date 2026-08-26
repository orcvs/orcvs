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
}
