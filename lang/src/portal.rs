use arrayvec::ArrayVec;

use crate::Atom;

#[derive(Clone, Debug)]
pub struct Portal {
    pub atom: Atom,
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug)]
pub struct Coord(pub usize, pub usize);

impl Portal {
    pub fn new(atom: Atom, x: usize, y: usize) -> Self {
        Self { atom, x, y }
    }
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            atom: Atom::Empty,
            x: 0,
            y: 1,
        }
    }
}
