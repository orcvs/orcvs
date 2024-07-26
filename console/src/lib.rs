#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod source;
mod style;

pub use app::ConsoleApp;
use egui::Color32;
use tracing::info;

pub struct Color(Color32);

impl Color {
    const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(Color32::from_rgba_premultiplied(r, g, b, 255))
    }

    const fn with_alpha(self, a: u8) -> Self {
        Self(Color32::from_rgba_premultiplied(
            self.0.r(),
            self.0.g(),
            self.0.b(),
            a,
        ))
    }

    const fn build(self) -> Color32 {
        self.0
    }
}

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
}

#[cfg(test)]
mod test {

    use std::sync::Once;

    use tracing::info;
    use tracing_subscriber::EnvFilter;

    use crate::Coord;

    #[allow(dead_code)]
    static INIT: Once = Once::new();

    #[allow(dead_code)]
    pub fn trace() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_env_filter("debug")
                // .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .pretty()
                .init();
        });
    }

    #[test]
    fn test_in_grid() {
        trace();
        let grid_size = 8.0;

        let selected = Coord::<100, 100>::from(5, 5);
        assert!(!selected.in_grid(10, 10, grid_size));
        for x in 0..grid_size as usize {
            for y in 0..grid_size as usize {
                assert!(selected.in_grid(x, y, grid_size));
            }
        }

        let selected = Coord::<100, 100>::from(8, 8);
        assert!(!selected.in_grid(1, 1, grid_size));

        for x in 8..=16 as usize {
            for y in 8..=16 as usize {
                assert!(selected.in_grid(x, y, grid_size));
            }
        }

        // Grid X 5 Y 6
        let selected = Coord::<100, 100>::from(42, 51);
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
}
