use std::time::Duration;
use tokio::time::Instant;

use crate::coord::Coord;

#[derive(Debug)]
pub struct Cursor {
    pub coord: Coord,
    pub on: bool,

    at: Instant,
    delay_ms: u64,
}

impl Cursor {
    pub fn new(cols: usize, rows: usize, delay: u64) -> Self {
        Self {
            coord: Coord::new(0, 0, cols, rows),
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

    #[inline]
    pub fn up(&mut self) {
        self.select(self.coord.up());
    }

    #[inline]
    pub fn down(&mut self) {
        self.select(self.coord.down());
    }

    #[inline]
    pub fn left(&mut self) {
        self.select(self.coord.left());
    }

    #[inline]
    pub fn right(&mut self) {
        self.select(self.coord.right());
    }
}
