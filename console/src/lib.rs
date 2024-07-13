#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod source;
mod style;

pub use app::ConsoleApp;
use tracing::info;

// pub fn distance(Coord(x1, y1): Coord, Coord(x2, y2): Coord) -> f64 {
//     let x = (x2 as f64 - x1 as f64) as f64;
//     let y = (y2 as f64 - y1 as f64) as f64;

//     let d = y.hypot(x);
//     d.floor()
// }

#[derive(Clone, Copy, PartialEq)]
pub struct Coord<const X: usize, const Y: usize> {
    x: usize,
    y: usize,
}

impl<const X: usize, const Y: usize> Coord<X, Y> {
    fn from(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn from_x(self, x: usize) -> Self {
        Self { x, y: self.y }
    }

    fn from_y(self, y: usize) -> Self {
        Self { x: self.x, y }
    }

    fn up(&self) -> Self {
        let y = match self.y {
            0 | 1 => 0,
            _ => self.y - 1,
        };
        self.from_y(y)
    }

    fn down(&self) -> Self {
        let y = (self.y + 1).clamp(0, Y - 1);
        self.from_y(y)
    }

    fn left(&self) -> Self {
        let x = match self.x {
            0 | 1 => 0,
            _ => self.x - 1,
        };
        self.from_x(x)
    }

    fn right(&self) -> Self {
        let x = (self.x + 1).clamp(0, X - 1);
        self.from_x(x)
    }

    pub fn in_grid(self, x: usize, y: usize, grid_size: f32) -> bool {
        assert!(grid_size != 0.0);

        let min_x = (self.x as f32 / grid_size).floor() * grid_size;
        let max_x = ((self.x as f32 / grid_size).ceil() * grid_size).max(grid_size);

        let min_y = (self.y as f32 / grid_size).floor() * grid_size;
        let max_y = ((self.y as f32 / grid_size).ceil() * grid_size).max(grid_size);

        x as f32 >= min_x && x as f32 <= max_x && y as f32 >= min_y && y as f32 <= max_y
    }
}

#[cfg(test)]
mod test {
    use std::sync::Once;

    use tracing::info;

    use crate::Coord;

    #[test]
    fn test_in_grid() {
        trace();
        let grid_size = 8.0;

        let selected = Coord::<100, 100>::from(0, 0);

        for x in 0..=grid_size as usize {
            for y in 0..=grid_size as usize {
                assert!(selected.in_grid(x, y, grid_size));
            }
        }

        let selected = Coord::<100, 100>::from(5, 5);

        for x in 0..=grid_size as usize {
            for y in 0..=grid_size as usize {
                assert!(selected.in_grid(x, y, grid_size));
            }
        }

        // Grid X 5 Y 6
        let selected = Coord::<100, 100>::from(42, 51);

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
        let coord = Coord::<10, 10>::from(3, 3);

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

    #[test]
    fn test_distance() {
        trace();

        // let a = Coord(1, 1);
        // let b = Coord(1, 2);
        // let d = distance(a, b);

        // assert_eq!(d, 1.0);

        // let a = Coord(1, 1);
        // let b = Coord(3, 3);
        // let d = distance(a, b);

        // assert_eq!(d, 2.0);

        // let a = Coord(1, 1);
        // let b = Coord(7, 5);
        // let d = distance(a, b);

        // assert_eq!(d, 7.0);
    }

    #[allow(dead_code)]
    static INIT: Once = Once::new();

    #[allow(dead_code)]
    pub fn trace() {
        INIT.call_once(|| {
            use tracing_subscriber::FmtSubscriber;

            let subscriber = FmtSubscriber::builder()
                .with_max_level(tracing::Level::DEBUG) // Set the maximum level of tracing events that should be logged.
                .with_line_number(true)
                .with_target(true)
                .finish();

            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");
        });
    }
}
