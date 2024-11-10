use arrayvec::ArrayVec;

use crate::Atom;

#[derive(Clone, Debug)]
pub struct Portal {
    pub atom: Atom,
    pub ports: ArrayVec<Coord, 16>,
}

#[derive(Clone, Debug)]
pub struct Coord(pub usize, pub usize);

impl Portal {
    pub fn new(atom: Atom) -> Self {
        let mut ports = ArrayVec::new();

        ports.push(Coord::default());

        Self { atom, ports }
    }
}

impl Default for Portal {
    fn default() -> Self {
        let ports: ArrayVec<Coord, 16> = ArrayVec::new();

        Self {
            atom: Atom::Empty,
            ports,
        }
    }
}

impl Default for Coord {
    fn default() -> Self {
        Self(0, 1)
    }
}
